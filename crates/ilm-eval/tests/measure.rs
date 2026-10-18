use ilm_core::base::lanes::store_offered;
use ilm_core::choice::{inner_parts, take};
use ilm_eval::measure::board::{compare_boards, grown_jobs, grown_ways};
use ilm_eval::measure::kind::{Asked, Ground, learned, line, measure, renamed};
use ilm_eval::measure::question_list::question_list_share;
use ilm_core::split::parts;
use ilm_core::cases::Case;
use ilm_core::tools::call::{AnswerMark, Learned, Learner, Lessons, Said, Tool};
use ilm_core::tools::plain::{PLAIN_TOOL, QUESTION_TOOL};
use std::collections::HashMap;

const PLACES: [&str; 6] = ["kitchen", "garden", "office", "bedroom", "hallway", "cellar"];
const NAMES: [&str; 5] = ["Mary", "John", "Sandra", "Daniel", "Fred"];

fn moves(cases: usize) -> Vec<Case> {
    (0..cases)
        .map(|case| {
            let first = PLACES[(case * 7 + 1) % PLACES.len()];
            let then = PLACES[(case * 5 + 3) % PLACES.len()];
            let who = NAMES[case % NAMES.len()];
            let other = NAMES[(case + 1 + case / NAMES.len()) % NAMES.len()];
            Case {
                sentences: vec![format!("{who} went to the {first}."), format!("{other} went to the {then}."), format!("Story {case} ends here.")],
                question: format!("Where is {who}?"),
                answer: if other == who { then.to_string() } else { first.to_string() },
            }
        })
        .collect()
}

struct LastPlace;

impl Learner for LastPlace {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let who = question.trim_end_matches('?').rsplit(' ').next().unwrap_or_default();
        let place = sentences.iter().rev().find(|sentence| sentence.starts_with(who)).and_then(|sentence| sentence.trim_end_matches('.').rsplit(' ').next());
        Said { words: place.map(|place| vec![place.to_lowercase()]).unwrap_or_default(), note: "the last place of the name".to_string(), mark: if place.is_some() { AnswerMark::Proof } else { AnswerMark::NoAnswer } }
    }
}

fn learn_last_place(_lessons: Lessons, _lanes: usize) -> Learned {
    Ok(Box::new(LastPlace))
}

fn learn_nothing(_lessons: Lessons, _lanes: usize) -> Learned {
    Err("this tool reads no kind".to_string())
}

fn learn_from_the_store_alone(_lessons: Lessons, _lanes: usize) -> Learned {
    if store_offered() { Ok(Box::new(LastPlace)) } else { Err("no store is offered".to_string()) }
}

const LAST_PLACE: Tool = Tool { name: "last place", needs_labels: false, learn: learn_last_place };
const STORE_ALONE: Tool = Tool { name: "store alone", needs_labels: false, learn: learn_from_the_store_alone };
const NOTHING: Tool = Tool { name: "nothing", needs_labels: false, learn: learn_nothing };
const LABELS_ONLY: Tool = Tool { name: "labels only", needs_labels: true, learn: learn_last_place };

fn asked(lanes: usize) -> Asked {
    Asked { fit: 600, held: 400, sealed: false, new_words: true, curve: false, show: 0, questions: 0, tool: None, lanes, small: false, control_fit: None }
}

fn ground<'a>(cases: &'a [Case], table: Option<&'a [(String, String)]>) -> Ground<'a> {
    Ground { name: "moves", cases, labels: &[], start: None, stories: false, final_part: None, sealed: None, new_words: table, answerer: None, classes: None, before: None, stored: true }
}

#[test]
fn parts_of_a_kind() {
    let cases = moves(3000);
    let cut = parts(&cases, 600, 400, None, false);
    assert_eq!((cut.learn.len(), cut.read.len(), cut.sealed.len()), (600, 400, 400));
    assert_eq!(cut.learn, (0..600).collect::<Vec<usize>>());
    assert_eq!(cut.read, (2600..3000).collect::<Vec<usize>>());
    assert_eq!(cut.sealed, (1600..2000).collect::<Vec<usize>>());
    let small = parts(&moves(1000), 600, 400, None, false);
    assert_eq!((small.learn.len(), small.read.len(), small.sealed.len()), (600, 400, 0));
    let halves = parts(&cases, 600, 400, Some(1000), false);
    assert!(halves.learn.iter().all(|place| *place < 1000));
    assert!(halves.read.iter().chain(halves.sealed.iter()).all(|place| *place >= 1000));
}

#[test]
fn one_story_stays_in_one_part() {
    let mut cases = moves(1200);
    for case in 0..1200 {
        cases[case].sentences = cases[case / 3 * 3].sentences.clone();
    }
    let cut = parts(&cases, 600, 400, None, false);
    let story = |place: &usize| place / 3;
    let learned: std::collections::HashSet<usize> = cut.learn.iter().map(story).collect();
    assert!(cut.read.iter().map(story).all(|key| !learned.contains(&key)));
    let (first, last) = inner_parts(&cases[..600]);
    let early: std::collections::HashSet<usize> = first.iter().map(story).collect();
    assert_eq!(first.len(), 402);
    assert!(last.iter().map(story).all(|key| !early.contains(&key)));
}

#[test]
fn best_tool_is_taken() {
    let cases = moves(600);
    let tools = [PLAIN_TOOL, NOTHING, LABELS_ONLY, LAST_PLACE, QUESTION_TOOL];
    let taken = take(&tools, Lessons::of(&cases), 4, true).unwrap();
    assert_eq!(tools[taken.tool].name, "last place");
    assert_eq!(taken.shares[3], 1.0);
    assert_eq!(taken.shares[1], -1.0);
    assert_eq!(taken.said[1], "this tool reads no kind");
    assert_eq!(taken.said[2], "the lessons of this kind have no labels");
    assert!(taken.stopped.is_empty());
    assert_eq!(taken.answer(&cases[0].sentences, &cases[0].question).note, "last place: the last place of the name");
    let equal = take(&[LAST_PLACE, LAST_PLACE], Lessons::of(&cases), 1, true).unwrap();
    assert_eq!(equal.tool, 0);
    assert!(take(&[NOTHING], Lessons::of(&cases), 1, true).is_err());
    assert!(take(&[], Lessons::of(&cases), 1, true).is_err());
}

#[test]
fn no_tool_is_tried_after_one_that_reads_all_the_rest() {
    let cases = moves(600);
    let tools = [PLAIN_TOOL, NOTHING, LAST_PLACE, QUESTION_TOOL];
    let every = take(&tools, Lessons::of(&cases), 4, true).unwrap();
    let first = take(&tools, Lessons::of(&cases), 4, false).unwrap();
    assert_eq!(first.tool, every.tool);
    assert_eq!(tools[first.tool].name, "last place");
    assert_eq!(first.shares[..3], every.shares[..3]);
    assert_eq!(first.shares[3], -1.0);
    assert!(every.shares[3] >= 0.0);
    for case in &cases[..50] {
        assert_eq!(first.answer(&case.sentences, &case.question), every.answer(&case.sentences, &case.question));
    }
    assert!(take(&[NOTHING], Lessons::of(&cases), 1, false).is_err());
}

#[test]
fn same_row_for_every_number_of_threads() {
    let cases = moves(1000);
    let table = vec![("mary".to_string(), "emma".to_string()), ("kitchen".to_string(), "library".to_string())];
    let tools = [PLAIN_TOOL, LAST_PLACE];
    let alone = measure(&ground(&cases, Some(&table)), &tools, &asked(1));
    let many = measure(&ground(&cases, Some(&table)), &tools, &asked(24));
    assert_eq!(alone.learner, Some(100.0));
    assert!(learned(&alone));
    assert_eq!(alone.tool, "last place");
    assert_eq!((alone.learn, alone.read, alone.copies, alone.answers), (600, 400, 0, 6));
    let words = alone.new_words.unwrap();
    assert_eq!(words.0, 100.0);
    assert!(words.1 > 0.0);
    assert_eq!(words.2, 0);
    let seconds = |text: String| text.rsplitn(3, " | ").collect::<Vec<&str>>().join("#");
    assert_eq!(seconds(line(&alone)).len(), seconds(line(&many)).len());
    assert_eq!((alone.one_answer, alone.plain, alone.question, alone.learner, alone.not_copies, alone.new_words), (many.one_answer, many.plain, many.question, many.learner, many.not_copies, many.new_words));
    assert!(line(&alone).starts_with("moves                              |   1000   600   400 |     0 |     6 |"));
    assert!(line(&alone).contains("|  100.0 LEARNED |  100.0 |"));
}

#[test]
fn kind_with_too_few_cases() {
    let cases = moves(100);
    let row = measure(&ground(&cases, None), &[LAST_PLACE], &asked(2));
    assert_eq!(row.learner, None);
    assert_eq!(line(&row), "moves                              |    100 cases: too few to learn from and to read (60 and 40)");
}

#[test]
fn model_that_fails_leaves_the_controls() {
    let cases = moves(1000);
    let row = measure(&ground(&cases, None), &[NOTHING], &asked(2));
    assert_eq!(row.learner, None);
    assert!(row.one_answer > 0.0);
    assert!(line(&row).ends_with("no result of the learner: no tool learns from these lessons: nothing: this tool reads no kind"), "{}", line(&row));
}

#[test]
fn a_run_of_fewer_lessons_than_its_set_gives_is_offered_no_store() {
    let cases = moves(1000);
    let own = measure(&ground(&cases, None), &[STORE_ALONE], &asked(1));
    assert!(own.learner.is_some(), "{}", line(&own));
    let fewer = measure(&ground(&cases, None), &[STORE_ALONE], &Asked { fit: 200, control_fit: Some(600), ..asked(1) });
    assert_eq!(fewer.learner, None);
    assert!(line(&fewer).ends_with("store alone: no store is offered"), "{}", line(&fewer));
}

#[test]
fn new_words_in_a_text() {
    let table: HashMap<&str, &str> = [("mary", "emma"), ("kitchen", "library")].into_iter().collect();
    assert_eq!(renamed("Mary went to the kitchen. MARY's kitchens? mary2", &table), "Emma went to the library. Emma's kitchens? emma2");
    assert_eq!(renamed("", &table), "");
    assert_eq!(renamed("3 + 4", &table), "3 + 4");
}

fn lights(cases: usize) -> Vec<Case> {
    (0..cases)
        .map(|case| {
            let colour = ["red", "green", "blue"][case % 3];
            let size = ["big", "small"][(case / 3) % 2];
            Case { sentences: vec![format!("Light {case} is on.")], question: format!("Must a car stop at a {size} {colour} light?"), answer: if colour == "red" && size == "big" { "yes" } else { "no" }.to_string() }
        })
        .collect()
}

#[test]
fn the_words_of_the_question_alone_give_a_control() {
    let cases = lights(600);
    assert_eq!(question_list_share(&cases[..400], &cases[400..]), Some(100.0));
    assert_eq!(question_list_share(&moves(600)[..400], &moves(600)[400..]), None);
}

const OLD_BOARD: &str = "The board of the tasks (the read part), sets: src_a\n3 kinds in 3 tasks; 2 kinds learned; 1 tasks learned in 1 sets; whole run 10 s on 4 threads (first pass 9 s)\n\nset | task | lowest share | rules by | seconds\nA | one | 100.0 | given | 2\nA | two | 50.0 | given | 1\nA | three | no result: too few cases\n\nnew words: 1 of 1\n";

const NEW_BOARD: &str = "The board of the tasks (the read part), sets: src_a\n3 kinds in 3 tasks; 2 kinds learned; 1 tasks learned in 1 sets; whole run 30 s on 1 threads (first pass 29 s)\n\nset | task | lowest share | rules by | seconds\nA | one | 100.0 | mixed | 9\nA | two | 50.0 | given | 3\nA | four | 10.0 | given | 1\n\nnew words: 0 of 1\n";

#[test]
fn a_board_compare_names_each_changed_field_the_slower_tasks_and_the_changed_lines_after_the_table() {
    let lines = compare_boards(OLD_BOARD, NEW_BOARD);
    assert_eq!(lines[0], "compared with the reference: 3 and 3 task lines; 1 the same, 1 changed, 1 only in the reference, 1 only here; 1 slower by 2 times and 3 s or more");
    for wanted in [
        "changed | A | one | rules by: given -> mixed",
        "only in the reference | A | three",
        "only here | A | four",
        "slower | A | one | 2 s -> 9 s",
        "summary | new words: 1 of 1 -> new words: 0 of 1",
        "whole run | 10 s on 4 threads (first pass 9 s) -> 30 s on 1 threads (first pass 29 s)",
    ] {
        assert!(lines.iter().any(|line| line == wanted), "{wanted} in {lines:?}");
    }
    assert_eq!(lines.len(), 7, "{lines:?}");
}

#[test]
fn a_case_whose_story_the_kind_reads_never_reaches_its_lessons_through_an_earlier_kind() {
    use ilm_eval::measure::board::{kept_apart, read_stories};
    let story = |name: &str| vec![format!("{name} moved to the attic."), format!("{name} moved to the porch.")];
    let read = Case { sentences: story("Wren"), question: "Where is Wren?".to_string(), answer: "porch".to_string() };
    let shared = Case { sentences: story("Wren"), question: "Is Wren in the porch?".to_string(), answer: "yes".to_string() };
    let other = Case { sentences: story("Tovin"), question: "Where is Tovin?".to_string(), answer: "porch".to_string() };
    let kept = kept_apart(vec![shared, other.clone()], &read_stories(&[&read]));
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].sentences, other.sentences);
}

#[test]
fn a_kind_learned_over_the_kinds_before_it_is_marked() {
    let row = ilm_eval::measure::kind::Row { kind: "moves".to_string(), cases: 1000, learn: 600, read: 400, learner: Some(100.0), curriculum: true, ..ilm_eval::measure::kind::Row::default() };
    assert!(line(&row).contains("EARLIER KINDS"), "{}", line(&row));
    let alone = ilm_eval::measure::kind::Row { curriculum: false, ..row };
    assert!(!line(&alone).contains("EARLIER KINDS"));
}

const ORDER_JOBS: &str = "kind | pass | guess of work | start s | end s | seconds | tool | ways of the learning | chains of the learning | seen while it ran
logic_grid_puzzle | first | 100 | 0.0 | 12.0 | 12.0 |  | ways: texts 0.12 s, wording promise 1.34 s in 54 runs, wording forms 10.32 s in 3 runs, wording 0.06 s | chains: 861 made, 1 ran away, 0.20 s
babi01 | first | 100 | 0.0 | 1.0 | 1.0 | explain | ways: texts 0.30 s, bars 0.20 s | chains: 10 made, 0 ran away, 0.01 s";

#[test]
fn a_job_whose_way_grew_against_the_order_file_is_flagged() {
    let jobs: Vec<String> = [
        "kind | pass | guess of work | start s | end s | seconds | tool | ways of the learning | chains of the learning | seen while it ran",
        "logic_grid_puzzle | first | 100 | 0.0 | 40.0 | 40.0 |  | ways: texts 0.07 s, wording promise 2.06 s in 54 runs, wording forms 38.26 s in 3 runs, wording 0.04 s | chains: 861 made, 1 ran away, 0.20 s",
        "babi01 | first | 100 | 0.0 | 1.0 | 1.0 | explain | ways: texts 0.90 s, bars 0.20 s | chains: 10 made, 0 ran away, 0.01 s",
        "babi02 | first | 100 | 0.0 | 1.0 | 1.0 | explain | ways: texts 9.00 s | chains: 10 made, 0 ran away, 0.01 s",
    ]
    .iter()
    .map(|line| line.to_string())
    .collect();
    let flagged = grown_ways(&jobs, ORDER_JOBS);
    assert_eq!(flagged[0], jobs[0]);
    assert!(flagged[1].ends_with(" | grew against the order file: wording forms 10.3 s to 38.3 s"), "{}", flagged[1]);
    assert_eq!(flagged[2], jobs[2], "a way three times dearer by less than the seconds of a slower task is not flagged");
    assert_eq!(flagged[3], jobs[3], "a job that the order file does not hold is not flagged");
    assert_eq!(grown_jobs(&flagged), vec!["logic_grid_puzzle (first): wording forms 10.3 s to 38.3 s".to_string()]);
}

#[test]
fn the_board_names_the_kinds_before_a_kind_by_their_places_in_its_set() {
    use ilm_eval::measure::board::earlier_kinds_named;
    let kinds: Vec<String> = ["babi01", "babi02", "babi03", "babi07", "babi08"].iter().map(|kind| kind.to_string()).collect();
    for later in ["babi03", "babi07", "babi08"] {
        assert_eq!(earlier_kinds_named(&kinds, later, &[1]), vec!["babi02".to_string()], "{later} names the kind whose lessons gave its bridges");
    }
    assert_eq!(earlier_kinds_named(&kinds, "babi07", &[0, 2]), vec!["babi01".to_string(), "babi03".to_string()]);
    assert!(earlier_kinds_named(&kinds, "babi03", &[2]).is_empty(), "a place past the kinds before it names nothing");
    assert!(earlier_kinds_named(&kinds, "babi01", &[0]).is_empty(), "the first kind has no kind before it");
}
