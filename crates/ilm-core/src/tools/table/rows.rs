use crate::base::words::words;
use crate::cases::Case;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct ToldTable;
source!(ToldTable, "a story whose told sentences have one form is a table: the words that most told sentences of the lessons hold are the form, and the other words of a sentence are the fields of its row in their order; a sentence with another number of fields than most sentences of its story is no row; a word of the question that a field of the story holds, or a word with a digit, is a value of the question, and the other words of the question are its form");

const FORM_SHARE: f64 = 0.5;
because!(FORM_SHARE, ToldTable, "a word is a word of the form when at least this share of the told sentences of the lessons hold it: the words that only some sentences hold are what tells one row from another");

pub(super) const TABLE_SHARE: f64 = 0.9;
because!(TABLE_SHARE, ToldTable, "the lessons are tables when at least this share of their told sentences have the form of most sentences: a story of free sentences has many forms, and the tool leaves it to the other tools");

const VALUE_MARK: &str = "#";
because!(VALUE_MARK, ToldTable, "the mark that stands in the form of a sentence or of a question in the place of a field or a value");

pub(super) const FORM_GAP: &str = " ";
because!(FORM_GAP, ToldTable, "the sign between the words of a form when it is kept as one text");

pub(super) struct Reader {
    form: HashSet<String>,
}
because!(Reader, ToldTable, "the words of the form, counted from the told sentences of the lessons");

#[derive(Clone, Debug, Default)]
pub(super) struct ToldRows {
    pub(super) rows: Vec<Vec<String>>,
    pub(super) width: usize,
}
because!(ToldRows, ToldTable, "the rows of a story and the number of fields of each of them");

#[derive(Clone, Debug, Default)]
pub(super) struct Asked {
    pub(super) form: String,
    pub(super) values: Vec<String>,
}
because!(Asked, ToldTable, "a question as its form, with a mark in the place of each value, and its values in their order");

pub(super) fn amount(word: &str) -> Option<i64> {
    if !word.chars().next().is_some_and(|sign| sign.is_ascii_digit()) {
        return None;
    }
    let digits: String = word.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}
because!(amount, ToldTable, "the number of a word that starts with digits (a year, a count, or an ordinal word with its ending), and none for a word that starts with a letter, as a name with a number in it");

impl Reader {
    pub(super) fn learn(cases: &[Case]) -> Reader {
        let mut count: HashMap<String, usize> = HashMap::new();
        let mut sentences = 0usize;
        for case in cases {
            for sentence in &case.sentences {
                sentences += 1;
                let held: HashSet<String> = words(sentence).into_iter().collect();
                for word in held {
                    *count.entry(word).or_insert(0) += 1;
                }
            }
        }
        let form = count.into_iter().filter(|(_, held)| *held as f64 >= FORM_SHARE * sentences as f64).map(|(word, _)| word).collect();
        Reader { form }
    }

    fn sentence_form(&self, sentence: &str) -> (String, Vec<String>) {
        let mut form: Vec<&str> = Vec::new();
        let mut fields: Vec<String> = Vec::new();
        let cut = words(sentence);
        for word in &cut {
            if self.form.contains(word) {
                form.push(word);
            } else {
                form.push(VALUE_MARK);
                fields.push(word.clone());
            }
        }
        (form.join(FORM_GAP), fields)
    }

    pub(super) fn table_share(&self, cases: &[Case]) -> f64 {
        let mut forms: HashMap<String, usize> = HashMap::new();
        let mut all = 0usize;
        for case in cases {
            for sentence in &case.sentences {
                *forms.entry(self.sentence_form(sentence).0).or_insert(0) += 1;
                all += 1;
            }
        }
        forms.values().copied().max().unwrap_or(0) as f64 / all.max(1) as f64
    }

    pub(super) fn table(&self, sentences: &[String]) -> ToldRows {
        let rows: Vec<Vec<String>> = sentences.iter().map(|sentence| self.sentence_form(sentence).1).collect();
        let mut widths: HashMap<usize, usize> = HashMap::new();
        for row in &rows {
            *widths.entry(row.len()).or_insert(0) += 1;
        }
        let width = widths.iter().max_by(|one, other| one.1.cmp(other.1).then_with(|| other.0.cmp(one.0))).map_or(0, |(width, _)| *width);
        ToldRows { rows: rows.into_iter().filter(|row| row.len() == width).collect(), width }
    }

    pub(super) fn asked(&self, question: &str, table: &ToldRows) -> Asked {
        let held: HashSet<&String> = table.rows.iter().flatten().collect();
        let mut form: Vec<String> = Vec::new();
        let mut values: Vec<String> = Vec::new();
        for word in words(question) {
            if held.contains(&word) || amount(&word).is_some() {
                form.push(VALUE_MARK.to_string());
                values.push(word);
            } else {
                form.push(word);
            }
        }
        Asked { form: form.join(FORM_GAP), values }
    }
}
