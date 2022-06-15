//! A parser for Quake .map files, targeted at Valve format.
//! It aims to be compatible with (i.e. as lenient as) TrenchBroom in what kind of inputs it accepts.

use nom::{
    combinator::map,
    error::{context, ContextError, ParseError},
    multi::many1,
    sequence::{delimited, terminated},
    IResult,
};

use crate::map_data::Map;

pub mod components;
use components::entity;

mod util;
use util::ignored;

pub fn parse_map<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Map, E> {
    context(
        "map",
        map(
            delimited(ignored, many1(terminated(entity, ignored)), ignored),
            |entities| Map { entities },
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::parse_map;
    use crate::test_utils;

    #[test]
    fn test_parse_map() {
        assert!(parse_map::<()>(test_utils::TEST_MAP).is_ok());
    }
}
