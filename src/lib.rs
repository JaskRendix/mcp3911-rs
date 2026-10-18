#![no_std]

//! # MCP3911
//!
//! Driver for the Microchip MCP3911 dual-channel 24-bit AFE.
//!
//! ## Voltage Conversion
//!
//! ```rust
//! use mcp3911_rs::{raw_to_uv, registers::Gain};
//!
//! let uv = raw_to_uv(1_000_000, Gain::X1, 1200);
//!
//! assert!(uv > 0);
//! ```
//!
//! ## Timing Utilities
//!
//! ```rust
//! use mcp3911_rs::registers::{
//!     sample_rate_hz,
//!     Oversampling,
//!     Prescale,
//! };
//!
//! let rate = sample_rate_hz(
//!     4_000_000,
//!     Oversampling::O256,
//!     Prescale::Mclk1,
//! );
//!
//! assert_eq!(rate, 3906);
//! ```

pub mod driver;
pub mod registers;

pub use driver::{Error, MCP3911, raw_to_uv, sign_extend24, to_u24};

#[cfg(test)]
mod tests;
