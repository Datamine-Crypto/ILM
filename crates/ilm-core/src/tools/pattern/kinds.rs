use crate::tools::pattern::classes::{KIND_MARK, ABSENT, PlaceKinds};
use crate::tools::pattern::hash::{CheapMap, CheapSet};
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::{WordTexts, UnseenWords};
use patterns::{because, source};
use std::collections::HashMap;
use std::hash::BuildHasher;

pub struct JoinedForms;
source!(JoinedForms, "the substitution test of Harris for a constituent: a run of words that stands where one word can stand is read as one word; and a plural and its singular are one word when the texts hold both");

pub const MOST_RUN: usize = 4;
because!(MOST_RUN, JoinedForms, "the longest run of words that is read as one word: the names of people and things are not longer, and each word more makes one more search at every place of every text");

pub const PAIR_RUN: usize = 2;
because!(PAIR_RUN, JoinedForms, "the words of the shortest run, and of a name made of a fixed word and one word of a kind");

pub const JOINT: &str = "_";
because!(JOINT, JoinedForms, "the mark between the words of a run that is read as one word: no word of a text holds it, so a joined word is never taken for a word of a text");

pub struct WordBits;
source!(WordBits, "the facts about a word that every case asks for are bits of one byte in an array by word numbers, so a test of a word is one load and one mask and not a search in a table");

pub const IN_LESSONS: u8 = 1;
because!(IN_LESSONS, WordBits, "the bit of a word that the texts of the lessons hold; a word without it is a new word");

pub const STARTS_NAMES: u8 = 2;
because!(STARTS_NAMES, WordBits, JoinedForms, "the bit of a fixed word that starts names of more than one word in the lessons, as an article does");

pub const ENDS_NAMES: u8 = 4;
because!(ENDS_NAMES, WordBits, JoinedForms, "the bit of a fixed word that ends names of more than one word in the lessons");

pub fn put(row: &mut Vec<u32>, word: u32, value: u32) {
    let place = word as usize;
    if row.len() <= place {
        row.resize(place + 1, ABSENT);
    }
    row[place] = value;
}
because!(put, PlaceKinds, "one value of a table that is an array by word numbers; the array grows to the word, and a word that was never put has no value");

pub fn seen_as(row: &[u32], word: u32) -> u32 {
    match row.get(word as usize) {
        Some(base) if *base != ABSENT => *base,
        _ => word,
    }
}
because!(seen_as, JoinedForms, "the word that a word is read as, and the word itself when the table has no other word for it");

pub fn joined_text<N: WordTexts>(names: &N, run: &[u32]) -> String {
    run.iter().map(|word| names.name(*word)).collect::<Vec<&str>>().join(JOINT)
}
because!(joined_text, JoinedForms, "the text of a run of words that is read as one word: its words with the mark of a run between them");

#[derive(Clone, Debug, Default)]
pub struct PatternWords {
    pub read_as: Vec<u32>,
    pub runs: CheapMap<Vec<u32>, u32>,
    pub run_parts: CheapMap<u32, Vec<u32>>,
    pub kinds: Vec<u32>,
    pub kind_names: Vec<String>,
    pub answer_kinds: Vec<u32>,
    pub flags: Vec<u8>,
    pub members: Vec<Vec<u32>>,
    pub shapes: CheapMap<usize, FirstSightTable<Vec<u32>, u32>>,
    pub sent_shapes: CheapMap<usize, CheapSet<Vec<u32>>>,
    pub answer_forms: CheapMap<u32, FirstSightTable<u32, u32>>,
    pub answer_changes: Vec<((String, String), u32)>,
}
because!(PatternWords, PlaceKinds, JoinedForms, UnseenWords, "the words of a set of lessons as the pattern learner reads them, in arrays by word numbers: which word is read as which (a plural as its singular), the runs of words that are one word, the kind of each word that has one, the kinds of the answers, the bits of each word (it stands in the lessons, it starts or ends names), the words of each kind, the forms of the lesson texts and of the told sentences, and how the answers write a word. learn makes it from lessons; norm, join and read turn the words of a text into the words that are counted; lesson_form, new_kinds and new_names give kinds to the words of a case that no lesson held");

impl PatternWords {
    pub fn norm(&self, word: u32) -> u32 {
        seen_as(&self.read_as, seen_as(&self.read_as, word))
    }

    pub fn kind_of(&self, word: u32) -> u32 {
        self.kinds.get(word as usize).copied().unwrap_or(ABSENT)
    }

    pub fn has(&self, word: u32, flag: u8) -> bool {
        self.flags.get(word as usize).is_some_and(|bits| bits & flag != 0)
    }

    pub fn flag(&mut self, word: u32, flag: u8) {
        let place = word as usize;
        if self.flags.len() <= place {
            self.flags.resize(place + 1, 0);
        }
        self.flags[place] |= flag;
    }

    pub fn kind_with(&self, word: u32, known: &[(u32, u32)]) -> u32 {
        new_kind(known, word).unwrap_or_else(|| self.kind_of(word))
    }

    pub fn shape_at(&self, word: u32, known: &[(u32, u32)]) -> u32 {
        let kind = self.kind_with(word, known);
        if kind == ABSENT { word } else { KIND_MARK | kind }
    }

    pub fn shape_of(&self, text: &[u32], known: &[(u32, u32)]) -> Vec<u32> {
        text.iter().map(|word| self.shape_at(*word, known)).collect()
    }

    pub fn join(&self, text: &[u32]) -> Vec<u32> {
        join_with(text, &self.runs)
    }

    pub fn norm_row<N: WordTexts>(&self, names: &mut N, found: &[String]) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::with_capacity(found.len());
        for word in found {
            let number = names.number(word);
            out.push(self.norm(number));
        }
        out
    }

    pub fn read<N: WordTexts>(&self, names: &mut N, found: &[String]) -> Vec<u32> {
        self.join(&self.norm_row(names, found))
    }
}

pub fn new_kind(known: &[(u32, u32)], word: u32) -> Option<u32> {
    known.binary_search_by_key(&word, |(new, _)| *new).ok().map(|place| known[place].1)
}
because!(new_kind, UnseenWords, "the kind of a new word of a case in the list of the new words of the case, which is sorted by word so that a look-up halves the list, and nothing for a word that the list does not hold");

pub fn set_new_kind(known: &mut Vec<(u32, u32)>, word: u32, kind: u32, again: bool) {
    match known.binary_search_by_key(&word, |(new, _)| *new) {
        Ok(place) => {
            if again {
                known[place].1 = kind;
            }
        }
        Err(place) => known.insert(place, (word, kind)),
    }
}
because!(set_new_kind, UnseenWords, "a new word with its kind in the sorted list of the new words of a case: a word that the list holds gets the new kind only when it is asked for, else it keeps the kind that it had");

pub fn join_with<S: BuildHasher>(tokens: &[u32], found: &HashMap<Vec<u32>, u32, S>) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::with_capacity(tokens.len());
    let mut at = 0;
    while at < tokens.len() {
        let mut size = MOST_RUN.min(tokens.len() - at);
        while size > 1 && !found.contains_key(&tokens[at..at + size]) {
            size -= 1;
        }
        out.push(if size > 1 { found[&tokens[at..at + size]] } else { tokens[at] });
        at += size.max(1);
    }
    out
}
because!(join_with, JoinedForms, UnseenWords, "a text with each run of a table read as one word, the longest run first at each place: the runs of the lessons, and the new names of two words or more of one case");
