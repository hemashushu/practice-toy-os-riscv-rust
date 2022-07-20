.globl _start
_start:
    li s1, 0x10000000 # set s1 = 0x1000_0000
    li s2, 0x41       # set s2 = 0x48
    sb s2, 0(s1)      # set memory[s1 + 0] = s2
