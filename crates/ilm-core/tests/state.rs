use ilm_core::cases::Case;
use ilm_core::tools::call::Learner;
use ilm_core::tools::state::model::StateModel;
use std::time::Instant;

const MOVERS: [&str; 6] = ["Mary", "John", "Sandra", "Daniel", "Fred", "Julie"];
const ROOMS: [&str; 5] = ["kitchen", "garden", "office", "hallway", "bedroom"];
const VERBS: [&str; 2] = ["moved to", "went to"];

fn step(seed: &mut u64) -> usize {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*seed >> 33) as usize
}

fn stories(count: usize, seed: u64) -> Vec<Case> {
    stories_of(count, seed, &MOVERS, &ROOMS)
}

fn stories_of(count: usize, seed: u64, movers: &[&str], rooms: &[&str]) -> Vec<Case> {
    let mut seed = seed;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let told = 2 + step(&mut seed) % 5;
        let mut sentences = Vec::new();
        let mut last = vec![None; movers.len()];
        for _ in 0..told {
            let person = step(&mut seed) % movers.len();
            let room = step(&mut seed) % rooms.len();
            sentences.push(format!("{} {} the {}.", movers[person], VERBS[step(&mut seed) % VERBS.len()], rooms[room]));
            last[person] = Some(room);
        }
        let known: Vec<usize> = (0..movers.len()).filter(|person| last[*person].is_some()).collect();
        let person = known[step(&mut seed) % known.len()];
        out.push(Case { sentences, question: format!("Where is {}?", movers[person]), answer: rooms[last[person].unwrap()].to_string() });
    }
    out
}

#[test]
fn a_program_from_made_stories() {
    let lessons = stories(300, 7);
    let read = stories(200, 11);
    let model = StateModel::learn(&lessons, 4);
    assert_eq!(model.found.len(), 1);
    for case in &read {
        assert_eq!(model.answer(&case.sentences, &case.question).words, [case.answer.clone()]);
    }
}

#[test]
fn new_names_and_rooms_take_the_kind_of_their_place() {
    let lessons = stories(300, 7);
    let model = StateModel::learn(&lessons, 4);
    let movers = ["Zoe", "Liam", "Olivia", "Noah", "Ava", "Lucas"];
    let rooms = ["pantry", "attic", "library", "cellar", "balcony"];
    for case in stories_of(200, 13, &movers, &rooms) {
        assert_eq!(model.answer(&case.sentences, &case.question).words, [case.answer.clone()]);
    }
}

#[test]
fn the_same_program_on_any_number_of_threads() {
    let lessons = stories(400, 3);
    let read = stories(100, 5);
    let alone = StateModel::learn(&lessons, 1);
    for lanes in [2, 8, 24] {
        let many = StateModel::learn(&lessons, lanes);
        assert_eq!(many.work, alone.work);
        for case in &read {
            assert_eq!(many.answer(&case.sentences, &case.question), alone.answer(&case.sentences, &case.question));
        }
    }
}

#[test]
fn speed_of_the_state_tool() {
    let lessons = stories(2000, 17);
    let read = stories(2000, 19);
    let lanes = std::thread::available_parallelism().map(usize::from).unwrap_or(1);
    let started = Instant::now();
    let model = StateModel::learn(&lessons, lanes);
    let learn = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let right = read.iter().filter(|case| model.answer(&case.sentences, &case.question).words == [case.answer.clone()]).count();
    let answer = started.elapsed().as_secs_f64();
    println!("state tool: {} lessons learned in {learn:.3} s ({:.0} lessons a second), {} cases answered on one thread in {answer:.3} s ({:.0} cases a second), {right} right", lessons.len(), lessons.len() as f64 / learn, read.len(), read.len() as f64 / answer);
    assert_eq!(right, read.len());
}
