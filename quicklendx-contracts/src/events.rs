use soroban_sdk::{Env, symbol_short};
use crate::invoice::Invoice;

pub fn emit_invoice_uploaded(env: &Env, invoice: &Invoice) {
    env.events().publish(
        (symbol_short!("inv_up"),),
        (invoice.id.clone(), invoice.business.clone(), invoice.amount, invoice.currency.clone(), invoice.due_date),
    );
}

pub fn emit_invoice_verified(env: &Env, invoice: &Invoice) {
    env.events().publish(
        (symbol_short!("inv_ver"),),
        (invoice.id.clone(), invoice.business.clone()),
    );
} 