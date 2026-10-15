use crate::know::signs::{is_space, is_word_sign};
use patterns::{because, source};

pub struct TextSigns;
source!(TextSigns, "a text is cut for counting by the classes of its signs, as the tables of the Unicode standard give them: a word sign is a letter, a digit or the low line; a space parts tokens and is no token; a token is a run of word signs, or one sign that is neither a word sign nor a space; a value is found in a text as whole words without regard to the case of its letters");

const WRITE_LOW_LINE: char = '_';
because!(WRITE_LOW_LINE, TextSigns, "the one word sign that is neither a letter nor a digit: a token that starts with it can stand twice in a row, as a line of low lines does");

const WRITE_LIST_SPACE: char = ' ';
because!(WRITE_LIST_SPACE, TextSigns, "the plain space that follows the sign between the items of a list inside one value");

pub fn letter_or_digit(text: &str) -> bool {
    text.chars().next().is_some_and(|sign| is_word_sign(sign) && sign != WRITE_LOW_LINE)
}
because!(letter_or_digit, TextSigns, "does the text start with a letter or a digit: people do not write such a token twice in a row, while a sign such as a point can stand twice");

pub fn holds_token(text: &str) -> bool {
    text.chars().any(|sign| !is_space(sign))
}
because!(holds_token, TextSigns, "does the text hold one token at least; a value with none is an empty value for the writer");

pub fn token_spans(text: &str, from: usize, to: usize, out: &mut Vec<(usize, usize)>) {
    let mut open: Option<usize> = None;
    for (at, sign) in text[from..to].char_indices() {
        let at = from + at;
        if is_word_sign(sign) {
            if open.is_none() {
                open = Some(at);
            }
            continue;
        }
        if let Some(start) = open.take() {
            out.push((start, at));
        }
        if !is_space(sign) {
            out.push((at, at + sign.len_utf8()));
        }
    }
    if let Some(start) = open {
        out.push((start, to));
    }
}
because!(token_spans, TextSigns, "the places of the tokens of a part of a text, in their order: each run of word signs, and each sign that is neither a word sign nor a space");

pub fn low_tokens(text: &str) -> Vec<String> {
    let low = text.to_lowercase();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    token_spans(&low, 0, low.len(), &mut spans);
    spans.iter().map(|(start, end)| low[*start..*end].to_string()).collect()
}
because!(low_tokens, TextSigns, "the tokens of a text in small letters; the text is made small as a whole first, so that a letter whose small form depends on its place in a word is cut with that form");

pub fn first_low(text: &str) -> String {
    text.chars().next().map(|sign| sign.to_string().to_lowercase()).unwrap_or_default()
}
because!(first_low, TextSigns, "the first sign of a text in small letters, or nothing for an empty text: what is counted after a unit to see if a value can follow it");

pub fn first_capital(text: &str) -> String {
    let mut signs = text.chars();
    match signs.next() {
        Some(first) => first.to_uppercase().chain(signs).collect(),
        None => String::new(),
    }
}
because!(first_capital, TextSigns, "the text with its first sign as a capital letter, as a written text starts");

pub fn name_ending(value: &str, signs: usize) -> String {
    let low = value.to_lowercase();
    let first = low.split(is_space).find(|word| !word.is_empty()).unwrap_or("");
    let length = first.chars().count();
    first.chars().skip(length.saturating_sub(signs)).collect()
}
because!(name_ending, TextSigns, "the last signs of the first word of a name in small letters: the form of a name, which is counted with the unit to its left");

pub fn list_cuts(value: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut signs = value.chars().peekable();
    while let Some(sign) = signs.next() {
        if !is_word_sign(sign) && !is_space(sign) && signs.peek() == Some(&WRITE_LIST_SPACE) {
            signs.next();
            found.push([sign, WRITE_LIST_SPACE].iter().collect());
        }
    }
    found
}
because!(list_cuts, TextSigns, "every place in a value where one sign that is no word sign and no space stands before a plain space: what can stand between the items of a list");
