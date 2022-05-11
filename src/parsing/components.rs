//! Parsers for components of the .map file

use std::collections::HashMap;

use nom::{
    character::complete::{char, multispace0, multispace1},
    combinator::map,
    error::{context, ContextError, ParseError},
    multi::{count, many0, many1},
    number::complete::float,
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

use super::{
    data::{Brush, BrushFace, Entity, Point, UvAxis},
    util::{escaped_string, float_list, identifier, ignored},
};

/// Matches a 3D coordinate in ( x y z ) form, as used in brushes
fn point<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Point, E> {
    context(
        "point",
        map(delimited(char('('), float_list(3), char(')')), |v| Point {
            x: v[0],
            y: v[1],
            z: v[2],
        }),
    )(i)
}

/// Matches Valve 220 style UV settings in [ Tx, Ty, Tz, Toffset ] form
fn uv_axis<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, UvAxis, E> {
    context(
        "uv_axis",
        map(delimited(char('['), float_list(4), char(']')), |v| UvAxis {
            x: v[0],
            y: v[1],
            z: v[2],
            offset: v[3],
        }),
    )(i)
}

/// Matches a brush face definition (one line in a (normal) brush, although they can technically all be in one line)
fn brush_face<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, BrushFace, E> {
    context(
        "brush_face",
        map(
            tuple((
                count(terminated(point, multispace0), 3),
                terminated(identifier, multispace1),
                count(terminated(uv_axis, multispace0), 2),
                terminated(float, multispace0),
                terminated(float, multispace0),
                float,
            )),
            |t| {
                let (points, texture, mut uv_axes, rotation, x_scale, y_scale) = t;

                BrushFace {
                    points: points.try_into().unwrap(),
                    texture: texture.to_string(),
                    u: uv_axes.remove(0),
                    v: uv_axes.remove(0),
                    rotation,
                    x_scale,
                    y_scale,
                }
            },
        ),
    )(i)
}

fn brush<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Brush, E> {
    context(
        "brush",
        map(
            delimited(
                char('{'),
                many1(delimited(ignored, brush_face, ignored)),
                char('}'),
            ),
            |v| Brush { faces: v },
        ),
    )(i)
}

pub fn entity<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, Entity, E> {
    context(
        "entity",
        map(
            delimited(
                terminated(char('{'), ignored),
                tuple((
                    many1(terminated(
                        pair(escaped_string, preceded(ignored, escaped_string)),
                        ignored,
                    )),
                    many0(terminated(brush, ignored)),
                )),
                preceded(ignored, char('}')),
            ),
            |(prop_tuples, brushes)| {
                let mut properties = HashMap::new();

                for (key, value) in prop_tuples {
                    properties.insert(key, value);
                }

                Entity {
                    properties,
                    brushes,
                }
            },
        ),
    )(i)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        super::data::{Brush, BrushFace, Entity, Point, UvAxis},
        brush, brush_face, entity, point, uv_axis,
    };

    fn test_brush_face(i: f32) -> BrushFace {
        BrushFace {
            points: [
                Point {
                    x: i,
                    y: i + 1.0,
                    z: i + 2.0,
                },
                Point {
                    x: i + 3.0,
                    y: i + 4.0,
                    z: i + 5.0,
                },
                Point {
                    x: i + 6.0,
                    y: i + 7.0,
                    z: i + 8.0,
                },
            ],
            texture: "TEXTURE".to_string(),
            u: UvAxis {
                x: i + 9.0,
                y: i + 10.0,
                z: i + 11.0,
                offset: i + 12.0,
            },
            v: UvAxis {
                x: i + 13.0,
                y: i + 14.0,
                z: i + 15.0,
                offset: i + 16.0,
            },
            rotation: i + 17.0,
            x_scale: i + 18.0,
            y_scale: i + 19.0,
        }
    }

    #[test]
    fn test_parse_point() {
        assert_eq!(
            point::<()>("( 16 16.0 0.375 )"),
            Ok((
                "",
                Point {
                    x: 16.0,
                    y: 16.0,
                    z: 0.375
                }
            ))
        );
    }

    #[test]
    fn test_parse_uv_axis() {
        assert_eq!(
            uv_axis::<()>("[ -1.0 0.0 1.0 2.0 ]"),
            Ok((
                "",
                UvAxis {
                    x: -1.0,
                    y: 0.0,
                    z: 1.0,
                    offset: 2.0,
                }
            ))
        )
    }

    #[test]
    fn test_parse_brush_face() {
        assert_eq!(
            brush_face::<()>(
                "( 0 1 2 ) (3 4 5) (6.0 7 8)\n\
                TEXTURE [9 10 11 12]    [13 14 15 16  ]   17 18 19.0000 asdf"
            ),
            Ok((" asdf", test_brush_face(0.0)))
        )
    }

    #[test]
    fn test_parse_brush() {
        assert_eq!(
            brush::<()>(
                "{
                (0 1 2) (3 4 5) (6 7 8) TEXTURE [9 10 11 12] [13 14 15 16] 17 18 19 // test comment
                (20 21 22) (23 24 25)
                (26 27 28) TEXTURE [29 30 31 32] [33 34 35 36] 37 38 39 (40 41 42) (43 44 45) (46 47 48)
                    TEXTURE [49 50 51 52] [53 54 55 56] 57 58 59
                } asdf"
            ),
            Ok((" asdf", Brush {
                faces: vec![test_brush_face(0.0), test_brush_face(20.0), test_brush_face(40.0)]
            }))
        );
    }

    #[test]
    fn test_parse_entity() {
        let mut properties = HashMap::new();
        properties.insert("classname".to_string(), "worldspawn".to_string());
        properties.insert("mapversion".to_string(), "220".to_string());

        assert_eq!(
            entity::<nom::error::VerboseError<&str>>(
                "{
                \"classname\"                 \"worldspawn\" // comment \r\n\r\n
                \"mapversion\"\t\t\t\"220\"\n
                {
                (0 1 2) (3 4 5) (6 7 8) TEXTURE [9 10 11 12] [13 14 15 16] 17 18 19 // comment
                } // comment
                } asdf"
            ),
            Ok((
                " asdf",
                Entity {
                    properties,
                    brushes: vec![Brush {
                        faces: vec![test_brush_face(0.0)]
                    }]
                }
            ))
        );
    }
}
