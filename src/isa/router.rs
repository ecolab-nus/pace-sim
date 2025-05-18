#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RouterInDir {
    EastIn,
    SouthIn,
    WestIn,
    NorthIn,
    ALURes,
    ALUReg,
    Open,
}

#[derive(Debug, Clone, Copy)]
pub struct RouterSwitchConfig {
    pub predicate: RouterInDir,
    pub alu_lhs: RouterInDir,
    pub alu_rhs: RouterInDir,
    pub east_out: RouterInDir,
    pub south_out: RouterInDir,
    pub west_out: RouterInDir,
    pub north_out: RouterInDir,
}

impl Default for RouterSwitchConfig {
    fn default() -> Self {
        RouterSwitchConfig {
            predicate: RouterInDir::Open,
            alu_lhs: RouterInDir::Open,
            alu_rhs: RouterInDir::Open,
            east_out: RouterInDir::Open,
            south_out: RouterInDir::Open,
            west_out: RouterInDir::Open,
            north_out: RouterInDir::Open,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouterExtraConfig {
    pub bypass_register: [bool; 16],
    pub register_write_enable: [bool; 16],
}

#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub switch_config: RouterSwitchConfig,
    pub extra_config: RouterExtraConfig,
}
