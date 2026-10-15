use ilm_core::base::first_sight::FirstSightTable;
use ilm_core::base::words::{answer_words, words};
use crate::controls::file::ChangedCase;
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct RequestChanges;
source!(RequestChanges, "changed requests: a lesson request has a question for every slot that it has, so the place of every value in its text is known from the lessons alone, and the new truth of a changed request follows from the change. The values of two slots of one sort change places, where a whole value fills both slots in the lessons; the value of one slot becomes another value that the lessons hold for it, the next one in the order of the lessons; each word of the value of one slot becomes the new word that the set gives for it. After a change every question of the request is asked again");

const REQUEST_NONE: &str = "none";
because!(REQUEST_NONE, RequestChanges, "the answer of a slot question for a request that has no such slot");

const SORT_SIDES: &str = "sides";
because!(SORT_SIDES, RequestChanges, "the sort of the change where the values of two slots of one sort change places");

const SORT_OTHER: &str = "other";
because!(SORT_OTHER, RequestChanges, "the sort of the change where a value becomes another value of its slot");

const SORT_NEW: &str = "new";
because!(SORT_NEW, RequestChanges, "the sort of the change where the words of a value become new words");

const REQUEST_BETWEEN: &str = " ";
because!(REQUEST_BETWEEN, RequestChanges, "the mark between the words of a changed request and of a new truth");

type RequestTable = FirstSightTable<Vec<String>, FirstSightTable<String, Vec<String>>>;
because!(RequestTable, RequestChanges, "the requests of some lessons in the order of first sight, each with its questions and their answers; a later lesson with the same request and question gives its answer in place of the earlier one");

fn request_table<'a>(cases: impl Iterator<Item = &'a Case>) -> RequestTable {
    let mut table: RequestTable = FirstSightTable::new();
    for case in cases {
        let request: Vec<String> = case.sentences.iter().flat_map(|sentence| words(sentence)).collect();
        table.at(request).put(case.question.clone(), answer_words(&case.answer));
    }
    table
}
because!(request_table, RequestChanges, "the requests of some lessons with the answer of each question");

fn request_found(words: &[String], value: &[String], taken: &[bool]) -> Option<usize> {
    let ends = words.len().checked_sub(value.len())?;
    (0..=ends).find(|at| words[*at..*at + value.len()] == *value && !taken[*at..*at + value.len()].contains(&true))
}
because!(request_found, RequestChanges, "the first place where the words of a value stand in a request as one run on places that are not taken");

fn request_places(words: &[String], has: &[(String, Vec<String>)]) -> Option<HashMap<String, (usize, usize)>> {
    let mut order: Vec<&(String, Vec<String>)> = has.iter().collect();
    order.sort_by(|left, right| right.1.len().cmp(&left.1.len()).then_with(|| left.0.cmp(&right.0)));
    let mut taken = vec![false; words.len()];
    let mut out: HashMap<String, (usize, usize)> = HashMap::new();
    for (question, value) in order {
        let at = request_found(words, value, &taken)?;
        taken[at..at + value.len()].fill(true);
        out.insert(question.clone(), (at, at + value.len()));
    }
    Some(out)
}
because!(request_places, RequestChanges, "the place of the value of each slot of a request, the longest value first and of equal lengths the question first in the order of texts; nothing when a value does not stand in the text as one run");

type RequestChange = (&'static str, Vec<String>, Vec<(String, Vec<String>)>);
because!(RequestChange, RequestChanges, "one change of a request: its sort, the changed words, and the new answer of each question");

fn request_with(asked: &FirstSightTable<String, Vec<String>>, changes: &[(&String, &Vec<String>)]) -> Vec<(String, Vec<String>)> {
    asked.rows().iter().map(|(question, answer)| (question.clone(), changes.iter().find(|(held, _)| *held == question).map_or_else(|| answer.clone(), |(_, value)| (*value).clone()))).collect()
}
because!(request_with, RequestChanges, "the answers of the questions of a request after a change: the changed slots take their new values, the others keep theirs");

fn request_changed(words: &[String], asked: &FirstSightTable<String, Vec<String>>, places: &HashMap<String, (usize, usize)>, sorts: &HashMap<String, String>, values: &HashMap<String, Vec<(Vec<String>, Vec<String>)>>, table: &HashMap<String, String>) -> Vec<RequestChange> {
    let mut out: Vec<RequestChange> = Vec::new();
    let mut has: Vec<&String> = places.keys().collect();
    has.sort();
    let answer = |question: &String| asked.get(question).cloned().unwrap_or_default();
    for (at, one) in has.iter().enumerate() {
        for other in &has[at + 1..] {
            if sorts.get(*one) != sorts.get(*other) || answer(one) == answer(other) {
                continue;
            }
            let (mut first, mut second) = (*one, *other);
            if places[*one].0 >= places[*other].0 {
                (first, second) = (*other, *one);
            }
            let (early, late) = (places[first], places[second]);
            let mut changed: Vec<String> = words[..early.0].to_vec();
            changed.extend(answer(second));
            changed.extend_from_slice(&words[early.1..late.0]);
            changed.extend(answer(first));
            changed.extend_from_slice(&words[late.1..]);
            let (first_value, second_value) = (answer(first), answer(second));
            out.push((SORT_SIDES, changed, request_with(asked, &[(first, &second_value), (second, &first_value)])));
        }
    }
    for question in has {
        let (start, end) = places[question];
        let value = answer(question);
        if let Some((_, other)) = values.get(question).and_then(|row| row.iter().find(|(held, _)| *held == value)) {
            let mut changed: Vec<String> = words[..start].to_vec();
            changed.extend(other.iter().cloned());
            changed.extend_from_slice(&words[end..]);
            out.push((SORT_OTHER, changed, request_with(asked, &[(question, other)])));
        }
        let fresh: Vec<String> = value.iter().flat_map(|word| ilm_core::base::words::words(table.get(word).map_or(word.as_str(), String::as_str))).collect();
        if fresh != value {
            let mut changed: Vec<String> = words[..start].to_vec();
            changed.extend(fresh.iter().cloned());
            changed.extend_from_slice(&words[end..]);
            out.push((SORT_NEW, changed, request_with(asked, &[(question, &fresh)])));
        }
    }
    out
}
because!(request_changed, RequestChanges, "the changes of one request, in a fixed order: each pair of slots of one sort with different values, the slots in the order of their questions, then for each slot the change to another value and the change to new words");

pub fn request_changes(lessons: &[Case], whole: bool, table: &HashMap<String, String>) -> Vec<ChangedCase> {
    let none = answer_words(REQUEST_NONE);
    let (first, last): (Vec<usize>, Vec<usize>) = if whole { ((0..lessons.len()).collect(), (0..lessons.len()).collect()) } else { inner_parts(lessons) };
    let mut fills: FirstSightTable<String, HashSet<Vec<String>>> = FirstSightTable::new();
    let mut in_order: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    for (words, asked) in request_table(first.iter().map(|place| &lessons[*place])).rows() {
        let has: Vec<(String, Vec<String>)> = asked.rows().iter().filter(|(_, value)| *value != none).cloned().collect();
        let runs = request_places(words, &has).unwrap_or_default();
        for (question, value) in &has {
            fills.at(question.clone()).insert(value.clone());
            let row = in_order.entry(question.clone()).or_default();
            if !row.contains(value) && runs.contains_key(question) {
                row.push(value.clone());
            }
        }
    }
    let mut questions: Vec<String> = fills.rows().iter().map(|(question, _)| question.clone()).collect();
    questions.sort();
    let mut sorts: HashMap<String, String> = questions.iter().map(|question| (question.clone(), question.clone())).collect();
    for one in &questions {
        for other in &questions {
            let shared = fills.get(one).zip(fills.get(other)).is_some_and(|(mine, theirs)| mine.iter().any(|value| theirs.contains(value)));
            if one < other && shared {
                let (old, new) = (sorts[other].clone(), sorts[one].clone());
                for sort in sorts.values_mut() {
                    if *sort == old {
                        *sort = new.clone();
                    }
                }
            }
        }
    }
    let values: HashMap<String, Vec<(Vec<String>, Vec<String>)>> = in_order.iter().filter(|(_, row)| row.len() > 1).map(|(question, row)| (question.clone(), row.iter().enumerate().map(|(at, value)| (value.clone(), row[(at + 1) % row.len()].clone())).collect())).collect();
    let mut place_of: HashMap<(Vec<String>, String), usize> = HashMap::new();
    for place in &last {
        let case = &lessons[*place];
        let request: Vec<String> = case.sentences.iter().flat_map(|sentence| words(sentence)).collect();
        place_of.entry((request, case.question.clone())).or_insert(*place);
    }
    let mut made: Vec<ChangedCase> = Vec::new();
    for (request, asked) in request_table(last.iter().map(|place| &lessons[*place])).rows() {
        let has: Vec<(String, Vec<String>)> = asked.rows().iter().filter(|(_, value)| *value != none).cloned().collect();
        if has.is_empty() {
            continue;
        }
        let Some(places) = request_places(request, &has).filter(|places| !places.is_empty()) else { continue };
        if has.iter().any(|(question, _)| !sorts.contains_key(question)) || words(&request.join(REQUEST_BETWEEN)) != *request {
            continue;
        }
        for (sort, changed, truth) in request_changed(request, asked, &places, &sorts, &values, table) {
            for (question, _) in asked.rows() {
                let Some(place) = place_of.get(&(request.clone(), question.clone())) else { continue };
                let said = truth.iter().find(|(held, _)| held == question).map(|(_, answer)| answer.join(REQUEST_BETWEEN)).unwrap_or_default();
                made.push(ChangedCase { place: Some(*place), sort: sort.to_string(), sentences: vec![changed.join(REQUEST_BETWEEN)], question: question.clone(), truth: said });
            }
        }
    }
    made
}
because!(request_changes, RequestChanges, "the changed requests of a kind from its lessons: from the lessons of the model the sorts of slots (two slots are of one sort when a value fills both) and the values of each slot in order; from the part that is held back, or from all lessons, each change of each request with every question of it, in the order of the requests");
