pub mod mnemonics {
    use std::{collections::HashMap, fmt::Display};

    use nom::{
        IResult, Parser,
        branch::alt,
        bytes::complete::tag,
        character::complete::multispace0,
        combinator::map,
        multi::{separated_list0, separated_list1},
        sequence::{delimited, preceded},
    };

    use crate::isa::router::{
        Direction, DirectionsOpt, RouterConfig, RouterInDir, RouterSwitchConfig,
    };

    // Parser for the enum variants
    fn parse_router_in(input: &str) -> IResult<&str, RouterInDir> {
        let (input, var) = alt((
            map(tag("EastIn"), |_| RouterInDir::EastIn),
            map(tag("SouthIn"), |_| RouterInDir::SouthIn),
            map(tag("WestIn"), |_| RouterInDir::WestIn),
            map(tag("NorthIn"), |_| RouterInDir::NorthIn),
            map(tag("ALUOut"), |_| RouterInDir::ALUOut),
            map(tag("ALURes"), |_| RouterInDir::ALURes),
            map(tag("Open"), |_| RouterInDir::Open),
        ))
        .parse(input)?;
        Ok((input, var))
    }

    fn parse_router_out_field(input: &str) -> IResult<&str, String> {
        alt((
            map(tag("predicate"), |_| String::from("predicate")),
            map(tag("alu_op1"), |_| String::from("alu_op1")),
            map(tag("alu_op2"), |_| String::from("alu_op2")),
            map(tag("east_out"), |_| String::from("east_out")),
            map(tag("south_out"), |_| String::from("south_out")),
            map(tag("west_out"), |_| String::from("west_out")),
            map(tag("north_out"), |_| String::from("north_out")),
        ))
        .parse(input)
    }

    fn parse_assignment(input: &str) -> IResult<&str, (String, RouterInDir)> {
        let (input, (_, dir, _, field, _, _, _)) = (
            multispace0,
            parse_router_in,
            delimited(multispace0, tag("->"), multispace0),
            parse_router_out_field,
            multispace0,
            tag(","),
            multispace0,
        )
            .parse(input)?;
        Ok((input, (field, dir)))
    }

    pub fn parse_switching_config(input: &str) -> IResult<&str, RouterSwitchConfig> {
        let (input, _) = multispace0(input)?;
        let (input, _) = tag("switch_config")(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = tag(":")(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = tag("{")(input)?;
        let (input, _) = multispace0(input)?;
        let (input, assignments) = separated_list1(multispace0, parse_assignment).parse(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = tag("}")(input)?;
        let (input, _) = multispace0(input)?;
        let (input, _) = tag(";")(input)?;
        let (input, _) = multispace0(input)?;

        // Collect into a map for easy lookup
        let mut map: HashMap<String, RouterInDir> = HashMap::new();
        for (field, dir) in assignments {
            map.insert(field, dir);
        }

        // Extract each required field or default to Open
        let mut get = |name| -> RouterInDir { map.remove(name).unwrap_or(RouterInDir::Open) };

        let config = RouterSwitchConfig {
            predicate: get("predicate"),
            alu_op1: get("alu_op1"),
            alu_op2: get("alu_op2"),
            east_out: get("east_out"),
            south_out: get("south_out"),
            west_out: get("west_out"),
            north_out: get("north_out"),
        };

        Ok((input, config))
    }

    fn parse_direction(input: &str) -> IResult<&str, Direction> {
        let (input, dir) = alt((
            map(tag("east"), |_| Direction::East),
            map(tag("south"), |_| Direction::South),
            map(tag("west"), |_| Direction::West),
            map(tag("north"), |_| Direction::North),
        ))
        .parse(input)?;
        Ok((input, dir))
    }

    fn parse_directions_opt(input: &str) -> IResult<&str, DirectionsOpt> {
        let (input, dirs) = delimited(
            preceded(multispace0, tag("{")),
            separated_list0(
                delimited(multispace0, tag(","), multispace0),
                parse_direction,
            ),
            preceded(multispace0, tag("}")),
        )
        .parse(input)?;

        let mut result = DirectionsOpt::default();
        for d in dirs {
            match d {
                Direction::East => result.east = true,
                Direction::South => result.south = true,
                Direction::West => result.west = true,
                Direction::North => result.north = true,
            }
        }
        Ok((input, result))
    }

    /// Parse a field name and its direction list: e.g.
    /// "input_register_bypass: {north, south};"
    fn parse_named_directions(input: &str) -> IResult<&str, (String, DirectionsOpt)> {
        let (input, _) = multispace0(input)?;
        let (input, name) = alt((
            map(tag("input_register_bypass"), |_| "input_register_bypass"),
            map(tag("input_register_write"), |_| "input_register_write"),
        ))
        .parse(input)?;
        let (input, _) = preceded(multispace0, tag(":")).parse(input)?;
        let (input, dirs) = parse_directions_opt(input)?;
        let (input, _) = preceded(multispace0, tag(";")).parse(input)?;
        Ok((input, (name.to_string(), dirs)))
    }

    pub fn parse_extra_config(input: &str) -> IResult<&str, (DirectionsOpt, DirectionsOpt)> {
        let mut bypass = DirectionsOpt::default();
        let mut write = DirectionsOpt::default();
        let (input, (name, dirs1)) = parse_named_directions(input)?;
        match name.as_str() {
            "input_register_bypass" => bypass = dirs1,
            "input_register_write" => write = dirs1,
            _ => {
                panic!("Unknown field for router extra config: {}", name);
            }
        }
        let (input, (name, dirs2)) = parse_named_directions(input)?;
        match name.as_str() {
            "input_register_bypass" => {
                if bypass == DirectionsOpt::default() {
                    bypass = dirs2
                } else {
                    panic!("Multiple input_register_bypass fields found");
                }
            }
            "input_register_write" => {
                if write == DirectionsOpt::default() {
                    write = dirs2
                } else {
                    panic!("Multiple input_register_write fields found");
                }
            }
            _ => {
                panic!("Unknown field for router extra config: {}", name);
            }
        }
        Ok((input, (bypass, write)))
    }

    pub fn parse_router_config(input: &str) -> IResult<&str, RouterConfig> {
        let (input, switch_config) = parse_switching_config(input)?;
        let (input, extra_config) = parse_extra_config(input)?;
        Ok((
            input,
            RouterConfig {
                switch_config,
                input_register_bypass: extra_config.0,
                input_register_write: extra_config.1,
            },
        ))
    }

    impl Display for RouterInDir {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                RouterInDir::EastIn => write!(f, "EastIn"),
                RouterInDir::SouthIn => write!(f, "SouthIn"),
                RouterInDir::WestIn => write!(f, "WestIn"),
                RouterInDir::NorthIn => write!(f, "NorthIn"),
                RouterInDir::ALUOut => write!(f, "ALUOut"),
                RouterInDir::ALURes => write!(f, "ALURes"),
                RouterInDir::Invalid => write!(f, "Invalid"),
                RouterInDir::Open => write!(f, "Open"),
            }
        }
    }

    impl RouterConfig {
        pub fn to_mnemonics(&self) -> String {
            format!(
                "switch_config: {};\ninput_register_bypass: {};\ninput_register_write: {};",
                self.switch_config.to_mnemonics(),
                self.input_register_bypass.to_mnemonics(),
                self.input_register_write.to_mnemonics()
            )
        }
    }

    impl RouterSwitchConfig {
        pub fn to_mnemonics(&self) -> String {
            // Syntax:
            // switch_config: {
            //     Open -> predicate,
            //     SouthIn -> south_out,
            //     WestIn -> west_out,
            //     NorthIn -> north_out,
            //     EastIn -> east_out,
            //     ALURes -> alu_op2,
            //     ALUOut -> alu_op1,
            // };
            let mut result = String::new();
            result.push_str("{\n");
            result.push_str(&format!("    {} -> predicate,\n", self.predicate));
            result.push_str(&format!("    {} -> south_out,\n", self.south_out));
            result.push_str(&format!("    {} -> west_out,\n", self.west_out));
            result.push_str(&format!("    {} -> north_out,\n", self.north_out));
            result.push_str(&format!("    {} -> east_out,\n", self.east_out));
            result.push_str(&format!("    {} -> alu_op2,\n", self.alu_op2));
            result.push_str(&format!("    {} -> alu_op1,\n", self.alu_op1));
            result.push_str("}");
            result
        }
    }

    impl DirectionsOpt {
        pub fn to_mnemonics(&self) -> String {
            let mut result = String::new();
            result.push_str("{");
            if self.north {
                result.push_str("north");
            }
            if self.south {
                result.push_str("south");
            }
            if self.west {
                result.push_str("west");
            }
            if self.east {
                result.push_str("east");
            }
            result.push_str("}");
            result
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_parse_assignment() {
            let input = "Open -> predicate,";
            let (_, (field, dir)) = parse_assignment(input).unwrap();
            assert_eq!(field, "predicate");
            assert_eq!(dir, RouterInDir::Open);
            let input = "SouthIn -> south_out,";
            let (_, (field, dir)) = parse_assignment(input).unwrap();
            assert_eq!(field, "south_out");
            assert_eq!(dir, RouterInDir::SouthIn);
            let input = "WestIn -> west_out,";
            let (_, (field, dir)) = parse_assignment(input).unwrap();
            assert_eq!(field, "west_out");
            assert_eq!(dir, RouterInDir::WestIn);
            let input = "NorthIn -> north_out,";
            let (_, (field, dir)) = parse_assignment(input).unwrap();
            assert_eq!(field, "north_out");
            assert_eq!(dir, RouterInDir::NorthIn);
            let input = "EastIn -> east_out,";
            let (_, (field, dir)) = parse_assignment(input).unwrap();
            assert_eq!(field, "east_out");
            assert_eq!(dir, RouterInDir::EastIn);
            let input = "ALURes -> alu_op2,";
            let (_, (field, dir)) = parse_assignment(input).unwrap();
            assert_eq!(field, "alu_op2");
            assert_eq!(dir, RouterInDir::ALURes);
            let input = "ALUOut -> alu_op1,";
            let (_, (field, dir)) = parse_assignment(input).unwrap();
            assert_eq!(field, "alu_op1");
            assert_eq!(dir, RouterInDir::ALUOut);
        }

        #[test]
        fn test_parse_switching_config() {
            let input = r"switch_config: {
            Open -> predicate,
            SouthIn -> south_out,
            WestIn -> west_out,
            NorthIn -> north_out,
            EastIn -> east_out,
            ALURes -> alu_op2,
            ALUOut -> alu_op1,
        };";
            let (_, cfg) = parse_switching_config(input).unwrap();
            let expected = RouterSwitchConfig {
                predicate: RouterInDir::Open,
                alu_op1: RouterInDir::ALUOut,
                alu_op2: RouterInDir::ALURes,
                east_out: RouterInDir::EastIn,
                south_out: RouterInDir::SouthIn,
                west_out: RouterInDir::WestIn,
                north_out: RouterInDir::NorthIn,
            };
            assert_eq!(cfg, expected);
        }

        #[test]
        fn test_parse_directions_opt() {
            let (_, dirs) = parse_directions_opt("{west,south,east,north}").unwrap();
            assert_eq!(
                dirs,
                DirectionsOpt {
                    east: true,
                    south: true,
                    west: true,
                    north: true
                }
            );
        }

        #[test]
        fn test_parse_router_config() {
            let input = r"switch_config: {
            Open -> predicate,
            SouthIn -> south_out,
            WestIn -> west_out,
            NorthIn -> north_out,
            EastIn -> east_out,
            ALURes -> alu_op2,
            ALUOut -> alu_op1,
        };
        input_register_bypass: {north, south};
        input_register_write: {east, west};";
            let (_, cfg) = parse_router_config(input).unwrap();
            let expected = RouterConfig {
                switch_config: RouterSwitchConfig {
                    predicate: RouterInDir::Open,
                    alu_op1: RouterInDir::ALUOut,
                    alu_op2: RouterInDir::ALURes,
                    east_out: RouterInDir::EastIn,
                    south_out: RouterInDir::SouthIn,
                    west_out: RouterInDir::WestIn,
                    north_out: RouterInDir::NorthIn,
                },
                input_register_bypass: DirectionsOpt {
                    north: true,
                    south: true,
                    ..Default::default()
                },
                input_register_write: DirectionsOpt {
                    east: true,
                    west: true,
                    ..Default::default()
                },
            };
            assert_eq!(cfg, expected);
        }
    }
}

pub mod binary {
    use crate::isa::{
        binary::{ConfigField, ConfigurationField},
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
        pub fn from_binary(code: u32) -> Self {
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
        pub fn to_binary(&self) -> u32 {
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
        pub fn from_binary(code: u64) -> Self {
            let switch_config = RouterSwitchConfig::from_binary(
                code.get_field(ConfigField::RouterSwitchConfig) as u32,
            );
            let input_register_bypass =
                DirectionsOpt::from_binary(code.get_field(ConfigField::RouterBypass) as u8);
            let input_register_write =
                DirectionsOpt::from_binary(code.get_field(ConfigField::RouterWriteEnable) as u8);
            Self {
                switch_config,
                input_register_bypass,
                input_register_write,
            }
        }

        pub fn to_binary(&self) -> u64 {
            let mut code: u64 = 0;
            code.set_field(
                ConfigField::RouterSwitchConfig,
                self.switch_config.to_binary() as u32,
            );
            code.set_field(
                ConfigField::RouterBypass,
                self.input_register_bypass.to_binary() as u32,
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
            let binary = switch_config.to_binary();
            let switch_config_from_binary = RouterSwitchConfig::from_binary(binary);
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
                input_register_bypass: DirectionsOpt {
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
            let binary = router_config.to_binary();
            let router_config_from_binary = RouterConfig::from_binary(binary);
            assert_eq!(router_config, router_config_from_binary);
        }
    }
}
