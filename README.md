# README #

This is the pretty rushed, demo-like client for Method Fi.

## Run ##

1. Create a `tmp` folder in the root directory of this repo.
2. Create a `data` folder in the root directory of this repo.

In the `data` folder, it should contain the API token `methodfi-api`.

Then, run with:

```bash
env RUST_LOG=debug cargo run .
```

## Design ##

In the `./src` directory:

- `main.rs`

  Uses `actix_web` as the server framework to run a web server.

- `caller.rs`

  Includes the API call and the entity of the API.

- `lib.rs`

  Contains logic layer functions and helper functions.

- `xml_parser.rs`

  Includes the data structure of the XML (row).

### Workflow ###

1. The server runs.
2. The user submits an XML file.
3. The XML file is parsed and a simple table is shown for review. The user can click to confirm or cancel.
4. After confirmation, the service generates the entities, accounts, and makes the payment.
5. Meanwhile, the service will generate three reports that the user can download as CSV files.

## Something Left ##

>Creating Liability Accounts directly is only supported on a case-by-case basis.
>If you need to create a Liability Account, contact your Method CSM.

While I signed up for my account and obtained the API token, I don't actually run to make the payment.

## The Steps I Would Take to Make This Production-Ready ##

**Tests**

Need more unit tests. The few tests I left inside so far just for making sure the parser is currect. 

**Error Handling**

All the `.unwrap()` instances should be handled properly. 

**API Rate Limit**

I currently use `time::interval(Duration::from_secs(60))` and a counter variable to handle the rate limit. This should be managed more carefully.

**Async Design**

The `confirm_payment` function is actually blocked by `payouts_call`. This means when the input data is very large, the browser won't get a response from this call. It should be called inside `tokio::spawn` so the browser will get a response, and the user can check back on the progress. **Therefore, I need:**

**Persistence**

A database is required to keep track of the status of payments. This will store which payments have been processed and which haven't. If the service crashes, we can restore the records to continue working.

**Too many duplication code**

Need more abstract
