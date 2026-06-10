//! Integration tests for the SchiroEngine workspace.
//!
//! This crate is dedicated to unit tests that span the public
//! surface of every other crate. The production crates stay free of
//! `#[cfg(test)]` modules so that their binary output is not bloated
//! by test code.
//!
//! # Layout
//!
//! - `tests/` — each file is a separate `cargo test` binary, one per
//!   public crate under test. Sharing helpers is done through
//!   `tests/common/mod.rs`.
//! - `src/lib.rs` — small in-process helpers that can be reused
//!   across the integration tests (assertion macros, builders...).
//!
//! # Running
//!
//! ```text
//! cargo test -p schiro-tests
//! ```
