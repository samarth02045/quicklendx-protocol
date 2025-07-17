#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, vec, Address, BytesN, Env, String, Vec, symbol_short,
};

mod invoice;
mod errors;

use invoice::{Invoice, InvoiceStatus, InvoiceStorage};
use errors::QuickLendXError;

#[contract]
pub struct QuickLendXContract;

#[contractimpl]
impl QuickLendXContract {
    /// Store an invoice in the contract
    pub fn store_invoice(
        env: Env,
        business: Address,
        amount: i128,
        currency: Address,
        due_date: u64,
        description: String,
    ) -> Result<BytesN<32>, QuickLendXError> {
        // Validate input parameters
        if amount <= 0 {
            return Err(QuickLendXError::InvalidAmount);
        }

        let current_timestamp = env.ledger().timestamp();
        if due_date <= current_timestamp {
            return Err(QuickLendXError::InvoiceDueDateInvalid);
        }

        if description.len() == 0 {
            return Err(QuickLendXError::InvalidDescription);
        }

        // Create new invoice
        let invoice = Invoice::new(
            &env,
            business,
            amount,
            currency,
            due_date,
            description,
        );

        // Store the invoice
        InvoiceStorage::store_invoice(&env, &invoice);

        // Emit event
        env.events().publish(
            (symbol_short!("invoice_created"),),
            (invoice.id.clone(), business, amount, currency, due_date),
        );

        Ok(invoice.id)
    }

    /// Get an invoice by ID
    pub fn get_invoice(env: Env, invoice_id: BytesN<32>) -> Result<Invoice, QuickLendXError> {
        InvoiceStorage::get_invoice(&env, &invoice_id)
            .ok_or(QuickLendXError::InvoiceNotFound)
    }

    /// Get all invoices for a specific business
    pub fn get_business_invoices(env: Env, business: Address) -> Vec<BytesN<32>> {
        InvoiceStorage::get_business_invoices(&env, &business)
    }

    /// Get all invoices by status
    pub fn get_invoices_by_status(env: Env, status: InvoiceStatus) -> Vec<BytesN<32>> {
        InvoiceStorage::get_invoices_by_status(&env, &status)
    }

    /// Get all available invoices (verified and not funded)
    pub fn get_available_invoices(env: Env) -> Vec<BytesN<32>> {
        InvoiceStorage::get_invoices_by_status(&env, &InvoiceStatus::Verified)
    }

    /// Update invoice status (admin function)
    pub fn update_invoice_status(
        env: Env,
        invoice_id: BytesN<32>,
        new_status: InvoiceStatus,
    ) -> Result<(), QuickLendXError> {
        let mut invoice = InvoiceStorage::get_invoice(&env, &invoice_id)
            .ok_or(QuickLendXError::InvoiceNotFound)?;

        // Remove from old status list
        InvoiceStorage::remove_from_status_invoices(&env, &invoice.status, &invoice_id);

        // Update status
        match new_status {
            InvoiceStatus::Verified => invoice.verify(),
            InvoiceStatus::Paid => invoice.mark_as_paid(env.ledger().timestamp()),
            InvoiceStatus::Defaulted => invoice.mark_as_defaulted(),
            _ => return Err(QuickLendXError::InvalidStatus),
        }

        // Store updated invoice
        InvoiceStorage::update_invoice(&env, &invoice);

        // Add to new status list
        InvoiceStorage::add_to_status_invoices(&env, &invoice.status, &invoice_id);

        // Emit event
        env.events().publish(
            (symbol_short!("invoice_status_updated"),),
            (invoice_id, new_status),
        );

        Ok(())
    }

    /// Get invoice count by status
    pub fn get_invoice_count_by_status(env: Env, status: InvoiceStatus) -> u32 {
        let invoices = InvoiceStorage::get_invoices_by_status(&env, &status);
        invoices.len() as u32
    }

    /// Get total invoice count
    pub fn get_total_invoice_count(env: Env) -> u32 {
        let pending = Self::get_invoice_count_by_status(env.clone(), InvoiceStatus::Pending);
        let verified = Self::get_invoice_count_by_status(env.clone(), InvoiceStatus::Verified);
        let funded = Self::get_invoice_count_by_status(env.clone(), InvoiceStatus::Funded);
        let paid = Self::get_invoice_count_by_status(env.clone(), InvoiceStatus::Paid);
        let defaulted = Self::get_invoice_count_by_status(env, InvoiceStatus::Defaulted);
        
        pending + verified + funded + paid + defaulted
    }
}

#[cfg(test)]
mod test; 