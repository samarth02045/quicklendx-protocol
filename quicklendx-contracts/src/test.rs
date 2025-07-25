#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

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

    let invoice_id = client.store_invoice(&business, &amount, &currency, &due_date, &description);

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
    let result = client.try_store_invoice(
        &business,
        &1000,
        &currency,
        &(env.ledger().timestamp() - 86400),
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
fn test_unique_bid_id_generation() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);

    env.as_contract(&contract_id, || {
        let mut ids = Vec::new(&env);

        // Generate 100 unique bid IDs (reduced for faster testing)
        for _ in 0..100 {
            let id = crate::bid::BidStorage::generate_unique_bid_id(&env);

            // Check if this ID already exists in our vector
            for i in 0..ids.len() {
                let existing_id = ids.get(i).unwrap();
                assert_ne!(id, existing_id, "Duplicate bid ID generated");
            }

            ids.push_back(id);
        }
    });
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

// Business KYC/Verification Tests

#[test]
fn test_submit_kyc_application() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let kyc_data = String::from_str(&env, "Business registration documents");

    // Mock business authorization
    env.mock_all_auths();

    client.submit_kyc_application(&business, &kyc_data);

    // Verify KYC was submitted
    let verification = client.get_business_verification_status(&business);
    assert!(verification.is_some());
    let verification = verification.unwrap();
    assert_eq!(verification.business, business);
    assert_eq!(verification.kyc_data, kyc_data);
    assert!(matches!(
        verification.status,
        verification::BusinessVerificationStatus::Pending
    ));
}

#[test]
fn test_verify_business() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let business = Address::generate(&env);
    let kyc_data = String::from_str(&env, "Business registration documents");

    // Set admin
    client.set_admin(&admin);

    // Submit KYC application
    env.mock_all_auths();
    client.submit_kyc_application(&business, &kyc_data);

    // Verify business
    env.mock_all_auths();
    client.verify_business(&admin, &business);

    // Check verification status
    let verification = client.get_business_verification_status(&business);
    assert!(verification.is_some());
    let verification = verification.unwrap();
    assert!(matches!(
        verification.status,
        verification::BusinessVerificationStatus::Verified
    ));
    assert!(verification.verified_at.is_some());
    assert_eq!(verification.verified_by, Some(admin));
}

#[test]
fn test_reject_business() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let business = Address::generate(&env);
    let kyc_data = String::from_str(&env, "Business registration documents");
    let rejection_reason = String::from_str(&env, "Incomplete documentation");

    // Set admin
    client.set_admin(&admin);

    // Submit KYC application
    env.mock_all_auths();
    client.submit_kyc_application(&business, &kyc_data);

    // Reject business
    env.mock_all_auths();
    client.reject_business(&admin, &business, &rejection_reason);

    // Check verification status
    let verification = client.get_business_verification_status(&business);
    assert!(verification.is_some());
    let verification = verification.unwrap();
    assert!(matches!(
        verification.status,
        verification::BusinessVerificationStatus::Rejected
    ));
    assert_eq!(verification.rejection_reason, Some(rejection_reason));
}

#[test]
fn test_upload_invoice_requires_verification() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let currency = Address::generate(&env);
    let amount = 1000;
    let due_date = env.ledger().timestamp() + 86400;
    let description = String::from_str(&env, "Test invoice");

    // Mock business authorization
    env.mock_all_auths();

    // Try to upload invoice without verification - should fail
    let result = client.try_upload_invoice(&business, &amount, &currency, &due_date, &description);
    assert!(result.is_err());

    // Submit KYC and verify business
    let admin = Address::generate(&env);
    let kyc_data = String::from_str(&env, "Business registration documents");

    client.set_admin(&admin);
    env.mock_all_auths();
    client.submit_kyc_application(&business, &kyc_data);

    env.mock_all_auths();
    client.verify_business(&admin, &business);

    // Now try to upload invoice - should succeed
    env.mock_all_auths();
    let _invoice_id = client.upload_invoice(&business, &amount, &currency, &due_date, &description);
}

#[test]
fn test_kyc_already_pending() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let business = Address::generate(&env);
    let kyc_data = String::from_str(&env, "Business registration documents");

    // Mock business authorization
    env.mock_all_auths();

    // Submit KYC application
    client.submit_kyc_application(&business, &kyc_data);

    // Try to submit again - should fail
    let result = client.try_submit_kyc_application(&business, &kyc_data);
    assert!(result.is_err());
}

#[test]
fn test_kyc_already_verified() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let business = Address::generate(&env);
    let kyc_data = String::from_str(&env, "Business registration documents");

    // Set admin and submit KYC
    client.set_admin(&admin);
    env.mock_all_auths();
    client.submit_kyc_application(&business, &kyc_data);

    // Verify business
    env.mock_all_auths();
    client.verify_business(&admin, &business);

    // Try to submit KYC again - should fail
    let result = client.try_submit_kyc_application(&business, &kyc_data);
    assert!(result.is_err());
}

#[test]
fn test_kyc_resubmission_after_rejection() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let business = Address::generate(&env);
    let kyc_data = String::from_str(&env, "Business registration documents");
    let rejection_reason = String::from_str(&env, "Incomplete documentation");

    // Set admin and submit KYC
    client.set_admin(&admin);
    env.mock_all_auths();
    client.submit_kyc_application(&business, &kyc_data);

    // Reject business
    env.mock_all_auths();
    client.reject_business(&admin, &business, &rejection_reason);

    // Try to resubmit KYC - should succeed
    let new_kyc_data = String::from_str(&env, "Updated business registration documents");
    env.mock_all_auths();
    client.submit_kyc_application(&business, &new_kyc_data);

    // Check status is back to pending
    let verification = client.get_business_verification_status(&business);
    assert!(verification.is_some());
    let verification = verification.unwrap();
    assert!(matches!(
        verification.status,
        verification::BusinessVerificationStatus::Pending
    ));
    assert_eq!(verification.kyc_data, new_kyc_data);
}

#[test]
fn test_verification_unauthorized_access() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let business = Address::generate(&env);
    let unauthorized_admin = Address::generate(&env);

    // Set admin
    client.set_admin(&admin);

    // Submit KYC application
    env.mock_all_auths();
    let kyc_data = String::from_str(&env, "Business registration documents");
    client.submit_kyc_application(&business, &kyc_data);

    // Try to verify with unauthorized admin - should fail
    env.mock_all_auths();
    let result = client.try_verify_business(&unauthorized_admin, &business);
    assert!(result.is_err());
}

#[test]
fn test_get_verification_lists() {
    let env = Env::default();
    let contract_id = env.register_contract(None, QuickLendXContract);
    let client = QuickLendXContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let business1 = Address::generate(&env);
    let business2 = Address::generate(&env);
    let business3 = Address::generate(&env);

    // Set admin
    client.set_admin(&admin);

    // Submit KYC applications
    env.mock_all_auths();
    let kyc_data = String::from_str(&env, "Business registration documents");
    client.submit_kyc_application(&business1, &kyc_data);
    client.submit_kyc_application(&business2, &kyc_data);
    client.submit_kyc_application(&business3, &kyc_data);

    // Verify business1, reject business2, leave business3 pending
    env.mock_all_auths();
    client.verify_business(&admin, &business1);
    client.reject_business(&admin, &business2, &String::from_str(&env, "Rejected"));

    // Check lists
    let verified = client.get_verified_businesses();
    let pending = client.get_pending_businesses();
    let rejected = client.get_rejected_businesses();

    assert_eq!(verified.len(), 1);
    assert_eq!(pending.len(), 1);
    assert_eq!(rejected.len(), 1);

    assert!(verified.contains(&business1));
    assert!(pending.contains(&business3));
    assert!(rejected.contains(&business2));
}
