use patterns::{because, source};

pub struct WordCut;
source!(WordCut, "the cut of a text into words as every learner here counts them: the text in small letters, a word is a run of the letters a to z and of digits, or one sign of arithmetic or one bracket; every other mark parts words and is no word");

pub struct SentenceStart;
source!(SentenceStart, "a written sentence of English starts with a capital letter");

pub struct PathAnswer;
source!(PathAnswer, "the path task of the twenty tasks of Weston and others writes an answer as letters with commas, the first letters of the four directions; an answer of this form is read as the words of the directions");

const SIGNS: &str = "-+*/=<>()[]{}";
because!(SIGNS, WordCut, "the marks that are words of their own: the signs of arithmetic and the brackets");

const PATH_BETWEEN: char = ',';
because!(PATH_BETWEEN, PathAnswer, "the mark between the letters of a path answer");

const DIRECTIONS: &[(char, &str)] = &[('n', "north"), ('s', "south"), ('e', "east"), ('w', "west")];
because!(DIRECTIONS, PathAnswer, "the letter of each direction of a path answer and the word that it stands for");

pub fn words(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut run = String::new();
    for c in text.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            run.push(c);
            continue;
        }
        if !run.is_empty() {
            out.push(std::mem::take(&mut run));
        }
        if SIGNS.contains(c) {
            out.push(c.to_string());
        }
    }
    if !run.is_empty() {
        out.push(run);
    }
    out
}
because!(words, WordCut, "the words of a text in their order; the measure, the lessons and every answer are compared in these words");

pub fn sign_runs(text: &str, keep: fn(char) -> bool) -> Vec<&str> {
    text.split(|sign: char| !keep(sign)).filter(|run| !run.is_empty()).collect()
}
because!(sign_runs, WordCut, "the runs of a text whose signs all pass a test, with their capital letters kept, as names are written: the names of a story, or the runs of letters of the plain alphabet");

fn direction(letter: &str) -> Option<&'static str> {
    let mut letters = letter.chars();
    let first = letters.next()?;
    if letters.next().is_some() {
        return None;
    }
    DIRECTIONS.iter().find(|(c, _)| *c == first).map(|(_, word)| *word)
}
because!(direction, PathAnswer, "the word of a direction for its one letter, and nothing for any other text");

fn path_words(low: &str) -> Option<Vec<String>> {
    let parts: Vec<&str> = low.split(PATH_BETWEEN).map(str::trim).collect();
    if parts.len() <= 1 {
        return None;
    }
    parts.iter().map(|part| direction(part).map(str::to_string)).collect()
}
because!(path_words, PathAnswer, "the words of the directions when the whole answer is two or more direction letters with commas between them; no words for any other answer");

pub fn answer_words(answer: &str) -> Vec<String> {
    let low = answer.to_lowercase();
    let low = low.trim();
    match path_words(low) {
        Some(steps) => steps,
        None => words(low),
    }
}
because!(answer_words, WordCut, "the words of an answer as the measure compares them: a path written in letters as the words of its directions, every other answer as its words");

pub fn first_capital(text: &str) -> String {
    let mut signs = text.chars();
    match signs.next() {
        Some(first) => first.to_uppercase().chain(signs).collect(),
        None => String::new(),
    }
}
because!(first_capital, SentenceStart, "the text with its first sign as a capital letter, as a written sentence starts");
