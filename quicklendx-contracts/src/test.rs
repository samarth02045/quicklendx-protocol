#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short, vec, Address, BytesN, Env, String, Symbol,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
};

#[test]
fn test_store_invoice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let currency = Address::generate(&env);
    let amount = 1000;
    let due_date = env.ledger().timestamp() + 86400; // 1 day from now
    let description = String::from_str(&env, "Test invoice for services");

    let invoice_id = client.store_invoice(
        &business,
        &amount,
        &currency,
        &due_date,
        &description,
    );

    // Verify invoice was stored
    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.business, business);
    assert_eq!(invoice.amount, amount);
    assert_eq!(invoice.currency, currency);
    assert_eq!(invoice.due_date, due_date);
    assert_eq!(invoice.description, description);
    assert_eq!(invoice.status, InvoiceStatus::Pending);
    assert_eq!(invoice.funded_amount, 0);
    assert!(invoice.investor.is_none());
}

#[test]
fn test_store_invoice_validation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;

    // Test invalid amount
    let result = client.try_store_invoice(
        &business,
        &0,
        &currency,
        &due_date,
        &String::from_str(&env, "Test"),
    );
    assert!(result.is_err());

    // Test invalid due date (past date)
    let past_timestamp = if env.ledger().timestamp() > 86400 {
        env.ledger().timestamp() - 86400
    } else {
        0
    };
    let result = client.try_store_invoice(
        &business,
        &1000,
        &currency,
        &past_timestamp,
        &String::from_str(&env, "Test"),
    );
    assert!(result.is_err());

    // Test empty description
    let result = client.try_store_invoice(
        &business,
        &1000,
        &currency,
        &due_date,
        &String::from_str(&env, ""),
    );
    assert!(result.is_err());
}

#[test]
fn test_get_business_invoices() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business1 = Address::generate(&env);
    let business2 = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;

    // Create invoices for business1
    let invoice1_id = client.store_invoice(
        &business1,
        &1000,
        &currency,
        &due_date,
        &String::from_str(&env, "Invoice 1"),
    );

    let invoice2_id = client.store_invoice(
        &business1,
        &2000,
        &currency,
        &due_date,
        &String::from_str(&env, "Invoice 2"),
    );

    // Create invoice for business2
    let invoice3_id = client.store_invoice(
        &business2,
        &3000,
        &currency,
        &due_date,
        &String::from_str(&env, "Invoice 3"),
    );

    // Get invoices for business1
    let business1_invoices = client.get_business_invoices(&business1);
    assert_eq!(business1_invoices.len(), 2);
    assert!(business1_invoices.contains(&invoice1_id));
    assert!(business1_invoices.contains(&invoice2_id));

    // Get invoices for business2
    let business2_invoices = client.get_business_invoices(&business2);
    assert_eq!(business2_invoices.len(), 1);
    assert!(business2_invoices.contains(&invoice3_id));
}

#[test]
fn test_get_invoices_by_status() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;

    // Create invoices
    let invoice1_id = client.store_invoice(
        &business,
        &1000,
        &currency,
        &due_date,
        &String::from_str(&env, "Invoice 1"),
    );

    let invoice2_id = client.store_invoice(
        &business,
        &2000,
        &currency,
        &due_date,
        &String::from_str(&env, "Invoice 2"),
    );

    // Get pending invoices
    let pending_invoices = client.get_invoices_by_status(&InvoiceStatus::Pending);
    assert_eq!(pending_invoices.len(), 2);
    assert!(pending_invoices.contains(&invoice1_id));
    assert!(pending_invoices.contains(&invoice2_id));

    // Get verified invoices (should be empty initially)
    let verified_invoices = client.get_invoices_by_status(&InvoiceStatus::Verified);
    assert_eq!(verified_invoices.len(), 0);
}

#[test]
fn test_update_invoice_status() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;

    let invoice_id = client.store_invoice(
        &business,
        &1000,
        &currency,
        &due_date,
        &String::from_str(&env, "Test invoice"),
    );

    // Verify invoice starts as pending
    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.status, InvoiceStatus::Pending);

    // Update to verified
    client.update_invoice_status(&invoice_id, &InvoiceStatus::Verified);

    let invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.status, InvoiceStatus::Verified);

    // Check status lists
    let pending_invoices = client.get_invoices_by_status(&InvoiceStatus::Pending);
    assert_eq!(pending_invoices.len(), 0);

    let verified_invoices = client.get_invoices_by_status(&InvoiceStatus::Verified);
    assert_eq!(verified_invoices.len(), 1);
    assert!(verified_invoices.contains(&invoice_id));
}

#[test]
fn test_get_available_invoices() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;

    // Create invoices
    let invoice1_id = client.store_invoice(
        &business,
        &1000,
        &currency,
        &due_date,
        &String::from_str(&env, "Invoice 1"),
    );

    let invoice2_id = client.store_invoice(
        &business,
        &2000,
        &currency,
        &due_date,
        &String::from_str(&env, "Invoice 2"),
    );

    // Initially no available invoices (all pending)
    let available_invoices = client.get_available_invoices();
    assert_eq!(available_invoices.len(), 0);

    // Verify one invoice
    client.update_invoice_status(&invoice1_id, &InvoiceStatus::Verified);

    // Now one available invoice
    let available_invoices = client.get_available_invoices();
    assert_eq!(available_invoices.len(), 1);
    assert!(available_invoices.contains(&invoice1_id));
}

#[test]
fn test_invoice_count_functions() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;

    // Create invoices
    client.store_invoice(
        &business,
        &1000,
        &currency,
        &due_date,
        &String::from_str(&env, "Invoice 1"),
    );

    client.store_invoice(
        &business,
        &2000,
        &currency,
        &due_date,
        &String::from_str(&env, "Invoice 2"),
    );

    // Test count by status
    let pending_count = client.get_invoice_count_by_status(&InvoiceStatus::Pending);
    assert_eq!(pending_count, 2);

    let verified_count = client.get_invoice_count_by_status(&InvoiceStatus::Verified);
    assert_eq!(verified_count, 0);

    // Test total count
    let total_count = client.get_total_invoice_count();
    assert_eq!(total_count, 2);
}

#[test]
fn test_invoice_not_found() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let fake_id = BytesN::from_array(&env, &[0u8; 32]);

    let result = client.try_get_invoice(&fake_id);
    assert!(result.is_err());
}

#[test]
fn test_invoice_lifecycle() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;

    let invoice_id = client.store_invoice(
        &business,
        &1000,
        &currency,
        &due_date,
        &String::from_str(&env, "Test invoice"),
    );

    // Test lifecycle: Pending -> Verified -> Paid
    let mut invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.status, InvoiceStatus::Pending);

    client.update_invoice_status(&invoice_id, &InvoiceStatus::Verified);
    invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.status, InvoiceStatus::Verified);

    client.update_invoice_status(&invoice_id, &InvoiceStatus::Paid);
    invoice = client.get_invoice(&invoice_id);
    assert_eq!(invoice.status, InvoiceStatus::Paid);
    assert!(invoice.settled_at.is_some());
}

#[test]
fn test_simple_bid_storage() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register( QuickLendXContract, ());
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let investor = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;

    // Create and verify invoice
    let invoice_id = client.store_invoice(
        &business,
        &1000,
        &currency,
        &due_date,
        &String::from_str(&env, "Test invoice"),
    );
    
    client.update_invoice_status(&invoice_id, &InvoiceStatus::Verified);
    
    // Place a single bid to test basic functionality
    let bid_id = client.place_bid(&investor, &invoice_id, &1001, &1100);
    
    // Verify that the bid can be retrieved
    let bid = client.get_bid(&bid_id);
    assert!(bid.is_some(), "Bid should be retrievable");
    let bid = bid.unwrap();
    assert_eq!(bid.bid_amount, 1001);
    assert_eq!(bid.expected_return, 1100);
}

#[test]
fn test_unique_bid_id_generation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register( QuickLendXContract, ());
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let investor = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;

    // Create and verify invoice
    let invoice_id = client.store_invoice(
        &business,
        &1000,
        &currency,
        &due_date,
        &String::from_str(&env, "Test invoice"),
    );
    
    client.update_invoice_status(&invoice_id, &InvoiceStatus::Verified);
    
    // Place first bid
    let bid_id_1 = client.place_bid(&investor, &invoice_id, &1001, &1100);
    
    // Verify first bid was stored correctly
    let bid_1 = client.get_bid(&bid_id_1);
    assert!(bid_1.is_some(), "First bid should be retrievable");
    
    // Place second bid
    let bid_id_2 = client.place_bid(&investor, &invoice_id, &1002, &1200);
    
    // Verify that the bid IDs are different
    assert_ne!(bid_id_1, bid_id_2);
}

#[test]
fn test_escrow_creation_on_bid_acceptance() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(QuickLendXContract, ());
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let investor = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;
    let bid_amount = 1000i128;

    // Create and verify invoice
    let invoice_id = client.store_invoice(
        &business,
        &bid_amount,
        &currency,
        &due_date,
        &String::from_str(&env, "Test invoice"),
    );
    client.update_invoice_status(&invoice_id, &InvoiceStatus::Verified);

    // Place bid
    let bid_id = client.place_bid(&investor, &invoice_id, &bid_amount, &1100);

    // Accept bid (should create escrow)
    client.accept_bid(&invoice_id, &bid_id);

    // Verify escrow was created
    let escrow_details = client.get_escrow_details(&invoice_id);
    assert_eq!(escrow_details.invoice_id, invoice_id);
    assert_eq!(escrow_details.investor, investor);
    assert_eq!(escrow_details.business, business);
    assert_eq!(escrow_details.amount, bid_amount);
    assert_eq!(escrow_details.currency, currency);
    assert_eq!(escrow_details.status, crate::payments::EscrowStatus::Held);

    // Verify escrow status
    let escrow_status = client.get_escrow_status(&invoice_id);
    assert_eq!(escrow_status, crate::payments::EscrowStatus::Held);
}

#[test]
fn test_escrow_release_on_verification() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(QuickLendXContract, ());
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let investor = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;
    let bid_amount = 1000i128;

    // Create invoice
    let invoice_id = client.store_invoice(
        &business,
        &bid_amount,
        &currency,
        &due_date,
        &String::from_str(&env, "Test invoice"),
    );
    client.update_invoice_status(&invoice_id, &InvoiceStatus::Verified);

    // Place and accept bid (creates escrow)
    let bid_id = client.place_bid(&investor, &invoice_id, &bid_amount, &1100);
    client.accept_bid(&invoice_id, &bid_id);

    // Verify escrow is held
    let escrow_status = client.get_escrow_status(&invoice_id);
    assert_eq!(escrow_status, crate::payments::EscrowStatus::Held);

    // Release escrow funds
    client.release_escrow_funds(&invoice_id);

    // Verify escrow is released
    let escrow_status = client.get_escrow_status(&invoice_id);
    assert_eq!(escrow_status, crate::payments::EscrowStatus::Released);
}

#[test]
fn test_escrow_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(QuickLendXContract,());
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let investor = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;
    let bid_amount = 1000i128;

    // Create invoice
    let invoice_id = client.store_invoice(
        &business,
        &bid_amount,
        &currency,
        &due_date,
        &String::from_str(&env, "Test invoice"),
    );
    client.update_invoice_status(&invoice_id, &InvoiceStatus::Verified);

    // Place and accept bid (creates escrow)
    let bid_id = client.place_bid(&investor, &invoice_id, &bid_amount, &1100);
    client.accept_bid(&invoice_id, &bid_id);

    // Verify escrow is held
    let escrow_status = client.get_escrow_status(&invoice_id);
    assert_eq!(escrow_status, crate::payments::EscrowStatus::Held);

    // Refund escrow funds
    client.refund_escrow_funds(&invoice_id);

    // Verify escrow is refunded
    let escrow_status = client.get_escrow_status(&invoice_id);
    assert_eq!(escrow_status, crate::payments::EscrowStatus::Refunded);
}

#[test]
fn test_escrow_status_tracking() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register( QuickLendXContract, ());
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let investor = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;
    let bid_amount = 1000i128;

    // Create and verify invoice
    let invoice_id = client.store_invoice(
        &business,
        &bid_amount,
        &currency,
        &due_date,
        &String::from_str(&env, "Test invoice"),
    );
    client.update_invoice_status(&invoice_id, &InvoiceStatus::Verified);

    // Place and accept bid
    let bid_id = client.place_bid(&investor, &invoice_id, &bid_amount, &1100);
    client.accept_bid(&invoice_id, &bid_id);

    // Test escrow details
    let escrow_details = client.get_escrow_details(&invoice_id);
    assert_eq!(escrow_details.status, crate::payments::EscrowStatus::Held);
    // created_at is set to ledger timestamp (u64 is always >= 0)
    assert_eq!(escrow_details.amount, bid_amount);

    // Test status progression: Held -> Released
    client.release_escrow_funds(&invoice_id);
    let escrow_details = client.get_escrow_details(&invoice_id);
    assert_eq!(escrow_details.status, crate::payments::EscrowStatus::Released);
}

#[test]
fn test_escrow_error_cases() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register( QuickLendXContract, ());
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let fake_invoice_id = BytesN::from_array(&env, &[1u8; 32]);

    // Test getting escrow for non-existent invoice
    let result = client.try_get_escrow_status(&fake_invoice_id);
    assert!(result.is_err());

    let result = client.try_get_escrow_details(&fake_invoice_id);
    assert!(result.is_err());

    // Test releasing escrow for non-existent invoice
    let result = client.try_release_escrow_funds(&fake_invoice_id);
    assert!(result.is_err());

    // Test refunding escrow for non-existent invoice
    let result = client.try_refund_escrow_funds(&fake_invoice_id);
    assert!(result.is_err());
}

#[test]
fn test_escrow_double_operation_prevention() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(QuickLendXContract, ());
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let investor = Address::generate(&env);
    let currency = Address::generate(&env);
    let due_date = env.ledger().timestamp() + 86400;
    let bid_amount = 1000i128;

    // Create and verify invoice
    let invoice_id = client.store_invoice(
        &business,
        &bid_amount,
        &currency,
        &due_date,
        &String::from_str(&env, "Test invoice"),
    );
    client.update_invoice_status(&invoice_id, &InvoiceStatus::Verified);

    // Place and accept bid
    let bid_id = client.place_bid(&investor, &invoice_id, &bid_amount, &1100);
    client.accept_bid(&invoice_id, &bid_id);

    // Release escrow funds
    client.release_escrow_funds(&invoice_id);

    // Try to release again (should fail)
    let result = client.try_release_escrow_funds(&invoice_id);
    assert!(result.is_err());

    // Try to refund after release (should fail)
    let result = client.try_refund_escrow_funds(&invoice_id);
    assert!(result.is_err());
}

#[test]
fn test_unique_investment_id_generation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    
    env.as_contract(&contract_id, || {
        let mut ids = Vec::new(&env);
        
        // Generate 100 unique investment IDs (reduced for faster testing)
        for _ in 0..100 {
            let id = crate::investment::InvestmentStorage::generate_unique_investment_id(&env);
            
            // Check if this ID already exists in our vector
            for i in 0..ids.len() {
                let existing_id = ids.get(i).unwrap();
                assert_ne!(id, existing_id, "Duplicate investment ID generated");
            }
            
            ids.push_back(id);
        }
    });
}