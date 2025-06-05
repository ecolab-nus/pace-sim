# ISA
Configuration (or the instruction) can be divided into the opration and the routing configuration :**Configuration** = **Operation** + **RoutingConfig**

## Operation

### ALU_OP(!)(imm)
- ALU_OP the operation code, 
- ! for marking the update of reg_alu_res, corresponding to UpdateRes in the simulator.
- imm is Option\<imm\>, if imm is present, is is used as the second operand, corresponding to Immediate in the simulator. If imm is present, a bit is

```
op1 = reg_op1;
op2 = if (imm.is_some()) config.imm, else reg_op2
wire_alu_out = op(op1, op2)
reg_alu_res = if (!) reg
```
        
#### List of ALU_OP
including ADD, SUB, MULT, LS, RS, ASR, AND, OR, XOR, SEL, CMERGE, CMP, CLT, CGT
semantics:
    wire_alu_out: u64 = op1: u64 + op2: u64
comments:
    Overflow not managed by hardware

### SUB(!)(imm)
semantics:
    wire_alu_out: u64 = op1: u64 + op2: u64
comments:
    Overflow not managed by hardware


