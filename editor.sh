# SPDX-License-Identifier: MIT OR Apache-2.0

RUST_LOG="fastui_cosmic=debug,editor=debug" cargo run --release --package editor -- "$@"
