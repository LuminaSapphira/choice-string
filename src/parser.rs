use crate::{Selection, SomeElementType};
use nom::branch::alt;
use nom::bytes::complete::{is_a, tag, tag_no_case};
use nom::character::complete::{digit1, space1};

use nom::combinator::{complete, cut, eof, map, map_res};

use nom::multi::many1;
use nom::sequence::tuple;
use nom::IResult;

use std::str::FromStr;

/// Selects `/(none)?$/`
fn select_none(input: &str) -> IResult<&str, Selection> {
    fn none_literal(input: &str) -> IResult<&str, Selection> {
        map(tuple((tag_no_case("none"), cut(eof))), |_| Selection::None)(input)
    }
    alt((map(eof, |_| Selection::None), none_literal))(input)
}

/// Selects `/^((([0-9]+-[0-9]+)|([0-9]+))( ;,)*)+$/`
fn select_some(input: &str) -> IResult<&str, Selection> {
    /// The (last) `/([0-9]+)/` part
    fn individual_element(input: &str) -> IResult<&str, SomeElementType> {
        map(
            map_res(digit1, usize::from_str),
            SomeElementType::Individual,
        )(input)
    }
    /// The `/([0-9]+-[0-9]+)/` part
    fn range_element(input: &str) -> IResult<&str, SomeElementType> {
        map(
            tuple((
                map_res(digit1, usize::from_str),
                is_a("-"),
                map_res(cut(digit1), usize::from_str),
            )),
            |(start, _, end)| SomeElementType::Range(start..=end),
        )(input)
    }
    /// The `/(([0-9]+-[0-9]+)|([0-9]+))/` part
    fn some_element(input: &str) -> IResult<&str, SomeElementType> {
        alt((range_element, individual_element))(input)
    }
    /// The `/( ;,)*/` part
    fn element_separator(input: &str) -> IResult<&str, &str> {
        alt((map(many1(alt((tag(","), tag(";"), space1))), |_| ""), eof))(input)
    }
    map(
        tuple((
            many1(map(
                tuple((some_element, element_separator)),
                |(etype, _rem)| etype,
            )),
            eof,
        )),
        |a| Selection::Some(a.0),
    )(input)
}

/// Selects `/all$/`
fn select_all(input: &str) -> IResult<&str, Selection> {
    map(tuple((tag_no_case("all"), cut(eof))), |_| Selection::All)(input)
}

/// Parses the full selection
fn selection(input: &str) -> IResult<&str, Selection> {
    complete(alt((select_none, select_all, select_some)))(input)
}

/// Parses a choice string to a [`Selection`]. This does not do any de-duplicating or condensing of
/// parsed ranges.
pub fn parse(input: &str) -> Result<Selection, crate::Error> {
    match selection(input) {
        Ok((_, sel)) => Ok(sel),
        Err(err) => {
            if let nom::Err::Failure(error) = err {
                Err(crate::Error::ParsingFailed(error.code))
            } else {
                panic!("Internal parser error");
            }
        }
    }
}

#[cfg(test)]
mod parser_tests {

    use super::*;
    use nom::error::ErrorKind;

    macro_rules! does_parse {
        ($input:literal, $name:ident) => {
            #[test]
            fn $name() {
                selection($input).unwrap();
            }
        };
    }

    does_parse!("all", all);
    does_parse!("none", none);
    does_parse!("", none_no_input);
    does_parse!("1", single_digit);
    does_parse!("1-9", single_range);
    does_parse!("8-2", inverse_range);
    does_parse!("1-90", single_range_multi_digit_end);
    does_parse!("10-90", single_range_multi_digit_start_end);
    does_parse!("10-0", single_range_multi_digit_start);

    does_parse!("1 2 3", multiple_individuals);
    does_parse!("1-3 5-8", multiple_ranges);
    does_parse!("1    5 8", multiple_separators);
    does_parse!("1,,,,5,8", multiple_separators_2);
    does_parse!("1;;;;5;;;;8", multiple_separators_3);
    does_parse!("1;,,;5 ,;  ;8", multiple_separators_mixed);
    does_parse!("1;5;8", separators_1);
    does_parse!("1,5,8", separators_2);

    does_parse!("1-10 15 20", mixed_elements);

    #[test]
    fn fails_broken_range_start() {
        selection("1-").unwrap_err();
    }

    #[test]
    fn fails_broken_range_both() {
        selection("-").unwrap_err();
    }

    #[test]
    fn fails_broken_range_end() {
        selection("-5").unwrap_err();
    }

    #[test]
    fn content_none() {
        assert_eq!(parse("").unwrap(), Selection::None);
    }

    #[test]
    fn content_none2() {
        assert_eq!(parse("none").unwrap(), Selection::None);
    }

    #[test]
    fn content_all() {
        assert_eq!(parse("all").unwrap(), Selection::All);
    }

    #[test]
    fn content_individual() {
        assert_eq!(
            parse("8").unwrap(),
            Selection::Some(vec![SomeElementType::Individual(8)])
        );
    }

    #[test]
    fn content_individual_multi() {
        assert_eq!(
            parse("8 9 10").unwrap(),
            Selection::Some(vec![
                SomeElementType::Individual(8),
                SomeElementType::Individual(9),
                SomeElementType::Individual(10)
            ])
        );
    }

    #[test]
    fn content_individual_multi_ranges_individuals() {
        assert_eq!(
            parse("8 9-12 4").unwrap(),
            Selection::Some(vec![
                SomeElementType::Individual(8),
                SomeElementType::Range(9..=12),
                SomeElementType::Individual(4)
            ])
        );
    }

    #[test]
    fn test_error() {
        let err = parse("1 3 5 6-8 1-;455").unwrap_err();
        match err {
            crate::Error::ParsingFailed(kind) => assert_eq!(kind, ErrorKind::Digit),
            #[allow(unreachable_patterns)]
            _ => panic!("Wrong kind"),
        }
    }
}
