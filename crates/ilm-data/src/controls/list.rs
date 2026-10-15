use ilm_core::base::first_sight::{FirstSightCount, FirstSightTable};
use ilm_core::base::words::answer_words;
use crate::controls::file::ChangedCase;
use crate::sets::readers::common::python_words;
use ilm_core::know::signs::is_space;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::HashMap;

pub struct ListChanges;
source!(ListChanges, "changed counts over a told list: the told sentence is cut at its commas into parts. Counted from the lessons: the words that start every first part, the word that starts most last parts and no part between, and the most frequent first word of the other parts, the unit word. The number behind an answer is the smallest number of parts of a lesson with this answer whose parts all start with the unit word, and a lesson with just this number of parts is plain: every thing of it belongs. So the truth of a changed list is known: the parts in the opposite order keep it; the first part taken out of a plain lesson gives one less; the unit word of the first part of a plain lesson made the answer of a larger number gives more; and the parts of the next plain lesson with the same question put after the list add their number. A change whose new answer no lesson has is not made");

const LIST_CUT: &str = ", ";
because!(LIST_CUT, ListChanges, "the mark between the parts of a told list");

const LIST_SENTENCE_END: char = '.';
because!(LIST_SENTENCE_END, ListChanges, "the mark that ends a told list");

const LIST_GAP: &str = " ";
because!(LIST_GAP, ListChanges, "the mark between the words of a part and between the lead and the list");

const LIST_GAP_SIGN: char = ' ';
because!(LIST_GAP_SIGN, ListChanges, "the mark after the first word of a part, where the amount of a part ends");

const LIST_PART_WORDS: usize = 2;
because!(LIST_PART_WORDS, ListChanges, "the fewest words of a part: an amount or a unit word, and a thing");

const LIST_ENDS: usize = 2;
because!(LIST_ENDS, ListChanges, "the parts of a list that are not between others: the first part and the last part");

const LIST_MORE: [usize; 2] = [2, 3];
because!(LIST_MORE, ListChanges, "the larger amounts that the first part of a plain list is given, so that a learner must add more than one");

const LIST_ORDER: &str = "order";
because!(LIST_ORDER, ListChanges, "the sort of the change where the parts stand in the opposite order");

const LIST_OUT: &str = "out";
because!(LIST_OUT, ListChanges, "the sort of the change where the first part of a plain list is taken out");

const LIST_AMOUNT: &str = "amount";
because!(LIST_AMOUNT, ListChanges, "the sort of the change where the first part of a plain list gets a larger amount");

const LIST_BOTH_PLAIN: &str = "two lists, both plain";
because!(LIST_BOTH_PLAIN, ListChanges, "the sort of the change where the parts of another plain list follow a plain list");

const LIST_NOT_BELONG: &str = "two lists, the first with things that do not belong";
because!(LIST_NOT_BELONG, ListChanges, "the sort of the change where the parts of a plain list follow a list with things that do not belong, which stay in it");

struct ListForm {
    lead: String,
    join: String,
    unit: String,
    number_of: FirstSightTable<Vec<String>, usize>,
    answer_of: HashMap<usize, String>,
}
because!(ListForm, ListChanges, "the form of the lists of a kind as the lessons give it: the lead words, the joining word, the unit word, the number behind each answer in the order of first sight, and the answer of each number. counted makes the form from the lessons; read gives the parts of a told list without the lead and the joining word, or nothing for a told text of another form; write makes the told sentence of some parts; plain says whether every part of a list starts with the unit word and the list is as long as the number behind its answer");

fn list_cut(sentence: &str) -> Vec<&str> {
    sentence.trim_end_matches(is_space).trim_end_matches(LIST_SENTENCE_END).split(LIST_CUT).collect()
}
because!(list_cut, ListChanges, "the parts of a told list, without the space and the marks at its end");

fn first_word(part: &str) -> String {
    python_words(part).first().map_or_else(String::new, |word| word.to_string())
}
because!(first_word, ListChanges, "the first word of a part, or no word for a part with none");

impl ListForm {
    fn counted(lessons: &[Case]) -> ListForm {
        let rows: Vec<Vec<&str>> = lessons.iter().filter(|case| case.sentences.len() == 1).map(|case| list_cut(&case.sentences[0])).collect();
        let firsts: Vec<Vec<&str>> = rows.iter().map(|row| python_words(row[0])).collect();
        let mut lead = 0;
        while !firsts.is_empty() && firsts.iter().all(|first| first.len() > lead + 1) && firsts.iter().all(|first| first[lead] == firsts[0][lead]) {
            lead += 1;
        }
        let lead = firsts.first().map_or_else(String::new, |first| first[..lead].join(LIST_GAP));
        let mut between: FirstSightCount<String> = FirstSightCount::new();
        for row in &rows {
            for part in row.iter().skip(1).take(row.len().saturating_sub(LIST_ENDS)) {
                between.add(first_word(part), 1);
            }
        }
        let mut last: FirstSightCount<String> = FirstSightCount::new();
        for row in rows.iter().filter(|row| row.len() > 1) {
            last.add(first_word(row[row.len() - 1]), 1);
        }
        let join = last.by_count().into_iter().map(|(word, _)| word).find(|word| !between.holds(word)).cloned().unwrap_or_default();
        let unit = between.first_most().map(|(word, _)| word.clone()).unwrap_or_default();
        let mut form = ListForm { lead, join, unit, number_of: FirstSightTable::new(), answer_of: HashMap::new() };
        let mut texts: HashMap<Vec<String>, String> = HashMap::new();
        for case in lessons {
            let Some(items) = form.read(&case.sentences) else { continue };
            if items.iter().all(|item| first_word(item) == form.unit) {
                let key = answer_words(&case.answer);
                let least = form.number_of.get(&key).map_or(items.len(), |held| items.len().min(*held));
                form.number_of.put(key.clone(), least);
                texts.insert(key, case.answer.clone());
            }
        }
        for (key, number) in form.number_of.rows() {
            form.answer_of.insert(*number, texts[key].clone());
        }
        form
    }

    fn read(&self, sentences: &[String]) -> Option<Vec<String>> {
        let [sentence] = sentences else { return None };
        if !sentence.trim_end_matches(is_space).ends_with(LIST_SENTENCE_END) {
            return None;
        }
        let mut parts: Vec<String> = list_cut(sentence).into_iter().map(String::from).collect();
        let lead = format!("{}{LIST_GAP}", self.lead);
        let join = format!("{}{LIST_GAP}", self.join);
        let joined = parts.len() > 1 && !self.join.is_empty();
        if !parts[0].starts_with(&lead) || (joined && !parts[parts.len() - 1].starts_with(&join)) {
            return None;
        }
        parts[0] = parts[0][lead.len()..].to_string();
        if joined {
            let end = parts.len() - 1;
            parts[end] = parts[end][join.len()..].to_string();
        }
        parts.iter().all(|part| python_words(part).len() >= LIST_PART_WORDS).then_some(parts)
    }

    fn write(&self, items: &[String]) -> Vec<String> {
        let mut parts: Vec<String> = items.to_vec();
        if parts.len() > 1 && !self.join.is_empty() {
            let end = parts.len() - 1;
            parts[end] = format!("{}{LIST_GAP}{}", self.join, parts[end]);
        }
        vec![format!("{}{LIST_GAP}{}{LIST_SENTENCE_END}", self.lead, parts.join(LIST_CUT))]
    }

    fn plain(&self, sentences: &[String], answer: &str) -> bool {
        let Some(items) = self.read(sentences) else { return false };
        !items.is_empty() && items.iter().all(|item| first_word(item) == self.unit) && self.number_of.get(&answer_words(answer)) == Some(&items.len())
    }
}

pub fn list_changes(lessons: &[Case], whole: bool) -> Vec<ChangedCase> {
    let form = ListForm::counted(lessons);
    let mut is_plain: HashMap<(&[String], &str), bool> = HashMap::new();
    for case in lessons {
        is_plain.insert((&case.sentences, &case.question), form.plain(&case.sentences, &case.answer));
    }
    let plain_of = |case: &Case| is_plain.get(&(case.sentences.as_slice(), case.question.as_str())).copied().unwrap_or_default();
    let part: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let mut out: Vec<ChangedCase> = Vec::new();
    for (at, place) in part.iter().enumerate() {
        let case = &lessons[*place];
        let Some(items) = form.read(&case.sentences) else { continue };
        let mut add = |sort: &str, sentences: Vec<String>, truth: Option<&String>| {
            if let Some(truth) = truth {
                out.push(ChangedCase { place: Some(*place), sort: sort.to_string(), sentences, question: case.question.clone(), truth: truth.clone() });
            }
        };
        let turned: Vec<String> = items.iter().rev().cloned().collect();
        add(LIST_ORDER, form.write(&turned), Some(&case.answer));
        let number = form.number_of.get(&answer_words(&case.answer)).copied();
        let plain = plain_of(case);
        if plain {
            if items.len() > 1 {
                add(LIST_OUT, form.write(&items[1..]), form.answer_of.get(&(items.len() - 1)));
            }
            for more in LIST_MORE {
                let (Some(amount), Some((_, thing))) = (form.answer_of.get(&more), items[0].split_once(LIST_GAP_SIGN)) else { continue };
                let mut changed = vec![format!("{amount}{LIST_GAP}{thing}")];
                changed.extend(items[1..].iter().cloned());
                add(LIST_AMOUNT, form.write(&changed), form.answer_of.get(&(items.len() - 1 + more)));
            }
        }
        let others = part[at + 1..].iter().chain(&part[..at]).map(|other| &lessons[*other]);
        let other = others.filter(|other| other.question == case.question && plain_of(other)).map(|other| form.read(&other.sentences)).next().flatten();
        if let (Some(other), Some(number)) = (other, number)
            && !other.is_empty()
        {
            let sort = if plain { LIST_BOTH_PLAIN } else { LIST_NOT_BELONG };
            let mut joined = items.clone();
            joined.extend(other.iter().cloned());
            add(sort, form.write(&joined), form.answer_of.get(&(number + other.len())));
        }
    }
    out
}
because!(list_changes, ListChanges, "the changed lists of the lessons that the model does not learn, or of all lessons for a model of all lessons, each with the true answer that the form gives");
