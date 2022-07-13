FROM ubuntu:20.04
LABEL maintainer="hemashushu" \
      version="1.0" \
      description="Development environment for developing Risc-V programs with Rust"

# install deps
RUN set -x; \
    apt-get update \
    && apt-get install -y curl wget git \
    && DEBIAN_FRONTEND=noninteractive TZ=Asia/Shanghai apt-get -y install tzdata \
    && apt-get install -y autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
              gawk build-essential bison flex texinfo gperf libtool patchutils bc \
              zlib1g-dev libexpat-dev pkg-config libglib2.0-dev libpixman-1-dev tmux python3 python3-pip ninja-build

# install rust
RUN set -x; \
    RUSTUP='/root/rustup.sh' \
    && cd $HOME \
    && curl https://sh.rustup.rs -sSf > $RUSTUP \
    && chmod +x $RUSTUP \
    && $RUSTUP -y --default-toolchain nightly --profile minimal \
    && rm $RUSTUP

# install riscv target and deps
RUN set -x; \
    export PATH="$HOME/.cargo/bin:$PATH" \
    && rustup target add riscv64gc-unknown-none-elf \
    && rustup component add llvm-tools-preview \
    && rustup component add rust-src \
    && rustup component add rustfmt \
    && cargo install cargo-binutils

# install qemu
RUN set -x; \
    cd $HOME \
    && wget https://download.qemu.org/qemu-7.0.0.tar.xz \
    && tar xvJf qemu-7.0.0.tar.xz \
    && cd qemu-7.0.0 \
    && ./configure --target-list=riscv64-softmmu,riscv64-linux-user \
    && make -j$(nproc) install \
    && cd $HOME \
    && rm -rf qemu-7.0.0 qemu-7.0.0.tar.xz
