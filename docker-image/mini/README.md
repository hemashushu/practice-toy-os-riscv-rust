# Docker image `rust-riscv`

这个 Docker Image 基于 ubuntu 20.04，安装了：

1. Rust
2. Rust 的 `riscv64gc-unknown-none-elf` 编译目标
3. QEMU 7.0

在当前目录执行命令（或者执行目录当中的脚本 `build-image`）：

`$ docker build -t rust-riscv .`

Docker 会开始构建，构建完成后执行命令：

`$ docker image list`

检查是否存在一项 `rust-riscv`，若存在则表示构建成功。

因为 Risc-v toolchains 的体积较大，所以 `mini` 版默认不安装这个工具链，如果需要安装可以在 Image 构建完成之后，进入该 Container 然后使用下面的命令手动安装：

```bash
$ cd /opt
$ wget https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/2022.06.10/riscv64-elf-ubuntu-20.04-nightly-2022.06.10-nightly.tar.gz
$ tar xzf riscv64-elf-ubuntu-20.04-nightly-2022.06.10-nightly.tar.gz
$ rm riscv64-elf-ubuntu-20.04-nightly-2022.06.10-nightly.tar.gz
$ echo 'export PATH=/opt/riscv/bin:$PATH' >> ~/.bashrc
$ . ~/.bashrc
```