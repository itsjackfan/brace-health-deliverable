use intake::{PayerClaim, PayerId};
use insurance::{Medicare, UnitedHealthGroup, Anthem, Insurance, Remittance};
use std::collections::HashSet;

pub fn validate_claim(claim: &PayerClaim) -> Result<(), String> {
    validate_non_empty_fields(claim)?;
    validate_formats(claim)?;
    validate_business_rules(claim)?;
    Ok(())
}

fn validate_non_empty_fields(claim: &PayerClaim) -> Result<(), String> {
    let empty_check = |field: &str, value: &str| {
        if value.trim().is_empty() {
            Err(format!("{} cannot be empty", field))
        } else {
            Ok(())
        }
    };

    empty_check("claim_id", &claim.claim_id)?;
    empty_check("patient.first_name", &claim.patient.first_name)?;
    empty_check("patient.last_name", &claim.patient.last_name)?;
    empty_check("patient.dob", &claim.patient.dob)?;
    empty_check("organization.name", &claim.organization.name)?;
    empty_check("rendering_provider.first_name", &claim.rendering_provider.first_name)?;
    empty_check("rendering_provider.last_name", &claim.rendering_provider.last_name)?;
    empty_check("rendering_provider.npi", &claim.rendering_provider.npi)?;
    empty_check("insurance.patient_member_id", &claim.insurance.patient_member_id)?;

    for (i, line) in claim.service_lines.iter().enumerate() {
        empty_check(&format!("service_lines[{}].service_line_id", i), &line.service_line_id)?;
        empty_check(&format!("service_lines[{}].procedure_code", i), &line.procedure_code)?;
        empty_check(&format!("service_lines[{}].details", i), &line.details)?;
        empty_check(&format!("service_lines[{}].unit_charge_currency", i), &line.unit_charge_currency)?;
    }

    Ok(())
}

fn validate_formats(claim: &PayerClaim) -> Result<(), String> {
    // NPI validation (10 digits)
    let validate_npi = |npi: &str, field: &str| {
        if npi.len() != 10 || !npi.chars().all(|c| c.is_ascii_digit()) {
            Err(format!("{} must be exactly 10 digits", field))
        } else {
            Ok(())
        }
    };

    validate_npi(&claim.rendering_provider.npi, "rendering_provider.npi")?;
    
    if let Some(ref npi) = claim.organization.billing_npi {
        validate_npi(npi, "organization.billing_npi")?;
    }

    // EIN validation (XX-XXXXXXX)
    if let Some(ref ein) = claim.organization.ein {
        if ein.len() != 10 || ein.chars().nth(2) != Some('-') || 
           !ein[..2].chars().all(|c| c.is_ascii_digit()) ||
           !ein[3..].chars().all(|c| c.is_ascii_digit()) {
            return Err("organization.ein must match format XX-XXXXXXX".to_string());
        }
    }

    // ZIP code validation
    if let Some(ref address) = claim.patient.address {
        if let Some(ref zip) = address.zip {
            let valid_zip = zip.len() == 5 && zip.chars().all(|c| c.is_ascii_digit()) ||
                          (zip.len() == 10 && zip.chars().nth(5) == Some('-') &&
                           zip[..5].chars().all(|c| c.is_ascii_digit()) &&
                           zip[6..].chars().all(|c| c.is_ascii_digit()));
            if !valid_zip {
                return Err("patient.address.zip must be XXXXX or XXXXX-XXXX format".to_string());
            }
        }
    }

    // Currency code validation (3 uppercase letters)
    for (i, line) in claim.service_lines.iter().enumerate() {
        if line.unit_charge_currency.len() != 3 || !line.unit_charge_currency.chars().all(|c| c.is_ascii_uppercase()) {
            return Err(format!("service_lines[{}].unit_charge_currency must be 3 uppercase letters", i));
        }
    }

    Ok(())
}

fn validate_business_rules(claim: &PayerClaim) -> Result<(), String> {
    // Place of service code range
    if claim.place_of_service_code < 1 || claim.place_of_service_code > 99 {
        return Err("place_of_service_code must be between 1-99".to_string());
    }

    // Service lines must not be empty
    if claim.service_lines.is_empty() {
        return Err("service_lines must contain at least one item".to_string());
    }

    // Service line business rules
    let mut service_line_ids = HashSet::new();
    let first_currency = &claim.service_lines[0].unit_charge_currency;

    for (i, line) in claim.service_lines.iter().enumerate() {
        // Unique service line IDs
        if !service_line_ids.insert(&line.service_line_id) {
            return Err(format!("Duplicate service_line_id: {}", line.service_line_id));
        }

        // Units must be >= 1
        if line.units < 1 {
            return Err(format!("service_lines[{}].units must be at least 1", i));
        }

        // Amount must be positive
        if line.unit_charge_amount <= 0.0 {
            return Err(format!("service_lines[{}].unit_charge_amount must be positive", i));
        }

        // Currency consistency
        if line.unit_charge_currency != *first_currency {
            return Err("All service lines must use the same currency".to_string());
        }

        // Line total must be positive
        let total = line.unit_charge_amount * line.units as f64;
        if total <= 0.0 {
            return Err(format!("service_lines[{}] total must be positive", i));
        }
    }

    // Provider NPI != Organization billing NPI
    if let Some(ref billing_npi) = claim.organization.billing_npi {
        if &claim.rendering_provider.npi == billing_npi {
            return Err("rendering_provider.npi cannot equal organization.billing_npi".to_string());
        }
    }

    Ok(())
}

pub fn submit_claim_to_payer(claim: &PayerClaim) -> Result<Remittance, String> {
    match claim.insurance.payer_id {
        PayerId::Medicare => {
            let insurance = Medicare::new();
            insurance.submit_claim(claim)
        }
        PayerId::UnitedHealthGroup => {
            let insurance = UnitedHealthGroup::new();
            insurance.submit_claim(claim)
        }
        PayerId::Anthem => {
            let insurance = Anthem::new();
            insurance.submit_claim(claim)
        }
    }
}

pub fn submit_remittance_to_submitter(remittance: &Remittance) -> Result<(), String> {
    todo!("Implement remittance submission logic")
}