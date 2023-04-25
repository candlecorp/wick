#!/bin/bash

export DOCKER_HOST=${DOCKER_HOST}

container_name="wick_test_$(basename "${BASH_SOURCE[0]}")"

# Source utils
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source $script_dir/utils.sh

up() {
  run_docker $container_name kennethreitz/httpbin -p ${HTTPBIN_PORT}:80
}

down() {
  cleanup $container_name
}

init() {
  echo "Initialized"
}

handle "$@"