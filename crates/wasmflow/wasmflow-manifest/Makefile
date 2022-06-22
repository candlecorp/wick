# Need a Makefile primer? Head over to isaacs's tutorial: https://gist.github.com/isaacs/62a2d1825d04437c6f08

VERSIONS:=v0 v1

CLEAN_FILES:= $(wildcard docs/**/*) $(wildcard src/v*.rs)  $(wildcard json-schema/**/*)

src/v%.rs: definitions/v%/manifest.apex definitions/v%/*/*/*
	apex-template $< definitions/v$*/templates/rust/root.hbs -p definitions/v$*/templates/rust/partials > $@

json-schema/v%/manifest.json: definitions/v%/manifest.apex definitions/v%/*/*/*
	apex-template $< definitions/v$*/templates/$(basename $@).hbs -p definitions/v$*/templates/json-schema/partials > $@
	prettier --write $@

.PHONY: clean
clean:
	@rm -f $(CLEAN_FILES)

.PHONY: codegen
codegen:
	$(MAKE) src/v0.rs
	$(MAKE) src/v1.rs

docs/v%.html: definitions/v%/manifest.apex definitions/v%/*/*/*
	mkdir -p docs
	# cargo doc --no-deps
	@apex-template $< definitions/v$*/doc-templates/rust/root.hbs -p definitions/v$*/doc-templates/rust/partials > $@
