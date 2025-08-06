use serde::{Serialize, Deserialize};
use intake::ServiceLine as IntakeServiceLine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remittance {
    pub remittance_id: String,
    pub claim_id: String,
    pub payer_id: String,
    pub payee_npi: String,
    pub service_lines: Vec<ServiceLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLine {
    pub service_line_id: String,
    pub procedure_code: String,
    pub billed_amount: f64,
    pub payer_paid_amount: f64,
    pub coinsurance_amount: f64,
    pub copay_amount: f64,
    pub deductible_amount: f64,
    pub not_allowed_amount: f64,
    pub remark_codes: Option<Vec<String>>,
}

impl ServiceLine {
    pub fn new(
        line: &IntakeServiceLine, 
        billed_amount: f64, 
        payer_paid_amount: f64, 
        coinsurance_amount: f64, 
        copay_amount: f64, 
        deductible_amount: f64, 
        not_allowed_amount: f64
    ) -> Result<ServiceLine, String> {
        if line.do_not_bill.unwrap_or(false) {
            return Ok(ServiceLine {
                service_line_id: line.service_line_id.clone(),
                procedure_code: line.procedure_code.clone(),
                billed_amount: 0.0,
                payer_paid_amount: 0.0,
                coinsurance_amount: 0.0,
                copay_amount: 0.0,
                deductible_amount: 0.0,
                not_allowed_amount: 0.0,
                remark_codes: None,
            });
        }
        Ok(ServiceLine {
            service_line_id: line.service_line_id.clone(),
            procedure_code: line.procedure_code.clone(),
            billed_amount: billed_amount,
            payer_paid_amount: payer_paid_amount,
            coinsurance_amount: coinsurance_amount,
            copay_amount: copay_amount,
            deductible_amount: deductible_amount,
            not_allowed_amount: not_allowed_amount,
            remark_codes: None,
        })
    }
}