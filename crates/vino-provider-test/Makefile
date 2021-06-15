.PHONY: all deps codegen build clean doc test

SCHEMA_DIR:=./schemas
GENERATED_DIR:=./src/components/generated

src/components/generated.rs:
	echo "// This is generated, do not edit by hand" >> $@; \
	for file in src/components/generated/*.rs; do \
		FS_NAME=`basename -s .rs $$file | tr '-' '_'`; \
		echo "pub (crate) mod $${FS_NAME};" >> $@; \
	done

src/components/generated/%.rs:
	BASENAME=$(shell basename -s .rs "$@"); \
	WIDLNAME=$(shell basename -s .rs "$@" | tr '_' '-'); \
	vino-codegen rust "${SCHEMA_DIR}/$$WIDLNAME.widl" -t provider-component | rustfmt --quiet >${GENERATED_DIR}/$$BASENAME.rs

src/components.rs:
	echo "" > ./all_schemas.widl;\
	for file in ${SCHEMA_DIR}/*.widl; do \
		FS_NAME=`basename -s .widl $$file`; \
		echo "import * from \"$${FS_NAME}\"" >> ./all_schemas.widl; \
	done; \
	vino-codegen rust "./all_schemas.widl" -t provider-integration -r ${SCHEMA_DIR} | rustfmt --quiet >$@; \
	# rm ./all_schemas.widl

all: deps codegen build sign claims

deps:
	mkdir -p build

codegen:
	for file in ${SCHEMA_DIR}/*.widl; do \
		FS_NAME=`basename -s .widl $$file | tr '-' '_'`; \
		make ${GENERATED_DIR}/$$FS_NAME.rs; \
	done; \
	make src/components/generated.rs; \
	make src/components.rs; \

build: codegen
	cargo build --release

clean:
	# cargo clean
	# rm -Rf build
	rm src/components/generated.rs
	rm src/components/generated/*.rs
	rm src/components.rs

doc:

test:
	cargo test