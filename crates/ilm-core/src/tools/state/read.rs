use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::kinds::{JOINT, PatternWords};
use crate::tools::pattern::names::WordTexts;
use crate::tools::state::reader::Unit;
use crate::tools::state::table::{Coded, SPAN_ROOM, WorldState, get, kept_of, top};
use patterns::{because, source};
use std::sync::Arc;

pub struct TableReadings;
source!(TableReadings, "an answer is read from the table by one of a fixed set of readings, each a question about the state: the value of a holder, the end of its chain, the members of a holder in one of three orders, their count, whether a thing has a value, the value before another, and the values in the copies that holders keep; a reading is found by search and not written for a task");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Name {
    Value,
    Top,
    Second,
    First,
    Members,
    MembersRev,
    MembersFirst,
    Count,
    Is,
    IsTop,
    IsFirst,
    Before,
    Kept,
    IsKept,
    Kept2,
    Older,
    IsKept2,
    IsOlder,
}
because!(Name, TableReadings, "the readings: value, the end of the chain, the value of the value, the first end value; members oldest first, newest first, or by their first set; the count; is, is the end, was the first end; the end value before a given one; the value in the copy that a holder keeps, whether it is a given one; the value in a copy inside a copy and in the older of two copies, and whether it is a given one");

const COPY_DEPTH: usize = 2;
because!(COPY_DEPTH, TableReadings, "the depth of the copy that the reading of a copy inside a copy reads, as its name says it");

pub const ONE_ARG: [Name; 8] = [Name::Value, Name::Top, Name::Second, Name::First, Name::Members, Name::MembersRev, Name::MembersFirst, Name::Count];
because!(ONE_ARG, TableReadings, "the readings with one argument, in their order in the list of readings");

pub const TWO_ARGS: [Name; 5] = [Name::Is, Name::IsTop, Name::IsFirst, Name::Before, Name::Kept];
because!(TWO_ARGS, TableReadings, "the readings with two arguments, in their order");

pub const THREE_ARGS: [Name; 3] = [Name::IsKept, Name::Kept2, Name::Older];
because!(THREE_ARGS, TableReadings, "the readings with three arguments, in their order");

pub const FOUR_ARGS: [Name; 2] = [Name::IsKept2, Name::IsOlder];
because!(FOUR_ARGS, TableReadings, "the readings with four arguments, in their order");

impl Name {
    pub fn text(self) -> String {
        let plain = match self {
            Name::Value => "value",
            Name::Top => "top",
            Name::Second => "second",
            Name::First => "first",
            Name::Members => "members",
            Name::MembersRev => "members_rev",
            Name::MembersFirst => "members_first",
            Name::Count => "count",
            Name::Is => "is",
            Name::IsTop => "is_top",
            Name::IsFirst => "is_first",
            Name::Before => "before",
            Name::Kept => "kept",
            Name::IsKept => "is_kept",
            Name::Kept2 => return format!("kept{COPY_DEPTH}"),
            Name::Older => "older",
            Name::IsKept2 => return format!("is_kept{COPY_DEPTH}"),
            Name::IsOlder => "is_older",
        };
        plain.to_string()
    }

    pub fn history(self) -> bool {
        matches!(self, Name::Before | Name::First | Name::IsFirst)
    }

    pub fn first_order(self) -> bool {
        matches!(self, Name::MembersFirst)
    }

    pub fn copies(self) -> bool {
        matches!(self, Name::Kept | Name::IsKept | Name::Kept2 | Name::IsKept2 | Name::Older | Name::IsOlder)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StateReading {
    pub name: Name,
    pub args: Vec<(u8, u8)>,
}
because!(StateReading, TableReadings, "a reading with its arguments, each a span of tokens of the question: a slot, or a run of slots next to each other");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StateOut {
    Unknown,
    Phrase(u32),
    Many(Arc<[u32]>),
    Count(u32),
    Truth(bool),
}
because!(StateOut, TableReadings, "the result of a reading: the table does not know; words, as the number of a known sequence of words, or the words themselves when no text had them; a count; true or false");

impl StateOut {
    pub fn is_words(&self) -> bool {
        matches!(self, StateOut::Phrase(_) | StateOut::Many(_))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Codes {
    pub spans: CheapMap<Vec<u32>, u32>,
    pub pieces: Vec<u32>,
    pub phrases: CheapMap<Vec<u32>, u32>,
    pub words: Vec<Vec<u32>>,
    pub normal: Vec<u32>,
}
because!(Codes, TableReadings, "the numbers of the spans of tokens and of the sequences of words of a set of cases: equal spans and equal words have one number each, so the table and the results are compared by numbers; each span has the words of its tokens, and each sequence of words the number of the same words in their normal form");

impl Codes {
    pub fn phrase(&mut self, first: &PatternWords, words: &[u32]) -> u32 {
        if let Some(found) = self.phrases.get(words) {
            return *found;
        }
        let number = self.words.len() as u32;
        self.phrases.insert(words.to_vec(), number);
        self.words.push(words.to_vec());
        self.normal.push(number);
        let norm: Vec<u32> = words.iter().map(|word| first.norm(*word)).collect();
        let normal = self.phrase(first, &norm);
        self.normal[number as usize] = normal;
        number
    }

    pub fn span<N: WordTexts>(&mut self, first: &PatternWords, names: &mut N, tokens: &[u32]) -> u32 {
        if let Some(found) = self.spans.get(tokens) {
            return *found;
        }
        let mut pieces: Vec<u32> = Vec::new();
        for token in tokens {
            let text = names.name(*token).to_string();
            for word in text.split(JOINT) {
                pieces.push(names.number(word));
            }
        }
        let piece = self.phrase(first, &pieces);
        let number = self.pieces.len() as u32;
        self.spans.insert(tokens.to_vec(), number);
        self.pieces.push(piece);
        number
    }

    pub fn code<N: WordTexts>(&mut self, first: &PatternWords, names: &mut N, unit: &Unit) -> Coded {
        let count = unit.tokens.len();
        let mut spans = vec![ABSENT; count * SPAN_ROOM];
        let mut raws = vec![ABSENT; count * SPAN_ROOM];
        for from in 0..count {
            let mut raw: Vec<u32> = Vec::new();
            for to in from + 1..=count.min(from + SPAN_ROOM) {
                raw.extend_from_slice(&unit.raws[to - 1]);
                spans[from * SPAN_ROOM + to - from - 1] = self.span(first, names, &unit.tokens[from..to]);
                raws[from * SPAN_ROOM + to - from - 1] = self.phrase(first, &raw);
            }
        }
        Coded { fid: unit.fid, tokens: unit.tokens.clone(), spans, raws }
    }

    pub fn out_of(&self, words: Vec<u32>) -> StateOut {
        match self.phrases.get(&words) {
            Some(found) => StateOut::Phrase(*found),
            None => StateOut::Many(words.into()),
        }
    }

    pub fn words_of<'a>(&'a self, out: &'a StateOut) -> &'a [u32] {
        match out {
            StateOut::Phrase(found) => &self.words[*found as usize],
            StateOut::Many(words) => words,
            _ => &[],
        }
    }

    pub fn normal_is(&self, first: &PatternWords, out: &StateOut, truth: u32) -> bool {
        match out {
            StateOut::Phrase(found) => self.normal[*found as usize] == truth,
            StateOut::Many(words) => {
                let want = &self.words[truth as usize];
                words.len() == want.len() && words.iter().zip(want).all(|(word, held)| first.norm(*word) == *held)
            }
            _ => false,
        }
    }
}

pub fn thing_words(state: &WorldState, codes: &Codes, key: u32) -> u32 {
    let raw = get(&state.raw, key);
    if raw == ABSENT { codes.pieces[key as usize] } else { raw }
}
because!(thing_words, TableReadings, "the words of a thing of the table: as written where an operation named it last, else its tokens cut into words");

const THIRD: usize = 2;
because!(THIRD, TableReadings, "the place of the third argument of a reading in its list of arguments");

const FOURTH: usize = 3;
because!(FOURTH, TableReadings, "the place of the fourth argument of a reading in its list of arguments");

pub fn read(reading: &StateReading, q: &Coded, state: &WorldState, hist: &[(u32, &[u32])], codes: &Codes) -> StateOut {
    let words = |key: u32| if key == ABSENT { StateOut::Unknown } else { StateOut::Phrase(thing_words(state, codes, key)) };
    let (from, to) = reading.args[0];
    let first = q.span(from, to);
    let seen = || hist.iter().find(|(held, _)| *held == first).map_or(&[][..], |(_, values)| *values);
    match reading.name {
        Name::Value => return words(get(&state.at, first)),
        Name::Top => return words(top(&state.at, first)),
        Name::Second => {
            let value = get(&state.at, first);
            return if value == ABSENT { StateOut::Unknown } else { words(get(&state.at, value)) };
        }
        Name::First => return seen().first().map_or(StateOut::Unknown, |value| words(*value)),
        _ => {}
    }
    if reading.args.len() == 1 {
        let mut found: Vec<u32> = state.at.iter().filter(|(_, value)| *value == first).map(|(key, _)| *key).collect();
        if reading.name == Name::Count || found.is_empty() {
            return StateOut::Count(found.len() as u32);
        }
        if reading.name == Name::MembersRev {
            found.reverse();
        } else if reading.name == Name::MembersFirst {
            let last = state.first.len() as u32;
            found.sort_by_key(|key| state.first.get(&(*key, first)).copied().unwrap_or(last));
        }
        if found.len() == 1 {
            return words(found[0]);
        }
        let all: Vec<u32> = found.iter().flat_map(|key| codes.words[thing_words(state, codes, *key) as usize].iter().copied()).collect();
        return codes.out_of(all);
    }
    let (from, to) = reading.args[1];
    let second = q.span(from, to);
    match reading.name {
        Name::Is => return StateOut::Truth(get(&state.at, first) == second),
        Name::IsTop => return StateOut::Truth(top(&state.at, first) == second),
        Name::IsFirst => return StateOut::Truth(seen().first() == Some(&second)),
        Name::Before => {
            let held = seen();
            for place in (1..held.len()).rev() {
                if held[place] == second {
                    return words(held[place - 1]);
                }
            }
            return StateOut::Unknown;
        }
        _ => {}
    }
    let mine = kept_of(&state.kept, first);
    if reading.name == Name::Kept {
        return words(get(mine.map_or(&state.at, |copy| &copy.at), second));
    }
    let Some((from, to)) = reading.args.get(THIRD).copied() else { return StateOut::Unknown };
    let third = q.span(from, to);
    if reading.name == Name::IsKept {
        return StateOut::Truth(get(mine.map_or(&state.at, |copy| &copy.at), second) == third);
    }
    let value = if matches!(reading.name, Name::Older | Name::IsOlder) {
        let theirs = kept_of(&state.kept, second);
        match (mine, theirs) {
            (Some(mine), Some(theirs)) => get(if mine.time <= theirs.time { &mine.at } else { &theirs.at }, third),
            (Some(mine), None) => get(&mine.at, third),
            (None, Some(theirs)) => get(&theirs.at, third),
            (None, None) => get(&state.at, third),
        }
    } else {
        let (table, others) = match mine {
            Some(copy) => (&copy.at, &copy.kept),
            None => (&state.at, &state.kept),
        };
        let theirs = kept_of(others, second);
        get(theirs.map_or(table, |copy| &copy.at), third)
    };
    if matches!(reading.name, Name::Kept2 | Name::Older) {
        return words(value);
    }
    let Some((from, to)) = reading.args.get(FOURTH).copied() else { return StateOut::Unknown };
    StateOut::Truth(value == q.span(from, to))
}
because!(read, TableReadings, "the result of a reading of the table for a question: words, a count (no member counts as none), true or false, or nothing when the table does not know, which is never a right answer");
