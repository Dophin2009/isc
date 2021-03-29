and     &&
or      \|\|

%%

[ ]     { /* ignore whitespace */ }

true    { return TRUE; }

false   { return FALSE; }

{and}   { return AND; }

{or}    { return OR; }

!       { return NOT; }

\n      { return yytext[0]; }

.       { }
