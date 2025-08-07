use intake::{Config, parse_line, read_file, TokenBucket, PayerClaim};
use clearinghouse::{validate_claim, submit_claim_to_payer, submit_remittance_to_submitter, ARData};
use app::{calculate_aging_buckets, calculate_patient_statistics};

use std::env;
use std::thread;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{}", now)
}

fn log_header(component: &str) -> String {
    format!("{} [{}]:", timestamp(), component.to_uppercase())
}

struct WorkItem {
    claim: PayerClaim,
}

enum WorkerMessage {
    Process(WorkItem),
    Shutdown,
}

enum TaskMessage {
    Claim(PayerClaim),
    ParseError(String),
    EndOfFile,
}

enum ResultMessage {
    Completed { claim_id: String },
    Error { claim_id: String, error: String },
    // Status(String),
}

struct ThreadPool {
    workers: Vec<thread::JoinHandle<()>>,
    work_sender: mpsc::Sender<WorkerMessage>,
}

impl ThreadPool {
    fn new(
        num_threads: usize,
        result_sender: mpsc::Sender<ResultMessage>,
        ar_data: Arc<Mutex<Vec<ARData>>>
    ) -> ThreadPool {
        let (work_sender, work_receiver) = mpsc::channel::<WorkerMessage>();
        let work_receiver = Arc::new(Mutex::new(work_receiver));
        
        let mut workers = Vec::with_capacity(num_threads);
        
        for id in 0..num_threads {
            let receiver = Arc::clone(&work_receiver);
            let result_sender = result_sender.clone();
            let ar_data = Arc::clone(&ar_data);
            
            let worker = thread::spawn(move || {
                worker_thread(id, receiver, result_sender, ar_data);
            });
            
            workers.push(worker);
        }
        
        ThreadPool { workers, work_sender }
    }
    
    fn shutdown(self) {
        for _ in 0..self.workers.len() {
            let _ = self.work_sender.send(WorkerMessage::Shutdown);
        }
        
        for worker in self.workers {
            let _ = worker.join();
        }
    }
}

fn worker_thread(
    worker_id: usize,
    work_receiver: Arc<Mutex<mpsc::Receiver<WorkerMessage>>>,
    result_sender: mpsc::Sender<ResultMessage>,
    ar_data: Arc<Mutex<Vec<ARData>>>,
) {
    eprintln!("{} Worker {} started", log_header("worker"), worker_id);
    
    loop {
        let work_item = {
            let receiver = work_receiver.lock().unwrap();
            receiver.recv()
        };
        
        match work_item {
            Ok(WorkerMessage::Process(item)) => {
                eprintln!("{} Worker {} received claim {}", log_header("worker"), worker_id, item.claim.claim_id);
                
                match process_claim_direct(&item.claim) {
                    Ok(ar_data_item) => {
                        ar_data.lock().unwrap().push(ar_data_item);
                        eprintln!("{} Worker {} completed claim {}", log_header("worker"), worker_id, item.claim.claim_id);
                        let _ = result_sender.send(ResultMessage::Completed {
                            claim_id: item.claim.claim_id.clone(),
                        });
                    }
                    Err(e) => {
                        eprintln!("{} Worker {} failed claim {}: {}", log_header("worker"), worker_id, item.claim.claim_id, e);
                        let _ = result_sender.send(ResultMessage::Error {
                            claim_id: item.claim.claim_id.clone(),
                            error: e,
                        });
                    }
                }
            }
            Ok(WorkerMessage::Shutdown) => {
                eprintln!("{} Worker {} shutting down", log_header("worker"), worker_id);
                break;
            }
            Err(_) => {
                eprintln!("{} Worker {} channel disconnected", log_header("worker"), worker_id);
                break;
            }
        }
    }
}

fn display_ar_report(data: &Vec<ARData>, total_claims: usize) {
    let buckets = calculate_aging_buckets(&data);
    let (avg_copay, avg_coinsurance, avg_deductible, num_patients) = calculate_patient_statistics(&data);
    
    println!("=== AR Aging Report ===");
    println!("Total Claims: {}", total_claims);
    println!("0-1 minutes: {}", buckets[0]);
    println!("1-2 minutes: {}", buckets[1]);
    println!("2-3 minutes: {}", buckets[2]);
    println!("3+ minutes: {}", buckets[3]);
    println!();
    println!("=== Patient Statistics ===");
    println!("Total Patients: {}", num_patients);
    println!("Average Copay per Patient: ${:.2}", avg_copay);
    println!("Average Coinsurance per Patient: ${:.2}", avg_coinsurance);
    println!("Average Deductible per Patient: ${:.2}", avg_deductible);
    println!("========================");
}

fn ar_reporting_thread(ar_data: Arc<Mutex<Vec<ARData>>>) {
    loop {
        thread::sleep(Duration::from_secs(5));
        
        let data = ar_data.lock().unwrap();
        let total_claims = data.len();
        
        if total_claims == 0 {
            println!("=== AR Aging Report ===\nTotal Claims: 0\n========================");
            continue;
        }

        display_ar_report(&data, total_claims);
    }
}

fn parser_thread(lines: Vec<String>, config: &Config, task_sender: mpsc::SyncSender<TaskMessage>) {
    eprintln!("{} Starting parser thread", log_header("parser"));
    let mut token_bucket = TokenBucket::new(config.rate_per_second, config.refill_rate);
    let mut parsed_count = 0;
    let mut error_count = 0;
    
    for (line_num, line) in lines.into_iter().enumerate() {
        while !token_bucket.try_consume(1) {
            // eprintln!("{} Parser thread waiting for token bucket", log_header("parser"));
            thread::sleep(Duration::from_millis(config.rate_per_second as u64 * 1000));
        }
        
        match parse_line(&line) {
            Ok(claim) => {
                parsed_count += 1;
                if parsed_count % 5 == 0 {
                    eprintln!("{} Parsed {} claims", log_header("parser"), parsed_count);
                }
                if task_sender.send(TaskMessage::Claim(claim)).is_err() {
                    eprintln!("{} Task channel closed, stopping parser", log_header("parser"));
                    break;
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!("{} Parse error on line {}: {}", log_header("parser"), line_num + 1, e);
                if task_sender.send(TaskMessage::ParseError(format!("Failed to parse line: {}", e))).is_err() {
                    eprintln!("{} Task channel closed, stopping parser", log_header("parser"));
                    break;
                }
            }
        }
    }
    
    eprintln!("{} Parser complete: {} parsed, {} errors", log_header("parser"), parsed_count, error_count);
    let _ = task_sender.send(TaskMessage::EndOfFile);
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    eprintln!("{} Initializing configuration", log_header("config"));
    let config = Config::build(args.into_iter()).map_err(|e| format!("Config error: {}", e))?;
    eprintln!("{} Configuration loaded: file={}, threads={}, rate={}/sec", 
        log_header("config"), config.file_path, config.num_threads, config.rate_per_second);
    
    eprintln!("{} Reading file: {}", log_header("file_io"), config.file_path);
    let lines: Vec<String> = read_file(&config)
        .map_err(|e| format!("Failed to read file: {}", e))?
        .collect();
    
    let total_lines = lines.len();
    eprintln!("{} File read complete: {} lines loaded", log_header("file_io"), total_lines);
    
    let ar_data = Arc::new(Mutex::new(Vec::new()));
    let ar_data_clone = ar_data.clone();
    
    eprintln!("{} Creating worker thread pool with {} threads", log_header("thread_pool"), config.num_threads);
    let (result_sender, result_receiver) = mpsc::channel::<ResultMessage>();
    let thread_pool = ThreadPool::new(config.num_threads as usize, result_sender.clone(), ar_data.clone());
    
    eprintln!("{} Starting AR reporting thread", log_header("reporting"));
    let _reporting_handle = thread::spawn(move || {
        ar_reporting_thread(ar_data_clone);
    });
    
    eprintln!("{} Starting parser thread", log_header("coordination"));
    let (task_sender, task_receiver) = mpsc::sync_channel::<TaskMessage>(1000);
    let config_clone = config.clone();
    let _parser_handle = thread::spawn(move || {
        parser_thread(lines, &config_clone, task_sender);
    });
    
    let mut active_claims = 0usize;
    let mut processed_claims = 0usize;
    let mut parse_errors = 0usize;
    let mut parsing_complete = false;
    
    eprintln!("{} Main event loop starting: {} lines to process", log_header("coordination"), total_lines);
    eprintln!("{} Configuration: {} threads, {} claims/sec limit", log_header("coordination"), config.num_threads, config.rate_per_second);
    
    loop {
        // Try to get parsed claims from parser thread
        if !parsing_complete {
            match task_receiver.try_recv() {
                Ok(TaskMessage::Claim(claim)) => {
                    let work_item = WorkItem { claim };
                    
                    if thread_pool.work_sender.send(WorkerMessage::Process(work_item)).is_err() {
                        return Err("Thread pool shutdown unexpectedly".to_string());
                    }
                    active_claims += 1;
                }
                Ok(TaskMessage::ParseError(_error)) => {
                    parse_errors += 1;
                }
                Ok(TaskMessage::EndOfFile) => {
                    parsing_complete = true;
                    eprintln!("{} Parsing phase complete: {} errors", log_header("coordination"), parse_errors);
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err("Task channel disconnected".to_string());
                }
            }
        }
        
        // Process worker results
        match result_receiver.try_recv() {
            Ok(ResultMessage::Completed { claim_id }) => {
                active_claims -= 1;
                processed_claims += 1;
                eprintln!("{} Claim {} processed", log_header("coordination"), claim_id);
                if processed_claims % 50 == 0 {
                    eprintln!("{} Progress: {}/{} processed, {} active", 
                        log_header("coordination"), processed_claims, total_lines - parse_errors, active_claims);
                }
            }
            Ok(ResultMessage::Error { claim_id, error }) => {
                active_claims -= 1;
                processed_claims += 1;
                eprintln!("{} Claim {} failed: {}", log_header("coordination"), claim_id, error);
            }
            // Ok(ResultMessage::Status(status)) => {
            //     // Status messages already have proper headers from worker threads
            // }
            Err(mpsc::TryRecvError::Empty) => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err("Result channel disconnected".to_string());
            }
        }
        
        if parsing_complete && active_claims == 0 {
            break;
        }
    }
    
    eprintln!("{} Shutting down thread pool", log_header("coordination"));
    thread_pool.shutdown();
    
    eprintln!("{} Processing complete: {} claims processed, {} parse errors", log_header("coordination"), processed_claims, parse_errors);
    display_ar_report(&ar_data.lock().unwrap(), processed_claims);
    Ok(())
}

fn process_claim_direct(claim: &PayerClaim) -> Result<ARData, String> {
    eprintln!("{} Starting validation for claim {}", log_header("claim_processor"), claim.claim_id);
    if let Err(e) = validate_claim(&claim) {
        return Err(format!("Validation failed: {}", e));
    }
    
    eprintln!("{} Submitting claim {} to payer", log_header("claim_processor"), claim.claim_id);
    let remittance = submit_claim_to_payer(&claim)?;
    eprintln!("{} Remittance {} received for claim {}", log_header("claim_processor"), remittance.remittance_id, claim.claim_id);

    eprintln!("{} Submitting remittance {} to clearinghouse", log_header("claim_processor"), remittance.remittance_id);
    let ar_data = submit_remittance_to_submitter(&remittance)?;
    eprintln!("{} AR data generated for claim {}", log_header("claim_processor"), claim.claim_id);
    
    Ok(ar_data)
}