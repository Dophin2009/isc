CFLAGS = -Wimplicit-function-declaration

.PHONY: build 

default : build

build : lexer.l parser.y
	$(LEX) -o lex.yy.c lexer.l
	$(YACC) -o y.tab.c parser.y
	$(CC) y.tab.c -ly -lfl -o calculator $(CFLAGS)
