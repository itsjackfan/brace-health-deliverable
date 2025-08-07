pub mod types;

pub use types::{Remittance, ServiceLine};
use intake::{PayerClaim, PayerId};
use uuid::Uuid;
use std::thread;
use std::time::Duration;

pub struct Medicare {
    pub min_response_time_secs: u64,
    pub max_response_time_secs: u64,
}

impl Medicare {
    pub fn new() -> Self {
        Self { min_response_time_secs: 10, max_response_time_secs: 30 }
    }
}

pub struct UnitedHealthGroup {
    pub min_response_time_secs: u64,
    pub max_response_time_secs: u64,
}

impl UnitedHealthGroup {
    pub fn new() -> Self {
        Self { min_response_time_secs: 10, max_response_time_secs: 30 }
    }
}

pub struct Anthem {
    pub min_response_time_secs: u64,
    pub max_response_time_secs: u64,
}

impl Anthem {
    pub fn new() -> Self {
        Self { min_response_time_secs: 10, max_response_time_secs: 30 }
    }
}

pub trait Insurance {
    fn submit_claim(&self, claim: &PayerClaim) -> Result<Remittance, String>;
}

impl Insurance for Medicare {
    fn submit_claim(&self, claim: &PayerClaim) -> Result<Remittance, String> {
        let mut service_lines = Vec::new();

        for line in &claim.service_lines {
            let billed_amount = line.unit_charge_amount * line.units as f64;
            
            // Medicare denial rate: 5-10% for services not meeting guidelines
            let denial_rate = 0.05 + (rand::random::<f64>() * 0.05); // 5-10%
            let not_allowed_amount = billed_amount * denial_rate;
            let allowed_amount = billed_amount - not_allowed_amount;
            
            // Medicare Part B 2025 deductible: $257 per year (applied to allowed amount)
            let deductible_amount = if allowed_amount > 257.0 { 257.0 } else { allowed_amount };
            let remaining_after_deductible = allowed_amount - deductible_amount;
            
            // Medicare Part B standard: 80% coverage, 20% coinsurance after deductible
            let payer_paid_amount = remaining_after_deductible * 0.8;
            let coinsurance_amount = remaining_after_deductible * 0.2;
            
            // Medicare Part B typically doesn't use copays for physician services
            let copay_amount = 0.0;

            let service_line = ServiceLine::new(
                line, 
                billed_amount, 
                payer_paid_amount, 
                coinsurance_amount, 
                copay_amount, 
                deductible_amount, 
                not_allowed_amount
            )?;
            
            service_lines.push(service_line);
        }

        // random sleep because insurance is slow
        let sleep_duration = rand::random_range(self.min_response_time_secs..=self.max_response_time_secs);
        thread::sleep(Duration::from_secs(sleep_duration));

        let remittance = create_remittance(service_lines, claim);
        Ok(remittance)
    }
}

impl Insurance for UnitedHealthGroup {
    fn submit_claim(&self, claim: &PayerClaim) -> Result<Remittance, String> {
        let mut service_lines = Vec::new();

        for line in &claim.service_lines {
            let billed_amount = line.unit_charge_amount * line.units as f64;
            
            // Private insurers typically have lower denial rates: 3-7%
            let denial_rate = 0.03 + (rand::random::<f64>() * 0.04); // 3-7%
            let not_allowed_amount = billed_amount * denial_rate;
            let allowed_amount = billed_amount - not_allowed_amount;
            
            // UnitedHealth average individual deductible: ~$1,800 (applied to allowed amount)
            let deductible_amount = if allowed_amount > 1800.0 { 1800.0 } else { allowed_amount };
            let remaining_after_deductible = allowed_amount - deductible_amount;
            
            // UnitedHealth typical copay for routine services: $25-35
            let copay_base = 25.0 + (rand::random::<f64>() * 10.0); // $25-35
            let copay_amount = if remaining_after_deductible > copay_base { copay_base } else { 0.0 };
            let remaining_after_copay = remaining_after_deductible - copay_amount;
            
            // UnitedHealth typical coverage: 75% (between 70-80% range)
            // Patient coinsurance: 25% (typical private insurance 20-30% range)
            let coverage_rate = 0.70 + (rand::random::<f64>() * 0.1); // 70-80%
            let payer_paid_amount = remaining_after_copay * coverage_rate;
            let coinsurance_amount = remaining_after_copay * (1.0 - coverage_rate);

            let service_line = ServiceLine::new(
                line, 
                billed_amount, 
                payer_paid_amount, 
                coinsurance_amount, 
                copay_amount, 
                deductible_amount, 
                not_allowed_amount
            )?;
            
            service_lines.push(service_line);
        }

        // random sleep because insurance is slow
        let sleep_duration = rand::random_range(self.min_response_time_secs..=self.max_response_time_secs);
        thread::sleep(Duration::from_secs(sleep_duration));

        let remittance = create_remittance(service_lines, claim);
        Ok(remittance)
    }
}

impl Insurance for Anthem {
    fn submit_claim(&self, claim: &PayerClaim) -> Result<Remittance, String> {
        let mut service_lines = Vec::new();

        for line in &claim.service_lines {
            let billed_amount = line.unit_charge_amount * line.units as f64;
            
            // Anthem denial rate: 5-8% (moderate for private insurer)
            let denial_rate = 0.05 + (rand::random::<f64>() * 0.03); // 5-8%
            let not_allowed_amount = billed_amount * denial_rate;
            let allowed_amount = billed_amount - not_allowed_amount;
            
            // Anthem average individual deductible: ~$1,650-2,000 (applied to allowed amount)
            let deductible_base = 1650.0 + (rand::random::<f64>() * 350.0); // $1,650-2,000
            let deductible_amount = if allowed_amount > deductible_base { deductible_base } else { allowed_amount };
            let remaining_after_deductible = allowed_amount - deductible_amount;
            
            // Anthem typical copay for routine services: $20-30
            let copay_base = 20.0 + (rand::random::<f64>() * 10.0); // $20-30
            let copay_amount = if remaining_after_deductible > copay_base { copay_base } else { 0.0 };
            let remaining_after_copay = remaining_after_deductible - copay_amount;
            
            // Anthem Silver plan structure: 70% coverage, 30% coinsurance
            // This is based on typical Anthem Silver plan coinsurance rates
            let payer_paid_amount = remaining_after_copay * 0.7;
            let coinsurance_amount = remaining_after_copay * 0.3;

            let service_line = ServiceLine::new(
                line, 
                billed_amount, 
                payer_paid_amount, 
                coinsurance_amount, 
                copay_amount, 
                deductible_amount, 
                not_allowed_amount
            )?;
            
            service_lines.push(service_line);
        }

        // random sleep because insurance is slow
        let sleep_duration = rand::random_range(self.min_response_time_secs..=self.max_response_time_secs);
        thread::sleep(Duration::from_secs(sleep_duration));

        let remittance = create_remittance(service_lines, claim);
        Ok(remittance)
    }
}

pub fn create_remittance(service_lines: Vec<ServiceLine>, claim: &PayerClaim) -> Remittance {
    let claim_id = claim.claim_id.clone();
    let payer_id = match claim.insurance.payer_id {
        PayerId::Medicare => "Medicare".to_string(),
        PayerId::UnitedHealthGroup => "UnitedHealthGroup".to_string(),
        PayerId::Anthem => "Anthem".to_string(),
    };
    let patient_id = format!("{}-{}", payer_id, claim.insurance.patient_member_id);
    let payee_npi = claim.organization.billing_npi.clone().unwrap_or("".to_string());
    let remittance_id = format!("{}", Uuid::new_v4());

    Remittance {
        remittance_id: remittance_id,
        claim_id: claim_id,
        payer_id: payer_id,
        payee_npi: payee_npi,
        patient_id: patient_id,
        service_lines: service_lines,
        initial_claim_ts: claim.initial_claim_ts,
    }
}