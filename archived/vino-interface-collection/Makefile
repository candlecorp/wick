.PHONY: all deps codegen build clean doc test
noop =
space = $(noop) $(noop)

SCHEMA_DIR := ./schemas
GENERATED_DIR := ./src/generated
GENERATED_MODULE:=$(GENERATED_DIR)/mod.rs

# Gets list of widl files in SCHEMA_DIR
SCHEMAS=$(wildcard ${SCHEMA_DIR}/*.widl)

# Translates those to GENERATED_DIR/file.rs (transposing hyphens to underscores)
GENERATED_COMPONENTS=$(patsubst %.widl,%.rs,$(patsubst ${SCHEMA_DIR}%,${GENERATED_DIR}%,$(subst -,_,$(SCHEMAS))))
GENERATED_FS_NAMES=$(patsubst %.rs,%,$(notdir $(GENERATED_COMPONENTS)))
GENERATED_MOD_IMPORTS=$(patsubst $(GENERATED_DIR)/%,pub (crate) mod %;,$(patsubst %.rs,%,$(GENERATED_COMPONENTS)))


define WIDL_CODEGEN
$(patsubst %.widl,%.rs,$(patsubst ${SCHEMA_DIR}%,${GENERATED_DIR}%,$(subst -,_,$(1)))): $1 | ${GENERATED_DIR}
endef

$(foreach schema,$(SCHEMAS),$(eval $(call WIDL_CODEGEN,$(schema))))

$(GENERATED_MODULE):
	echo "/* This is generated, do not edit by hand */$(GENERATED_MOD_IMPORTS)" >> $@; \

$(GENERATED_DIR):
	mkdir $@

$(GENERATED_COMPONENTS):
	vino-codegen rust "$<" -t provider-component | rustfmt --quiet >$(subst -,_,$@)

all: deps codegen build sign claims

deps:
	mkdir -p build

codegen:
	echo ${GENERATED_COMPONENTS}
	echo ${SCHEMAS}
	# for file in ${SCHEMA_DIR}/*.widl; do \
	# 	FS_NAME=`basename -s .widl $$file | tr '-' '_'`; \
	# 	make ${GENERATED_DIR}/$$FS_NAME.rs; \
	# done; \
	make $(GENERATED_COMPONENTS)
	make $(GENERATED_MODULE)
	# make src/components.rs

build: codegen
	cargo build --release

clean:
	# cargo clean
	# rm -Rf build
	rm -f $(GENERATED_COMPONENTS) $(GENERATED_MODULE)

doc:

test:
	cargo test