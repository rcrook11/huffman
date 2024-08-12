use std::env;
use huffman::Config;
use huffman::run;

fn main() {
    let config = Config::build(env::args())
        .unwrap_or_else(|err| {
            eprintln!("Problem parsing arguments: {}", err);
            std::process::exit(1);
        });

    if let Err(err) = run(config) {
        println!("Application error: {}", err);
        std::process::exit(1);
    }
}
