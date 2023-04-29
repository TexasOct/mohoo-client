FROM docker.io/library/fedora:37

# Prepare Fedora environment pkg
RUN dnf install -y clang cmake wget gcc g++ git gawk gettext ncurses-devel zlib-devel \
    openssl-devel libxslt wget which @c-development @development-tools \
    @development-libs zlib-static which python3 make libstdc++-devel.i686 glibc-devel.i686

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=nightly \
    MOHOO_TOOLCHAIN=nightly-2023-02-27

RUN set -eux; \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o rustup-init; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile default --default-toolchain $RUST_VERSION; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME;

RUN rustup toolchain add $MOHOO_TOOLCHAIN && \
    rustup target add mipsel-unknown-linux-musl --toolchain $MOHOO_TOOLCHAIN && \
    rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu && \
    rustup component add rust-src --toolchain $MOHOO_TOOLCHAIN

WORKDIR /mnt