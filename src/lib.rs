use nom::error::ErrorKind;
use std::ops::RangeInclusive;
use std::str::FromStr;

/// Parser-related functions
mod parser;

/// Error type for errors that may arise during the parsing of choice-strings.
/// Very simple at the moment, only wrapping a nom [`ErrorKind`]
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Error {
    #[error("Invalid token {}", 0)]
    ParsingFailed(ErrorKind),
}

/// A parsed selection. Can represent all, none, or some set of ranges and items.
#[derive(Debug, PartialEq)]
pub enum Selection {
    /// All elements are in the selected set
    All,
    /// Some elements are in the selected set. The list of selections are in the Vec.
    Some(Vec<SomeElementType>),
    /// No elements are in the selected set
    None,
}

impl Selection {
    /// Check if the selection contains an item. Returns true if All, false if None,
    /// and checks included elements if Some.
    pub fn contains_item(&self, item: usize) -> bool {
        match self {
            Selection::Some(v) => {
                v.iter()
                    .any(|element| {
                        match element {
                            SomeElementType::Individual(num) => item == *num,
                            SomeElementType::Range(range) => range.contains(&item),
                        }
                    })
            },
            Selection::All => true,
            Selection::None => false,
        }
    }
}

/// A selected element. Can either be an individual item, or a range of items.
#[derive(Debug, PartialEq)]
pub enum SomeElementType {
    Individual(usize),
    Range(RangeInclusive<usize>),
}

pub use parser::parse as parse_raw;

/// Parse a choice string input to a [`Selection`]. Additionally reduces the set of ranges to the
/// minimum representable by using a union operation.
/// Wrapper for [`str::parse`].
pub fn parse(input: &str) -> Result<Selection, Error> {
    input.parse()
}

impl FromStr for Selection {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match parse_raw(s)? {
            Selection::Some(v) => Selection::Some(condense_selections(v)),
            other => other,
        })
    }
}

/// Union the elements together to produce a set of individual elements and ranges that represents
/// the same set and reduces the amount of elements.
fn condense_selections(selections: Vec<SomeElementType>) -> Vec<SomeElementType> {
    let mut union = range_union_find::IntRangeUnionFind::new();
    selections
        .iter()
        .map(|a| match a {
            SomeElementType::Individual(num) => *num..=*num,
            SomeElementType::Range(range) => range.clone(),
        })
        .filter(|a| !a.is_empty())
        .try_for_each(|a| union.insert_range(&a))
        .expect("bad ranges - shouldn't happen is bug");

    let v = union.into_collection::<Vec<_>>();
    v.into_iter()
        .map(|a| {
            if a.start() == a.end() {
                SomeElementType::Individual(*a.start())
            } else {
                SomeElementType::Range(a)
            }
        })
        .collect()
}

#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn selection_contains_item() {
        assert!(Selection::All.contains_item(6543268));
        assert!(!Selection::None.contains_item(385188));

        assert!(Selection::Some(vec![SomeElementType::Individual(1)]).contains_item(1));
        assert!(Selection::Some(vec![SomeElementType::Range(1..=1)]).contains_item(1));
        assert!(Selection::Some(vec![SomeElementType::Range(1..=3)]).contains_item(1));
        assert!(Selection::Some(vec![SomeElementType::Range(1..=3)]).contains_item(3));

        let selection = Selection::Some(vec![
            SomeElementType::Individual(2),
            SomeElementType::Individual(6),
            SomeElementType::Range(4..=8),
        ]);
        assert!(selection.contains_item(2));
        assert!(selection.contains_item(6));
        assert!(selection.contains_item(5));
        assert!(selection.contains_item(8));
        assert!(selection.contains_item(4));
        assert!(!selection.contains_item(3));
        assert!(!selection.contains_item(9));
        assert!(!selection.contains_item(1));
        assert!(!selection.contains_item(3));

    }

    #[test]
    fn condense_ranges() {
        let c = condense_selections(vec![
            SomeElementType::Individual(1),
            SomeElementType::Individual(3),
            SomeElementType::Range(5..=9),
            SomeElementType::Individual(8),
            SomeElementType::Individual(10),
        ]);

        assert_eq!(
            c,
            vec![
                SomeElementType::Individual(1),
                SomeElementType::Individual(3),
                SomeElementType::Range(5..=10),
            ]
        );
    }

    #[test]
    fn condense_ranges_more_complex() {
        let c = condense_selections(vec![
            SomeElementType::Individual(1),
            SomeElementType::Individual(3),
            SomeElementType::Range(5..=9),
            SomeElementType::Range(11..=20),
            SomeElementType::Individual(10),
        ]);

        assert_eq!(
            c,
            vec![
                SomeElementType::Individual(1),
                SomeElementType::Individual(3),
                SomeElementType::Range(5..=20),
            ]
        );
    }
}
