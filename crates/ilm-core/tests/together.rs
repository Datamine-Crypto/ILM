use ilm_core::cases::Case;
use ilm_core::tools::call::{Learner, Lessons};
use ilm_core::tools::together::TOGETHER_TOOL;
use std::time::Instant;

const NAMES: [&str; 8] = ["Anna", "Ben", "Cora", "Dan", "Eva", "Finn", "Gus", "Hana"];
const THINGS: [&str; 6] = ["apples", "pens", "cards", "shells", "coins", "books"];

struct Mix {
    state: u64,
}

impl Mix {
    fn below(&mut self, n: usize) -> usize {
        self.state = (self.state * 1103515245 + 12345) % 2147483648;
        ((self.state >> 8) as usize) % n
    }
}

fn case(sentences: Vec<String>, question: String, answer: String) -> Case {
    Case { sentences, question, answer }
}

fn problem(mix: &mut Mix, form: usize) -> Case {
    let name = NAMES[mix.below(NAMES.len())];
    let things = THINGS[mix.below(THINGS.len())];
    let (a, b) = (2 + mix.below(40), 2 + mix.below(9));
    match form {
        0 => case(vec![format!("{name} has {a} {things}."), format!("{name} buys {b} more {things}.")], format!("How many {things} does {name} have now?"), (a + b).to_string()),
        1 => case(vec![format!("{name} has {} {things}.", a + b), format!("{name} gives {b} {things} to a friend.")], format!("How many {things} does {name} have left?"), a.to_string()),
        _ => case(vec![format!("{name} has {b} bags."), format!("Each bag holds {a} {things}.")], format!("How many {things} does {name} have in all?"), (a * b).to_string()),
    }
}

fn problems(n: usize) -> Vec<Case> {
    let mut mix = Mix { state: 3 };
    (0..n).map(|at| problem(&mut mix, at % 3)).collect()
}

fn walks(n: usize) -> Vec<Case> {
    let ways = ["north", "south", "west", "east"];
    let mut mix = Mix { state: 11 };
    let mut out = Vec::new();
    for _ in 0..n {
        let mut sentences = vec!["The robot leaves the dock.".to_string()];
        let (mut across, mut ahead) = (0i64, 0i64);
        for _ in 0..2 + mix.below(3) {
            let way = mix.below(ways.len());
            let steps = 1 + mix.below(5) as i64;
            sentences.push(format!("The robot rolls {steps} meters {}.", ways[way]));
            match way {
                0 => ahead += steps,
                1 => ahead -= steps,
                2 => across -= steps,
                _ => across += steps,
            }
        }
        if mix.below(2) == 1 {
            let (last_across, last_ahead) = (across, ahead);
            if last_ahead != 0 {
                sentences.push(format!("The robot rolls {} meters {}.", last_ahead.abs(), if last_ahead > 0 { "south" } else { "north" }));
                ahead = 0;
            }
            if last_across != 0 {
                sentences.push(format!("The robot rolls {} meters {}.", last_across.abs(), if last_across > 0 { "west" } else { "east" }));
                across = 0;
            }
        }
        let answer = if across == 0 && ahead == 0 { "Yes" } else { "No" };
        out.push(case(sentences, "Is the robot back at the dock?".to_string(), answer.to_string()));
    }
    out
}

fn said(model: &dyn Learner, case: &Case) -> String {
    model.answer(&case.sentences, &case.question).words.join(" ")
}

#[test]
fn the_roles_of_the_numbers_are_counted_from_the_lessons() {
    let model = (TOGETHER_TOOL.learn)(Lessons::of(&problems(300)), 1).expect("the roles read word problems");
    let sum = case(vec!["Gus has 17 coins.".to_string(), "Gus buys 6 more coins.".to_string()], "How many coins does Gus have now?".to_string(), String::new());
    let rest = case(vec!["Eva has 29 shells.".to_string(), "Eva gives 8 shells to a friend.".to_string()], "How many shells does Eva have left?".to_string(), String::new());
    let product = case(vec!["Dan has 7 bags.".to_string(), "Each bag holds 12 pens.".to_string()], "How many pens does Dan have in all?".to_string(), String::new());
    assert_eq!(said(model.as_ref(), &sum), "23");
    assert_eq!(said(model.as_ref(), &rest), "21");
    assert_eq!(said(model.as_ref(), &product), "84");
}

#[test]
fn a_walk_is_read_by_its_program() {
    let model = (TOGETHER_TOOL.learn)(Lessons::of(&walks(300)), 1).expect("the walk reads steps");
    let rolls = |lines: &[&str]| -> Vec<String> { std::iter::once("The robot leaves the dock.").chain(lines.iter().copied()).map(str::to_string).collect() };
    let asked = || "Is the robot back at the dock?".to_string();
    let back = case(rolls(&["The robot rolls 3 meters north.", "The robot rolls 2 meters west.", "The robot rolls 3 meters south.", "The robot rolls 2 meters east."]), asked(), String::new());
    let away = case(rolls(&["The robot rolls 4 meters north.", "The robot rolls 1 meters west."]), asked(), String::new());
    assert_eq!(said(model.as_ref(), &back), "yes");
    assert_eq!(said(model.as_ref(), &away), "no");
}

#[test]
fn a_kind_with_no_numbers_and_no_walk_is_not_read() {
    let cases: Vec<Case> = (0..60).map(|at| case(vec![format!("{} went home.", NAMES[at % 8])], format!("Where is {}?", NAMES[at % 8]), "home".to_string())).collect();
    assert!((TOGETHER_TOOL.learn)(Lessons::of(&cases), 1).is_err());
}

#[test]
fn the_speed_of_the_together_tool() {
    let cases = problems(600);
    let begun = Instant::now();
    let model = (TOGETHER_TOOL.learn)(Lessons::of(&cases), 1).expect("the roles read word problems");
    let learned = begun.elapsed().as_secs_f64();
    let begun = Instant::now();
    let right = cases.iter().filter(|one| said(model.as_ref(), one) == one.answer).count();
    let read = begun.elapsed().as_secs_f64();
    println!("together tool: {} lessons learned per second, {} cases read per second", (cases.len() as f64 / learned) as u64, (cases.len() as f64 / read) as u64);
    assert!(right * 10 >= cases.len() * 9);
}
