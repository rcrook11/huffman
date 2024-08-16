pub struct Config {
    pub cmd: String,
    pub file_path: String,
}

impl Config {
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
