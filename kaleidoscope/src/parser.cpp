#include "parser.hpp"



std::unique_ptr<ExprAST> Parser::log_error(const char *str) {
  fprintf(stderr, "LogError: %s\n", str);
  return nullptr;
}

std::unique_ptr<PrototypeAST> Parser::log_error_p(const char *str) {
  this->log_error(str);
  return nullptr;
}
