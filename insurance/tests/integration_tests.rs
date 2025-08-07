use insurance::*;
use intake::*;
use std::time::Instant;

// Helper function to create a test claim
fn create_test_claim(payer_id: PayerId, service_lines: Vec<intake::ServiceLine>) -> PayerClaim {
    PayerClaim {
        claim_id: "TEST001".to_string(),
        place_of_service_code: 11,
        insurance: intake::Insurance {
            payer_id,
            patient_member_id: "PAT123".to_string(),
        },
        patient: Patient {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            gender: Gender::Male,
            dob: "1980-01-15".to_string(),
            email: Some("john.doe@test.com".to_string()),
            address: None,
        },
        organization: Organization {
            name: "Test Clinic".to_string(),
            billing_npi: Some("1234567890".to_string()),
            ein: Some("12-3456789".to_string()),
            contact: None,
            address: None,
        },
        rendering_provider: RenderingProvider {
            first_name: "Dr. Jane".to_string(),
            last_name: "Smith".to_string(),
            npi: "9876543210".to_string(),
        },
        service_lines,
        initial_claim_ts: 1640995200000, // Fixed timestamp for predictable tests
    }
}

// Helper function to create a test service line
fn create_test_service_line(
    id: &str,
    procedure_code: &str,
    units: i32,
    unit_charge: f64,
    do_not_bill: Option<bool>,
) -> intake::ServiceLine {
    intake::ServiceLine {
        service_line_id: id.to_string(),
        procedure_code: procedure_code.to_string(),
        modifiers: None,
        units,
        details: "Test service".to_string(),
        unit_charge_currency: "USD".to_string(),
        unit_charge_amount: unit_charge,
        do_not_bill,
    }
}

#[test]
fn test_medicare_new() {
    let medicare = Medicare::new();
    assert_eq!(medicare.min_response_time_secs, 10);
    assert_eq!(medicare.max_response_time_secs, 30);
}

#[test]
fn test_united_health_group_new() {
    let uhg = UnitedHealthGroup::new();
    assert_eq!(uhg.min_response_time_secs, 10);
    assert_eq!(uhg.max_response_time_secs, 30);
}

#[test]
fn test_anthem_new() {
    let anthem = Anthem::new();
    assert_eq!(anthem.min_response_time_secs, 10);
    assert_eq!(anthem.max_response_time_secs, 30);
}

#[test]
fn test_medicare_submit_claim_single_service_line() {
    let medicare = Medicare::new();
    let service_line = create_test_service_line("SL001", "99213", 1, 100.0, None);
    let claim = create_test_claim(PayerId::Medicare, vec![service_line]);
    
    let start_time = Instant::now();
    let result = medicare.submit_claim(&claim);
    let elapsed = start_time.elapsed();
    
    assert!(result.is_ok());
    let remittance = result.unwrap();
    
    // Verify remittance structure
    assert_eq!(remittance.claim_id, "TEST001");
    assert_eq!(remittance.payer_id, "Medicare");
    assert_eq!(remittance.patient_id, "Medicare-PAT123");
    assert_eq!(remittance.payee_npi, "1234567890");
    assert_eq!(remittance.service_lines.len(), 1);
    assert_eq!(remittance.initial_claim_ts, 1640995200000);
    
    // Verify service line calculations (100 * 1 = 100 billed)
    let service_line = &remittance.service_lines[0];
    assert_eq!(service_line.service_line_id, "SL001");
    assert_eq!(service_line.procedure_code, "99213");
    assert_eq!(service_line.billed_amount, 100.0);
    
    // Medicare realistic heuristics: $257 deductible, 80/20 split after deductible
    // For $100 bill, expect most as deductible since it's under $257
    assert!(service_line.deductible_amount > 0.0);
    assert!(service_line.coinsurance_amount >= 0.0);
    assert_eq!(service_line.copay_amount, 0.0); // Medicare Part B no copays
    assert!(service_line.not_allowed_amount >= 0.0);
    
    // Verify amounts sum to billed amount (within small tolerance for floating point)
    let total = service_line.payer_paid_amount + service_line.coinsurance_amount + 
                service_line.copay_amount + service_line.deductible_amount + service_line.not_allowed_amount;
    assert!((total - service_line.billed_amount).abs() < 0.01);
    assert_eq!(service_line.remark_codes, None);
    
    // Verify timing (should sleep between min and max)
    assert!(elapsed.as_secs() >= 10);  // Medicare::new() uses min_response_time_secs: 10
    assert!(elapsed.as_secs() <= 35);  // Allow some buffer for processing time
}

#[test]
fn test_united_health_group_submit_claim() {
    let uhg = UnitedHealthGroup {
        min_response_time_secs: 1,
        max_response_time_secs: 2
    };
    let service_line = create_test_service_line("SL002", "99214", 2, 75.0, None);
    let claim = create_test_claim(PayerId::UnitedHealthGroup, vec![service_line]);
    
    let result = uhg.submit_claim(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    assert_eq!(remittance.payer_id, "UnitedHealthGroup");
    assert_eq!(remittance.patient_id, "UnitedHealthGroup-PAT123");
    
    // Verify calculations (75 * 2 = 150 billed)
    let service_line = &remittance.service_lines[0];
    assert_eq!(service_line.billed_amount, 150.0);
    
    // UnitedHealthGroup realistic heuristics: ~$1800 deductible, 70-80% coverage, $25-35 copay
    // For $150 bill, expect most as deductible since it's under $1800
    assert!(service_line.deductible_amount > 0.0);
    assert!(service_line.coinsurance_amount >= 0.0);
    assert!(service_line.copay_amount >= 0.0); // UHG uses copays
    assert!(service_line.not_allowed_amount >= 0.0);
    
    // Verify amounts sum to billed amount
    let total = service_line.payer_paid_amount + service_line.coinsurance_amount + 
                service_line.copay_amount + service_line.deductible_amount + service_line.not_allowed_amount;
    assert!((total - service_line.billed_amount).abs() < 0.01);
}

#[test]
fn test_anthem_submit_claim() {
    let anthem = Anthem {
        min_response_time_secs: 1,
        max_response_time_secs: 2
    };
    let service_line = create_test_service_line("SL003", "99215", 1, 200.0, None);
    let claim = create_test_claim(PayerId::Anthem, vec![service_line]);
    
    let result = anthem.submit_claim(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    assert_eq!(remittance.payer_id, "Anthem");
    assert_eq!(remittance.patient_id, "Anthem-PAT123");
    
    // Verify calculations (200 * 1 = 200 billed)
    let service_line = &remittance.service_lines[0];
    assert_eq!(service_line.billed_amount, 200.0);
    
    // Anthem realistic heuristics: ~$1650-2000 deductible, 70/30 split, $20-30 copay
    // For $200 bill, expect most as deductible since it's under typical deductible
    assert!(service_line.deductible_amount > 0.0);
    assert!(service_line.coinsurance_amount >= 0.0);
    assert!(service_line.copay_amount >= 0.0); // Anthem uses copays
    assert!(service_line.not_allowed_amount >= 0.0);
    
    // Verify amounts sum to billed amount
    let total = service_line.payer_paid_amount + service_line.coinsurance_amount + 
                service_line.copay_amount + service_line.deductible_amount + service_line.not_allowed_amount;
    assert!((total - service_line.billed_amount).abs() < 0.01);
}

#[test]
fn test_multiple_service_lines() {
    let medicare = Medicare {
        min_response_time_secs: 1,
        max_response_time_secs: 2
    };
    let service_lines = vec![
        create_test_service_line("SL001", "99213", 1, 100.0, None),
        create_test_service_line("SL002", "99214", 2, 150.0, None),
        create_test_service_line("SL003", "99215", 1, 250.0, None),
    ];
    let claim = create_test_claim(PayerId::Medicare, service_lines);
    
    let result = medicare.submit_claim(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    assert_eq!(remittance.service_lines.len(), 3);
    
    // Verify each service line
    assert_eq!(remittance.service_lines[0].billed_amount, 100.0);
    assert_eq!(remittance.service_lines[1].billed_amount, 300.0); // 150 * 2
    assert_eq!(remittance.service_lines[2].billed_amount, 250.0);
}

#[test]
fn test_do_not_bill_true() {
    let medicare = Medicare {
        min_response_time_secs: 1,
        max_response_time_secs: 2
    };
    let service_line = create_test_service_line("SL001", "99213", 1, 100.0, Some(true));
    let claim = create_test_claim(PayerId::Medicare, vec![service_line]);
    
    let result = medicare.submit_claim(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    let service_line = &remittance.service_lines[0];
    
    // All amounts should be zero when do_not_bill is true
    assert_eq!(service_line.billed_amount, 0.0);
    assert_eq!(service_line.payer_paid_amount, 0.0);
    assert_eq!(service_line.coinsurance_amount, 0.0);
    assert_eq!(service_line.copay_amount, 0.0);
    assert_eq!(service_line.deductible_amount, 0.0);
    assert_eq!(service_line.not_allowed_amount, 0.0);
}

#[test]
fn test_do_not_bill_false() {
    let medicare = Medicare {
        min_response_time_secs: 1,
        max_response_time_secs: 2
    };
    let service_line = create_test_service_line("SL001", "99213", 1, 100.0, Some(false));
    let claim = create_test_claim(PayerId::Medicare, vec![service_line]);
    
    let result = medicare.submit_claim(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    let service_line = &remittance.service_lines[0];
    
    // Should calculate normally when do_not_bill is false
    assert_eq!(service_line.billed_amount, 100.0);
    // Verify amounts sum correctly instead of exact values
    let total = service_line.payer_paid_amount + service_line.coinsurance_amount + 
                service_line.copay_amount + service_line.deductible_amount + service_line.not_allowed_amount;
    assert!((total - service_line.billed_amount).abs() < 0.01);
}

#[test]
fn test_do_not_bill_none() {
    let medicare = Medicare {
        min_response_time_secs: 1,
        max_response_time_secs: 2
    };
    let service_line = create_test_service_line("SL001", "99213", 1, 100.0, None);
    let claim = create_test_claim(PayerId::Medicare, vec![service_line]);
    
    let result = medicare.submit_claim(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    let service_line = &remittance.service_lines[0];
    
    // Should calculate normally when do_not_bill is None
    assert_eq!(service_line.billed_amount, 100.0);
    // Verify amounts sum correctly instead of exact values
    let total = service_line.payer_paid_amount + service_line.coinsurance_amount + 
                service_line.copay_amount + service_line.deductible_amount + service_line.not_allowed_amount;
    assert!((total - service_line.billed_amount).abs() < 0.01);
}

#[test]
fn test_mixed_do_not_bill_service_lines() {
    let medicare = Medicare {
        min_response_time_secs: 1,
        max_response_time_secs: 2
    };
    let service_lines = vec![
        create_test_service_line("SL001", "99213", 1, 100.0, Some(true)),  // Should be zero
        create_test_service_line("SL002", "99214", 1, 150.0, Some(false)), // Should calculate
        create_test_service_line("SL003", "99215", 1, 200.0, None),        // Should calculate
    ];
    let claim = create_test_claim(PayerId::Medicare, service_lines);
    
    let result = medicare.submit_claim(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    
    // First service line (do_not_bill = true)
    assert_eq!(remittance.service_lines[0].billed_amount, 0.0);
    assert_eq!(remittance.service_lines[0].payer_paid_amount, 0.0);
    
    // Second service line (do_not_bill = false) - should calculate normally
    assert_eq!(remittance.service_lines[1].billed_amount, 150.0);
    let total_sl2 = remittance.service_lines[1].payer_paid_amount + remittance.service_lines[1].coinsurance_amount + 
                    remittance.service_lines[1].copay_amount + remittance.service_lines[1].deductible_amount + 
                    remittance.service_lines[1].not_allowed_amount;
    assert!((total_sl2 - 150.0).abs() < 0.01);
    
    // Third service line (do_not_bill = None) - should calculate normally
    assert_eq!(remittance.service_lines[2].billed_amount, 200.0);
    let total_sl3 = remittance.service_lines[2].payer_paid_amount + remittance.service_lines[2].coinsurance_amount + 
                    remittance.service_lines[2].copay_amount + remittance.service_lines[2].deductible_amount + 
                    remittance.service_lines[2].not_allowed_amount;
    assert!((total_sl3 - 200.0).abs() < 0.01);
}

#[test]
fn test_zero_unit_charge_amount() {
    let medicare = Medicare {
        min_response_time_secs: 1,
        max_response_time_secs: 2
    };
    let service_line = create_test_service_line("SL001", "99213", 1, 0.0, None);
    let claim = create_test_claim(PayerId::Medicare, vec![service_line]);
    
    let result = medicare.submit_claim(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    let service_line = &remittance.service_lines[0];
    
    // All amounts should be zero when unit charge is zero
    assert_eq!(service_line.billed_amount, 0.0);
    assert_eq!(service_line.payer_paid_amount, 0.0);
    assert_eq!(service_line.coinsurance_amount, 0.0);
    assert_eq!(service_line.copay_amount, 0.0);
    assert_eq!(service_line.deductible_amount, 0.0);
    assert_eq!(service_line.not_allowed_amount, 0.0);
}

#[test]
fn test_large_amounts() {
    let medicare = Medicare {
        min_response_time_secs: 1,
        max_response_time_secs: 2
    };
    let service_line = create_test_service_line("SL001", "99213", 100, 999.99, None);
    let claim = create_test_claim(PayerId::Medicare, vec![service_line]);
    
    let result = medicare.submit_claim(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    let service_line = &remittance.service_lines[0];
    
    // Verify large amount calculations (999.99 * 100 = 99999.00)
    assert!((service_line.billed_amount - 99999.0).abs() < 0.01);
    
    // For large amounts, Medicare deductible is capped at $257, so most should be subject to 80/20 split
    // After $257 deductible, remaining ~$99,742 should be split 80/20 with some denials
    assert!(service_line.deductible_amount <= 257.0); // Medicare 2025 deductible cap
    assert!(service_line.payer_paid_amount > 70000.0); // Should be substantial payer payment
    assert!(service_line.coinsurance_amount > 15000.0); // Should have coinsurance on remaining amount
    assert_eq!(service_line.copay_amount, 0.0); // Medicare Part B no copays
    assert!(service_line.not_allowed_amount > 0.0); // Some denial expected
    
    // Verify amounts sum to billed amount
    let total = service_line.payer_paid_amount + service_line.coinsurance_amount + 
                service_line.copay_amount + service_line.deductible_amount + service_line.not_allowed_amount;
    assert!((total - service_line.billed_amount).abs() < 0.01);
}

#[test]
fn test_missing_billing_npi() {
    let medicare = Medicare {
        min_response_time_secs: 1,
        max_response_time_secs: 2
    };
    let service_line = create_test_service_line("SL001", "99213", 1, 100.0, None);
    let mut claim = create_test_claim(PayerId::Medicare, vec![service_line]);
    claim.organization.billing_npi = None; // Remove billing NPI
    
    let result = medicare.submit_claim(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    assert_eq!(remittance.payee_npi, ""); // Should default to empty string
}

#[test]
fn test_remittance_id_uniqueness() {
    let medicare = Medicare {
        min_response_time_secs: 1,
        max_response_time_secs: 2
    };
    let service_line = create_test_service_line("SL001", "99213", 1, 100.0, None);
    let claim = create_test_claim(PayerId::Medicare, vec![service_line]);
    
    let remittance1 = medicare.submit_claim(&claim).unwrap();
    let remittance2 = medicare.submit_claim(&claim).unwrap();
    
    // Remittance IDs should be unique (UUIDs)
    assert_ne!(remittance1.remittance_id, remittance2.remittance_id);
    assert!(remittance1.remittance_id.len() > 0);
    assert!(remittance2.remittance_id.len() > 0);
}

#[test]
fn test_create_remittance_function() {
    let service_line = create_test_service_line("SL001", "99213", 1, 100.0, None);
    let claim = create_test_claim(PayerId::Medicare, vec![service_line.clone()]);
    
    // Create a service line for remittance
    let remittance_service_line = insurance::ServiceLine::new(
        &service_line, 
        100.0, 
        80.0, 
        10.0, 
        5.0, 
        5.0, 
        5.0
    ).unwrap();
    
    let remittance = create_remittance(vec![remittance_service_line], &claim);
    
    assert_eq!(remittance.claim_id, "TEST001");
    assert_eq!(remittance.payer_id, "Medicare");
    assert_eq!(remittance.patient_id, "Medicare-PAT123");
    assert_eq!(remittance.payee_npi, "1234567890");
    assert_eq!(remittance.service_lines.len(), 1);
    assert_eq!(remittance.initial_claim_ts, 1640995200000);
    assert!(remittance.remittance_id.len() > 0);
}

#[test]
fn test_service_line_new_success() {
    let intake_service_line = create_test_service_line("SL001", "99213", 1, 100.0, None);
    
    let result = insurance::ServiceLine::new(
        &intake_service_line,
        100.0,
        80.0,
        10.0,
        5.0,
        5.0,
        5.0,
    );
    
    assert!(result.is_ok());
    let service_line = result.unwrap();
    
    assert_eq!(service_line.service_line_id, "SL001");
    assert_eq!(service_line.procedure_code, "99213");
    assert_eq!(service_line.billed_amount, 100.0);
    assert_eq!(service_line.payer_paid_amount, 80.0);
    assert_eq!(service_line.coinsurance_amount, 10.0);
    assert_eq!(service_line.copay_amount, 5.0);
    assert_eq!(service_line.deductible_amount, 5.0);
    assert_eq!(service_line.not_allowed_amount, 5.0);
    assert_eq!(service_line.remark_codes, None);
}

#[test]
fn test_service_line_new_do_not_bill() {
    let intake_service_line = create_test_service_line("SL001", "99213", 1, 100.0, Some(true));
    
    let result = insurance::ServiceLine::new(
        &intake_service_line,
        100.0,
        80.0,
        10.0,
        5.0,
        5.0,
        5.0,
    );
    
    assert!(result.is_ok());
    let service_line = result.unwrap();
    
    // All amounts should be zero when do_not_bill is true
    assert_eq!(service_line.service_line_id, "SL001");
    assert_eq!(service_line.procedure_code, "99213");
    assert_eq!(service_line.billed_amount, 0.0);
    assert_eq!(service_line.payer_paid_amount, 0.0);
    assert_eq!(service_line.coinsurance_amount, 0.0);
    assert_eq!(service_line.copay_amount, 0.0);
    assert_eq!(service_line.deductible_amount, 0.0);
    assert_eq!(service_line.not_allowed_amount, 0.0);
    assert_eq!(service_line.remark_codes, None);
}

// #[test]
// fn test_all_payers_same_calculation_logic() {
//     let service_line = create_test_service_line("SL001", "99213", 1, 100.0, None);
    
//     let medicare = Medicare::new();
//     let uhg = UnitedHealthGroup::new();
//     let anthem = Anthem::new();
    
//     let claim_medicare = create_test_claim(PayerId::Medicare, vec![service_line.clone()]);
//     let claim_uhg = create_test_claim(PayerId::UnitedHealthGroup, vec![service_line.clone()]);
//     let claim_anthem = create_test_claim(PayerId::Anthem, vec![service_line]);
    
//     let remittance_medicare = medicare.submit_claim(&claim_medicare).unwrap();
//     let remittance_uhg = uhg.submit_claim(&claim_uhg).unwrap();
//     let remittance_anthem = anthem.submit_claim(&claim_anthem).unwrap();
    
//     // All should have same calculation logic
//     let sl_medicare = &remittance_medicare.service_lines[0];
//     let sl_uhg = &remittance_uhg.service_lines[0];
//     let sl_anthem = &remittance_anthem.service_lines[0];
    
//     assert_eq!(sl_medicare.billed_amount, sl_uhg.billed_amount);
//     assert_eq!(sl_medicare.billed_amount, sl_anthem.billed_amount);
    
//     assert_eq!(sl_medicare.payer_paid_amount, sl_uhg.payer_paid_amount);
//     assert_eq!(sl_medicare.payer_paid_amount, sl_anthem.payer_paid_amount);
    
//     assert_eq!(sl_medicare.coinsurance_amount, sl_uhg.coinsurance_amount);
//     assert_eq!(sl_medicare.coinsurance_amount, sl_anthem.coinsurance_amount);
// }