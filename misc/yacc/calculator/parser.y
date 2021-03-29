%{
#include <ctype.h>
#include <stdio.h>
#define YYSTYPE double
%}

%token NUMBER

%left '+' '-'
%left '*' '/'
%right UMINUS

%%

lines :   lines expr '\n' { printf("%g\n", $2); }
      |   lines '\n'
      |
      ;

expr  :   expr '+' expr   { $$ = $1 + $3; }
      |   expr '-' expr   { $$ = $1 - $3; }
      |   expr '*' expr   { $$ = $1 * $3; }
      |   expr '/' expr   { $$ = $1 / $3; }
      |   '(' expr ')'    { $$ = $2; }
      |   '-' expr        %prec UMINUS { $$ = - $2; }
      |   NUMBER
      ;

%%

#include "lex.yy.c"

int yywrap() {
  return 1;
}
