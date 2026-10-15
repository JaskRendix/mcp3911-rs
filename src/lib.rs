#![no_std]

pub mod driver;
pub mod registers;

pub use driver::{raw_to_uv, sign_extend24, to_u24, Error, MCP3911};

#[cfg(test)]
mod tests;
