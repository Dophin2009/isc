use nir::Compiler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let compiler = Compiler::new();

    let input = "pub struct A {}";
    let ast = compiler.parse(input.chars())?;
    println!("{:?}", ast);

    Ok(())
}
