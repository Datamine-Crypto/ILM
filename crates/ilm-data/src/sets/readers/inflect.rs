use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::readers::common::{LESSON_FILES, READ_FILES, lines_of, text_of};
use patterns::{because, source};

pub struct InflectSet;
source!(InflectSet, "the forms of words of the shared task of CoNLL and SIGMORPHON on universal morphological reinflection, its first task, of Cotterell and others: each line gives a word in its dictionary form, the form of the word with a set of features, and the features with semicolons between them. A case is the letters of the dictionary form, each a word, then the features, each a word, and its answer is the letters of the form. A letter from a to z stays as it is, and every other character is u with its code in hex, so that the cut of the measure, which knows a to z and the digits, keeps every character apart; a feature is t with its plain letters and digits in small letters, and every other sign of it is x with its code in hex. The lessons are the first lines of the medium train file, the lines of the dev file follow, where the cases to read start, and the large train file serves a control alone");

const INFLECT_FIELD: char = '\t';
because!(INFLECT_FIELD, InflectSet, "the mark between the dictionary form, the form and the features of a line");

const INFLECT_LINE_END: char = '\n';
because!(INFLECT_LINE_END, InflectSet, "the end of a line, which is no part of its features");

const INFLECT_FEATURES: char = ';';
because!(INFLECT_FEATURES, InflectSet, "the mark between the features of a line");

const INFLECT_FIELDS: usize = 3;
because!(INFLECT_FIELDS, InflectSet, "the fields of a line of a case: the dictionary form, the form and the features, none of them empty");

const INFLECT_LESSONS: usize = 1000;
because!(INFLECT_LESSONS, InflectSet, "the lessons of a language in the medium setting of the task, which are the first lines of its medium train file");

const INFLECT_LOW: char = 'a';
because!(INFLECT_LOW, InflectSet, "the first letter that a word of a case keeps as it is");

const INFLECT_HIGH_LETTER: char = 'z';
because!(INFLECT_HIGH_LETTER, InflectSet, "the last letter that a word of a case keeps as it is");

const INFLECT_CODE: char = 'u';
because!(INFLECT_CODE, InflectSet, "the start of the word of a character that is no letter from a to z, before its code");

const INFLECT_CODE_DIGITS: usize = 4;
because!(INFLECT_CODE_DIGITS, InflectSet, "the fewest hex digits of the code of a character, with noughts in front, so that the code of a character of the first planes has one length");

const INFLECT_FEATURE: char = 't';
because!(INFLECT_FEATURE, InflectSet, "the start of the word of a feature, which no word of letters of a case has as a word of more than one sign");

const INFLECT_SIGN: char = 'x';
because!(INFLECT_SIGN, InflectSet, "the start of the code of a sign of a feature that is no plain letter or digit");

const INFLECT_SIGN_DIGITS: usize = 2;
because!(INFLECT_SIGN_DIGITS, InflectSet, "the fewest hex digits of the code of a sign of a feature");

const INFLECT_HEX: u32 = 16;
because!(INFLECT_HEX, InflectSet, "the base of the codes of characters");

const INFLECT_GAP: &str = " ";
because!(INFLECT_GAP, InflectSet, "the mark between the words of a case");

pub const INFLECT_LARGE: &str = "high";
because!(INFLECT_LARGE, InflectSet, "the role in the description of the large train file of a language, which a control of changed cases reads for the true forms of a word with other features");

fn padded_code(code: u32, digits: usize) -> String {
    let hex = format!("{code:x}");
    let nought = char::from_digit(0, INFLECT_HEX).unwrap_or_default();
    let mut out: String = std::iter::repeat_n(nought, digits.saturating_sub(hex.len())).collect();
    out.push_str(&hex);
    out
}
because!(padded_code, InflectSet, "the code of a character in small hex digits, with noughts in front up to the fewest digits");

fn inflect_letters(text: &str) -> String {
    let words: Vec<String> = text.chars().map(|sign| if (INFLECT_LOW..=INFLECT_HIGH_LETTER).contains(&sign) { sign.to_string() } else { format!("{INFLECT_CODE}{}", padded_code(u32::from(sign), INFLECT_CODE_DIGITS)) }).collect();
    words.join(INFLECT_GAP)
}
because!(inflect_letters, InflectSet, "the characters of a word or of a form, each as one word of a case");

fn inflect_feature(text: &str) -> String {
    let mut out = INFLECT_FEATURE.to_string();
    for sign in text.chars() {
        if sign.is_ascii_alphanumeric() {
            out.push(sign.to_ascii_lowercase());
        } else {
            out.push(INFLECT_SIGN);
            out.push_str(&padded_code(u32::from(sign), INFLECT_SIGN_DIGITS));
        }
    }
    out
}
because!(inflect_feature, InflectSet, "one feature as one word of a case");

fn inflect_rows(place: &str) -> Result<Vec<Case>, String> {
    let mut out: Vec<Case> = Vec::new();
    for line in lines_of(&text_of(place)?) {
        let fields: Vec<&str> = line.trim_end_matches(INFLECT_LINE_END).split(INFLECT_FIELD).collect();
        if fields.len() != INFLECT_FIELDS {
            continue;
        }
        let (word, form, features) = (fields[0], fields[1], fields[INFLECT_FIELDS - 1]);
        if word.is_empty() || form.is_empty() || features.is_empty() {
            continue;
        }
        let written: Vec<String> = features.split(INFLECT_FEATURES).map(inflect_feature).collect();
        out.push(Case { sentences: Vec::new(), question: format!("{}{INFLECT_GAP}{}", inflect_letters(word), written.join(INFLECT_GAP)), answer: inflect_letters(form) });
    }
    Ok(out)
}
because!(inflect_rows, InflectSet, "the cases of one file in its order: the lines with three fields that are not empty");

pub fn inflect_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let mut cases = inflect_rows(&described.file(kind, LESSON_FILES)?)?;
    cases.truncate(INFLECT_LESSONS);
    let read_from = Some(cases.len());
    for place in described.files(kind, READ_FILES) {
        cases.extend(inflect_rows(&place)?);
    }
    Ok(KindCases { cases, read_from, ..KindCases::default() })
}
because!(inflect_cases, InflectSet, "the cases of a language: its lessons from the medium train file, then the cases of its dev file, and the place of the first of them");

pub fn inflect_large(described: &Described, kind: &str) -> Result<Vec<Case>, String> {
    inflect_rows(&described.file(kind, INFLECT_LARGE)?)
}
because!(inflect_large, InflectSet, "the cases of the large train file of a language, which serve a control of changed cases alone");
