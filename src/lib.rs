use std::error::Error as Err;

mod config;
mod huffman_utils;

pub use config::Config;
pub use huffman_utils::{huff, puff};

pub fn run(config: Config) -> Result<(), Box<dyn Err>> {
    match config.cmd.as_str() {
        "huff" => huffman_utils::huff(&config.file_path),
        "puff" => huffman_utils::puff(&config.file_path),
        _ => Err(Box::from("Invalid command")),
    }
}