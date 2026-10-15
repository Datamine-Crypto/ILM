use ilm_core::cases::Case;
use ilm_core::choice::take;
use ilm_core::tools::call::{Learned, Learner, Lessons, Said, Tool};
use ilm_core::tools::pattern::PATTERNS_TOOL;
use ilm_core::tools::ranks::RANKS_TOOL;

const NAMES: [&str; 8] = ["Anna", "Ben", "Cora", "Dan", "Eva", "Finn", "Gus", "Hana"];
const TOWNS: [&str; 6] = ["Lund", "Oslo", "Riga", "Bern", "Kiel", "Graz"];

struct Mix {
    state: u64,
}

impl Mix {
    fn below(&mut self, n: usize) -> usize {
        self.state = (self.state * 1103515245 + 12345) % 2147483648;
        ((self.state >> 8) as usize) % n
    }
}

fn case(sentences: &[String], question: &str, answer: &str) -> Case {
    Case { sentences: sentences.to_vec(), question: question.to_string(), answer: answer.to_string() }
}

fn distances(n: usize, seed: u64, low: usize, high: usize) -> Vec<Case> {
    let mut mix = Mix { state: seed };
    let mut out = Vec::new();
    while out.len() < n {
        let name = NAMES[mix.below(NAMES.len())];
        let first = TOWNS[mix.below(TOWNS.len())];
        let second = TOWNS[mix.below(TOWNS.len())];
        let (near, far) = (low + mix.below(high - low), low + mix.below(high - low));
        if first == second || near == far {
            continue;
        }
        let told = format!("{name} is {near} miles from {first} and {far} miles from {second}.");
        let question = format!("{name} is nearer to {first} than to {second}.");
        out.push(case(&[told], &question, if near < far { "true" } else { "false" }));
    }
    out
}

fn richer(n: usize, seed: u64) -> Vec<Case> {
    let mut mix = Mix { state: seed };
    let mut out = Vec::new();
    while out.len() < n {
        let (one, other) = (NAMES[mix.below(NAMES.len())], NAMES[mix.below(NAMES.len())]);
        let (mine, theirs) = (1 + mix.below(900), 1 + mix.below(900));
        if one == other || mine == theirs {
            continue;
        }
        let told = vec![format!("{one} has {mine} coins."), format!("{other} has {theirs} coins.")];
        out.push(case(&told, "How many coins does the richer one have?", &mine.max(theirs).to_string()));
    }
    out
}

fn right(model: &dyn Learner, cases: &[Case]) -> usize {
    cases.iter().filter(|case| model.answer(&case.sentences, &case.question).words == vec![case.answer.to_lowercase()]).count()
}

#[test]
fn ranks_compare_numbers_that_no_lesson_held() {
    let lessons = distances(300, 7, 10, 500);
    let read = distances(200, 99, 600, 990);
    let ranks = (RANKS_TOOL.learn)(Lessons::of(&lessons), 1).expect("the lessons order two numbers");
    let patterns = (PATTERNS_TOOL.learn)(Lessons::of(&lessons), 1).expect("the pattern learner reads every kind");
    assert_eq!(right(ranks.as_ref(), &read), read.len());
    assert!(right(patterns.as_ref(), &read) < read.len());
}

#[test]
fn ranks_give_back_the_number_of_a_rank() {
    let lessons = richer(300, 11);
    let read = richer(100, 12);
    let ranks = (RANKS_TOOL.learn)(Lessons::of(&lessons), 1).expect("the lessons order two numbers");
    assert_eq!(right(ranks.as_ref(), &read), read.len());
    let said = ranks.answer(&["Ben has 0042 coins.".to_string(), "Eva has 7 coins.".to_string()], "How many coins does the richer one have?");
    assert_eq!(said.words, vec!["0042".to_string()]);
}

#[test]
fn ranks_read_no_kind_without_two_numbers() {
    let lessons: Vec<Case> = (0..40).map(|at| case(&[format!("{} has {at} coins.", NAMES[at % NAMES.len()])], "Who has coins?", NAMES[at % NAMES.len()])).collect();
    assert!((RANKS_TOOL.learn)(Lessons::of(&lessons), 1).is_err());
    let mut marked = distances(40, 3, 10, 500);
    marked[5].sentences.push("The rank7 of the town is high.".to_string());
    assert!((RANKS_TOOL.learn)(Lessons::of(&marked), 1).is_err());
}

struct Oracle;

impl Learner for Oracle {
    fn answer(&self, sentences: &[String], _question: &str) -> Said {
        let numbers: Vec<usize> = sentences.iter().flat_map(|sentence| sentence.split(' ').filter_map(|word| word.parse::<usize>().ok()).collect::<Vec<usize>>()).collect();
        let wrong = sentences.iter().any(|sentence| sentence.starts_with(NAMES[0]));
        let truth = if (numbers.len() == 2 && numbers[0] < numbers[1]) != wrong { "true" } else { "false" };
        Said { words: vec![truth.to_string()], note: String::new() }
    }
}

fn learn_oracle(_lessons: Lessons, _lanes: usize) -> Learned {
    Ok(Box::new(Oracle))
}

const ORACLE: Tool = Tool { name: "oracle", needs_labels: false, probe: false, asks: false, over: None, learn: learn_oracle };

#[test]
fn ranks_wait_for_the_pattern_learner_to_lead() {
    let lessons = distances(120, 5, 10, 500);
    let led = take(&[PATTERNS_TOOL, ORACLE, RANKS_TOOL], Lessons::of(&lessons), 1, false).expect("a tool is taken");
    assert_eq!(led.tool, 1);
    assert!(led.shares[2] < 0.0);
    let every = take(&[PATTERNS_TOOL, ORACLE, RANKS_TOOL], Lessons::of(&lessons), 1, true).expect("a tool is taken");
    assert_eq!(every.shares, led.shares);
    let alone = take(&[PATTERNS_TOOL, RANKS_TOOL], Lessons::of(&lessons), 1, false).expect("a tool is taken");
    assert_eq!(alone.tool, 1);
}
