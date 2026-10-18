use ilm_core::cases::Case;
use ilm_core::choice::{inner_parts, picked};
use ilm_core::base::shares::share;
use ilm_core::base::words::{answer_words, words};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct QuestionList;
source!(QuestionList, "the control by the words of the question alone: the decision list of the labels by words (Rivest), as the explain tool learns it over the told sentences, here over the words of the question and the pairs of its common words, learned from the first part of the lessons with no told sentence; a kind whose answers follow from the words of the question needs no reading of the text, and a learned kind must stand clearly above this control");

const LIST_SUPPORT: usize = 3;
because!(LIST_SUPPORT, QuestionList, "the fewest lessons on which an answer of the list must be right, so that a word rests on more than a chance case, as in the labels by words");

const LIST_RIGHT_PERCENTS: &[usize] = &[90, 80, 70, 60];
because!(LIST_RIGHT_PERCENTS, QuestionList, "the shares of the lessons not yet taken where its word stands that an answer of the list must answer right, tried from the strictest down, as in the labels by words; the share that answers most of the last lessons is taken");

const LIST_WHOLE: usize = 100;
because!(LIST_WHOLE, QuestionList, "hundredths, so that counts of lessons compare with a percent in whole numbers");

const MOST_LIST_ANSWERS: usize = 200;
because!(MOST_LIST_ANSWERS, QuestionList, "the most answers of the list, as in the labels by words: a longer list learns its lessons one by one");

struct WordList {
    answers: Vec<(Vec<String>, Vec<String>)>,
    otherwise: Option<Vec<String>>,
}
because!(WordList, QuestionList, "a learned list: each answer with the words of the question that it needs, in the order in which they are tried, and the answer when no words of the list stand in the question");

fn question_words(question: &str) -> HashSet<String> {
    words(question).into_iter().filter(|word| word.chars().any(char::is_alphanumeric)).collect()
}
because!(question_words, QuestionList, "the words of a question that hold a letter or a digit, each once: a mark tells no answer");

fn most_label<'a>(counts: HashMap<&'a Vec<String>, usize>) -> Option<(&'a Vec<String>, usize)> {
    counts.into_iter().max_by(|one, other| one.1.cmp(&other.1).then_with(|| other.0.cmp(one.0)))
}
because!(most_label, QuestionList, "the answer with the most lessons, and of equals the first in the order of their words, so that the same lessons give the same list in every run");

fn word_list(cases: &[Case], right_percent: usize) -> Option<WordList> {
    let labels: Vec<Vec<String>> = cases.iter().map(|case| answer_words(&case.answer)).collect();
    let distinct: HashSet<&Vec<String>> = labels.iter().collect();
    if distinct.len() < LIST_SUPPORT - 1 || distinct.len() * LIST_SUPPORT > cases.len() {
        return None;
    }
    let asked: Vec<HashSet<String>> = cases.iter().map(|case| question_words(&case.question)).collect();
    let mut by_word: HashMap<Vec<String>, Vec<usize>> = HashMap::new();
    for (at, words) in asked.iter().enumerate() {
        for word in words {
            by_word.entry(vec![word.clone()]).or_default().push(at);
        }
    }
    let common: HashSet<String> = by_word.iter().filter(|(_, lessons)| lessons.len() >= LIST_SUPPORT).flat_map(|(words, _)| words.clone()).collect();
    for (at, words) in asked.iter().enumerate() {
        let mut held: Vec<&String> = words.iter().filter(|word| common.contains(*word)).collect();
        held.sort();
        for (place, one) in held.iter().enumerate() {
            for other in &held[place + 1..] {
                by_word.entry(vec![(*one).clone(), (*other).clone()]).or_default().push(at);
            }
        }
    }
    let mut keys: Vec<(Vec<String>, Vec<usize>)> = by_word.into_iter().filter(|(_, lessons)| lessons.len() >= LIST_SUPPORT).collect();
    keys.sort_by(|one, other| one.0.len().cmp(&other.0.len()).then_with(|| one.0.cmp(&other.0)));
    let mut taken = vec![false; cases.len()];
    let mut answers: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    while answers.len() < MOST_LIST_ANSWERS {
        let mut best: Option<(usize, &Vec<String>, &Vec<String>)> = None;
        for (key, lessons) in &keys {
            let mut counts: HashMap<&Vec<String>, usize> = HashMap::new();
            let mut held = 0usize;
            for at in lessons.iter().filter(|at| !taken[**at]) {
                held += 1;
                *counts.entry(&labels[*at]).or_default() += 1;
            }
            let Some((label, right)) = most_label(counts) else { continue };
            if right < LIST_SUPPORT || right * LIST_WHOLE < held * right_percent {
                continue;
            }
            let score = right - (held - right);
            if best.is_none_or(|(known, _, _)| score > known) {
                best = Some((score, key, label));
            }
        }
        let Some((_, key, label)) = best else { break };
        for (at, words) in asked.iter().enumerate() {
            if key.iter().all(|one| words.contains(one)) {
                taken[at] = true;
            }
        }
        answers.push((label.clone(), key.clone()));
    }
    let mut rest: HashMap<&Vec<String>, usize> = HashMap::new();
    for (label, _) in labels.iter().zip(&taken).filter(|(_, taken)| !**taken) {
        *rest.entry(label).or_default() += 1;
    }
    let otherwise = most_label(rest).map(|(label, _)| label.clone());
    (!answers.is_empty()).then_some(WordList { answers, otherwise })
}
because!(word_list, QuestionList, "the list from some lessons at one trust in a word: the single words of the questions and the pairs of their common words, each taken once, the one that answers the most lessons not yet taken right less those it answers wrong first, as long as it is right on the share of the lessons where it stands; nothing for a kind whose answers are not labels (a few, each given by many lessons) or where no word tells an answer");

impl WordList {
    fn says(&self, question: &str) -> Option<&Vec<String>> {
        let asked = question_words(question);
        self.answers.iter().find(|(_, key)| key.iter().all(|word| asked.contains(word))).map(|(label, _)| label).or(self.otherwise.as_ref())
    }

    fn right(&self, cases: &[Case]) -> usize {
        cases.iter().filter(|case| self.says(&case.question) == Some(&answer_words(&case.answer))).count()
    }
}

pub fn question_list_share(learn: &[Case], read: &[Case]) -> Option<f64> {
    let (first, last) = inner_parts(learn);
    let (first_cases, last_cases) = (picked(learn, &first), picked(learn, &last));
    let (_, list) = LIST_RIGHT_PERCENTS.iter().filter_map(|percent| word_list(&first_cases, *percent).map(|list| (list.right(&last_cases), list))).fold(None, |best: Option<(usize, WordList)>, (right, list)| match best {
        Some((known, held)) if known >= right => Some((known, held)),
        _ => Some((right, list)),
    })?;
    Some(share(list.right(read), read.len()))
}
because!(question_list_share, QuestionList, "the share of the read part that the list of the words of the question answers right: learned from the first part of the lessons, at the trust in a word that answers the most of the last lessons (the strictest of equals), and read on the question of each read case alone");
