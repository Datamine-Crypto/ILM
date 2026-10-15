use ilm_core::base::exact::Amount;
use ilm_core::cases::Case;
use ilm_core::tools::call::{Learner, Lessons};
use ilm_core::tools::list::model::ListModel;
use std::time::Instant;

const NUMBER: [&str; 13] = ["zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven", "twelve"];
const NAMES: [&str; 8] = ["Anna", "Ben", "Cora", "Dan", "Eva", "Finn", "Gus", "Hana"];
const PLACES: [&str; 6] = ["kitchen", "garden", "office", "cellar", "attic", "hall"];

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

fn lists(n: usize) -> Vec<Case> {
    let tools = ["hammer", "saw", "drill", "wrench", "chisel"];
    let toys = ["doll", "kite", "ball", "puzzle", "robot"];
    let mut mix = Mix { state: 1 };
    let mut out = Vec::new();
    while out.len() < n {
        let (asked, mine, other) = if mix.below(2) == 1 { ("tools", tools, toys) } else { ("toys", toys, tools) };
        let (plain, odd) = (mix.below(3) == 0, mix.below(4) == 0);
        let mut things: Vec<&str> = (0..5).filter(|_| mix.below(2) == 1).map(|k| mine[k]).collect();
        if odd {
            let at = mix.below(things.len() + 1);
            things.insert(at, other[mix.below(5)]);
        }
        let items: Vec<(usize, &str)> = things.iter().map(|thing| (if plain { 1 } else { [1, 1, 2, 3][mix.below(4)] }, *thing)).collect();
        let total: usize = items.iter().filter(|(_, thing)| mine.contains(thing)).map(|(amount, _)| amount).sum();
        if items.len() < 2 || !(2..=12).contains(&total) {
            continue;
        }
        let parts: Vec<String> = items.iter().map(|(amount, thing)| if *amount == 1 { format!("a {thing}") } else { format!("{} {thing}s", NUMBER[*amount]) }).collect();
        let told = format!("I have {}, and {}.", parts[..parts.len() - 1].join(", "), parts[parts.len() - 1]);
        out.push(case(&[told], &format!("How many {asked} do I have?"), NUMBER[total]));
    }
    out
}

fn moves() -> Vec<Case> {
    (0..96)
        .map(|i: usize| {
            let a = NAMES[i % 8];
            let mut b = NAMES[(i * 3 + 1) % 8];
            if a == b {
                b = NAMES[(i * 3 + 2) % 8];
            }
            let (pa, pb) = (PLACES[(i * 5) % 6], PLACES[(i * 7 + 2) % 6]);
            let (who, place) = if i % 2 == 1 { (a, pa) } else { (b, pb) };
            case(&[format!("{a} went to the {pa}."), format!("{b} went to the {pb}.")], &format!("Where is {who}?"), place)
        })
        .collect()
}

fn amount_of_word(model: &ListModel, word: &str) -> Option<Amount> {
    let number = model.counts.symbols.find(word)?;
    model.counts.amount.get(&vec![number]).copied()
}

#[test]
fn the_amounts_are_counted_from_the_lessons() {
    let model = ListModel::made(&Lessons::of(&lists(150)));
    assert!(model.reads);
    assert_eq!(amount_of_word(&model, "a"), Some(Amount::whole(1)));
    assert_eq!(amount_of_word(&model, "three"), Some(Amount::whole(3)));
}

#[test]
fn the_things_of_the_other_class_are_left_out() {
    let model = ListModel::made(&Lessons::of(&lists(150)));
    let told = ["I have two saws, a kite, three hammers, and a doll.".to_string()];
    let said = model.answer(&told, "How many tools do I have?");
    assert_eq!(said.words, ["five"]);
    assert!(said.note.contains("left out: kite, doll"), "{}", said.note);
    assert_eq!(model.answer(&told, "How many toys do I have?").words, ["two"]);
}

#[test]
fn lessons_of_another_shape_are_not_read() {
    let model = ListModel::made(&Lessons::of(&moves()));
    assert!(!model.reads);
    assert!(model.answer(&["Hana went to the attic.".to_string()], "Where is Hana?").words.is_empty());
}

#[test]
fn speed_of_the_list_tool() {
    let cases = lists(600);
    let started = Instant::now();
    let model = ListModel::made(&Lessons::of(&cases));
    let made = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let right = cases.iter().filter(|one| model.answer(&one.sentences, &one.question).words == [one.answer.clone()]).count();
    let read = started.elapsed().as_secs_f64();
    println!("list tool: {:.0} lessons a second for the model, {:.0} cases a second to read; {right} of {} right", cases.len() as f64 / made, cases.len() as f64 / read, cases.len());
    assert!(right * 10 > cases.len() * 9);
}
