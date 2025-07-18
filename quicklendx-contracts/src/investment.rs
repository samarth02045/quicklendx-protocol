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
} 