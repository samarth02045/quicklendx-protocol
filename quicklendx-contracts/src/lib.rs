#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, vec, Address, BytesN, Env, String, Vec, symbol_short,
};

mod invoice;
mod errors;
mod verification;
mod events;
mod bid;
mod investment;
mod payments;
mod settlement;
mod profits;
mod defaults;

use invoice::{Invoice, InvoiceStatus, InvoiceStorage};
use errors::QuickLendXError;
use verification::verify_invoice_data;
use events::{emit_invoice_uploaded, emit_invoice_verified};
use bid::{Bid, BidStatus, BidStorage};
use investment::{Investment, InvestmentStatus, InvestmentStorage};
use payments::transfer_funds;
use settlement::settle_invoice as do_settle_invoice;
use profits::calculate_profit as do_calculate_profit;
use defaults::handle_default as do_handle_default;

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
            business.clone(),
            amount,
            currency.clone(),
            due_date,
            description,
        );

        // Store the invoice
        InvoiceStorage::store_invoice(&env, &invoice);

        // Emit event
        env.events().publish(
            (symbol_short!("created"),),
            (invoice.id.clone(), business, amount, currency, due_date),
        );

        Ok(invoice.id)
    }

    /// Upload an invoice (business only)
    pub fn upload_invoice(
        env: Env,
        business: Address,
        amount: i128,
        currency: Address,
        due_date: u64,
        description: String,
    ) -> Result<BytesN<32>, QuickLendXError> {
        // Only the business can upload their own invoice
        business.require_auth();
        // Basic validation
        verify_invoice_data(&env, &business, amount, &currency, due_date, &description)?;
        // Create and store invoice
        let invoice = Invoice::new(&env, business.clone(), amount, currency.clone(), due_date, description.clone());
        InvoiceStorage::store_invoice(&env, &invoice);
        emit_invoice_uploaded(&env, &invoice);
        Ok(invoice.id)
    }

    /// Verify an invoice (admin or automated process)
    pub fn verify_invoice(env: Env, invoice_id: BytesN<32>) -> Result<(), QuickLendXError> {
        let mut invoice = InvoiceStorage::get_invoice(&env, &invoice_id)
            .ok_or(QuickLendXError::InvoiceNotFound)?;
        // Only allow verification if pending
        if invoice.status != InvoiceStatus::Pending {
            return Err(QuickLendXError::InvalidStatus);
        }
        // (Optional: Only admin can verify, add check here if needed)
        invoice.verify();
        InvoiceStorage::update_invoice(&env, &invoice);
        emit_invoice_verified(&env, &invoice);
        Ok(())
    }

    /// Get an invoice by ID
    pub fn get_invoice(env: Env, invoice_id: BytesN<32>) -> Result<Invoice, QuickLendXError> {
        InvoiceStorage::get_invoice(&env, &invoice_id)
            .ok_or(QuickLendXError::InvoiceNotFound)
    }

    /// Get all invoices for a business
    pub fn get_invoice_by_business(env: Env, business: Address) -> Vec<BytesN<32>> {
        InvoiceStorage::get_business_invoices(&env, &business)
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

        // Add to new status list - handled by store_invoice
        InvoiceStorage::update_invoice(&env, &invoice);

        // Emit event
        env.events().publish(
            (symbol_short!("updated"),),
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

    /// Place a bid on an invoice
    pub fn place_bid(
        env: Env,
        investor: Address,
        invoice_id: BytesN<32>,
        bid_amount: i128,
        expected_return: i128,
    ) -> Result<BytesN<32>, QuickLendXError> {
        // Only allow bids on verified invoices
        let invoice = invoice::InvoiceStorage::get_invoice(&env, &invoice_id)
            .ok_or(QuickLendXError::InvoiceNotFound)?;
        if invoice.status != invoice::InvoiceStatus::Verified {
            return Err(QuickLendXError::InvalidStatus);
        }
        if bid_amount <= 0 {
            return Err(QuickLendXError::InvalidAmount);
        }
        // Only the investor can place their own bid
        investor.require_auth();
        // Create bid
        let bid_id = BidStorage::generate_unique_bid_id(&env);
        let bid = Bid {
            bid_id: bid_id.clone(),
            invoice_id: invoice_id.clone(),
            investor: investor.clone(),
            bid_amount,
            expected_return,
            timestamp: env.ledger().timestamp(),
            status: BidStatus::Placed,
        };
        BidStorage::store_bid(&env, &bid);
        BidStorage::add_bid_to_invoice(&env, &invoice_id, &bid_id);
        Ok(bid_id)
    }

    /// Accept a bid (business only)
    pub fn accept_bid(
        env: Env,
        invoice_id: BytesN<32>,
        bid_id: BytesN<32>,
    ) -> Result<(), QuickLendXError> {
        let mut invoice = invoice::InvoiceStorage::get_invoice(&env, &invoice_id)
            .ok_or(QuickLendXError::InvoiceNotFound)?;
        let mut bid = BidStorage::get_bid(&env, &bid_id)
            .ok_or(QuickLendXError::StorageKeyNotFound)?;
        // Only the business owner can accept a bid
        invoice.business.require_auth();
        // Only allow accepting if invoice is verified and bid is placed
        if invoice.status != invoice::InvoiceStatus::Verified || bid.status != BidStatus::Placed {
            return Err(QuickLendXError::InvalidStatus);
        }
        // Transfer funds from investor to business
        let transfer_success = transfer_funds(&env, &bid.investor, &invoice.business, bid.bid_amount);
        if !transfer_success {
            return Err(QuickLendXError::InsufficientFunds);
        }
        // Mark bid as accepted
        bid.status = BidStatus::Accepted;
        BidStorage::update_bid(&env, &bid);
        // Mark invoice as funded
        invoice.mark_as_funded(bid.investor.clone(), bid.bid_amount, env.ledger().timestamp());
        invoice::InvoiceStorage::update_invoice(&env, &invoice);
        // Track investment
        let investment_id = InvestmentStorage::generate_unique_investment_id(&env);
        let investment = Investment {
            investment_id: investment_id.clone(),
            invoice_id: invoice_id.clone(),
            investor: bid.investor.clone(),
            amount: bid.bid_amount,
            funded_at: env.ledger().timestamp(),
            status: InvestmentStatus::Active,
        };
        InvestmentStorage::store_investment(&env, &investment);
        Ok(())
    }

    /// Withdraw a bid (investor only, before acceptance)
    pub fn withdraw_bid(
        env: Env,
        bid_id: BytesN<32>,
    ) -> Result<(), QuickLendXError> {
        let mut bid = BidStorage::get_bid(&env, &bid_id)
            .ok_or(QuickLendXError::StorageKeyNotFound)?;
        // Only the investor can withdraw their own bid
        bid.investor.require_auth();
        // Only allow withdrawal if bid is placed (not accepted/withdrawn)
        if bid.status != BidStatus::Placed {
            return Err(QuickLendXError::OperationNotAllowed);
        }
        bid.status = BidStatus::Withdrawn;
        BidStorage::update_bid(&env, &bid);
        Ok(())
    }

    /// Settle an invoice (business or automated process)
    pub fn settle_invoice(
        env: Env,
        invoice_id: BytesN<32>,
        payment_amount: i128,
        platform: Address,
        platform_fee_bps: i128,
    ) -> Result<(), QuickLendXError> {
        do_settle_invoice(&env, &invoice_id, payment_amount, &platform, platform_fee_bps)
    }

    /// Handle invoice default (admin or automated process)
    pub fn handle_default(
        env: Env,
        invoice_id: BytesN<32>,
    ) -> Result<(), QuickLendXError> {
        do_handle_default(&env, &invoice_id)
    }

    /// Calculate profit and platform fee
    pub fn calculate_profit(
        _env: Env,
        investment_amount: i128,
        payment_amount: i128,
        platform_fee_bps: i128,
    ) -> (i128, i128) {
        do_calculate_profit(investment_amount, payment_amount, platform_fee_bps)
    }
}

#[cfg(test)]
mod test; 