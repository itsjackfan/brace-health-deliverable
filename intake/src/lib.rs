pub mod types;

// Re-export all types for easier access from other crates
pub use types::*;

use std::time::Instant;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use serde_json;

pub struct Config {
    pub file_path: String,
    pub rate_per_second: u32,
    pub refill_rate: u32,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, String> {
        args.next();

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path".to_string()),
        };

        let refill_rate = match args.next() {
            Some(arg) => arg.parse().map_err(|e| format!("Invalid refill rate: {}", e))?,
            None => return Err("Didn't get a refill rate".to_string()),
        };

        let rate_per_second = match args.next() {
            Some(arg) => arg.parse().map_err(|e| format!("Invalid rate per second: {}", e))?,
            None => return Err("Didn't get a rate per second".to_string()),
        };

        Ok(Config { file_path, rate_per_second, refill_rate })
    }
}

// could also use eg governor library but why not for fun/practice
pub struct TokenBucket {
    pub capacity: u32,
    pub tokens: u32,
    pub refill_rate: u32,
    pub last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: u32) -> TokenBucket {
        TokenBucket {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn try_consume(&mut self, amount: u32) -> bool {
        self.refill();

        if self.tokens >= amount {
            self.tokens -= amount;
            true
        } else {
            false
        }
    }

    pub fn refill(&mut self) {
        let now = Instant::now();
        let time_since_last_refill = now.duration_since(self.last_refill);

        let tokens_to_add = (time_since_last_refill.as_millis() as u64 * self.refill_rate as u64 / 1000) as u32;

        if tokens_to_add > 0 {
            self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
            self.last_refill = now;
        }        
    }
}

pub fn read_file(config: &Config) -> Result<impl Iterator<Item = String>, String> {
    let file = File::open(&config.file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    Ok(reader
        .lines()
        .map(|line| line.unwrap_or_else(|_| "".to_string()))
    )
}

pub fn parse_line(line: &str) -> Result<PayerClaim, String> {
    let claim: PayerClaim = serde_json::from_str(line).map_err(|e| format!("Failed to parse line: {}", e))?;
    Ok(claim)
}

pub fn run_intake(config: &Config) -> Result<Vec<PayerClaim>, String> {
    let lines = read_file(config).map_err(|e| format!("Failed to read file: {}", e))?;
    let mut queue: VecDeque<String> = lines.collect();
    let mut claims = Vec::new();
    
    // TODO: read in number of threads to run from config + other async stuff

    let mut token_bucket = TokenBucket::new(config.rate_per_second, config.refill_rate);
    
    while let Some(line) = queue.pop_front() {
        if !token_bucket.try_consume(1) {
            queue.push_back(line);
            continue;
        }
        let claim = parse_line(&line).map_err(|e| format!("Failed to parse line: {}", e))?;
        claims.push(claim);
    }
    
    Ok(claims)
}