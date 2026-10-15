use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::readers::common::{even_lessons, in_one_case, lines_of, text_of};
use patterns::{because, source};

pub struct RegexSet;
source!(RegexSet, "NL-RX of Locascio and others: descriptions of regular expressions and the expressions, from a grammar and in the words of people, as plain files of the repository of the authors: one line of the file of descriptions with the same line of the file of expressions; an expression is written as words, a word for each sign and each run of letters or digits, a capital run after the word up; the first lines of the files are the lessons and the last lines are read");

const REGEX_TRAIN: usize = 7500;
because!(REGEX_TRAIN, RegexSet, "the lines of a file that are lessons, as the authors cut their train part; the lines after them are read");

const TEXTS_ROLE: &str = "src";
because!(TEXTS_ROLE, RegexSet, "the role in the description of the file of descriptions");

const ROWS_ROLE: &str = "targ";
because!(ROWS_ROLE, RegexSet, "the role in the description of the file of expressions");

const UP_WORD: &str = "up";
because!(UP_WORD, RegexSet, "the word before a run of capital letters, written in small letters after it");

const REGEX_WORD_GAP: &str = " ";
because!(REGEX_WORD_GAP, RegexSet, "the mark between the words of an expression");

const REGEX_LINE_END: char = '\n';
because!(REGEX_LINE_END, RegexSet, "the sign that a line of the files ends with and that is no part of a description or an expression");

const REGEX_SIGNS: &[(char, &str)] = &[('(', "lpar"), (')', "rpar"), ('[', "lsq"), (']', "rsq"), ('{', "lcur"), ('}', "rcur"), ('.', "dot"), ('*', "star"), ('+', "plus"), (',', "comma"), ('-', "dash"), ('|', "bar"), ('&', "amp"), ('~', "tilde"), ('\\', "bslash")];
because!(REGEX_SIGNS, RegexSet, "the word of each sign of an expression: the signs of the regular expressions of the set and their names");

fn pieces(expression: &str) -> Vec<String> {
    let signs: Vec<char> = expression.chars().collect();
    let mut out: Vec<String> = Vec::new();
    let mut at = 0usize;
    while at < signs.len() {
        let sign = signs[at];
        let same: Option<fn(&char) -> bool> = if sign.is_ascii_uppercase() {
            Some(char::is_ascii_uppercase)
        } else if sign.is_ascii_lowercase() {
            Some(char::is_ascii_lowercase)
        } else if sign.is_ascii_digit() {
            Some(char::is_ascii_digit)
        } else {
            None
        };
        let end = match same {
            Some(kind) => at + signs[at..].iter().take_while(|next| kind(next)).count(),
            None => at + 1,
        };
        out.push(signs[at..end].iter().collect());
        at = end;
    }
    out
}
because!(pieces, RegexSet, "the pieces of an expression: each run of capital letters, of small letters or of digits of the ASCII alphabet, and each other sign alone");


pub fn regex_words(expression: &str) -> Result<String, String> {
    let mut out: Vec<String> = Vec::new();
    for piece in pieces(expression) {
        let mut signs = piece.chars();
        let sign = (signs.next(), signs.next());
        if let (Some(sign), None) = sign {
            if let Some((_, word)) = REGEX_SIGNS.iter().find(|(held, _)| *held == sign) {
                out.push(word.to_string());
                continue;
            }
        }
        if in_one_case(&piece, true) {
            out.push(UP_WORD.to_string());
            out.push(piece.to_lowercase());
        } else if piece.chars().all(char::is_alphanumeric) {
            out.push(piece);
        } else {
            return Err(format!("a sign with no word: {piece}"));
        }
    }
    Ok(out.join(REGEX_WORD_GAP))
}
because!(regex_words, RegexSet, "an expression as words: a word for each sign, each run of letters or digits as a word, a capital run as the word up and the run in small letters");

fn regex_plain(words: &str) -> String {
    let mut out = String::new();
    let mut up = false;
    for word in words.split_whitespace() {
        if word == UP_WORD {
            up = true;
            continue;
        }
        match REGEX_SIGNS.iter().find(|(_, held)| *held == word) {
            Some((sign, _)) => out.push(*sign),
            None if up => out.push_str(&word.to_uppercase()),
            None => out.push_str(word),
        }
        up = false;
    }
    out
}
because!(regex_plain, RegexSet, "the expression from its words, the other way of the words of an expression, to check that no expression loses a sign");

fn regex_lines(place: &str) -> Result<Vec<String>, String> {
    let mut lines: Vec<String> = lines_of(&text_of(place)?).into_iter().map(|line| line.trim_end_matches(REGEX_LINE_END).to_string()).collect();
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    Ok(lines)
}
because!(regex_lines, RegexSet, "the lines of a file of the set without their ends, and without the empty lines at the end of the file");

pub fn regex_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let texts = regex_lines(&described.file(kind, TEXTS_ROLE)?)?;
    let rows = regex_lines(&described.file(kind, ROWS_ROLE)?)?;
    if texts.len() != rows.len() {
        return Err(format!("{kind}: {} descriptions and {} expressions", texts.len(), rows.len()));
    }
    let mut cases: Vec<Case> = Vec::with_capacity(rows.len());
    for (text, row) in texts.into_iter().zip(rows) {
        let words = regex_words(&row)?;
        if regex_plain(&words) != row {
            return Err(format!("{kind}: an expression does not come back from its words"));
        }
        cases.push(Case { sentences: Vec::new(), question: text, answer: words });
    }
    let test = cases.split_off(REGEX_TRAIN.min(cases.len()));
    Ok(KindCases { cases: even_lessons(cases, test), read_from: Some(REGEX_TRAIN), ..KindCases::default() })
}
because!(regex_cases, RegexSet, "the cases of a kind of the set: the descriptions as questions and the expressions in words as answers, the lessons spread evenly over the first lines, then the last lines, which are read");
