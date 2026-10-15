use ilm_core::cases::Case;
use ilm_core::tools::call::{Learner, Lessons};
use ilm_core::tools::grid::GRID_TOOL;

const ORDINALS: [&str; 4] = ["first", "second", "third", "fourth"];
const FRUITS: [&str; 6] = ["sells apples", "sells pears", "sells plums", "sells figs", "sells limes", "sells dates"];
const AWNINGS: [&str; 6] = ["has a green awning", "has a grey awning", "has a pink awning", "has a tan awning", "has a teal awning", "has a gold awning"];
const KEEPERS: [&str; 6] = ["is run by Ada", "is run by Bo", "is run by Cy", "is run by Di", "is run by Ed", "is run by Flo"];
const NEW_FRUITS: [&str; 4] = ["sells kiwis", "sells mangos", "sells nuts", "sells olives"];
const NEW_AWNINGS: [&str; 4] = ["has a red awning", "has a blue awning", "has a white awning", "has a black awning"];

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
    NotAt(usize, usize),
    JustLeft(usize, usize),
    RightOf(usize, usize),
    Neighbours(usize, usize),
    OneBetween(usize, usize),
}

fn holds(said: &Said, place: &[usize]) -> bool {
    match said {
        Said::At(thing, at) => place[*thing] == *at,
        Said::NotAt(thing, at) => place[*thing] != *at,
        Said::JustLeft(one, two) => place[*one] + 1 == place[*two],
        Said::RightOf(one, two) => place[*one] > place[*two],
        Said::Neighbours(one, two) => place[*one].abs_diff(place[*two]) == 1,
        Said::OneBetween(one, two) => place[*one].abs_diff(place[*two]) == 2,
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

fn arrangements(n: usize, kinds: usize) -> Vec<Vec<usize>> {
    let each = orders(n);
    let mut out: Vec<Vec<usize>> = vec![Vec::new()];
    for _ in 0..kinds {
        out = out.iter().flat_map(|done| each.iter().map(move |order| done.iter().chain(order.iter()).copied().collect())).collect();
    }
    out
}

fn text(said: &Said, names: &[&str]) -> String {
    match said {
        Said::At(thing, at) => format!("The stall that {} is the {} stall.", names[*thing], ORDINALS[*at]),
        Said::NotAt(thing, at) => format!("The stall that {} is not the {} stall.", names[*thing], ORDINALS[*at]),
        Said::JustLeft(one, two) => format!("The stall that {} is just left of the stall that {}.", names[*one], names[*two]),
        Said::RightOf(one, two) => format!("The stall that {} is somewhere right of the stall that {}.", names[*one], names[*two]),
        Said::Neighbours(one, two) => format!("The stall that {} and the stall that {} are neighbours.", names[*one], names[*two]),
        Said::OneBetween(one, two) => format!("There is one stall between the stall that {} and the stall that {}.", names[*one], names[*two]),
    }
}

fn pick(mix: &mut Mix, pool: &[&'static str], n: usize) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::new();
    while names.len() < n {
        let name = pool[mix.below(pool.len())];
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

fn listed(about: &str, names: &[&str]) -> String {
    let pieces: Vec<String> = names.iter().map(|name| format!("one {name}")).collect();
    let (last, rest) = pieces.split_last().expect("a list has pieces");
    format!("Each stall differs in {about}: {}, and {last}.", rest.join(", "))
}

fn puzzles(count: usize, seed: u64, pools: &[&[&'static str]]) -> Vec<Case> {
    let n = ORDINALS.len();
    let kinds = pools.len();
    let all = arrangements(n, kinds);
    let mut mix = Mix { state: seed };
    let mut out = Vec::new();
    while out.len() < count {
        let names: Vec<&'static str> = pools.iter().flat_map(|pool| pick(&mut mix, pool, n)).collect();
        let truth = all[mix.below(all.len())].clone();
        let asked = mix.below(names.len());
        let mut told: Vec<Said> = Vec::new();
        let mut left: Vec<&Vec<usize>> = all.iter().collect();
        while left.iter().any(|place| place[asked] != truth[asked]) {
            let (one, two) = (mix.below(names.len()), mix.below(names.len()));
            let said = match mix.below(6) {
                0 => Said::At(one, truth[one]),
                1 if truth[one] != (truth[one] + 1) % n => Said::NotAt(one, (truth[one] + 1) % n),
                _ if one == two || truth[one] == truth[two] => continue,
                2 if truth[one] + 1 == truth[two] => Said::JustLeft(one, two),
                3 if truth[one] > truth[two] => Said::RightOf(one, two),
                4 if truth[one].abs_diff(truth[two]) == 1 => Said::Neighbours(one, two),
                5 if truth[one].abs_diff(truth[two]) == 2 => Said::OneBetween(one, two),
                _ => continue,
            };
            left.retain(|place| holds(&said, place));
            told.push(said);
        }
        let mut sentences = vec![format!("There are {n} stalls in a row."), listed("fruit", &names[..n]), listed("awnings", &names[n..2 * n])];
        if kinds > 2 {
            sentences.push(listed("keepers", &names[2 * n..]));
        }
        sentences.extend(told.iter().enumerate().map(|(number, said)| format!("{}. {}", number + 1, text(said, &names))));
        let mut numbers: Vec<usize> = (1..=n).collect();
        let turn = mix.below(n);
        numbers.rotate_left(turn);
        let choices: Vec<String> = numbers.iter().map(|number| format!("pick: {number}")).collect();
        let question = format!("Which stall {}? {}", names[asked], choices.join(" "));
        out.push(Case { sentences, question, answer: (truth[asked] + 1).to_string() });
    }
    out
}

fn right(model: &dyn Learner, cases: &[Case]) -> usize {
    cases.iter().filter(|case| model.answer(&case.sentences, &case.question).words == ilm_core::base::words::answer_words(&case.answer)).count()
}

#[test]
fn grid_learns_what_each_form_means() {
    let lessons = puzzles(120, 3, &[&FRUITS, &AWNINGS, &KEEPERS]);
    let model = (GRID_TOOL.learn)(Lessons::of(&lessons), 1).expect("the stories list their things");
    let read = puzzles(60, 17, &[&FRUITS, &AWNINGS, &KEEPERS]);
    assert_eq!(right(model.as_ref(), &read), read.len());
    let fewer = puzzles(30, 23, &[&FRUITS, &AWNINGS]);
    assert_eq!(right(model.as_ref(), &fewer), fewer.len());
    let fresh = puzzles(30, 29, &[&NEW_FRUITS, &NEW_AWNINGS]);
    assert_eq!(right(model.as_ref(), &fresh), fresh.len());
}

#[test]
fn grid_reads_no_kind_without_lists() {
    let lessons: Vec<Case> = puzzles(40, 5, &[&FRUITS, &AWNINGS]).into_iter().map(|case| Case { sentences: case.sentences.iter().map(|sentence| sentence.replace(':', ";")).collect(), ..case }).collect();
    assert!((GRID_TOOL.learn)(Lessons::of(&lessons), 1).is_err());
}
