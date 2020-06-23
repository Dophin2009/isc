TARGET ?= target

CARGO ?= cargo
CARGO_BUILD = $(CARGO) build

R ?= R
RSCRIPT ?= Rscript

# Target to build book
BOOK = book
BOOK_TARGET = $(TARGET)/$(BOOK)

define BOOK_BUILD_BODY
	source("renv/activate.R");																\
	rmarkdown::render("src/index.Rmd", "pdf_document",				\
										output_file = "$(KNIT_DIR)/index.pdf",	\
										output_dir = "$(KNIT_DIR)",							\
										intermediates_dir = "$(KNIT_DIR)",			\
										knit_root_dir = "$(KNIT_DIR)")
endef

$(BOOK) : KNIT_DIR = $(realpath .)/$(BOOK_TARGET)
$(BOOK) :
	@echo "Building $@..."
	mkdir -p $(BOOK_TARGET)
	cd $(BOOK) && \
		$(RSCRIPT) -e '$(BOOK_BUILD_BODY)'
