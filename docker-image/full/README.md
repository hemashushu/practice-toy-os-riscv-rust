# Docker image `rust-riscv`

这个 Docker Image 基于 ubuntu 20.04，安装了：

1. Rust
2. Rust 的 `riscv64gc-unknown-none-elf` 编译目标
3. Qemu 7.0
4. Risc-v toolchains

在当前目录执行命令（或者执行目录当中的脚本 `build-image`）：

`$ docker build -t rust-riscv .`

Docker 会开始构建，构建完成后执行命令：

`$ docker image list`

检查是否存在一项 `rust-riscv`，若存在则表示构建成功。
