# SPDX-License-Identifier: MIT OR Apache-2.0

RUST_LOG="fastui_cosmic=debug,multiview=debug" cargo run --release --package multiview -- "$@"
