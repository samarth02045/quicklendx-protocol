use crate::invoice::Invoice;
use soroban_sdk::{symbol_short, Address, Env};

pub fn emit_invoice_uploaded(env: &Env, invoice: &Invoice) {
    env.events().publish(
        (symbol_short!("inv_up"),),
        (
            invoice.id.clone(),
            invoice.business.clone(),
            invoice.amount,
            invoice.currency.clone(),
            invoice.due_date,
        ),
    );
}

pub fn emit_invoice_verified(env: &Env, invoice: &Invoice) {
    env.events().publish(
        (symbol_short!("inv_ver"),),
        (invoice.id.clone(), invoice.business.clone()),
    );
}

pub fn emit_invoice_settled(
    env: &Env,
    invoice: &crate::invoice::Invoice,
    investor_return: i128,
    platform_fee: i128,
) {
    env.events().publish(
        (symbol_short!("inv_set"),),
        (
            invoice.id.clone(),
            invoice.business.clone(),
            investor_return,
            platform_fee,
        ),
    );
}

pub fn emit_invoice_defaulted(env: &Env, invoice: &crate::invoice::Invoice) {
    env.events().publish(
        (symbol_short!("inv_def"),),
        (invoice.id.clone(), invoice.business.clone()),
    );
}

// Business verification events
pub fn emit_kyc_submitted(env: &Env, business: &Address) {
    env.events().publish(
        (symbol_short!("kyc_sub"),),
        (business.clone(), env.ledger().timestamp()),
    );
}

pub fn emit_business_verified(env: &Env, business: &Address, admin: &Address) {
    env.events().publish(
        (symbol_short!("bus_ver"),),
        (business.clone(), admin.clone(), env.ledger().timestamp()),
    );
}

pub fn emit_business_rejected(env: &Env, business: &Address, admin: &Address) {
    env.events().publish(
        (symbol_short!("bus_rej"),),
        (business.clone(), admin.clone(), env.ledger().timestamp()),
    );
}
