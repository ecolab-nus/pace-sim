type Register = i32;

pub enum Instruction {
    Nop,
    Add(Register, Register, Register),
}