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

# AGU
