TARGET ?= target

CARGO ?= cargo
CARGO_BUILD = $(CARGO) build

R ?= R
RSCRIPT ?= Rscript

NASM = nasm

# Target to build asm examples
asm-ex = asm-ex
asm-ex/% : $(asm-ex)/%.o
	$(LD) -o $(TARGET)/$(asm-ex)/$* $(TARGET)/$<
asm-ex/%.o : $(asm-ex)/%.s
	mkdir -p $(TARGET)/$(asm-ex)
	$(NASM) -f elf64 -o $(TARGET)/$(asm-ex)/$*.o $<

# Target to build Yacc examples
yacc-ex = yacc-ex
yacc-ex-% : src_dir = $(yacc-ex)/$*
yacc-ex-% : local_target = $(TARGET)/$(yacc-ex)/$*
yacc-ex-% : CFLAGS = -Wimplicit-function-declaration
yacc-ex-% : $(yacc-ex)/%/lexer.l $(yacc-ex)/%/parser.y
	mkdir -p $(local_target)
	$(LEX) -o $(local_target)/lex.yy.c $(src_dir)/lexer.l
	$(YACC) -o $(local_target)/y.tab.c $(src_dir)/parser.y
	$(CC) $(local_target)/y.tab.c -ly -lfl -o $(local_target)/a.out $(CFLAGS)

# Target to build book
book = book

define book_build_body
	source("renv/activate.R");                                \
	rmarkdown::render("src/index.Rmd", "pdf_document",        \
										output_file = "$(knit_dir)/index.pdf",  \
										output_dir = "$(knit_dir)",             \
										intermediates_dir = "$(knit_dir)",      \
										knit_root_dir = "$(knit_dir)")
endef

$(book) : local_target = $(TARGET)/$(BOOK)
$(book) : knit_dir = $(realpath .)/$(local_target)
$(book) :
	@echo "Building $@..."
	mkdir -p $(local_target)
	cd $(book) && $(RSCRIPT) -e '$(book_build_body)'
