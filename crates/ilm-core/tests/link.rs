use ilm_core::cases::Case;
use ilm_core::tools::call::{Learner, Lessons};
use ilm_core::tools::link::LINK_TOOL;
use std::time::Instant;

const NAMES: [&str; 12] = ["Anna", "Ben", "Cora", "Dan", "Eva", "Finn", "Gus", "Hana", "Ivo", "Jana", "Karl", "Lena"];

struct Mix {
    state: u64,
}

impl Mix {
    fn below(&mut self, n: usize) -> usize {
        self.state = (self.state * 1103515245 + 12345) % 2147483648;
        ((self.state >> 8) as usize) % n
    }

    fn names(&mut self, n: usize) -> Vec<&'static str> {
        let mut out: Vec<&'static str> = Vec::new();
        while out.len() < n {
            let name = NAMES[self.below(NAMES.len())];
            if !out.contains(&name) {
                out.push(name);
            }
        }
        out
    }
}

fn parent(older: &str, younger: &str) -> String {
    format!("{older} is the parent of {younger}.")
}

fn asked(from: &str, to: &str) -> String {
    format!("How is {from} related to {to}?")
}

fn chain(mix: &mut Mix, links: usize, backward: bool, answer: &str) -> Case {
    let names = mix.names(links + 3);
    let mut sentences: Vec<String> = (0..links).map(|at| parent(names[at], names[at + 1])).collect();
    sentences.insert(mix.below(sentences.len() + 1), parent(names[links + 1], names[links + 2]));
    let question = if backward { asked(names[links], names[0]) } else { asked(names[0], names[links]) };
    Case { sentences, question, answer: answer.to_string() }
}

fn apart(mix: &mut Mix) -> Case {
    let names = mix.names(4);
    let sentences = vec![parent(names[0], names[1]), parent(names[2], names[3])];
    Case { sentences, question: asked(names[0], names[3]), answer: "unrelated".to_string() }
}

fn lessons(n: usize, longer: bool) -> Vec<Case> {
    let mut mix = Mix { state: 7 };
    let mut out = Vec::new();
    for at in 0..n {
        let case = match at % if longer { 6 } else { 5 } {
            0 => chain(&mut mix, 1, false, "parent"),
            1 => chain(&mut mix, 1, true, "child"),
            2 => chain(&mut mix, 2, false, "grandparent"),
            3 => chain(&mut mix, 2, true, "grandchild"),
            4 => apart(&mut mix),
            _ => chain(&mut mix, 3, false, "great grandparent"),
        };
        out.push(case);
    }
    out
}

fn learned(cases: &[Case]) -> Box<dyn Learner> {
    (LINK_TOOL.learn)(Lessons::of(cases), 1).expect("the link tool reads every kind")
}

fn said(model: &dyn Learner, case: &Case) -> (String, String) {
    let found = model.answer(&case.sentences, &case.question);
    (found.words.join(" "), found.note)
}

#[test]
fn a_way_of_two_links_is_joined_by_the_table_of_the_lessons() {
    let model = learned(&lessons(200, false));
    let story = vec![parent("Lena", "Karl"), parent("Anna", "Ben"), parent("Karl", "Ivo")];
    let forward = Case { sentences: story.clone(), question: asked("Lena", "Ivo"), answer: String::new() };
    let backward = Case { sentences: story, question: asked("Ivo", "Lena"), answer: String::new() };
    let (words, note) = said(model.as_ref(), &forward);
    assert_eq!(words, "grandparent");
    assert!(note.contains("links joined"), "{note}");
    assert_eq!(said(model.as_ref(), &backward).0, "grandchild");
}

#[test]
fn a_link_that_says_an_answer_is_read_as_that_answer() {
    let model = learned(&lessons(200, false));
    let case = Case { sentences: vec![parent("Gus", "Hana")], question: asked("Hana", "Gus"), answer: String::new() };
    assert_eq!(said(model.as_ref(), &case), ("child".to_string(), "one link".to_string()));
}

#[test]
fn a_longer_way_is_joined_at_a_later_step() {
    let model = learned(&lessons(300, true));
    let story = vec![parent("Dan", "Eva"), parent("Eva", "Finn"), parent("Finn", "Gus"), parent("Jana", "Cora")];
    let case = Case { sentences: story, question: asked("Dan", "Gus"), answer: String::new() };
    let (words, note) = said(model.as_ref(), &case);
    assert_eq!(words, "great grandparent");
    assert!(note.contains("links joined"), "{note}");
}

#[test]
fn no_way_gives_the_answer_of_the_lessons_with_no_way() {
    let model = learned(&lessons(200, false));
    let story = vec![parent("Ben", "Cora"), parent("Ivo", "Jana")];
    let case = Case { sentences: story, question: asked("Ben", "Jana"), answer: String::new() };
    assert_eq!(said(model.as_ref(), &case), ("unrelated".to_string(), "no way between the two things".to_string()));
}

#[test]
fn a_question_with_one_thing_gets_the_most_frequent_answer() {
    let cases = lessons(200, false);
    let model = learned(&cases);
    let case = Case { sentences: vec![parent("Ben", "Cora")], question: "How old is Ben?".to_string(), answer: String::new() };
    assert_eq!(said(model.as_ref(), &case), ("parent".to_string(), "not two things in the question".to_string()));
}

const RAISED: [&str; 4] = ["raised", "fostered", "reared", "nursed"];

fn raised(mix: &mut Mix, verbs: &[usize]) -> Case {
    let names = mix.names(verbs.len() + 1);
    let sentences: Vec<String> = verbs.iter().enumerate().map(|(at, verb)| format!("{} {} {}.", names[at], RAISED[*verb], names[at + 1])).collect();
    let answer = if verbs.len() == 2 { "grandparent" } else { "great grandparent" };
    Case { sentences, question: asked(names[0], names[verbs.len()]), answer: answer.to_string() }
}

#[test]
fn a_join_that_no_lesson_shows_is_lent_by_an_alike_label() {
    let mut mix = Mix { state: 11 };
    let last = RAISED.len() - 1;
    let mut cases: Vec<Case> = Vec::new();
    for round in 0..4 {
        for one in 0..RAISED.len() {
            for other in 0..RAISED.len() {
                if one == last && other == last {
                    continue;
                }
                cases.push(raised(&mut mix, &[one, other]));
                for third in 0..RAISED.len() {
                    if (other == last && third == last) || (round + one + third) % 2 == 1 {
                        continue;
                    }
                    cases.push(raised(&mut mix, &[one, other, third]));
                }
            }
        }
    }
    let model = learned(&cases);
    let (words, note) = said(model.as_ref(), &raised(&mut mix, &[last, last]));
    assert_eq!(words, "grandparent");
    assert!(note.contains("links joined"), "{note}");
}

#[test]
fn the_speed_of_the_link_tool() {
    let cases = lessons(600, true);
    let begun = Instant::now();
    let model = learned(&cases);
    let learned = begun.elapsed().as_secs_f64();
    let begun = Instant::now();
    let right = cases.iter().filter(|case| said(model.as_ref(), case).0 == case.answer).count();
    let read = begun.elapsed().as_secs_f64();
    println!("link tool: {} lessons learned per second, {} cases read per second", (cases.len() as f64 / learned) as u64, (cases.len() as f64 / read) as u64);
    assert_eq!(right, cases.len());
}
