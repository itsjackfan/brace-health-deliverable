# BRACE HEALTH TAKEHOME DELIVERABLE
Created by: Jack Fan
Start date: 05 Aug 2025
Completion/submission date: 07 Aug 2025

## STATEMENT ON OVERALL ARCHITECTURE UNDERSTANDING
From my understanding overall of how the different healthcare systems and entities work together, the patient, payer, and provider form the "core triangle" in that the patient, who is insured by the payer, gets services from the provider. From there, the provider will need to submit the claim through the system implemented in the deliverable to the provider. Once the provider adjudicates based on the EDI835 standard, the payer will send back a remittance to the provider, which then allows the provider to communciate to the patient what/how much they need to pay. From there, both the patient and the provider will pay the necessary amounts to the provider.

---

## STEP 1
Standard file read-in and parsing into `PayerClaim` structs.
The load-balancing and whatnot is taken care of through a **token bucket** instance, which is configurable both for standard load balancing and burst-based load balancing for high thoroughput. Any calls that get "load-balanced"/rejected will be automatically requeued. The read-in process will run until all lines have successfully been read in.
When choosing the rate-limiter, the major trade-off considered was ordering vs. burst capacity. The token bucket approach was selected because:
- Order of claims doesn't matter (no explicit priority system)
- Allows for natural "bursty" parsing patterns
- Configurable burst capacity and steady-state rate

The parsing implementation uses a dedicated thread rather than inline parsing to:
- Maintain consistent rate limiting behavior
- Separate I/O concerns from worker thread processing
- Enable better progress tracking and error handling
- Allow the main thread to focus on coordination and message passing

## STEP 2
From my understanding, the simulated clearinghouse is essentially for validation and routing and serves as a "middleman" between the payer and the insurance provider. Thus, the functionality can be split up into a couple most relevant segments:

### Pre-parse validation
This includes only what functionality items are necessary to ensure `serde`'s read-in doesn't crash (like JSON validity and field existence); this is carried out at the JSON parsing level using Rust's internal validation and typechecking systems with `struct`s and `enum`s. This type of validation has more or less been abstracted away in the parsing function from step 1.

### Post-parse validation
This includes all of the business logic and field-specific validation rules that are incorrect on an application level but do not present immediate errors/crashes. Specifics and detailed implementation can be found in `clearinghouse/validate_claim()` and relevant functions.

Any invalid claims will bubble up with the relevant error message and why it failed validation, and will not be processed.

## STEP 3 
Once the claim is determined as valid, it will then forward to the relevant payer out of the 3 using the `clearinghouse/submit_claim_to_payer()` function. 

## STEP 4
For each insurance provider, a heuristic calculation is made during the "adjudication process" to ascertain the amounts within the remittance, and a boiled-down/simplified version of the information contained within the EDI835 document is then submitted as the `Remittance` return type from each of these functions.

The calculations themselves are found in the trait implementations within `insurance/lib.rs` for each of the 3 supported payers, and were determined roughly through shallow research.

## STEP 5
Once the remittance has been successfully calculated and the bureaucracy/red tape has been awaited, the payer will (finally) submit the remittance back to the clearinghouse using the `clearinghouse/submit_remittance_to_submitter()` function. This function essentially abstracts away the "processing" that the provider would need to do in order to get the data into an AR aging report format. For simplicity once again, only the necessary information from the remittance is passed on into the `ARData` struct.

## STEP 6
The overall application implements a **multi-threaded architecture** with the following components:

### Threading Architecture
- **Parser Thread**: Handles file reading and JSON parsing with token bucket rate limiting
- **Worker Thread Pool**: Configurable number of worker threads (default 1) process claims through validation, payer submission, and AR data generation
- **AR Reporting Thread**: Generates periodic aging reports every 5 seconds
- **Main Coordination Thread**: Orchestrates message passing between parser and workers, tracks progress

### Message Passing System
The application uses Rust's `mpsc` channels for thread communication:
- `TaskMessage` enum: Parser → Main (parsed claims, errors, EOF)
- `WorkerMessage` enum: Main → Workers (work items, shutdown signals)
- `ResultMessage` enum: Workers → Main (completion status, errors)

### Concurrency & Performance
- Claims are processed concurrently by the worker pool
- Token bucket prevents parser from overwhelming the system
- Each payer introduces artificial 10-30 second delays (simulating real processing time)
- Progress tracking and comprehensive logging with timestamps and component headers

Note that AR reports are printed to `stdout` while all logging goes to `stderr` for output separation.

## STEP 7
All tests are contained within the `.../tests/` directory within each crate. Test fixtures are located in `.../test_fixtures/` directories where needed. Tests can be run with:
- `cargo test --package (CRATE)` for individual crate testing
- `cargo test` for all integration tests

### Test Coverage
- **intake**: JSON parsing, validation, token bucket rate limiting, file I/O
- **clearinghouse**: Claim validation rules, payer routing, remittance processing  
- **insurance**: Payer-specific calculations, remittance generation, timing delays
- **app**: AR aging calculations, patient statistics, multi-threaded integration

## ADDITIONAL ARCHITECTURAL DETAILS

### Data Flow
1. **File Input** → Lines read synchronously into memory
2. **Parser Thread** → JSON parsing with rate limiting via token bucket
3. **Main Thread** → Distributes parsed claims to worker pool
4. **Worker Threads** → Validate → Submit to payer → Process remittance → Generate AR data
5. **AR Reporting Thread** → Periodic statistics and aging bucket reports
6. **Output** → Final AR report with patient statistics and aging analysis

### Key Data Structures
- `PayerClaim`: Input claim with patient, provider, service line details
- `Remittance`: Payer response with adjudicated amounts per service line  
- `ARData`: Final accounts receivable data for reporting
- `TokenBucket`: Rate limiting implementation with configurable burst capacity

### Configuration
Application accepts command-line arguments: `file_path refill_rate rate_per_second [num_threads]`
- Configurable rate limiting and thread pool sizing
- Comprehensive logging system with component-specific headers and timestamps