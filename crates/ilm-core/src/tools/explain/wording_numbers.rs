use crate::cases::Case;
use crate::tools::explain::text::{GROUPED_FIRST, LogicPiece, NUMBER_POINT, PLAIN_PLACES, Topic, fact_words, place_letter, read_topic, word_not_mark};
use crate::tools::explain::wording::{CONDITIONS_JOINED, LEAST_SUPPORT, ListKeys, ListShape, decision_list, list_answers, majority_otherwise, written_form};
use crate::tools::explain::wording_words::{held_word_condition, held_word_rows};
use patterns::{because, source};
use std::collections::HashSet;

pub struct ReckonedAnswers;
source!(ReckonedAnswers, "the answer of a word problem as a step of arithmetic over the numbers that its story tells, in their order (the first number, the second, ...), learned from its lessons as a decision list over the words of the story and the question (Rivest): each answer of the list is one way to reckon (the first number minus the second, the second times the third, then divided by the first), chosen for a word, or two words of one sentence, that stand in the told text, the way that answers the most lessons where the word stands, as long as it is right on nearly all of them; the last answer is the way that answers the most of the lessons that no word took; the list is a list of the learned wording over the facts of a text that tells the numbers of a story, written as a question form of the library, so a person reads which word asks for which reckoning");

const NUMBER_FACT_START: &str = "number ";
because!(NUMBER_FACT_START, ReckonedAnswers, "the word before the place of a number in the order of its story, in the fact that a text of the library makes for each number of a story");

const NUMBER_FACT_MIDDLE: &str = " of the story is ";
because!(NUMBER_FACT_MIDDLE, ReckonedAnswers, "the words between the place of a number in its story and its value, in the fact of a number");

pub const NUMBER_FACT_WORDS: &[&str] = &["number", "of", "the", "story", "is"];
because!(NUMBER_FACT_WORDS, ReckonedAnswers, "the fixed words of the fact of a number of a story, by which the text that tells the numbers is found among the texts of the library");

const QUESTION_LEAD: &str = "the question is ";
because!(QUESTION_LEAD, ReckonedAnswers, "the words that the engine puts before the question when it tells it as one more sentence of the story, so that its numbers come last, as the text that tells the numbers reads them");

const MOST_NUMBERS: usize = 4;
because!(MOST_NUMBERS, ReckonedAnswers, "the most numbers of a story that a way to reckon reads: word problems of one or two steps tell few numbers, and with more the ways to reckon grow too many to judge");

const STEPS: &[(&str, Step)] = &[("plus", Step::Plus), ("minus", Step::Minus), ("times", Step::Times), ("divided by", Step::Over)];
because!(STEPS, ReckonedAnswers, "the steps of arithmetic of a way to reckon, with the words that the library writes for each");

const THOUSANDS_MARK: char = ',';
because!(THOUSANDS_MARK, ReckonedAnswers, "the mark between the thousands of a number in the answer of a lesson, taken out before the answer is read as a number");

const NEAR: f64 = 1e-6;
because!(NEAR, ReckonedAnswers, "how far a reckoned value may lie from the answer of a lesson and still be the same number, for numbers with a point");

const MOST_RECKON_ANSWERS: usize = 120;
because!(MOST_RECKON_ANSWERS, ReckonedAnswers, "the most answers of the list: a few dozen ways to reckon, each told by a few words, while a longer list learns its lessons one by one");

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Step {
    Plus,
    Minus,
    Times,
    Over,
}
because!(Step, ReckonedAnswers, "a step of arithmetic that a way to reckon takes, as the engine reckons it with exact amounts");

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct WayToReckon {
    first: usize,
    second: usize,
    step: Step,
    then: Option<(Step, usize)>,
}
because!(WayToReckon, ReckonedAnswers, "a way to reckon: the first number of the step and the second (their places in the story), the step, and maybe a second step with the number that it takes");

fn stepped(step: Step, one: f64, other: f64) -> Option<f64> {
    match step {
        Step::Plus => Some(one + other),
        Step::Minus => Some(one - other),
        Step::Times => Some(one * other),
        Step::Over => (other.abs() > NEAR).then(|| one / other),
    }
}
because!(stepped, ReckonedAnswers, "the value of one step, and nothing for a division by nought");

fn value_of(reckoning: &WayToReckon, numbers: &[Option<f64>]) -> Option<f64> {
    let first = stepped(reckoning.step, numbers.get(reckoning.first).copied().flatten()?, numbers.get(reckoning.second).copied().flatten()?)?;
    match reckoning.then {
        Some((step, third)) => stepped(step, first, numbers.get(third).copied().flatten()?),
        None => Some(first),
    }
}
because!(value_of, ReckonedAnswers, "the value of a way to reckon on the numbers of a story, and nothing when a number it reads is missing");

fn reckonings(count: usize) -> Vec<WayToReckon> {
    let steps: Vec<Step> = STEPS.iter().map(|(_, step)| *step).collect();
    let mut out: Vec<WayToReckon> = Vec::new();
    for first in 0..count {
        for second in (0..count).filter(|second| *second != first) {
            for step in &steps {
                out.push(WayToReckon { first, second, step: *step, then: None });
                for third in (0..count).filter(|third| *third != first && *third != second) {
                    for then in &steps {
                        out.push(WayToReckon { first, second, step: *step, then: Some((*then, third)) });
                    }
                }
            }
        }
    }
    out
}
because!(reckonings, ReckonedAnswers, "every way to reckon over a number of numbers: one step on two of them, in either order, and that step followed by one more on a third");

fn is_number(word: &str) -> bool {
    word.chars().next().is_some_and(|first| first.is_ascii_digit()) && word.chars().all(|letter| letter.is_ascii_digit() || letter == NUMBER_POINT) && word.parse::<f64>().is_ok()
}
because!(is_number, ReckonedAnswers, "whether a word of a story is a number as the engine reads one: digits, with a point between them for a part of one");

fn story_numbers(case: &Case) -> Option<Vec<Option<f64>>> {
    let mut told: Vec<(String, usize, usize)> = Vec::new();
    let question = format!("{QUESTION_LEAD}{}", case.question);
    for (sentence_at, sentence) in case.sentences.iter().chain([&question]).enumerate() {
        for (word_at, word) in fact_words(sentence).into_iter().enumerate().filter(|(_, word)| is_number(word)) {
            told.push((word, sentence_at, word_at));
        }
    }
    let mut values: Vec<&String> = told.iter().map(|(word, _, _)| word).collect();
    values.sort();
    values.dedup();
    let mut out: Vec<Option<f64>> = Vec::new();
    for value in values {
        let before: HashSet<&String> = told.iter().filter(|(_, sentence, place)| told.iter().any(|(later, later_sentence, later_place)| later == value && (sentence < later_sentence || (sentence == later_sentence && place < later_place)))).map(|(earlier, _, _)| earlier).collect();
        let place = before.len();
        if place >= MOST_NUMBERS {
            return None;
        }
        if out.len() <= place {
            out.resize(place + 1, None);
        }
        if out[place].is_some() {
            return None;
        }
        out[place] = value.parse::<f64>().ok();
    }
    Some(out)
}
because!(story_numbers, ReckonedAnswers, "the numbers of a story by their places, as the text that tells them gives them: a number's place is one more than the count of the other values told before one of its tellings, sentence by sentence and the question last, so that a number told twice counts itself and leaves a place empty, as the engine reckons it; nothing for a story whose two values come to one place, or with more places than a way to reckon reads");

fn step_words(step: Step) -> &'static str {
    STEPS.iter().find(|(_, known)| *known == step).map_or_else(|| STEPS[0].0, |(words, _)| words)
}
because!(step_words, ReckonedAnswers, "a step as a line of the library writes it between its two numbers, so that the engine reads the learned answer as it reads a written one");

fn reckoning_text(reckoning: &WayToReckon) -> String {
    let (one, other, third_value) = (place_letter(PLAIN_X), place_letter(PLAIN_X + 1), place_letter(PLAIN_X + 1 + 1));
    let number = |at: usize, letter: &str| format!("{NUMBER_FACT_START}{}{NUMBER_FACT_MIDDLE}{letter}", at + 1);
    let result = place_letter(RESULT_PLACE);
    match reckoning.then {
        None => [number(reckoning.first, &one), number(reckoning.second, &other), format!("{one} {} {other}{RECKON_IS}{result}", step_words(reckoning.step))].join(CONDITIONS_JOINED),
        Some((then, third)) => {
            let between = place_letter(RESULT_PLACE + 1);
            [number(reckoning.first, &one), number(reckoning.second, &other), number(third, &third_value), format!("{one} {} {other}{RECKON_IS}{between}", step_words(reckoning.step)), format!("{between} {} {third_value}{RECKON_IS}{result}", step_words(then))].join(CONDITIONS_JOINED)
        }
    }
}
because!(reckoning_text, ReckonedAnswers, "a way to reckon as the conditions of an answer of the library: the numbers it reads by their places, and the steps from them to the answer");

const PLAIN_X: usize = PLAIN_PLACES + 6;
because!(PLAIN_X, ReckonedAnswers, "the first of the places of one word (X, Y, Z, the last three) that hold the numbers that a way to reckon reads");

const RESULT_PLACE: usize = 11;
because!(RESULT_PLACE, ReckonedAnswers, "the place (M) of the answer of a way to reckon, and after it the place (N) of the value between its two steps");

const RECKON_IS: &str = " is ";
because!(RECKON_IS, ReckonedAnswers, "the word before the result of a step");

pub fn reckoned_topics(numbers_text: &Topic, cases: &[&Case], right_percents: &[usize]) -> Vec<Option<Topic>> {
    let none = || right_percents.iter().map(|_| None).collect();
    let mut read: Vec<(Vec<Vec<String>>, HashSet<WayToReckon>)> = Vec::new();
    for case in cases {
        let Some(numbers) = story_numbers(case) else { continue };
        let Ok(answer) = case.answer.trim().replace(THOUSANDS_MARK, "").parse::<f64>() else { continue };
        let ways: HashSet<WayToReckon> = reckonings(numbers.len()).into_iter().filter(|way| value_of(way, &numbers).is_some_and(|value| (value - answer).abs() < NEAR)).collect();
        let question = format!("{QUESTION_LEAD}{}", case.question);
        let sentences: Vec<Vec<String>> = case.sentences.iter().chain([&question]).map(|sentence| fact_words(sentence).into_iter().filter(|word| word_not_mark(word) && !is_number(word)).collect()).collect();
        read.push((sentences, ways));
    }
    if read.len() < LEAST_SUPPORT {
        return none();
    }
    let mut ways: Vec<WayToReckon> = read.iter().flat_map(|(_, ways)| ways.iter().cloned()).collect();
    ways.sort();
    ways.dedup();
    let keys = ListKeys { of: read.iter().map(|(_, held)| held.iter().map(|way| ways.binary_search(way).unwrap_or_default()).collect()).collect(), count: ways.len() };
    let told: Vec<Vec<HashSet<&String>>> = read.iter().map(|(sentences, _)| sentences.iter().map(|sentence| sentence.iter().collect()).collect()).collect();
    let (words, lists): (Vec<Vec<String>>, Vec<Vec<usize>>) = held_word_rows(&told).into_iter().unzip();
    right_percents.iter().map(|right_percent| reckoned_list(numbers_text, (&words, &lists), (&keys, &ways), *right_percent)).collect()
}
because!(reckoned_topics, ReckonedAnswers, "the learned text of the word problems of a kind at each given share: the text that tells the numbers of a story with a question form for any question, whose answers are a decision list over the words of the story and the question (not their numbers) with the ways to reckon as the keys of the lessons, a lesson right under a way that gives its answer; the words and the ways found once for all the shares; nothing when too few lessons tell their numbers once each");

fn reckoned_list(numbers_text: &Topic, (words, lists): (&[Vec<String>], &[Vec<usize>]), (keys, ways): (&ListKeys, &[WayToReckon]), right_percent: usize) -> Option<Topic> {
    let shape = ListShape { most: MOST_RECKON_ANSWERS, right_percent, agreed_otherwise: false };
    let filled: Vec<Option<Vec<bool>>> = vec![None; lists.len()];
    let chosen = decision_list(keys, lists, (&filled, &vec![false; lists.len()]), shape);
    let labels = list_answers(keys, lists, &filled, &chosen);
    let otherwise = majority_otherwise(keys, lists, &chosen, shape);
    if chosen.is_empty() && otherwise.is_none() {
        return None;
    }
    let result = vec![place_letter(RESULT_PLACE)];
    let mut answers: Vec<(Vec<String>, String)> = chosen.iter().zip(&labels).filter_map(|(at, way)| Some((result.clone(), format!("{}{CONDITIONS_JOINED}{}", held_word_condition(&words[*at]), reckoning_text(&ways[(*way)?]))))).collect();
    answers.extend(otherwise.map(|way| (result.clone(), reckoning_text(&ways[way]))));
    let line = written_form(&vec![LogicPiece::Open(GROUPED_FIRST)], false, &answers, None);
    let read = read_topic(&numbers_text.name, &line).ok()?;
    Some(Topic { questions: read.questions, reply_lines: read.reply_lines, ..numbers_text.clone() })
}
because!(reckoned_list, ReckonedAnswers, "the learned text of the word problems of a kind at one share: the decision list over the words of the lessons, each answer the way to reckon that most of the lessons that its words take first are right under, and last the way that most of the lessons that no words take are right under, as the answer of any question whose numbers it reads");
