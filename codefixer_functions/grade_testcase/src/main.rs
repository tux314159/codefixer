use anyhow::Result;
use lambda_runtime::{Error, LambdaEvent, run, service_fn, tracing};
use serde::{Deserialize, Serialize};

use codefixer_shared_interface::{ProblemLanguage, ProblemType};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}

/// This is a made-up example. Incoming messages come into the runtime as unicode
/// strings in json format, which can map to any structure that implements `serde::Deserialize`
/// The runtime pays no attention to the contents of the incoming message payload.
#[derive(Debug, Clone, Deserialize)]
struct IncomingMessage {
    runtype: ProblemType,
    language: ProblemLanguage,
}

/// This is a made-up example of what an outgoing message structure may look like.
/// There is no restriction on what it can be. The runtime requires responses
/// to be serialized into json. The runtime pays no attention
/// to the contents of the outgoing message payload.
#[derive(Serialize)]
struct OutgoingMessage {
    req_id: String,
    msg: String,
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<IncomingMessage>) -> Result<OutgoingMessage> {
    // Extract some useful info from the request
    let language = event.payload.language;

    // Prepare the outgoing message
    let resp = OutgoingMessage {
        req_id: event.context.request_id,
        msg: format!("language {:?}.", language),
    };

    // Return `OutgoingMessage` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}
