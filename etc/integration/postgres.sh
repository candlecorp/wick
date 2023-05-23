#!/bin/bash

export DOCKER_HOST=${DOCKER_HOST}

container_name="wick_test_$(basename "${BASH_SOURCE[0]}")"

username=postgres
db=wick_test
pw="${TEST_PASSWORD}"

# Source utils
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source $script_dir/utils.sh

up() {
  run_docker $container_name postgres -p ${POSTGRES_PORT}:5432 -e POSTGRES_PASSWORD="$pw"
  local num_tries=10
  local sleep_time=5
  local i=0
  while [ $i -lt $num_tries ]; do
    echo "Waiting for database to be ready..."
    sleep $sleep_time
    docker exec -it $container_name psql -U $username -h localhost -c "SELECT 1" > /dev/null && break
    i=$((i+1))
  done
}

down() {
  cleanup $container_name
}

init() {
  echo "Dropping test db '$db'"
  docker exec -it $container_name psql -U $username -h localhost -c "DROP DATABASE IF EXISTS $db;"
  echo "Initializing database '$db'"
  docker exec -it $container_name psql -U $username -h localhost -c "CREATE DATABASE $db;"
  echo "Creating table 'user'"
  docker exec -it $container_name psql -U $username -h localhost -d $db -c "CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255) NOT NULL, email VARCHAR(255) NOT NULL);"
  echo "Creating user 'Test User'"
  docker exec -it $container_name psql -U $username -h localhost -d $db -c "INSERT INTO users (name, email) VALUES ('Test User', 'test_users@example.com');"
  echo "Creating table 'num_types'"
  docker exec -it $container_name psql -U $username -h localhost -d $db -c "CREATE TABLE num_types (id SERIAL PRIMARY KEY, i16 smallint, i32 integer, i64 bigint, db_decimal decimal, db_numeric numeric, f32 real, f64 double precision);"
}

handle "$@"