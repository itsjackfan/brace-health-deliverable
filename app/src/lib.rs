use std::collections::HashMap;
use clearinghouse::ARData;

use chrono;

pub fn calculate_patient_statistics(data: &Vec<ARData>) -> (f64, f64, f64, usize) {
    if data.is_empty() {
        return (0.0, 0.0, 0.0, 0);
    }
    
    let mut patient_totals: HashMap<String, (f64, f64, f64, usize)> = HashMap::new();
    
    for ar in data.iter() {
        let entry = patient_totals.entry(ar.patient_id.clone()).or_insert((0.0, 0.0, 0.0, 0));
        entry.0 += ar.total_copay_amount;
        entry.1 += ar.total_coinsurance_amount;
        entry.2 += ar.total_deductible_amount;
        entry.3 += 1;
    }
    
    let num_patients = patient_totals.len();
    if num_patients == 0 {
        return (0.0, 0.0, 0.0, 0);
    }
    
    let (total_copay, total_coinsurance, total_deductible): (f64, f64, f64) = patient_totals
        .values()
        .fold((0.0, 0.0, 0.0), |acc, &(copay, coinsurance, deductible, claims)| {
            let avg_copay = copay / claims as f64;
            let avg_coinsurance = coinsurance / claims as f64;
            let avg_deductible = deductible / claims as f64;
            (acc.0 + avg_copay, acc.1 + avg_coinsurance, acc.2 + avg_deductible)
        });
    
    (
        total_copay / num_patients as f64,
        total_coinsurance / num_patients as f64,
        total_deductible / num_patients as f64,
        num_patients
    )
}

pub fn calculate_aging_buckets(data: &Vec<ARData>) -> [u32; 4] {
    let now = chrono::Utc::now().timestamp_millis();
    let mut buckets = [0u32; 4];
    
    for ar in data.iter() {
        let minutes_old = (now - ar.initial_claim_ts) as f64 / (1000.0 * 60.0);
        let bucket_idx = match minutes_old {
            x if x <= 1.0 => 0,
            x if x <= 2.0 => 1,
            x if x <= 3.0 => 2,
            _ => 3,
        };
        buckets[bucket_idx] += 1;
    }

    buckets
}