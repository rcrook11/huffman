//! Configuration module for the huffman cli tool

/// Configuration struct for the huffman cli tool
pub struct Config {
    pub cmd: String,
    pub file_path: String,
}

impl Config {
    /// Builds a new Config struct from the provided arguments
    ///
    /// # Arguments
    ///
    /// * `args` - An iterator of arguments passed into the cli tool
    ///
    /// # Returns
    ///
    /// A Result containing the Config struct if successful, or an error message if not
    ///
    /// # Errors
    ///
    /// * Throws an error if the number of arguments is incorrect
    pub fn build(mut args: impl Iterator<Item = String> + ExactSizeIterator) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Not enough arguments");
        } else if args.len() > 3 {
            return Err("Too many arguments");
        }

        args.next();

        let cmd = match args.next() {
            Some(arg) => arg,
            None => return Err("No command provided"),
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("No file path provided"),
        };

        Ok(Config{ cmd, file_path })
    }
}
