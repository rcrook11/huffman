//! # Huffman
//!
//! `huffman` Library module, a simple implementation of the Huffman coding algorithm in Rust.
//!
//! ## Usage
//!
//! ```rust
//! huffman::huff("path/to/file");
//!
//! huffman::puff("path/to/file.huff");
//!
//! ```

/// Configuration module for the huffman cli tool
mod config;

/// Huffman coding implementation
mod huffman_utils;

use std::error::Error as Err;

pub use config::Config;
pub use huffman_utils::{huff, puff};

/// Runs the huffman cli tool with the provided configuration
///
/// # Arguments
///
/// * `config` - The configuration for the huffman cli tool
///
/// # Returns
///
/// A Result containing nothing if successful, or an error message if not
pub fn run(config: Config) -> Result<(), Box<dyn Err>> {
    match config.cmd.as_str() {
        "huff" => huffman_utils::huff(&config.file_path),
        "puff" => huffman_utils::puff(&config.file_path),
        _ => Err(Box::from("Invalid command")),
    }
}