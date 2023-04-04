#!/bin/bash
# build_and_run_registry.sh

# Set container name
container_name="simple_registry"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Function to stop and remove the container
cleanup() {
    echo "Cleaning up..."
    docker stop $container_name
    docker rm $container_name
}

# Check if the container with the same name exists
if [ "$(docker ps -a -q -f name=$container_name)" ]; then
    # Stop and remove the existing container
    cleanup
fi

docker build -t $container_name "$script_dir"
sleep 2
docker run -d -p 8888:5000 --name simple_registry simple_registry
sleep 2