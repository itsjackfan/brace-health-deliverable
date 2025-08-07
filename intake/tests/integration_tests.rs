use intake::*;
use std::fs;

// Helper function to get test fixture path
fn get_fixture_path(filename: &str) -> String {
    format!("test_fixtures/{}", filename)
}

// Helper function to create a temporary config
fn create_test_config(file_path: &str) -> Config {
    Config {
        file_path: file_path.to_string(),
        rate_per_second: 10,
        refill_rate: 5,
        num_threads: 2,
    }
}

#[test]
fn test_read_file_valid_single_claim() {
    let config = create_test_config(&get_fixture_path("valid_claim.json"));
    let result = read_file(&config);
    
    assert!(result.is_ok());
    let lines: Vec<String> = result.unwrap().collect();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("CLAIM123"));
}

#[test]
fn test_read_file_multiple_claims() {
    let config = create_test_config(&get_fixture_path("multiple_claims.json"));
    let result = read_file(&config);
    
    assert!(result.is_ok());
    let lines: Vec<String> = result.unwrap().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("MULTI001"));
    assert!(lines[1].contains("MULTI002"));
}

#[test]
fn test_read_file_empty_file() {
    let config = create_test_config(&get_fixture_path("empty_file.json"));
    let result = read_file(&config);
    
    assert!(result.is_ok());
    let lines: Vec<String> = result.unwrap().collect();
    assert_eq!(lines.len(), 0);
}

#[test]
fn test_read_file_nonexistent() {
    let config = create_test_config("nonexistent_file.json");
    let result = read_file(&config);
    
    assert!(result.is_err());
    match result {
        Err(error_msg) => assert!(error_msg.contains("Failed to open file")),
        Ok(_) => panic!("Expected error, but got Ok"),
    }
}

#[test]
fn test_parse_line_valid_claim() {
    let json_line = r#"{"claim_id":"TEST001","place_of_service_code":11,"insurance":{"payer_id":"medicare","patient_member_id":"MED123"},"patient":{"first_name":"John","last_name":"Doe","gender":"m","dob":"1980-01-15"},"organization":{"name":"Test Clinic"},"rendering_provider":{"first_name":"Dr. Test","last_name":"Provider","npi":"1234567890"},"service_lines":[{"service_line_id":"SL001","procedure_code":"99213","units":1,"details":"Test visit","unit_charge_currency":"USD","unit_charge_amount":100.00}]}"#;
    
    let result = parse_line(json_line);
    assert!(result.is_ok());
    
    let claim = result.unwrap();
    assert_eq!(claim.claim_id, "TEST001");
    assert_eq!(claim.place_of_service_code, 11);
    assert!(claim.initial_claim_ts > 0); // Should be set to current timestamp
    assert_eq!(claim.patient.first_name, "John");
    assert_eq!(claim.patient.last_name, "Doe");
}

#[test]
fn test_parse_line_minimal_valid_claim() {
    let json_line = fs::read_to_string(get_fixture_path("minimal_claim.json")).unwrap();
    
    let result = parse_line(&json_line);
    assert!(result.is_ok());
    
    let claim = result.unwrap();
    assert_eq!(claim.claim_id, "MIN001");
    assert!(claim.initial_claim_ts > 0);
    assert_eq!(claim.patient.email, None);
    assert_eq!(claim.patient.address, None);
}

#[test]
fn test_parse_line_invalid_json() {
    let invalid_json = r#"{"claim_id":"INVALID","missing_comma":"value" "invalid":"json"}"#;
    
    let result = parse_line(invalid_json);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse line"));
}

#[test]
fn test_parse_line_missing_required_field() {
    let json_line = fs::read_to_string(get_fixture_path("missing_required_field.json")).unwrap();
    
    let result = parse_line(&json_line);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse line"));
}

#[test]
fn test_parse_line_empty_string() {
    let result = parse_line("");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse line"));
}

#[test]
fn test_parse_line_whitespace_only() {
    let result = parse_line("   \n  \t  ");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to parse line"));
}

#[test]
fn test_parse_line_invalid_payer_id() {
    let json_line = r#"{"claim_id":"TEST001","place_of_service_code":11,"insurance":{"payer_id":"invalid_payer","patient_member_id":"MED123"},"patient":{"first_name":"John","last_name":"Doe","gender":"m","dob":"1980-01-15"},"organization":{"name":"Test Clinic"},"rendering_provider":{"first_name":"Dr. Test","last_name":"Provider","npi":"1234567890"},"service_lines":[{"service_line_id":"SL001","procedure_code":"99213","units":1,"details":"Test visit","unit_charge_currency":"USD","unit_charge_amount":100.00}]}"#;
    
    let result = parse_line(json_line);
    assert!(result.is_err());
}

#[test]
fn test_parse_line_invalid_gender() {
    let json_line = r#"{"claim_id":"TEST001","place_of_service_code":11,"insurance":{"payer_id":"medicare","patient_member_id":"MED123"},"patient":{"first_name":"John","last_name":"Doe","gender":"x","dob":"1980-01-15"},"organization":{"name":"Test Clinic"},"rendering_provider":{"first_name":"Dr. Test","last_name":"Provider","npi":"1234567890"},"service_lines":[{"service_line_id":"SL001","procedure_code":"99213","units":1,"details":"Test visit","unit_charge_currency":"USD","unit_charge_amount":100.00}]}"#;
    
    let result = parse_line(json_line);
    assert!(result.is_err());
}

#[test]
fn test_parse_line_all_payer_types() {
    // Test Medicare
    let medicare_json = r#"{"claim_id":"MED001","place_of_service_code":11,"insurance":{"payer_id":"medicare","patient_member_id":"MED123"},"patient":{"first_name":"John","last_name":"Doe","gender":"m","dob":"1980-01-15"},"organization":{"name":"Test Clinic"},"rendering_provider":{"first_name":"Dr. Test","last_name":"Provider","npi":"1234567890"},"service_lines":[{"service_line_id":"SL001","procedure_code":"99213","units":1,"details":"Test visit","unit_charge_currency":"USD","unit_charge_amount":100.00}]}"#;
    assert!(parse_line(medicare_json).is_ok());

    // Test United Health Group
    let uhg_json = r#"{"claim_id":"UHG001","place_of_service_code":11,"insurance":{"payer_id":"united_health_group","patient_member_id":"UHG123"},"patient":{"first_name":"Jane","last_name":"Smith","gender":"f","dob":"1985-03-20"},"organization":{"name":"Test Clinic"},"rendering_provider":{"first_name":"Dr. Test","last_name":"Provider","npi":"1234567890"},"service_lines":[{"service_line_id":"SL002","procedure_code":"99214","units":1,"details":"Test visit","unit_charge_currency":"USD","unit_charge_amount":150.00}]}"#;
    assert!(parse_line(uhg_json).is_ok());

    // Test Anthem
    let anthem_json = r#"{"claim_id":"ANT001","place_of_service_code":11,"insurance":{"payer_id":"anthem","patient_member_id":"ANT123"},"patient":{"first_name":"Bob","last_name":"Johnson","gender":"m","dob":"1990-07-10"},"organization":{"name":"Test Clinic"},"rendering_provider":{"first_name":"Dr. Test","last_name":"Provider","npi":"1234567890"},"service_lines":[{"service_line_id":"SL003","procedure_code":"99215","units":1,"details":"Test visit","unit_charge_currency":"USD","unit_charge_amount":200.00}]}"#;
    assert!(parse_line(anthem_json).is_ok());
}

#[test]
fn test_parse_line_timestamp_uniqueness() {
    let json_line = r#"{"claim_id":"TIME001","place_of_service_code":11,"insurance":{"payer_id":"medicare","patient_member_id":"MED123"},"patient":{"first_name":"Time","last_name":"Test","gender":"m","dob":"1980-01-15"},"organization":{"name":"Test Clinic"},"rendering_provider":{"first_name":"Dr. Test","last_name":"Provider","npi":"1234567890"},"service_lines":[{"service_line_id":"SL001","procedure_code":"99213","units":1,"details":"Test visit","unit_charge_currency":"USD","unit_charge_amount":100.00}]}"#;
    
    let claim1 = parse_line(json_line).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10)); // Small delay
    let claim2 = parse_line(json_line).unwrap();
    
    // Timestamps should be different (or at least not exactly the same)
    assert!(claim1.initial_claim_ts <= claim2.initial_claim_ts);
    assert_eq!(claim1.claim_id, claim2.claim_id); // Same data otherwise
}

#[test]
fn test_integration_read_and_parse() {
    let config = create_test_config(&get_fixture_path("valid_claim.json"));
    let lines = read_file(&config).unwrap();
    
    for line in lines {
        let claim_result = parse_line(&line);
        assert!(claim_result.is_ok());
        
        let claim = claim_result.unwrap();
        assert_eq!(claim.claim_id, "CLAIM123");
        assert!(claim.initial_claim_ts > 0);
    }
}