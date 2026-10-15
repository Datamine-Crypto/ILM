use ilm_core::base::symbols::Symbols;
use ilm_core::cases::Case;
use ilm_core::tools::call::{Learner, read_story};
use ilm_core::tools::pattern::hash::{CheapHasher, CheapMap, mix};
use ilm_core::tools::pattern::kinds::PatternWords;
use ilm_core::tools::pattern::indexed::IndexedStory;
use ilm_core::tools::pattern::learn::PatternLearner;
use ilm_core::tools::pattern::table::ChildTable;
use std::hash::Hasher;
use std::time::Instant;

const PEOPLE: [&str; 8] = ["Mary", "John", "Sandra", "Daniel", "Anne", "Fred", "Julie", "Bill"];
const PLACES: [&str; 6] = ["kitchen", "garden", "office", "hallway", "bedroom", "bathroom"];
const MOVES: [&str; 3] = ["moved to", "went to", "journeyed to"];

fn next(seed: &mut u64) -> usize {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*seed >> 33) as usize
}

fn made_cases(count: usize, seed: u64) -> Vec<Case> {
    let mut seed = seed;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let told = 2 + next(&mut seed) % 6;
        let mut sentences = Vec::new();
        let mut last = vec![None; PEOPLE.len()];
        for _ in 0..told {
            let person = next(&mut seed) % PEOPLE.len();
            let place = next(&mut seed) % PLACES.len();
            let verb = MOVES[next(&mut seed) % MOVES.len()];
            sentences.push(format!("{} {verb} the {}.", PEOPLE[person], PLACES[place]));
            last[person] = Some(place);
        }
        let known: Vec<usize> = (0..PEOPLE.len()).filter(|person| last[*person].is_some()).collect();
        let person = known[next(&mut seed) % known.len()];
        out.push(Case { sentences, question: format!("Where is {}?", PEOPLE[person]), answer: PLACES[last[person].unwrap()].to_string() });
    }
    out
}

#[test]
fn a_rule_from_made_lessons() {
    let lessons = made_cases(300, 7);
    let read = made_cases(200, 11);
    let learner = PatternLearner::learn(&lessons, 4);
    assert_eq!(learner.model.order.len(), 1);
    for case in &read {
        let said = learner.answer(&case.sentences, &case.question);
        assert_eq!(said.words, [case.answer.clone()]);
        assert!(said.note.ends_with("list"));
    }
}

#[test]
fn the_same_answers_on_any_number_of_threads() {
    let lessons = made_cases(400, 3);
    let read = made_cases(100, 5);
    let alone = PatternLearner::learn(&lessons, 1);
    for lanes in [2, 8, 24] {
        let many = PatternLearner::learn(&lessons, lanes);
        assert_eq!(many.model.table.parent.len(), alone.model.table.parent.len());
        for case in &read {
            assert_eq!(many.answer(&case.sentences, &case.question), alone.answer(&case.sentences, &case.question));
        }
    }
}

#[test]
fn new_names_take_the_kind_of_their_place() {
    let lessons = made_cases(300, 13);
    let learner = PatternLearner::learn(&lessons, 4);
    let sentences = vec!["Zork went to the attic.".to_string(), "Mary moved to the garden.".to_string(), "Zork journeyed to the cellar.".to_string()];
    assert_eq!(learner.answer(&sentences, "Where is Zork?").words, ["cellar"]);
}

#[test]
fn speed_of_learning_and_answering() {
    let lessons = made_cases(5000, 17);
    let read = made_cases(5000, 19);
    let started = Instant::now();
    let learner = PatternLearner::learn(&lessons, std::thread::available_parallelism().map(usize::from).unwrap_or(1));
    let learn = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let right = read.iter().filter(|case| learner.answer(&case.sentences, &case.question).words == [case.answer.clone()]).count();
    let answer = started.elapsed().as_secs_f64();
    println!("pattern learner: {} lessons learned in {learn:.3} s ({:.0} lessons a second), {} cases answered on one thread in {answer:.3} s ({:.0} cases a second), {right} right", lessons.len(), lessons.len() as f64 / learn, read.len(), read.len() as f64 / answer);
    assert_eq!(right, read.len());
}

fn long_story(people: usize) -> (Vec<String>, Vec<(String, String)>) {
    let mut sentences = Vec::new();
    let mut last = Vec::new();
    for person in 0..people {
        let name = format!("Person{person}");
        let mut place = "";
        for step in 0..2 {
            place = PLACES[(person * 5 + step * 3) % PLACES.len()];
            sentences.push(format!("{name} went to the {place}."));
        }
        last.push((name, place.to_string()));
    }
    (sentences, last)
}

#[test]
fn a_long_story_read_once_gives_the_answers_of_the_model() {
    let learner = PatternLearner::learn(&made_cases(300, 23), 4);
    let (sentences, last) = long_story(40);
    assert!(IndexedStory::read(&learner, &sentences).short());
    let mut story = read_story(&learner, &sentences);
    for (name, place) in &last {
        let question = format!("Where is {name}?");
        let said = story.ask(&question);
        assert_eq!(said, learner.answer(&sentences, &question));
        assert_eq!(said.words, [place.clone()]);
    }
    let question = "Where is Nobody?";
    assert_eq!(story.ask(question), learner.answer(&sentences, question));
}

#[test]
fn a_frozen_table_of_longer_patterns_gives_what_the_parts_gave() {
    let mut table = ChildTable::default();
    let mut seed = 29u64;
    let mut keys: Vec<(u32, u32)> = Vec::new();
    for number in 0..20000u32 {
        let parent = if number % 7 == 0 { u32::MAX } else { (next(&mut seed) % 3000) as u32 };
        let step = (next(&mut seed) % 500) as u32;
        if table.get(parent, step).is_none() {
            table.insert(parent, step, number);
            keys.push((parent, step));
        }
    }
    let asked: Vec<(u32, u32)> = keys.iter().copied().chain((0..20000).map(|_| ((next(&mut seed) % 3100) as u32, (next(&mut seed) % 520) as u32))).collect();
    let before: Vec<Option<u32>> = asked.iter().map(|(parent, step)| table.get(*parent, *step)).collect();
    table.freeze();
    let after: Vec<Option<u32>> = asked.iter().map(|(parent, step)| table.get(*parent, *step)).collect();
    assert_eq!(before, after);
    assert!(before.iter().filter(|found| found.is_none()).count() > 1000);
    let steps: Vec<u32> = (0..520).collect();
    let mut found: Vec<Option<u32>> = Vec::new();
    for parent in [u32::MAX, 0, 17, 2999, 3050] {
        table.get_all(parent, &steps, &mut found);
        assert_eq!(found, steps.iter().map(|step| table.get(parent, *step)).collect::<Vec<Option<u32>>>());
    }
    let mut empty = ChildTable::default();
    empty.freeze();
    assert_eq!(empty.get(1, 2), None);
}

const ROOMS: [&str; 8] = ["kitchen", "garden", "office", "living room", "dining hall", "front yard", "guest room", "hallway"];

fn room_cases(count: usize, seed: u64) -> Vec<Case> {
    let mut seed = seed;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let person = next(&mut seed) % PEOPLE.len();
        let first = next(&mut seed) % ROOMS.len();
        let then = next(&mut seed) % ROOMS.len();
        let verb = MOVES[next(&mut seed) % MOVES.len()];
        let sentences = vec![format!("{} {verb} the {}.", PEOPLE[person], ROOMS[first]), format!("{} {verb} the {}.", PEOPLE[person], ROOMS[then])];
        out.push(Case { sentences, question: format!("Where is {}?", PEOPLE[person]), answer: ROOMS[then].to_string() });
    }
    out
}

#[test]
fn runs_of_words_get_the_same_numbers_in_every_build() {
    let lessons = room_cases(300, 5);
    let build = || {
        let mut symbols = Symbols::new();
        let words = PatternWords::learn(&lessons, &mut symbols);
        let mut runs: Vec<(Vec<u32>, u32)> = words.runs.iter().map(|(run, joined)| (run.clone(), *joined)).collect();
        runs.sort_unstable();
        (symbols.names().to_vec(), runs)
    };
    let first = build();
    assert!(first.1.len() >= 2, "runs found: {:?}", first.1);
    for _ in 0..6 {
        assert_eq!(build(), first);
    }
}

fn hash_by_bytes(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0;
    let mut word: u64 = 0;
    let mut held: u32 = 0;
    for byte in bytes {
        word |= u64::from(*byte) << held;
        held += 8;
        if held == u64::BITS {
            hash = mix(hash, word);
            word = 0;
            held = 0;
        }
    }
    if held > 0 {
        hash = mix(hash, word);
    }
    hash ^ (hash >> 32)
}

#[test]
fn cheap_hash_of_bytes_is_the_hash_of_their_words() {
    let mut seed = 11;
    for size in 0..40 {
        let bytes: Vec<u8> = (0..size).map(|_| (next(&mut seed) % 256) as u8).collect();
        let mut hasher = CheapHasher::default();
        hasher.write(&bytes);
        assert_eq!(hasher.finish(), hash_by_bytes(&bytes), "{size} bytes");
    }
    let row: Vec<u32> = (0..9).map(|_| next(&mut seed) as u32).collect();
    let mut table: CheapMap<Vec<u32>, usize> = CheapMap::default();
    table.insert(row.clone(), 1);
    assert_eq!(table.get(row.as_slice()), Some(&1));
}
