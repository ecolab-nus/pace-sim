# Basic Usage
Build:
```
cargo build
```

Tests (including the example simulations in tests/)
```
cargo test
```

You can simulate with snapshot and memory dump.
Refer to
```
target/debug/simulation --help
```
for the details.

You can convert between format in binary string and mnemonic of PE instructions with
```
target/debug/convert_config <file> <file>
```
The file type recognization relies on the file extension:
.binprog for binary string
.prog for mnemonic (human readable and writeable)

# Simulation framework
A folder should contain 
1. The configuration files in binary string. You can run the convertion tool to get the binary version from mnemonic.
2. The initial content of data memories, one DM per file
3. The AGUs files (optinal for grid simulation, obligatory for PACE complete simulations). In mnemonic only
4. The PE files are named "PE-YyXx", Y=0 X=0 top left corner
5. The DM files are named "DMx" every 2 edge PEs share one DM. Order : top left -> bottom left -> top right -> bottom right.
6. The AGU files are named "AGUx", every edge PE has one AGU. Order : top left -> bottom left -> top right -> bottom right.

The Grid (SingleSided or DoubleSided) loads all files, the become runnable.
See examples in tests/test_add_2x2.rs or tests/test_array_add_*.

The PACESystem is for loading a folder only according to the complete PACE setup. It is convertable to Grid.

# ISA
Configuration (or the instruction) can be divided into the opration and the routing configuration :**Configuration** = **Operation** + **RoutingConfig**. You find the semantics of ISA in **src/isa/ **. You find the syntax of ISA in mnemonic in **src/isa/mnemonic/ **, you find the syntax of ISA in binary in **src/isa/binary/**.

## 1. Operation

### 1.1 ALU_OP(!)(imm)
- ALU_OP the operation code, 
- ! for marking the update of reg_alu_res, corresponding to operation.update_res in the simulator.
- imm is Option\<imm\>, if imm is present, is is used as the second operand, corresponding to operation.immediate in the simulator. 
- All ALU operations are 16 bits operation
- only 16bits of alu_out is used, no overflow management above 16-bit

```
op1 = reg_op1;
op2 = if (imm.is_some()) imm, else reg_op2
wire_alu_out = op(op1, op2):u16 as u64
reg_alu_res = if (!) reg
```

#### List of supported operations
- ADD: 16b addition, no overflow management
- SUB: 16b substraction, no overflow management
- MULT: 16b modular multiplication (wrapping MUL, check Rust doc for wrapping_mul)
- DIV: 16b modular division, (wrapping DIV, check Rust code for wrapping_div)
- LS: Logical shift left (within the 16b result)
- RS: Logical shift right (within the 16b result)
- ARS: Arithmetic shift right (keeping the sign, within the 16b result)
- AND: l6b bit-wise and
- OR: 16b bit-wise or
- XOR: 16b bti-wise xor
- SEL: 
``` 
if (operation.update_res)
wire_alu_out = immediate
else
if (the MSB of the op1 == '1')
wire_alu_out = op1
else if (the MSB of the op2 == '1')
wire_alu_out = op2
else 
wire_alu_out = 0
```

- CMERGE 
```
if (operation.imm.is_some()) wire_alu_out = imm
else wire_alu_out = op1
```

- CMP 
```
if (op1 == op2)
wire_alu_out = 0x01
else
wire_alu_out = 0x00
```

- CLT
Compare Lower Than
```
if (op1 as i16 < op2 as i16)
wire_alu_out = 0x01
else
wire_alu_out = 0x00
```
- CGT
Compare Greater Than
```
if (op1 as i16 < op2 as i16)
wire_alu_out = 0x01
else
wire_alu_out = 0x00
```

### 1.2 MEM_OP (imm)
STOREB(8b) STORE(16b) STORED(64b)
LOADB(8b) LOAD(16b) LOADD(64b)
```
op1 = reg_op1;
op2 = if (imm.is_some()) imm, else reg_op2
```

STORE:
```
if config.is_load()
wire_dmem_data = op1
wire_dmem_addr = op2
```

LOAD:
```
if config.is_store()
wire_dmem_addr = op2
```

## 1.3. Jump dst [loop_start, loop_end]
```
reset reg_predicate
using inst[49:45] as the destination  (jump_dst)
assign loop_start
assign loop_end
```
jump_dst is optional, if not used, just move to the next instruction. Jump also sets the loop_start and loop_end.
Currently only supports the case where loop_start = jump_dst

# 2. Loop Start/ Loop End
the instruction "Jump" or some called "SoftReset" set the loop_start and loop_end register.
Once PC reaches loop_end, it branches back to loop_start.

# AGU
Control Memory (CM) holds instructions
Address Register File (ARF) holds the address to be used for the Data Memory
The number of instructions is the same of the number of address register
The PC of the AGU points to the current instruction and the address

There are only 2 instructions: Const and Strided(S).
For Const, the value (address) of the corresponding address register never changes
For Strided, the value of the corresponding address register is incremented by S every time AFTER using the address for the DM.

Each PE memory instruction triggers the use of current address and then the incrementation of PC.

### AGU stop condition
AGU has a MAX_COUNT value that is set before execution.
Each time AGU complete **ALL instructions**, the internal counter is incremented by 1.
When the internal counter is equal to MAX_COUNT, AGU signals an "END OF EXECUTION".
**Careful** AGU counter starts by 0, so MAX_COUNT is actually total number of iterations. 
**Careful** AGU counter is incremented just after PE memory operation, however, the comparison to MAX_COUNT is done the next memory operation. That allows the entire system to finish the last iteration


# Global Address
19-bit address - [18:0], 64 bits per address

Target selection : [18:17] 
00 PE
01 DM
10 LUT
11 Cluster exec enabled

### PE ([18:17] = "00")
[15:10]  which PE out of 8x8 array, in y*Y + x, top left is y=0, x=0, bottom right y=7, x=7
[9:8] Within PE level decoding:
00 PE CM
01 AGU CM
10 AGU ARF
11 MAX_iter (32b unsigned)

[7:4] location within CM/AGU_CM/ARF - 16 locations


### DM ([18:17] = "01"): 
[16] LEFT = 0, RIGHT = 1
[15:14] which of the 4 DMs at each side (from top to bottom)
[9:0] DM content, one address per 64 bits word
