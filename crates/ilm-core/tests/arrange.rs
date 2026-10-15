use ilm_core::cases::Case;
use ilm_core::tools::arrange::ARRANGE_TOOL;
use ilm_core::tools::call::{Learner, Lessons};

const THINGS: [&str; 10] = ["kettle", "lamp", "drum", "vase", "clock", "bell", "jar", "fan", "rope", "brush"];
const NEW_THINGS: [&str; 6] = ["harp", "mug", "sled", "kite", "comb", "tent"];
const PLACES: [&str; 4] = ["first", "second", "third", "fourth"];

struct Mix {
    state: u64,
}

impl Mix {
    fn below(&mut self, n: usize) -> usize {
        self.state = (self.state * 1103515245 + 12345) % 2147483648;
        ((self.state >> 8) as usize) % n
    }
}

#[derive(Clone)]
enum Said {
    At(usize, usize),
    Before(usize, usize),
    After(usize, usize),
}

fn holds(said: &Said, place: &[usize]) -> bool {
    match said {
        Said::At(thing, at) => place[*thing] == *at,
        Said::Before(one, two) => place[*one] < place[*two],
        Said::After(one, two) => place[*one] > place[*two],
    }
}

fn orders(n: usize) -> Vec<Vec<usize>> {
    if n == 0 {
        return vec![Vec::new()];
    }
    let mut out = Vec::new();
    for smaller in orders(n - 1) {
        for at in 0..n {
            let mut grown: Vec<usize> = smaller.iter().map(|place| if *place >= at { place + 1 } else { *place }).collect();
            grown.push(at);
            out.push(grown);
        }
    }
    out
}

fn text(said: &Said, names: &[&str]) -> String {
    match said {
        Said::At(thing, at) => format!("The {} hangs {} on the hook.", names[*thing], PLACES[*at]),
        Said::Before(one, two) => format!("The {} hangs nearer the door than the {}.", names[*one], names[*two]),
        Said::After(one, two) => format!("The {} hangs further from the door than the {}.", names[*one], names[*two]),
    }
}

fn puzzles(count: usize, seed: u64, pool: &[&'static str]) -> Vec<Case> {
    let n = PLACES.len();
    let all = orders(n);
    let mut mix = Mix { state: seed };
    let mut out = Vec::new();
    while out.len() < count {
        let mut names: Vec<&str> = Vec::new();
        while names.len() < n {
            let name = pool[mix.below(pool.len())];
            if !names.contains(&name) {
                names.push(name);
            }
        }
        let truth = all[mix.below(all.len())].clone();
        let mut told: Vec<Said> = Vec::new();
        let mut left: Vec<&Vec<usize>> = all.iter().collect();
        while left.len() > 1 {
            let (one, two) = (mix.below(n), mix.below(n));
            let said = match mix.below(3) {
                0 => Said::At(one, truth[one]),
                _ if one == two => continue,
                1 if truth[one] < truth[two] => Said::Before(one, two),
                1 => Said::After(one, two),
                _ if truth[one] > truth[two] => Said::After(one, two),
                _ => Said::Before(one, two),
            };
            left.retain(|place| holds(&said, place));
            told.push(said);
        }
        let asked = mix.below(n);
        let answer = (0..n).find(|thing| truth[*thing] == asked).unwrap_or(0);
        let mut sentences = vec![format!("A hook holds {} things.", PLACES.len())];
        sentences.extend(told.iter().map(|said| text(said, &names)));
        let choices: Vec<String> = (0..n).map(|thing| format!("pick The {} hangs {} on the hook.", names[thing], PLACES[asked])).collect();
        out.push(Case { sentences, question: choices.join(" "), answer: format!("The {} hangs {} on the hook.", names[answer], PLACES[asked]) });
    }
    out
}

fn right(model: &dyn Learner, cases: &[Case]) -> usize {
    cases.iter().filter(|case| model.answer(&case.sentences, &case.question).words == ilm_core::base::words::answer_words(&case.answer)).count()
}

#[test]
fn arrange_learns_what_each_form_means() {
    let lessons = puzzles(150, 3, &THINGS);
    let model = (ARRANGE_TOOL.learn)(Lessons::of(&lessons), 1).expect("the questions list their choices");
    let read = puzzles(80, 17, &THINGS);
    assert_eq!(right(model.as_ref(), &read), read.len());
    let fresh = puzzles(40, 29, &NEW_THINGS);
    assert_eq!(right(model.as_ref(), &fresh), fresh.len());
}

#[test]
fn arrange_reads_no_kind_without_choices() {
    let lessons: Vec<Case> = puzzles(40, 5, &THINGS).into_iter().map(|case| Case { question: "Which one hangs first?".to_string(), ..case }).collect();
    assert!((ARRANGE_TOOL.learn)(Lessons::of(&lessons), 1).is_err());
}
