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
            // 1. calculate amounts
            let billed_amount = line.unit_charge_amount * line.units as f64;
            let payer_paid_amount = billed_amount * 0.8;
            let coinsurance_amount = billed_amount * 0.1;
            let copay_amount = billed_amount * 0.05;
            let deductible_amount = billed_amount * 0.05;
            let not_allowed_amount = billed_amount * 0.05;

            // 2. create service line
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

        // 4. return minimalistic remittance doc
        let remittance = create_remittance(service_lines, claim);
        Ok(remittance)
    }
}

impl Insurance for UnitedHealthGroup {
    fn submit_claim(&self, claim: &PayerClaim) -> Result<Remittance, String> {
        let mut service_lines = Vec::new();

        for line in &claim.service_lines {
            // 1. calculate amounts
            let billed_amount = line.unit_charge_amount * line.units as f64;
            let payer_paid_amount = billed_amount * 0.8;
            let coinsurance_amount = billed_amount * 0.1;
            let copay_amount = billed_amount * 0.05;
            let deductible_amount = billed_amount * 0.05;
            let not_allowed_amount = billed_amount * 0.05;

            // 2. create service line
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

        // 3. return minimalistic remittance doc
        let remittance = create_remittance(service_lines, claim);
        Ok(remittance)
    }
}

impl Insurance for Anthem {
    fn submit_claim(&self, claim: &PayerClaim) -> Result<Remittance, String> {
        let mut service_lines = Vec::new();

        for line in &claim.service_lines {
            // 1. calculate amounts
            let billed_amount = line.unit_charge_amount * line.units as f64;
            let payer_paid_amount = billed_amount * 0.8;
            let coinsurance_amount = billed_amount * 0.1;
            let copay_amount = billed_amount * 0.05;
            let deductible_amount = billed_amount * 0.05;
            let not_allowed_amount = billed_amount * 0.05;

            // 2. create service line
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

        // 3. return minimalistic remittance doc
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