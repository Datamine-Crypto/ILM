use ilm_data::controls::babi::babi_exact;
use ilm_data::controls::chain::chain_changes;
use ilm_data::controls::deduction::deduction_changes;
use ilm_data::controls::held::held_changes;
use ilm_data::controls::loops::loop_changes;
use ilm_data::controls::things::thing_changes;
use ilm_core::cases::Case;

fn told(sentences: &[&str]) -> Vec<String> {
    sentences.iter().map(|sentence| sentence.to_string()).collect()
}

fn case(sentences: &[&str], question: &str, answer: &str) -> Case {
    Case { sentences: told(sentences), question: question.to_string(), answer: answer.to_string() }
}

#[test]
fn exact_readings_of_made_stories() {
    let moves = told(&["Mary moved to the bathroom.", "John went to the hallway.", "Mary picked up the apple there.", "Mary went to the garden."]);
    assert_eq!(babi_exact(1, &moves, "Where is Mary?"), Some(vec!["garden".to_string()]));
    assert_eq!(babi_exact(2, &moves, "Where is the apple?"), Some(vec!["garden".to_string()]));
    assert_eq!(babi_exact(3, &moves, "Where was the apple before the garden?"), Some(vec!["bathroom".to_string()]));
    assert_eq!(babi_exact(8, &moves, "What is Mary carrying?"), Some(vec!["apple".to_string()]));
    assert_eq!(babi_exact(8, &moves, "What is John carrying?"), Some(vec!["nothing".to_string()]));
    let ways = told(&["The kitchen is north of the hallway.", "The garden is east of the kitchen."]);
    assert_eq!(babi_exact(19, &ways, "How do you go from the hallway to the garden?"), Some(vec!["north".to_string(), "east".to_string()]));
    let sizes = told(&["The box fits inside the chest.", "The chest is bigger than the ball."]);
    assert_eq!(babi_exact(18, &sizes, "Does the box fit in the chest?"), Some(vec!["yes".to_string()]));
    assert_eq!(babi_exact(18, &sizes, "Is the box bigger than the chest?"), Some(vec!["no".to_string()]));
    assert_eq!(babi_exact(18, &sizes, "Is the box bigger than the ball?"), None);
    assert_eq!(babi_exact(21, &sizes, "Where is Mary?"), None);
}

#[test]
fn needed_units_of_a_made_theory() {
    let lesson = case(&["The cow is big.", "If something is big then it is red."], "The cow is red.", "True");
    let labels: Vec<String> = ["U\t0\t1", "F\tcow\tis\tbig\tyes", "U\t1\t2", "C\t?\tis\tbig\tyes", "H\t?\tis\tred\tyes", "T\tcow\tis\tred\tyes"].iter().map(|line| line.to_string()).collect();
    let changes = deduction_changes(std::slice::from_ref(&lesson), &[labels], true);
    let found: Vec<(&str, &str)> = changes.iter().map(|change| (change.sort.as_str(), change.truth.as_str())).collect();
    assert_eq!(found, [("a needed told fact taken out", "False"), ("a needed told rule taken out", "False")]);
    assert_eq!(changes[0].sentences, told(&["If something is big then it is red."]));
}

#[test]
fn a_chain_of_made_classes() {
    let first = case(&["Every gorpus is a lempus.", "Ann is a gorpus."], "Is Ann a lempus or rompus?", "Ann is a lempus.");
    let second = case(&["Every rompus is a vumpus.", "Bob is a rompus."], "Is Bob a vumpus or lempus?", "Bob is a vumpus.");
    let lesson = case(&["Every wumpus is a yumpus.", "Tom is a wumpus.", "Every zumpus is a dumpus."], "Is Tom a yumpus or dumpus?", "Tom is a yumpus.");
    let lessons = vec![first, second, lesson];
    let changes = chain_changes(&lessons, "src_prosqa");
    let found: Vec<(Option<usize>, &str, usize)> = changes.iter().map(|change| (change.place, change.sort.as_str(), change.sentences.len())).collect();
    assert_eq!(found, [(Some(2), "every link of the first thing taken out", 2), (Some(2), "a needed sentence taken out", 2), (Some(2), "a needed sentence taken out", 2), (Some(2), "a sentence that is not on the way taken out", 2)]);
    assert_eq!(changes[3].truth, "Tom is a yumpus.");
}

#[test]
fn changed_texts_of_made_tables() {
    let lessons = vec![case(&[], "00 ta .", "00 01"), case(&[], "01 ta .", "01 00"), case(&[], "00 tb .", "00 00"), case(&[], "01 tb .", "01 01"), case(&[], "00 ta tb .", "00 01 01")];
    let changes = held_changes(&lessons, "src_lookup", "lookup_s1", None).unwrap();
    let found: Vec<(Option<usize>, &str, &str, &str)> = changes.iter().map(|change| (change.place, change.sort.as_str(), change.question.as_str(), change.truth.as_str())).collect();
    assert_eq!(found, [(Some(4), "row", "01 ta tb .", "01 00 00"), (Some(4), "table", "00 tb tb .", "00 00 00"), (Some(4), "table", "00 ta ta .", "00 01 00"), (Some(4), "sides", "00 tb ta .", "00 00 01")]);
    assert!(held_changes(&lessons, "src_babi", "babi01", None).is_err());
}

#[test]
fn things_that_change_names() {
    let lessons: Vec<Case> = (0..3)
        .map(|_| case(&["The Cup stands left of the pot.", "The pot stands left of the red jar."], "pick the cup stands last pick the pot stands last pick the red jar stands last", "the red jar stands last"))
        .collect();
    let changes = thing_changes(&lessons, true);
    assert_eq!(changes.len(), 9);
    assert_eq!(changes[0].sentences, ["The Red jar stands left of the pot.", "The pot stands left of the cup."]);
    assert_eq!(changes[0].truth, "the cup stands last");
    assert_eq!(changes[1].sentences, ["The Cup stands left of the red jar.", "The red jar stands left of the pot."]);
    assert_eq!(changes[1].truth, "the pot stands last");
    assert_eq!(changes[2].sentences, ["The Pot stands left of the cup.", "The cup stands left of the red jar."]);
    assert_eq!(changes[2].truth, "the red jar stands last");
    let unread = vec![case(&["The cup stands left of the pot."], "Who stands last?", "the pot")];
    assert!(thing_changes(&unread, true).is_empty());
}

#[test]
fn graphs_that_gain_or_lose_a_loop() {
    let looped = case(&["Edges:", "(0,1)", "(1,2)", "(2,0)", "(2,3)"], "Is there a cycle?", "Yes, a cycle.");
    let open = case(&["Edges:", "(0,1)", "(1,2)", "(3,4)"], "Is there a cycle?", "No cycle.");
    let changes = loop_changes(&[looped, open], true);
    let sorts: Vec<(&str, &str)> = changes.iter().map(|change| (change.sort.as_str(), change.truth.as_str())).collect();
    assert_eq!(sorts, [("an edge of the only loops taken out", "no cycle"), ("an edge taken out and a loop left", "yes a cycle"), ("an edge added between two joined nodes", "yes a cycle"), ("an edge added between two nodes that no chain joins", "no cycle")]);
    assert_eq!(changes[0].sentences, ["Edges:", "(1,2)", "(2,0)", "(2,3)"]);
    assert_eq!(changes[2].sentences, ["Edges:", "(0,1)", "(1,2)", "(3,4)", "(0,2)"]);
    assert_eq!(changes[3].sentences, ["Edges:", "(0,1)", "(1,2)", "(3,4)", "(0,3)"]);
}
