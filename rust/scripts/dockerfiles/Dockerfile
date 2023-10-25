# dockerfile meant for running subcommands

FROM rust:1.72

WORKDIR /checkout
RUN apt-get update \
    && apt-get install -y \
        binutils \
        bison \
        build-essential \
        cmake \
        cmake-curses-gui \
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
    && rm -rf /var/lib/apt/lists/*
    # ^ clang has to go last because it doesn't play well with other installs

ENV RUST_BACKTRACE=1
ENV RUST_LOG=debug
ENV CARGO_TARGET_DIR=/obj/build-cargo
ENV BUILD_DIR=/obj/build-mariadb
ENV CMD_CCMAKE="ccmake -S/checkout -B${BUILD_DIR}"
ENV SCCACHE_DIR=/obj/sccache
ENV SCCACHE_CACHE_SIZE="1G"
ENV BUILD_LD=lld