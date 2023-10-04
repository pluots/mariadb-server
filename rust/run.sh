#!/bin/bash
# Create a docker image with volumes set up. This script is an entrypoint
# and lets you choose an operation

set -eaux

this_path=$(cd "$(dirname "$0")" && pwd)/$(basename "$0")
maria_root="$(dirname "$(dirname "$this_path")")"
rust_dir=${maria_root}/rust
script_dir=${rust_dir}/scripts
dockerfile=${rust_dir}/scripts/dockerfiles/Dockerfile
dockerfile_prebuilt=${rust_dir}/scripts/dockerfiles/Dockerfile.prebuilt
obj_dir=${maria_root}/docker_obj

echo "using root $maria_root"
echo "using script_dir $script_dir"
echo "using dockerfile $dockerfile"
echo "using dockerfile_prebuilt $dockerfile_prebuilt"
echo "using obj_dir $obj_dir"

mkdir -p "$obj_dir"

docker_args=()
# docker_args="$docker_args --volume $maria_root:/checkout"
docker_args=("${docker_args[@]}" "--volume" "$maria_root:/checkout:ro")
docker_args=("${docker_args[@]}" "--volume" "$obj_dir:/obj")
docker_args=("${docker_args[@]}" "--rm")
docker_name="mdb-plugin-test"

second_cmd=""
second_args=()

build_cmd="/checkout/rust/scripts/build_maria.sh"
copy_plugin_cmd="/checkout/rust/scripts/copy_plugins.sh"
test_cmd="/checkout/rust/scripts/run_mtr.sh"
start_cmd="/checkout/rust/scripts/install_run_maria.sh"
launch_quick_cmd="/checkout/rust/scripts/launch_quick.sh"

make_exports="export BUILD_CMD=$build_cmd && export TEST_CMD=test_cmd && export START_CMD=start_cmd"

help="USAGE: ./run.sh build|test|shell|quickstart|quickshell [--nobuild --podman]"

# defaults
launch="docker"
nobuild=""

loopcount=0
for var in "$@"
do
    loopcount=$((loopcount + 1))
    if [ "$loopcount" -eq 1 ]; then
        continue
    fi

    if [ "$var" = "--nobuild" ]; then
        nobuild="true"
        echo nobuild set
    elif [ "$var" = "--podman" ]; then
        launch="podman"
        echo podman set
    else
        echo "unrecognized argument $var"
        exit 1
    fi
done

if [ -z "${1:-""}" ]; then
    echo "$help"
    exit 1
elif [ "$1" = "shell" ]; then
    echo "building for terminal"
    command="$make_exports && /bin/bash"
    docker_args=("${docker_args[@]}" "-it")
elif [ "$1" = "build" ]; then
    echo "building mariadb"
    command="$make_exports && $build_cmd"
elif [ "$1" = "rebuild" ]; then
    echo "build while a container is already open"

    orig_docker_name="$docker_name"
    docker_name="mdb-plugin-rebuild"
    
    command="$make_exports && $build_cmd"
    second_cmd="$launch"
    second_args=("exec" "$orig_docker_name" "/bin/bash" "-c" "$copy_plugin_cmd")
elif [ "$1" = "test" ]; then
    echo "building then testing mariadb"
    command="$make_exports && $build_cmd && $test_cmd"
elif [ "$1" = "start" ]; then
    echo "building then starting mariadb"
    docker_args=("${docker_args[@]}" "-it")
    command="$make_exports && $build_cmd && $start_cmd"
elif [ "$1" = "startshell" ]; then
    echo "launching a shell in a started container"
    "$launch" exec -it "$docker_name" bash
    exit
elif [ "$1" = "quickstart" ]; then
    # Option to avoid reinstalling
    echo "building then launching a preinstalled docker container"
    "$launch" build -f "$dockerfile_prebuilt" --tag mdb-prebuilt-img .

    command="$build_cmd"
    second_cmd="$launch"
    second_args=(
        run
        "${docker_args[@]}"
        "-it"
        "--name"
        "mdb-plugin-prebuilt"
        "mdb-prebuilt-img"
        "/bin/bash"
        "-c"
        "$launch_quick_cmd && /bin/bash"
    )

    if [ "$nobuild" = "true" ]; then
        echo "skipping build"
        "$second_cmd" "${second_args[@]}"
        exit
    fi
elif [ "$1" = "quickshell" ]; then
    # Option to avoid reinstalling
    echo "launching shell in quickstart container"
    "$launch" exec -it mdb-plugin-prebuilt bash
    exit
else
    echo "invalid command $1"
    exit 1
fi

echo "command: $command"
echo "run args:" "${docker_args[@]}"
    
"$launch" build --file "$dockerfile" --tag mdb-rust .

"$launch" run \
    --workdir /obj \
    "${docker_args[@]}" \
    --name "$docker_name" \
    mdb-rust \
    /bin/bash -c "$command"


[ -z "$second_cmd" ] || "$second_cmd" "${second_args[@]}"
