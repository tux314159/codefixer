use anyhow::Result;
use derive_more::Display;
use lambda_runtime::{Diagnostic, Error, LambdaEvent, run, service_fn, tracing};
use serde::{Deserialize, Serialize};

use codefixer_shared_interface::{ProblemLanguage, ProblemType};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    println!("Hi");

    run(service_fn(function_handler)).await
}

#[derive(Debug, Clone, Deserialize)]
struct IncomingMessage {
    sub_id: i64,
    sub_uri: String,
    runtype: ProblemType,
    language: ProblemLanguage,
}

#[derive(Serialize)]
struct OutgoingMessage {
    req_id: String,
    exe_uri: String,
}

#[derive(Debug, Clone, Display, Serialize)]
enum FunctionError {
    TimeoutError,
    CompileError(String),
    UnknownError,
}

impl std::error::Error for FunctionError {}

impl From<std::io::Error> for FunctionError {
    fn from(_: std::io::Error) -> Self {
        FunctionError::UnknownError
    }
}

impl From<FunctionError> for Diagnostic {
    fn from(e: FunctionError) -> Self {
        use FunctionError::*;
        Diagnostic {
            error_type: match e {
                TimeoutError => "compile timeout",
                CompileError(_) => "compile error",
                UnknownError => "unknown error",
            }
            .to_owned(),
            error_message: if let CompileError(m) = e {
                m
            } else {
                String::new()
            },
        }
    }
}

const COMPILE_OUTPUT_FILE: &str = "exe";

async fn function_handler(
    event: LambdaEvent<IncomingMessage>,
) -> Result<OutgoingMessage, FunctionError> {
    let sub_id = event.payload.sub_id;
    let sub_uri = event.payload.sub_uri;
    let runtype = event.payload.runtype;
    let language = event.payload.language;

    compile_file(language, sub_uri).await?;
    Command::new("mv")
        .args(vec![COMPILE_OUTPUT_FILE, &sub_id.to_string()])
        .spawn()?
        .wait()
        .await?; // TODO: upload instead

    // Prepare the outgoing message
    let resp = OutgoingMessage {
        req_id: event.context.request_id,
        exe_uri: format!("language {:?}.", language),
    };

    Ok(resp)
}

async fn compile_file(language: ProblemLanguage, uri: String) -> Result<(), FunctionError> {
    use FunctionError::*;
    use ProblemLanguage::*;
    // TODO: download source and upload executable from/to S3.
    let mut cmd = Command::new("timeout");
    cmd.arg("10");
    match language {
        Unknown => {
            cmd.args(vec!["false"]); // just fail
        }
        Cpp => {
            cmd.args(vec![
                "g++",
                "-o",
                COMPILE_OUTPUT_FILE,
                "-std=gnu++17", // C++17
                "-O2",          // optimise
                "-w",           // suppress warnings
                "-s",           // strip
                "-lm",          // link math lib
                &uri,
            ]);
        }
        C => {
            cmd.args(vec![
                "gcc",
                "-o",
                COMPILE_OUTPUT_FILE,
                "-std=gnu17", // C17
                "-O2",        // optimise
                "-w",         // suppress warnings
                "-s",         // strip
                "-lm",        // link math lib
                &uri,
            ]);
        }
        Python => {
            cmd.args(vec!["cp", &uri, COMPILE_OUTPUT_FILE]); // just copy
        }
    };

    let output = cmd.output().await?;
    match output.status.code() {
        None => Err(UnknownError),
        Some(0) => Ok(()),
        Some(1) => Err(CompileError(
            String::from_utf8(output.stderr).unwrap_or(String::new()),
        )),
        Some(124) => Err(TimeoutError),
        Some(_) => Err(UnknownError),
    }
}
