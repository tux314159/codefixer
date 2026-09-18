FROM rust:slim-trixie AS build-base-img
RUN apt-get update && apt-get install -y openssl libssl-dev pkg-config libseccomp2 libseccomp-dev

RUN mkdir -p /src
RUN mkdir -p /src/codefixer_srv/src
RUN mkdir -p /src/codefixer_functions/src
RUN mkdir -p /src/codefixer_shared_interface/src
COPY Cargo.toml Cargo.lock /src
COPY codefixer_srv/Cargo.toml /src/codefixer_srv/Cargo.toml
COPY codefixer_functions/Cargo.toml /src/codefixer_functions/Cargo.toml
COPY codefixer_shared_interface/Cargo.toml /src/codefixer_shared_interface/Cargo.toml

# Create stub main files
COPY <<EOF /src/stub_main.rs
fn main() -> () {}
EOF
RUN cp /src/stub_main.rs /src/codefixer_shared_interface/src/lib.rs
RUN cp /src/stub_main.rs /src/codefixer_srv/src/main.rs
RUN cp /src/stub_main.rs /src/codefixer_srv/src/gen_openapi.rs
RUN cp /src/stub_main.rs /src/codefixer_functions/src/compile_submission.rs
RUN cp /src/stub_main.rs /src/codefixer_functions/src/grade_testcase.rs

# Build dependencies
WORKDIR /src
RUN cargo build --workspace --bin=compile_submission
RUN cargo clean --workspace
RUN find codefixer_shared_interface -name '*.rs' -delete
RUN find codefixer_srv -name '*.rs' -delete
RUN find codefixer_functions -name '*.rs' -delete

# Build

FROM build-base-img AS build-all

COPY codefixer_shared_interface/src /src/codefixer_shared_interface/src
COPY codefixer_srv/src /src/codefixer_srv/src
COPY codefixer_functions/src /src/codefixer_functions/src
COPY codefixer_functions/build.rs /src/codefixer_functions/build.rs
COPY migrations /src/migrations

WORKDIR /src
RUN cargo build --bin=compile_submission
RUN cargo build --bin=grade_testcase

# Run
FROM debian:trixie-slim AS codefixer-compile-submission
WORKDIR ${LAMBDA_TASK_ROOT}

RUN apt-get update && apt-get install -y openssl sqlite3 libseccomp2

# Copy in the binary
COPY --from=build-all /src/target/debug/compile_submission .

RUN useradd app
USER app

CMD ["./compile_submission"]
