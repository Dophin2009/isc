use lexgen::lexer;

#[derive(Debug)]
pub enum Token {
    Ident(String),
    Integer(i64),
    None,
}

lexer! {
    pub fn next_token(text) -> Token;

    "" => { Token::None }
}

fn main() {
    let mut input = String::new();
    while let Some(token) = next_token(&mut input) {
        print!("{:?} ", token);
    }
}
