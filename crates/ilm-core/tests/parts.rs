use ilm_core::cases::Case;
use ilm_core::tools::call::Lessons;
use ilm_core::tools::parts::PARTS_TOOL;
use ilm_core::tools::parts::cutter::{row_words, told_words};
use std::time::Instant;

const ACTIONS: [(&str, &str); 4] = [("jump", "JUMP"), ("walk", "WALK"), ("run", "RUN"), ("look", "LOOK")];
const TIMES: [(&str, usize); 3] = [("once", 1), ("twice", 2), ("thrice", 3)];

fn command(seed: usize) -> (String, String) {
    let one = |at: usize| {
        let (word, said) = ACTIONS[at % ACTIONS.len()];
        let (times, count) = TIMES[(at / ACTIONS.len()) % TIMES.len()];
        (format!("{word} {times}"), vec![said; count].join(" "))
    };
    let (first, first_said) = one(seed * 7 + 3);
    let (second, second_said) = one(seed * 11 + 5);
    match seed % 3 {
        0 => (first, first_said),
        1 => (format!("{first} and {second}"), format!("{first_said} {second_said}")),
        _ => (format!("{first} after {second}"), format!("{second_said} {first_said}")),
    }
}

fn cases(count: usize, from: usize) -> Vec<Case> {
    (from..from + count)
        .map(|seed| {
            let (question, answer) = command(seed);
            Case { sentences: Vec::new(), question, answer }
        })
        .collect()
}

#[test]
fn words_of_a_text_and_of_an_answer() {
    assert_eq!(told_words(&["Mary's cat, a pet;".to_string()], "Where is (it)?"), ["mary's", "cat", ",", "a", "pet", ";", "where", "is", "(", "it", ")"]);
    assert_eq!(row_words("I_JUMP, then: Größe!"), ["i_jump", "then", "größe"]);
}

#[test]
fn commands_built_from_parts() {
    let lessons = cases(600, 0);
    let model = (PARTS_TOOL.learn)(Lessons::of(&lessons), 1).expect("the parts tool reads every kind");
    let read = cases(300, 10000);
    let right = read.iter().filter(|case| model.answer(&case.sentences, &case.question).words == row_words(&case.answer)).count();
    assert_eq!(right, read.len());
}

#[test]
fn speed_of_learning_and_reading() {
    let lessons = cases(1000, 0);
    let started = Instant::now();
    let model = (PARTS_TOOL.learn)(Lessons::of(&lessons), 1).expect("the parts tool reads every kind");
    let learned = started.elapsed().as_secs_f64();
    let read = cases(5000, 20000);
    let started = Instant::now();
    let right = read.iter().filter(|case| model.answer(&case.sentences, &case.question).words == row_words(&case.answer)).count();
    let answered = started.elapsed().as_secs_f64();
    println!("parts tool: {} lessons learned in {:.2} s ({:.0} lessons a second); {} cases answered in {:.2} s on one thread ({:.0} cases a second), {} right", lessons.len(), learned, lessons.len() as f64 / learned, read.len(), answered, read.len() as f64 / answered, right);
    assert_eq!(right, read.len());
}
