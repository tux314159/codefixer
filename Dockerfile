FROM rust:slim-trixie AS build-base-img
RUN apt-get update && apt-get install -y openssl libssl-dev sqlite3 libsqlite3-dev pkg-config

RUN mkdir -p /src
RUN mkdir -p /src/codefixer_srv/src
RUN mkdir -p /src/codefixer_functions/src
RUN mkdir -p /src/codefixer_shared_interface/src
COPY Cargo.toml Cargo.lock /src
COPY codefixer_srv/Cargo.toml /src/codefixer_srv/Cargo.toml

# Create stub main files
COPY <<EOF /src/stub_main.rs
fn main() -> () {}
EOF
RUN cp /src/stub_main.rs /src/codefixer_srv/src/main.rs
RUN cp /src/stub_main.rs /src/codefixer_srv/src/gen_openapi.rs
RUN cp /src/stub_main.rs /src/codefixer_functions/src/compile_submission.rs
RUN cp /src/stub_main.rs /src/codefixer_shared_interface/src/lib.rs

COPY codefixer_functions/Cargo.toml /src/codefixer_functions/Cargo.toml
COPY codefixer_shared_interface/Cargo.toml /src/codefixer_shared_interface/Cargo.toml

WORKDIR /src
RUN cargo build --workspace --all-targets

# Build

FROM build-base-img AS build-all

COPY codefixer_shared_interface/src src/codefixer_shared_interface/src
COPY codefixer_srv/src src/codefixer_srv/src
COPY codefixer_functions/src src/codefixer_functions/src
COPY migrations src/migrations
COPY state /src/state

WORKDIR /src
ENV DATABASE_URL=sqlite://state/db.sqlite3
RUN cargo build --bin=compile_submission

# Run
FROM debian:trixie-slim AS codefixer-compile-submission
WORKDIR ${LAMBDA_TASK_ROOT}

ENV DATABASE_URL sqlite://state/db.sqlite3

RUN apt-get update && apt-get install -y openssl sqlite3

# Copy in the source code
COPY --from=build-all /src/target/debug/compile_submission .

RUN useradd app
USER app

CMD ["./compile_submission"]
