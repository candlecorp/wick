


# Function to stop and remove the container
cleanup() {
  local container_name="$1"
  echo "Cleaning up..."

  if [ "$(docker ps -a -q -f name=$container_name)" ]; then
    echo "Stopping and removing container $container_name"
    run_cmd "docker stop $container_name"
    run_cmd "docker rm $container_name"
  fi
}

run_cmd() {
  echo "$@"
  eval "$@"
}

run_docker() {
  local container_name="$1"
  local image_name="$2"
  local args="${@:3}"

  # Check if the container with the same name exists
  if [ "$(docker ps -a -q -f name=$container_name)" ]; then
      # Stop and remove the existing container
      cleanup $container_name
  fi

  echo "Running container $container_name"
  run_cmd "docker run -d $args --name $container_name $image_name"
}

build() {
  local container_name="$1"
  local script_dir="$2"

  echo "Building container $container_name"
  run_cmd "docker build -t $container_name $script_dir"
}

handle() {

  for command in "$@"
  do
    if [ "$command" == "up" ]; then
      up
    elif [ "$command" == "down" ]; then
      down
    elif [ "$command" == "init" ]; then
      init
    else
      echo "Usage: $0 [up|down|init]"
      exit 1
    fi
  done
}