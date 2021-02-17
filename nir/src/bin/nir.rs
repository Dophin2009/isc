use nir::Compiler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let compiler = Compiler::new();

    let input = r#"pub struct A {
        b: u8,
        c: str,
    }"#;

    compiler.parse_emit(input.chars());

    Ok(())
}
