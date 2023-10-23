#!/usr/bin/env python3
# Create a docker image with volumes set up. This script is an entrypoint
# and lets you choose an operation

import argparse
import os
import subprocess as sp
from os.path import dirname
from pathlib import Path

SCRIPT_DIR = "/checkout/rust/scripts"
BUILD_CMD = f"{SCRIPT_DIR}/build.sh"
COPY_PLUGIN_CMD = f"{SCRIPT_DIR}/copy_plugins.sh"
INSTALL_CMD = f"{SCRIPT_DIR}/install.sh"
LAUNCH_QUICK_CMD = f"{SCRIPT_DIR}/launch_quick.sh"
LAUNCH_DEBUG_CMD = f"{SCRIPT_DIR}/launch_debug.sh"
TEST_CMD = f"{SCRIPT_DIR}/run_mtr.sh"
START_SAFE_CMD = f"{SCRIPT_DIR}/start_safe.sh"
START_CMD = f"{SCRIPT_DIR}/start.sh"

MAKE_EXPORTS = f"export BUILD_CMD={BUILD_CMD} && \
    export INSTALL_CMD={INSTALL_CMD} && \
    export TEST_CMD={TEST_CMD} && \
    export START_CMD={START_CMD} && \
    export START_SAFE_CMD={START_SAFE_CMD}"

BUILT_TAG = "mdb-rust"
DOCKER_NAME = "mdb-plugin-test"
DOCKER_NAME_REBUILD = "mdb-plugin-rebuild"

# Global state
LAUNCHER = None
MARIA_ROOT = None
DOCKERFILE_BUILDER = None
DOCKERFILE_PREBUILT = None
OBJ_DIR = None


def parse_args():
    """Set up CLI arguments."""

    def add_nobuild(parser):
        parser.add_argument(
            "--nobuild",
            action="store_true",
            help="assume MDB has been built recently, skip the building step",
        )

    def add_prebuilt(parser):
        parser.add_argument(
            "-p",
            "--prebuilt",
            action="store_true",
            help="use a prebuilt image rather than building from source",
        )

    parser = argparse.ArgumentParser(
        prog="rust-mdb ",
        description="Launcher for rust+mariadb debugging",
    )

    parser.add_argument(
        "--launcher",
        help="specify a container launcher such as podman",
        default="docker",
    )

    subparsers = parser.add_subparsers(
        title="action", description="set the action to perform", required=True
    )
    p_build = subparsers.add_parser(
        "build", aliases=["b"], help="build from source and exit"
    )
    p_start = subparsers.add_parser("start", aliases=["s"], help="start mariadb")
    p_shell = subparsers.add_parser(
        "shell", aliases=["sh"], help="build from source and enter a shell"
    )
    p_test = subparsers.add_parser(
        "test", aliases=["mtr", "t"], help="build from source then test with MTR"
    )
    p_rebuild = subparsers.add_parser(
        "rebuild", help="rebuild files while a server is running"
    )
    p_stop = subparsers.add_parser("stop", help="stop a running server")

    # argparse needs a way to differentiate arguments
    p_build.set_defaults(action="build")
    p_start.set_defaults(action="start")
    p_shell.set_defaults(action="shell")
    p_test.set_defaults(action="test")
    p_rebuild.set_defaults(action="rebuild")
    p_stop.set_defaults(action="stop")

    add_nobuild(p_start)
    add_nobuild(p_shell)

    add_prebuilt(p_shell)
    add_prebuilt(p_test)
    add_prebuilt(p_rebuild)

    p_shell.add_argument(
        "--started",
        help="launch a shell against a started container",
        action="store_true",
        default=False,
    )
    p_shell.add_argument(
        "--sql",
        help="launch a SQL shell rather than bash",
        action="store_true",
        default=False,
    )

    p_start.add_argument(
        "--debug",
        help="launch with GDB server on port 2345",
        action="store_true",
        default=False,
    )

    args = parser.parse_args()
    return args


def run_global_config(args):
    """Configure our global state"""
    global DOCKERFILE_PREBUILT
    global DOCKERFILE_BUILDER
    global MARIA_ROOT
    global OBJ_DIR
    global LAUNCHER

    MARIA_ROOT = Path(dirname(dirname(os.path.realpath(__file__))))
    rust_dir = MARIA_ROOT.joinpath("rust")
    script_dir = rust_dir.joinpath("scripts")
    DOCKERFILE_BUILDER = script_dir.joinpath("dockerfiles", "Dockerfile")
    DOCKERFILE_PREBUILT = script_dir.joinpath("dockerfiles", "Dockerfile.prebuilt")
    OBJ_DIR = MARIA_ROOT.joinpath("docker_obj")
    LAUNCHER = args.launcher
    print(f"using obj_dir {OBJ_DIR}")


def docker_create_img(dockerfile):
    """Create the docker image (does not build MariaDB)"""
    sp.run(
        [LAUNCHER, "build", "--file", dockerfile, "--tag", BUILT_TAG, MARIA_ROOT],
        check=True,
    )


def docker_run_inner(
    command: str, name: str, extra_docker_args: [str] = None, check=True
):
    """Launch a docker"""
    extra_docker_args = extra_docker_args or []
    args = (
        [
            LAUNCHER,
            "run",
            "--workdir=/obj",
            "--volume",
            f"{MARIA_ROOT}:/checkout:ro",
            "--volume",
            f"{OBJ_DIR}:/obj",
            "-p2345:2345", # GDB port
            "--rm",
        ]
        + extra_docker_args
        + [
            "--name",
            name,
            BUILT_TAG,
            "/bin/bash",
            "-c",
            command,
        ]
    )

    print(args)

    sp.run(args, check=check)


def docker_run(
    extra_commands: [str] = None,
    extra_docker_args: [str] = None,
    should_build: bool = True,
    name: str = "mdb-plugin-test",
    check=True,
):
    """Build the docker builder image then run MariaDB with an optional build"""
    cmds = [MAKE_EXPORTS]
    extra_commands = extra_commands or []
    extra_docker_args = extra_docker_args or []

    if should_build:
        cmds.append(BUILD_CMD)
    else:
        print("skipping build")

    docker_create_img(DOCKERFILE_BUILDER)
    docker_run_inner(
        "&&".join(cmds + extra_commands), name=name, extra_docker_args=extra_docker_args
    )


def main():
    args = parse_args()
    run_global_config(args)
    should_build = True

    if "nobuild" in args and args.nobuild is True:
        should_build = False

    if args.action == "build":
        docker_run(should_build=should_build)

    elif args.action == "rebuild":
        print("building with a container already open")
        docker_run(name=DOCKER_NAME_REBUILD)

    elif args.action == "shell":
        shell = "/bin/bash"
        if args.sql:
            shell = "mariadb"
        if args.started:
            print(f"launching a shell in a {DOCKER_NAME}")
            args = [LAUNCHER, "exec", "-it", DOCKER_NAME, shell]
            return sp.run(args, check=True)

        print("launching a shell")
        docker_run([shell], ["-it"], should_build=should_build)

    elif args.action == "test":
        print("building then testing mariadb")
        docker_run([TEST_CMD], should_build=should_build)

    elif args.action == "start":
        print("building then starting mariadb")
        if args.debug:
            launch = LAUNCH_DEBUG_CMD
        else:
            launch = START_SAFE_CMD

        docker_run(
            [INSTALL_CMD, launch],
            ["-it"],
            should_build=should_build,
            check=False,
        )

    elif args.action == "stop":
        sp.run([LAUNCHER, "stop", DOCKER_NAME], check=True)


if __name__ == "__main__":
    main()
