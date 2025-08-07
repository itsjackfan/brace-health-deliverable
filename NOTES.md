## OVERALL ARCHITECTURE
> TODO

## DATA STRUCTURES AND TYPES
> TODO
---

## STEP 1
Standard file read-in and parsing into `PayerClaim` structs.
The load-balancing and whatnot is taken care of through a **token bucket** instance, which is configurable both for standard load balancing and burst-based load balancing for high thoroughput. Any calls that get "load-balanced"/rejected will be automatically requeued. The read-in process will run until all lines have successfully been read in.
> **_VERIFY THIS STATEMENT_** Note that the token bucket load is actually 1/n of the actual load where n is the number of threads available to the machine because parsing occurs concurrently and asynchronously for each line to be parsed (a separate `parse_line` call is made in a different thread).
When choosing the rate-limiter, the major trade-off that was considered was the ordering of read-ins and exact rate configuration limits. However, because the order of read-ins in this instance doesn't matter strongly (there's no explicit priority, though wait time or amount of money or some cross-feature heuristic could be calculated) and because disk usage is more "bursty" by nature anyway, the token bucket makes more sense to use.

## STEP 2
From my understanding, the simulated clearinghouse is essentially for validation and routing and serves as a "middleman" between the payer and the insurance provider. Thus, the functionality can be split up into a couple most relevant segments:

### Pre-parse validation
This includes only what functionality items are necessary to ensure `serde`'s read-in doesn't crash (like JSON validity and field existence); this is carried out at the JSON parsing level using Rust's internal validation and typechecking systems with `struct`s and `enum`s

### Post-parse validation
This includes all of the business logic and field-specific validation rules that are incorrect on an application level but do not present immediate errors/crashes. Specifically, the following are included:
**_05 AUG -- TO BE DONE TOMORROW. REFER TO PERPLEXITY LIST OF VALIDATORS_**

All of the aforementioned pieces of logic as well as the routing is handled in `clearinghouse/validate_claim()`.

### Routing
Once the claim is determined as valid, it will then forward to the relevant payer out of the 3 using the `clearinghouse/route_claim()` function. The actual activity itself is implemented in the specific crates for each of the three payers, and includes some relatively simple custom LLM logic.

- How do we handle invalid claims?
- Should we automatically submit all claims or write some sort of either CLI or simple frontend? (maybe this can be debug mode vs. "user" mode)
 