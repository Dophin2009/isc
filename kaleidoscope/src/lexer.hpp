#include <string>
#include <variant>

struct Token {
  struct TokenEof {};
  struct TokenDef {};
  struct TokenExtern {};
  struct TokenIdent {
    std::string str;
    TokenIdent(std::string s) : str{s} {}
  };
  struct TokenNumber {
    double val;

    TokenNumber(double v) : val{v} {}
  };
  struct TokenChar {
    int val;
    TokenChar(int v) : val{v} {}
  };
  using variants = std::variant<TokenEof, TokenDef, TokenExtern, TokenIdent,
                                TokenNumber, TokenChar>;
  variants value;

  Token(variants value) { this->value = value; }
  static Token Eof() { return Token(Token::TokenEof()); };
  static Token Def() { return Token(Token::TokenDef()); };
  static Token Extern() { return Token(Token::TokenExtern()); };
  static Token Ident(std::string str) { return Token(Token::TokenIdent(str)); };
  static Token Number(double val) { return Token(Token::TokenNumber(val)); };
  static Token Char(int val) { return Token(Token::TokenChar(val)); };
};

class Lexer {
private:
  int last;

public:
  Lexer();

  Token next();
};
