# Need a Makefile primer? Head over to isaacs's tutorial: https://gist.github.com/isaacs/62a2d1825d04437c6f08

V0 := src/v0.rs
V0_schema := json-schema/v0/manifest.json
V0_docs := docs/v0.html
V0_DEF := definitions/v0
V0_WIDL := $(shell find $(V0_DEF) -type f -name '*.widl')
V0_TEMPLATES := $(shell find $(V0_DEF)/templates -type f -name '*')
V0_DOC_TEMPLATES := $(shell find $(V0_DEF)/doc-templates -type f -name '*.hbs')
V0_SOURCES := $(V0_DEF)/manifest.widl $(V0_WIDL) $(V0_TEMPLATES)

CLEAN_FILES:= $(V0) $(V0_schema) $(V0_docs)

$(V0): $(V0_SOURCES) $(V0_TEMPLATES)
	@widl-template $< $(V0_DEF)/templates/rust/root.hbs -p $(V0_DEF)/templates/rust/partials > $@
	@rustfmt $@

$(V0_schema): $(V0_SOURCES) $(V0_TEMPLATES)
	@widl-template $(V0_DEF)/manifest.widl $(V0_DEF)/templates/$(basename $@).hbs -p $(V0_DEF)/templates/json-schema/partials > $@
	@prettier --write $@

.PHONY: clean
clean:
	@rm -f $(CLEAN_FILES)

.PHONY: codegen
codegen: $(V0) $(V0_schema) $(V0_docs)

docs:
	@mkdir docs

$(V0_docs): $(V0_SOURCES) $(V0_DOC_TEMPLATES) docs
	# cargo doc --no-deps
	@widl-template $< $(V0_DEF)/doc-templates/rust/root.hbs -p $(V0_DEF)/doc-templates/rust/partials > $@
