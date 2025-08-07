use clearinghouse::*;
use intake::*;
use insurance;

// Helper function to create a valid test claim
fn create_valid_test_claim() -> PayerClaim {
    PayerClaim {
        claim_id: "CLAIM001".to_string(),
        place_of_service_code: 11,
        insurance: Insurance {
            payer_id: PayerId::Medicare,
            patient_member_id: "MED123456".to_string(),
        },
        patient: Patient {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            gender: Gender::Male,
            dob: "1980-01-15".to_string(),
            email: Some("john.doe@test.com".to_string()),
            address: Some(Address {
                street: Some("123 Main St".to_string()),
                city: Some("Anytown".to_string()),
                state: Some("CA".to_string()),
                zip: Some("90210".to_string()),
                country: Some("US".to_string()),
            }),
        },
        organization: Organization {
            name: "Test Medical Practice".to_string(),
            billing_npi: Some("1234567890".to_string()),
            ein: Some("12-3456789".to_string()),
            contact: Some(Contact {
                first_name: Some("Jane".to_string()),
                last_name: Some("Smith".to_string()),
                phone_number: Some("555-0123".to_string()),
            }),
            address: Some(Address {
                street: Some("456 Business Ave".to_string()),
                city: Some("Anytown".to_string()),
                state: Some("CA".to_string()),
                zip: Some("90210-1234".to_string()),
                country: Some("US".to_string()),
            }),
        },
        rendering_provider: RenderingProvider {
            first_name: "Dr. Alice".to_string(),
            last_name: "Johnson".to_string(),
            npi: "9876543210".to_string(),
        },
        service_lines: vec![
            ServiceLine {
                service_line_id: "SL001".to_string(),
                procedure_code: "99213".to_string(),
                modifiers: Some(vec!["25".to_string()]),
                units: 1,
                details: "Office visit".to_string(),
                unit_charge_currency: "USD".to_string(),
                unit_charge_amount: 150.00,
                do_not_bill: Some(false),
            }
        ],
        initial_claim_ts: 1640995200000,
    }
}

// Helper function to create a test remittance
fn create_test_remittance() -> insurance::Remittance {
    insurance::Remittance {
        remittance_id: "REM123".to_string(),
        claim_id: "CLAIM001".to_string(),
        payer_id: "Medicare".to_string(),
        payee_npi: "1234567890".to_string(),
        patient_id: "Medicare-MED123456".to_string(),
        service_lines: vec![
            insurance::ServiceLine {
                service_line_id: "SL001".to_string(),
                procedure_code: "99213".to_string(),
                billed_amount: 150.0,
                payer_paid_amount: 120.0,
                coinsurance_amount: 15.0,
                copay_amount: 7.5,
                deductible_amount: 7.5,
                not_allowed_amount: 0.0,
                remark_codes: None,
            }
        ],
        initial_claim_ts: 1640995200000,
    }
}

#[test]
fn test_validate_claim_success() {
    let claim = create_valid_test_claim();
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

// Tests for validate_non_empty_fields
#[test]
fn test_validate_non_empty_fields_success() {
    let claim = create_valid_test_claim();
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

#[test]
fn test_validate_empty_claim_id() {
    let mut claim = create_valid_test_claim();
    claim.claim_id = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("claim_id cannot be empty"));
}

#[test]
fn test_validate_whitespace_only_claim_id() {
    let mut claim = create_valid_test_claim();
    claim.claim_id = "   \t  \n ".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("claim_id cannot be empty"));
}

#[test]
fn test_validate_empty_patient_first_name() {
    let mut claim = create_valid_test_claim();
    claim.patient.first_name = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("patient.first_name cannot be empty"));
}

#[test]
fn test_validate_empty_patient_last_name() {
    let mut claim = create_valid_test_claim();
    claim.patient.last_name = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("patient.last_name cannot be empty"));
}

#[test]
fn test_validate_empty_patient_dob() {
    let mut claim = create_valid_test_claim();
    claim.patient.dob = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("patient.dob cannot be empty"));
}

#[test]
fn test_validate_empty_organization_name() {
    let mut claim = create_valid_test_claim();
    claim.organization.name = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("organization.name cannot be empty"));
}

#[test]
fn test_validate_empty_rendering_provider_first_name() {
    let mut claim = create_valid_test_claim();
    claim.rendering_provider.first_name = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("rendering_provider.first_name cannot be empty"));
}

#[test]
fn test_validate_empty_rendering_provider_last_name() {
    let mut claim = create_valid_test_claim();
    claim.rendering_provider.last_name = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("rendering_provider.last_name cannot be empty"));
}

#[test]
fn test_validate_empty_rendering_provider_npi() {
    let mut claim = create_valid_test_claim();
    claim.rendering_provider.npi = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("rendering_provider.npi cannot be empty"));
}

#[test]
fn test_validate_empty_insurance_patient_member_id() {
    let mut claim = create_valid_test_claim();
    claim.insurance.patient_member_id = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("insurance.patient_member_id cannot be empty"));
}

#[test]
fn test_validate_empty_service_line_id() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].service_line_id = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].service_line_id cannot be empty"));
}

#[test]
fn test_validate_empty_procedure_code() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].procedure_code = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].procedure_code cannot be empty"));
}

#[test]
fn test_validate_empty_service_details() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].details = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].details cannot be empty"));
}

#[test]
fn test_validate_empty_currency() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].unit_charge_currency = "".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].unit_charge_currency cannot be empty"));
}

// Tests for validate_formats
#[test]
fn test_validate_rendering_provider_npi_valid() {
    let mut claim = create_valid_test_claim();
    claim.rendering_provider.npi = "1111111111".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

#[test]
fn test_validate_rendering_provider_npi_too_short() {
    let mut claim = create_valid_test_claim();
    claim.rendering_provider.npi = "123456789".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("rendering_provider.npi must be exactly 10 digits"));
}

#[test]
fn test_validate_rendering_provider_npi_too_long() {
    let mut claim = create_valid_test_claim();
    claim.rendering_provider.npi = "12345678901".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("rendering_provider.npi must be exactly 10 digits"));
}

#[test]
fn test_validate_rendering_provider_npi_non_numeric() {
    let mut claim = create_valid_test_claim();
    claim.rendering_provider.npi = "123456789A".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("rendering_provider.npi must be exactly 10 digits"));
}

#[test]
fn test_validate_organization_billing_npi_valid() {
    let mut claim = create_valid_test_claim();
    claim.organization.billing_npi = Some("2222222222".to_string());
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

#[test]
fn test_validate_organization_billing_npi_invalid() {
    let mut claim = create_valid_test_claim();
    claim.organization.billing_npi = Some("987654321".to_string());
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("organization.billing_npi must be exactly 10 digits"));
}

#[test]
fn test_validate_organization_billing_npi_none() {
    let mut claim = create_valid_test_claim();
    claim.organization.billing_npi = None;
    let result = validate_claim(&claim);
    assert!(result.is_ok()); // Should be valid when None
}

#[test]
fn test_validate_ein_valid() {
    let mut claim = create_valid_test_claim();
    claim.organization.ein = Some("12-3456789".to_string());
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

#[test]
fn test_validate_ein_invalid_format() {
    let mut claim = create_valid_test_claim();
    claim.organization.ein = Some("123456789".to_string());
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("organization.ein must match format XX-XXXXXXX"));
}

#[test]
fn test_validate_ein_wrong_dash_position() {
    let mut claim = create_valid_test_claim();
    claim.organization.ein = Some("123-456789".to_string());
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("organization.ein must match format XX-XXXXXXX"));
}

#[test]
fn test_validate_ein_non_numeric() {
    let mut claim = create_valid_test_claim();
    claim.organization.ein = Some("1A-3456789".to_string());
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("organization.ein must match format XX-XXXXXXX"));
}

#[test]
fn test_validate_ein_none() {
    let mut claim = create_valid_test_claim();
    claim.organization.ein = None;
    let result = validate_claim(&claim);
    assert!(result.is_ok()); // Should be valid when None
}

#[test]
fn test_validate_zip_code_5_digits() {
    let mut claim = create_valid_test_claim();
    if let Some(ref mut address) = claim.patient.address {
        address.zip = Some("90210".to_string());
    }
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

#[test]
fn test_validate_zip_code_9_digits() {
    let mut claim = create_valid_test_claim();
    if let Some(ref mut address) = claim.patient.address {
        address.zip = Some("90210-1234".to_string());
    }
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

#[test]
fn test_validate_zip_code_invalid_length() {
    let mut claim = create_valid_test_claim();
    if let Some(ref mut address) = claim.patient.address {
        address.zip = Some("9021".to_string());
    }
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("patient.address.zip must be XXXXX or XXXXX-XXXX format"));
}

#[test]
fn test_validate_zip_code_non_numeric() {
    let mut claim = create_valid_test_claim();
    if let Some(ref mut address) = claim.patient.address {
        address.zip = Some("9021A".to_string());
    }
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("patient.address.zip must be XXXXX or XXXXX-XXXX format"));
}

#[test]
fn test_validate_zip_code_none() {
    let mut claim = create_valid_test_claim();
    if let Some(ref mut address) = claim.patient.address {
        address.zip = None;
    }
    let result = validate_claim(&claim);
    assert!(result.is_ok()); // Should be valid when None
}

#[test]
fn test_validate_currency_valid() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].unit_charge_currency = "USD".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

#[test]
fn test_validate_currency_too_short() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].unit_charge_currency = "US".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].unit_charge_currency must be 3 uppercase letters"));
}

#[test]
fn test_validate_currency_too_long() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].unit_charge_currency = "USDD".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].unit_charge_currency must be 3 uppercase letters"));
}

#[test]
fn test_validate_currency_lowercase() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].unit_charge_currency = "usd".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].unit_charge_currency must be 3 uppercase letters"));
}

#[test]
fn test_validate_currency_mixed_case() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].unit_charge_currency = "Usd".to_string();
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].unit_charge_currency must be 3 uppercase letters"));
}

// Tests for validate_business_rules
#[test]
fn test_validate_place_of_service_code_valid_min() {
    let mut claim = create_valid_test_claim();
    claim.place_of_service_code = 1;
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

#[test]
fn test_validate_place_of_service_code_valid_max() {
    let mut claim = create_valid_test_claim();
    claim.place_of_service_code = 99;
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

#[test]
fn test_validate_place_of_service_code_too_low() {
    let mut claim = create_valid_test_claim();
    claim.place_of_service_code = 0;
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("place_of_service_code must be between 1-99"));
}

#[test]
fn test_validate_place_of_service_code_too_high() {
    let mut claim = create_valid_test_claim();
    claim.place_of_service_code = 100;
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("place_of_service_code must be between 1-99"));
}

#[test]
fn test_validate_empty_service_lines() {
    let mut claim = create_valid_test_claim();
    claim.service_lines = vec![];
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines must contain at least one item"));
}

#[test]
fn test_validate_duplicate_service_line_ids() {
    let mut claim = create_valid_test_claim();
    claim.service_lines = vec![
        ServiceLine {
            service_line_id: "SL001".to_string(),
            procedure_code: "99213".to_string(),
            modifiers: None,
            units: 1,
            details: "Office visit".to_string(),
            unit_charge_currency: "USD".to_string(),
            unit_charge_amount: 150.00,
            do_not_bill: None,
        },
        ServiceLine {
            service_line_id: "SL001".to_string(), // Duplicate ID
            procedure_code: "99214".to_string(),
            modifiers: None,
            units: 1,
            details: "Follow-up visit".to_string(),
            unit_charge_currency: "USD".to_string(),
            unit_charge_amount: 200.00,
            do_not_bill: None,
        },
    ];
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Duplicate service_line_id: SL001"));
}

#[test]
fn test_validate_units_zero() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].units = 0;
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].units must be at least 1"));
}

#[test]
fn test_validate_units_negative() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].units = -1;
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].units must be at least 1"));
}

#[test]
fn test_validate_unit_charge_amount_zero() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].unit_charge_amount = 0.0;
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].unit_charge_amount must be positive"));
}

#[test]
fn test_validate_unit_charge_amount_negative() {
    let mut claim = create_valid_test_claim();
    claim.service_lines[0].unit_charge_amount = -100.0;
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("service_lines[0].unit_charge_amount must be positive"));
}

#[test]
fn test_validate_currency_consistency() {
    let mut claim = create_valid_test_claim();
    claim.service_lines = vec![
        ServiceLine {
            service_line_id: "SL001".to_string(),
            procedure_code: "99213".to_string(),
            modifiers: None,
            units: 1,
            details: "Office visit".to_string(),
            unit_charge_currency: "USD".to_string(),
            unit_charge_amount: 150.00,
            do_not_bill: None,
        },
        ServiceLine {
            service_line_id: "SL002".to_string(),
            procedure_code: "99214".to_string(),
            modifiers: None,
            units: 1,
            details: "Follow-up visit".to_string(),
            unit_charge_currency: "EUR".to_string(), // Different currency
            unit_charge_amount: 200.00,
            do_not_bill: None,
        },
    ];
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("All service lines must use the same currency"));
}

#[test]
fn test_validate_provider_npi_equals_organization_npi() {
    let mut claim = create_valid_test_claim();
    claim.rendering_provider.npi = "1234567890".to_string();
    claim.organization.billing_npi = Some("1234567890".to_string()); // Same NPI
    let result = validate_claim(&claim);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("rendering_provider.npi cannot equal organization.billing_npi"));
}

#[test]
fn test_validate_provider_npi_different_from_organization_npi() {
    let mut claim = create_valid_test_claim();
    claim.rendering_provider.npi = "1234567890".to_string();
    claim.organization.billing_npi = Some("9876543210".to_string()); // Different NPI
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

#[test]
fn test_validate_multiple_service_lines_valid() {
    let mut claim = create_valid_test_claim();
    claim.service_lines = vec![
        ServiceLine {
            service_line_id: "SL001".to_string(),
            procedure_code: "99213".to_string(),
            modifiers: Some(vec!["25".to_string()]),
            units: 1,
            details: "Office visit".to_string(),
            unit_charge_currency: "USD".to_string(),
            unit_charge_amount: 150.00,
            do_not_bill: Some(false),
        },
        ServiceLine {
            service_line_id: "SL002".to_string(),
            procedure_code: "99214".to_string(),
            modifiers: Some(vec!["59".to_string()]),
            units: 2,
            details: "Follow-up visit".to_string(),
            unit_charge_currency: "USD".to_string(),
            unit_charge_amount: 200.00,
            do_not_bill: None,
        },
        ServiceLine {
            service_line_id: "SL003".to_string(),
            procedure_code: "99215".to_string(),
            modifiers: None,
            units: 1,
            details: "Complex visit".to_string(),
            unit_charge_currency: "USD".to_string(),
            unit_charge_amount: 300.00,
            do_not_bill: Some(true),
        },
    ];
    let result = validate_claim(&claim);
    assert!(result.is_ok());
}

// Tests for submit_claim_to_payer
#[test]
fn test_submit_claim_to_medicare() {
    let mut claim = create_valid_test_claim();
    claim.insurance.payer_id = PayerId::Medicare;
    
    let result = submit_claim_to_payer(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    assert_eq!(remittance.claim_id, "CLAIM001");
    assert_eq!(remittance.payer_id, "Medicare");
    assert_eq!(remittance.patient_id, "Medicare-MED123456");
    assert_eq!(remittance.payee_npi, "1234567890");
    assert_eq!(remittance.service_lines.len(), 1);
}

#[test]
fn test_submit_claim_to_united_health_group() {
    let mut claim = create_valid_test_claim();
    claim.insurance.payer_id = PayerId::UnitedHealthGroup;
    claim.insurance.patient_member_id = "UHG123456".to_string();
    
    let result = submit_claim_to_payer(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    assert_eq!(remittance.claim_id, "CLAIM001");
    assert_eq!(remittance.payer_id, "UnitedHealthGroup");
    assert_eq!(remittance.patient_id, "UnitedHealthGroup-UHG123456");
    assert_eq!(remittance.payee_npi, "1234567890");
    assert_eq!(remittance.service_lines.len(), 1);
}

#[test]
fn test_submit_claim_to_anthem() {
    let mut claim = create_valid_test_claim();
    claim.insurance.payer_id = PayerId::Anthem;
    claim.insurance.patient_member_id = "ANT123456".to_string();
    
    let result = submit_claim_to_payer(&claim);
    assert!(result.is_ok());
    
    let remittance = result.unwrap();
    assert_eq!(remittance.claim_id, "CLAIM001");
    assert_eq!(remittance.payer_id, "Anthem");
    assert_eq!(remittance.patient_id, "Anthem-ANT123456");
    assert_eq!(remittance.payee_npi, "1234567890");
    assert_eq!(remittance.service_lines.len(), 1);
}

// Tests for submit_remittance_to_submitter  
#[test]
fn test_submit_remittance_to_submitter_success() {
    let remittance = create_test_remittance();
    
    let result = submit_remittance_to_submitter(&remittance);
    assert!(result.is_ok());
    
    let ar_data = result.unwrap();
    assert_eq!(ar_data.claim_id, "CLAIM001");
    assert_eq!(ar_data.remittance_id, "REM123");
    assert_eq!(ar_data.payer_id, "Medicare");
    assert_eq!(ar_data.payee_npi, "1234567890");
    assert_eq!(ar_data.patient_id, "Medicare-MED123456");
    assert_eq!(ar_data.initial_claim_ts, 1640995200000);
    assert_eq!(ar_data.service_lines.len(), 1);
}

#[test]
fn test_submit_remittance_calculates_totals() {
    let mut remittance = create_test_remittance();
    remittance.service_lines = vec![
        insurance::ServiceLine {
            service_line_id: "SL001".to_string(),
            procedure_code: "99213".to_string(),
            billed_amount: 150.0,
            payer_paid_amount: 120.0,
            coinsurance_amount: 15.0,
            copay_amount: 7.5,
            deductible_amount: 7.5,
            not_allowed_amount: 0.0,
            remark_codes: None,
        },
        insurance::ServiceLine {
            service_line_id: "SL002".to_string(),
            procedure_code: "99214".to_string(),
            billed_amount: 400.0,
            payer_paid_amount: 320.0,
            coinsurance_amount: 40.0,
            copay_amount: 20.0,
            deductible_amount: 20.0,
            not_allowed_amount: 0.0,
            remark_codes: None,
        },
    ];
    
    let result = submit_remittance_to_submitter(&remittance);
    assert!(result.is_ok());
    
    let ar_data = result.unwrap();
    
    // Verify totals are calculated correctly
    assert_eq!(ar_data.total_billed_amount, 550.0);        // 150 + 400
    assert_eq!(ar_data.total_payer_paid_amount, 440.0);    // 120 + 320
    assert_eq!(ar_data.total_coinsurance_amount, 55.0);    // 15 + 40
    assert_eq!(ar_data.total_copay_amount, 27.5);          // 7.5 + 20
    assert_eq!(ar_data.total_deductible_amount, 27.5);     // 7.5 + 20
    assert_eq!(ar_data.total_not_allowed_amount, 0.0);     // 0 + 0
}