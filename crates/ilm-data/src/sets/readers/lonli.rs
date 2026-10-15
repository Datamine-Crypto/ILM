use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::parquet::records::Value;
use crate::sets::readers::common::{NEW_WORD_START, rows_where, text_value, told_sentences};
use patterns::{because, source};
use std::collections::BTreeSet;

pub struct LonliSet;
source!(LonliSet, "LoNLI of Tarunesh and others: one or two told sentences and a statement to judge, in seventeen groups of reasoning, one parquet file whose column file names the form of each case and its group before the last hyphen; the new words of the set are the words that it writes with a capital letter inside a sentence and never in small letters, each with zq before it");

const LONLI_PREFIX: &str = "lonli_";
because!(LONLI_PREFIX, LonliSet, "the start of the name of a kind of the set, before its group");

const FORM_END: char = '-';
because!(FORM_END, LonliSet, "the mark before the number of the form of a case in its column file, after the group");

const ALL_ROLE: &str = "all";
because!(ALL_ROLE, LonliSet, "the role in the description of the one file of the set");

const LONLI_COLUMNS: usize = 4;
because!(LONLI_COLUMNS, LonliSet, "the columns of a row that the reader takes: the premise, the statement, the label and the form");

const SHORTEST_NAME: usize = 3;
because!(SHORTEST_NAME, LonliSet, "the letters of a word with a capital at least for it to get a new word: a shorter one is mostly a word of the language");

fn lonli_rows(described: &Described, kind: &str) -> Result<Vec<[Value; LONLI_COLUMNS]>, String> {
    rows_where(&described.file(kind, ALL_ROLE)?, ["premise", "hypothesis", "label", "file"], None)
}
because!(lonli_rows, LonliSet, "every row of the set with its premise, its statement, its label and its form");

fn alphabet_runs(text: &str) -> Vec<&str> {
    text.split(|sign: char| !sign.is_ascii_alphabetic()).filter(|piece| !piece.is_empty()).collect()
}
because!(alphabet_runs, LonliSet, "the runs of letters of the plain alphabet in a text");

fn lonli_new_words(rows: &[[Value; LONLI_COLUMNS]]) -> Result<Vec<(String, String)>, String> {
    let mut inside: BTreeSet<String> = BTreeSet::new();
    let mut lower: BTreeSet<String> = BTreeSet::new();
    for [premise, hypothesis, _, _] in rows {
        for text in [premise, hypothesis] {
            for sentence in told_sentences(&text_value(text, ALL_ROLE)?) {
                let words = alphabet_runs(&sentence);
                for word in words.iter().skip(1) {
                    let mut signs = word.chars();
                    let first = signs.next().unwrap_or_default();
                    let rest = signs.as_str();
                    if first.is_ascii_uppercase() && !rest.is_empty() && rest.chars().all(|sign| sign.is_ascii_lowercase()) {
                        inside.insert(word.to_string());
                    }
                }
                lower.extend(words.iter().filter(|word| word.chars().all(|sign| sign.is_ascii_lowercase())).map(|word| word.to_string()));
            }
        }
    }
    Ok(inside.into_iter().filter(|word| word.len() >= SHORTEST_NAME && !lower.contains(&word.to_lowercase())).map(|word| {
        let low = word.to_lowercase();
        (low.clone(), format!("{NEW_WORD_START}{low}"))
    }).collect())
}
because!(lonli_new_words, LonliSet, "the table of new words of the set: each word written with a capital letter and small letters after it, not the first word of its sentence, that is long enough and that the set never writes in small letters, in the order of the words");

pub fn lonli_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let group = kind.strip_prefix(LONLI_PREFIX).ok_or_else(|| format!("the kind {kind} is no kind of this set"))?;
    let rows = lonli_rows(described, kind)?;
    let mut cases: Vec<Case> = Vec::new();
    for [premise, hypothesis, label, form] in &rows {
        let form = text_value(form, kind)?;
        if form.rsplit_once(FORM_END).map_or(form.as_str(), |(head, _)| head) == group {
            cases.push(Case { sentences: told_sentences(&text_value(premise, kind)?), question: text_value(hypothesis, kind)?, answer: text_value(label, kind)? });
        }
    }
    Ok(KindCases { cases, read_from: None, new_words: Some(lonli_new_words(&rows)?), ..KindCases::default() })
}
because!(lonli_cases, LonliSet, "the cases of a group of the set in the order of the file, the premise cut into sentences, the statement as the question, and the table of new words of the whole set");
