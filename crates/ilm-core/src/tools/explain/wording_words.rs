use crate::base::words::answer_words;
use crate::cases::Case;
use crate::tools::explain::chain::Facts;
use crate::tools::explain::text::{AskedForm, LogicPiece, MARKED_FIRST, Pattern, Topic, place_letter, read_topic, word_not_mark};
use crate::tools::explain::wording::{CONDITIONS_JOINED, LEAST_SUPPORT, LearnedForm, ListShape, QUESTION_END, SaidCase, WordingLesson, answer_keys, decision_list, form_line, lesson_of, library_writing, list_answers, majority_otherwise, missed, one_question, probed, said_case};
use crate::tools::explain::words::{word_number, word_numbers};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct HeldWords;
source!(HeldWords, "the words that the told sentences of a lesson hold, as conditions of a learned answer, chosen as a decision list over words is learned (Rivest): a word, or a pair of words that one told sentence holds, tells a label of the whole told text (the intent of a request), each answer the one that answers right the most lessons not yet taken less those that it answers wrong while it is right on the share of the list, and the last answer the most common label of the lessons that no words take; written as a question form of the library, so a person reads which word gives which label");

const HELD_TOLD: &str = " is told";
because!(HELD_TOLD, HeldWords, "the words after the place of a told sentence in a condition of the library that reads the words of that sentence");

const HELD_WORD: &str = " holds the word ";
because!(HELD_WORD, HeldWords, "the words between the place of a told sentence and a word that it holds, in a condition of the library");

pub(super) fn held_word_condition(words: &[String]) -> String {
    let sentence = place_letter(MARKED_FIRST);
    let held: Vec<String> = words.iter().map(|word| format!("{sentence}{HELD_WORD}{word}")).collect();
    format!("{sentence}{HELD_TOLD}{CONDITIONS_JOINED}{}", held.join(CONDITIONS_JOINED))
}
because!(held_word_condition, HeldWords, "the condition that a told sentence holds some words, as a line of the library writes it");

pub(super) fn held_word_rows(sentences: &[Vec<HashSet<&String>>]) -> Vec<(Vec<String>, Vec<usize>)> {
    let mut by_words: HashMap<Vec<String>, Vec<usize>> = HashMap::new();
    for (at, lesson) in sentences.iter().enumerate() {
        let held: HashSet<&String> = lesson.iter().flatten().copied().collect();
        for word in held {
            by_words.entry(vec![word.clone()]).or_default().push(at);
        }
    }
    let common: HashSet<String> = by_words.iter().filter(|(_, lessons)| lessons.len() >= LEAST_SUPPORT).flat_map(|(words, _)| words.clone()).collect();
    for (at, lesson) in sentences.iter().enumerate() {
        let mut pairs: HashSet<(&String, &String)> = HashSet::new();
        for sentence in lesson {
            let mut held: Vec<&String> = sentence.iter().copied().filter(|word| common.contains(*word)).collect();
            held.sort();
            for (place, one) in held.iter().enumerate() {
                for other in &held[place + 1..] {
                    pairs.insert((*one, *other));
                }
            }
        }
        for (one, other) in pairs {
            by_words.entry(vec![one.clone(), other.clone()]).or_default().push(at);
        }
    }
    let mut rows: Vec<(Vec<String>, Vec<usize>)> = by_words.into_iter().filter(|(_, lessons)| lessons.len() >= LEAST_SUPPORT).collect();
    rows.sort_by(|one, other| one.0.len().cmp(&other.0.len()).then_with(|| one.0.cmp(&other.0)));
    rows
}
because!(held_word_rows, HeldWords, "the conditions on the words of the told sentences of some lessons (the words of each sentence that the caller reads), with the lessons where each holds, in their order: each word (not a mark) that enough lessons tell, and each pair of such words that one told sentence of enough lessons holds, the single words first and each in the order of the alphabet, so that of conditions as good the shorter and first one is taken");

const MOST_WORD_ANSWERS: usize = 200;
because!(MOST_WORD_ANSWERS, HeldWords, "the most answers of a list over the words of the told text: a kind has a few dozen labels and each has a few words that tell it, while a longer list learns its lessons one by one");

pub const WORD_RIGHT_PERCENTS: &[usize] = &[90, 80, 70, 60];
because!(WORD_RIGHT_PERCENTS, HeldWords, "the shares of the lessons not yet answered where its words stand that an answer of a list over the words of the told text must answer right, tried from the strictest down: a word that is wrong at times can still tell a label that no other word tells, so the share that serves a kind best is found on its last lessons");

fn word_form((form, closed_as_question): (&Pattern, bool), lessons: &[&WordingLesson], (words, lists): (&[Vec<String>], &[Vec<usize>]), right_percent: usize) -> Option<LearnedForm> {
    let form_text = library_writing(form);
    let filled: Vec<Option<Vec<bool>>> = vec![None; lists.len()];
    let shape = ListShape { most: MOST_WORD_ANSWERS, right_percent, agreed_otherwise: false };
    let (distinct, keys) = answer_keys(lessons);
    let chosen = decision_list(&keys, lists, (&filled, &vec![false; lists.len()]), shape);
    let labels = list_answers(&keys, lists, &filled, &chosen);
    let otherwise = majority_otherwise(&keys, lists, &chosen, shape);
    let mut answers: Vec<(Vec<String>, String)> = Vec::new();
    let mut probes: Vec<AskedForm> = Vec::new();
    for (at, label) in chosen.iter().zip(&labels) {
        let Some(label) = label else { continue };
        let condition = held_word_condition(&words[*at]);
        let candidate = probed(&form_text, &condition, None, false)?;
        answers.push((distinct[*label].clone(), condition));
        probes.push(candidate.probe.as_ref().clone());
    }
    if answers.is_empty() {
        return None;
    }
    let wrong = missed(&keys, (lists, &filled), (&chosen, &labels, otherwise)).len();
    let fills = vec![false; answers.len()];
    Some(LearnedForm { question: form.clone(), closed_as_question, answers, lessons: lessons.len(), right: lessons.len() - wrong, otherwise: otherwise.map(|key| distinct[key].clone()), taught: (0..lessons.len()).collect(), probes, fills })
}
because!(word_form, HeldWords, "the learned form of a question whose answers are labels of the told text: a decision list over the words that the told sentences hold, learned on all its lessons at the given share, each answer the most common label of the lessons that its words take first, and the answer otherwise the most common label of the lessons that no words take; nothing when no words reach the share");

pub fn word_topics(cases: &[&Case], right_percents: &[usize]) -> Vec<Option<Topic>> {
    let none = || right_percents.iter().map(|_| None).collect();
    let Some(question) = one_question(cases) else { return none() };
    let labels: HashSet<&str> = cases.iter().map(|case| case.answer.trim()).collect();
    if labels.len() < LEAST_SUPPORT - 1 || labels.len() * LEAST_SUPPORT > cases.len() {
        return none();
    }
    let said: Vec<SaidCase> = cases.iter().map(|case| said_case(case)).collect();
    let told: Vec<Vec<HashSet<&String>>> = said.iter().map(|said| said.sentences.iter().map(|sentence| sentence.iter().filter(|word| word_not_mark(word)).collect()).collect()).collect();
    let (words, lists): (Vec<Vec<String>>, Vec<Vec<usize>>) = held_word_rows(&told).into_iter().unzip();
    let base = Arc::new(Facts::default());
    let read: Vec<WordingLesson> = cases.iter().zip(&said).map(|(case, said)| lesson_of(word_numbers(&said.question), Arc::clone(&base), answer_words(&case.answer))).collect();
    let lessons: Vec<&WordingLesson> = read.iter().collect();
    let form: Pattern = question.iter().map(|word| LogicPiece::Word(word_number(word))).collect();
    let closed_as_question = cases.iter().all(|case| case.question.trim_end().ends_with(QUESTION_END));
    right_percents.iter().map(|right_percent| read_topic(WORDS_TEXT, &form_line(&word_form((&form, closed_as_question), &lessons, (&words, &lists), *right_percent)?)).ok()).collect()
}
because!(word_topics, HeldWords, "the learned text of a kind whose one question asks for a label of the told text (the intent of a request), at each given share: its question form with a decision list over the words of the told sentences, the words found once for all the shares; no facts are made of the stories, as the list reads only their words; nothing for a kind whose questions differ, whose answers have one label, or have so many labels that a label has fewer lessons than a learned answer needs");

const WORDS_TEXT: &str = "words of the told text";
because!(WORDS_TEXT, HeldWords, "the name of the learned text of a list over the words of the told text");
