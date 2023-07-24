#!/bin/bash

# Source utils
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source $script_dir/utils.sh

up() {
  # create the sqlite database if it doesn't exist
  if [ ! -f $SQLITE_DB ]; then
    sqlite3 $SQLITE_DB < $script_dir/sqlite/schema.sql
    sqlite3 $SQLITE_DB < $script_dir/sqlite/init.sql
  fi

}

down() {
  rm $SQLITE_DB
}

init() {
  # Already initialized, exit successfully
  exit 0
}

handle "$@"