use ilm_core::cases::Case;
use crate::sets::csv::{field_named, named_rows};
use crate::sets::export::{Described, KindCases};
use crate::sets::readers::common::{LESSON_FILES, NEW_WORD_START, READ_FILES, python_words, stripped, text_of, told_sentences};
use crate::sets::textmatch::TextPattern;
use patterns::{because, source};
use std::collections::BTreeSet;

pub struct BeliefStatements;
source!(BeliefStatements, "ToMi of Le and others in the copy of the hub that gives each question as a statement to judge, as tab separated files (train, val, test) with the story, the true statement and the false statement of each row: a row whose true statement holds the fixed words of the kind gives two cases, the story with the true statement and the answer entailment, and the story with the false statement and the answer not_entailment, unless both statements end with the same place; the new words are the names, rooms, things and containers that the events of the train stories name, but the colours of the places, each with zq before it, in the order of the words");

const TOMI_PREFIX: &str = "tomi_";
because!(TOMI_PREFIX, BeliefStatements, "the start of the name of a kind of ToMi, before the sort of its statement");

const TOMI_MARKS: &[(&str, &str)] = &[("real", " really is in "), ("begin", " at the beginning was in "), ("look", " look for "), ("think", " think that ")];
because!(TOMI_MARKS, BeliefStatements, "the fixed words of each sort of statement: where the thing really is, where it was at the beginning, where a person looks for it, what a person thinks that another one does");

const TOMI_FIELDS: char = '\t';
because!(TOMI_FIELDS, BeliefStatements, "the mark between the fields of a row of the files");

const STORY_FIELD: &str = "premise";
because!(STORY_FIELD, BeliefStatements, "the column whose text the told sentences of a case are cut from");

const TRUE_FIELD: &str = "valid_hypothesis";
because!(TRUE_FIELD, BeliefStatements, "the column of the statement that the story makes so, the question of the case with the answer entailment");

const FALSE_FIELD: &str = "invalid_hypothesis";
because!(FALSE_FIELD, BeliefStatements, "the column of the statement that the story does not make so, the question of the case with the answer not_entailment");

const TRUE_ANSWER: &str = "entailment";
because!(TRUE_ANSWER, BeliefStatements, "the answer of a case with the true statement, a word of the label field of the copy");

const FALSE_ANSWER: &str = "not_entailment";
because!(FALSE_ANSWER, BeliefStatements, "the answer of a case with the false statement, a word of the label field of the copy");

const MARK_GAP: &str = " ";
because!(MARK_GAP, BeliefStatements, "the space put before and after a statement, so that the fixed words of a sort also match at its ends");

const COLOUR_END: char = '_';
because!(COLOUR_END, BeliefStatements, "the mark after the colour in the last word of a true statement, which names a place as the colour and the container");

const ENTERED: &str = r"(\w+) (?:entered|exited) the ([\w ]+)\.$";
because!(ENTERED, BeliefStatements, "the event of a person who enters or leaves a room: the name and the words of the room");

const MOVED: &str = r"(\w+) moved the ([\w ]+?) to the ([\w ]+)\.$";
because!(MOVED, BeliefStatements, "the event of a person who moves a thing to a container: the name, the words of the thing, and the words of the container, whose first word is its colour");

const PLACED: &str = r"The ([\w ]+?) is in the ([\w ]+)\.$";
because!(PLACED, BeliefStatements, "the sentence that a thing is in a container: the words of the thing and the words of the container, whose first word is its colour");

fn statement_cases(place: &str, mark: &str) -> Result<Vec<Case>, String> {
    let mut out: Vec<Case> = Vec::new();
    for row in named_rows(&text_of(place)?, TOMI_FIELDS) {
        let field = |name: &str| field_named(&row, name).ok_or_else(|| format!("{place}: a row with no field {name}"));
        let (yes, no) = (stripped(field(TRUE_FIELD)?), stripped(field(FALSE_FIELD)?));
        let last = |text: &str| python_words(text).last().map(|word| word.to_string());
        if !format!("{MARK_GAP}{yes}{MARK_GAP}").contains(mark) || last(yes) == last(no) {
            continue;
        }
        let told = told_sentences(field(STORY_FIELD)?);
        out.push(Case { sentences: told.clone(), question: yes.to_string(), answer: TRUE_ANSWER.to_string() });
        out.push(Case { sentences: told, question: no.to_string(), answer: FALSE_ANSWER.to_string() });
    }
    Ok(out)
}
because!(statement_cases, BeliefStatements, "the two cases of each row of a file whose true statement holds the fixed words of the sort and whose two statements end with other places");

fn tomi_new_words(place: &str) -> Result<Vec<(String, String)>, String> {
    let [entered, moved, placed] = [ENTERED, MOVED, PLACED].map(TextPattern::new);
    let (entered, moved, placed) = (entered?, moved?, placed?);
    let rows = named_rows(&text_of(place)?, TOMI_FIELDS);
    let mut found: BTreeSet<String> = BTreeSet::new();
    let mut colours: BTreeSet<String> = BTreeSet::new();
    for row in &rows {
        let field = |name: &str| field_named(row, name).ok_or_else(|| format!("{place}: a row with no field {name}"));
        for piece in told_sentences(field(STORY_FIELD)?) {
            let words = |group: &Option<String>, from: usize| python_words(group.as_deref().unwrap_or_default()).into_iter().skip(from).map(str::to_string).collect::<Vec<String>>();
            if let Some([_, name, room]) = entered.find(&piece, false).as_deref() {
                found.extend(words(name, 0));
                found.extend(words(room, 0));
            } else if let Some([_, name, thing, container]) = moved.find(&piece, false).as_deref() {
                found.extend(words(name, 0));
                found.extend(words(thing, 0));
                found.extend(words(container, 1));
            } else if let Some([_, thing, container]) = placed.find(&piece, false).as_deref() {
                found.extend(words(thing, 0));
                found.extend(words(container, 1));
            }
        }
        let last = python_words(field(TRUE_FIELD)?).last().map(|word| word.to_string()).unwrap_or_default();
        colours.insert(last.split(COLOUR_END).next().unwrap_or_default().to_string());
    }
    let mut table: Vec<(String, String)> = Vec::new();
    for word in found.iter().map(|word| word.to_lowercase()) {
        if !colours.contains(&word) && !table.iter().any(|(held, _)| *held == word) {
            table.push((word.clone(), format!("{NEW_WORD_START}{word}")));
        }
    }
    Ok(table)
}
because!(tomi_new_words, BeliefStatements, "the table of new words of the set from the train file: each word that an event names in small letters, the colours of the places left out, in the order of the words as they stand");

pub fn tomi_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let sort = kind.strip_prefix(TOMI_PREFIX).ok_or_else(|| format!("the kind {kind} is no kind of this set"))?;
    let mark = TOMI_MARKS.iter().find(|(held, _)| *held == sort).map(|(_, mark)| *mark).ok_or_else(|| format!("no sort of statement {sort}"))?;
    let lessons = described.files(kind, LESSON_FILES);
    let mut cases: Vec<Case> = Vec::new();
    for place in &lessons {
        cases.extend(statement_cases(place, mark)?);
    }
    let read_from = Some(cases.len());
    for place in described.files(kind, READ_FILES) {
        cases.extend(statement_cases(&place, mark)?);
    }
    let first = lessons.first().ok_or("no train file")?;
    Ok(KindCases { cases, read_from, new_words: Some(tomi_new_words(first)?), ..KindCases::default() })
}
because!(tomi_cases, BeliefStatements, "the cases of a kind of ToMi, the train file first, then the files to read, with the table of new words of the train file");
