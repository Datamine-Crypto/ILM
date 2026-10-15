use ilm_core::base::exact::Amount;
use crate::controls::file::ChangedCase;
use crate::controls::text::{byte_runs, run_texts};
use crate::sets::readers::common::stripped;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct NumberChanges;
source!(NumberChanges, "changed problems that tell numbers and ask for a number. A number of the text is changed, and the true answer of the changed problem comes from every calculation of one, two or three of the numbers of the problem, each used once, with the four operations in every order and grouping, that gives the answer of the lesson: the change is made when all these calculations use the number and give one new result, whole and above nought as the old one, or when none of them uses the number, so that the answer stays. Two told sentences that hold a number and start with words of their own change places, and the answer stays. Every name gets a new word, and the answer stays. A learner that reads the numbers and calculates follows all of them");

const NUMBER_POINT: u8 = b'.';
because!(NUMBER_POINT, NumberChanges, "the mark between the whole part of a number of a text and its part below one");

const ANSWER_SLASH: char = '/';
because!(ANSWER_SLASH, NumberChanges, "the mark between the parts of an answer that is written as a fraction");

const ANSWER_PLUS: char = '+';
because!(ANSWER_PLUS, NumberChanges, "the sign that an answer may write before an amount above nought");

const ANSWER_MINUS: char = '-';
because!(ANSWER_MINUS, NumberChanges, "the sign that an answer writes before an amount below nought");

const NUMBER_THOUSANDS: &str = ",";
because!(NUMBER_THOUSANDS, NumberChanges, "the mark between the thousands of an answer, which is no part of its value");

const NUMBER_MOST: usize = 6;
because!(NUMBER_MOST, NumberChanges, "the most numbers of a problem whose calculations are all tried, so that the trials stay few");

const NUMBER_PLACES: usize = 6;
because!(NUMBER_PLACES, NumberChanges, "the most places after the point of a result that the sets write as a decimal");

const NUMBER_COMMON_SHARE: f64 = 0.1;
because!(NUMBER_COMMON_SHARE, NumberChanges, "the share of the lessons of the model that a word must stand in to be common, so that it is no first word of its own");

const NUMBER_CHANGES: &[(i128, i128)] = &[(1, 1), (1, 2), (1, 3), (2, 0), (3, 0), (1, 10)];
because!(NUMBER_CHANGES, NumberChanges, "the changes of a number that are tried in turn, each a factor and an amount added: one more, two more, three more, twice, three times, and ten more; the first that gives a new result is taken");

const NUMBER_NEW_NAMES: &[&str] = &["Zorvik", "Quenda", "Tharben", "Ulmira", "Vexlan", "Brondis", "Kelvora", "Yantro", "Drusel", "Ombrik", "Feldra", "Girvan", "Hestol", "Ivrana", "Jorbek", "Lunzor", "Mirvek", "Nestra", "Polvin", "Rastik"];
because!(NUMBER_NEW_NAMES, NumberChanges, "made-up names that a problem gets in place of its names, the first one that its kind does not hold first");

const NUMBER_SENTENCE_ENDS: &str = ".?!";
because!(NUMBER_SENTENCE_ENDS, NumberChanges, "the marks that end a sentence, after which a word with a capital letter is no name by its place");

const NUMBER_GAP: &str = " ";
because!(NUMBER_GAP, NumberChanges, "the mark between the told sentences and the question when their words are counted together");

const SORT_NUMBER_CHANGED: &str = "a number changed";
because!(SORT_NUMBER_CHANGED, NumberChanges, "the sort of the change of a number that every calculation of the answer uses");

const SORT_NUMBER_UNUSED: &str = "a number that is not used changed";
because!(SORT_NUMBER_UNUSED, NumberChanges, "the sort of the change of a number that no calculation of the answer uses");

const SORT_NUMBER_SWAP: &str = "two sentences change places";
because!(SORT_NUMBER_SWAP, NumberChanges, "the sort of the change where two told sentences with numbers change places");

const SORT_NUMBER_NAMES: &str = "new names";
because!(SORT_NUMBER_NAMES, NumberChanges, "the sort of the change where every name gets a new word");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation {
    Plus,
    Minus,
    Times,
    Over,
}
because!(Operation, NumberChanges, "one of the four operations of a calculation, in the order in which they are tried");

const OPERATIONS: &[Operation] = &[Operation::Plus, Operation::Minus, Operation::Times, Operation::Over];
because!(OPERATIONS, NumberChanges, "the four operations in the order in which they are tried");

fn operate(operation: Operation, left: Option<Amount>, right: Option<Amount>) -> Option<Amount> {
    let (left, right) = (left?, right?);
    match operation {
        Operation::Plus => left.plus(right),
        Operation::Minus => left.minus(right),
        Operation::Times => left.times(right),
        Operation::Over => left.over(right),
    }
}
because!(operate, NumberChanges, "the result of an operation on two amounts; nothing when a side has none or a division is by nought");

#[derive(Clone, Copy, Debug)]
enum Calculation {
    One(usize),
    Two(usize, usize, Operation),
    Left(usize, usize, usize, Operation, Operation),
    Right(usize, usize, usize, Operation, Operation),
}
because!(Calculation, NumberChanges, "a calculation of numbers of a problem: one number, an operation on two, or two operations on three grouped to the left or to the right");

impl Calculation {
    fn value(&self, amounts: &[Amount]) -> Option<Amount> {
        match *self {
            Calculation::One(at) => Some(amounts[at]),
            Calculation::Two(left, right, operation) => operate(operation, Some(amounts[left]), Some(amounts[right])),
            Calculation::Left(first, second, third, inner, outer) => operate(outer, operate(inner, Some(amounts[first]), Some(amounts[second])), Some(amounts[third])),
            Calculation::Right(first, second, third, inner, outer) => operate(outer, Some(amounts[first]), operate(inner, Some(amounts[second]), Some(amounts[third]))),
        }
    }

    fn uses(&self, place: usize) -> bool {
        match *self {
            Calculation::One(at) => at == place,
            Calculation::Two(left, right, _) => left == place || right == place,
            Calculation::Left(first, second, third, ..) | Calculation::Right(first, second, third, ..) => [first, second, third].contains(&place),
        }
    }
}

fn calculations(amounts: &[Amount], truth: Amount) -> Vec<Calculation> {
    let count = amounts.len();
    let mut out: Vec<Calculation> = Vec::new();
    let mut try_one = |calculation: Calculation| {
        if calculation.value(amounts) == Some(truth) {
            out.push(calculation);
        }
    };
    for at in 0..count {
        try_one(Calculation::One(at));
    }
    for left in 0..count {
        for right in (0..count).filter(|right| *right != left) {
            for operation in OPERATIONS.iter().copied() {
                if [Operation::Plus, Operation::Times].contains(&operation) && left > right {
                    continue;
                }
                try_one(Calculation::Two(left, right, operation));
            }
        }
    }
    for first in 0..count {
        for second in (0..count).filter(|second| *second != first) {
            for third in (0..count).filter(|third| *third != first && *third != second) {
                for inner in OPERATIONS.iter().copied() {
                    for outer in OPERATIONS.iter().copied() {
                        try_one(Calculation::Left(first, second, third, inner, outer));
                        try_one(Calculation::Right(first, second, third, inner, outer));
                    }
                }
            }
        }
    }
    out
}
because!(calculations, NumberChanges, "every calculation of one, two or three of the amounts that gives the truth; a sum or a product of two is taken in one order only");

fn number_runs_after_letters(text: &str) -> Vec<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let after_letter = at > 0 && bytes[at - 1].is_ascii_alphabetic();
        if !bytes[at].is_ascii_digit() || after_letter {
            at += 1;
            continue;
        }
        let mut end = at;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if bytes.get(end) == Some(&NUMBER_POINT) && bytes.get(end + 1).is_some_and(u8::is_ascii_digit) {
            end += 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
        }
        out.push((at, end));
        at = end;
    }
    out
}
because!(number_runs_after_letters, NumberChanges, "the places of the numbers of a text, each a run of digits with a part after a point, where a digit right after a letter starts no number");

fn number_amount(text: &str) -> Option<Amount> {
    let (before, after) = text.split_once(char::from(NUMBER_POINT)).unwrap_or((text, ""));
    Amount::from_digits(before, after)
}
because!(number_amount, NumberChanges, "the exact value of the digits of a text, with the part after the point as a fraction, so that no result is rounded");

fn answer_value(answer: &str) -> Option<Amount> {
    let text = answer.replace(NUMBER_THOUSANDS, "");
    let text = stripped(&text);
    let (negative, text) = match text.chars().next() {
        Some(ANSWER_MINUS) => (true, &text[1..]),
        Some(ANSWER_PLUS) => (false, &text[1..]),
        _ => (false, text),
    };
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    let value = if let Some((top, bottom)) = text.split_once(ANSWER_SLASH) {
        let (top, bottom) = (stripped(top), stripped(bottom));
        if !digits(top) || !digits(bottom) {
            return None;
        }
        Amount::from_digits(top, "")?.over(Amount::from_digits(bottom, "")?)?
    } else {
        let (before, after) = text.split_once(char::from(NUMBER_POINT)).unwrap_or((text, ""));
        let starts = before.bytes().next().or_else(|| after.bytes().next()).is_some_and(|byte| byte.is_ascii_digit());
        if !starts || !before.bytes().all(|byte| byte.is_ascii_digit()) || !after.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        Amount::from_digits(before, after)?
    };
    if negative { value.negated() } else { Some(value) }
}
because!(answer_value, NumberChanges, "the exact amount of an answer: a whole number, a decimal or a fraction with an optional sign and marks between the thousands; nothing for any other answer or a fraction over nought");

struct FoundNumber {
    text: usize,
    start: usize,
    end: usize,
    amount: Amount,
}
because!(FoundNumber, NumberChanges, "a number of a problem: the text that holds it, the told sentences first and the question last, its place in that text, and its amount");

fn problem_numbers(texts: &[&str]) -> Option<Vec<FoundNumber>> {
    let mut out: Vec<FoundNumber> = Vec::new();
    for (text, words) in texts.iter().enumerate() {
        for (start, end) in number_runs_after_letters(words) {
            out.push(FoundNumber { text, start, end, amount: number_amount(&words[start..end])? });
        }
    }
    Some(out)
}
because!(problem_numbers, NumberChanges, "each number of the told sentences and of the question with its text, its place and its exact value, in the order of the texts; nothing when a value does not fit");

fn with_number(texts: &[&str], found: &FoundNumber, written: &str) -> (Vec<String>, String) {
    let mut out: Vec<String> = texts.iter().map(|text| text.to_string()).collect();
    let held = &texts[found.text];
    out[found.text] = format!("{}{written}{}", &held[..found.start], &held[found.end..]);
    let question = out.pop().unwrap_or_default();
    (out, question)
}
because!(with_number, NumberChanges, "the told sentences and the question of a problem with one of its numbers written anew");

fn changed_number(case: &Case, truth: Amount) -> Vec<(bool, Vec<String>, String, Amount)> {
    let texts: Vec<&str> = case.sentences.iter().map(String::as_str).chain([case.question.as_str()]).collect();
    let Some(found) = problem_numbers(&texts) else { return Vec::new() };
    let amounts: Vec<Amount> = found.iter().map(|number| number.amount).collect();
    if !(1..=NUMBER_MOST).contains(&amounts.len()) || truth <= Amount::whole(0) {
        return Vec::new();
    }
    let ways = calculations(&amounts, truth);
    if ways.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<(bool, Vec<String>, String, Amount)> = Vec::new();
    for (place, amount) in amounts.iter().enumerate() {
        if !amount.is_whole() {
            continue;
        }
        let using: Vec<&Calculation> = ways.iter().filter(|way| way.uses(place)).collect();
        if !using.is_empty() && using.len() < ways.len() {
            continue;
        }
        for (factor, added) in NUMBER_CHANGES {
            let Some(changed) = amount.times(Amount::whole(*factor)).and_then(|value| value.plus(Amount::whole(*added))) else { continue };
            let mut new = amounts.clone();
            new[place] = changed;
            let Some(written) = changed.written(0) else { continue };
            if using.is_empty() {
                let (sentences, question) = with_number(&texts, &found[place], &written);
                out.push((false, sentences, question, truth));
                break;
            }
            let results: Vec<Option<Amount>> = using.iter().map(|way| way.value(&new)).collect();
            let result = results[0];
            let one = results.iter().all(|other| *other == result);
            if let Some(result) = result.filter(|value| one && *value > Amount::whole(0) && *value != truth && (value.is_whole() || !truth.is_whole())) {
                let (sentences, question) = with_number(&texts, &found[place], &written);
                out.push((true, sentences, question, result));
                break;
            }
        }
    }
    out
}
because!(changed_number, NumberChanges, "the trials of a lesson: for each whole number that all calculations of the answer use, the first change that gives one new result, or for each whole number that none of them uses, its first change, with the changed texts and the true answer");

fn letter_runs(text: &str) -> Vec<(usize, usize)> {
    byte_runs(text, u8::is_ascii_alphabetic)
}
because!(letter_runs, NumberChanges, "the places of the words of a text: the runs of letters of the Latin alphabet");

fn case_words(case: &Case) -> HashSet<String> {
    let joined = format!("{}{NUMBER_GAP}{}", case.sentences.join(NUMBER_GAP), case.question);
    run_texts(&joined, &letter_runs(&joined)).into_iter().map(str::to_lowercase).collect()
}
because!(case_words, NumberChanges, "the words of the told sentences and the question of a case, in small letters, each once");

fn swap_pairs(sentences: &[String], common: &HashSet<String>) -> Vec<(usize, usize)> {
    let firsts: Vec<Option<String>> = sentences.iter().map(|sentence| run_texts(sentence, &letter_runs(sentence)).first().map(|word| word.to_lowercase()).filter(|word| !number_runs_after_letters(sentence).is_empty() && !common.contains(word))).collect();
    let mut out: Vec<(usize, usize)> = Vec::new();
    for one in 0..sentences.len() {
        for other in one + 1..sentences.len() {
            if let (Some(first), Some(second)) = (&firsts[one], &firsts[other])
                && first != second
            {
                out.push((one, other));
            }
        }
    }
    out
}
because!(swap_pairs, NumberChanges, "the pairs of told sentences that can change places: both hold a number and start with different words that are not common");

fn names_of(lessons: &[Case]) -> HashSet<String> {
    let mut out: HashSet<String> = HashSet::new();
    for case in lessons {
        for text in case.sentences.iter().chain([&case.question]) {
            let mut tokens: Vec<&str> = Vec::new();
            for (at, sign) in text.char_indices() {
                if NUMBER_SENTENCE_ENDS.contains(sign) {
                    tokens.push(&text[at..at + sign.len_utf8()]);
                } else if sign.is_ascii_alphabetic() && !text[..at].ends_with(|before: char| before.is_ascii_alphabetic()) {
                    let end = text[at..].find(|after: char| !after.is_ascii_alphabetic()).map_or(text.len(), |length| at + length);
                    tokens.push(&text[at..end]);
                }
            }
            for pair in tokens.windows(NUMBER_PAIR) {
                let (before, word) = (pair[0], pair[1]);
                let mut letters = word.chars();
                let capital = letters.next().is_some_and(|first| first.is_ascii_uppercase());
                let rest = letters.as_str();
                let small = rest.chars().any(|sign| sign.is_ascii_lowercase()) && !rest.chars().any(|sign| sign.is_ascii_uppercase());
                if capital && small && !NUMBER_SENTENCE_ENDS.contains(before) {
                    out.insert(word.to_string());
                }
            }
        }
    }
    out
}
because!(names_of, NumberChanges, "the words that the lessons hold with a capital letter and small letters after it, after a word and not at the start of a sentence");

const NUMBER_PAIR: usize = 2;
because!(NUMBER_PAIR, NumberChanges, "a word and the word right after it, for a capital letter that is not at the start of a sentence");

fn with_new_names(case: &Case, names: &HashSet<String>, vocabulary: &HashSet<String>) -> (Vec<String>, String, usize) {
    let free: Vec<&str> = NUMBER_NEW_NAMES.iter().copied().filter(|name| !vocabulary.contains(&name.to_lowercase())).collect();
    let mut table: HashMap<String, &str> = HashMap::new();
    let mut renamed = |text: &str| {
        let mut out = String::with_capacity(text.len());
        let mut last = 0;
        for (start, end) in letter_runs(text) {
            let word = &text[start..end];
            out.push_str(&text[last..start]);
            if names.contains(word) && !table.contains_key(word) && table.len() < free.len() {
                table.insert(word.to_string(), free[table.len()]);
            }
            out.push_str(table.get(word).copied().unwrap_or(word));
            last = end;
        }
        out.push_str(&text[last..]);
        out
    };
    let sentences: Vec<String> = case.sentences.iter().map(|sentence| renamed(sentence)).collect();
    let question = renamed(&case.question);
    (sentences, question, table.len())
}
because!(with_new_names, NumberChanges, "the problem with every name made a new word, one new word for one name, the new words that the kind does not hold in their order; and how many names were changed");

pub fn number_changes(lessons: &[Case], whole: bool) -> Vec<ChangedCase> {
    let (first, last): (Vec<usize>, Vec<usize>) = if whole { ((0..lessons.len()).collect(), (0..lessons.len()).collect()) } else { inner_parts(lessons) };
    let mut held: HashMap<String, usize> = HashMap::new();
    for place in &first {
        for word in case_words(&lessons[*place]) {
            *held.entry(word).or_default() += 1;
        }
    }
    let common: HashSet<String> = held.into_iter().filter(|(_, count)| *count as f64 >= NUMBER_COMMON_SHARE * first.len() as f64).map(|(word, _)| word).collect();
    let vocabulary: HashSet<String> = lessons.iter().flat_map(case_words).collect();
    let names = names_of(lessons);
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let case = &lessons[place];
        let Some(truth) = answer_value(&case.answer) else { continue };
        let mut add = |sort: &str, sentences: Vec<String>, question: String, truth: String| out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences, question, truth });
        for (used, sentences, question, result) in changed_number(case, truth) {
            let Some(written) = result.written(NUMBER_PLACES) else { continue };
            if used {
                add(SORT_NUMBER_CHANGED, sentences, question, written);
            } else {
                add(SORT_NUMBER_UNUSED, sentences, question, case.answer.clone());
            }
        }
        for (one, other) in swap_pairs(&case.sentences, &common) {
            let mut sentences = case.sentences.clone();
            sentences.swap(one, other);
            add(SORT_NUMBER_SWAP, sentences, case.question.clone(), case.answer.clone());
        }
        let (sentences, question, changed) = with_new_names(case, &names, &vocabulary);
        if changed > 0 {
            add(SORT_NUMBER_NAMES, sentences, question, case.answer.clone());
        }
    }
    out
}
because!(number_changes, NumberChanges, "the changed problems of the lessons that the model does not learn, or of all lessons for a model of all lessons, with their true answers");
