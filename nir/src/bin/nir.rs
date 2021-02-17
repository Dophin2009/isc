use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

use nir::Compiler;
use utf8_chars::BufReadCharsExt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let compiler = Compiler::new();

    let stdin = io::stdin();
    let mut input: Box<dyn BufRead> = match env::args().nth(1) {
        Some(filepath) => {
            let file = File::open(filepath)?;
            Box::new(BufReader::new(file))
        }
        None => Box::new(stdin.lock()),
    };

    // TODO: find better way to do this
    let chars = input.chars().filter_map(|c| c.ok());

    let stderr = io::stderr();
    compiler.parse_emit(&mut stderr.lock(), chars)?;

    Ok(())
}
