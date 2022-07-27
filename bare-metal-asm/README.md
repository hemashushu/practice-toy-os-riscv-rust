# RISC-V 裸机汇编程序

## 第一个程序

### 汇编

新建一个文件，名称为 `first.s`，内容如下：

```asm
.globl _start
_start:
    li s1, 0x10000000 # set s1 = 0x1000_0000
    li s2, 0x41       # set s2 = 0x48
    sb s2, 0(s1)      # set memory[s1 + 0] = s2
```

简单讲解：`.globl _start` 定义个全局 `符号`，类比 "一个库的导出函数（的名称），可供外部查看和调用"，`_start` 定义一个位置，类比 `自动行号`。最后 3 行是 RISC-V 指令，作用看句末的注释。

关于 RISC-V ISA 的基本知识，可以参考 [《RISC-V 手册》](http://riscvbook.com/chinese/RISC-V-Reader-Chinese-v2p1.pdf)，有关指令更详细的资料可以参考 [《RISC-V 规范》](https://riscv.org/technical/specifications/)。

下面命令将汇编源码汇编（动词）为目标文件：

```bash
$ riscv64-unknown-elf-as -g -o first.o first.s
```

`g` 参数用于生成调试信息。

### 链接

新建一个文件，名称为 `default.lds`，内容如下：

```ld
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80000000;

SECTIONS
{
  . = BASE_ADDRESS;

  .text : {
    *(.text.entry)
    *(.text .text.*)
  }

  .rodata : {
    *(.rodata .rodata.*)
  }

  .data : {
    . = ALIGN(4096);
    *(.sdata .sdata.*)
    *(.data .data.*)
  }

  .bss :{
    *(.sbss .sbss.*)
    *(.bss .bss.*)
  }

}

```

这是一个链接脚本，可见一个常见可执行文件的 `.text`, `.rodata`, `.data`, `.bss` 等段的定义，其中 `BASE_ADDRESS` 用于指定程序的开始位置，之所以值为 0x8000_0000 是因为模拟器程序 `qemu-system-riscv64 -machine virt` 启动后，PC 寄存器的值为 0x1000，也就是说位置 0x1000 的指令将会第一个被执行，通过调试可以发现该处的指令如下：

```asm
0x1000:      auipc   t0,0x0         # set t0 = $pc + sign_extend(immediate[31:12] << 12)
                                    # 现在 t0 == 0x1000，即当前指令的位置
0x1004:      addi    a2,t0,40       # set a2 = t0 + 0x28
                                    # 现在 a2 == 0x1028
                                    # 暂时不用理会
0x1008:      csrr    a0,mhartid     # Hart ID Register (mhartid), 运行当前代码的硬件线程（hart）的 ID
                                    # 现在 a0 == 0
                                    # 暂时不用理会
0x100c:      ld      a1,32(t0)      # set a1 = int64(t0 + 0x20)
                                    # 现在 a1 == 0x87000000
                                    # 可以使用命令 `x/2wx 0x1020` 查看
                                    # 暂时不用理会
0x1010:      ld      t0,24(t0)      # set t0 = int64(t0 + 0x18)
                                    # 现在 t0 == 0x80000000
                                    # 可以使用命令 `x/2wx 0x1018` 查看
0x1014:      jr      t0             # 跳转到 0x80000000
0x1018:              0x0000
0x101a:      .2byte  0x8000
0x101c:              0x0000
0x101e:              0x0000
```

其中 `jr t0` 表示即将会跳到寄存器 `t0` 的值所指向的位置。

用下面的命令链接（动词）得出目标文件：

```bash
$ riscv64-unknown-elf-ld -T default.lds -o first first.o
```

### 运行

使用 QEMU 运行上一步得到的目标文件：

```bash
$ qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios none \
    -kernel first
```

应该能看到一个字符 `A` 输出。

按 `Ctrl+a` 然后再按 `x` 退出 QEMU （温馨提示，退出 QEMU 不是按 `Ctrl+x`，也不是 `:q`）

### 调试

在运行 QEMU 的命令后面加上 `-s -S` 能启动 GDB 调试服务端，打开另外一个终端窗口，运行下面命令进入 GDB 调试客户端

```bash
$ riscv64-elf-gdb
```

进入后输入 `target remote :1234` 连接服务端。

**调试命令**

- 命令 `x/10i $pc`

查看 $pc 位置的 10 条指令，命令 `x` 用于查看内存

- 命令 `si`

逐条指令运行

- 命令 `b *0x80000000`

设置断点

- 命令 `c`

可以持续运行程序直到遇到断点

- 命令 `p/d $x1`

打印 x1 寄存器的数值

- 命令 `p/x $sp`

同样也是打印寄存器的数值，以 16 进制格式打印

- 命令 `i r`

列出所有寄存器的值。

- 命令 `q`

退出调试环境

输入命令 `help` 获取各个命令的帮助信息，比如：

`help info`

会列出 `info` 命令的详细用法。如果有时忘记命令的完整名称，可以在输入前面的一两个字符时，按下 `tab` 键列出提示或者自动补完，比如输入 `info reg` 按下 `tab` 键，会自动补完为 `info registers`。

对于高频次使用的命令，只需输入命令的第一个字符即可，比如 `info` 可以输入 `i` 代替，同样 `i registers` 可以输入 `i r` 代替。

> 注，在 GDB 里是没法直接输入和执行 RISC-V 指令的，所以如果想要测试一些 RISC-V 指令，需要编写一个简单的汇编程序，然后再使用上述的步骤运行和调试。

## 附录：GDB 常用命令

需注意下面部分命令仅当指定源码时才有效（比如调试由 C 语言编译而得的程序，如果直接由汇编源码汇编出目标文件，需要加上 `--gstabs` 参数，比如：`as --gstabs hello.s -o hello.o`）

gdb基本命令 1

| 命令 | 描述 |
| --- | --- |
| backtrace（或bt） | 查看各级函数调用及参数 |
| finish | 连续运行到当前函数返回为止，然后停下来等待命令 |
| frame（或f） 帧编号 | 选择栈帧 |
| info（或i） locals | 查看当前栈帧局部变量的值 |
| list（或l） | 列出源代码，接着上次的位置往下列，每次列10行 |
| list 行号 | 列出从第几行开始的源代码 |
| list 函数名 | 列出某个函数的源代码 |
| next（或n） | 执行下一行语句 |
| print（或p） | 打印表达式的值，通过表达式可以修改变量的值或者调用函数 |
| quit（或q） | 退出gdb调试环境 |
| set var | 修改变量的值 |
| start | 开始执行程序，停在main函数第一行语句前面等待命令 |
| step（或s） | 执行下一行语句，如果有函数调用则进入到函数中 |

gdb基本命令2

| 命令 | 描述 |
| --- | --- |
| break（或b） 行号 | 在某一行设置断点 |
| break 函数名 | 在某个函数开头设置断点 |
| break ... if ... | 设置条件断点 |
| continue（或c） | 从当前位置开始连续运行程序 |
| delete breakpoints 断点号 | 删除断点 |
| display 变量名 | 跟踪查看某个变量，每次停下来都显示它的值 |
| disable breakpoints 断点号 | 禁用断点 |
| enable 断点号 | 启用断点 |
| info（或i） breakpoints | 查看当前设置了哪些断点 |
| run（或r） | 从头开始连续运行程序 |
| undisplay 跟踪显示号 | 取消跟踪显示 |

gdb基本命令3

| 命令 | 描述 |
| --- | --- |
| watch | 设置观察点 |
| info（或i） watchpoints | 查看当前设置了哪些观察点 |
| x | 从某个位置开始打印存储单元的内容，全部当成字节来看，而不区分哪个字节属于哪个变量 |

上面表格出自 [《Linux C编程一站式学习》](https://akaedu.github.io/book/ch10s01.html)
