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

pub fn emit_invoice_settled(env: &Env, invoice: &crate::invoice::Invoice, investor_return: i128, platform_fee: i128) {
    env.events().publish(
        (symbol_short!("inv_set"),),
        (invoice.id.clone(), invoice.business.clone(), investor_return, platform_fee),
    );
}

pub fn emit_invoice_defaulted(env: &Env, invoice: &crate::invoice::Invoice) {
    env.events().publish(
        (symbol_short!("inv_def"),),
        (invoice.id.clone(), invoice.business.clone()),
    );
} 