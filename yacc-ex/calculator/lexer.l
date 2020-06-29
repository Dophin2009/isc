number      [0-9]+\.?|[0-9]*\.[0-9]+

%%

[ ]         {}

{number}    { sscanf(yytext, "%lf", &yylval);
              return NUMBER;
            }

.|\n        { return yytext[0]; }

%%
