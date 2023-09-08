#!/bin/bash
# Create a docker image with volumes set up. This script is an entrypoint
# and lets you choose an operation

set -eaux

help=$(cat <<-END
USAGE: ./run.sh [action] [flags]

Actions:
    build: build from source and exit
    rebuild: rebuild files while a server is running, to avoid startup time. Works
        with the non-quick commands.
    shell: build from source then enter a shell with the built files
    test: build from source then test with MTR
    start: build from source then launch the server
    startshell: launch a shell on a started container
    quickstart: build plugins from source, launch a prebuilt MariaDB container,
        copy the plugins
    quickshell: enter a shell on a container started with 'quickstart'
    quickrebuild: like 'rebuild' but for 'quick' commands
Flags:
    --nobuild: when used with 'quickstart', don't rebuild before launching
    --podman: use 'podman' instead of 'docker'

All build actions take place in docker, output are in 'docker_obj'
END
)


# defaults
launcher="docker"
nobuild=""

loopcount=0
for var in "$@"
do
    # Check all args, skip the first
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
        echo "$help"
        exit 1
    fi
done

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

build_cmd="/checkout/rust/scripts/build.sh"
copy_plugin_cmd="/checkout/rust/scripts/copy_plugins.sh"
install_cmd="/checkout/rust/scripts/install.sh"
launch_quick_cmd="/checkout/rust/scripts/launch_quick.sh"
test_cmd="/checkout/rust/scripts/run_mtr.sh"
start_safe_cmd="/checkout/rust/scripts/start_safe.sh"
start_cmd="/checkout/rust/scripts/start.sh"

make_exports="export BUILD_CMD=$build_cmd &&
    export INSTALL_CMD=install_cmd &&
    export TEST_CMD=test_cmd &&
    export START_CMD=start_cmd &&
    export START_SAFE_CMD=start_safe_cmd"

if [ "$nobuild" = "true" ]; then
    build_cmd="echo skipping build"
fi

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
    second_cmd="$launcher"
    second_args=("exec" "$orig_docker_name" "/bin/bash" "-c" "$copy_plugin_cmd")
elif [ "$1" = "test" ]; then
    echo "building then testing mariadb"
    command="$make_exports && $build_cmd && $install_cmd && $test_cmd"
elif [ "$1" = "start" ]; then
    echo "building then starting mariadb"
    docker_args=("${docker_args[@]}" "-it")
    command="$make_exports && $build_cmd && $install_cmd && $start_safe_cmd"
elif [ "$1" = "startshell" ]; then
    echo "launching a shell in a started container"
    "$launcher" exec -it "$docker_name" bash
    exit
elif [ "$1" = "quickstart" ]; then
    # Option to avoid reinstalling
    echo "building then launching a preinstalled docker container"
    "$launcher" build -f "$dockerfile_prebuilt" --tag mdb-prebuilt-img .

    command="$build_cmd"
    second_cmd="$launcher"
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
    "$launcher" exec -it mdb-plugin-prebuilt bash
    exit
elif [ "$1" = "quickrebuild" ]; then
    echo "build while a container is already open"

    "$launcher" build -f "$dockerfile_prebuilt" --tag mdb-prebuilt-img .

    command="$make_exports && $build_cmd"
    second_cmd="$launcher"
    second_args=("exec" "mdb-plugin-prebuilt" "/bin/bash" "-c" "$copy_plugin_cmd")
else
    echo "invalid command $1"
    echo "$help"
    exit 1
fi

echo "command: $command"
echo "run args:" "${docker_args[@]}"
    
"$launcher" build --file "$dockerfile" --tag mdb-rust .

"$launcher" run \
    --workdir /obj \
    "${docker_args[@]}" \
    --name "$docker_name" \
    mdb-rust \
    /bin/bash -c "$command"


[ -z "$second_cmd" ] || "$second_cmd" "${second_args[@]}"
