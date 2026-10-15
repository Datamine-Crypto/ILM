use ilm_core::cases::Case;
use crate::sets::csv::{field_named, named_rows};
use crate::sets::export::{Described, KindCases};
use crate::sets::readers::common::{NEW_WORD_START, python_words, text_of};
use patterns::{because, source};
use std::collections::BTreeSet;

pub struct RequestSets;
source!(RequestSets, "requests of people with their intents and the slots that runs of their words fill, one label for each word (the beginning of a run, the inside of a run, or outside), as rows of two files of the hub, ATIS of Hemphill and others and SNIPS of Coucke and others: the slot kind asks for each slot that a request has and for as many slots that it lacks, taken from the next requests of the file, with the answer none; the intent kind asks for the intent; the new words are the words of the names in the requests that stand nowhere else, each with zq before it");

const ATIS_SLOTS: &str = "atis_slots";
because!(ATIS_SLOTS, RequestSets, "the kind of ATIS that asks for the words of slots; the other kind of the set asks for the intent");

const SNIPS_SLOTS: &str = "snips_slots";
because!(SNIPS_SLOTS, RequestSets, "the kind of SNIPS that asks for the words of slots; the other kind of the set asks for the intent");

const ATIS_TEXT: &str = "text";
because!(ATIS_TEXT, RequestSets, "the column of the words of a request of ATIS");

const SNIPS_TEXT: &str = "input";
because!(SNIPS_TEXT, RequestSets, "the column of the words of a request of SNIPS");

const SLOTS_COLUMN: &str = "slots";
because!(SLOTS_COLUMN, RequestSets, "the column of the labels of the words of a request, one for each word");

const INTENT_COLUMN: &str = "intent";
because!(INTENT_COLUMN, RequestSets, "the column with what the person asks the service to do, which the intent kind gives as its answer");

const INTENT_QUESTION: &str = "intent";
because!(INTENT_QUESTION, RequestSets, "the one question of every case of an intent kind, so that the case asks what the request wants");

const NONE_ANSWER: &str = "none";
because!(NONE_ANSWER, RequestSets, "the answer for a slot that a request does not fill");

const BEGIN_LABEL: &str = "B-";
because!(BEGIN_LABEL, RequestSets, "the start of the label of the first word of a run that fills a slot");

const INSIDE_LABEL: &str = "I-";
because!(INSIDE_LABEL, RequestSets, "the start of the label of a word inside a run that fills a slot");

const ATIS_NAME_END: &str = "_name";
because!(ATIS_NAME_END, RequestSets, "the end of the type of a slot of ATIS that holds a name: a city, an airport, a state, a country, an airline");

const NOT_NAMES: &[&str] = &["day_name", "month_name"];
because!(NOT_NAMES, RequestSets, "the types of slots of ATIS that end like a name and are no name of a place or a company");

const SLOT_PARTS: char = '.';
because!(SLOT_PARTS, RequestSets, "the mark between the role and the type in the name of a slot of ATIS");

const SNIPS_NAMES: &[&str] = &["album", "artist", "city", "country", "entity_name", "geographic_poi", "location_name", "movie_name", "object_name", "playlist", "restaurant_name", "state", "track"];
because!(SNIPS_NAMES, RequestSets, "the slots of SNIPS that hold a city, a place, a name or a title");

const DELIMITER: char = ',';
because!(DELIMITER, RequestSets, "the mark between the fields of a row of the files");

const REQUEST_TRAIN: &str = "train";
because!(REQUEST_TRAIN, RequestSets, "the role in the description of the train file, whose requests are the lessons");

const REQUEST_TEST: &str = "test";
because!(REQUEST_TEST, RequestSets, "the role in the description of the test file, whose requests are read");

const REQUEST_WORD_GAP: &str = " ";
because!(REQUEST_WORD_GAP, RequestSets, "the mark between the words of a request and of an answer");

struct Request {
    text: String,
    intent: String,
    runs: Vec<(String, Vec<String>)>,
}
because!(Request, RequestSets, "one request: its words as one text, its intent, and the runs of words that fill slots, each with its slot");

fn requests(place: &str, column: &str) -> Result<Vec<Request>, String> {
    let mut out: Vec<Request> = Vec::new();
    for row in named_rows(&text_of(place)?, DELIMITER) {
        let field = |name: &str| field_named(&row, name).ok_or_else(|| format!("{place}: a row with no field {name}"));
        let words = python_words(field(column)?);
        let labels = python_words(field(SLOTS_COLUMN)?);
        if words.len() != labels.len() || words.is_empty() {
            continue;
        }
        let mut runs: Vec<(String, Vec<String>)> = Vec::new();
        let mut open = false;
        for (word, label) in words.iter().zip(&labels) {
            let inside = label.strip_prefix(INSIDE_LABEL);
            if let Some(slot) = label.strip_prefix(BEGIN_LABEL).or(inside.filter(|slot| !open || runs.last().is_none_or(|run| run.0 != *slot))) {
                runs.push((slot.to_string(), vec![word.to_string()]));
                open = true;
            } else if inside.is_some() {
                if let Some(run) = runs.last_mut() {
                    run.1.push(word.to_string());
                }
            } else {
                open = false;
            }
        }
        out.push(Request { text: words.join(REQUEST_WORD_GAP), intent: field(INTENT_COLUMN)?.to_string(), runs });
    }
    Ok(out)
}
because!(requests, RequestSets, "the requests of a file in its order: a row whose words and labels differ in number is left out; a run starts at a beginning label, or at an inside label of another slot than the open run, or with no open run");

fn slot_cases(requests: &[Request]) -> Vec<Case> {
    let have: Vec<Vec<&str>> = requests
        .iter()
        .map(|request| {
            let mut slots: Vec<&str> = Vec::new();
            for (slot, _) in &request.runs {
                if !slots.contains(&slot.as_str()) {
                    slots.push(slot);
                }
            }
            slots
        })
        .collect();
    let mut out: Vec<Case> = Vec::new();
    for (place, request) in requests.iter().enumerate() {
        let told = vec![request.text.clone()];
        for slot in &have[place] {
            let words: Vec<&str> = request.runs.iter().filter(|(held, _)| held == slot).flat_map(|(_, words)| words.iter().map(String::as_str)).collect();
            out.push(Case { sentences: told.clone(), question: slot.to_string(), answer: words.join(REQUEST_WORD_GAP) });
        }
        let mut lacks: Vec<&str> = Vec::new();
        let mut next = (place + 1) % requests.len();
        while lacks.len() < have[place].len() && next != place {
            let room = have[place].len() - lacks.len();
            let fresh: Vec<&str> = have[next].iter().copied().filter(|slot| !have[place].contains(slot) && !lacks.contains(slot)).take(room).collect();
            lacks.extend(fresh);
            next = (next + 1) % requests.len();
        }
        out.extend(lacks.iter().map(|slot| Case { sentences: told.clone(), question: slot.to_string(), answer: NONE_ANSWER.to_string() }));
    }
    out
}
because!(slot_cases, RequestSets, "the even cases of the requests of one file: for each request a case for each slot that it has, with its words, and a case with the answer none for as many slots that it lacks, taken in turn from the next requests of the file, the first request after the last");

fn intent_cases(requests: &[Request]) -> Vec<Case> {
    requests.iter().map(|request| Case { sentences: vec![request.text.clone()], question: INTENT_QUESTION.to_string(), answer: request.intent.clone() }).collect()
}
because!(intent_cases, RequestSets, "the cases of the intents of the requests of one file");

fn small_letters(text: &str) -> Vec<String> {
    let low = text.to_lowercase();
    low.split(|sign: char| !sign.is_ascii_lowercase()).filter(|piece| !piece.is_empty()).map(str::to_string).collect()
}
because!(small_letters, RequestSets, "the runs of the small letters of the plain alphabet in a text written in small letters");

fn request_new_words(all: &[Request], names: &dyn Fn(&str) -> bool) -> Vec<(String, String)> {
    let mut inside: BTreeSet<String> = BTreeSet::new();
    let mut outside: BTreeSet<String> = [NONE_ANSWER, INTENT_QUESTION].iter().map(|word| word.to_string()).collect();
    for request in all {
        let named_words: Vec<&str> = request.runs.iter().filter(|(slot, _)| names(slot)).flat_map(|(_, words)| words.iter().map(String::as_str)).collect();
        inside.extend(named_words.iter().flat_map(|word| small_letters(word)));
        let mut rest: Vec<&str> = python_words(&request.text);
        for word in &named_words {
            if let Some(place) = rest.iter().position(|held| held == word) {
                rest.remove(place);
            }
        }
        outside.extend(rest.iter().flat_map(|word| small_letters(word)));
        outside.extend(request.runs.iter().flat_map(|(slot, _)| small_letters(slot)));
        outside.extend(small_letters(&request.intent));
    }
    inside.difference(&outside).map(|word| (word.clone(), format!("{NEW_WORD_START}{word}"))).collect()
}
because!(request_new_words, RequestSets, "the table of new words of a set of requests: each run of small letters of a word of a name that stands nowhere else in the set, outside the names, in the slots or in the intents, in the order of the words");

fn request_cases(described: &Described, kind: &str, column: &str, slots_kind: &str, names: &dyn Fn(&[Request]) -> Box<dyn Fn(&str) -> bool>) -> Result<KindCases, String> {
    let train = requests(&described.file(kind, REQUEST_TRAIN)?, column)?;
    let test = requests(&described.file(kind, REQUEST_TEST)?, column)?;
    let make = |part: &[Request]| if kind == slots_kind { slot_cases(part) } else { intent_cases(part) };
    let mut cases = make(&train);
    let read_from = Some(cases.len());
    cases.extend(make(&test));
    let mut all: Vec<Request> = train;
    all.extend(test);
    let table = request_new_words(&all, &*names(&all));
    Ok(KindCases { cases, read_from, new_words: Some(table), ..KindCases::default() })
}
because!(request_cases, RequestSets, "the cases of a kind of a set of requests, the train file first, with the table of new words of the whole set");

pub fn atis_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    request_cases(described, kind, ATIS_TEXT, ATIS_SLOTS, &|all: &[Request]| {
        let mut slots: BTreeSet<String> = BTreeSet::new();
        for request in all {
            slots.extend(request.runs.iter().map(|(slot, _)| slot.clone()));
        }
        let kept: BTreeSet<String> = slots.into_iter().filter(|slot| slot.ends_with(ATIS_NAME_END) && !NOT_NAMES.contains(&slot.rsplit(SLOT_PARTS).next().unwrap_or_default())).collect();
        Box::new(move |slot: &str| kept.contains(slot))
    })
}
because!(atis_cases, RequestSets, "the cases of a kind of ATIS; its names are the slots whose type ends like a name, but the day and the month");

pub fn snips_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    request_cases(described, kind, SNIPS_TEXT, SNIPS_SLOTS, &|_: &[Request]| Box::new(|slot: &str| SNIPS_NAMES.contains(&slot)))
}
because!(snips_cases, RequestSets, "the cases of a kind of SNIPS; its names are the slots of its list of cities, places, names and titles");
