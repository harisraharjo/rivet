    # This is a comment
    // This is also a comment
.section .data 10

.string "Etoo"

main:
    lw x5, 0x1000(x0)
    my_symbol x11, x22, 11 //this is a wrong instruction pattern
    sw x7, 0x2000symbol_wrong(x9)
    addi x6, 0x2000(x4)
    lui x6, 0b1110(x4)