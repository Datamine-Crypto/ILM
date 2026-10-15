use ilm_core::base::words::{answer_words, words};
use crate::controls::file::ChangedCase;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use crate::sets::export::Described;
use crate::sets::readers::inflect::inflect_large;
use patterns::{because, source};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct HeldChanges;
source!(HeldChanges, "changed texts of the tables used one after the other: the tables of a sample are read from the lessons that show one table alone, and a text is a row of bits with the names of the tables in the order of their use, so the truth of a changed text is the row and what each named table gives in turn. The row becomes each other row, each name of a table becomes each other name at each place in turn, and the two tables of a pair change places. The tables stand whole only in all lessons, so a model of all lessons reads the changes of its lessons, and a changed text that is a lesson is left out");

pub struct WordForms;
source!(WordForms, "changed cases of the forms of words: the large train file of a language gives the true form of a dictionary form with other sets of features, so a lesson that the model does not learn is changed into the same dictionary form with every other set of features that the large file holds for it, where no lesson of the model holds that text, and the large file gives the new truth");

const HELD_LOOKUP: &str = "src_lookup";
because!(HELD_LOOKUP, HeldChanges, "the set of the tables used one after the other, whose changes are made from all lessons");

const HELD_INFLECT: &str = "src_inflect";
because!(HELD_INFLECT, WordForms, "the set of the forms of words, whose changes are made from the lessons that the model does not learn");

const HELD_SETS: &str = "data/sets/";
because!(HELD_SETS, WordForms, "the folder of the descriptions of the sets, where the description of the set of a kind is found by its name when no other is named");

const HELD_TEXT_END: &str = ".txt";
because!(HELD_TEXT_END, WordForms, "the end of the name of the description of a set");

const HELD_FEATURE: char = 't';
because!(HELD_FEATURE, WordForms, "the first sign of the word of a feature, which a word of letters of more than one sign does not have");

const SORT_HELD_FEATURES: &str = "features";
because!(SORT_HELD_FEATURES, WordForms, "the sort of the change where a dictionary form gets another set of features");

const HELD_ALONE: usize = 2;
because!(HELD_ALONE, HeldChanges, "the words of a lesson that shows one table alone: the row and the name of the table");

const HELD_PAIR: usize = 3;
because!(HELD_PAIR, HeldChanges, "the words of a text with a pair of tables, whose two tables can change places");

const HELD_END: &str = " .";
because!(HELD_END, HeldChanges, "the end of a text of the set after its last name");

const HELD_GAP: &str = " ";
because!(HELD_GAP, HeldChanges, "the mark between the words of a changed text and of a truth");

const SORT_HELD_ROW: &str = "row";
because!(SORT_HELD_ROW, HeldChanges, "the sort of the change where the row of bits becomes another row");

const SORT_HELD_TABLE: &str = "table";
because!(SORT_HELD_TABLE, HeldChanges, "the sort of the change where one name of a table becomes another name");

const SORT_HELD_SIDES: &str = "sides";
because!(SORT_HELD_SIDES, HeldChanges, "the sort of the change where the two tables of a pair change places");

type HeldTables = BTreeMap<String, BTreeMap<String, String>>;
because!(HeldTables, HeldChanges, "each table by its name, with the row that it gives for each row, as the lessons that show one table alone give them");

fn held_tables(lessons: &[Case]) -> HeldTables {
    let mut tables: HeldTables = BTreeMap::new();
    for lesson in lessons {
        let (text, said) = (words(&lesson.question), answer_words(&lesson.answer));
        if let ([row, name], Some(next)) = (text.as_slice(), said.get(1))
            && text.len() == HELD_ALONE
        {
            tables.entry(name.clone()).or_default().insert(row.clone(), next.clone());
        }
    }
    tables
}
because!(held_tables, HeldChanges, "the row that each table gives for each row, read from the lessons that show one table alone, by the name of the table");

fn held_truth(tables: &HeldTables, text: &[String]) -> Option<Vec<String>> {
    let mut out: Vec<String> = vec![text.first()?.clone()];
    for name in &text[1..] {
        let next = tables.get(name)?.get(out.last()?)?.clone();
        out.push(next);
    }
    Some(out)
}
because!(held_truth, HeldChanges, "the truth of a text: its row and what each named table gives in turn; nothing when a table has no row for it");

fn held_changed(text: &[String], tables: &HeldTables) -> Vec<(&'static str, Vec<String>)> {
    let rows: BTreeSet<&String> = tables.values().flat_map(|table| table.keys()).collect();
    let mut out: Vec<(&'static str, Vec<String>)> = Vec::new();
    let Some(first) = text.first() else { return out };
    for row in rows.into_iter().filter(|row| *row != first) {
        let mut changed = text.to_vec();
        changed[0] = row.clone();
        out.push((SORT_HELD_ROW, changed));
    }
    for at in 1..text.len() {
        for name in tables.keys().filter(|name| **name != text[at]) {
            let mut changed = text.to_vec();
            changed[at] = name.clone();
            out.push((SORT_HELD_TABLE, changed));
        }
    }
    if text.len() == HELD_PAIR && text[1] != text[HELD_ALONE] {
        out.push((SORT_HELD_SIDES, vec![text[0].clone(), text[HELD_ALONE].clone(), text[1].clone()]));
    }
    out
}
because!(held_changed, HeldChanges, "the changed texts of a lesson, each with its sort: every other row, every other name at each place, and the pair turned round");

fn word_letters(text: &[String]) -> Vec<String> {
    text.iter().filter(|word| !(word.starts_with(HELD_FEATURE) && word.len() > 1)).cloned().collect()
}
because!(word_letters, WordForms, "the letters of the dictionary form of a text: its words that are no feature");

fn form_changes(lessons: &[Case], large: &[Case]) -> Vec<ChangedCase> {
    let (first, last) = inner_parts(lessons);
    let seen: HashSet<Vec<String>> = first.iter().map(|place| words(&lessons[*place].question)).collect();
    let mut by_word: HashMap<Vec<String>, BTreeMap<Vec<String>, (String, Vec<String>)>> = HashMap::new();
    for case in large {
        let text = words(&case.question);
        by_word.entry(word_letters(&text)).or_default().insert(text, (case.question.clone(), answer_words(&case.answer)));
    }
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let text = words(&lessons[place].question);
        let Some(found) = by_word.get(&word_letters(&text)) else { continue };
        for (other, (question, truth)) in found {
            if *other != text && !seen.contains(other) {
                out.push(ChangedCase { place: Some(place), sort: SORT_HELD_FEATURES.to_string(), sentences: Vec::new(), question: question.clone(), truth: truth.join(HELD_GAP) });
            }
        }
    }
    out
}
because!(form_changes, WordForms, "the changed cases of the lessons that the model does not learn: each other text of the large file with the same letters and other features that no lesson of the model holds, in the order of their words, with its form");

pub fn held_from_all(set: &str) -> bool {
    set == HELD_LOOKUP
}
because!(held_from_all, HeldChanges, "whether the changes of a set are made from all lessons, for a model of all lessons");

pub fn held_changes(lessons: &[Case], set: &str, kind: &str, about: Option<&Path>) -> Result<Vec<ChangedCase>, String> {
    if set == HELD_INFLECT {
        let place = about.map_or_else(|| PathBuf::from(format!("{HELD_SETS}{set}{HELD_TEXT_END}")), Path::to_path_buf);
        let large = inflect_large(&Described::read(&place)?, kind)?;
        return Ok(form_changes(lessons, &large));
    }
    if set != HELD_LOOKUP {
        return Err("this control writes the changes of the tables used one after the other and of the forms of words".to_string());
    }
    let tables = held_tables(lessons);
    let seen: HashSet<Vec<String>> = lessons.iter().map(|lesson| words(&lesson.question)).collect();
    let mut out: Vec<ChangedCase> = Vec::new();
    for (place, lesson) in lessons.iter().enumerate() {
        for (sort, changed) in held_changed(&words(&lesson.question), &tables) {
            let question = format!("{}{HELD_END}", changed.join(HELD_GAP));
            let text = words(&question);
            if seen.contains(&text) {
                continue;
            }
            let truth = held_truth(&tables, &text).ok_or_else(|| format!("no row of a table for the changed text {question}"))?;
            out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences: Vec::new(), question, truth: truth.join(HELD_GAP) });
        }
    }
    Ok(out)
}
because!(held_changes, HeldChanges, "the changed texts of all lessons of a kind of the tables that no lesson holds, each with the place of its lesson, its sort and its truth by the tables");
