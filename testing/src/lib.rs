use std::fs;
use std::error::Error;
use std::env;

pub fn run(config : Config) -> Result<(), Box<dyn Error>>{
    // let contents: String = fs::read_to_string(config.filename).expect("Something went wrong reading the file");

    // Box<dyn Error> cleans the code
    let contents: String = fs::read_to_string(config.filename)?;

    // let lines = search(&config.query, &contents);
    // println!("{:?}", lines);

    // println!("{}", "*".repeat(100));

    let results = if config.case_insensitive {
        search(&config.query, &contents)
    } else {
        search_case_insensitive(&config.query, &contents)
    };

    for line in results {
        println!("{}", line);
    }

    Ok(())
}

pub struct Config {
    pub query : String,
    pub filename : String,
    pub case_insensitive : bool,
}

impl Config {
    pub fn new(args : &[String]) -> Result<Config, &str> {

        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let query: String = args[1].clone();
        let filename: String = args[2].clone();

        // export CASE_INSENSITIVE=true
        // unset CASE_INSENSITIVE
        let case_insensitive: bool = env::var("CASE_INSENSITIVE").is_err();

        Ok(Config { query, filename, case_insensitive })
    }
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {

    // Because you return references to lines inside contents.

    // That means these lines are not new Strings, they are slices pointing into contents.

    // So Rust needs to guarantee: “The returned &str will not outlive contents.”

    // 'a means in Simple analogy "If contents is a book, the return value is a list of page references."

    let mut results = Vec::new();

    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }

    results
} 

pub fn search_case_insensitive<'a> (
    query : &str,
    contents : &'a str,
) -> Vec<&'a str> {
    let query = query.to_lowercase();
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }

    results
}

#[cfg(test)] 
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query: &str = "duct";
        let contents: &str = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents))
    }

    #[test]
    fn case_insensitive() {
        let query: &str = "rUsT";
        let contents: &str = "\
Rust:
safe, fast, productive.
Pick three.
Trust me";

        assert_eq!(vec!["Rust:", "Trust me"], search_case_insensitive(query, contents))
    }
}