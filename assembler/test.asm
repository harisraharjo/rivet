    # This is a comment
    // This is also a comment
.section .data 10

.string "Etoo"

main:
    # 0x1000MP # invalid literal bin
    # 99beto // invalid literal decimal
    # 0b11kl // invalid literal Binary
    lw x5, 0x1000(x0)
    my_symbol x11, x22, 11 //this is a wrong instruction pattern
    sw x7, 0x2000(x9) // invalid literal Hex
    eds0110xFF //valid symbol
    addi x6, 0x2000(x4)
    lui x6, 0b111(x4)