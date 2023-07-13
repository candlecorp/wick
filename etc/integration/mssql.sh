#!/bin/bash

export DOCKER_HOST=${DOCKER_HOST}

container_name="wick_test_$(basename "${BASH_SOURCE[0]}")"

test_pw="${TEST_PASSWORD}"
db="wick_test"
username="SA"

# Source utils
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source $script_dir/utils.sh

up() {
  run_docker $container_name mcr.microsoft.com/mssql/server:2019-latest -e "ACCEPT_EULA=Y" -e "MSSQL_PID=Express" -e "MSSQL_SA_PASSWORD=$test_pw" -p ${MSSQL_PORT}:1433
  local num_tries=10
  local sleep_time=5
  local i=0
  while [ $i -lt $num_tries ]; do
    echo "Waiting for database to be ready..."
    sleep $sleep_time
    docker exec -i $container_name /opt/mssql-tools/bin/sqlcmd -S localhost -U $username -P "$test_pw" -Q "SELECT 1" > /dev/null && break
    i=$((i+1))
  done
}

down() {
  cleanup $container_name
}

init() {
  echo "Dropping database 'wick_test'"
  docker exec -i $container_name /opt/mssql-tools/bin/sqlcmd -S localhost -U $username -P "$test_pw" -Q "DROP DATABASE IF EXISTS $db;"
  sleep 4
  echo "Initializing database 'wick_test'"
  docker exec -i $container_name /opt/mssql-tools/bin/sqlcmd -S localhost -U $username -P "$test_pw" -Q "CREATE DATABASE $db;"
  echo "Creating table 'users'"
  docker exec -i $container_name /opt/mssql-tools/bin/sqlcmd -S localhost -U $username -P "$test_pw" -d $db -Q "CREATE TABLE users (id INT IDENTITY(1,1) PRIMARY KEY, name VARCHAR(255) NOT NULL, email VARCHAR(255) NOT NULL);"
  echo "Creating user 'Test User'"
  docker exec -i $container_name /opt/mssql-tools/bin/sqlcmd -S localhost -U $username -P "$test_pw" -d $db -Q "INSERT INTO users (name, email) VALUES ('Test User', 'test_user@example.com');"
  echo "Creating table 'num_types'"
  docker exec -i $container_name /opt/mssql-tools/bin/sqlcmd -S localhost -U $username -P "$test_pw" -d $db -Q "CREATE TABLE num_types (u8 tinyint, i16 smallint, i32 int, i64 bigint, db_decimal decimal, db_numeric numeric, f32 real, f64 float);"
}

handle "$@"