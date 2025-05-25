use std::collections::HashMap;

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
    Direction, DirectionsOpt, RouterConfig, RouterExtraConfig, RouterInDir, RouterSwitchConfig,
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

pub fn parse_extra_config(input: &str) -> IResult<&str, RouterExtraConfig> {
    let mut bypass = DirectionsOpt::default();
    let mut write = DirectionsOpt::default();
    let (input, (name, dirs1)) = parse_named_directions(input)?;
    match name.as_str() {
        "input_register_bypass" => bypass = dirs1,
        "input_register_write" => write = dirs1,
        _ => {
            println!("Unknown field for router extra config: {}", name);
        }
    }
    let (input, (name, dirs2)) = parse_named_directions(input)?;
    match name.as_str() {
        "input_register_bypass" => {
            if bypass.is_default() {
                bypass = dirs2
            } else {
                panic!("Multiple input_register_bypass fields found");
            }
        }
        "input_register_write" => {
            if write.is_default() {
                write = dirs2
            } else {
                panic!("Multiple input_register_write fields found");
            }
        }
        _ => {
            panic!("Unknown field for router extra config: {}", name);
        }
    }
    Ok((
        input,
        RouterExtraConfig {
            input_register_bypass: bypass,
            input_register_write: write,
        },
    ))
}

pub fn parse_router_config(input: &str) -> IResult<&str, RouterConfig> {
    let (input, switch_config) = parse_switching_config(input)?;
    let (input, extra_config) = parse_extra_config(input)?;
    Ok((
        input,
        RouterConfig {
            switch_config,
            extra_config,
        },
    ))
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
            extra_config: RouterExtraConfig {
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
            },
        };
        assert_eq!(cfg, expected);
    }

    #[test]
    fn test_parse_router_config_file() {
        let input = include_str!("tests/router_config.cfg");
        let (_, cfg) = parse_router_config(input).unwrap();
        println!("{:?}", cfg);
    }
}
