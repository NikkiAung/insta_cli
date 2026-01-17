use std::env;
use std::process;

use grep_file::Config;

fn main() {
    let args : Vec<String> = env::args().collect();

    let config : Config = Config::new(&args).unwrap_or_else(|err : &str| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    println!("Searching for {}", config.query);
    println!("In file {}", config.filename);

    if let Err(e) = grep_file::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}