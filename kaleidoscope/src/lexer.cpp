#include "lexer.hpp"

Lexer::Lexer() { this->last = ' '; }

Token Lexer::next() {
  this->last = ' ';
  while (isspace(this->last)) {
    this->last = getchar();
  }

  if (isalpha(this->last)) {
    std::string ident_str = std::to_string(this->last);
    this->last = getchar();
    while (isalnum(this->last)) {
      ident_str += this->last;
    }

    if (ident_str == "def") {
      return Token::Def();
    } else if (ident_str == "extern") {
      return Token::Extern();
    } else {
      return Token::Ident(ident_str);
    }
  }

  if (isdigit(this->last) || this->last == '.') {
    std::string num_str;
    do {
      num_str += this->last;
      this->last = getchar();
    } while (isdigit(this->last) || this->last == '.');

    auto val = strtod(num_str.c_str(), 0);
    return Token::Number(val);
  }

  if (this->last == '#') {
    do {
      this->last = getchar();
    } while (this->last != EOF && this->last != '\n' && this->last != '\r');
    if (this->last != EOF) {
      return Lexer::next();
    }
  }

  if (this->last == EOF) {
    return Token::Eof();
  }

  int c = this->last;
  this->last = getchar();
  return Token::Char(c);
}
