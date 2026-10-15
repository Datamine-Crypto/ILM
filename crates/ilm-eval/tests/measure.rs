use ilm_core::choice::{inner_parts, take};
use ilm_eval::measure::kind::{Asked, Ground, learned, line, measure, renamed};
use ilm_core::split::parts;
use ilm_core::cases::Case;
use ilm_core::tools::call::{Learned, Learner, Lessons, Said, Tool};
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
        Said { words: place.map(|place| vec![place.to_lowercase()]).unwrap_or_default(), note: "the last place of the name".to_string() }
    }
}

fn learn_last_place(_lessons: Lessons, _lanes: usize) -> Learned {
    Ok(Box::new(LastPlace))
}

fn learn_nothing(_lessons: Lessons, _lanes: usize) -> Learned {
    Err("this tool reads no kind".to_string())
}

const LAST_PLACE: Tool = Tool { name: "last place", needs_labels: false, probe: false, asks: false, over: None, learn: learn_last_place };
const NOTHING: Tool = Tool { name: "nothing", needs_labels: false, probe: false, asks: false, over: None, learn: learn_nothing };
const LABELS_ONLY: Tool = Tool { name: "labels only", needs_labels: true, probe: false, asks: false, over: None, learn: learn_last_place };

fn asked(lanes: usize) -> Asked {
    Asked { fit: 600, held: 400, sealed: false, new_words: true, curve: false, show: 0, questions: 0, tool: None, lanes, small: false }
}

fn ground<'a>(cases: &'a [Case], table: Option<&'a [(String, String)]>) -> Ground<'a> {
    Ground { name: "moves", cases, labels: &[], start: None, stories: false, final_part: None, sealed: None, new_words: table, answerer: None, classes: None }
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
    assert!(line(&row).ends_with("no result of the learner: the run of the learner ended with an error"));
}

#[test]
fn new_words_in_a_text() {
    let table: HashMap<&str, &str> = [("mary", "emma"), ("kitchen", "library")].into_iter().collect();
    assert_eq!(renamed("Mary went to the kitchen. MARY's kitchens? mary2", &table), "Emma went to the library. Emma's kitchens? emma2");
    assert_eq!(renamed("", &table), "");
    assert_eq!(renamed("3 + 4", &table), "3 + 4");
}
