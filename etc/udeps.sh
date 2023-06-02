#!/bin/bash

PROJECT="$1"

deps=$(cargo read-manifest --manifest-path ${PROJECT}/Cargo.toml | jq -r '.dependencies[] | select(.kind==null) | .name' | tr '-' '_')

UNUSED=""
for dep in $deps; do
    NUM_FILES=$(egrep -rl "use ${dep}|${dep}::|${dep} as |extern crate ${dep}" $PROJECT/src)
    if [[ "$NUM_FILES" ==  "" ]]; then
      UNUSED="$UNUSED$dep\n"
    fi
done

if [[ "$UNUSED" != "" ]]; then
  echo Project $PROJECT has unused dependencies:
  echo -e $UNUSED
fi

# xargs -n1 -I {}  egrep -rl 'use {}|{}::' crates/bins/wick/src
exit 0