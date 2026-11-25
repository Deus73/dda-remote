#!/usr/bin/env bash
set -euo pipefail
cargo test --workspace
cargo build --release
printf 'Built: %s\n' "$(pwd)/target/release/dda-remote"
