# Dockerfile voor de DDA Remote Server
FROM rust:1.78-slim-buster as builder

# Werkdirectory binnen de container
WORKDIR /app

# Copieer Cargo.toml en Cargo.lock om dependencies te installeren
COPY dda-remote/Cargo.toml dda-remote/Cargo.lock ./

# Gebruik de workspace-setup om alle dependencies te compileren
RUN cargo build --release --workspace

# Copieer de resterende source code
COPY dda-remote/crates/ ./crates/
COPY dda-remote/src/ ./src/

# De server runtime is in dda-core
RUN cp crates/dda-core/target/release/dda-core /app/dda-server

# Start de server
CMD ["/app/dda-server"]