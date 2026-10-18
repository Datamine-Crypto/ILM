use ilm_core::cases::Case;
use ilm_core::tools::call::{AnswerMark, Learner, Lessons};
use ilm_core::tools::explain::learn::Library;
use ilm_core::tools::explain::text::read_topic;
use ilm_core::tools::explain::chain::{keeps_history, knowledge_base};
use ilm_core::tools::explain::discovery::discovery_report;
use ilm_core::tools::explain::learn::model_of;
use ilm_core::tools::explain::wording::{learn_rules, learn_told, worded_model};
use ilm_core::tools::explain::wording_words::word_topics;
use ilm_core::tools::explain::text::LogicPiece;
use ilm_core::tools::explain::words::{word_number, word_text};
use std::sync::Arc;

const NAMES: &[&str] = &["Wren", "Tovin", "Ilse", "Marek", "Odile", "Pavo", "Quill", "Rasmo", "Selka", "Tamsin", "Ulmo", "Vesna"];

fn case(first: &str, second: &str, first_year: u32, second_year: u32, asked_before: bool) -> Case {
    let truth = if asked_before { first_year < second_year } else { first_year > second_year };
    let question = if asked_before { format!("{first} was born before {second}.") } else { format!("{first} was born after {second}.") };
    Case {
        sentences: vec![format!("{first} was born in {first_year}."), format!("{second} was born in {second_year}.")],
        question,
        answer: if truth { "yes" } else { "no" }.to_string(),
    }
}

fn lessons() -> Vec<Case> {
    let mut out = Vec::new();
    for at in 0..90usize {
        let first = NAMES[at % NAMES.len()];
        let second = NAMES[(at * 5 + 3) % NAMES.len()];
        if first == second {
            continue;
        }
        let first_year = 1900 + ((at * 37) % 97) as u32;
        let second_year = 1900 + ((at * 53 + 11) % 97) as u32;
        if first_year == second_year {
            continue;
        }
        out.push(case(first, second, first_year, second_year, at % 2 == 0));
    }
    out
}

fn births_library() -> Library {
    let text = "If A was born in C and B was born in D and C is less than D then A was born before B.";
    Library::of(vec![read_topic("births", text).expect("the text reads")]).expect("the library builds")
}

#[test]
fn question_forms_are_learned_for_a_text_of_rules() {
    let cases = lessons();
    let library = births_library();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let earlier = case("Aldous", "Brisa", 1950, 1960, true);
    assert_eq!(model.answer(&earlier.sentences, &earlier.question).words, vec!["yes".to_string()]);
    let later = case("Aldous", "Brisa", 1950, 1960, false);
    assert_eq!(model.answer(&later.sentences, &later.question).words, vec!["no".to_string()]);
}

#[test]
fn no_wording_is_learned_when_no_rule_reads_the_stories() {
    let mut cases = lessons();
    for lesson in &mut cases {
        lesson.sentences = vec!["The kettle sings on the stove.".to_string()];
    }
    assert!(worded_model(&births_library(), &Lessons::of(&cases)).is_none());
}

const PLACES: &[&str] = &["kitchen", "garden", "hallway", "office", "cellar", "attic"];

const KINDS: &[&str] = &["swan", "frog", "lion", "rhino", "mouse"];

const COLOURS: &[&str] = &["white", "green", "yellow", "gray"];

fn moves(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let first = PLACES[at % PLACES.len()];
    let last = PLACES[(at * 5 + 1) % PLACES.len()];
    Case { sentences: vec![format!("{name} went to the {first}."), format!("{name} went to the {last}.")], question: format!("Where is {name}?"), answer: last.to_string() }
}

#[test]
fn an_answer_that_is_the_value_of_the_latest_fact_is_learned() {
    let cases: Vec<Case> = (0..90).map(moves).collect();
    let library = Library::of(vec![read_topic("places", "If A went to the B then the place of A is B.").expect("the text reads")]).expect("the library builds");
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let sentences = vec!["Aldous went to the garden.".to_string(), "Aldous went to the cellar.".to_string()];
    assert_eq!(model.answer(&sentences, "Where is Aldous?").words, vec!["cellar".to_string()]);
}

#[test]
fn the_report_of_a_discovery_names_the_rule_found_and_counts_an_island_as_not_discovered() {
    let cases: Vec<Case> = (0..90).map(moves).collect();
    let library = Library::of(vec![read_topic("places", "If A walked to the B then A went to the B.").expect("the text reads")]).expect("the library builds");
    let kinds = vec![("moves".to_string(), cases[..60].iter().collect::<Vec<&Case>>(), cases[60..].iter().collect::<Vec<&Case>>())];
    let lines = discovery_report(&library, "places", &kinds, &[("places".to_string(), "If A went to the B then the place of A is B.".to_string())], (3, &[])).expect("the report is made");
    assert!(lines.iter().any(|line| line.trim() == "If A went to the B then Where is A B."));
    assert!(lines.iter().any(|line| line.starts_with("If A went to the B then the place of A is B. | not discovered | found only as an island, If A went to the B then Where is A B.")));
}

fn kinds(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 5) % NAMES.len()];
    let kind = KINDS[at % KINDS.len()];
    let colour = COLOURS[(at * 3 + 1) % COLOURS.len()];
    Case { sentences: vec![format!("{other} is a {kind}."), format!("{other} is {colour}."), format!("{name} is a {kind}.")], question: format!("What color is {name}?"), answer: colour.to_string() }
}

#[test]
fn an_answer_reached_through_a_second_fact_is_learned() {
    let cases: Vec<Case> = (0..90).map(kinds).collect();
    let text = ["White is a colour.", "Green is a colour.", "Yellow is a colour.", "Gray is a colour.", "If D is a colour and C is D and C is a B then the colour of the kind B is D."].join("\n");
    let library = Library::of(vec![read_topic("kinds", &text).expect("the text reads")]).expect("the library builds");
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let sentences = vec!["Brisa is a lion.".to_string(), "Brisa is gray.".to_string(), "Aldous is a lion.".to_string()];
    assert_eq!(model.answer(&sentences, "What color is Aldous?").words, vec!["gray".to_string()]);
}

#[test]
fn a_learned_model_lists_its_learned_lines_and_the_rules_that_a_proof_rests_on() {
    let cases: Vec<Case> = (0..90).map(kinds).collect();
    let text = ["White is a colour.", "Green is a colour.", "Yellow is a colour.", "Gray is a colour.", "If D is a colour and C is D and C is a B then the colour of the kind B is D."].join("
");
    let library = Library::of(vec![read_topic("kinds", &text).expect("the text reads")]).expect("the library builds");
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert!(model.lines_from_lessons().iter().any(|line| line.starts_with("To answer")));
    let sentences = vec!["Brisa is a lion.".to_string(), "Brisa is gray.".to_string(), "Aldous is a lion.".to_string()];
    assert!(model.rules_used(&sentences, "What color is Aldous?").iter().any(|(rule, _)| rule.contains("the colour of the kind")));
    assert!(model.proof(&sentences, "What color is Aldous?").iter().any(|line| line.contains("brisa is gray")));
}

fn holding_case(name: &str, thing: &str, other: &str, other_thing: &str) -> Case {
    Case { sentences: vec![format!("{name} got {thing}."), format!("{other} got {other_thing}.")], question: format!("What does {name} hold?"), answer: thing.to_string() }
}

#[test]
fn a_learned_answer_may_be_the_words_that_a_fact_fills() {
    let things = ["kite", "drum", "lamp", "rope", "flute", "bell", "kettle"];
    let cases: Vec<Case> = (0..60usize)
        .filter_map(|at| {
            let first = NAMES[at % NAMES.len()];
            let second = NAMES[(at * 5 + 3) % NAMES.len()];
            (first != second).then(|| holding_case(first, things[at % things.len()], second, things[(at + 3) % things.len()]))
        })
        .collect();
    let library = Library::of(vec![read_topic("holding", "If A got C then A holds C.").expect("the text reads")]).expect("the library builds");
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let new = holding_case("Aldous", "harp", "Brisa", "sled");
    assert_eq!(model.answer(&new.sentences, &new.question).words, vec!["harp".to_string()]);
}

fn pairs_case(at: usize) -> Case {
    let pairs = 1 + at % 5;
    let odd = at % 3 == 0;
    let mut signs: Vec<&str> = std::iter::repeat_n(["x", "y"], pairs).flatten().collect();
    if odd {
        signs.insert(at % signs.len(), "y");
    }
    Case { sentences: vec![format!("Seq: {}", signs.join(" "))], question: "Even?".to_string(), answer: if odd { "no" } else { "yes" }.to_string() }
}

#[test]
fn a_question_with_no_place_learns_from_the_told_sentences_that_a_text_writes_again() {
    let text = "Write AAAA x y BBBB as AAAA BBBB.\nWrite AAAA x y as AAAA.";
    let library = Library::of(vec![read_topic("pairs", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(pairs_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&["Seq: x y x x y y x y".to_string()], "Even?").words, vec!["yes".to_string()]);
    assert_eq!(model.answer(&["Seq: x y y x".to_string()], "Even?").words, vec!["no".to_string()]);
}

const SWAPS: &str = "If R And S swap and before that time S holds CC then R holds CC.\nIf R And S swap and before that time R holds BB then S holds BB.\nIf R holds BB and latest R holds CC then R ends with CC.";

fn swap_story(names: [&str; 3], toys: [&str; 3], swaps: [(usize, usize); 2], asked: usize) -> Case {
    let mut held = toys;
    for (one, other) in swaps {
        held.swap(one, other);
    }
    let sentences = vec![
        format!("{}, {}, and {} are friends who trade toys.", names[0], names[1], names[2]),
        format!("At the start of the day, they each have a toy: {} gets a {}, {} gets a {}, and {} gets a {}.", names[0], toys[0], names[1], toys[1], names[2], toys[2]),
        format!("First, {} and {} swap toys.", names[swaps[0].0], names[swaps[0].1]),
        format!("Then, {} and {} swap toys.", names[swaps[1].0], names[swaps[1].1]),
    ];
    Case { sentences, question: format!("At the end of the day, {} has", names[asked]), answer: format!("a {}.", held[asked]) }
}

#[test]
fn a_told_list_and_its_steps_are_read_by_learned_rules() {
    let toys = ["kite", "drum", "lamp", "rope", "flute", "bell", "harp"];
    let pairs = [(0usize, 1usize), (1, 2), (0, 2), (2, 1), (1, 0), (2, 0)];
    let cases: Vec<Case> = (0..90usize)
        .map(|at| {
            let names = [NAMES[at % NAMES.len()], NAMES[(at + 4) % NAMES.len()], NAMES[(at + 8) % NAMES.len()]];
            let story_toys = [toys[at % toys.len()], toys[(at + 2) % toys.len()], toys[(at + 5) % toys.len()]];
            swap_story(names, story_toys, [pairs[at % pairs.len()], pairs[(at / 6) % pairs.len()]], at % 3)
        })
        .collect();
    let topic = read_topic("swaps", SWAPS).expect("the text reads");
    let base = Arc::new(knowledge_base(&[], &topic.rules, true));
    let fit: Vec<&Case> = cases.iter().collect();
    let lines = learn_told(&topic, &base, &fit);
    let text = format!("{SWAPS}\n{}\nTo answer \"At the end of the day , A has\": BB if A ends with BB.", lines.join("\n"));
    let model = model_of(vec![read_topic("swaps learned", &text).expect("the learned text reads")]);
    let new = swap_story(["Aldous", "Brisa", "Corin"], ["sled", "kite", "drum"], [(0, 1), (1, 2)], 2);
    assert_eq!(new.answer, "a sled.");
    assert_eq!(model.answer(&new.sentences, &new.question).words, vec!["a".to_string(), "sled".to_string()]);
}

fn book_story(names: &[&str], books: &[&str], swaps: &[(usize, usize)], asked: usize) -> Case {
    let mut held: Vec<&str> = books.to_vec();
    for (one, other) in swaps {
        held.swap(*one, *other);
    }
    let items: Vec<String> = names.iter().zip(books).map(|(name, book)| format!("{name} gets {book}")).collect();
    let (last, first) = items.split_last().expect("a list has items");
    let mut sentences = vec![
        format!("{} and {} are readers who trade books.", names[..names.len() - 1].join(", "), names[names.len() - 1]),
        format!("At the start of the term, they each buy one book: {}, and {last}.", first.join(", ")),
    ];
    for (step, (one, other)) in swaps.iter().enumerate() {
        sentences.push(format!("{}, {} and {} swap books.", if step == 0 { "First" } else { "Then" }, names[*one], names[*other]));
    }
    Case { sentences, question: format!("At the end of the term, {} has", names[asked]), answer: format!("{}.", held[asked]) }
}

#[test]
fn a_list_learned_from_three_items_reads_a_longer_list_and_a_title_with_a_hyphen() {
    let books = ["Ulysses", "Dracula", "Emma", "Beowulf", "Catch-22", "Ivanhoe", "Walden"];
    let pairs = [(0usize, 1usize), (1, 2), (0, 2), (2, 1), (1, 0), (2, 0)];
    let cases: Vec<Case> = (0..90usize)
        .map(|at| {
            let names = [NAMES[at % NAMES.len()], NAMES[(at + 4) % NAMES.len()], NAMES[(at + 8) % NAMES.len()]];
            let story_books = [books[at % books.len()], books[(at + 2) % books.len()], books[(at + 5) % books.len()]];
            book_story(&names, &story_books, &[pairs[at % pairs.len()], pairs[(at / 6) % pairs.len()]], at % 3)
        })
        .collect();
    let topic = read_topic("swaps", SWAPS).expect("the text reads");
    let base = Arc::new(knowledge_base(&[], &topic.rules, true));
    let fit: Vec<&Case> = cases.iter().collect();
    let lines = learn_told(&topic, &base, &fit);
    let text = format!("{SWAPS}\n{}\nTo answer \"At the end of the term , A has\": BB if A ends with BB.", lines.join("\n"));
    let model = model_of(vec![read_topic("swaps learned", &text).expect("the learned text reads")]);
    let new = book_story(&["Aldous", "Brisa", "Corin", "Dovan", "Elske"], &["Ulysses", "Catch-22", "Emma", "Walden", "Dracula"], &[(1, 4), (4, 0)], 0);
    assert_eq!(new.answer, "Catch-22.");
    assert_eq!(model.answer(&new.sentences, &new.question).words, vec!["catch".to_string(), "-".to_string(), "22".to_string()]);
}

fn told_pairs_case(at: usize) -> Case {
    let pairs = 1 + at % 4;
    let odd = at % 3 == 0;
    let mut digits: Vec<String> = (0..pairs).flat_map(|pair| { let digit = (at + pair * 3) % 9 + 1; [digit.to_string(), digit.to_string()] }).collect();
    if odd {
        digits.insert(at % digits.len(), "0".to_string());
    }
    Case { sentences: Vec::new(), question: format!("Pairs: {}", digits.join(" ")), answer: if odd { "no" } else { "yes" }.to_string() }
}

#[test]
fn a_question_that_holds_the_data_is_told_and_written_again_by_the_text() {
    let text = "Write AAAA RR RR BBBB as AAAA BBBB.\nWrite AAAA RR RR as AAAA.";
    let library = Library::of(vec![read_topic("equal pairs", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(told_pairs_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&[], "Pairs: 7 7 2 2 4 4").words, vec!["yes".to_string()]);
    assert_eq!(model.answer(&[], "Pairs: 7 7 5 2 2").words, vec!["no".to_string()]);
}

fn sum_case(at: usize) -> Case {
    let terms: Vec<usize> = (0..2 + at % 5).map(|step| (at * 7 + step * 3) % 9 + 1).collect();
    let total: usize = terms.iter().sum();
    Case { sentences: Vec::new(), question: format!("{} =", terms.iter().map(usize::to_string).collect::<Vec<String>>().join(" + ")), answer: total.to_string() }
}

#[test]
fn a_question_whose_data_changes_in_one_long_stretch_gets_a_run_form() {
    let text = "Write AA R + S BB as AA T BB if R plus S is T.";
    let library = Library::of(vec![read_topic("sums in a row", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(sum_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&[], "4 + 5 + 6 + 7 + 8 + 9 + 1 =").words, vec!["40".to_string()]);
}

fn doubled_case(at: usize) -> Case {
    let value = (at * 7) % 40 + 1;
    Case { sentences: vec![format!("Value: {value}")], question: "Doubled:".to_string(), answer: (value * 2).to_string() }
}

#[test]
fn a_question_with_no_place_learns_an_answer_that_the_rules_work_out() {
    let text = "If Value : N and N plus N is M then the doubled value is M.";
    let library = Library::of(vec![read_topic("doubles", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(doubled_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&["Value: 21".to_string()], "Doubled:").words, vec!["42".to_string()]);
}

const BRACKETS: &[(&str, &str)] = &[("(", ")"), ("[", "]"), ("{", "}"), ("<", ">")];

fn brackets_case(at: usize) -> Case {
    let opened: Vec<usize> = (0..1 + at % 4).map(|step| (at * 3 + step * 7 + at / 4) % BRACKETS.len()).collect();
    let input = opened.iter().map(|kind| BRACKETS[*kind].0).collect::<Vec<&str>>().join(" ");
    let output = opened.iter().rev().map(|kind| BRACKETS[*kind].1).collect::<Vec<&str>>().join(" ");
    Case { sentences: vec![format!("Input: {input}")], question: "Output:".to_string(), answer: output }
}

#[test]
fn an_answer_of_marks_that_a_fact_holds_fills_a_place_of_any_signs() {
    let text = "( is closed by ).
[ is closed by ].
{ is closed by }.
< is closed by >.
If Input : AAAA then AAAA stays open.
If AAAA RR stays open then AAAA stays open.
If RR stays open and RR is closed by SS then SS closes RR.
If AAAA RR stays open and RR is closed by SS and BBBB closes AAAA then SS BBBB closes AAAA RR.
If Input : AAAA and BBBB closes AAAA then the output is BBBB.";
    let library = Library::of(vec![read_topic("brackets", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(brackets_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&["Input: < ( [ {".to_string()], "Output:").words, ["}", "]", ")", ">"].iter().map(|sign| sign.to_string()).collect::<Vec<String>>());
}

fn kept_case(at: usize) -> Case {
    let name = format!("{}{}", ["Al", "Bri", "Cor", "Dan", "Eli", "Fen", "Gil", "Hal"][at % 8], ["dous", "sa", "win", "ra", "mo"][at / 8 % 5]);
    let thing = format!("{}{}", ["ka", "mo", "ti", "ru", "se", "lo", "pa", "ne", "vi", "du"][at % 10], ["rel", "bin", "tok", "sar", "mun"][at / 10 % 5]);
    let kept = at % 12 == 0;
    let asked = if kept { thing.clone() } else { format!("{thing}a") };
    Case { sentences: vec![format!("{name} got {thing}.")], question: format!("{name} holds {asked}."), answer: if kept { "yes" } else { "no" }.to_string() }
}

fn asked_case(at: usize) -> Case {
    let name = format!("{}{}", ["Al", "Bri", "Cor", "Dan", "Eli", "Fen", "Gil", "Hal"][at % 8], ["dous", "sa", "win", "ra", "mo"][at / 8 % 5]);
    let thing = format!("{}{}", ["ka", "mo", "ti", "ru", "se", "lo", "pa", "ne", "vi", "du"][at % 10], ["rel", "bin", "tok", "sar", "mun"][at / 10 % 5]);
    let quality = format!("{}{}", ["ve", "zo", "qui", "ha", "yu", "wo", "ja"][at % 7], ["rish", "lent", "mous", "tic"][at / 7 % 4]);
    let has = at % 3 != 0;
    let told = if has { format!("{name} is {quality}.") } else { format!("{name} is not {quality}.") };
    let sentences = vec![format!("{name} got {thing}."), told];
    if at % 2 == 0 {
        let kept = at % 4 == 0;
        let asked = if kept { thing } else { format!("{thing}a") };
        return Case { sentences, question: format!("Does {name} hold {asked}?"), answer: if kept { "yes" } else { "no" }.to_string() };
    }
    Case { sentences, question: format!("Is {name} truly {quality}?"), answer: if has { "yes" } else { "no" }.to_string() }
}

#[test]
fn a_told_fact_that_holds_the_words_of_the_question_is_a_condition_in_the_places_of_its_form() {
    let library = Library::of(vec![read_topic("holding", "If A got C then A holds C.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..120).map(asked_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&["Aldous got harp.".to_string(), "Aldous is not brisk.".to_string()], "Is Aldous truly brisk?").words, vec!["no".to_string()]);
    assert_eq!(model.answer(&["Aldous got harp.".to_string(), "Aldous is brisk.".to_string()], "Is Aldous truly brisk?").words, vec!["yes".to_string()]);
}

#[test]
fn a_condition_that_holds_on_every_lesson_is_no_reading_but_the_answer_otherwise() {
    let library = Library::of(vec![read_topic("holding", "If A got C then A holds C.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(kept_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&["Aldous got harp.".to_string()], "Aldous holds harp.").words, vec!["yes".to_string()]);
    let said = model.answer(&["Brisa got sled.".to_string()], "Aldous holds harp.");
    assert_eq!(said.words, vec!["no".to_string()]);
    assert_eq!(said.mark, AnswerMark::Default);
}

#[test]
fn a_story_with_a_sentence_of_a_frame_that_no_lesson_showed_is_answered_as_a_guess() {
    let library = Library::of(vec![read_topic("holding", "If A got C then A holds C.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(kept_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned").framed(&cases);
    let named = model.answer(&["Zeno got lute.".to_string()], "Zeno holds lute.");
    assert_eq!((named.words, named.mark), (vec!["yes".to_string()], AnswerMark::Proof));
    let unread = model.answer(&["Zeno got lute.".to_string(), "Then Zeno lost lute.".to_string()], "Zeno holds lute.");
    assert_eq!((unread.words, unread.mark), (vec!["yes".to_string()], AnswerMark::Guess));
}

const CARGO: &[&str] = &["kelp", "plank", "reed", "cork", "moss", "twine", "sail", "net", "oar", "rope", "tarp", "pail"];

fn raft_case(at: usize) -> Case {
    let load = |from: usize, count: usize| (0..count).map(|step| CARGO[(from + step * 5) % CARGO.len()]).collect::<Vec<&str>>();
    let mut first = load(at, 2 + at % 3);
    let mut second = load(at + 7, 2 + at / 3 % 3);
    let sinks_first = at % 2 == 0;
    let heavy = if sinks_first { &mut first } else { &mut second };
    heavy.insert(at % heavy.len(), "rock");
    Case { sentences: Vec::new(), question: format!("Which raft floats: p \" {} \" q \" {} \" ?", first.join(" "), second.join(" ")), answer: if sinks_first { "q" } else { "p" }.to_string() }
}

#[test]
fn a_question_that_changes_in_several_stretches_gets_a_run_place_for_each() {
    let text = "If AAA \" BB \" CCC and not BB holds the word \" then the load ( BB ) is told.\nIf AAA \" BB \" and not BB holds the word \" then the load ( BB ) is told.\nIf the load ( BB ) is told and BB holds the word rock then the load ( BB ) sinks.";
    let library = Library::of(vec![read_topic("rafts", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(raft_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    for (question, answer) in [("Which raft floats: p \" oar rock pail \" q \" cork twine \" ?", "q"), ("Which raft floats: p \" net sail \" q \" rock moss reed kelp \" ?", "p")] {
        let said = model.answer(&[], question);
        assert_eq!(said.words, vec![answer.to_string()]);
        assert_ne!(said.mark, AnswerMark::Default, "{}", said.note);
    }
}

const FURNITURE: &[&str] = &["pole", "stool", "table", "bench", "throne"];

fn feet_case(at: usize) -> Case {
    let name = format!("{}{}", ["Al", "Bri", "Cor", "Dan", "Eli", "Fen", "Gil", "Hal"][at % 8], ["dous", "sa", "win", "ra", "mo"][at / 8 % 5]);
    let feet = 1 + at % FURNITURE.len();
    Case { sentences: vec![format!("{name} stands on {feet} feet.")], question: format!("What is {name}?"), answer: FURNITURE[feet - 1].to_string() }
}

#[test]
fn an_answer_that_a_fact_of_the_story_holds_is_filled_from_the_form_of_that_fact() {
    let text = (0..FURNITURE.len()).map(|at| format!("If A stands on {} feet then A is a {}.", at + 1, FURNITURE[at])).collect::<Vec<String>>().join("\n");
    let library = Library::of(vec![read_topic("furniture", &text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(feet_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    for (feet, thing) in FURNITURE.iter().enumerate() {
        assert_eq!(model.answer(&[format!("Aldous stands on {} feet.", feet + 1)], "What is Aldous?").words, vec![thing.to_string()]);
    }
}

fn verb_case(at: usize) -> Case {
    let name = format!("{}{}", ["Al", "Bri", "Cor", "Dan", "Eli", "Fen", "Gil", "Hal"][at % 8], ["dous", "sa", "win", "ra", "mo"][at / 8 % 5]);
    let verb = format!("{}{}", ["bl", "sk", "tr", "gr", "pl", "fr", "kn"][at % 7], ["orp", "ee", "amp", "ult", "isk"][at / 7 % 5]);
    let thing = format!("{}{}", ["ka", "mo", "ti", "ru", "se", "lo", "pa", "ne", "vi", "du"][at % 10], ["rel", "bin", "tok", "sar", "mun"][at / 10 % 5]);
    let other = format!("{thing}a");
    let does = at % 3 != 0;
    let sentences = vec![format!("{name} got a {thing}."), format!("{name} {verb}s the {}.", if does { &thing } else { &other })];
    if at % 2 == 0 {
        let asked = if does { &thing } else { &other };
        return Case { sentences, question: format!("Does {name} hold the {asked}?"), answer: if *asked == thing { "yes" } else { "no" }.to_string() };
    }
    Case { sentences, question: format!("Does {name} ever {verb} the {thing}?"), answer: if does { "yes" } else { "no" }.to_string() }
}

#[test]
fn a_fact_that_writes_the_word_of_a_place_with_an_ending_names_that_place() {
    let library = Library::of(vec![read_topic("holding", "If A got a C then A holds C.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..120).map(verb_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&["Aldous got a harp.".to_string(), "Aldous zwinks the harp.".to_string()], "Does Aldous ever zwink the harp?").words, vec!["yes".to_string()]);
    assert_eq!(model.answer(&["Aldous got a harp.".to_string(), "Aldous zwinks the sled.".to_string()], "Does Aldous ever zwink the harp?").words, vec!["no".to_string()]);
}

fn always_case(at: usize) -> Case {
    let name = format!("{}{}", ["Al", "Bri", "Cor", "Dan", "Eli", "Fen", "Gil", "Hal"][at % 8], ["dous", "sa", "win", "ra", "mo"][at / 8 % 5]);
    let thing = format!("{}{}", ["ka", "mo", "ti", "ru", "se", "lo", "pa", "ne", "vi", "du"][at % 10], ["rel", "bin", "tok", "sar", "mun"][at / 10 % 5]);
    Case { sentences: vec![format!("{name} got {thing}.")], question: format!("{name} holds {thing}."), answer: "yes".to_string() }
}

#[test]
fn a_condition_that_names_only_the_places_of_the_question_is_a_reading_even_on_every_lesson() {
    let library = Library::of(vec![read_topic("holding", "If A got C then A holds C.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(always_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&["Aldous got harp.".to_string()], "Aldous holds harp.").words, vec!["yes".to_string()]);
    assert!(model.answer(&["Aldous got sled.".to_string()], "Aldous holds harp.").words.is_empty());
}

fn closed_case(at: usize) -> Case {
    let name = format!("{}{}", ["Al", "Bri", "Cor", "Dan", "Eli", "Fen", "Gil", "Hal"][at % 8], ["dous", "sa", "win", "ra", "mo"][at / 8 % 5]);
    let thing = format!("{}{}", ["ka", "mo", "ti", "ru", "se", "lo", "pa", "ne", "vi", "du"][at % 10], ["rel", "bin", "tok", "sar", "mun"][at / 10 % 5]);
    let (sentence, answer) = match at % 3 {
        0 => (format!("{name} got {thing}."), "yes"),
        1 => (format!("{name} sings."), "no"),
        _ => (format!("{name} got {thing}a."), "no"),
    };
    Case { sentences: vec![sentence], question: format!("Does {name} hold {thing}?"), answer: answer.to_string() }
}

#[test]
fn a_later_condition_of_the_label_otherwise_gives_way_to_the_answer_otherwise() {
    let library = Library::of(vec![read_topic("holding", "If A got C then A holds C.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(closed_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&["Aldous got harp.".to_string()], "Does Aldous hold harp?").words, vec!["yes".to_string()]);
    let said = model.answer(&["Aldous got sled.".to_string()], "Does Aldous hold harp?");
    assert_eq!(said.words, vec!["no".to_string()]);
    assert!(said.note.ends_with("no otherwise"), "{}", said.note);
}

const NUMBER_WORDS: &[&str] = &["one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen", "nineteen", "twenty", "twentyone", "twentytwo", "twentythree", "twentyfour", "twentyfive"];

fn coins_case(at: usize) -> Case {
    let name = format!("{}{}", ["Al", "Bri", "Cor", "Dan", "Eli", "Fen", "Gil", "Hal"][at % 8], ["dous", "sa", "win", "ra", "mo"][at / 8 % 5]);
    let count = 1 + at * 7 % NUMBER_WORDS.len();
    let asked = if at % 3 == 0 { 1 + (count + at / 3) % NUMBER_WORDS.len() } else { count };
    let asked_text = if at % 2 == 0 { NUMBER_WORDS[asked - 1].to_string() } else { asked.to_string() };
    Case { sentences: vec![format!("{name} has {count} coins.")], question: format!("{name} has exactly {asked_text} coins."), answer: if asked == count { "entailment" } else { "contradiction" }.to_string() }
}

#[test]
fn two_facts_joined_by_a_value_that_the_question_does_not_name_give_a_label() {
    let mut text: Vec<String> = NUMBER_WORDS.iter().enumerate().map(|(at, word)| format!("{word} is the number {}.", at + 1)).collect();
    text.push("If A has B coins and B is at least 0 then the coins of A are B.".to_string());
    text.push("If B is the number C and C is at least 0 then C is a count.".to_string());
    let library = Library::of(vec![read_topic("coins", &text.join("\n")).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..150).map(coins_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&["Aldous has 7 coins.".to_string()], "Aldous has exactly seven coins.").words, vec!["entailment".to_string()]);
    assert_eq!(model.answer(&["Aldous has 8 coins.".to_string()], "Aldous has exactly seven coins.").words, vec!["contradiction".to_string()]);
}

fn dropped_case(at: usize) -> Case {
    let name = format!("{}{}", ["Al", "Bri", "Cor", "Dan", "Eli", "Fen", "Gil", "Hal"][at % 8], ["dous", "sa", "win", "ra", "mo"][at / 8 % 5]);
    let rooms = ["kitchen", "hall", "yard", "cellar", "attic", "porch"];
    let thing = format!("{}{}", ["ka", "mo", "ti", "ru", "se", "lo", "pa", "ne", "vi", "du"][at % 10], ["rel", "bin", "tok", "sar", "mun"][at / 10 % 5]);
    let (first, kept, last) = (rooms[at % rooms.len()], rooms[(at + 1) % rooms.len()], rooms[(at + 2) % rooms.len()]);
    Case {
        sentences: vec![format!("{name} went to the {first}."), format!("{name} grabbed the {thing}."), format!("{name} went to the {kept}."), format!("{name} dropped the {thing}."), format!("{name} went to the {last}.")],
        question: format!("Where is the {thing}?"),
        answer: kept.to_string(),
    }
}

#[test]
fn a_second_fact_may_hold_before_the_time_of_the_first() {
    let text = "If A grabbed the B then A holds the B.\nIf A dropped the B then A lost the B.\nIf A went to the C then A is in the C.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(dropped_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let story: Vec<String> = ["Aldous went to the porch.", "Aldous grabbed the harp.", "Aldous went to the attic.", "Aldous dropped the harp.", "Aldous went to the hall."].iter().map(|sentence| sentence.to_string()).collect();
    assert_eq!(model.answer(&story, "Where is the harp?").words, vec!["attic".to_string()]);
}

const REQUESTS: &[(&str, &str)] = &[("play the song", "music"), ("what is the weather in", "weather"), ("book a table at", "restaurant")];
const TOWNS: &[&str] = &["paris", "rome", "oslo", "lima", "cairo", "quito", "seoul", "dakar", "riga", "bern"];

fn request_case(at: usize) -> Case {
    let (start, label) = REQUESTS[at % REQUESTS.len()];
    Case { sentences: vec![format!("{start} {}", TOWNS[at / REQUESTS.len() % TOWNS.len()])], question: "intent".to_string(), answer: label.to_string() }
}

#[test]
fn a_label_of_the_told_text_is_learned_as_a_list_over_the_words_that_a_told_sentence_holds() {
    let cases: Vec<Case> = (0..60).map(request_case).collect();
    let lessons: Vec<&Case> = cases.iter().collect();
    let topic = word_topics(&lessons, &[90]).into_iter().next().flatten().expect("the list over the words of the told text is learned");
    let model = model_of(vec![topic]);
    assert_eq!(model.answer(&["play the song madrid".to_string()], "intent").words, vec!["music".to_string()]);
    assert_eq!(model.answer(&["what is the weather in madrid".to_string()], "intent").words, vec!["weather".to_string()]);
    assert_eq!(model.answer(&["book a table at madrid".to_string()], "intent").words, vec!["restaurant".to_string()]);
}

fn sum_behind_minus_case(at: usize) -> Case {
    let (one, other) = (10 + at * 7 % 80, 3 + at * 5 % 60);
    Case { sentences: Vec::new(), question: format!("{one} - {other} ="), answer: (one + other).to_string() }
}

#[test]
fn an_answer_that_a_step_of_arithmetic_works_out_from_the_question_is_learned_as_a_worked_condition() {
    let text = "If The question is A - B = then A is a term.";
    let library = Library::of(vec![read_topic("terms", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(sum_behind_minus_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&[], "12 - 5 =").words, vec!["17".to_string()]);
    assert_eq!(model.answer(&[], "40 - 40 =").words, vec!["80".to_string()]);
}

const SWAPPED: &[&str] = &["red", "blue", "cat", "dog", "one", "two", "up", "down", "sun", "moon", "salt", "pepper"];

fn swap_case(at: usize) -> Case {
    let (first, second) = (SWAPPED[at % SWAPPED.len()], SWAPPED[(at * 5 + 7) % SWAPPED.len()]);
    Case { sentences: vec![format!("Swap: {first} - {second}")], question: "Result?".to_string(), answer: format!("{second} ! {first}") }
}

#[test]
fn an_answer_made_of_the_words_of_the_told_sentence_is_learned_as_a_pattern_of_its_places() {
    let library = Library::of(vec![read_topic("swaps", "If Swap : A - B then A is a word.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(swap_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&["Swap: tea - milk".to_string()], "Result?").words, vec!["milk".to_string(), "tea".to_string()]);
}

const PICKS: &[&str] = &["banana", "house", "kite", "lamp"];

fn sum_among_picks_case(at: usize) -> Case {
    let (one, other) = (3 + at * 7 % 50, 2 + at * 11 % 40);
    let picks: Vec<String> = [(one + other).to_string(), (one * other + 1).to_string(), PICKS[at % PICKS.len()].to_string(), (one + 3).to_string(), PICKS[(at + 1) % PICKS.len()].to_string()].into_iter().cycle().skip(at % 3).take(5).collect();
    Case { sentences: Vec::new(), question: format!("What is {one} plus {other}? {}", picks.iter().map(|pick| format!("pick: {pick}")).collect::<Vec<String>>().join(" ")), answer: (one + other).to_string() }
}

#[test]
fn a_question_with_too_many_runs_for_a_cut_gets_the_form_of_its_worked_examples() {
    let library = Library::of(vec![read_topic("asked", "If The question is AAAA then the question is known.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(sum_among_picks_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&[], "What is 20 plus 22? pick: 7 pick: kite pick: 42 pick: 9 pick: lamp").words, vec!["42".to_string()]);
}

fn one_more_case(at: usize) -> Case {
    let (one, other) = (2 + at * 3 % 9, 1 + at * 5 % 8);
    Case { sentences: Vec::new(), question: format!("{one} * {other} ->"), answer: (one * other + 1).to_string() }
}

#[test]
fn a_step_whose_result_falls_short_by_the_same_number_adds_that_number() {
    let library = Library::of(vec![read_topic("asked", "If The question is AAAA then the question is known.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(one_more_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    assert_eq!(model.answer(&[], "5 * 5 ->").words, vec!["26".to_string()]);
}

const ROOMS: &[&str] = &["kitchen", "hall", "yard", "cellar", "attic", "porch"];

fn carried_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 5) % NAMES.len()];
    let thing = format!("{}{}", ["ka", "mo", "ti", "ru", "se", "lo", "pa", "ne", "vi", "du"][at % 10], ["rel", "bin", "tok", "sar", "mun"][at / 10 % 5]);
    let room = |step: usize| ROOMS[(at + step) % ROOMS.len()];
    let (took, put) = (["took", "grabbed"][at % 2], ["put down", "left"][at / 2 % 2]);
    let mut sentences = vec![format!("{name} moved to the {}.", room(0)), format!("{name} {took} the {thing}."), format!("{other} moved to the {}.", room(3)), format!("{name} moved to the {}.", room(1))];
    let answer = match at % 4 {
        0 => room(1),
        1 => {
            sentences.extend([format!("{name} {put} the {thing}."), format!("{name} moved to the {}.", room(2)), format!("{name} moved to the {}.", room(4))]);
            room(1)
        }
        2 => {
            sentences.insert(2, format!("{name} {put} the {thing}."));
            room(0)
        }
        _ => {
            sentences.extend([format!("{name} {put} the {thing}."), format!("{other} moved to the {}.", room(1)), format!("{other} {took} the {thing}."), format!("{other} moved to the {}.", room(2))]);
            room(2)
        }
    };
    Case { sentences, question: format!("Where is the {thing}?"), answer: answer.to_string() }
}

#[test]
fn a_state_that_holds_from_one_event_until_another_is_invented_when_no_fact_of_the_text_names_it() {
    let text = "If A moved to the B then A went to the B.
If A took the B then A got the B.
If A grabbed the B then A got the B.
If A put down the B then A dropped the B.
If A left the B then A dropped the B.
Reply to \"Where is the A?\" with B: The A is in the B.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..150).map(carried_case).collect();
    let lines = learn_rules(topic, &base, &cases.iter().collect::<Vec<&Case>>());
    assert!(lines.iter().any(|line| line.ends_with(" is no more.")), "no state is invented: {lines:?}");
    let learned = read_topic("rooms", &format!("{text}
{}
Where is the A? B if latest The A is in the B.", lines.join("
"))).expect("the learned rules read");
    let model = model_of(vec![learned]);
    let story = |sentences: &[&str]| -> Vec<String> { sentences.iter().map(|sentence| sentence.to_string()).collect() };
    assert_eq!(model.answer(&story(&["Aldous moved to the porch.", "Aldous took the harp.", "Aldous moved to the attic."]), "Where is the harp?").words, vec!["attic".to_string()]);
    assert_eq!(model.answer(&story(&["Aldous moved to the porch.", "Aldous grabbed the harp.", "Aldous moved to the attic.", "Aldous left the harp.", "Aldous moved to the hall."]), "Where is the harp?").words, vec!["attic".to_string()]);
    let passed = story(&["Aldous moved to the porch.", "Aldous grabbed the harp.", "Aldous moved to the attic.", "Aldous left the harp.", "Brisa moved to the attic.", "Brisa took the harp.", "Brisa moved to the yard."]);
    assert_eq!(model.answer(&passed, "Where is the harp?").words, vec!["yard".to_string()]);
    let proof = model.proof(&passed, "Where is the harp?");
    assert!(proof.iter().any(|line| line.trim().starts_with("the got of the harp is brisa by the rule")), "{proof:?}");
    assert!(proof.iter().any(|line| line.trim() == "told: brisa took the harp"), "{proof:?}");
}

#[test]
fn a_learned_state_holds_again_when_its_event_comes_again() {
    let text = "If A moved to the B then A went to the B.
If A took the B then A got the B.
If A grabbed the B then A got the B.
If A put down the B then A dropped the B.
If A left the B then A dropped the B.
Reply to \"Where is the A?\" with B: The A is in the B.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..150).map(carried_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let story: Vec<String> = ["Aldous moved to the porch.", "Aldous took the harp.", "Aldous moved to the attic.", "Aldous left the harp.", "Aldous took the harp.", "Aldous moved to the yard."].iter().map(|sentence| sentence.to_string()).collect();
    assert_eq!(model.answer(&story, "Where is the harp?").words, vec!["yard".to_string()]);
}

#[test]
fn an_invented_state_is_about_a_thing_that_the_question_names() {
    let text = "If A moved to the B then A went to the B.
If A went to the C and before that time A went to the D then A went from D to C.
If A took the B then A got the B.
If A grabbed the B then A got the B.
If A put down the B then A dropped the B.
If A left the B then A dropped the B.
Reply to \"Where is the A?\" with B: The A is in the B.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..150).map(carried_case).collect();
    let lines = learn_rules(topic, &base, &cases.iter().collect::<Vec<&Case>>());
    assert!(lines.iter().any(|line| line.starts_with("If R got the S then") && line.ends_with(" is R.")), "no state of the thing is invented: {lines:?}");
}

#[test]
fn one_state_is_invented_for_the_several_sentences_that_start_it_and_end_it() {
    let text = "If A moved to the B then A went to the B.
Reply to \"Where is the A?\" with B: The A is in the B.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..150).map(carried_case).collect();
    let lines = learn_rules(topic, &base, &cases.iter().collect::<Vec<&Case>>());
    let started: Vec<&String> = lines.iter().filter(|line| line.ends_with(" is R.") && (line.starts_with("If R took the S then") || line.starts_with("If R grabbed the S then"))).collect();
    assert_eq!(started.len(), 2, "one state is not started by both sentences: {lines:?}");
    let learned = read_topic("rooms", &format!("{text}
{}
Where is the A? B if latest The A is in the B.", lines.join("
"))).expect("the learned rules read");
    let model = model_of(vec![learned]);
    let story: Vec<String> = ["Aldous moved to the porch.", "Aldous grabbed the harp.", "Aldous moved to the attic.", "Aldous left the harp.", "Brisa moved to the attic.", "Brisa took the harp.", "Brisa moved to the yard."].iter().map(|sentence| sentence.to_string()).collect();
    assert_eq!(model.answer(&story, "Where is the harp?").words, vec!["yard".to_string()]);
}

const WRADES: &str = "Is A the wrade of B? yes if A is the wrade of B. no otherwise.
Reply to \"Is A the wrade of B?\" with yes: A is the wrade of B.";

fn wrade_case(at: usize) -> Case {
    let one = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 5) % NAMES.len()];
    let third = NAMES[(at + 8) % NAMES.len()];
    let mut sentences = vec![format!("{third} is the flen of {other}."), format!("{one} is the flen of {other}.")];
    let (asked, answer) = match at % 3 {
        0 => (one, "yes"),
        1 => {
            sentences.insert(0, format!("{one} is a gorb."));
            (one, "no")
        }
        _ => {
            sentences.insert(0, format!("{third} is a gorb."));
            (one, "yes")
        }
    };
    Case { sentences, question: format!("Is {asked} the wrade of {other}?"), answer: answer.to_string() }
}

#[test]
fn a_rule_that_is_wrong_only_where_another_fact_holds_is_learned_with_that_fact_as_a_condition_that_must_not_hold() {
    let library = Library::of(vec![read_topic("wrades", WRADES).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..90).map(wrade_case).collect();
    let lines = learn_rules(topic, &base, &cases.iter().collect::<Vec<&Case>>());
    assert!(lines.iter().any(|line| line.contains(" is the flen of ") && line.contains(" and not ") && line.contains(" is a gorb then ") && line.ends_with(" is the wrade of S.")), "{lines:?}");
}

fn chain_case(at: usize) -> Case {
    let names: Vec<&str> = (0..4).map(|step| NAMES[(at + step * 3) % NAMES.len()]).collect();
    let kinds = if at % 2 == 0 { ["flen", "morp", "trep"] } else { ["flen", "trep", "morp"] };
    let mut sentences: Vec<String> = (0..3).map(|step| format!("{} is the {} of {}.", names[step], kinds[step], names[step + 1])).collect();
    sentences.rotate_left(at % 3);
    let answer = if at % 2 == 0 { "yes" } else { "no" };
    Case { sentences, question: format!("Is {} the wrade of {}?", names[0], names[3]), answer: answer.to_string() }
}

#[test]
fn a_concept_that_joins_three_told_facts_in_a_row_is_learned_as_one_rule_with_three_conditions() {
    let library = Library::of(vec![read_topic("wrades", WRADES).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..40).map(chain_case).collect();
    let lines = learn_rules(topic, &base, &cases.iter().collect::<Vec<&Case>>());
    assert!(lines.iter().any(|line| line.matches(" and ").count() == 2 && line.contains(" is the flen of ") && line.contains(" is the morp of ") && line.contains(" is the trep of ") && line.contains(" is the wrade of ")), "{lines:?}");
}

const THINGS: &[&str] = &["lamp", "coin", "book", "cup", "kite", "drum"];

fn rarely_told_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 4) % NAMES.len()];
    let thing = THINGS[at % THINGS.len()];
    let room = ROOMS[(at / 3) % ROOMS.len()];
    let picked = if at % 12 == 0 { format!("{name} picked up the {thing}.") } else { format!("{name} picked up the {thing} there.") };
    let mut sentences = vec![format!("{name} went to the {room}."), picked, format!("{other} went to the {}.", ROOMS[(at + 2) % ROOMS.len()])];
    if at % 12 == 6 {
        sentences.push(format!("{name} juggled the {thing}."));
    }
    if at % 2 == 1 {
        sentences.push(format!("{name} put down the {thing}."));
    }
    Case { sentences, question: format!("Where is the {thing}?"), answer: room.to_string() }
}

#[test]
fn a_rare_form_with_one_word_less_at_its_end_counts_with_its_common_form_for_the_support_of_a_state() {
    let cases: Vec<Case> = (0..120).map(rarely_told_case).collect();
    let below = ilm_core::tools::explain::states::told_forms_below_bar(&cases.iter().collect::<Vec<&Case>>());
    let holds = |word: &str| below.iter().any(|(form, _, _)| form.iter().any(|piece| matches!(piece, LogicPiece::Word(known) if word_text(*known) == word)));
    assert!(holds("juggled"), "{below:?}");
    assert!(!holds("picked"), "{below:?}");
}

fn taking_case(at: usize) -> Case {
    let mut case = being_in_case(at);
    if at % 3 == 0 {
        case.sentences.insert(1, format!("{} took the {}.", NAMES[at % NAMES.len()], THINGS[at % THINGS.len()]));
    }
    case
}

#[test]
fn a_person_who_takes_a_thing_does_not_start_the_state_of_where_the_person_is() {
    let cases: Vec<Case> = (0..180).map(taking_case).collect();
    let state = ilm_core::tools::explain::states::person_state(&cases.iter().collect::<Vec<&Case>>(), (true, false)).expect("a state of the person is invented");
    let took = LogicPiece::Word(word_number("took"));
    assert!(!state.starts.iter().any(|(event, _, _)| event.contains(&took)), "{:?}", state.starts);
    assert_eq!((state.starts.len(), state.ends.len()), (3, 2), "went to, moved to and is in start it; is no longer in and is not in end it");
}

const GOINGS: &[&str] = &["moved to", "journeyed to", "went to", "travelled to"];

fn carried_away_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 4) % NAMES.len()];
    let thing = THINGS[at % THINGS.len()];
    let room = |step: usize| ROOMS[(at / 4 + step) % ROOMS.len()];
    let going = |step: usize| GOINGS[(at + step) % GOINGS.len()];
    let sentences = vec![
        format!("{name} {} the {}.", going(0), room(0)),
        format!("{name} took the {thing} there."),
        format!("{other} {} the {}.", going(1), room(2)),
        format!("{name} {} the {}.", going(2), room(1)),
    ];
    Case { sentences, question: format!("Where is the {thing}?"), answer: room(1).to_string() }
}

#[test]
fn the_persons_of_stories_whose_questions_ask_about_things_have_a_state_of_where_they_are_that_every_verb_of_going_starts() {
    let cases: Vec<Case> = (0..120).map(carried_away_case).collect();
    let state = ilm_core::tools::explain::states::person_state(&cases.iter().collect::<Vec<&Case>>(), (false, true)).expect("a state of the persons of the stories is invented");
    let starts: Vec<String> = state.starts.iter().map(|(event, _, _)| event.iter().filter_map(|piece| if let LogicPiece::Word(word) = piece { Some(word_text(*word)) } else { None }).collect::<Vec<&str>>().join(" ")).collect();
    for going in GOINGS {
        assert!(starts.iter().any(|start| start.starts_with(going)), "{going}: {starts:?}");
    }
    assert!(!starts.iter().any(|start| start.contains("took")), "a thing taken is no place: {starts:?}");
    assert!(state.ends.is_empty(), "{:?}", state.ends);
    assert!(ilm_core::tools::explain::states::person_state(&cases.iter().collect::<Vec<&Case>>(), (true, false)).is_none(), "the questions name no person");
}

fn going_again_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 3) % NAMES.len()];
    let room = |step: usize| ROOMS[(at / 4 + step) % ROOMS.len()];
    let going = |step: usize| GOINGS[(at + step) % GOINGS.len()];
    let again = if at % 3 == 0 { room(0) } else { room(1) };
    let sentences = vec![format!("{name} {} the {}.", going(0), room(0)), format!("{other} {} the {}.", going(1), room(2)), format!("{name} {} the {again}.", going(2))];
    Case { sentences, question: format!("Where is {name}?"), answer: again.to_string() }
}

#[test]
fn a_person_who_goes_again_to_the_place_where_the_person_is_starts_the_state_again_and_ends_nothing() {
    let cases: Vec<Case> = (0..120).map(going_again_case).collect();
    let state = ilm_core::tools::explain::states::person_state(&cases.iter().collect::<Vec<&Case>>(), (true, false)).expect("a state of the person is invented");
    assert_eq!((state.starts.len(), state.ends.len()), (GOINGS.len(), 0), "every verb of going starts it: {:?} / {:?}", state.starts, state.ends);
}

const MAYBE_IN: &str = "If A moved to the B then A went to the B.
Reply to \"Is A in the B?\" with yes: Yes, A is in the B.
Reply to \"Is A in the B?\" with maybe: Maybe, A is in the B.
Reply to \"Is A in the B?\" with no: No, A is not in the B.";

fn maybe_in_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 5) % NAMES.len()];
    let room = |step: usize| ROOMS[(at / 8 + step) % ROOMS.len()];
    let either = format!("{name} is either in the {} or the {}.", room(0), room(1));
    let mut sentences = vec![format!("{other} is either in the {} or the {}.", room(2), room(3))];
    let (asked, answer) = match at % 8 {
        0 => {
            sentences.push(format!("{name} went to the {}.", room(0)));
            (room(0), "yes")
        }
        1 | 2 => {
            sentences.push(either);
            (room(at % 8 - 1), "maybe")
        }
        3 | 4 => {
            sentences.extend([either, format!("{name} moved to the {}.", room(2))]);
            (room(at % 8 - 3), "no")
        }
        5 | 6 => {
            sentences.extend([format!("{name} went to the {}.", room(2)), either]);
            (room(at % 8 - 5), "maybe")
        }
        _ => {
            sentences.push(format!("{name} moved to the {}.", room(2)));
            (room(0), "no")
        }
    };
    Case { sentences, question: format!("Is {name} in the {asked}?"), answer: answer.to_string() }
}

#[test]
fn a_move_after_an_either_ends_the_maybe_as_the_latest_whole_value_of_the_state_holds_the_asked_place_no_more() {
    let library = Library::of(vec![read_topic("maybe rooms", MAYBE_IN).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..240).map(maybe_in_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let story = |sentences: &[&str]| -> Vec<String> { sentences.iter().map(|sentence| sentence.to_string()).collect() };
    assert_eq!(model.answer(&story(&["Aldous is either in the shed or the barn."]), "Is Aldous in the barn?").words, vec!["maybe".to_string()]);
    assert_eq!(model.answer(&story(&["Aldous is either in the shed or the barn.", "Aldous moved to the loft."]), "Is Aldous in the shed?").words, vec!["no".to_string()]);
    assert_eq!(model.answer(&story(&["Aldous moved to the loft.", "Aldous is either in the shed or the barn."]), "Is Aldous in the shed?").words, vec!["maybe".to_string()]);
    assert_eq!(model.answer(&story(&["Aldous is either in the shed or the barn.", "Aldous went to the loft."]), "Is Aldous in the loft?").words, vec!["yes".to_string()]);
}

const BEING_IN: &str = "If A moved to the B then A went to the B.
Reply to \"Is A in the B?\" with yes: Yes, A is in the B.
Reply to \"Is A in the B?\" with no: No, A is not in the B.";

fn being_in_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 3) % NAMES.len()];
    let room = |step: usize| ROOMS[(at / 6 + step) % ROOMS.len()];
    let mut sentences = vec![format!("{other} went to the {}.", room(5))];
    let (asked, answer) = match at % 6 {
        0 => {
            sentences.push(format!("{name} went to the {}.", room(0)));
            (room(0), "yes")
        }
        1 => {
            sentences.extend([format!("{name} went to the {}.", room(0)), format!("{name} is no longer in the {}.", room(0))]);
            (room(0), "no")
        }
        2 => {
            sentences.extend([format!("{name} moved to the {}.", room(1)), format!("{name} is not in the {}.", room(1))]);
            (room(1), "no")
        }
        3 => {
            sentences.extend([format!("{name} is in the {}.", room(2)), format!("{name} is no longer in the {}.", room(2)), format!("{name} went to the {}.", room(2))]);
            (room(2), "yes")
        }
        4 => {
            sentences.extend([format!("{name} is in the {}.", room(3)), format!("{name} is not in the {}.", room(3))]);
            (room(3), "no")
        }
        _ => {
            sentences.push(format!("{name} moved to the {}.", room(4)));
            (room(1), "no")
        }
    };
    Case { sentences, question: format!("Is {name} in the {asked}?"), answer: answer.to_string() }
}

#[test]
fn a_state_of_the_person_that_a_question_names_ends_with_a_sentence_that_says_the_person_is_no_longer_there() {
    let library = Library::of(vec![read_topic("rooms", BEING_IN).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..180).map(being_in_case).collect();
    let state = ilm_core::tools::explain::states::person_state(&cases.iter().collect::<Vec<&Case>>(), (true, false)).expect("a state of the person is invented");
    assert_eq!((state.starts.len(), state.ends.len()), (3, 2), "went to, moved to and is in start it; is no longer in and is not in end it");
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let story = |sentences: &[&str]| -> Vec<String> { sentences.iter().map(|sentence| sentence.to_string()).collect() };
    assert_eq!(model.answer(&story(&["Aldous went to the porch.", "Aldous is no longer in the porch."]), "Is Aldous in the porch?").words, vec!["no".to_string()]);
    assert_eq!(model.answer(&story(&["Aldous is not in the attic.", "Aldous is in the yard."]), "Is Aldous in the yard?").words, vec!["yes".to_string()]);
    assert_eq!(model.answer(&story(&["Aldous moved to the hall.", "Aldous is not in the hall."]), "Is Aldous in the hall?").words, vec!["no".to_string()]);
}

const HOLDING: &str = "If A got the B there then A got the B.
If A got the B then the holder of the B is A.
If A passed the B to C then A gave the B to C.
If A gave the B to C then the holder of the B is C.
If A dropped the B there then A dropped the B.
If A dropped the B then the holder of the B is nobody.
Reply to \"Who has the B?\" with A: The holder of the B is A.";

fn held_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 5) % NAMES.len()];
    let thing = format!("{}{}", ["ka", "mo", "ti", "ru", "se", "lo", "pa", "ne", "vi", "du"][at % 10], ["rel", "bin", "tok", "sar", "mun"][at / 10 % 5]);
    let took = ["took", "grabbed", "got"][at % 3];
    let mut sentences = vec![format!("{other} went to the {}.", ROOMS[at % ROOMS.len()]), format!("{name} {took} the {thing} there.")];
    let answer = if at % 4 == 0 {
        sentences.push(format!("{name} dropped the {thing} there."));
        "nobody".to_string()
    } else {
        name.to_string()
    };
    Case { sentences, question: format!("Who has the {thing}?"), answer }
}

#[test]
fn told_forms_of_takes_are_not_joined_over_a_drop_that_the_text_reads_as_its_own_event() {
    let topic = read_topic("holding", HOLDING).expect("the text reads");
    let base = Arc::new(knowledge_base(&[], &topic.rules, true));
    let cases: Vec<Case> = (0..80).map(held_case).collect();
    let fit: Vec<&Case> = cases.iter().collect();
    let lines = learn_told(&topic, &base, &fit);
    assert!(lines.iter().any(|line| line.starts_with("If A took the B there then")), "{lines:?}");
    assert!(!lines.iter().any(|line| line.starts_with("If A B the C there then") || line.starts_with("If A C the B there then")), "{lines:?}");
}

fn carried_there_case(at: usize) -> Case {
    let mut case = carried_case(at);
    if at / 4 % 2 == 0 {
        for sentence in case.sentences.iter_mut().filter(|sentence| sentence.contains(" took the ") || sentence.contains(" grabbed the ")) {
            *sentence = sentence.replace('.', " there.");
        }
    }
    case
}

#[test]
fn an_invented_state_reads_a_thing_of_one_word_when_every_lesson_gives_one_word() {
    let text = "If A moved to the B then A went to the B.
Reply to \"Where is the A?\" with B: The A is in the B.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..150).map(carried_there_case).collect();
    let lines = learn_rules(topic, &base, &cases.iter().collect::<Vec<&Case>>());
    assert!(lines.iter().any(|line| line.starts_with("If R took the S there then") && line.ends_with(" is R.")), "{lines:?}");
    assert!(lines.iter().any(|line| line.starts_with("If R took the S then") && line.ends_with(" is R.")), "{lines:?}");
    let learned = read_topic("rooms", &format!("{text}
{}
Where is the A? B if latest The A is in the B.", lines.join("
"))).expect("the learned rules read");
    let model = model_of(vec![learned]);
    let story: Vec<String> = ["Lucy moved to the porch.", "Lucy took the kettle there.", "Lucy moved to the attic."].iter().map(|sentence| sentence.to_string()).collect();
    assert_eq!(model.answer(&story, "Where is the kettle?").words, vec!["attic".to_string()]);
    let proof = model.proof(&story, "Where is the kettle?");
    assert!(proof.iter().any(|line| line.contains(" of the kettle is lucy")), "{proof:?}");
    assert!(!proof.iter().any(|line| line.contains("of the kettle there")), "{proof:?}");
}

const THING_SHADES: &[&str] = &["red", "blue", "old", "small", "green"];

fn carried_two_word_case(at: usize) -> Case {
    let mut case = carried_case(at);
    let thing = case.question.trim_start_matches("Where is the ").trim_end_matches('?').to_string();
    let named = format!("{} {thing}", THING_SHADES[at % THING_SHADES.len()]);
    case.sentences = case.sentences.iter().map(|sentence| sentence.replace(&format!(" {thing}."), &format!(" {named}."))).collect();
    case.question = format!("Where is the {named}?");
    case
}

#[test]
fn an_invented_state_keeps_a_wide_place_for_a_thing_that_the_lessons_tell_in_two_words() {
    let text = "If A moved to the B then A went to the B.
Reply to \"Where is the A?\" with B: The A is in the B.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..150).map(carried_two_word_case).collect();
    let lines = learn_rules(topic, &base, &cases.iter().collect::<Vec<&Case>>());
    assert!(lines.iter().any(|line| line.starts_with("If R took the B then") && line.ends_with(" is R.")), "{lines:?}");
}

#[test]
fn the_discovery_report_names_every_state_tried_and_what_the_search_for_states_read() {
    use ilm_core::tools::explain::discovery::discovery_report;
    let text = "If A moved to the B then A went to the B.
Reply to \"Where is the A?\" with B: The A is in the B.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..150).map(carried_case).collect();
    let (fit, check): (Vec<&Case>, Vec<&Case>) = (cases.iter().take(100).collect(), cases.iter().skip(100).collect());
    let lines = discovery_report(&library, "rooms", &[("rooms".to_string(), fit, check)], &[], (1, &[])).expect("the report is made");
    assert!(lines.iter().any(|line| line.contains("the search for states read") && line.contains("told events")), "{lines:?}");
    assert!(lines.iter().any(|line| line.contains("state tried: right") && line.contains("If R took the S then")), "{lines:?}");
}

const VALUE_ROOMS: &[&str] = &["store room", "kitchen", "study room", "basement", "hall", "dining room", "attic", "cellar", "living room", "porch"];

fn moved_thing_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let thing = THINGS[at % THINGS.len()];
    let from = VALUE_ROOMS[at % VALUE_ROOMS.len()];
    let to = VALUE_ROOMS[(at + 1 + at / VALUE_ROOMS.len()) % VALUE_ROOMS.len()];
    let (asked, answer) = if at % 2 == 0 { (to, "entailment") } else { (from, "contradiction") };
    Case { sentences: vec![format!("{name} moved the {thing} to {to} from {from}.")], question: format!("The {thing} is now in {asked}."), answer: answer.to_string() }
}

#[test]
fn a_word_of_a_value_of_two_words_that_a_question_shares_with_its_story_is_no_frame_word() {
    use ilm_core::tools::explain::discovery::discovery_report;
    let text = "The A is now in B. entailment if The A is now in B. contradiction otherwise.\nReply to \"The A is now in B.\" with entailment: The A is now in B.";
    let library = Library::of(vec![read_topic("moved things", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..150).map(moved_thing_case).collect();
    let (fit, check): (Vec<&Case>, Vec<&Case>) = (cases.iter().take(100).collect(), cases.iter().skip(100).collect());
    let lines = discovery_report(&library, "moved things", &[("moved things".to_string(), fit, check)], &[], (1, &[])).expect("the report is made");
    let frame = lines.iter().find(|line| line.contains("the frame words of the told sentences")).expect("the frame words are told");
    let words: Vec<&str> = frame.rsplit(": ").next().unwrap_or_default().split(", ").collect();
    assert!(words.contains(&"moved") && words.contains(&"from"), "{frame}");
    assert!(!words.contains(&"room") && !words.contains(&"store"), "{frame}");
}

#[test]
fn the_discovery_report_says_what_the_sample_of_a_search_holds_beside_all_lessons() {
    use ilm_core::tools::explain::discovery::discovery_report;
    let text = "If A moved to the B then A went to the B.\nReply to \"Where is the A?\" with B: The A is in the B.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..150).map(carried_case).collect();
    let (fit, check): (Vec<&Case>, Vec<&Case>) = (cases.iter().take(100).collect(), cases.iter().skip(100).collect());
    let lines = discovery_report(&library, "rooms", &[("rooms".to_string(), fit, check)], &[], (1, &[])).expect("the report is made");
    assert!(lines.iter().any(|line| line.trim_start().starts_with("the sample: 40 lessons with") && line.ends_with("in all 100 lessons")), "{lines:?}");
}

fn before_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 5) % NAMES.len()];
    let thing = format!("{}{}", ["ka", "mo", "ti", "ru", "se", "lo", "pa", "ne", "vi", "du"][at % 10], ["rel", "bin", "tok", "sar", "mun"][at / 10 % 5]);
    let room = |step: usize| ROOMS[(at + step) % ROOMS.len()];
    let mut sentences = vec![format!("{name} moved to the {}.", room(0)), format!("{name} took the {thing}."), format!("{other} moved to the {}.", room(4)), format!("{name} moved to the {}.", room(1))];
    let (now, then) = if at % 2 == 0 {
        sentences.push(format!("{name} moved to the {}.", room(2)));
        (room(2), room(1))
    } else {
        sentences.extend([format!("{name} left the {thing}."), format!("{other} moved to the {}.", room(1)), format!("{other} took the {thing}."), format!("{other} moved to the {}.", room(2))]);
        (room(1), room(0))
    };
    Case { sentences, question: format!("Where was the {thing} before the {now}?"), answer: then.to_string() }
}

#[test]
fn the_rules_that_an_earlier_kind_of_a_set_taught_a_text_are_kept_for_a_later_kind() {
    use ilm_core::tools::explain::curriculum::taught_before;
    let text = "If A moved to the B then A went to the B.
Reply to \"Where is the A?\" with B: The A is in the B.
Reply to \"Where was the A before the B?\" with C: The A was in the C before the B.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let before = vec![(0..150).map(carried_case).collect::<Vec<Case>>()];
    let (_, _, lines, kinds) = taught_before(&library, 0, (&before, 1));
    assert!(lines.iter().any(|line| line.starts_with("If R grabbed the S then") && line.ends_with(" is R.")), "{lines:?}");
    assert_eq!(kinds, vec![0]);
    let second: Vec<Case> = (0..150).map(before_case).collect();
    let read = || before.clone();
    let taught = worded_model(&library, &Lessons { cases: &second, labels: &[], ask: None, classes: None, before: Some(&read) }).expect("the wording of the later kind is learned");
    let story: Vec<String> = ["Aldous moved to the porch.", "Aldous grabbed the harp.", "Aldous moved to the attic.", "Aldous moved to the yard."].iter().map(|sentence| sentence.to_string()).collect();
    assert_eq!(taught.answer(&story, "Where was the harp before the yard?").words, vec!["attic".to_string()]);
}

const STORY_PEOPLE: &[&str] = &["Mary", "John", "Sandra", "Daniel"];
const STORY_PLACES: &[&str] = &["kitchen", "garden", "office", "bedroom", "hallway", "bathroom"];
const STORY_THINGS: &[&str] = &["apple", "football", "milk"];
const STORY_MOVES: &[&str] = &["went to", "journeyed to", "travelled to", "moved to"];
const STORY_TAKES: &[&str] = &["picked up", "grabbed", "took", "got"];
const STORY_DROPS: &[&str] = &["dropped", "put down", "left", "discarded"];

struct Drawn(u64);

impl Drawn {
    fn below(&mut self, most: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % most
    }
}

fn carried_story(draw: &mut Drawn, events: usize) -> (Vec<String>, Vec<Vec<usize>>) {
    let mut at: Vec<Option<usize>> = vec![None; STORY_PEOPLE.len()];
    let mut holder: Vec<Option<usize>> = vec![None; STORY_THINGS.len()];
    let mut place: Vec<Option<usize>> = vec![None; STORY_THINGS.len()];
    let mut trail: Vec<Vec<usize>> = vec![Vec::new(); STORY_THINGS.len()];
    let mut sentences = Vec::new();
    for _ in 0..events {
        let person = draw.below(STORY_PEOPLE.len());
        let roll = draw.below(10);
        let held: Vec<usize> = (0..STORY_THINGS.len()).filter(|thing| holder[*thing] == Some(person)).collect();
        if roll < 2 && !held.is_empty() {
            let thing = held[draw.below(held.len())];
            holder[thing] = None;
            sentences.push(format!("{} {} the {}.", STORY_PEOPLE[person], STORY_DROPS[draw.below(STORY_DROPS.len())], STORY_THINGS[thing]));
            continue;
        }
        if roll < 5 && at[person].is_some() {
            let here: Vec<usize> = (0..STORY_THINGS.len()).filter(|thing| holder[*thing].is_none() && (place[*thing].is_none() || place[*thing] == at[person])).collect();
            if !here.is_empty() {
                let thing = here[draw.below(here.len())];
                holder[thing] = Some(person);
                if place[thing] != at[person] {
                    place[thing] = at[person];
                    trail[thing].push(at[person].unwrap_or_default());
                }
                sentences.push(format!("{} {} the {}.", STORY_PEOPLE[person], STORY_TAKES[draw.below(STORY_TAKES.len())], STORY_THINGS[thing]));
                continue;
            }
        }
        let to = draw.below(STORY_PLACES.len());
        at[person] = Some(to);
        sentences.push(format!("{} {} the {}.", STORY_PEOPLE[person], STORY_MOVES[draw.below(STORY_MOVES.len())], STORY_PLACES[to]));
        for thing in 0..STORY_THINGS.len() {
            if holder[thing] == Some(person) && place[thing] != Some(to) {
                place[thing] = Some(to);
                trail[thing].push(to);
            }
        }
    }
    (sentences, trail)
}

fn person_lesson(seed: u64) -> Case {
    let mut draw = Drawn(seed ^ 0x51ed);
    let mut at: Vec<Option<usize>> = vec![None; STORY_PEOPLE.len()];
    let mut sentences = Vec::new();
    for _ in 0..6 {
        let person = draw.below(STORY_PEOPLE.len());
        let to = draw.below(STORY_PLACES.len());
        at[person] = Some(to);
        sentences.push(format!("{} {} the {}.", STORY_PEOPLE[person], STORY_MOVES[draw.below(STORY_MOVES.len())], STORY_PLACES[to]));
    }
    let person = (0..STORY_PEOPLE.len()).find(|person| at[*person].is_some()).unwrap_or_default();
    Case { sentences, question: format!("Where is {}?", STORY_PEOPLE[person]), answer: STORY_PLACES[at[person].unwrap_or_default()].to_string() }
}

fn thing_lesson(seed: u64) -> Case {
    let mut draw = Drawn(seed);
    loop {
        let (sentences, trail) = carried_story(&mut draw, 8);
        if let Some(thing) = (0..STORY_THINGS.len()).find(|thing| !trail[*thing].is_empty()) {
            return Case { sentences, question: format!("Where is the {}?", STORY_THINGS[thing]), answer: STORY_PLACES[trail[thing][trail[thing].len() - 1]].to_string() };
        }
    }
}

fn thing_before_lesson(seed: u64) -> Case {
    let mut draw = Drawn(seed ^ 0x9e37);
    loop {
        let (sentences, trail) = carried_story(&mut draw, 12);
        if let Some(thing) = (0..STORY_THINGS.len()).find(|thing| trail[*thing].len() >= 2) {
            let path = &trail[thing];
            return Case { sentences, question: format!("Where was the {} before the {}?", STORY_THINGS[thing], STORY_PLACES[path[path.len() - 1]]), answer: STORY_PLACES[path[path.len() - 2]].to_string() };
        }
    }
}

const LOCATION_GIVEN: &str = "Builds on movement, possession, reference, grammar.
If A went to the B then the place of A is B.
If A is in the B then the place of A is B.
If A is no longer in the B then the place of A is B no more.
If A is not in the B then the place of A is B no more.
If A is either in the B or the C then the place of A is B or C.
If the place of A is B or C then B or C holds B.
If the place of A is B or C then B or C holds C.
If A moved the B to the C then the place of the B is C.
If A moved the B to C from D then the place of the B is C.
If A moved the B from D to C then the place of the B is C.
If A moved to B from C then A lives in B.
If A moved from C to B then A lives in B.
If A got the B and at that time the place of A is C then the place of the B is C.
If the holder of the B is A and A went to the C and at that time the holder of the B is A then the place of the B is C.
If A went to the C and before that time the place of A is D then A went from D to C.
If the holder of the B is A and A went from D to C and at that time the holder of the B is A then the B came to C from D.
Reply to \"Where is A?\" with B: A is in the B.
Reply to \"Is A in the B?\" with yes: Yes, A is in the B.
Reply to \"Is A in the B?\" with no: No, A is not in the B.
Reply to \"Is A in the B?\" with maybe: A may be in the B.
Reply to \"Where is the A really?\" with B: The A is really in the B.
Reply to \"Where was the A before the B?\" with C: The A was in the C before the B.
Reply to \"Where does A live?\" with B: A lives in B.";

#[test]
fn a_kind_names_the_kind_before_it_whose_lessons_taught_it_and_not_a_kind_that_taught_it_nothing() {
    use ilm_core::tools::explain::learn::{LIBRARY_FOLDER, read_library};
    use std::path::Path;
    let texts: Vec<_> = read_library(&Path::new("../..").join(LIBRARY_FOLDER)).expect("the library reads").into_iter().map(|topic| if topic.name == "location" { read_topic("location", LOCATION_GIVEN).expect("the rules of location read") } else { topic }).collect();
    let mut wanted: Vec<String> = vec!["geography".to_string()];
    let mut at = 0;
    while at < wanted.len() {
        let named: Vec<String> = texts.iter().filter(|topic| topic.name == wanted[at]).flat_map(|topic| topic.builds_on.clone()).collect();
        for name in named {
            if !wanted.contains(&name) {
                wanted.push(name);
            }
        }
        at += 1;
    }
    let library = Library::of(texts.into_iter().filter(|topic| wanted.contains(&topic.name)).collect()).expect("the library builds");
    let persons: Vec<Case> = (1..=200).map(person_lesson).collect();
    let things: Vec<Case> = (1..=200).map(thing_lesson).collect();
    let later: Vec<Case> = (1..=200).map(thing_before_lesson).collect();
    let read: Vec<Case> = (1001..=1060).map(thing_before_lesson).collect();
    let right = |model: &dyn Learner| read.iter().filter(|case| model.answer(&case.sentences, &case.question).words == vec![case.answer.clone()]).count();
    let silent = || vec![persons.clone()];
    let alone = worded_model(&library, &Lessons { cases: &later, labels: &[], ask: None, classes: None, before: Some(&silent) }).expect("the wording of the later kind is learned");
    assert!(!alone.used_kinds_before() && alone.kinds_used_before().is_empty(), "a kind before it that taught it nothing is not named");
    let both = || vec![persons.clone(), things.clone()];
    let taught = worded_model(&library, &Lessons { cases: &later, labels: &[], ask: None, classes: None, before: Some(&both) }).expect("the wording of the later kind is learned");
    assert!(taught.used_kinds_before());
    assert_eq!(taught.kinds_used_before(), vec![1], "the later kind names the kind whose lessons taught it, by its place in the set");
    let (with, without) = (right(&taught), right(&alone));
    assert!(with * 10 >= read.len() * 9 && without * 10 < read.len() * 8, "the later kind reads with the earlier kind {with} of {}, without it {without}", read.len());
}

#[test]
fn a_bridge_is_suggested_to_a_sentence_that_blocks_a_rule_from_a_sentence_of_the_stories_with_as_many_places() {
    use ilm_core::tools::explain::bridges::bridge_candidates;
    use ilm_core::tools::explain::text::pattern;
    let text = "If A moved to the B then A went to the B.
If A took the B then A got the B.
If A got the B and at that time the place of A is C then the place of the B is C.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..40).map(carried_case).collect();
    let patterns: Vec<_> = ["A went to the B", "A got the B"].iter().map(|sentence| pattern(sentence)).collect();
    let bridges = bridge_candidates(topic, &base, &cases.iter().collect::<Vec<&Case>>(), &patterns);
    assert!(bridges.iter().any(|(line, _)| line == "If A went to the B then The place of A is B."), "{bridges:?}");
}

fn visit_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 5) % NAMES.len()];
    let room = |step: usize| ROOMS[(at + step) % ROOMS.len()];
    let sentences = vec![format!("{name} moved to the {}.", room(0)), format!("{other} moved to the {}.", room(3)), format!("{name} moved to the {}.", room(1)), format!("{name} moved to the {}.", room(2))];
    Case { sentences, question: format!("Where was {name} before the {}?", room(2)), answer: room(1).to_string() }
}

#[test]
fn a_learned_form_that_reads_a_fact_before_a_time_reads_a_fact_that_came_twice_at_both_times() {
    let library = Library::of(vec![read_topic("rooms", "If A moved to the B then A went to the B.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..90).map(visit_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let story: Vec<String> = ["Aldous moved to the porch.", "Aldous moved to the attic.", "Aldous moved to the hall.", "Aldous moved to the attic.", "Aldous moved to the yard."].iter().map(|sentence| sentence.to_string()).collect();
    assert_eq!(model.answer(&story, "Where was Aldous before the hall?").words, vec!["attic".to_string()], "{:?}", model.lines_from_lessons());
}

#[test]
fn a_given_line_of_two_sentences_that_start_one_invented_state_counts_as_found_by_the_state() {
    use ilm_core::tools::explain::states::state_covers;
    let discovered = read_topic("found", "If A took the B then The got of the B is A.
If A got the B then The got of the B is A.
If A left the B then The got of the B is no more.
If A put down the B then The got of the B is no more.").expect("the rules read").rules;
    let given = |line: &str| read_topic("given", line).expect("the line reads").rules.remove(0);
    assert_eq!(state_covers(&given("If A took the B then A got the B."), &discovered), Some((0, 1)));
    assert_eq!(state_covers(&given("If A put down the B then A left the B."), &discovered), Some((3, 2)));
    assert_eq!(state_covers(&given("If A took the B then A left the B."), &discovered), None);
    assert_eq!(state_covers(&given("If A took the B then B got the A."), &discovered), None);
}

#[test]
fn discover_why_counts_a_given_line_that_two_starts_of_an_invented_state_cover() {
    use ilm_core::tools::explain::discovery::discovery_report;
    let text = "If A moved to the B then A went to the B.
Reply to \"Where is the A?\" with B: The A is in the B.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..150).map(carried_case).collect();
    let (fit, check): (Vec<&Case>, Vec<&Case>) = (cases.iter().take(100).collect(), cases.iter().skip(100).collect());
    let lines = discovery_report(&library, "rooms", &[("rooms".to_string(), fit, check)], &[("rooms".to_string(), "If A grabbed the B then A took the B.".to_string())], (1, &[])).expect("the report is made");
    assert!(lines.iter().any(|line| line.contains("two ways to start or to end one invented state")), "{lines:?}");
}

const COUNTS: &[(usize, usize)] = &[(3, 1), (1, 4), (1, 4), (5, 2), (5, 2), (2, 5), (2, 5), (4, 1), (4, 1), (1, 3), (1, 3), (3, 1)];

fn counted_case(at: usize) -> Case {
    let name = format!("{}{}", ["Al", "Bri", "Cor", "Dan", "Eli", "Fen", "Gil", "Hal"][at % 8], ["dous", "sa", "win", "ra", "mo"][at / 8 % 5]);
    let thing = format!("{}{}", ["ka", "mo", "ti", "ru", "se", "lo", "pa", "ne", "vi", "du"][at % 10], ["rel", "bin", "tok", "sar", "mun"][at / 10 % 5]);
    if at % 15 != 0 {
        let kept = at % 3 == 0;
        let asked = if kept { thing.clone() } else { format!("{thing}a") };
        return Case { sentences: vec![format!("{name} got {thing}.")], question: format!("{name} holds {asked}."), answer: if kept { "yes" } else { "no" }.to_string() };
    }
    let step = at / 15;
    let (told, asked) = COUNTS[step];
    let (frame, holds) = if step % 2 == 0 { ("atleast", told >= asked) } else { ("atmost", told <= asked) };
    Case { sentences: vec![format!("{name} owns {told} {thing}.")], question: format!("{name} owns {frame} {asked} {thing}."), answer: if holds { "yes" } else { "no" }.to_string() }
}

#[test]
fn a_rare_word_of_a_question_frame_and_an_amount_that_the_story_tells_learn_a_comparison() {
    let library = Library::of(vec![read_topic("holding", "If A got C then A holds C.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..180).map(counted_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let lines = model.lines_from_lessons();
    let said = |told: &str, question: &str| model.answer(&[told.to_string()], question).words;
    assert_eq!(said("Aldous owns 4 kites.", "Aldous owns atleast 2 kites."), vec!["yes".to_string()], "{lines:?}");
    assert_eq!(said("Aldous owns 1 kites.", "Aldous owns atleast 3 kites."), vec!["no".to_string()], "{lines:?}");
    assert_eq!(said("Aldous owns 2 kites.", "Aldous owns atmost 4 kites."), vec!["yes".to_string()], "{lines:?}");
    assert_eq!(said("Aldous owns 5 kites.", "Aldous owns atmost 2 kites."), vec!["no".to_string()], "{lines:?}");
}

const KIN_KNOWN: &str = "Father means a male parent.
Mother means a female parent.
Son means a male child.
Daughter means a female child.
Husband means a male spouse.
Wife means a female spouse.";

fn kin_case(at: usize) -> Case {
    let (one, other) = (NAMES[at % NAMES.len()], NAMES[(at + 5) % NAMES.len()]);
    let mixed = at.wrapping_mul(2_654_435_761) % 1_009;
    let relation = mixed / 2 % 6;
    let told = ["father", "mother", "son", "daughter", "husband", "wife"][relation];
    let (asked, answer) = if mixed % 3 > 0 {
        (["child", "child", "parent", "parent", "spouse", "spouse"][relation], "entailment")
    } else {
        (["father", "mother", "son", "daughter", "husband", "wife"][(relation + 1 + mixed / 12 % 5) % 6], "contradiction")
    };
    Case { sentences: vec![format!("{one} is {told} of {other}.")], question: format!("{other} is {asked} of {one}."), answer: answer.to_string() }
}

#[test]
fn a_word_that_varies_in_the_told_sentences_is_a_place_that_the_known_facts_of_its_words_scope() {
    let library = Library::of(vec![read_topic("kin", KIN_KNOWN).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..240).map(kin_case).collect();
    let lines = learn_rules(topic, &base, &cases.iter().collect::<Vec<&Case>>());
    for wanted in ["If A is B of C and B means a D parent then C is child of A.", "If A is B of C and B means a D child then C is parent of A.", "If A is B of C and B means a D spouse then C is spouse of A."] {
        assert!(lines.iter().any(|line| line == wanted), "{wanted} is not learned: {lines:?}");
    }
}

#[test]
fn rules_that_differ_in_one_word_of_a_condition_are_made_one_with_a_place_there() {
    use ilm_core::tools::explain::scopes::merged_rules;
    use ilm_core::tools::explain::text::{Condition, condition_text, place_letter};
    use ilm_core::tools::explain::words::word_text;
    let rule = |line: &str| read_topic("kin", line).expect("the line reads").rules.remove(0);
    let rules = [rule("If A And his B went to C then A went to C with a friend."), rule("If A And her B went to C then A went to C with a friend."), rule("If A And her B went to C then A went to C with a guide.")];
    let shapes: Vec<(Vec<(String, _)>, _)> = rules.iter().map(|rule| (rule.conditions.iter().map(|condition| match condition {
        Condition::Fact { pattern, .. } => (String::new(), pattern.clone()),
        _ => panic!("a rule of facts"),
    }).collect(), rule.outcome.clone())).collect();
    let listed: Vec<(&[(String, _)], &_)> = shapes.iter().map(|(conditions, outcome)| (conditions.as_slice(), outcome)).collect();
    let merged = merged_rules(&listed);
    assert_eq!(merged.len(), 1, "only the two rules with the same outcome are made one");
    let piece = |logic_piece: &ilm_core::tools::explain::text::LogicPiece| match logic_piece {
        ilm_core::tools::explain::text::LogicPiece::Word(word) => word_text(*word).to_string(),
        ilm_core::tools::explain::text::LogicPiece::Open(place) | ilm_core::tools::explain::text::LogicPiece::OpenWithEnd(place, _) => place_letter(*place),
    };
    let condition = Condition::Fact { pattern: merged[0].conditions[0].1.clone(), when: ilm_core::tools::explain::text::Moment::Any };
    assert_eq!(condition_text(&condition, &piece), "A and D B went to C");
    assert_eq!(merged[0].members, vec![0, 1]);
}

#[test]
fn a_given_rule_is_matched_in_the_words_of_the_told_sentences_through_a_reading_line_of_a_text() {
    use ilm_core::tools::explain::bridges::worded_rules;
    use ilm_core::tools::explain::discovery::discovery_report;
    let rule = |line: &str| read_topic("kin", line).expect("the line reads").rules.remove(0);
    let given = rule("If A has B as C and C means a D child then A is a parent of B.");
    let worded = worded_rules(&given, &[rule("If A is B of C then C has A as B.")]);
    assert_eq!(worded.len(), 1);
    let spelled = |condition: &ilm_core::tools::explain::text::Condition| ilm_core::tools::explain::text::condition_text(condition, &|logic_piece| match logic_piece {
        ilm_core::tools::explain::text::LogicPiece::Word(word) => ilm_core::tools::explain::words::word_text(*word).to_string(),
        ilm_core::tools::explain::text::LogicPiece::Open(place) | ilm_core::tools::explain::text::LogicPiece::OpenWithEnd(place, _) => ilm_core::tools::explain::text::place_letter(*place),
    });
    assert_eq!(spelled(&worded[0].0.conditions[0]), "B is C of A");
    let library = Library::of(vec![read_topic("kin", KIN_KNOWN).expect("the text reads"), read_topic("kin_said", "Builds on kin.\nIf A is B of C then C has A as B.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..300).map(kin_case).collect();
    let kinds = vec![("kin".to_string(), cases[..240].iter().collect::<Vec<&Case>>(), cases[240..].iter().collect::<Vec<&Case>>())];
    let lines = discovery_report(&library, "kin", &kinds, &[("kin".to_string(), format!("{KIN_KNOWN}\nIf A has B as C and C means a D child then A is a parent of B."))], (1, &[])).expect("the report is made");
    assert!(lines.iter().any(|line| line.starts_with("If A has B as C and C means a D child then A is a parent of B. |") && line.contains("If A is B of C and B means a D child then C is parent of A.") && line.contains("renamed through the reading line \"If A is B of C then C has A as B\" of kin_said")), "{lines:?}");
}

#[test]
fn a_rule_learned_in_the_words_of_a_kind_is_split_through_a_sentence_that_the_question_forms_of_the_text_read() {
    let text = format!("{KIN_KNOWN}
What is B to A? C if A is a parent of B and C means a D child. C if A is a spouse of B and C means a D spouse.");
    let library = Library::of(vec![read_topic("kin", &text).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..240).map(kin_case).collect();
    let lines = learn_rules(topic, &base, &cases.iter().collect::<Vec<&Case>>());
    for wanted in ["If A is B of C and B means a D parent then A is a parent of C.", "If A is a parent of C then C is child of A."] {
        assert!(lines.iter().any(|line| line == wanted), "{wanted} is not learned: {lines:?}");
    }
}

const SHADES: &[&str] = &["zorn", "pell", "quib", "dask", "vemt", "rilo", "suth", "gane", "hult", "mirp", "tobe", "wakk"];

const OBJECTS: &[&str] = &["kite", "mug", "lamp", "sock", "vase", "drum", "bell", "cart", "fork", "boot"];

fn shade_case(at: usize) -> Case {
    let thing = OBJECTS[at % OBJECTS.len()];
    let asked = SHADES[at / OBJECTS.len() % SHADES.len()];
    let same = at % 3 != 0;
    let told = if same { asked } else { SHADES[(at / OBJECTS.len() + 5) % SHADES.len()] };
    Case { sentences: vec![format!("The {thing} is {told}.")], question: format!("Is {asked} the colour of the {thing}?"), answer: if same { "yes" } else { "no" }.to_string() }
}

#[test]
fn a_common_word_that_stands_among_many_values_is_a_place_of_one_form() {
    let library = Library::of(vec![read_topic("shades", "If the A is C then the colour of the A is C.").expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..120).map(shade_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let lines = model.lines_from_lessons();
    assert_eq!(model.named_words().map(|(named, _)| named), Some(0), "{lines:?}");
    assert_eq!(model.answer(&["The kite is teal.".to_string()], "Is teal the colour of the kite?").words, vec!["yes".to_string()], "{lines:?}");
    assert_eq!(model.answer(&["The kite is ochre.".to_string()], "Is teal the colour of the kite?").words, vec!["no".to_string()], "{lines:?}");
}

const EITHER_TEXT: &str = "If A went to the B then the place of A is B.
If A is either in the B or the C then the place of A is B or C.
If the place of A is B or C then B or C holds B.
If the place of A is B or C then B or C holds C.";

fn either_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let room = |step: usize| PLACES[(at * 7 + step * 5 + at / 6) % PLACES.len()];
    let mut sentences = vec![format!("{name} went to the {}.", room(0))];
    let mut latest = vec![room(0)];
    if at % 3 != 0 {
        let (one, other) = (room(1), room(2));
        if one != other {
            sentences.push(format!("{name} is either in the {one} or the {other}."));
            latest = vec![one, other];
        }
    }
    if at % 4 == 0 {
        sentences.push(format!("{name} went to the {}.", room(3)));
        latest = vec![room(3)];
    }
    let asked = match at % 5 {
        0 => latest[0],
        1 => latest[latest.len() - 1],
        2 => room(0),
        3 => room(1),
        _ => room(4),
    };
    let answer = if !latest.contains(&asked) { "no" } else if latest.len() == 1 { "yes" } else { "maybe" };
    Case { sentences, question: format!("Is {name} in the {asked}?"), answer: answer.to_string() }
}

#[test]
fn a_value_of_the_latest_fact_and_a_relation_of_the_text_read_a_maybe_answer() {
    let library = Library::of(vec![read_topic("either", EITHER_TEXT).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..180).map(either_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let lines = model.lines_from_lessons();
    let story = |told: &[&str]| told.iter().map(|sentence| sentence.to_string()).collect::<Vec<String>>();
    let either = story(&["Aldous went to the attic.", "Aldous is either in the garden or the office."]);
    assert_eq!(model.answer(&either, "Is Aldous in the office?").words, vec!["maybe".to_string()], "{lines:?}");
    assert_eq!(model.answer(&either, "Is Aldous in the garden?").words, vec!["maybe".to_string()], "{lines:?}");
    assert_eq!(model.answer(&either, "Is Aldous in the attic?").words, vec!["no".to_string()], "{lines:?}");
    let moved = story(&["Aldous is either in the garden or the office.", "Aldous went to the cellar."]);
    assert_eq!(model.answer(&moved, "Is Aldous in the garden?").words, vec!["no".to_string()], "{lines:?}");
    assert_eq!(model.answer(&moved, "Is Aldous in the cellar?").words, vec!["yes".to_string()], "{lines:?}");
}

const VESSELS: &[&str] = &["box", "bag", "tin", "jar", "crate", "pouch", "urn", "sack"];

fn moved_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let thing = OBJECTS[at / NAMES.len() % OBJECTS.len()];
    let (first, second) = (VESSELS[at % VESSELS.len()], VESSELS[(at / VESSELS.len() + at + 1) % VESSELS.len()]);
    let sentences = vec![format!("The {thing} is in the {first}."), format!("{name} moved the {thing} to the {second}.")];
    Case { sentences, question: format!("Where is the {thing} really?"), answer: second.to_string() }
}

#[test]
fn a_value_told_more_than_once_is_read_from_the_latest_fact() {
    let text = "If A moved the B to the C then the place of the B is C.
If the B is in the C then the place of the B is C.";
    let library = Library::of(vec![read_topic("vessels", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..120).map(moved_case).filter(|case| case.sentences[0] != case.sentences[1]).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let lines = model.lines_from_lessons();
    let story: Vec<String> = ["The kite is in the box.", "Wren moved the kite to the bag.", "Tovin moved the kite to the tin."].iter().map(|sentence| sentence.to_string()).collect();
    assert_eq!(model.answer(&story, "Where is the kite really?").words, vec!["tin".to_string()], "{lines:?}");
}

fn told_condition_case(at: usize) -> Case {
    let name = NAMES[at % NAMES.len()];
    let other = NAMES[(at + 1 + at / NAMES.len()) % NAMES.len()];
    let place = PLACES[at / 2 % PLACES.len()];
    let elsewhere = PLACES[(at / 2 + 1 + at / 7) % PLACES.len()];
    let held = at % 3 != 0;
    let there = if held { place } else { elsewhere };
    let (condition, event, question) = if at % 2 == 0 {
        (format!("{name} will come to the {place} if {other} comes."), format!("{other} came to the {there}."), format!("{name} came to the {place}."))
    } else {
        (format!("{name} will stay at the {place} if {other} stays."), format!("{other} stayed at the {there}."), format!("{name} stayed at the {place}."))
    };
    Case { sentences: vec![condition, event], question, answer: if held { "entailment" } else { "neutral" }.to_string() }
}

#[test]
fn a_rule_over_a_told_condition_keeps_the_words_that_its_told_fact_holds() {
    let text = "If C will come to the B if A comes then C may come to the B.
If C will stay at the B if A stays then C may stay at the B.
If A came to the B then A was at the B.
If A stayed at the B then A was at the B.";
    let library = Library::of(vec![read_topic("conditions", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..240).map(told_condition_case).collect();
    let (fit, check): (Vec<&Case>, Vec<&Case>) = (cases.iter().take(160).collect(), cases.iter().skip(160).collect());
    let lines = discovery_report(&library, "conditions", &[("conditions".to_string(), fit, check)], &[], (1, &[])).expect("the report is made");
    let found = lines.iter().position(|line| line.trim_start().starts_with("rules discovered for this kind")).expect("the report names the rules discovered");
    assert!(lines[found..].iter().any(|line| line.trim() == "If A came to the B and C will come to the B if A comes then C came to the B."), "no rule over the told condition of coming: {:?}", &lines[found..]);
}

fn capital_case(at: usize) -> Case {
    let cities = ["canberra", "ottawa", "lima", "oslo", "quito", "accra"];
    let countries = ["australia", "canada", "peru", "norway", "ecuador", "ghana"];
    let name = NAMES[at % NAMES.len()];
    let city = at / NAMES.len() % cities.len();
    let asked = (city + at % 2) % countries.len();
    Case { sentences: vec![format!("{name} lives in {}.", cities[city])], question: format!("{name} lives in {}.", countries[asked]), answer: if asked == city { "entailment" } else { "neutral" }.to_string() }
}

#[test]
fn a_value_that_only_the_outcome_names_stays_refused() {
    let library = Library::of(vec![read_topic("homes", "Note: where people live.").expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..240).map(capital_case).collect();
    let lines = learn_rules(topic, &base, &cases.iter().collect::<Vec<&Case>>());
    assert!(!lines.iter().any(|line| line.ends_with("lives in australia.") || line.ends_with("lives in canada.")), "a rule says a country of the answer whatever its places hold: {lines:?}");
}

#[test]
fn a_rule_that_joins_two_told_facts_needs_no_known_fact_to_scope_it() {
    let text = format!("Note: told conditions and told events.
{}", NAMES.iter().map(|name| format!("{name} is a name.")).collect::<Vec<String>>().join("
"));
    let library = Library::of(vec![read_topic("conditions", &text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..240).map(told_condition_case).collect();
    let (fit, check): (Vec<&Case>, Vec<&Case>) = (cases.iter().take(160).collect(), cases.iter().skip(160).collect());
    let lines = discovery_report(&library, "conditions", &[("conditions".to_string(), fit, check)], &[], (1, &[])).expect("the report is made");
    let found = lines.iter().position(|line| line.trim_start().starts_with("rules discovered for this kind")).expect("the report names the rules discovered");
    let discovered: Vec<&String> = lines[found..].iter().filter(|line| line.trim_start().starts_with("If ")).collect();
    assert!(discovered.iter().any(|line| line.trim() == "If A came to the B and C will come to the B if A comes then C came to the B."), "no rule that joins the told facts of coming: {discovered:?}");
    assert!(discovered.iter().any(|line| line.trim() == "If A stayed at the B and C will stay at the B if A stays then C stayed at the B."), "no rule that joins the told facts of staying: {discovered:?}");
}

fn guided_case(at: usize) -> Case {
    let (one, other) = (NAMES[at % NAMES.len()], NAMES[(at + 3) % NAMES.len()]);
    let role = ["mentor", "pupil", "rival", "scout", "envoy", "tutor", "herald"][at % 7];
    Case { sentences: vec![format!("{one} {role} {other}.")], question: format!("('{one}', '{other}')"), answer: role.to_string() }
}

#[test]
fn a_reply_line_of_the_text_names_the_asked_fact_of_a_question_of_another_form_in_the_order_that_the_lessons_tell() {
    use ilm_core::tools::explain::discovery::discovery_report;
    let text = "What is B to A? C if B is the C of A.
Reply to \"What is B to A?\" with C: B is the C of A.";
    let library = Library::of(vec![read_topic("roles", text).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..140).map(guided_case).collect();
    let lines = discovery_report(&library, "roles", &[("roles".to_string(), cases[..100].iter().collect(), cases[100..].iter().collect())], &[], (1, &[])).expect("the report is made");
    assert!(lines.iter().any(|line| line.contains("the asked fact: named by the reply line \"B is the C of A.\", its places in the order A B, as \"marek is the mentor of wren\"")), "{lines:?}");
}

const KIN_FORM: &str = "Son means a male child.
Daughter means a female child.
Brother means a male sibling.
Sister means a female sibling.
What is B to A? C if A is a parent of B and B is D and C means a D child. C if A is a sibling of B and B is D and C means a D sibling.
Reply to \"What is B to A?\" with C: B is the C of A.";

fn two_step_case(at: usize) -> Case {
    let mixed = at.wrapping_mul(2_654_435_761) % 1_009;
    let (one, two, three) = (NAMES[at % NAMES.len()], NAMES[(at + 4) % NAMES.len()], NAMES[(at + 7) % NAMES.len()]);
    let first = ["son", "daughter", "brother", "sister"][mixed % 4];
    let sibling = ["brother", "sister"][mixed / 4 % 2];
    let answer = match (mixed % 4 < 2, sibling == "brother") {
        (true, true) => "son",
        (true, false) => "daughter",
        (false, true) => "brother",
        (false, false) => "sister",
    };
    Case { sentences: vec![format!("{one} {first} {two}."), format!("{two} {sibling} {three}.")], question: format!("('{one}', '{three}')"), answer: answer.to_string() }
}

#[test]
fn the_facts_that_the_text_s_question_form_needs_for_an_answer_teach_the_readings_of_the_told_sentences_and_the_rules_between_them() {
    let library = Library::of(vec![read_topic("kin", KIN_FORM).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..200).map(two_step_case).collect();
    let lines = learn_rules(topic, &base, &cases.iter().collect::<Vec<&Case>>());
    for wanted in ["If A C B and C means a D child then A is a parent of B.", "If A C B and C means a D sibling then A is a sibling of B."] {
        assert!(lines.iter().any(|line| line == wanted), "{wanted} is not learned: {lines:?}");
    }
    assert!(lines.iter().any(|line| line.starts_with("If A C B and C means a ") && line.ends_with(" then B is E.")), "the sex of a person is not learned as one rule: {lines:?}");
}

const FIRST_SOUNDS: &[&str] = &["Ka", "Lo", "Mi", "Ne", "Po", "Ru", "Sa", "Ti", "Vo", "Ze"];

const LAST_SOUNDS: &[&str] = &["ran", "lin", "mo", "dor", "vek", "tum", "bis"];

const LAND_SOUNDS: &[&str] = &["Mar", "Dun", "Kel", "Fen", "Gor", "Hal", "Jor", "Nim", "Ost", "Pel", "Quor", "Tav", "Wex"];

fn shifted_case(at: usize) -> Case {
    let mixed = ((at as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 33) as usize;
    let name = format!("{}{}", FIRST_SOUNDS[mixed % FIRST_SOUNDS.len()], LAST_SOUNDS[mixed / FIRST_SOUNDS.len() % LAST_SOUNDS.len()]);
    let (old, new) = (format!("{}ia", LAND_SOUNDS[mixed % LAND_SOUNDS.len()]), format!("{}holm", LAND_SOUNDS[mixed / 3 % LAND_SOUNDS.len()]));
    let asked_new = mixed / 7 % 2 == 0;
    Case {
        sentences: vec![format!("{name} shifted from {old} to {new}.")],
        question: format!("{name} now dwells in {}.", if asked_new { &new } else { &old }),
        answer: if asked_new { "entailment" } else { "contradiction" }.to_string(),
    }
}

#[test]
fn a_concept_that_no_text_holds_is_learned_from_lessons_of_one_told_sentence_alone() {
    let cases: Vec<Case> = (0..150).map(shifted_case).collect();
    let model = worded_model(&births_library(), &Lessons::of(&cases)).expect("the rules of a new concept are learned from the lessons alone");
    let told = vec!["Aldous shifted from Brisa to Corvin.".to_string()];
    let new_home = model.answer(&told, "Aldous now dwells in Corvin.");
    assert_eq!(new_home.words, vec!["entailment".to_string()], "{}", new_home.note);
    assert!(new_home.note.contains("of the lessons alone"), "{}", new_home.note);
    assert_eq!(model.answer(&told, "Aldous now dwells in Brisa.").words, vec!["contradiction".to_string()]);
}

#[test]
fn a_group_of_stored_lines_is_taken_whole_when_it_holds_on_a_kind_and_passed_over_when_a_line_answers_wrong() {
    use ilm_core::tools::explain::wording::kept_of_stored;
    let text = "If A moved to the B then A went to the B.
If A took the B then A got the B.
If A grabbed the B then A got the B.
If A put down the B then A dropped the B.
If A left the B then A dropped the B.
Reply to \"Where is the A?\" with B: The A is in the B.";
    let library = Library::of(vec![read_topic("rooms", text).expect("the text reads")]).expect("the library builds");
    let topic = &library.topics[0];
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, keeps_history(&topic.rules, &topic.questions)));
    let cases: Vec<Case> = (0..150).map(carried_case).collect();
    let fit: Vec<&Case> = cases.iter().collect();
    let lines = learn_rules(topic, &base, &fit);
    assert!(!lines.is_empty());
    let own = ("kin_own (src_kin), 150 lessons", lines.clone());
    let taken_own = Some((own.0.to_string(), lines.clone()));
    assert_eq!(kept_of_stored(topic, &base, &fit, &[own.clone()]), taken_own, "the lines discovered on the kind's own lessons hold there");
    let wrong = "If A went to the B and A got the C then The C is in the B.".to_string();
    let spoiled = ("kin_spoiled (src_kin), 150 lessons", lines.iter().cloned().chain([wrong.clone()]).collect::<Vec<String>>());
    assert_eq!(kept_of_stored(topic, &base, &fit, &[spoiled.clone()]), None, "a group with a line that answers wrong on the kind is passed over whole");
    assert_eq!(kept_of_stored(topic, &base, &fit, &[spoiled, own]), taken_own, "the next group that holds is taken whole");
    assert_eq!(kept_of_stored(topic, &base, &fit, &[("kin_wrong (src_kin), 150 lessons", vec![wrong])]), None, "a store that leaves the sample wrong is not taken, and the search runs");
}

#[test]
fn the_store_keeps_the_lines_of_each_kind_whole_and_reads_back_what_it_wrote() {
    use ilm_core::tools::explain::known_good::{source_kind, store_text, stored_groups, stored_of};
    let first = store_text(&[], &["If A is a parent of B then B is a child of A.".to_string()], "kin_one (src_kin), 40 lessons");
    let known = stored_of(&first);
    assert_eq!(known.len(), 1);
    assert_eq!(known[0].source, "kin_one (src_kin), 40 lessons");
    let both = stored_of(&store_text(&known, &["If A is a parent of B then B is a child of A.".to_string(), "If A is a child of B then B is a parent of A.".to_string()], "kin_two (src_kin), 60 lessons"));
    let parent = "If A is a parent of B then B is a child of A.";
    let child = "If A is a child of B then B is a parent of A.";
    assert_eq!(both.iter().map(|stored| (stored.line.as_str(), stored.source.as_str())).collect::<Vec<(&str, &str)>>(), vec![(parent, "kin_one (src_kin), 40 lessons"), (parent, "kin_two (src_kin), 60 lessons"), (child, "kin_two (src_kin), 60 lessons")], "a line that a later kind discovers again is stored again under it");
    let groups = stored_groups(&both);
    assert_eq!(groups, vec![("kin_one (src_kin), 40 lessons", vec![parent.to_string()]), ("kin_two (src_kin), 60 lessons", vec![parent.to_string(), child.to_string()])]);
    assert_eq!(source_kind(groups[1].0), "kin_two");
    let again = stored_of(&store_text(&both, &[child.to_string()], "kin_two (src_kin), 60 lessons"));
    assert_eq!(again, both, "a kind that stores its lines again adds nothing");
}

fn went_case(at: usize) -> Case {
    let mixed = ((at as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 33) as usize;
    let name = NAMES[mixed % NAMES.len()];
    let other = NAMES[(mixed / 7 + 1 + mixed) % NAMES.len()];
    let place = PLACES[mixed / 3 % PLACES.len()];
    let elsewhere = PLACES[(mixed / 5 + 1 + mixed / 3) % PLACES.len()];
    Case { sentences: vec![format!("{other} went to the {elsewhere}."), format!("{name} went to the {place}.")], question: format!("Where did {name} go?"), answer: place.to_string() }
}

#[test]
fn a_kind_whose_answer_a_told_sentence_names_is_read_by_forms_alone_when_no_text_promises_a_wording() {
    let cases: Vec<Case> = (0..150).map(went_case).filter(|case| !case.sentences[0].starts_with(case.question.split_whitespace().nth(2).unwrap_or_default())).collect();
    let model = worded_model(&births_library(), &Lessons::of(&cases)).expect("question forms are learned on a text with no line");
    let said = model.answer(&["Ilse went to the cellar.".to_string(), "Marek went to the attic.".to_string()], "Where did Marek go?");
    assert_eq!(said.words, vec!["attic".to_string()], "{}", said.note);
    assert!(said.note.contains("of the lessons alone"), "{}", said.note);
}

const CARRIED_TEXT: &str = "If A moved to the B then A went to the B.
If A got the B then The holder of the B is A.
If A dropped the B then The holder of the B is no more.
If A went to the B and The holder of the C is A and at that time A went to the B then The C is in the B.
If The holder of the C is A and A went to the B and at that time The holder of the C is A then The C is in the B.";

fn two_things_case(at: usize) -> Case {
    let (first, second) = (NAMES[at % NAMES.len()], NAMES[(at + 1) % NAMES.len()]);
    let things = ["ball", "cup", "key", "pen", "box", "hat", "jar"];
    let (carried, later) = (things[at % things.len()], things[(at + 3) % things.len()]);
    let room = |step: usize| ROOMS[(at + step) % ROOMS.len()];
    let sentences = vec![format!("{first} moved to the {}.", room(0)), format!("{first} got the {carried}."), format!("{first} moved to the {}.", room(1)), format!("{first} moved to the {}.", room(2)), format!("{first} moved to the {}.", room(4)), format!("{second} moved to the {}.", room(3)), format!("{second} got the {later}."), format!("{second} moved to the {}.", room(2))];
    if at % 2 == 0 {
        Case { sentences, question: format!("Where was the {carried} before the {}?", room(2)), answer: room(1).to_string() }
    } else {
        Case { sentences, question: format!("Where was the {later} before the {}?", room(2)), answer: room(3).to_string() }
    }
}

#[test]
fn where_a_thing_was_before_a_place_is_read_from_that_thing_and_not_from_the_latest_thing_in_the_place() {
    let library = Library::of(vec![read_topic("carried", CARRIED_TEXT).expect("the text reads")]).expect("the library builds");
    let cases: Vec<Case> = (0..120).map(two_things_case).collect();
    let model = worded_model(&library, &Lessons::of(&cases)).expect("the wording of the kind is learned");
    let story: Vec<String> = ["Gus moved to the attic.", "Gus got the mug.", "Gus moved to the porch.", "Gus moved to the yard.", "Gus moved to the cellar.", "Hal moved to the hall.", "Hal got the jar.", "Hal moved to the yard."].iter().map(|sentence| sentence.to_string()).collect();
    assert_eq!(model.answer(&story, "Where was the mug before the yard?").words, vec!["porch".to_string()], "{:?}", model.lines_from_lessons());
    assert_eq!(model.answer(&story, "Where was the jar before the yard?").words, vec!["hall".to_string()], "{:?}", model.lines_from_lessons());
}
