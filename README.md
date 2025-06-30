# PACE-Sim

# Basic Usage
Build:
```
cargo build
```

You can simulate with snapshot and memory dump.
Refer to
```
target/debug/simulation --help
```
for the details.

You can convert between format with
```
target/debug/convert <file> <file>
```
The file type recognization relies on the file extension:
.binprog for binary string
.prog for mnemonic (human readable and writeable)

### Next TODOs:
- AGU
- Floating Point SIMD

### Further TODOs:
TODOs:
- Complete ISA

# Simulation Framework
## PE-Memory connectivity
PE-YyX0 (the left edge PEs) are connected to the datamemory y%2. 
PE-YyXX (the right edge PEs) are connected to the datamemory y%2+Y.
This means every to PEs are connected to the same data memory.

# ISA
Configuration (or the instruction) can be divided into the opration and the routing configuration :**Configuration** = **Operation** + **RoutingConfig**

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
- MULT: 16b multiplication, no overflow management
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

## 1.3. Jump
```
reset reg_predicate
assign loop_start
assign loop_end
```
the Jump instruction set the loop_start and loop_end.

# 2. Loop Start/ Loop End

TODO: i don't understand:
```
                if (control_reg_data[34:30]==5'b11110 && jumpl_reg != 5'b11110) begin
                    addr_shifted_tile <= {1'b0,control_reg_data[49:45]};
```

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
19-bit address - [18:0]

Target selection : [18:17] 
00 PE
01 DM
10 LUT
11 Cluster exec enabled

### PE ([18:17] = "00")
[15:10]  which PE out of 8x8 array, in y*Y + x, top left is y=0, x=0, bottom right y=7, x=7
[9:8] WIthin PE level decoding:
00 PE CM
01 AGU CM
10 AGU ARF
11 MAX_iter (32b unsigned)

PE CM:
[6:0] 16 * (8 bytes per configuration) = 128 bytes PE CM, so [6:3] the index of the configuration

AGU CM:
[3:0] 16 * (1 byte per CM) = 16 bytes AGU CM 

AGU ARF:
[4:0] 16 * (13-bit aligned to 2 bytes) = 32 bytes AGU ARF. 13b into 2 bytes still little-endian


### DM ([18:17] = "01"): 

[9:0] 1024 bytes content (byte addressable) for each DM
[12:10] 8 DMs's index (top left 0, bottom left 3, top right 4, top down 7)



## List:
| Address range (hex)                          | Content / purpose (write **“unused – …”** when not decoded)                            |
| -------------------------------------------- | -------------------------------------------------------------------------------------- |
| 0×00000 – 0×0007F + 0×400 × k (k = 0…63) | **PE CM** – 16 × 8-byte configs (one stripe per PE)                                    |
| 0×00080 – 0×000FF + 0×400 × k                | **unused – upper half of PE CM bit (bit 7 unused)**                        |
| 0×00100 – 0×0010F + 0×400 × k                | **AGU CM** – 16 × 1-byte configs                                                       |
| 0×00110 – 0×001FF + 0×400 × k                | **unused – padding after AGU CM**                                                      |
| 0×00200 – 0×0021F + 0×400 × k                | **AGU ARF** – 16 registers, 13 bits packed into 2 bytes                                |
| 0×00220 – 0×002FF + 0×400 × k                | **unused – padding after AGU ARF**                                                     |
| 0×00300 – 0×00303 + 0×400 × k                | **max\_iter** – single 32-bit counter                                                  |
| 0×00304 – 0×003FF + 0×400 × k                | **unused – padding after max\_iter**                                                   |
| 0×10000 – 0×1 FFFF                       | **unused – bit 16 unused**                              |
| 0×20000 – 0×21FFF, step 0×400                        | **DM j** – 1 KiB byte-addressable memory                                               |
| 0×22000 – 0×3 FFFF                       | **unused – DM region with bits \[16:13] unused** |
| 0×40000 – 0×5 FFFF                       | **Reserved – LUT region (top-level selector 10)**                               |
| 0×60000 – 0×7 FFFF                       | **Reserved – Cluster-exec reserved (top-level selector 11)**                      |
