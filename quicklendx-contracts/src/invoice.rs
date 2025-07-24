use soroban_sdk::{
    contracttype, Address, BytesN, Env, Map, String, Vec, symbol_short, vec,
};

/// Invoice status enumeration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvoiceStatus {
    Pending,    // Invoice uploaded, awaiting verification
    Verified,   // Invoice verified and available for bidding
    Funded,     // Invoice has been funded by an investor
    Paid,       // Invoice has been paid and settled
    Defaulted,  // Invoice payment is overdue/defaulted
}

/// Core invoice data structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invoice {
    pub id: BytesN<32>,                    // Unique invoice identifier
    pub business: Address,                  // Business that uploaded the invoice
    pub amount: i128,                       // Total invoice amount
    pub currency: Address,                  // Currency token address (XLM = Address::random())
    pub due_date: u64,                      // Due date timestamp
    pub status: InvoiceStatus,              // Current status of the invoice
    pub created_at: u64,                    // Creation timestamp
    pub description: String,                // Invoice description/metadata
    pub funded_amount: i128,                // Amount funded by investors
    pub funded_at: Option<u64>,             // When the invoice was funded
    pub investor: Option<Address>,          // Address of the investor who funded
    pub settled_at: Option<u64>,            // When the invoice was settled
}

impl Invoice {
    /// Create a new invoice
    pub fn new(
        env: &Env,
        business: Address,
        amount: i128,
        currency: Address,
        due_date: u64,
        description: String,
    ) -> Self {
        let id = Self::generate_unique_invoice_id(env);
        let created_at = env.ledger().timestamp();
        
        Self {
            id,
            business,
            amount,
            currency,
            due_date,
            status: InvoiceStatus::Pending,
            created_at,
            description,
            funded_amount: 0,
            funded_at: None,
            investor: None,
            settled_at: None,
        }
    }

    /// Check if invoice is available for funding
    pub fn is_available_for_funding(&self) -> bool {
        self.status == InvoiceStatus::Verified && self.funded_amount == 0
    }

    /// Check if invoice is overdue
    pub fn is_overdue(&self, current_timestamp: u64) -> bool {
        current_timestamp > self.due_date
    }

    /// Mark invoice as funded
    pub fn mark_as_funded(&mut self, investor: Address, funded_amount: i128, timestamp: u64) {
        self.status = InvoiceStatus::Funded;
        self.funded_amount = funded_amount;
        self.funded_at = Some(timestamp);
        self.investor = Some(investor);
    }

    /// Mark invoice as paid
    pub fn mark_as_paid(&mut self, timestamp: u64) {
        self.status = InvoiceStatus::Paid;
        self.settled_at = Some(timestamp);
    }

    /// Mark invoice as defaulted
    pub fn mark_as_defaulted(&mut self) {
        self.status = InvoiceStatus::Defaulted;
    }

    /// Verify the invoice
    pub fn verify(&mut self) {
        self.status = InvoiceStatus::Verified;
    }

    /// Generate a unique invoice ID
    fn generate_unique_invoice_id(env: &Env) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let counter_key = symbol_short!("inv_cnt");
        let counter = env.storage().instance().get(&counter_key).unwrap_or(0u64);
        env.storage().instance().set(&counter_key, &(counter + 1));
        
        let mut id_bytes = [0u8; 32];
        // Embed timestamp in first 8 bytes
        id_bytes[0..8].copy_from_slice(&timestamp.to_be_bytes());
        // Embed counter in next 8 bytes
        id_bytes[8..16].copy_from_slice(&counter.to_be_bytes());
        // Fill remaining bytes with a pattern to ensure uniqueness
        for i in 16..32 {
            id_bytes[i] = ((timestamp + counter as u64) % 256) as u8;
        }
        
        BytesN::from_array(env, &id_bytes)
    }
}

/// Storage keys for invoice data
pub struct InvoiceStorage;

impl InvoiceStorage {
    /// Get storage key for an invoice
    pub fn get_invoice_key(invoice_id: &BytesN<32>) -> String {
        String::from_str(&soroban_sdk::Env::default(), "invoice")
    }

    /// Get storage key for business invoices
    pub fn get_business_invoices_key(business: &Address) -> String {
        String::from_str(&soroban_sdk::Env::default(), "business_invoices")
    }

    /// Get storage key for invoices by status
    pub fn get_status_invoices_key(status: &InvoiceStatus) -> String {
        let status_str = match status {
            InvoiceStatus::Pending => "pending",
            InvoiceStatus::Verified => "verified",
            InvoiceStatus::Funded => "funded",
            InvoiceStatus::Paid => "paid",
            InvoiceStatus::Defaulted => "defaulted",
        };
        String::from_str(&soroban_sdk::Env::default(), "status_invoices")
    }

    /// Store an invoice
    pub fn store_invoice(env: &Env, invoice: &Invoice) {
        env.storage().instance().set(&invoice.id, invoice);
        
        // Add to business invoices list
        Self::add_to_business_invoices(env, &invoice.business, &invoice.id);
        
        // Add to status invoices list
        Self::add_to_status_invoices(env, &invoice.status, &invoice.id);
    }

    /// Get an invoice by ID
    pub fn get_invoice(env: &Env, invoice_id: &BytesN<32>) -> Option<Invoice> {
        env.storage().instance().get(invoice_id)
    }

    /// Update an invoice
    pub fn update_invoice(env: &Env, invoice: &Invoice) {
        env.storage().instance().set(&invoice.id, invoice);
    }

    /// Get all invoices for a business
    pub fn get_business_invoices(env: &Env, business: &Address) -> Vec<BytesN<32>> {
        let key = (symbol_short!("business"), business);
        env.storage().instance().get(&key).unwrap_or_else(|| Vec::new(env))
    }

    /// Get all invoices by status
    pub fn get_invoices_by_status(env: &Env, status: &InvoiceStatus) -> Vec<BytesN<32>> {
        let key = match status {
            InvoiceStatus::Pending => symbol_short!("pending"),
            InvoiceStatus::Verified => symbol_short!("verified"),
            InvoiceStatus::Funded => symbol_short!("funded"),
            InvoiceStatus::Paid => symbol_short!("paid"),
            InvoiceStatus::Defaulted => symbol_short!("default"),
        };
        env.storage().instance().get(&key).unwrap_or_else(|| Vec::new(env))
    }

    /// Add invoice to business invoices list
    fn add_to_business_invoices(env: &Env, business: &Address, invoice_id: &BytesN<32>) {
        let key = (symbol_short!("business"), business);
        let mut invoices = Self::get_business_invoices(env, business);
        invoices.push_back(invoice_id.clone());
        env.storage().instance().set(&key, &invoices);
    }

    /// Add invoice to status invoices list
    pub fn add_to_status_invoices(env: &Env, status: &InvoiceStatus, invoice_id: &BytesN<32>) {
        let key = match status {
            InvoiceStatus::Pending => symbol_short!("pending"),
            InvoiceStatus::Verified => symbol_short!("verified"),
            InvoiceStatus::Funded => symbol_short!("funded"),
            InvoiceStatus::Paid => symbol_short!("paid"),
            InvoiceStatus::Defaulted => symbol_short!("default"),
        };
        let mut invoices = env.storage().instance().get(&key).unwrap_or_else(|| Vec::new(env));
        invoices.push_back(invoice_id.clone());
        env.storage().instance().set(&key, &invoices);
    }



    /// Remove invoice from status invoices list
    pub fn remove_from_status_invoices(env: &Env, status: &InvoiceStatus, invoice_id: &BytesN<32>) {
        let key = match status {
            InvoiceStatus::Pending => symbol_short!("pending"),
            InvoiceStatus::Verified => symbol_short!("verified"),
            InvoiceStatus::Funded => symbol_short!("funded"),
            InvoiceStatus::Paid => symbol_short!("paid"),
            InvoiceStatus::Defaulted => symbol_short!("default"),
        };
        let mut invoices = Self::get_invoices_by_status(env, status);
        
        // Find and remove the invoice ID
        let mut new_invoices = Vec::new(env);
        for id in invoices.iter() {
            if id != *invoice_id {
                new_invoices.push_back(id);
            }
        }
        
        env.storage().instance().set(&key, &new_invoices);
    }
}