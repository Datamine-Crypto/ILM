use crate::base::symbols::Symbols;
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::holes::holed_groups;
use crate::tools::pattern::names::{EntryOrder, text_order};
use patterns::{because, source};
use std::collections::HashSet;

pub struct PlaceKinds;
source!(PlaceKinds, "the distributional method of Harris: two words are of one kind when they stand in the same place of the same text, and also when their texts are the same up to the kinds of the other words; a kind has the name of its most frequent word");

pub struct ClassUnion;
source!(ClassUnion, "the disjoint sets of Galler and Fischer with the halving of paths of Tarjan and van Leeuwen: the classes of words that were found in one place are joined in near constant time for each join");

pub const ABSENT: u32 = u32::MAX;
because!(ABSENT, PlaceKinds, "the mark of a place that holds no word and of a word that has no kind");

pub const KIND_MARK: u32 = 0x8000_0000;
because!(KIND_MARK, PlaceKinds, "the bit that says that a place of a form holds the number of a kind and not the number of a word; words stay below it");

pub const MOST_RARE: u32 = 2;
because!(MOST_RARE, PlaceKinds, "a word that stands in this many texts at most is a rare word: numbers and names that never come again get their kind from the frame of their text");

pub const LEAST_FRAME: usize = 3;
because!(LEAST_FRAME, PlaceKinds, "a frame of rare words gives kinds when this many texts or more have it");

pub const KIND_ROUNDS: usize = 8;
because!(KIND_ROUNDS, PlaceKinds, "the most rounds in which kinds are joined through the kinds of the other words, and in which runs are dropped: the classes settle in a few rounds, and the limit bounds the work for a set where they do not");

#[derive(Clone, Debug, Default)]
pub struct Classes {
    parent: CheapMap<u32, u32>,
}
because!(Classes, ClassUnion, "the classes of words: root gives the class of a word and takes the word in when it is new, join makes one class of two, known says that a word was taken in, class gives the class of such a word and nothing for another word, members gives every word that was taken in");

impl Classes {
    pub fn root(&mut self, word: u32) -> u32 {
        let mut at = word;
        loop {
            let up = *self.parent.entry(at).or_insert(at);
            if up == at {
                return at;
            }
            let over = *self.parent.get(&up).unwrap_or(&up);
            self.parent.insert(at, over);
            at = over;
        }
    }

    pub fn join(&mut self, word: u32, first: u32) -> bool {
        let a = self.root(word);
        let b = self.root(first);
        if a == b {
            return false;
        }
        self.parent.insert(a, b);
        true
    }

    pub fn known(&self, word: u32) -> bool {
        self.parent.contains_key(&word)
    }

    pub fn class(&self, word: u32) -> Option<u32> {
        let mut at = *self.parent.get(&word)?;
        loop {
            let up = *self.parent.get(&at).unwrap_or(&at);
            if up == at {
                return Some(at);
            }
            at = up;
        }
    }

    pub fn members(&self) -> Vec<u32> {
        self.parent.keys().copied().collect()
    }
}

pub fn sorted_words<S: std::hash::BuildHasher>(symbols: &Symbols, found: &HashSet<u32, S>) -> Vec<u32> {
    let mut out: Vec<u32> = found.iter().copied().collect();
    out.sort_by(|a, b| symbols.name(*a).cmp(symbols.name(*b)));
    out
}
because!(sorted_words, EntryOrder, "the words of a set in the order of their texts, so that the first word of a class is the same in every run");

pub fn sorted_texts<S: std::hash::BuildHasher>(symbols: &Symbols, found: &HashSet<Vec<u32>, S>) -> Vec<Vec<u32>> {
    let mut out: Vec<Vec<u32>> = found.iter().cloned().collect();
    out.sort_by(|a, b| text_order(symbols, a, b));
    out
}
because!(sorted_texts, EntryOrder, "the texts of a set in the order of their words, so that every table that is filled from them has one order in every run");

pub fn join_sorted(symbols: &Symbols, classes: &mut Classes, found: &[u32]) -> bool {
    if found.len() <= 1 {
        return false;
    }
    let mut order = found.to_vec();
    order.sort_by(|a, b| symbols.name(*a).cmp(symbols.name(*b)));
    let mut changed = false;
    for word in &order[1..] {
        changed |= classes.join(*word, order[0]);
    }
    changed
}
because!(join_sorted, PlaceKinds, ClassUnion, "the words that fill one place become one class, each joined to the first of them in the order of their texts; it says if a class changed");

pub fn word_classes(symbols: &Symbols, texts: &[Vec<u32>], pairs: &[(u32, u32)]) -> Classes {
    let places = holed_groups(texts, texts);
    let size = texts.iter().flatten().max().map_or(0, |top| *top as usize + 1);
    let mut often: Vec<u32> = vec![0; size];
    let mut last: Vec<u32> = vec![ABSENT; size];
    for (number, text) in texts.iter().enumerate() {
        for word in text {
            if last[*word as usize] != number as u32 {
                last[*word as usize] = number as u32;
                often[*word as usize] += 1;
            }
        }
    }
    let rare = |word: &u32| often[*word as usize] <= MOST_RARE;
    let mut frames: FirstSightTable<Vec<u32>, Vec<usize>> = FirstSightTable::new();
    for (number, text) in texts.iter().enumerate() {
        if text.iter().any(rare) {
            let frame: Vec<u32> = text.iter().map(|word| if rare(word) { ABSENT } else { *word }).collect();
            frames.entry(frame, Vec::new()).push(number);
        }
    }
    let mut classes = Classes::default();
    for found in &places {
        join_sorted(symbols, &mut classes, found);
    }
    let mut found: Vec<u32> = Vec::new();
    for (frame, group) in frames.rows() {
        if group.len() >= LEAST_FRAME {
            for (place, word) in frame.iter().enumerate() {
                if *word == ABSENT {
                    found.clear();
                    for number in group {
                        let held = texts[*number][place];
                        if !found.contains(&held) {
                            found.push(held);
                        }
                    }
                    join_sorted(symbols, &mut classes, &found);
                }
            }
        }
    }
    for (a, b) in pairs {
        if a != b {
            let (first, other) = if symbols.name(*a) <= symbols.name(*b) { (*a, *b) } else { (*b, *a) };
            classes.join(other, first);
        }
    }
    for _ in 0..KIND_ROUNDS {
        let shapes: Vec<Vec<u32>> = texts.iter().map(|text| text.iter().map(|word| classes.class(*word).unwrap_or(*word)).collect()).collect();
        let mut changed = false;
        for found in &holed_groups(&shapes, texts) {
            changed |= join_sorted(symbols, &mut classes, found);
        }
        if !changed {
            break;
        }
    }
    classes
}
because!(word_classes, PlaceKinds, ClassUnion, "the classes of the words of a set of texts: words that fill one place of one text, rare words that fill one place of one frame, the given pairs, and then, round by round, words that fill one place of texts that are the same up to the classes of their other words; a word with no other word of its class is in no class");
