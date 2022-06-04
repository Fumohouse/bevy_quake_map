use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, is_not, tag, take_till},
    character::complete::{char, multispace0, multispace1},
    combinator::{map, opt, value, verify},
    error::{context, ContextError, ParseError},
    multi::{many0, separated_list0},
    sequence::{delimited, pair},
    IResult,
};

/// Matches a string delimited by word/line end
pub fn identifier<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let ends = " \t\r\n";

    take_till(move |c| ends.contains(c))(i)
}

/// Matches a comment and discards it
pub fn comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
    value((), pair(tag("//"), is_not("\r\n")))(i)
}

/// Matches any whitespace or comment and discards it
pub fn ignored<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
    value(
        (),
        many0(alt((map(multispace1, |_| ()), map(comment, |_| ())))),
    )(i)
}

/// Parses a whitespace-separated list of floats, of a specific length
pub fn generic_list<'a, T, E: ParseError<&'a str>>(
    n: usize,
    parser: fn(&'a str) -> IResult<&'a str, T, E>
) -> impl FnMut(&'a str) -> IResult<&'a str, Vec<T>, E> {
    move |i| {
        verify(
            delimited(
                opt(multispace0),
                separated_list0(multispace1, parser),
                opt(multispace0),
            ),
            |v: &Vec<T>| v.len() == n,
        )(i)
    }
}

pub fn escaped_string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, String, E> {
    context(
        "string",
        delimited(
            char('\"'),
            escaped_transform(
                is_not("\\\""),
                '\\',
                alt((
                    value("\\", tag("\\")),
                    value("\"", tag("\"")),
                    value("\n", tag("\n")),
                )),
            ),
            char('\"'),
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use nom::number::complete::float;

    use super::{comment, escaped_string, generic_list, identifier, ignored};

    #[test]
    fn test_parse_identifier() {
        assert_eq!(identifier::<()>("test asdf"), Ok((" asdf", "test")));
    }

    #[test]
    fn test_parse_comment() {
        assert_eq!(comment::<()>("// this is a test"), Ok(("", ())));
    }

    #[test]
    fn test_parse_ignored() {
        assert_eq!(ignored::<()>(""), Ok(("", ())));

        assert_eq!(
            ignored::<()>(" // this is a test\r\nasdf"),
            Ok(("asdf", ()))
        );
    }

    #[test]
    fn test_parse_float_list() {
        assert_eq!(
            generic_list::<_, ()>(4, float)("   0.0  1.0 3.0 5.0"),
            Ok(("", vec![0.0, 1.0, 3.0, 5.0]))
        );

        assert!(generic_list::<_, ()>(5, float)("1.0 2.0 3.0 4.0").is_err());

        assert!(generic_list::<_, ()>(2, float)("1.0, 2.0").is_err());

        assert!(generic_list::<_, ()>(2, float)("1.302.2").is_err());
    }

    #[test]
    fn test_parse_escaped_string() {
        assert_eq!(escaped_string::<()>("\"\\\"\""), Ok(("", "\"".to_string())));

        assert!(escaped_string::<()>("\"\\\"").is_err());
    }
}
