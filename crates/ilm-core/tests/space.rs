use ilm_core::cases::Case;
use ilm_core::tools::call::{Learner, Lessons};
use ilm_core::tools::space::SPACE_TOOL;

const NAMES: [&str; 12] = ["Kip", "Lu", "Mo", "Ned", "Ola", "Pim", "Rue", "Sol", "Tam", "Uli", "Vin", "Wes"];
const WAYS: [((i32, i32), &str); 9] = [((1, 0), "east"), ((-1, 0), "west"), ((0, 1), "north"), ((0, -1), "south"), ((1, 1), "north east"), ((-1, 1), "north west"), ((1, -1), "south east"), ((-1, -1), "south west"), ((0, 0), "here")];

struct Mix {
    state: u64,
}

impl Mix {
    fn below(&mut self, n: usize) -> usize {
        self.state = (self.state * 1103515245 + 12345) % 2147483648;
        ((self.state >> 8) as usize) % n
    }
}

fn link(mix: &mut Mix, subject: &str, place: &str, way: (i32, i32)) -> String {
    let name = WAYS.iter().find(|(step, _)| *step == way).map(|(_, name)| *name).unwrap_or("here");
    match (way, mix.below(2)) {
        ((0, 0), 0) => format!("{subject} and {place} share a spot."),
        ((0, 0), _) => format!("{place} stands where {subject} stands."),
        (_, 0) => format!("{subject} waits just {name} of {place}."),
        _ => format!("Seen from {place}, {subject} is to the {name}."),
    }
}

fn stories(count: usize, seed: u64) -> Vec<Case> {
    let mut mix = Mix { state: seed };
    let mut out = Vec::new();
    while out.len() < count {
        let mut names: Vec<&str> = Vec::new();
        let size = 2 + mix.below(4);
        while names.len() < size {
            let name = NAMES[mix.below(NAMES.len())];
            if !names.contains(&name) {
                names.push(name);
            }
        }
        let mut at: Vec<(i32, i32)> = vec![(0, 0)];
        let mut sentences: Vec<String> = Vec::new();
        for next in 1..names.len() {
            let from = if mix.below(3) == 0 && next > 1 { mix.below(next) } else { next - 1 };
            let step = WAYS[mix.below(WAYS.len() - 1)].0;
            at.push((at[from].0 + step.0, at[from].1 + step.1));
            let (subject, place) = if mix.below(2) == 0 { (next, from) } else { (from, next) };
            let way = (at[subject].0 - at[place].0, at[subject].1 - at[place].1);
            sentences.push(link(&mut mix, names[subject], names[place], way));
        }
        let (one, other) = (mix.below(names.len()), mix.below(names.len()));
        if one == other {
            continue;
        }
        let gap = ((at[one].0 - at[other].0).signum(), (at[one].1 - at[other].1).signum());
        let answer = WAYS.iter().find(|(step, _)| *step == gap).map(|(_, name)| name.to_string()).unwrap_or_default();
        out.push(Case { sentences, question: format!("Where is {} from {}?", names[one], names[other]), answer });
    }
    out
}

fn right(model: &dyn Learner, cases: &[Case]) -> usize {
    cases.iter().filter(|case| model.answer(&case.sentences, &case.question).words == ilm_core::base::words::answer_words(&case.answer)).count()
}

#[test]
fn space_learns_the_steps_of_links_and_answers() {
    let lessons = stories(300, 7);
    let model = (SPACE_TOOL.learn)(Lessons::of(&lessons), 1).expect("the questions name two things");
    let read = stories(100, 41);
    assert_eq!(right(model.as_ref(), &read), read.len());
}

#[test]
fn space_reads_no_kind_without_two_named_things() {
    let lessons: Vec<Case> = stories(40, 9).into_iter().map(|case| Case { question: "Where is everyone?".to_string(), ..case }).collect();
    assert!((SPACE_TOOL.learn)(Lessons::of(&lessons), 1).is_err());
}
