# Assembler (WIP :construction:)

### TODO
- Block comment
- Octal
- Assembling. ch4.6
# RISCV Assembly language
## Directive
The `.byte`, `.half`, `.word`, and `.dword` directives add one or more values to the active section. Their
arguments may be expressed as immediate values
```
x: .byte 10, 12, ’A’, 5+5 # adds four 8-bit values to the active section (10, 12, 97, and 10) 
y: .word x # adds a 32-bit value associated with symbol x to the active section
z: .word y+4 # adds a 32-bit value to the active section, however, in this case, the value is computed by adding four to the value associated
            # with symbol y, which is the address assigned to label y
i: .word 0
j: .word 1
```

The compiler usually inserts a .align 2 directive5 before routine labels to ensure
the routine instructions start on addresses that are multiple of four. The following
code shows an assembly code that uses the .align 2 directive to align the location
counter before each routine.
```
.text
.align 2
func1:
    addi a0, a0, 2
    ret
.align 2
func2:
    addi a0, a0, 42
    ret
```

## Label
### Numeric
```
1:
    beqz a1, 1f # If a1 = 0 then done
    mul a0, a0, a2 # Else, multiply
    addi a1, a1, -1 # Decrements the counter
    j 1b # Repeat
1:
    ret
```
## Location Counter
An internal assembler counter that keeps track of
addresses when a program is being assembled. More specifically, it keeps the
address of the next available memory position. Each section (memory section) has its own location
counter, and the active location counter is the location counter of the active section


## Memory
Program instructions are expected to be placed on the .text section, while constants,
i.e., read-only data, must be placed on the `.rodata` section. Also, initialized
global variables must be placed on the `.data` section, and uninitialized global variables
should be placed on the .bss section.

### Section
#### BSS
The `.bss` section is dedicated for storing uninitialized global variables. These variables
need to be allocated on memory, but they do not need to be initialized by the loader
when a program is executed. As a consequence, their initial value do not need to be
stored on executable nor object files. Since no information is stored on the `.bss` section in object and executable files,
the assembler does not allow assembly programs to add data to the `.bss` section

## ISA
### Immediates
logic and shift instructions can only encode immediate values that
can be represented as a 12-bit twos’-complement signed number. In other words, the
immediate values used as operands on these instructions must be greater
or equal to -2048 (−211) and less or equal to 2047 (211 − 1)

Sequences started with the `“0x”` and the `“0b”` prefixes are interpreted as
hexadecimal and binary numbers, respectively. Octal numbers are represented by
a sequence of numeric digits starting with digit `“0”`. Sequences of numeric digits
starting with digits `“1”` to `“9”` are interpreted as decimal numbers.
Alphanumeric characters represented between single quotation marks are converted
to numeric values using the ASCII table. For example, the `’a’` operand is
converted into value ninety seven. `'a'`