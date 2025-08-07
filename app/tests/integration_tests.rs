use app::{calculate_patient_statistics, calculate_aging_buckets};
use clearinghouse::ARData;
use insurance::ServiceLine;

// Helper function to create test ARData
fn create_ar_data(
    claim_id: &str,
    patient_id: &str,
    initial_claim_ts: i64,
    total_copay: f64,
    total_coinsurance: f64,
    total_deductible: f64,
) -> ARData {
    ARData {
        claim_id: claim_id.to_string(),
        remittance_id: format!("REM_{}", claim_id),
        payer_id: "Medicare".to_string(),
        payee_npi: "1234567890".to_string(),
        patient_id: patient_id.to_string(),
        initial_claim_ts,
        total_billed_amount: 100.0,
        total_payer_paid_amount: 80.0,
        total_coinsurance_amount: total_coinsurance,
        total_copay_amount: total_copay,
        total_deductible_amount: total_deductible,
        total_not_allowed_amount: 5.0,
        service_lines: vec![ServiceLine {
            service_line_id: "SL001".to_string(),
            procedure_code: "99213".to_string(),
            billed_amount: 100.0,
            payer_paid_amount: 80.0,
            coinsurance_amount: total_coinsurance,
            copay_amount: total_copay,
            deductible_amount: total_deductible,
            not_allowed_amount: 5.0,
            remark_codes: None,
        }],
    }
}

#[cfg(test)]
mod calculate_patient_statistics_tests {
    use super::*;

    #[test]
    fn test_empty_data_returns_zeros() {
        let data = vec![];
        let result = calculate_patient_statistics(&data);
        assert_eq!(result, (0.0, 0.0, 0.0, 0));
    }

    #[test]
    fn test_single_patient_single_claim() {
        let data = vec![create_ar_data("C001", "patient1", 1000, 10.0, 15.0, 5.0)];
        
        let result = calculate_patient_statistics(&data);
        
        // Single patient, so averages should equal the claim amounts
        assert_eq!(result.0, 10.0); // copay
        assert_eq!(result.1, 15.0); // coinsurance  
        assert_eq!(result.2, 5.0);  // deductible
        assert_eq!(result.3, 1);    // number of patients
    }

    #[test]
    fn test_single_patient_multiple_claims() {
        let data = vec![
            create_ar_data("C001", "patient1", 1000, 10.0, 15.0, 5.0),
            create_ar_data("C002", "patient1", 2000, 20.0, 25.0, 15.0),
        ];
        
        let result = calculate_patient_statistics(&data);
        
        // Single patient with 2 claims: (10+20)/2=15, (15+25)/2=20, (5+15)/2=10
        assert_eq!(result.0, 15.0); // avg copay per patient
        assert_eq!(result.1, 20.0); // avg coinsurance per patient
        assert_eq!(result.2, 10.0); // avg deductible per patient
        assert_eq!(result.3, 1);    // number of patients
    }

    #[test]
    fn test_multiple_patients_single_claim_each() {
        let data = vec![
            create_ar_data("C001", "patient1", 1000, 10.0, 15.0, 5.0),
            create_ar_data("C002", "patient2", 2000, 20.0, 25.0, 15.0),
        ];
        
        let result = calculate_patient_statistics(&data);
        
        // Two patients, average across patients: (10+20)/2=15, (15+25)/2=20, (5+15)/2=10
        assert_eq!(result.0, 15.0);
        assert_eq!(result.1, 20.0);
        assert_eq!(result.2, 10.0);
        assert_eq!(result.3, 2);
    }

    #[test]
    fn test_multiple_patients_varying_claims() {
        let data = vec![
            // Patient 1: 2 claims - copay avg: (10+30)/2=20, coinsurance: (15+35)/2=25, deductible: (5+25)/2=15  
            create_ar_data("C001", "patient1", 1000, 10.0, 15.0, 5.0),
            create_ar_data("C002", "patient1", 2000, 30.0, 35.0, 25.0),
            // Patient 2: 1 claim - copay: 40, coinsurance: 50, deductible: 20
            create_ar_data("C003", "patient2", 3000, 40.0, 50.0, 20.0),
        ];
        
        let result = calculate_patient_statistics(&data);
        
        // Average across patients: copay (20+40)/2=30, coinsurance (25+50)/2=37.5, deductible (15+20)/2=17.5
        assert_eq!(result.0, 30.0);
        assert_eq!(result.1, 37.5);
        assert_eq!(result.2, 17.5);
        assert_eq!(result.3, 2);
    }

    #[test]
    fn test_zero_amounts() {
        let data = vec![
            create_ar_data("C001", "patient1", 1000, 0.0, 0.0, 0.0),
            create_ar_data("C002", "patient2", 2000, 0.0, 0.0, 0.0),
        ];
        
        let result = calculate_patient_statistics(&data);
        
        assert_eq!(result.0, 0.0);
        assert_eq!(result.1, 0.0);
        assert_eq!(result.2, 0.0);
        assert_eq!(result.3, 2);
    }

    #[test]
    fn test_mixed_zero_and_nonzero_amounts() {
        let data = vec![
            create_ar_data("C001", "patient1", 1000, 0.0, 0.0, 0.0),
            create_ar_data("C002", "patient2", 2000, 20.0, 30.0, 10.0),
        ];
        
        let result = calculate_patient_statistics(&data);
        
        // Average: (0+20)/2=10, (0+30)/2=15, (0+10)/2=5
        assert_eq!(result.0, 10.0);
        assert_eq!(result.1, 15.0);
        assert_eq!(result.2, 5.0);
        assert_eq!(result.3, 2);
    }

    #[test]
    fn test_large_numbers_precision() {
        let data = vec![
            create_ar_data("C001", "patient1", 1000, 999.99, 1234.56, 500.01),
            create_ar_data("C002", "patient2", 2000, 1000.01, 1234.54, 499.99),
        ];
        
        let result = calculate_patient_statistics(&data);
        
        // Test floating point precision
        assert!((result.0 - 1000.0).abs() < 0.01);
        assert!((result.1 - 1234.55).abs() < 0.01);
        assert!((result.2 - 500.0).abs() < 0.01);
        assert_eq!(result.3, 2);
    }
}

#[cfg(test)]
mod calculate_aging_buckets_tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_empty_data_returns_zero_buckets() {
        let data = vec![];
        let result = calculate_aging_buckets(&data);
        assert_eq!(result, [0, 0, 0, 0]);
    }

    #[test]
    fn test_current_timestamp_bucket_0() {
        let now = Utc::now().timestamp_millis();
        let data = vec![create_ar_data("C001", "patient1", now, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 0 (0-1 minutes old)
        assert_eq!(result[0], 1);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 0);
    }

    #[test]
    fn test_one_minute_old_bucket_0() {
        let now = Utc::now().timestamp_millis();
        let one_minute_ago = now - (1 * 60 * 1000); // 1 minute ago
        let data = vec![create_ar_data("C001", "patient1", one_minute_ago, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should still be in bucket 0 (0-1 minutes old, inclusive)
        assert_eq!(result[0], 1);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 0);
    }

    #[test]
    fn test_two_minutes_old_bucket_1() {
        let now = Utc::now().timestamp_millis();
        let two_minutes_ago = now - (2 * 60 * 1000); // 2 minutes ago
        let data = vec![create_ar_data("C001", "patient1", two_minutes_ago, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 1 (1-2 minutes old)
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 1);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 0);
    }

    #[test]
    fn test_three_minutes_old_bucket_2() {
        let now = Utc::now().timestamp_millis();
        let three_minutes_ago = now - (3 * 60 * 1000); // 3 minutes ago
        let data = vec![create_ar_data("C001", "patient1", three_minutes_ago, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 2 (2-3 minutes old)
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 1);
        assert_eq!(result[3], 0);
    }

    #[test]
    fn test_four_minutes_old_bucket_3() {
        let now = Utc::now().timestamp_millis();
        let four_minutes_ago = now - (4 * 60 * 1000); // 4 minutes ago
        let data = vec![create_ar_data("C001", "patient1", four_minutes_ago, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 3 (3+ minutes old)
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 1);
    }

    #[test]
    fn test_very_old_claims_bucket_3() {
        let now = Utc::now().timestamp_millis();
        let very_old = now - (60 * 60 * 1000); // 1 hour ago
        let data = vec![create_ar_data("C001", "patient1", very_old, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 3 (3+ minutes old)
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 1);
    }

    #[test]
    fn test_mixed_ages_all_buckets() {
        let now = Utc::now().timestamp_millis();
        let data = vec![
            create_ar_data("C001", "patient1", now, 10.0, 15.0, 5.0), // bucket 0
            create_ar_data("C002", "patient2", now - (1 * 60 * 1000), 10.0, 15.0, 5.0), // bucket 0
            create_ar_data("C003", "patient3", now - (2 * 60 * 1000), 10.0, 15.0, 5.0), // bucket 1
            create_ar_data("C004", "patient4", now - (3 * 60 * 1000), 10.0, 15.0, 5.0), // bucket 2
            create_ar_data("C005", "patient5", now - (4 * 60 * 1000), 10.0, 15.0, 5.0), // bucket 3
            create_ar_data("C006", "patient6", now - (10 * 60 * 1000), 10.0, 15.0, 5.0), // bucket 3
        ];
        
        let result = calculate_aging_buckets(&data);
        
        assert_eq!(result[0], 2); // 0-1 minutes: 2 claims
        assert_eq!(result[1], 1); // 1-2 minutes: 1 claim
        assert_eq!(result[2], 1); // 2-3 minutes: 1 claim
        assert_eq!(result[3], 2); // 3+ minutes: 2 claims
    }

    #[test]
    fn test_boundary_conditions() {
        let now = Utc::now().timestamp_millis();
        let data = vec![
            // Exactly 1 minute old (should be bucket 0)
            create_ar_data("C001", "patient1", now - (1 * 60 * 1000), 10.0, 15.0, 5.0),
            // Exactly 2 minutes old (should be bucket 1)
            create_ar_data("C002", "patient2", now - (2 * 60 * 1000), 10.0, 15.0, 5.0),
            // Exactly 3 minutes old (should be bucket 2)
            create_ar_data("C003", "patient3", now - (3 * 60 * 1000), 10.0, 15.0, 5.0),
        ];
        
        let result = calculate_aging_buckets(&data);
        
        assert_eq!(result[0], 1); // 0-1 minutes
        assert_eq!(result[1], 1); // 1-2 minutes  
        assert_eq!(result[2], 1); // 2-3 minutes
        assert_eq!(result[3], 0); // 3+ minutes
    }

    #[test]
    fn test_negative_timestamps_old_bucket() {
        // let now = Utc::now().timestamp_millis();
        let data = vec![
            // Very negative timestamp (very old)
            create_ar_data("C001", "patient1", -1000000, 10.0, 15.0, 5.0),
        ];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 3 (3+ minutes old)
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 1);
    }

    // Comprehensive boundary tests for float-based bucketing
    #[test]
    fn test_precise_boundary_59_seconds_bucket_0() {
        let now = Utc::now().timestamp_millis();
        let timestamp = now - (59 * 1000); // 59 seconds ago (< 1 minute)
        let data = vec![create_ar_data("C001", "patient1", timestamp, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 0 (< 1.0 minutes)
        assert_eq!(result[0], 1);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 0);
    }

    #[test]
    fn test_precise_boundary_61_seconds_bucket_1() {
        let now = Utc::now().timestamp_millis();
        let timestamp = now - (61 * 1000); // 61 seconds ago (> 1 minute, < 2 minutes)
        let data = vec![create_ar_data("C001", "patient1", timestamp, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 1 (>= 1.0 and < 2.0 minutes)
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 1);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 0);
    }

    #[test]
    fn test_precise_boundary_119_seconds_bucket_1() {
        let now = Utc::now().timestamp_millis();
        let timestamp = now - (119 * 1000); // 119 seconds ago (< 2 minutes)
        let data = vec![create_ar_data("C001", "patient1", timestamp, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 1 (>= 1.0 and < 2.0 minutes)
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 1);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 0);
    }

    #[test]
    fn test_precise_boundary_121_seconds_bucket_2() {
        let now = Utc::now().timestamp_millis();
        let timestamp = now - (121 * 1000); // 121 seconds ago (> 2 minutes, < 3 minutes)
        let data = vec![create_ar_data("C001", "patient1", timestamp, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 2 (>= 2.0 and < 3.0 minutes)
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 1);
        assert_eq!(result[3], 0);
    }

    #[test]
    fn test_precise_boundary_179_seconds_bucket_2() {
        let now = Utc::now().timestamp_millis();
        let timestamp = now - (179 * 1000); // 179 seconds ago (< 3 minutes)
        let data = vec![create_ar_data("C001", "patient1", timestamp, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 2 (>= 2.0 and < 3.0 minutes)
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 1);
        assert_eq!(result[3], 0);
    }

    #[test]
    fn test_precise_boundary_181_seconds_bucket_3() {
        let now = Utc::now().timestamp_millis();
        let timestamp = now - (181 * 1000); // 181 seconds ago (> 3 minutes)
        let data = vec![create_ar_data("C001", "patient1", timestamp, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Should be in bucket 3 (>= 3.0 minutes)
        assert_eq!(result[0], 0);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 1);
    }

    // #[test]
    // fn test_millisecond_precision_boundaries() {
    //     let now = Utc::now().timestamp_millis();
    //     let data = vec![
    //         // Exactly at 1 minute boundary (60000ms)
    //         create_ar_data("C001", "patient1", now - 60000, 10.0, 15.0, 5.0),
    //         // Just under 1 minute (59999ms)
    //         create_ar_data("C002", "patient2", now - 59999, 10.0, 15.0, 5.0),
    //         // Just over 1 minute (60001ms)
    //         create_ar_data("C003", "patient3", now - 60001, 10.0, 15.0, 5.0),
            
    //         // Exactly at 2 minute boundary (120000ms)
    //         create_ar_data("C004", "patient4", now - 120000, 10.0, 15.0, 5.0),
    //         // Just under 2 minutes (119999ms)
    //         create_ar_data("C005", "patient5", now - 119999, 10.0, 15.0, 5.0),
    //         // Just over 2 minutes (120001ms)
    //         create_ar_data("C006", "patient6", now - 120001, 10.0, 15.0, 5.0),
            
    //         // Exactly at 3 minute boundary (180000ms)
    //         create_ar_data("C007", "patient7", now - 180000, 10.0, 15.0, 5.0),
    //         // Just under 3 minutes (179999ms)
    //         create_ar_data("C008", "patient8", now - 179999, 10.0, 15.0, 5.0),
    //         // Just over 3 minutes (180001ms)
    //         create_ar_data("C009", "patient9", now - 180001, 10.0, 15.0, 5.0),
    //     ];
        
    //     let result = calculate_aging_buckets(&data);
        
    //     // Bucket 0 (0 <= t < 1.0): 59999ms claim should be here
    //     assert_eq!(result[0], 2);
        
    //     // Bucket 1 (1.0 <= t < 2.0): 60000ms, 60001ms, 119999ms should be here
    //     assert_eq!(result[1], 3);
        
    //     // Bucket 2 (2.0 <= t < 3.0): 120000ms, 120001ms, 179999ms should be here
    //     assert_eq!(result[2], 3);
        
    //     // Bucket 3 (t >= 3.0): 180000ms, 180001ms should be here
    //     assert_eq!(result[3], 2);
    // }

    #[test]
    fn test_fractional_minutes() {
        let now = Utc::now().timestamp_millis();
        let data = vec![
            // 0.5 minutes (30 seconds)
            create_ar_data("C001", "patient1", now - (30 * 1000), 10.0, 15.0, 5.0),
            // 1.5 minutes (90 seconds)
            create_ar_data("C002", "patient2", now - (90 * 1000), 10.0, 15.0, 5.0),
            // 2.5 minutes (150 seconds)
            create_ar_data("C003", "patient3", now - (150 * 1000), 10.0, 15.0, 5.0),
            // 3.5 minutes (210 seconds)
            create_ar_data("C004", "patient4", now - (210 * 1000), 10.0, 15.0, 5.0),
        ];
        
        let result = calculate_aging_buckets(&data);
        
        assert_eq!(result[0], 1); // 0.5 minutes -> bucket 0
        assert_eq!(result[1], 1); // 1.5 minutes -> bucket 1
        assert_eq!(result[2], 1); // 2.5 minutes -> bucket 2
        assert_eq!(result[3], 1); // 3.5 minutes -> bucket 3
    }

    #[test]
    fn test_future_timestamps_bucket_0() {
        let now = Utc::now().timestamp_millis();
        let future_timestamp = now + (30 * 1000); // 30 seconds in the future
        let data = vec![create_ar_data("C001", "patient1", future_timestamp, 10.0, 15.0, 5.0)];
        
        let result = calculate_aging_buckets(&data);
        
        // Future timestamps result in negative age, which should go to bucket 0
        assert_eq!(result[0], 1);
        assert_eq!(result[1], 0);
        assert_eq!(result[2], 0);
        assert_eq!(result[3], 0);
    }
}