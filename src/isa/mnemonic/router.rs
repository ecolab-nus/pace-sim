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

use crate::isa::router::{Direction, DirectionsOpt, RouterConfig, RouterInDir, RouterSwitchConfig};

impl Display for RouterConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_mnemonics())
    }
}

impl RouterConfig {
    pub fn to_mnemonics(&self) -> String {
        format!(
            "switch_config: {};\ninput_register_used: {};\ninput_register_write: {};",
            self.switch_config, self.input_register_used, self.input_register_write
        )
    }

    /// Parse the router configuration
    pub fn parse_router_config(input: &str) -> IResult<&str, RouterConfig> {
        let (input, switch_config) = parse_switching_config(input)?;
        let (input, extra_config) = parse_extra_config(input)?;
        Ok((
            input,
            RouterConfig {
                switch_config,
                input_register_used: extra_config.0,
                input_register_write: extra_config.1,
            },
        ))
    }
}
/// Parser the the RouterInDir enum variants
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

/// Parse a single assignment of a RouterInDir to a RouterOutDir
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

/// Parse the switching configuration of the router
/// The switching configuration is a map of RouterInDir to RouterOutDir
/// The order of the assignments is important, as it determines the order of the outputs
/// The default output is Open
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

/// Parse a single direction
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

/// Parse a set of directions, e.g. {north, south, west, east} or {all}
fn parse_directions_opt(input: &str) -> IResult<&str, DirectionsOpt> {
    let (input, dirs) = delimited(
        preceded(multispace0, tag("{")),
        alt((
            // Parse "all" as a special case
            map(tag("all"), |_| {
                vec![
                    Direction::North,
                    Direction::South,
                    Direction::West,
                    Direction::East,
                ]
            }),
            // Parse individual directions
            separated_list0(
                delimited(multispace0, tag(","), multispace0),
                parse_direction,
            ),
        )),
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
/// "input_register_used: {north, south};"
fn parse_named_directions(input: &str) -> IResult<&str, (String, DirectionsOpt)> {
    let (input, _) = multispace0(input)?;
    let (input, name) = alt((
        map(tag("input_register_used"), |_| "input_register_used"),
        map(tag("input_register_write"), |_| "input_register_write"),
    ))
    .parse(input)?;
    let (input, _) = preceded(multispace0, tag(":")).parse(input)?;
    let (input, dirs) = parse_directions_opt(input)?;
    let (input, _) = preceded(multispace0, tag(";")).parse(input)?;
    Ok((input, (name.to_string(), dirs)))
}

/// Parse the extra configuration of the router, i.e. input_register_used and input_register_write
/// Each is a set of directions, e.g. {north, south, west, east} or {all}
pub fn parse_extra_config(input: &str) -> IResult<&str, (DirectionsOpt, DirectionsOpt)> {
    let mut used = DirectionsOpt::default();
    let mut write = DirectionsOpt::default();
    let (input, (name, dirs1)) = parse_named_directions(input)?;
    match name.as_str() {
        "input_register_used" => used = dirs1,
        "input_register_write" => write = dirs1,
        _ => {
            panic!("Unknown field for router extra config: {}", name);
        }
    }
    let (input, (name, dirs2)) = parse_named_directions(input)?;
    match name.as_str() {
        "input_register_used" => {
            if used == DirectionsOpt::default() {
                used = dirs2
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
    Ok((input, (used, write)))
}

impl Display for RouterInDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_mnemonics())
    }
}

impl RouterInDir {
    pub fn to_mnemonics(&self) -> String {
        let mut result = String::new();
        match self {
            RouterInDir::EastIn => result.push_str("EastIn"),
            RouterInDir::SouthIn => result.push_str("SouthIn"),
            RouterInDir::WestIn => result.push_str("WestIn"),
            RouterInDir::NorthIn => result.push_str("NorthIn"),
            RouterInDir::ALUOut => result.push_str("ALUOut"),
            RouterInDir::ALURes => result.push_str("ALURes"),
            RouterInDir::Invalid => result.push_str("Invalid"),
            RouterInDir::Open => result.push_str("Open"),
        }
        result
    }
}

impl Display for RouterSwitchConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_mnemonics())
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

impl Display for DirectionsOpt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_mnemonics())
    }
}

impl DirectionsOpt {
    pub fn to_mnemonics(&self) -> String {
        // If all directions are selected, use the "all" shortcut
        if self.north && self.south && self.west && self.east {
            return String::from("{all}");
        }

        let mut result = String::new();
        result.push_str("{");

        let mut directions = Vec::new();
        if self.north {
            directions.push("north");
        }
        if self.south {
            directions.push("south");
        }
        if self.west {
            directions.push("west");
        }
        if self.east {
            directions.push("east");
        }

        result.push_str(&directions.join(","));
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
    fn test_parse_directions_opt_all() {
        let (_, dirs) = parse_directions_opt("{all}").unwrap();
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
    fn test_directions_opt_to_mnemonics() {
        // Test individual directions
        let dirs = DirectionsOpt {
            north: true,
            south: false,
            west: true,
            east: false,
        };
        assert_eq!(dirs.to_mnemonics(), "{north,west}");

        // Test all directions should use "all" shortcut
        let dirs_all = DirectionsOpt {
            north: true,
            south: true,
            west: true,
            east: true,
        };
        assert_eq!(dirs_all.to_mnemonics(), "{all}");

        // Test empty directions
        let dirs_empty = DirectionsOpt::default();
        assert_eq!(dirs_empty.to_mnemonics(), "{}");
    }

    #[test]
    fn test_directions_opt_roundtrip() {
        // Test that parsing "{all}" and converting back gives "{all}"
        let (_, dirs) = parse_directions_opt("{all}").unwrap();
        assert_eq!(dirs.to_mnemonics(), "{all}");

        // Test that parsing individual directions maintains them
        let (_, dirs) = parse_directions_opt("{north,west}").unwrap();
        assert_eq!(dirs.to_mnemonics(), "{north,west}");
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
        input_register_used: {north, south};
        input_register_write: {east, west};";
        let (_, cfg) = RouterConfig::parse_router_config(input).unwrap();
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
            input_register_used: DirectionsOpt {
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
