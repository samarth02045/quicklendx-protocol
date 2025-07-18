use soroban_sdk::{Address, Env, String};
use crate::errors::QuickLendXError;

pub fn verify_invoice_data(
    env: &Env,
    _business: &Address,
    amount: i128,
    _currency: &Address,
    due_date: u64,
    description: &String,
) -> Result<(), QuickLendXError> {
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
    Ok(())
} 