# dockerfile meant for running subcommands

FROM rust:1.75

WORKDIR /checkout
RUN apt-get update \
    && apt-get install -y \
        binutils \
        bison \
        build-essential \
        cmake \
        cmake-curses-gui \
        curl \
        gdb \
        gdbserver \
        libboost-all-dev \
        less \
        liblz4-dev \
        liblzo2-dev \
        libsnappy-dev \
        libzstd-dev \
        lld \
        lldb \
        mariadb-client \
        ninja-build \
        sccache \
        vim-tiny \
        wget \
        xxd \
        clang \
    && rm -rf /var/lib/apt/lists/* \
    && curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to /usr/bin/
    # ^ clang has to go last because it doesn't play well with other installs

ENV RUST_BACKTRACE=1
ENV RUST_LOG=debug
ENV CARGO_TARGET_DIR=/obj/build-cargo
ENV MARIA_SOURCE_DIR=/checkout
ENV MARIA_BUILD_DIR=/obj/build-mariadb
ENV MARIA_DATA_DIR=/mdb-install/data
ENV MARIA_PLUGIN_DIR=/mdb-install/plugins
ENV BUILD_DIR=/obj/build-mariadb
ENV CMD_CCMAKE="ccmake -S/checkout -B${BUILD_DIR}"
ENV SCCACHE_DIR=/obj/sccache
ENV SCCACHE_CACHE_SIZE="1G"
ENV BUILD_LD=lld
