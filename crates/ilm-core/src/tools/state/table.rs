use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::hash::CheapMap;
use patterns::{because, source};
use std::sync::Arc;

pub struct StateTable;
source!(StateTable, "the state of a story as one small table, thing to where it is, changed by the units of the story in their order, in the way of the world models of the tasks of Weston and others: a holder has one value, the set of a holder is all things whose value is the holder, and a holder can keep a copy of the table as it was, as a person who saw a scene keeps it in mind");

pub const OPERATION_WORK: u64 = 20;
because!(OPERATION_WORK, StateTable, "the work of one operation on the table, counted as this many units that change nothing: the count of work stops a search at the same point on every machine, so its parts keep their weight");

pub const TABLE_WORK: u64 = 2;
because!(TABLE_WORK, StateTable, "the work of each entry of the table at the end of a story, for the readings that look at it");

pub const TOP_STEPS: usize = 8;
because!(TOP_STEPS, StateTable, "the longest chain of values that is followed to its end: a thing in a box in a room in a house, and a bound for a chain that comes back to its start");

pub const SPAN_ROOM: usize = 6;
because!(SPAN_ROOM, StateTable, "the longest span of tokens that names a thing of the table: the longest span of an answer");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Op {
    Set,
    Drop,
    Release,
    Keep,
    Forget,
    Swap,
    SwapHolders,
}
because!(Op, StateTable, "the operations on the table: set the value of a holder; drop the value of a thing; release a thing to where its holder is; keep a copy of the table; forget the copy; swap the values of two holders; swap two holders with all their things");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Holder {
    Span(u8, u8),
    Each(u32),
    Last,
}
because!(Holder, StateTable, "what an operation is done on: a span of tokens of the unit, each word of a kind in the unit, or the holders of the last unit that did an operation, which a word such as she or they names");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OpChoice {
    pub op: Op,
    pub first: Holder,
    pub second: Option<(u8, u8)>,
}
because!(OpChoice, StateTable, "one choice for a form of unit: the operation, what it is done on, and the span of the value or of the second holder");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpStep {
    pub op: Op,
    pub first: Option<(u8, u8)>,
    pub second: Option<(u8, u8)>,
}
because!(OpStep, StateTable, "one simple operation of a choice: the operation, the span that it is done on or nothing for the last holders, and the span of the value");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Key {
    Form(u32),
    Word(u32, u32),
}
because!(Key, StateTable, "an entry of a program: a form of unit, or a form of unit with one word in the slot where the form is split");

#[derive(Clone, Debug, Default)]
pub struct Prog {
    pub ops: CheapMap<u32, Arc<Vec<OpStep>>>,
    pub sub_at: CheapMap<u32, usize>,
    pub subs: CheapMap<(u32, u32), Arc<Vec<OpStep>>>,
    pub said: CheapMap<Key, Option<OpChoice>>,
}
because!(Prog, StateTable, "a program: for each form of unit its operations; a form can be split by the word in one of its slots, and a word of a split form has operations of its own; said holds the choice of each entry. put gives the program with one entry changed, without the program with one entry gone");

impl OpChoice {
    pub fn text(&self, reader: &crate::tools::state::reader::StateReader, _shape: &[u32]) -> String {
        let op = match self.op {
            Op::Set => "set",
            Op::Drop => "drop",
            Op::Release => "release",
            Op::Keep => "keep",
            Op::Forget => "forget",
            Op::Swap => "swap",
            Op::SwapHolders => "swap_holders",
        };
        let first = match self.first {
            Holder::Span(from, to) => format!("{from}-{to}"),
            Holder::Each(kind) => format!("each {}", reader.tables.kind_names[kind as usize]),
            Holder::Last => "last".to_string(),
        };
        match self.second {
            Some((from, to)) => format!("{op} {first} {from}-{to}"),
            None => format!("{op} {first}"),
        }
    }
}

impl Prog {
    pub fn put(&self, key: Key, choice: Option<OpChoice>, steps: Arc<Vec<OpStep>>) -> Prog {
        let mut new = self.clone();
        match (key, choice) {
            (Key::Form(form), None) => {
                new.ops.remove(&form);
                new.said.remove(&key);
            }
            (Key::Word(form, word), None) => {
                new.subs.insert((form, word), Arc::new(Vec::new()));
                new.said.insert(key, None);
            }
            (Key::Form(form), Some(_)) => {
                new.ops.insert(form, steps);
                new.said.insert(key, choice);
            }
            (Key::Word(form, word), Some(_)) => {
                new.subs.insert((form, word), steps);
                new.said.insert(key, choice);
            }
        }
        new
    }

    pub fn without(&self, key: Key) -> Prog {
        let mut new = self.clone();
        match key {
            Key::Form(form) => {
                new.ops.remove(&form);
            }
            Key::Word(form, word) => {
                new.subs.remove(&(form, word));
            }
        }
        new.said.remove(&key);
        if let Key::Word(form, _) = key {
            if !new.subs.keys().any(|(held, _)| *held == form) {
                new.sub_at.remove(&form);
            }
        }
        new
    }
}

#[derive(Clone, Debug)]
pub struct Coded {
    pub fid: u32,
    pub tokens: Vec<u32>,
    pub spans: Vec<u32>,
    pub raws: Vec<u32>,
}
because!(Coded, StateTable, "a unit with the number of each of its spans of tokens and of the words as written of each span, so that the table is changed with numbers alone: the span from a token over a given length stands at the place of the token times the longest span plus the length less one");

impl Coded {
    pub fn span(&self, from: u8, to: u8) -> u32 {
        self.spans[from as usize * SPAN_ROOM + (to - from) as usize - 1]
    }

    pub fn raw(&self, from: u8, to: u8) -> u32 {
        self.raws[from as usize * SPAN_ROOM + (to - from) as usize - 1]
    }
}

#[derive(Debug)]
pub struct StateKept {
    pub at: Vec<(u32, u32)>,
    pub kept: Vec<(u32, Arc<StateKept>)>,
    pub time: usize,
}
because!(StateKept, StateTable, "a copy of the table that a holder keeps: the table as it was, the copies that other holders kept at that time, and a number that grows with time");

#[derive(Clone, Debug, Default)]
pub struct WorldState {
    pub at: Vec<(u32, u32)>,
    pub first: CheapMap<(u32, u32), u32>,
    pub kept: Vec<(u32, Arc<StateKept>)>,
    pub raw: Vec<(u32, u32)>,
    raw_at: CheapMap<u32, u32>,
    pub work: u64,
}
because!(WorldState, StateTable, "the table after a story: each thing with its value in the order in which it got it, the number of the first set of each pair, the copies that holders keep, the words as written of each span that an operation named with the place of each span in that list, and the work of the run. name_raw puts the words as written of a span in: a span that the list holds keeps its place and its length stays");

impl WorldState {
    pub fn name_raw(&mut self, key: u32, raw: u32) {
        match self.raw_at.get(&key) {
            Some(place) => self.raw[*place as usize].1 = raw,
            None => {
                self.raw_at.insert(key, self.raw.len() as u32);
                self.raw.push((key, raw));
            }
        }
    }
}

pub fn get(table: &[(u32, u32)], key: u32) -> u32 {
    table.iter().find(|(held, _)| *held == key).map_or(ABSENT, |(_, value)| *value)
}
because!(get, StateTable, "the value of a thing in a table, and nothing when it has none");

fn remove(table: &mut Vec<(u32, u32)>, key: u32) {
    if let Some(place) = table.iter().position(|(held, _)| *held == key) {
        table.remove(place);
    }
}
because!(remove, StateTable, "a thing leaves a table, and the order of the other things stays");

fn assign(table: &mut Vec<(u32, u32)>, key: u32, value: u32) {
    match table.iter_mut().find(|(held, _)| *held == key) {
        Some(entry) => entry.1 = value,
        None => table.push((key, value)),
    }
}
because!(assign, StateTable, "a thing gets a value: it keeps its place in the order when it had one, else it comes last");

pub fn kept_of(kept: &[(u32, Arc<StateKept>)], holder: u32) -> Option<&Arc<StateKept>> {
    kept.iter().find(|(held, _)| *held == holder).map(|(_, copy)| copy)
}
because!(kept_of, StateTable, "the copy that a holder keeps, when it keeps one");

pub fn top(table: &[(u32, u32)], key: u32) -> u32 {
    let mut value = get(table, key);
    if value == ABSENT {
        return ABSENT;
    }
    for _ in 0..TOP_STEPS {
        let next = get(table, value);
        if next == ABSENT || next == key {
            break;
        }
        value = next;
    }
    value
}
because!(top, StateTable, "the end of the chain of values that starts at a thing, and nothing when the thing has no value");

#[derive(Default)]
pub struct StoryRun {
    pub state: WorldState,
    pub hist: Vec<Vec<u32>>,
    last: Vec<u32>,
    named: Vec<u32>,
}
because!(StoryRun, StateTable, "the tables of a run of a story, kept from one run to the next on one thread so that a run takes no new memory: the table, the values that each watched thing had, and the holders of the last unit and of this unit. run fills them for a story under a program; the number of the first set of each pair is kept only when it is asked for, because only a reading that orders things by it reads it");

pub fn run_story(units: &[Coded], prog: &Prog, watch: &[u32]) -> (WorldState, Vec<Vec<u32>>) {
    let mut run = StoryRun::default();
    run.run(units, prog, watch, true);
    (run.state, run.hist)
}
because!(run_story, StateTable, "the table after all units of a story under a program, and for each watched thing the values that the end of its chain had, in order, in tables of their own");

impl StoryRun {
    pub fn run(&mut self, units: &[Coded], prog: &Prog, watch: &[u32], firsts: bool) {
        let StoryRun { state, hist, last, named } = self;
        state.at.clear();
        state.first.clear();
        state.kept.clear();
        state.raw.clear();
        state.raw_at.clear();
        hist.truncate(watch.len());
        for values in hist.iter_mut() {
            values.clear();
        }
        hist.resize(watch.len(), Vec::new());
        last.clear();
        let mut work = units.len() as u64;
        for unit in units {
            let mut steps = prog.ops.get(&unit.fid);
            if let Some(place) = if prog.sub_at.is_empty() { None } else { prog.sub_at.get(&unit.fid) } {
                if let Some(found) = prog.subs.get(&(unit.fid, unit.tokens[*place])) {
                    steps = Some(found);
                }
            }
            let Some(steps) = steps.filter(|steps| !steps.is_empty()) else { continue };
            named.clear();
            for step in steps.iter() {
                let one: [u32; 1];
                let keys: &[u32] = match step.first {
                    None => last,
                    Some((from, to)) => {
                        let key = unit.span(from, to);
                        state.name_raw(key, unit.raw(from, to));
                        one = [key];
                        &one
                    }
                };
                let mut value = ABSENT;
                if let Some((from, to)) = step.second {
                    value = unit.span(from, to);
                    state.name_raw(value, unit.raw(from, to));
                }
                work += OPERATION_WORK * keys.len() as u64;
                for key in keys {
                    let key = *key;
                    match step.op {
                        Op::Set => {
                            remove(&mut state.at, key);
                            state.at.push((key, value));
                            if firsts {
                                let next = state.first.len() as u32;
                                state.first.entry((key, value)).or_insert(next);
                            }
                            if !state.kept.is_empty() {
                                if let Some(place) = state.kept.iter().position(|(held, copy)| *held == key && get(&copy.at, key) == value) {
                                    state.kept.remove(place);
                                }
                            }
                        }
                        Op::Drop => remove(&mut state.at, key),
                        Op::Release => {
                            let holder = get(&state.at, key);
                            if holder != ABSENT {
                                let up = get(&state.at, holder);
                                if up == ABSENT {
                                    remove(&mut state.at, key);
                                } else {
                                    assign(&mut state.at, key, up);
                                }
                            }
                        }
                        Op::Keep => {
                            let copy = Arc::new(StateKept { at: state.at.clone(), kept: state.kept.clone(), time: state.raw.len() });
                            match state.kept.iter_mut().find(|(held, _)| *held == key) {
                                Some(entry) => entry.1 = copy,
                                None => state.kept.push((key, copy)),
                            }
                            work += (state.at.len() + state.kept.len()) as u64;
                        }
                        Op::Forget => {
                            if let Some(place) = state.kept.iter().position(|(held, _)| *held == key) {
                                state.kept.remove(place);
                            }
                        }
                        Op::Swap => {
                            let (mine, theirs) = (get(&state.at, key), get(&state.at, value));
                            if theirs == ABSENT {
                                remove(&mut state.at, key);
                            } else {
                                assign(&mut state.at, key, theirs);
                            }
                            if mine == ABSENT {
                                remove(&mut state.at, value);
                            } else {
                                assign(&mut state.at, value, mine);
                            }
                        }
                        Op::SwapHolders => {
                            work += state.at.len() as u64;
                            for entry in state.at.iter_mut() {
                                if entry.1 == key {
                                    entry.1 = value;
                                } else if entry.1 == value {
                                    entry.1 = key;
                                }
                            }
                        }
                    }
                }
                named.extend_from_slice(keys);
            }
            std::mem::swap(last, named);
            for (place, key) in watch.iter().enumerate() {
                let mut found = get(&state.at, *key);
                if found == ABSENT {
                    continue;
                }
                for _ in 0..TOP_STEPS {
                    let next = get(&state.at, found);
                    if next == ABSENT || next == *key {
                        break;
                    }
                    found = next;
                }
                if hist[place].last() != Some(&found) {
                    hist[place].push(found);
                }
            }
        }
        state.work = work + TABLE_WORK * state.at.len() as u64;
    }
}

pub fn names_thing(state: &WorldState, key: u32) -> bool {
    state.at.iter().any(|(held, value)| *held == key || *value == key) || state.kept.iter().any(|(holder, _)| *holder == key)
}
because!(names_thing, StateTable, "whether the table after a story names a thing: as a thing with a value, as a value, or as a holder that keeps a copy");
