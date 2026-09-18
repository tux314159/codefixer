use std::ffi::{CString, c_char};

use anyhow::{Result, anyhow};
use lambda_runtime::{Error, LambdaEvent, run, service_fn, tracing};
use serde::{Deserialize, Serialize};

use codefixer_shared_interface::ProblemType;
use tokio::fs;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}

#[derive(Debug, Clone, Deserialize)]
struct IncomingMessage {
    exe_uri: String,
    tl_ms: i64,
    ml_kib: i64,
    runtype: ProblemType,
}

#[derive(Serialize)]
struct OutgoingMessage {
    req_id: String,
    verdict: ChildStatus,
    info: ChildInfo,
}

// Interfacing with the C runner
#[repr(C)]
#[derive(Debug, Clone, Serialize)]
struct ChildInfo {
    pub time_ms: i64,
    pub max_mem_kib: i64,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Serialize)]
#[allow(dead_code)]
enum ChildStatus {
    AC = 0,
    WA = 1,
    TLE = 2,
    MLE = 3,
    RTE = 4,
    UnknownError = 5,
}

unsafe extern "C" {
    fn run_limited(
        exe_path: *const c_char,
        input_path: *const c_char,
        output_path: *const c_char,
        info: *mut ChildInfo,
        tl_ms: i64,
        ml_kib: i64,
    ) -> ChildStatus;
}

const EXE_FILE_NAME: &str = "exe";
const OUTPUT_FILE_NAME: &str = "output";
const TC_INPUT_FILE_NAME: &str = "tc.in";
const TC_OUTPUT_FILE_NAME: &str = "tc.out";

async fn function_handler(event: LambdaEvent<IncomingMessage>) -> Result<OutgoingMessage> {
    let exe_uri = event.payload.exe_uri;
    let tl_ms = event.payload.tl_ms;
    let ml_kib = event.payload.ml_kib;
    let runtype = event.payload.runtype;

    Command::new("mv")
        .args(vec![&exe_uri, EXE_FILE_NAME])
        .spawn()?
        .wait()
        .await?
        .success()
        .then_some(())
        .ok_or(anyhow!("Filesystem error"))?;
    Command::new("chmod")
        .args(vec!["755", EXE_FILE_NAME])
        .spawn()?
        .wait()
        .await?
        .success()
        .then_some(())
        .ok_or(anyhow!("Filesystem error"))?;

    let (mut status, info) = run_file(&exe_uri, tl_ms, ml_kib).await;

    let tc_out = normalise_whitespace(&fs::read_to_string(TC_OUTPUT_FILE_NAME).await?);
    let output = normalise_whitespace(&fs::read_to_string(OUTPUT_FILE_NAME).await?);
    if status == ChildStatus::AC && tc_out != output {
        status = ChildStatus::WA;
    }

    let resp = OutgoingMessage {
        req_id: event.context.request_id,
        verdict: status,
        info: info,
    };

    Ok(resp)
}

async fn run_file(uri: &str, tl_ms: i64, ml_kib: i64) -> (ChildStatus, ChildInfo) {
    let mut info = ChildInfo {
        time_ms: -1,
        max_mem_kib: -1,
    };

    let status = unsafe {
        run_limited(
            CString::new(EXE_FILE_NAME).unwrap().as_ptr(),
            CString::new(TC_INPUT_FILE_NAME).unwrap().as_ptr(),
            CString::new(OUTPUT_FILE_NAME).unwrap().as_ptr(),
            &mut info as *mut ChildInfo,
            tl_ms,
            ml_kib,
        )
    };

    (status, info)
}

fn normalise_whitespace(s: &str) -> String {
    let mut r = String::new();
    let mut prev_ws = false;
    for c in s.chars() {
        if c == '\t' || c == '\n' || c == ' ' {
            if !prev_ws {
                r.push(' ');
            }
            prev_ws = true;
        } else {
            r.push(c);
            prev_ws = false;
        }
    }
    r
}
