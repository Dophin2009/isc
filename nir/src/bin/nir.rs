use std::io;

use nir::Compiler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let compiler = Compiler::new();

    let input = r#"pub struct A {
        b: u8,
        c: str,
    }"#;

    let stderr = io::stderr();
    compiler.parse_emit(&mut stderr.lock(), input.chars())?;

    Ok(())
}
