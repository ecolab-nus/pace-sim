use crate::isa::{
    binary::configuration::{ConfigField, ConfigurationField},
    router::{DirectionsOpt, RouterConfig, RouterInDir, RouterSwitchConfig},
};

impl DirectionsOpt {
    /// Converts a 4-bit binary code to a DirectionsOpt
    /// in the order of North, South, West, East
    pub fn from_binary(code: u8) -> Self {
        assert!(code < 16, "Invalid directions code: {}", code);
        let mut directions = Self::default();
        directions.north = (code & 0b1000) != 0;
        directions.south = (code & 0b0100) != 0;
        directions.west = (code & 0b0010) != 0;
        directions.east = (code & 0b0001) != 0;
        directions
    }

    pub fn to_binary(&self) -> u8 {
        let mut code: u8 = 0;
        code |= (self.north as u8) << 3;
        code |= (self.south as u8) << 2;
        code |= (self.west as u8) << 1;
        code |= (self.east as u8) << 0;
        code
    }
}

impl RouterInDir {
    const BINARY_MAPPING: [RouterInDir; 8] = [
        RouterInDir::EastIn,  // 000
        RouterInDir::SouthIn, // 001
        RouterInDir::WestIn,  // 010
        RouterInDir::NorthIn, // 011
        RouterInDir::ALUOut,  // 100
        RouterInDir::ALURes,  // 101
        RouterInDir::Invalid, // 110
        RouterInDir::Open,    // 111
    ];

    pub fn from_binary(code: u8) -> Self {
        assert!(
            code <= 7 && code != 6,
            "Invalid router direction code: {}",
            code
        );
        Self::BINARY_MAPPING[code as usize]
    }

    pub fn to_binary(&self) -> u8 {
        assert!(
            !matches!(self, RouterInDir::Invalid),
            "Invalid router direction"
        );
        *self as u8
    }
}

impl RouterSwitchConfig {
    /// Converts a 21-bit binary code to a RouterSwitchConfig
    pub fn from_u32(code: u32) -> Self {
        // first, check if the code is in 21 bits
        assert!(
            code < (1 << 21),
            "Invalid router switch config code: {}",
            code
        );
        // From LSB to MSB
        // first 3 bits are the predicate
        let east_out = RouterInDir::from_binary((code & 0b111) as u8);
        // next 3 bits are the alu_op1 or RHS
        let south_out = RouterInDir::from_binary(((code >> 3) & 0b111) as u8);
        // next 3 bits are the alu_op2 or LHS
        let west_out = RouterInDir::from_binary(((code >> 6) & 0b111) as u8);
        // next 3 bits are the North Out
        let north_out = RouterInDir::from_binary(((code >> 9) & 0b111) as u8);
        // next 3 bits are the West Out
        let alu_op1 = RouterInDir::from_binary(((code >> 12) & 0b111) as u8);
        // next 3 bits are the South Out
        let alu_op2 = RouterInDir::from_binary(((code >> 15) & 0b111) as u8);
        // next 3 bits are the East Out
        let predicate = RouterInDir::from_binary(((code >> 18) & 0b111) as u8);
        Self {
            predicate,
            alu_op2,
            alu_op1,
            north_out,
            west_out,
            south_out,
            east_out,
        }
    }

    /// Converts a RouterSwitchConfig to a 21-bit binary code
    pub fn to_u32(&self) -> u32 {
        let mut code: u32 = 0;
        // MSB first 3 bits are the predicate
        code |= self.predicate.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the alu_op2
        code |= self.alu_op2.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the alu_op1
        code |= self.alu_op1.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the North Out
        code |= self.north_out.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the West Out
        code |= self.west_out.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the South Out
        code |= self.south_out.to_binary() as u32;
        // move to next 3 bits
        code <<= 3;
        // next 3 bits are the East Out
        code |= self.east_out.to_binary() as u32;
        code
    }
}

impl RouterConfig {
    pub fn from_u64(code: u64) -> Self {
        let switch_config =
            RouterSwitchConfig::from_u32(code.get_field(ConfigField::RouterSwitchConfig) as u32);
        let input_register_used =
            DirectionsOpt::from_binary(code.get_field(ConfigField::RouterBypass) as u8);
        let input_register_write =
            DirectionsOpt::from_binary(code.get_field(ConfigField::RouterWriteEnable) as u8);
        Self {
            switch_config,
            input_register_used,
            input_register_write,
        }
    }

    pub fn to_u64(&self) -> u64 {
        let mut code: u64 = 0;
        code.set_field(
            ConfigField::RouterSwitchConfig,
            self.switch_config.to_u32() as u32,
        );
        code.set_field(
            ConfigField::RouterBypass,
            self.input_register_used.to_binary() as u32,
        );
        code.set_field(
            ConfigField::RouterWriteEnable,
            self.input_register_write.to_binary() as u32,
        );

        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_router_switch_config_binary_conversions() {
        let switch_config = RouterSwitchConfig {
            predicate: RouterInDir::Open,
            alu_op1: RouterInDir::ALUOut,
            alu_op2: RouterInDir::ALURes,
            north_out: RouterInDir::NorthIn,
            west_out: RouterInDir::WestIn,
            south_out: RouterInDir::SouthIn,
            east_out: RouterInDir::EastIn,
        };
        let binary = switch_config.to_u32();
        let switch_config_from_binary = RouterSwitchConfig::from_u32(binary);
        assert_eq!(switch_config, switch_config_from_binary);
    }

    #[test]
    fn test_router_config_binary_conversions() {
        let router_config = RouterConfig {
            switch_config: RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::ALUOut,
                alu_op2: RouterInDir::ALURes,
                north_out: RouterInDir::NorthIn,
                west_out: RouterInDir::WestIn,
                south_out: RouterInDir::SouthIn,
                east_out: RouterInDir::EastIn,
            },
            input_register_used: DirectionsOpt {
                north: true,
                south: true,
                west: false,
                east: true,
            },
            input_register_write: DirectionsOpt {
                north: true,
                south: false,
                west: false,
                east: false,
            },
        };
        let binary = router_config.to_u64();
        let router_config_from_binary = RouterConfig::from_u64(binary);
        assert_eq!(router_config, router_config_from_binary);
    }
}
