# Need a Makefile primer? Head over to isaacs's tutorial: https://gist.github.com/isaacs/62a2d1825d04437c6f08

VERSIONS:=v0 v1

CLEAN_FILES:= $(wildcard docs/*) $(wildcard src/v*.rs)  $(wildcard json-schema/**/*)

src/v%.rs: definitions/v%/manifest.apex definitions/v%/*/*/*
	@apex-template $< definitions/v$*/templates/rust/root.hbs -p definitions/v$*/templates/rust/partials > $@

json-schema/v%/manifest.json: definitions/v%/manifest.apex definitions/v%/*/*/*
	@apex-template $< definitions/v$*/templates/json-schema/manifest.hbs -p definitions/v$*/templates/json-schema/partials > $@
	@prettier --write $@

.PHONY: clean
clean:
	@rm -f $(CLEAN_FILES)

.PHONY: test
test:
	@cargo test

.PHONY: codegen
codegen:
	@$(MAKE) src/v0.rs
	@$(MAKE) src/v1.rs
	@mkdir -p json-schema/v0
	@$(MAKE) json-schema/v0/manifest.json
	@mkdir -p json-schema/v1
	@$(MAKE) json-schema/v1/manifest.json
	@$(MAKE) json-schema/manifest.json
	@$(MAKE) docs/v0.html
	@$(MAKE) docs/v1.html

json-schema/manifest.json: json-schema/v0/manifest.json json-schema/v1/manifest.json
	@jq  \
		--slurpfile v0_manifest json-schema/v0/manifest.json \
		--slurpfile v1_manifest json-schema/v1/manifest.json \
		'.["$$defs"] += $$v0_manifest[0] + $$v1_manifest[0]' \
		definitions/json-schema.json.tpl > $@

docs/v%.html: definitions/v%/manifest.apex definitions/v%/*/*/*
	@mkdir -p docs
	@apex-template $< definitions/v$*/doc-templates/rust/root.hbs -p definitions/v$*/doc-templates/rust/partials > $@
