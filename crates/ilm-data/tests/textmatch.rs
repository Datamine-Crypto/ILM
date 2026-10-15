use ilm_data::sets::textmatch::TextPattern;

fn pattern(text: &str) -> TextPattern {
    TextPattern::new(text).expect("a pattern that the reader takes")
}

fn texts(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| item.to_string()).collect()
}

#[test]
fn a_text_is_cut_after_the_ends_of_sentences_as_python_cuts_it() {
    assert_eq!(pattern(r"(?<=[.!?])\s+").split("One. Two!  Three? Four"), texts(&["One.", "Two!", "Three?", "Four"]));
    let closed = pattern(r#"(?:(?<=[.?!])|(?<=[.?!]["')]))\s+"#);
    assert_eq!(closed.split("He said \"Go.\" Then left. End"), texts(&["He said \"Go.\"", "Then left.", "End"]));
}

#[test]
fn groups_lazy_and_greedy_repeats_and_alternatives_match_as_in_python() {
    let moved = pattern(r"(\w+) moved the ([\w ]+?) to the ([\w ]+)\.$").find("Ann moved the red ball to the green box.", false).expect("a match");
    assert_eq!(moved, vec![Some("Ann moved the red ball to the green box.".to_string()), Some("Ann".to_string()), Some("red ball".to_string()), Some("green box".to_string())]);
    assert_eq!(pattern(r"a(.*?)b").find("a1b2b", false).expect("a match")[1], Some("1".to_string()));
    assert_eq!(pattern(r"a(.*)b").find("a1b2b", false).expect("a match")[1], Some("1b2".to_string()));
    assert_eq!(pattern(r"[A-Za-z]+|[.?!]").find_all("Tom has 3 apples. Does Ann?"), vec![vec!["Tom"], vec!["has"], vec!["apples"], vec!["."], vec!["Does"], vec!["Ann"], vec!["?"]]);
    assert_eq!(pattern(r"(a)|(b)").find_all("ab"), vec![vec!["a", ""], vec!["", "b"]]);
}

#[test]
fn a_whole_match_the_end_and_the_flag_of_line_breaks_are_those_of_python() {
    let fact = pattern(r#"\("([^"]+)" "([^"]+)" "([^"]+)" "([^"]+)"\)"#).find(r#"("cow" "is" "big" "+")"#, true).expect("a whole match");
    assert_eq!(fact[1..].iter().flatten().cloned().collect::<Vec<String>>(), texts(&["cow", "is", "big", "+"]));
    assert!(pattern(r"b$").find("ab\n", false).is_some());
    assert!(pattern(r"b$").find("ab\n\n", false).is_none());
    let inside = pattern(r"(?s)\s*(\w+)\s*\((.*)\)\s*$").find("inform(name[A],\n x[B])", true).expect("a whole match");
    assert_eq!(inside[1..], [Some("inform".to_string()), Some("name[A],\n x[B]".to_string())]);
    assert_eq!(pattern(r"^\s*[QA]:\s*").replace_all("A: Q: x", ""), "Q: x");
}

#[test]
fn empty_matches_and_the_signs_of_unicode_are_those_of_python() {
    assert_eq!(pattern(r"x*").find_all("axb"), vec![vec![""], vec!["x"], vec![""], vec![""]]);
    assert_eq!(pattern(r"x*").replace_all("abxd", "-"), "-a-b--d-");
    assert_eq!(pattern(r"\s*\([^()]*\)").replace_all("12 (apples)", ""), "12");
    assert_eq!(pattern(r"\w+").find_all("caf\u{e9} na\u{ef}ve"), vec![vec!["caf\u{e9}"], vec!["na\u{ef}ve"]]);
    assert_eq!(pattern(r"\d+").find_all("\u{663}\u{664} 12"), vec![vec!["\u{663}\u{664}"], vec!["12"]]);
}

#[test]
fn a_counted_repeat_is_refused() {
    assert!(TextPattern::new("a{2}").is_err());
}
