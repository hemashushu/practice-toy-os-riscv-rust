# Practice Toy OS - rCore

本项目是教程 [《rCore-Tutorial-Book 第三版》](https://rcore-os.github.io/rCore-Tutorial-Book-v3/index.html) 的阅读笔记和保姆式详细攻略 😄，原教程旨在一步一步展示如何 **从零开始** 用 Rust 语言写一个基于 RISC-V 架构的 _类 Unix 内核_。

根据原教程的讲解，我将每一章的代码都整理成一个独立的文件夹。你可以一边阅读原教程，一边用你喜欢的代码编辑器切入相应的章节文件夹，试试运行看看运行的结果。

实际上官方也有每个章节的代码 [rCore-Tutorial-v3](https://github.com/rcore-os/rCore-Tutorial-v3)，不过该代码仓库将每个章节的代码组织为 Git 的分支，有时需要同时打开多个章节的代码对比查阅时会稍显不便。另外我也在原来的代码基础上 **添加了些许额外的注释，以及一些扩展资料的链接**。

## 开发环境的搭建和配置

如果要编译和运行教程的所有程序，开发环境必须有以下工具：

1. Rust
2. Rust 的 `riscv64gc-unknown-none-elf` 编译目标
3. Qemu 7.0
4. Risc-v toolchains

如果不想在当前系统上安装以上工具，也可以在 Docker 里搭建该开发环境。在 Docker 里编译和运行所有程序，教程学习完毕之后把该 Docker Image 删掉即可，对于当前系统来说就像什么事都没发生过一样。

### 在当前系统配置开发环境

操作系统建议使用 Arch Linux，该发行版的软件包数量巨多而且版本都是最新的，上面提到的工具直接用系统包管理工具安装即可，省去很多麻烦。

如果要在其他发行版或者系统安装，则 [根据教程的指引](https://rcore-os.github.io/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html) 下载和安装各个工具即可。

需注意 Risc-v toolchains 的最新版本的仓库地址是 <https://github.com/riscv-collab/riscv-gnu-toolchain>，如果不需要调试程序，不安装这个工具链也可以。

### 在 Docker 里配置开发环境

准确来说是构建一个 Docker Image，然后 `run` 这个 Image 并在里面完成教程所述的所有程序的开发和运行。如果你不想更改当前的系统，或者安装一些平时用不着的程序，推荐采用 Docker 搭建开发环境这种方式（前提是你得接受在系统里安装 Docker 或者 Podman 😁）。

我在本项目的 `docker-image` 目录里面放置了两个子目录：`mini` 和 `full`，进入其中的一个目录执行命令（或者执行目录当中的脚本 `build-image`）：

`$ docker build -t rust-riscv .`

然后 Docker 会开始构建，构建完成后执行命令：

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

## 编译和运行各章的代码

> 以下内容请按顺序阅读和运行，即必须先完成第一章的每一个步骤，才能进入第二章，如此类推。

在开始编译和运行各章的代码之前，首先切换到本项目的首层目录，然后你会看到诸如 `ch1`，`ch2`，`ch3` …… 等子目录，它们对应着各章的程序。

如果你是 Docker 的开发环境，则运行命令：

```bash
docker run -it --rm \
    --name rust-riscv \
    --mount type=bind,source=$PWD,target=/mnt \
    rust-riscv

```

该命令会创建一个容器，进入之后是一个 Bash shell，切换到 `/mnt` 目录即可看到 `ch1`，`ch2` …… 等子目录，这时候跟在当前系统里直接搭建的开发环境是一致的。

### Chapter 1

1. 进入 `ch1` 目录
2. 运行脚本 `build-bin` 开始编译
3. 运行脚本 `run` 运行程序，看到 `panic at (src/main.rs:86) Shutdown machine!` 字样则表示成功。

> 这些脚本只是为了简化命令，大部分脚本的内容都是非常简单的。如果你想知道脚本里面具体执行了什么，可以用文本编辑器打开查看。

教程第一章里有一个使用 GDB 进入调试环境的环节，这个步骤可以跳过。如果你还是想完整体验完所有环节，则运行脚本 `start-debug-server` 开始调试的服务端，接下来则根据开发环境的不同而不同：

* 对于在当前系统直接进行开发的，打开另一个终端窗口，然后在里面运行 `ch1` 目录当中的脚本 `start-debug-client-archlinux`。
* 对于在 Docker 里面进行开发的，打开另一个终端窗口，然后在里面运行本项目首层目录当中的脚本 `join-docker`，进入到刚才的容器，切换到 `/mnt` 目录，然后进入 `ch1` 目录，再运行脚本 `start-debug-client-docker`。

### Chapter 2

1. 进入 `ch2` 目录
2. 进入 `user` 目录
3. 运行脚本 `build-app` 开始编译 5 个用户应用程序
4. 运行脚本 `run` 会通过用户态模拟器 `qemu-riscv64` 来运行刚才编译出的应用程序。注意几个程序运行之后会显示错误信息，这是正常的，能看到 `Hello, world!` 和 `Test power OK!` 字样则表示成功。
5. 返回上一级目录，进入 `os` 目录
6. 运行脚本 `build-bin` 开始编译
7. 运行脚本 `run` 运行程序，看到 `All applications completed!` 字样则表示成功。

### Chapter 3

TODO