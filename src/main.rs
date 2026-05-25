use std::io::{self, Read, Write};
use std::process::ExitCode;

use json_parser::{parse, to_string_pretty};

fn main() -> ExitCode {
    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        let _ = writeln!(io::stderr(), "error reading stdin: {e}");
        return ExitCode::from(2);
    }
    match parse(&input) {
        Ok(value) => {
            println!("{}", to_string_pretty(&value, 2));
            ExitCode::SUCCESS
        }
        Err(e) => {
            let _ = writeln!(io::stderr(), "parse error: {e}");
            ExitCode::FAILURE
        }
    }
}
