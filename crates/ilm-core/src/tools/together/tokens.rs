use crate::base::exact::Amount;
use crate::tools::amount::tokens::{AMOUNT_ASK, AMOUNT_END, AMOUNT_NUMBER, AmountCut};
use crate::tools::pattern::names::WordTexts;
use patterns::{because, source};

pub struct ProblemTokens;
source!(ProblemTokens, "a problem is read as the amount tool reads it, with the digits that follow a letter taken out before: such digits are a part of a word (a name of a thing, a step of a list) and no amount");

pub const RESULT_PLACES: usize = 12;
because!(RESULT_PLACES, ProblemTokens, "a result is written as a decimal with at most this many places after the point, as the amount tool writes it; a longer result is not written");

const DECIMAL_POINT: char = '.';
because!(DECIMAL_POINT, ProblemTokens, "the mark between the whole part and the places of a written decimal");

pub fn places_after_point(text: &str) -> usize {
    text.split_once(DECIMAL_POINT).map_or(0, |(_, after)| after.len())
}
because!(places_after_point, ProblemTokens, "the places after the point of a written amount, nought for a whole number");

#[derive(Clone, Debug, Default)]
pub struct Problem {
    pub words: Vec<u32>,
    pub values: Vec<Option<Amount>>,
    pub numbers: usize,
}
because!(Problem, ProblemTokens, "a problem as the amount tool reads it: its tokens, with a mark at the end of each told sentence and one before the question, the exact amount of each token that is a number, and how many tokens are numbers");

#[derive(Clone, Copy, Debug, Default)]
pub struct TokenMarks {
    pub number: u32,
    pub end: u32,
    pub ask: u32,
}
because!(TokenMarks, ProblemTokens, "the tokens of a number, of the end of a told sentence and of the start of the question in a table of words. of takes them from a table; read gives the tokens of a problem");

pub fn without_word_digits(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut before: Option<char> = None;
    let mut dropping = false;
    for sign in text.chars() {
        if sign.is_ascii_digit() && (dropping || before.is_some_and(|held| held.is_ascii_alphabetic())) {
            dropping = true;
        } else {
            dropping = false;
            out.push(sign);
        }
        before = Some(sign);
    }
    out
}
because!(without_word_digits, ProblemTokens, "a text with each run of digits that directly follows a letter taken out");

pub fn answer_amount(answer: &str) -> Option<Amount> {
    let mut cut = AmountCut::default();
    let mut lower = String::new();
    let mut tokens: Vec<String> = Vec::new();
    cut.scan(answer, u32::MAX, &mut lower, &mut |word: &str| {
        tokens.push(word.to_string());
        tokens.len() as u32 - 1
    });
    if cut.words.len() == 1 && cut.spots.len() == 1 { cut.values[0] } else { None }
}
because!(answer_amount, ProblemTokens, "the amount of an answer that is one number and nothing else, as the amount tool reads it, and none for any other answer");

pub fn token_amount(token: &str) -> Option<Amount> {
    answer_amount(token)
}
because!(token_amount, ProblemTokens, "the amount of a token of a sentence when the amount tool reads the token as one number and nothing else");

impl TokenMarks {
    pub fn of<N: WordTexts>(names: &mut N) -> TokenMarks {
        TokenMarks { number: names.number(AMOUNT_NUMBER), end: names.number(AMOUNT_END), ask: names.number(AMOUNT_ASK) }
    }

    pub fn read<N: WordTexts>(&self, names: &mut N, sentences: &[String], question: &str) -> Problem {
        let mut cut = AmountCut::default();
        let mut lower = String::new();
        for sentence in sentences {
            cut.scan(&without_word_digits(sentence), self.number, &mut lower, &mut |word: &str| names.number(word));
            cut.words.push(self.end);
        }
        cut.words.push(self.ask);
        cut.scan(&without_word_digits(question), self.number, &mut lower, &mut |word: &str| names.number(word));
        let mut values: Vec<Option<Amount>> = vec![None; cut.words.len()];
        for (spot, value) in cut.spots.iter().zip(&cut.values) {
            values[*spot as usize] = *value;
        }
        Problem { words: cut.words, values, numbers: cut.spots.len() }
    }
}
