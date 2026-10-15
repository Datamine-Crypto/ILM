use crate::controls::file::ChangedCase;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct FixedFormGroups;
source!(FixedFormGroups, "the groups of a set that is made from fixed forms hold the changed cases themselves: two cases with the same form of statement and different labels differ in their told text alone, and two cases with the same told form and different labels differ in their statement alone. A case is read as it is when its form of statement, or its told form, has more than one label in the lessons, since a learner that reads one side alone can not be right for all such cases. A form is the text in small letters with each name and each number made a mark; a name is a word with a capital letter that the lessons never write in small letters, so the first word of a sentence is no name when the lessons also write it in small letters");

const LONLI_NUMBER_MARK: char = '0';
because!(LONLI_NUMBER_MARK, FixedFormGroups, "what a form writes in place of each number, so that statements that differ in their numbers alone have one form");

const LONLI_NAME_MARK: &str = "N";
because!(LONLI_NAME_MARK, FixedFormGroups, "what a form writes in place of each name, so that statements that differ in their names alone have one form");

const LONLI_GAP: &str = " ";
because!(LONLI_GAP, FixedFormGroups, "the mark between the words of a form");

const LONLI_TOLD_GAP: &str = " / ";
because!(LONLI_TOLD_GAP, FixedFormGroups, "the mark between the forms of the told sentences of a case in its told form");

const LONLI_LABELS: usize = 2;
because!(LONLI_LABELS, FixedFormGroups, "the labels that a form must have in the lessons, at the least, so that the other side of the case decides its label");

const SORT_TOLD_DECIDES: &str = "the told text decides";
because!(SORT_TOLD_DECIDES, FixedFormGroups, "the group of cases whose form of statement has more than one label, so that the told text decides");

const SORT_STATEMENT_DECIDES: &str = "the statement decides";
because!(SORT_STATEMENT_DECIDES, FixedFormGroups, "the group of cases whose told form has more than one label, so that the statement decides");

fn form_tokens(text: &str) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut out: Vec<&str> = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let test: fn(&u8) -> bool = if bytes[at].is_ascii_alphabetic() {
            u8::is_ascii_alphabetic
        } else if bytes[at].is_ascii_digit() {
            u8::is_ascii_digit
        } else {
            at += 1;
            continue;
        };
        let start = at;
        while at < bytes.len() && test(&bytes[at]) {
            at += 1;
        }
        out.push(&text[start..at]);
    }
    out
}
because!(form_tokens, FixedFormGroups, "the words of a text: the runs of letters of the Latin alphabet and the runs of digits");

fn form_names(lessons: &[Case]) -> HashSet<String> {
    let (mut capital, mut small): (HashSet<String>, HashSet<String>) = (HashSet::new(), HashSet::new());
    for case in lessons {
        for text in case.sentences.iter().chain([&case.question]) {
            for word in form_tokens(text) {
                let low = word.to_lowercase();
                if word.as_bytes()[0].is_ascii_uppercase() {
                    capital.insert(low);
                } else {
                    small.insert(low);
                }
            }
        }
    }
    capital.difference(&small).cloned().collect()
}
because!(form_names, FixedFormGroups, "the names of a kind: the words with a capital first letter that the lessons never write in small letters");

fn text_form(text: &str, names: &HashSet<String>) -> String {
    let marks: Vec<String> = form_tokens(text)
        .into_iter()
        .map(|word| {
            let low = word.to_lowercase();
            if word.as_bytes()[0].is_ascii_digit() {
                LONLI_NUMBER_MARK.to_string()
            } else if names.contains(&low) {
                LONLI_NAME_MARK.to_string()
            } else {
                low
            }
        })
        .collect();
    marks.join(LONLI_GAP)
}
because!(text_form, FixedFormGroups, "the form of a text: its words in small letters with each number and each name made a mark");

fn told_form(sentences: &[String], names: &HashSet<String>) -> String {
    let forms: Vec<String> = sentences.iter().map(|sentence| text_form(sentence, names)).collect();
    forms.join(LONLI_TOLD_GAP)
}
because!(told_form, FixedFormGroups, "the told form of a case: the forms of its told sentences in their order");

pub fn lonli_changes(lessons: &[Case], read: &[Case], read_part: bool) -> Vec<ChangedCase> {
    let names = form_names(lessons);
    let mut by_statement: HashMap<String, HashSet<&str>> = HashMap::new();
    let mut by_told: HashMap<String, HashSet<&str>> = HashMap::new();
    for case in lessons {
        by_statement.entry(text_form(&case.question, &names)).or_default().insert(case.answer.as_str());
        by_told.entry(told_form(&case.sentences, &names)).or_default().insert(case.answer.as_str());
    }
    let cases: Vec<&Case> = if read_part { read.iter().collect() } else { inner_parts(lessons).1.into_iter().map(|place| &lessons[place]).collect() };
    let mut out: Vec<ChangedCase> = Vec::new();
    for case in cases {
        let groups = [(SORT_TOLD_DECIDES, &by_statement, text_form(&case.question, &names)), (SORT_STATEMENT_DECIDES, &by_told, told_form(&case.sentences, &names))];
        for (sort, table, key) in groups {
            if table.get(&key).is_some_and(|labels| labels.len() >= LONLI_LABELS) {
                out.push(ChangedCase { place: None, sort: sort.to_string(), sentences: case.sentences.clone(), question: case.question.clone(), truth: case.answer.clone() });
            }
        }
    }
    out
}
because!(lonli_changes, FixedFormGroups, "the cases of the two groups of a kind, each read as it is: of the lessons that the model does not learn, or of the read part with a model of all lessons");
