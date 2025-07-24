use soroban_sdk::{contracttype, Address, BytesN, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvestmentStatus {
    Active,
    Withdrawn,
    Completed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Investment {
    pub investment_id: BytesN<32>,
    pub invoice_id: BytesN<32>,
    pub investor: Address,
    pub amount: i128,
    pub funded_at: u64,
    pub status: InvestmentStatus,
}

pub struct InvestmentStorage;

impl InvestmentStorage {
    pub fn store_investment(env: &Env, investment: &Investment) {
        env.storage().instance().set(&investment.investment_id, investment);
    }
    pub fn get_investment(env: &Env, investment_id: &BytesN<32>) -> Option<Investment> {
        env.storage().instance().get(investment_id)
    }
    pub fn update_investment(env: &Env, investment: &Investment) {
        env.storage().instance().set(&investment.investment_id, investment);
    }

    /// Generates a unique 32-byte investment ID using a sequential counter stored in contract storage.
    /// The counter is incremented for each new investment, and its value is stored in the last 16 bytes
    /// of the ID (big-endian, zero-padded). This guarantees uniqueness and is efficient.
    /// Note: If randomness becomes available in the SDK, consider switching to a random approach.
    pub fn generate_unique_investment_id(env: &Env) -> BytesN<32> {
        let counter_key = "unique_investment_counter";
        let mut counter: u128 = env.storage().instance().get(&counter_key).unwrap_or(0u128);
        counter += 1;
        env.storage().instance().set(&counter_key, &counter);
        // The first 16 bytes are zeros, the last 16 bytes are the counter (big-endian)
        let mut bytes = [0u8; 32];
        bytes[16..].copy_from_slice(&counter.to_be_bytes());
        BytesN::from_array(env, &bytes)
    }
} 