#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

MODULES=""

NATIVE_HOST_DIR="${SCRIPT_DIR}/.."

GENERATED_SCHEMA_DIR="${NATIVE_HOST_DIR}/generated"

TEMPLATE="${SCRIPT_DIR}/template.ejs"

GENERATED_MODULE_FILE="$GENERATED_SCHEMA_DIR/mod.rs"
NATIVE_ACTOR_MODULE_FILE="$NATIVE_HOST_DIR/mod.rs"

echo "// This file is generated, do not edit" >$GENERATED_MODULE_FILE

MOD_LINES="
// This file is generated, do not edit
use crate::native_component_actor::NativeActor;
pub(crate) mod generated;
"

FN_LINES="
pub(crate) fn get_native_actor(name:String) -> Option<Box<dyn NativeActor>> {
    match name.as_str() {
"

for WIDL_PATH in ${SCRIPT_DIR}/*.widl; do
  WIDL_FILE="${WIDL_PATH/$SCRIPT_DIR\//}"
  MODULE_NAME="${WIDL_FILE/\.widl/}"
  FS_MODULE_NAME=$(echo $MODULE_NAME | tr '-' '_')
  FS_MODULE_PATH="${GENERATED_SCHEMA_DIR}/${FS_MODULE_NAME}.rs"
  echo "Using $WIDL_FILE to generate $FS_MODULE_NAME.rs"
  widl-template $WIDL_PATH $TEMPLATE >$FS_MODULE_PATH
  echo "pub(crate) mod ${FS_MODULE_NAME};" >>$GENERATED_MODULE_FILE
  MOD_LINES="$MOD_LINES"$'\n'"pub(crate) mod ${FS_MODULE_NAME};"
  FN_LINES="$FN_LINES"$'\n'"\"vino::${MODULE_NAME}\" => Some(Box::new(${FS_MODULE_NAME}::Actor {})),"
  rustfmt $FS_MODULE_PATH 2>/dev/null
done
echo "$MOD_LINES"
echo "$MOD_LINES" >$NATIVE_ACTOR_MODULE_FILE
echo "$FN_LINES"
echo "$FN_LINES" >>$NATIVE_ACTOR_MODULE_FILE

(
  cat <<EOF
        _ => None,
    }
}
EOF
) >>$NATIVE_ACTOR_MODULE_FILE
