%{
#include <stdio.h>

typedef int bool;
#define true 1
#define false 0
#define YYSTYPE bool
%}

%token TRUE
%token FALSE
%token NOT
%token AND
%token OR

%left AND OR
%right NOT

%%

lines :   lines expr '\n'   { if ($2) { printf("true\n"); } else { printf("false\n"); } }
      |   lines '\n'
      |
      ;

expr  :   expr AND expr     { $$ = $1 && $3; }
      |   expr OR expr      { $$ = $1 || $3; }
      |   '(' expr ')'      { $$ = $2; }
      |   NOT expr          %prec NOT { $$ = !$2; }
      |   TRUE              { $$ = true; }
      |   FALSE             { $$ = false; }
      ;

%%

#include "lex.yy.c"

int yywrap() {
  return 1;
}
