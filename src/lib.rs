#![no_std]

pub mod driver;
pub mod registers;

pub use driver::{Error, MCP3911, raw_to_uv, sign_extend24, to_u24};

#[cfg(test)]
mod tests;
