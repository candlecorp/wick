#!/bin/bash

export DOCKER_HOST=${DOCKER_HOST}

container_name="wick_test_$(basename "${BASH_SOURCE[0]}")"

# Source utils
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source $script_dir/utils.sh

up() {
  # docker build -t $container_name -F "$script_dir"/Dockerfile.registry
  run_docker $container_name registry:2 -p ${DOCKER_REGISTRY_PORT}:5000
}

down() {
  cleanup $container_name
}

init() {
  echo "Initializing"
}

handle "$@"