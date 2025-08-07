use serde::{Serialize, Deserialize};
use insurance::ServiceLine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARData {
    pub claim_id: String,
    pub remittance_id: String,
    pub payer_id: String,
    pub payee_npi: String,
    pub patient_id: String,
    pub initial_claim_ts: i64,
    pub total_billed_amount: f64,
    pub total_payer_paid_amount: f64,
    pub total_coinsurance_amount: f64,
    pub total_copay_amount: f64,
    pub total_deductible_amount: f64,
    pub total_not_allowed_amount: f64,
    pub service_lines: Vec<ServiceLine>,
}