#include <string>

class ExprAST {
public:
  virtual ~ExprAST() {}
};

class NumberExprAST : public ExprAST {
  double val;

public:
  NumberExprAST(double v) : val(v) {}
};

class VariableExprAST : public ExprAST {
  std::string ident;

public:
  VariableExprAST(const std::string &ident) : ident(ident) {}
};
