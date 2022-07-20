.section .init
.global _start
_start:
    li s1, 0x10000000   # set s1 = 0x1000_0000
    la s2, message      # set s2 = address_of(message)
    addi s3, s2, 14     # set s3 = s2 + 14
                        # the message length is 14
next:
    lb s4, 0(s2)        # load byte from memory[s2+0] to s4
    sb s4, 0(s1)        # set memory[s1 +0] = s4
    addi s2, s2, 1      # set s2 = s2 + 1
    blt s2, s3, next    # if s2 < s3 then branch 'next'

.section .data
message:
  .string "Hello, world!\n"
