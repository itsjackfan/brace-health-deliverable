use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayerClaim {
    pub claim_id: String,
    pub place_of_service_code: i32,
    pub insurance: Insurance,
    pub patient: Patient,
    pub organization: Organization,
    pub rendering_provider: RenderingProvider,
    pub service_lines: Vec<ServiceLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insurance {
    pub payer_id: PayerId,
    pub patient_member_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayerId {
    Medicare,
    UnitedHealthGroup,
    Anthem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patient {
    pub first_name: String,
    pub last_name: String,
    pub gender: Gender,
    pub dob: String,
    pub email: Option<String>,
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Gender {
    #[serde(rename = "m")]
    Male,
    #[serde(rename = "f")]
    Female,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub name: String,
    pub billing_npi: Option<String>,
    pub ein: Option<String>,
    pub contact: Option<Contact>,
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingProvider {
    pub first_name: String,
    pub last_name: String,
    pub npi: String, // for 10-digit validation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLine {
    pub service_line_id: String,
    pub procedure_code: String,
    pub modifiers: Option<Vec<String>>,
    pub units: i32,
    pub details: String,
    pub unit_charge_currency: String,
    pub unit_charge_amount: f64, // for >0 validation
    pub do_not_bill: Option<bool>,
}