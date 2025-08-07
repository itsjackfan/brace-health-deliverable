use intake::{Config, run_intake};
use clearinghouse::{validate_claim, submit_claim_to_payer, submit_remittance_to_submitter};
use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    // Create config from command line args
    let config = Config::build(args.into_iter()).map_err(|e| format!("Config error: {}", e))?;
    
    // Read and parse claims from file with rate limiting
    println!("Reading claims from file: {}", config.file_path);
    let claims = run_intake(&config)?;
    println!("Successfully read {} claims", claims.len());

    let mut ar_data_vec= Vec::new();
    
    // Process each claim through validation and submission
    for (i, claim) in claims.iter().enumerate() {
        println!("Processing claim {} of {}", i + 1, claims.len());
        
        // Validate claim
        if let Err(e) = validate_claim(claim) {
            eprintln!("Validation failed for claim {}: {}", i + 1, e);
            continue;
        }
        
        // Submit to payer and get remittance
        let remittance = submit_claim_to_payer(claim)?;

        // Submit remittance to submitter
        let ar_data = submit_remittance_to_submitter(&remittance)?;
        ar_data_vec.push(ar_data);
    }
    
    println!("All claims processed!");
    Ok(())
}
