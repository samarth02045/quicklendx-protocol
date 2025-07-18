use soroban_sdk::{Address, BytesN, Env};
use crate::invoice::{Invoice, InvoiceStatus, InvoiceStorage};
use crate::investment::{Investment, InvestmentStatus, InvestmentStorage};
use crate::payments::transfer_funds;
use crate::profits::calculate_profit;
use crate::events::emit_invoice_settled;
use crate::errors::QuickLendXError;

pub fn settle_invoice(
    env: &Env,
    invoice_id: &BytesN<32>,
    payment_amount: i128,
    platform: &Address,
    platform_fee_bps: i128,
) -> Result<(), QuickLendXError> {
    let mut invoice = InvoiceStorage::get_invoice(env, invoice_id)
        .ok_or(QuickLendXError::InvoiceNotFound)?;
    if invoice.status != InvoiceStatus::Funded {
        return Err(QuickLendXError::InvalidStatus);
    }
    let investor = invoice.investor.as_ref().ok_or(QuickLendXError::NotInvestor)?;
    let investment = InvestmentStorage::get_investment(env, invoice_id)
        .ok_or(QuickLendXError::StorageKeyNotFound)?;
    // Calculate profit and platform fee
    let (investor_return, platform_fee) = calculate_profit(investment.amount, payment_amount, platform_fee_bps);
    // Transfer funds
    let investor_paid = transfer_funds(env, &invoice.business, investor, investor_return);
    let platform_paid = transfer_funds(env, &invoice.business, platform, platform_fee);
    if !investor_paid || !platform_paid {
        return Err(QuickLendXError::InsufficientFunds);
    }
    // Update invoice and investment status
    invoice.mark_as_paid(env.ledger().timestamp());
    InvoiceStorage::update_invoice(env, &invoice);
    let mut investment = investment;
    investment.status = InvestmentStatus::Completed;
    InvestmentStorage::update_investment(env, &investment);
    // Emit event
    emit_invoice_settled(env, &invoice, investor_return, platform_fee);
    Ok(())
} 