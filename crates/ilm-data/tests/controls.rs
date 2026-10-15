use ilm_data::controls::expression::{number_value, truth_value};
use ilm_data::controls::text::{capital_names, list_pieces, number_runs, with_numbers};
use ilm_data::sets::readers::common::in_one_case;

#[test]
fn pieces_of_a_list() {
    assert_eq!(list_pieces("Wren keeps a tin kettle, Odo keeps a clay jug, and Pim keeps a wool hat."), ["Wren keeps a tin kettle", "Odo keeps a clay jug", "Pim keeps a wool hat."]);
    assert_eq!(list_pieces(" a,b , and  c and d,andrew"), [" a", "b ", "c", "d", "andrew"]);
    assert_eq!(list_pieces("x and  y, and"), ["x", "y", "and"]);
}

#[test]
fn names_and_numbers() {
    assert_eq!(capital_names("Alice, Bob, and McDonald are friends; O'Neil A Ab"), ["Alice", "Bob", "Mc", "Donald", "Neil", "Ab"]);
    assert_eq!(capital_names("\u{c9}coleXy Zz"), ["Xy", "Zz"]);
    assert!(in_one_case("has", false) && !in_one_case("Has", false) && !in_one_case("42", false));
    let text = "What is 12 - -7? Choices: 5, -19, 19";
    let runs = number_runs(text, true);
    let found: Vec<&str> = runs.iter().map(|(start, end)| &text[*start..*end]).collect();
    assert_eq!(found, ["12", "-7", "5", "-19", "19"]);
    let unsigned: Vec<&str> = number_runs(text, false).iter().map(|(start, end)| &text[*start..*end]).collect();
    assert_eq!(unsigned, ["12", "7", "5", "19", "19"]);
    let values: Vec<String> = ["1", "2"].iter().map(|value| value.to_string()).collect();
    assert_eq!(with_numbers(text, &runs[..1], &values), "What is 1 - -7? Choices: 5, -19, 19");
    assert_eq!(with_numbers(text, &runs[..2], &values), "What is 1 - 2? Choices: 5, -19, 19");
}

fn logic_words(code: &str) -> Vec<&'static str> {
    let spelled = [("T", "True"), ("F", "False"), ("N", "not"), ("A", "and"), ("O", "or"), ("(", "("), (")", ")")];
    code.split(' ').map(|letter| spelled.iter().find(|(short, _)| *short == letter).map_or("?", |(_, word)| *word)).collect()
}

#[test]
fn values_of_expressions() {
    let cases = [("2 ** 3 ** 2", Some("512")), ("-2 ** 2", Some("-4")), ("((-1 + 2) * (3 - 4))", Some("-1")), ("05 + 1", None), ("00 + 1", Some("1")), ("(1)(2)", None), ("1 * * 2", None), ("+-+3", Some("-3")), ("(\n1 +\n2)", Some("3")), ("1 +\n2", None), ("7 - - 7 * 2", Some("21"))];
    for (text, value) in cases {
        assert_eq!(number_value(text).as_deref(), value, "{text}");
    }
    let truths = [("N ( T ) A ( F )", Some("False")), ("T O N F A F", Some("True")), ("( )", Some("False")), ("N ( )", Some("True")), ("T ( )", None), ("N N F", Some("False"))];
    for (code, value) in truths {
        let words = logic_words(code);
        assert_eq!(truth_value(&words).as_deref(), value, "{code}");
    }
}
