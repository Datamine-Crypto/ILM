use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::codes::{ALL_SPLIT, CodedNames, DOTS, EARLIER, FREE, LABEL_MASK, LITERAL, Point, SPLIT_STEP, Scope, SlotPatterns, Step, asked_name, name_step, slot_name};
use crate::tools::pattern::hash::{CheapMap, mix};
use crate::tools::pattern::kinds::PatternWords;
use crate::tools::pattern::story::Query;
use patterns::{because, source};

pub struct RelationalPaths;
source!(RelationalPaths, "the relational pathfinding of Richards and Mooney: a later sentence of a pattern is found from a word that the question or an earlier sentence binds, so a pattern is a path of told sentences that share words");

pub struct StampedSets;
source!(StampedSets, "a set over small numbers as an array of stamps, in the way of the sparse sets of Briggs and Torczon: a member holds the stamp of the present round, and the set is emptied by a new stamp and not by a pass over the array");

pub const FIRST: u32 = 1;
because!(FIRST, RelationalPaths, "the number of the first sentence of a pattern, which is found from the question alone");

pub const SIG_SEED: u64 = 0x9e37_79b9_7f4a_7c15;
because!(SIG_SEED, StampedSets, "the start of the hash of the names that are bound in a sentence: the golden ratio in fixed point, as Knuth gives it for a multiplicative hash, so that a sentence with no bound name does not hash to its own number");

pub trait Catalog {
    fn form(&mut self, row: &[u32]) -> Option<u32>;
    fn step(&mut self, step: &Step) -> Option<u32>;
}
because!(Catalog, SlotPatterns, "the numbers of the forms of sentences and of the steps of patterns: a table that grows while a model learns, and a table that only finds while a model answers, where a form or a step that no lesson had has no number and can lead to no pattern");

#[derive(Clone, Copy, Debug, Default)]
pub struct Found {
    pub step: u32,
    pub sent: u32,
    pub from: u32,
    pub len: u32,
}
because!(Found, SlotPatterns, "one way to point at a sentence of a case: the number of the step, the number of the sentence, and where its free slots lie in the row of slots");

#[derive(Clone, Debug, Default)]
pub struct Steps {
    pub rows: Vec<Found>,
    pub slots: Vec<(u32, u32)>,
}
because!(Steps, SlotPatterns, "the ways to point at one more sentence of a case, in a fixed order, with their free slots (the name of a slot and its word) in one flat row. slots_of gives the free slots of one way");

impl Steps {
    pub fn slots_of(&self, found: &Found) -> &[(u32, u32)] {
        &self.slots[found.from as usize..(found.from + found.len) as usize]
    }
}

#[derive(Clone, Debug, Default)]
pub struct FormSet {
    sent: u32,
    sig: Vec<u32>,
    forms: Vec<(u32, u32)>,
    free: Vec<(u32, u32)>,
    next: u32,
}
because!(FormSet, SlotPatterns, "the forms of one sentence under one set of bound names: the bound name of each of its places, the numbers of its forms (with its fixed words, without them, and one for each free word kept as it stands, which names the free place that it closes), its free places as label and word, and the next set with the same hash");

pub fn stamped(set: &mut Vec<u32>, round: u32, member: u32) -> bool {
    let place = member as usize;
    if set.len() <= place {
        set.resize(place + 1, 0);
    }
    let fresh = set[place] != round;
    set[place] = round;
    fresh
}
because!(stamped, StampedSets, "a member is put into a set of stamps; it says if the member was not in the set in this round");

pub struct Finder<'a, C: Catalog> {
    pub words: &'a PatternWords,
    pub point_kinds: &'a [u32],
    pub star: u32,
    pub catalog: C,
    bound: Vec<(u32, u32)>,
    name_of: Vec<u32>,
    stamp_of: Vec<u32>,
    round: u32,
    sets: Vec<FormSet>,
    set_at: CheapMap<u64, u32>,
    sig: Vec<u32>,
    row: Vec<u32>,
    cells: Vec<usize>,
    memo: Vec<(u32, u32, u32)>,
    ranges: Vec<(u32, u32)>,
    seen_all: Vec<u32>,
    seen_before: Vec<u32>,
    seen_after: Vec<u32>,
    seen_round: u32,
    idx: Vec<u32>,
    union: Vec<u32>,
}
because!(Finder, SlotPatterns, StampedSets, "the search for the sentences of patterns in one case at a time: open starts a case, reopen starts another question of the same story and keeps the forms of its sentences, first_steps gives every way to point at a first sentence from the slots of the question, next_steps every way to point at one more sentence from any name that is bound so far or right next to the last sentence. The names that are bound, the forms of each sentence under them and the forms that were seen are kept in arrays by numbers with stamps, so that no table is built for a single call");

impl<'a, C: Catalog> Finder<'a, C> {
    pub fn over(words: &'a PatternWords, point_kinds: &'a [u32], star: u32, catalog: C) -> Finder<'a, C> {
        Finder {
            words,
            point_kinds,
            star,
            catalog,
            bound: Vec::new(),
            name_of: Vec::new(),
            stamp_of: Vec::new(),
            round: 0,
            sets: Vec::new(),
            set_at: CheapMap::default(),
            sig: Vec::new(),
            row: Vec::new(),
            cells: Vec::new(),
            memo: Vec::new(),
            ranges: Vec::new(),
            seen_all: Vec::new(),
            seen_before: Vec::new(),
            seen_after: Vec::new(),
            seen_round: 0,
            idx: Vec::new(),
            union: Vec::new(),
        }
    }

    pub fn reopen(&mut self, case: &Query) {
        self.memo.clear();
        self.memo.resize(case.story.sents.len(), (0, 0, 0));
    }

    pub fn open(&mut self, case: &Query) {
        self.sets.clear();
        self.set_at.clear();
        self.memo.clear();
        self.memo.resize(case.story.sents.len(), (0, 0, 0));
    }

    pub fn bind(&mut self, case: &Query, slots: &[(u32, u32)]) {
        self.round += 1;
        self.bound.clear();
        self.ranges.clear();
        let asked = case.slots.iter().enumerate().map(|(place, word)| (*word, asked_name(place)));
        for (word, name) in asked.chain(slots.iter().map(|(name, word)| (*word, *name))) {
            if stamped(&mut self.stamp_of, self.round, word) {
                if self.name_of.len() < self.stamp_of.len() {
                    self.name_of.resize(self.stamp_of.len(), FREE);
                }
                self.name_of[word as usize] = name;
                self.bound.push((word, name));
            }
        }
    }

    pub fn bound_name(&self, word: u32) -> u32 {
        let place = word as usize;
        if self.stamp_of.get(place) == Some(&self.round) { self.name_of[place] } else { FREE }
    }

    pub fn form_set(&mut self, case: &Query, sent: usize) -> usize {
        let told = &case.story.sents[sent];
        self.sig.clear();
        let mut hash = mix(SIG_SEED, sent as u64);
        for word in &told.held {
            let name = self.bound_name(*word);
            self.sig.push(name);
            hash = mix(hash, u64::from(name));
        }
        let mut at = self.set_at.get(&hash).copied().unwrap_or(ABSENT);
        while at != ABSENT {
            let set = &self.sets[at as usize];
            if set.sent as usize == sent && set.sig == self.sig {
                return at as usize;
            }
            at = set.next;
        }
        self.row.clear();
        self.cells.clear();
        self.row.push(told.fix);
        for (label, name) in told.labels.iter().zip(&self.sig) {
            self.row.push(*label);
            self.cells.push(self.row.len());
            self.row.push(*name);
        }
        let mut set = FormSet { sent: sent as u32, sig: self.sig.clone(), forms: Vec::new(), free: Vec::new(), next: self.set_at.get(&hash).copied().unwrap_or(ABSENT) };
        set.forms.push((self.catalog.form(&self.row).unwrap_or(ABSENT), ABSENT));
        self.row[0] = DOTS;
        set.forms.push((self.catalog.form(&self.row).unwrap_or(ABSENT), ABSENT));
        for (place, word) in told.held.iter().enumerate() {
            if self.sig[place] != FREE {
                continue;
            }
            let cell = self.cells[place];
            self.row[cell] = if *word == self.star { FREE } else { LITERAL | *word };
            set.forms.push((self.catalog.form(&self.row).unwrap_or(ABSENT), set.free.len() as u32));
            self.row[cell] = FREE;
            set.free.push((told.labels[place], *word));
        }
        self.set_at.insert(hash, self.sets.len() as u32);
        self.sets.push(set);
        self.sets.len() - 1
    }

    pub fn forms_at(&mut self, case: &Query, sent: usize, step: u32, out: &mut Steps) -> (usize, usize) {
        if self.memo[sent].0 == self.round {
            return (self.memo[sent].1 as usize, self.memo[sent].2 as usize);
        }
        let number = self.form_set(case, sent);
        let set = &self.sets[number];
        let at = self.ranges.len();
        let base = out.slots.len() as u32;
        out.slots.extend(set.free.iter().map(|(label, word)| (slot_name(step, *label), *word)));
        let count = set.free.len() as u32;
        for (form, skip) in &set.forms {
            if *skip == ABSENT || *form == ABSENT {
                self.ranges.push((base, count));
                continue;
            }
            self.ranges.push((out.slots.len() as u32, count - 1));
            let kept = set.free.iter().enumerate().filter(|(place, _)| *place as u32 != *skip);
            out.slots.extend(kept.map(|(_, (label, word))| (slot_name(step, *label), *word)));
        }
        self.memo[sent] = (self.round, number as u32, at as u32);
        (number, at)
    }

    pub fn emit(&mut self, key: Step, sent: usize, range: (u32, u32), out: &mut Steps) {
        if let Some(step) = self.catalog.step(&key) {
            out.rows.push(Found { step, sent: sent as u32, from: range.0, len: range.1 });
        }
    }

    pub fn push_forms(&mut self, case: &Query, sent: usize, step: u32, lead: Step, out: &mut Steps) {
        let (set, at) = self.forms_at(case, sent, step, out);
        for index in 0..self.sets[set].forms.len() {
            let form = self.sets[set].forms[index].0;
            if form != ABSENT {
                self.emit(Step { item: form, ..lead }, sent, self.ranges[at + index], out);
            }
        }
    }

    pub fn anchored(&mut self, case: &Query, step: u32, lead: Step, own: &[u32], idx: &[u32], out: &mut Steps) {
        let Some(last) = idx.last() else { return };
        self.push_forms(case, *last as usize, step, Step { point: Point::Any, ..lead }, out);
        let kinds = self.point_kinds;
        for kind in kinds {
            for sent in idx.iter().rev() {
                let mut found = case.story.sents[*sent as usize].of_kind(*kind).iter().filter(|word| !own.contains(word));
                let (Some(only), None) = (found.next(), found.next()) else { continue };
                let bound = self.bound_name(*only);
                let key = Step { point: Point::Kind, item: *kind, value: bound, ..lead };
                let range = if bound == FREE { (out.slots.len() as u32, 1) } else { (0, 0) };
                if bound == FREE {
                    out.slots.push((slot_name(step, LABEL_MASK), *only));
                }
                self.emit(key, *sent as usize, range, out);
                break;
            }
        }
    }

    pub fn last_of(&mut self, case: &Query, step: u32, idx: &[u32], last: Option<u32>, out: &mut Steps) {
        self.seen_round += 1;
        let round = self.seen_round;
        let lead = Step { scope: Scope::Whole, point: Point::Last, name: FREE, second: FREE, item: FREE, value: FREE };
        for sent in idx.iter().rev() {
            let (set, at) = self.forms_at(case, *sent as usize, step, out);
            for index in 0..self.sets[set].forms.len() {
                let form = self.sets[set].forms[index].0;
                if form == ABSENT {
                    continue;
                }
                let range = self.ranges[at + index];
                let fresh = stamped(&mut self.seen_all, round, form);
                let Some(last) = last else {
                    if fresh {
                        self.emit(Step { item: form, ..lead }, *sent as usize, range, out);
                    }
                    continue;
                };
                if fresh {
                    self.emit(Step { scope: Scope::All, item: form, ..lead }, *sent as usize, range, out);
                }
                let (side, scope) = if *sent < last { (&mut self.seen_before, Scope::Before) } else { (&mut self.seen_after, Scope::After) };
                if stamped(side, round, form) {
                    self.emit(Step { scope, item: form, ..lead }, *sent as usize, range, out);
                }
            }
        }
    }

    pub fn free_kinds(&mut self, case: &Query, step: u32, lead: Step, sents: &[u32], out: &mut Steps) {
        let kinds = self.point_kinds;
        for kind in kinds {
            for sent in sents.iter().rev() {
                let found = case.story.sents[*sent as usize].of_kind(*kind);
                if found.len() != 1 {
                    continue;
                }
                let bound = if lead.point == Point::NearKind { self.bound_name(found[0]) } else { FREE };
                let range = if bound == FREE { (out.slots.len() as u32, 1) } else { (0, 0) };
                if bound == FREE {
                    out.slots.push((slot_name(step, LABEL_MASK), found[0]));
                }
                self.emit(Step { item: *kind, value: bound, ..lead }, *sent as usize, range, out);
                break;
            }
        }
    }

    pub fn first_steps(&mut self, case: &Query, out: &mut Steps) {
        out.rows.clear();
        out.slots.clear();
        self.bind(case, &[]);
        let story = &case.story;
        let lead = Step { scope: Scope::Whole, point: Point::Any, name: FREE, second: FREE, item: FREE, value: FREE };
        for (place, word) in case.slots.iter().enumerate() {
            self.anchored(case, FIRST, Step { name: asked_name(place), ..lead }, &[*word], story.at(*word), out);
        }
        for (place, word) in case.slots.iter().enumerate() {
            for (later, other) in case.slots.iter().enumerate().skip(place + 1) {
                let both: Vec<u32> = story.at(*word).iter().copied().filter(|sent| story.at(*other).contains(sent)).collect();
                self.anchored(case, FIRST, Step { name: asked_name(place), second: asked_name(later), ..lead }, &[*word, *other], &both, out);
            }
        }
        if !case.hits.is_empty() {
            self.last_of(case, FIRST, &case.hits, None, out);
        }
        if case.slots.is_empty() && !story.sents.is_empty() {
            let every: Vec<u32> = (0..story.sents.len() as u32).collect();
            self.last_of(case, FIRST, &every, None, out);
            self.push_forms(case, every.len() - 1, FIRST, Step { point: Point::End, ..lead }, out);
            self.free_kinds(case, FIRST, Step { point: Point::FreeKind, ..lead }, &every, out);
        }
    }

    pub fn next_steps(&mut self, case: &Query, step: u32, chosen: &[u32], slots: &[(u32, u32)], out: &mut Steps) {
        out.rows.clear();
        out.slots.clear();
        self.bind(case, slots);
        let story = &case.story;
        let Some(last) = chosen.last().copied() else { return };
        let lead = Step { scope: Scope::All, point: Point::Any, name: FREE, second: FREE, item: FREE, value: FREE };
        let mut union = std::mem::take(&mut self.union);
        let mut idx = std::mem::take(&mut self.idx);
        union.clear();
        for place in 0..self.bound.len() {
            let (word, name) = self.bound[place];
            idx.clear();
            idx.extend(story.at(word).iter().copied().filter(|sent| !chosen.contains(sent)));
            if idx.is_empty() {
                continue;
            }
            union.extend_from_slice(&idx);
            let cut = idx.partition_point(|sent| *sent < last);
            self.anchored(case, step, Step { name, ..lead }, &[word], &idx, out);
            self.anchored(case, step, Step { scope: Scope::Before, name, ..lead }, &[word], &idx[..cut], out);
            self.anchored(case, step, Step { scope: Scope::After, name, ..lead }, &[word], &idx[cut..], out);
        }
        if !union.is_empty() {
            union.sort_unstable();
            union.dedup();
            self.last_of(case, step, &union, Some(last), out);
        }
        self.union = union;
        self.idx = idx;
        let sides = [(Scope::Next, last.checked_add(1)), (Scope::Prev, last.checked_sub(1))];
        for (scope, sent) in sides {
            let Some(sent) = sent.filter(|sent| (*sent as usize) < story.sents.len() && !chosen.contains(sent)) else { continue };
            self.push_forms(case, sent as usize, step, Step { scope, point: Point::Near, ..lead }, out);
            self.free_kinds(case, step, Step { scope, point: Point::NearKind, ..lead }, &[sent], out);
        }
    }
}

pub fn place_of(answer: &[u32], case: &Query, slots: &[(u32, u32)], more: &[(u32, u32)], out: &mut Vec<u32>) -> bool {
    out.clear();
    for word in answer {
        if let Some(place) = case.slots.iter().position(|slot| slot == word) {
            out.push(asked_name(place));
            continue;
        }
        let mut found = slots.iter().chain(more).filter(|(_, held)| held == word);
        match (found.next(), found.next()) {
            (Some(_), Some(_)) => return false,
            (Some((name, _)), None) => out.push(*name),
            _ => out.push(LITERAL | *word),
        }
    }
    true
}
because!(place_of, SlotPatterns, CodedNames, "where the words of an answer stand in a case, one part for each word: a slot of the question, a free slot of a found sentence, or the word as it stands; it says no when a word of the answer stands in two free slots, because the reading does not say which slot gives the word");

pub fn words_at(place: &[u32], case: &Query, slots: &[(u32, u32)]) -> Option<Vec<u32>> {
    let mut out: Vec<u32> = Vec::with_capacity(place.len());
    for part in place {
        if part & LITERAL != 0 {
            out.push(part & !LITERAL);
        } else if *part == ALL_SPLIT {
            out.extend(slots.iter().filter(|(name, _)| name_step(*name) == SPLIT_STEP && name & EARLIER != 0).map(|(_, word)| *word));
        } else if part & EARLIER != 0 {
            out.push(slots.iter().find(|(name, _)| name == part)?.1);
        } else {
            out.push(*case.slots.get((*part as usize).checked_sub(1)?)?);
        }
    }
    Some(out)
}
because!(words_at, SlotPatterns, CodedNames, "the words of an answer from its place in a case, and nothing when the case has no slot that a part names");
