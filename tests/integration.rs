use choice_string::{Selection, SomeElementType};

#[test]
fn parse_good() {
    let out = choice_string::parse("1 3   5;;6-15   20").expect("parses");
    assert_eq!(
        out,
        Selection::Some(vec![
            SomeElementType::Individual(1),
            SomeElementType::Individual(3),
            SomeElementType::Range(5..=15),
            SomeElementType::Individual(20),
        ])
    );
}

#[test]
fn selection_parse_e2e() {
    let sel = choice_string::parse("1 2, 3,  5,, ,11-99").expect("parses");
    assert!(sel.contains_item(1));
    assert!(sel.contains_item(2));
    assert!(sel.contains_item(3));
    assert!(sel.contains_item(5));
    for i in 11..=99usize {
        assert!(sel.contains_item(i));
    }

    assert!(!sel.contains_item(4));
    for i in 6..=10usize {
        assert!(!sel.contains_item(i));
    }
}
