use crate::tools::parts::language::{Word, WordRow};
use crate::tools::parts::ordered::Kept;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct SignRewrites;
source!(SignRewrites, "a reading of a text as one word of signs that its condition changes, after the rule learners of morphological reinflection (Cotterell and others): the words of the text that the answers hold are the signs of the word, the other words are its condition, and the answer is the word with its end and its beginning replaced; the change of a lesson is found where the word and the answer share their longest run of signs. A word that the lessons hold under other conditions is changed by analogy: other words that change as it does under one of those conditions show how they change under the asked one, and of them those that share the longest ending with it decide, the change that most of them give. Else the change is the one that the lessons give most often under the condition for the longest ending of the word that they hold there, and in the same way at the beginning; when they hold no ending of it there, the change is carried over from another condition: the change of the word there (its own, or that of its longest ending held there) is the change of some words of the lessons there, and the change that most of those give under the asked condition is taken, from the condition where the word is known by the longest ending; a condition that no lesson holds is read as the condition of the lessons that shares the most of its words");

const VOTE_MAJORITY: usize = 2;
because!(VOTE_MAJORITY, SignRewrites, "a change by analogy is taken when more of the words that change alike give it than give all other changes together");

type Change = (usize, WordRow);
because!(Change, SignRewrites, "a change at one end of a word: how many signs are taken away there, and the signs put in their place");

type WordChange = (Change, Change);
because!(WordChange, SignRewrites, "the change of a whole word: the change at its end and the change at its beginning");

type ChangeTable = HashMap<(Vec<Word>, WordRow), Kept<Change, usize>>;
because!(ChangeTable, SignRewrites, "the changes at one end by the condition and the signs at that end of the word, each with how many lessons give it");

#[derive(Clone, Debug, Default)]
pub struct Rewrites {
    signs: HashSet<Word>,
    ends: ChangeTable,
    starts: ChangeTable,
    conditions: Kept<Vec<Word>, usize>,
    known: HashMap<(WordRow, Vec<Word>), WordChange>,
    by_word: HashMap<WordRow, Vec<(Vec<Word>, WordChange)>>,
    by_change: HashMap<(Vec<Word>, WordChange), Vec<WordRow>>,
    by_condition: HashMap<Vec<Word>, Vec<(WordRow, WordChange)>>,
    pub kept: usize,
}
because!(Rewrites, SignRewrites, "a model of changed words: the words that are signs, the changes at the end and at the beginning by their contexts, the conditions of the lessons with how many lessons hold each, the change of each word of the lessons under each of its conditions, the conditions and changes of each word, the words of each condition and change, and the words of each condition with their changes, and how many lessons keep some signs of their word. read gives the answer of a text");

fn longest_run(word: &[Word], answer: &[Word]) -> (usize, usize, usize) {
    let mut best = (0usize, 0usize, 0usize);
    let mut lengths = vec![0usize; answer.len() + 1];
    for (at, sign) in word.iter().enumerate() {
        let mut before = 0usize;
        for (place, other) in answer.iter().enumerate() {
            let kept = lengths[place + 1];
            lengths[place + 1] = if sign == other { before + 1 } else { 0 };
            before = kept;
            let length = lengths[place + 1];
            if length > best.0 {
                best = (length, at + 1 - length, place + 1 - length);
            }
        }
    }
    best
}
because!(longest_run, SignRewrites, "the longest run of signs that a word and an answer share, with where it starts in each; of runs of one length the one that ends first in the word, and there first in the answer");

fn applied(word: &[Word], change: &WordChange) -> Option<WordRow> {
    let ((end_cut, end_put), (start_cut, start_put)) = change;
    let kept = word.len().checked_sub(*end_cut)?;
    let changed = [&word[..kept], &end_put[..]].concat();
    let rest = changed.get(*start_cut..)?;
    Some([&start_put[..], rest].concat())
}
because!(applied, SignRewrites, "a word with a change of the whole word: its end replaced, then its beginning; nothing when the word is too short for it");

impl Rewrites {
    fn split(&self, words: &[Word]) -> (WordRow, Vec<Word>) {
        let word: WordRow = words.iter().copied().filter(|sign| self.signs.contains(sign)).collect();
        let mut condition: Vec<Word> = words.iter().copied().filter(|sign| !self.signs.contains(sign)).collect();
        condition.sort_unstable();
        condition.dedup();
        (word, condition)
    }

    pub fn learn(lessons: &[(Vec<Word>, WordRow)]) -> Rewrites {
        let signs: HashSet<Word> = lessons.iter().flat_map(|(_, answer)| answer.iter().copied()).collect();
        let mut model = Rewrites { signs, ..Rewrites::default() };
        for (words, answer) in lessons {
            let (word, condition) = model.split(words);
            let (length, at, place) = longest_run(&word, answer);
            model.kept += usize::from(length > 0);
            let end_cut = word.len() - at - length;
            let end: Change = (end_cut, answer[place + length..].to_vec());
            let start: Change = (at, answer[..place].to_vec());
            for size in end_cut..=word.len() {
                model.ends.entry((condition.clone(), word[word.len() - size..].to_vec())).or_default().count(end.clone(), 1);
            }
            for size in at..=word.len() {
                model.starts.entry((condition.clone(), word[..size].to_vec())).or_default().count(start.clone(), 1);
            }
            let change: WordChange = (end, start);
            if !model.known.contains_key(&(word.clone(), condition.clone())) {
                model.known.insert((word.clone(), condition.clone()), change.clone());
                model.by_word.entry(word.clone()).or_default().push((condition.clone(), change.clone()));
                model.by_change.entry((condition.clone(), change.clone())).or_default().push(word.clone());
                model.by_condition.entry(condition.clone()).or_default().push((word.clone(), change));
            }
            model.conditions.count(condition, 1);
        }
        model
    }

    fn known_condition(&self, condition: Vec<Word>) -> Vec<Word> {
        if self.conditions.holds(&condition) {
            return condition;
        }
        let shared = |other: &Vec<Word>| other.iter().filter(|word| condition.contains(word)).count();
        self.conditions.first_most(|other, count| (shared(other), *count)).map_or(condition.clone(), |(other, _)| other.clone())
    }

    fn change_of(table: &ChangeTable, condition: &[Word], word: &[Word], at_end: bool) -> Option<(usize, Change)> {
        (0..=word.len()).rev().find_map(|size| {
            let signs = if at_end { word[word.len() - size..].to_vec() } else { word[..size].to_vec() };
            table.get(&(condition.to_vec(), signs)).and_then(|found| found.most()).map(|(change, _)| (size, change.clone()))
        })
    }

    fn word_change(&self, word: &[Word], condition: &[Word]) -> (usize, Option<WordChange>) {
        if let Some(change) = self.known.get(&(word.to_vec(), condition.to_vec())) {
            return (word.len() + 1, Some(change.clone()));
        }
        let Some((size, end)) = Rewrites::change_of(&self.ends, condition, word, true) else { return (0, None) };
        let start = Rewrites::change_of(&self.starts, condition, word, false).map_or((0, Vec::new()), |(_, change)| change);
        (size, Some((end, start)))
    }

    fn carried(&self, word: &[Word], condition: &[Word]) -> Option<WordChange> {
        let mut best: Option<((usize, usize), WordChange)> = None;
        for (other, _) in self.conditions.pairs() {
            if other.as_slice() == condition {
                continue;
            }
            let (size, Some(change)) = self.word_change(word, other) else { continue };
            if size == 0 {
                continue;
            }
            let mut targets: Kept<WordChange, usize> = Kept::default();
            for (held, held_change) in self.by_condition.get(other).into_iter().flatten() {
                if *held_change == change {
                    if let Some(target) = self.known.get(&(held.clone(), condition.to_vec())) {
                        targets.count(target.clone(), 1);
                    }
                }
            }
            if let Some((target, count)) = targets.most() {
                if best.as_ref().is_none_or(|(measure, _)| (size, count) > *measure) {
                    best = Some(((size, count), target.clone()));
                }
            }
        }
        best.map(|(_, change)| change)
    }

    fn by_votes(&self, word: &WordRow, condition: &[Word]) -> Option<WordChange> {
        let mut votes: Kept<WordChange, usize> = Kept::default();
        for (other_condition, change) in self.by_word.get(word).into_iter().flatten() {
            for other in self.by_change.get(&(other_condition.clone(), change.clone())).into_iter().flatten() {
                if other != word {
                    if let Some(target) = self.known.get(&(other.clone(), condition.to_vec())) {
                        votes.count(target.clone(), 1);
                    }
                }
            }
        }
        let (change, count) = votes.most()?;
        (count > 1 && count * VOTE_MAJORITY > votes.total()).then(|| change.clone())
    }

    pub fn read(&self, words: &[Word]) -> Option<WordRow> {
        let (word, condition) = self.split(words);
        let condition = self.known_condition(condition);
        if let Some(change) = self.known.get(&(word.clone(), condition.clone())) {
            if let Some(changed) = applied(&word, change) {
                return Some(changed);
            }
        }
        let end = Rewrites::change_of(&self.ends, &condition, &word, true);
        if let Some(change) = self.by_votes(&word, &condition) {
            if let Some(changed) = applied(&word, &change) {
                return Some(changed);
            }
        }
        if end.as_ref().is_none_or(|(size, _)| *size == 0) {
            if let Some(changed) = self.carried(&word, &condition).and_then(|change| applied(&word, &change)) {
                return Some(changed);
            }
        }
        let mut changed = word.clone();
        if let Some((_, (cut, put))) = end.filter(|(_, (cut, _))| *cut <= changed.len()) {
            changed = [&changed[..changed.len() - cut], &put[..]].concat();
        }
        if let Some((_, (cut, put))) = Rewrites::change_of(&self.starts, &condition, &word, false).filter(|(_, (cut, _))| *cut <= changed.len()) {
            changed = [&put[..], &changed[cut..]].concat();
        }
        Some(changed)
    }
}
