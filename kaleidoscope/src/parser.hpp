#include <memory>
#include <string>
#include <vector>

#include "lexer.hpp"

struct ExprAST {
  virtual ~ExprAST() {}
};

struct NumberExprAST : public ExprAST {
  double val;

  NumberExprAST(double v) : val(v) {}
};

struct VariableExprAST : public ExprAST {
  std::string ident;

  VariableExprAST(const std::string &ident) : ident(ident) {}
};

struct BinaryExprAST : public ExprAST {
  char op;
  std::unique_ptr<ExprAST> lhs, rhs;

  BinaryExprAST(char op, std::unique_ptr<ExprAST> lhs,
                std::unique_ptr<ExprAST> rhs)
      : op(op), lhs(std::move(lhs)), rhs(std::move(rhs)) {}
};

struct CallExprAST : public ExprAST {
  std::string callee;
  std::vector<std::unique_ptr<ExprAST>> args;

  CallExprAST(const std::string &callee,
              std::vector<std::unique_ptr<ExprAST>> args)
      : callee(callee), args(std::move(args)) {}
};

struct PrototypeAST : public ExprAST {
  std::string name;
  std::vector<std::string> args;

  PrototypeAST(const std::string &name, std::vector<std::string> args)
      : name(name), args(std::move(args)) {}
};

struct FunctionAST {
  std::unique_ptr<PrototypeAST> proto;
  std::unique_ptr<ExprAST> body;

  FunctionAST(std::unique_ptr<PrototypeAST> proto,
              std::unique_ptr<ExprAST> body)
      : proto(std::move(proto)), body(std::move(body)) {}
};

class Parser {
  std::shared_ptr<Lexer> lexer;

public:
  Parser(std::shared_ptr<Lexer> lexer) : lexer(lexer) {}

  std::unique_ptr<ExprAST> parse_number_expr() {
    auto node = std::make_unique<NumberExprAST>();
  }

  std::unique_ptr<ExprAST> log_error(const char *str);
  std::unique_ptr<PrototypeAST> log_error_p(const char *str);
};
