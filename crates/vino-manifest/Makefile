# Need a Makefile primer? Head over to isaacs's tutorial: https://gist.github.com/isaacs/62a2d1825d04437c6f08

.PHONY: format clean codegen

V0 := src/v0.rs
V0_DEF := definitions/v0
V0_WIDL := $(shell find $(V0_DEF) -type f -name '*.widl')
V0_TEMPLATES := $(shell find $(V0_DEF)/templates -type f -name '*.hbs')
V0_SOURCES := $(V0_DEF)/manifest.widl $(V0_WIDL) $(V0_TEMPLATES)

CLEAN_FILES:= $(V0)

$(V0): $(V0_SOURCES)
	widl-template $< $(V0_DEF)/templates/rust/root.hbs -p $(V0_DEF)/templates/rust/partials > $@
	rustfmt $@

clean:
	rm -f $(V0)

codegen: $(V0)
