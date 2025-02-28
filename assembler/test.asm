    # This is a comment
    // This is also a comment
.section .data 10

.string "Etoo"

main:
    # 0x1000MP # invalid literal bin
    # 99beto // invalid literal decimal
    # 0b11kl // invalid literal Binary
    # lw x5, 0x1000(x0)
    
    # addi x5, x6, 10
    # # my_symbol x11, x22, 11 //this is a wrong instruction pattern
    # # sw x7, 0x2000(x9) // invalid literal Hex
    # # eds0110xFF //valid symbol
    # # addi x6, 0x2000(x4)
    # # lui x6, 0b111(x4)
    # lw x1, 10(x5)
    # sw x1, 111(x5)
    # lui x1, 0x1212
    add x1, x2, x3