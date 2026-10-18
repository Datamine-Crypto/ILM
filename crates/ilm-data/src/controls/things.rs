use crate::controls::file::ChangedCase;
use crate::controls::text::byte_runs;
use ilm_core::base::words::{answer_words, words};
use ilm_core::cases::Case;
use ilm_core::choice::inner_parts;
use patterns::{because, source};

pub struct ThingChanges;
source!(ThingChanges, "changed stories of a choice among things: the question lists its choices, which differ only in the thing that they name; two things of a story change names in every told sentence, and the question stays. The told facts of each thing are then the facts of the other, so the true choice names the other thing when it named one of the two, and stays when it named neither. A learner that reads the told facts of the things follows every change");

const THING_TURNS: &str = "two things change names, one of them the answer";
because!(THING_TURNS, ThingChanges, "the sort of the change where the thing of the true choice changes names with another thing, so that the truth turns to the choice of the other thing");

const THING_STAYS: &str = "two things change names, neither of them the answer";
because!(THING_STAYS, ThingChanges, "the sort of the change where two things that the true choice does not name change names, so that the truth stays");

const THING_LEAST_CHOICES: usize = 2;
because!(THING_LEAST_CHOICES, ThingChanges, "a question with fewer choices offers no choice, and the word that starts the choices stands this many times or more in a question");

const THING_SHARE: f64 = 0.98;
because!(THING_SHARE, ThingChanges, "the share of the lessons whose answer must be one of the choices for a word to start the choices: nearly all, as a set may hold a few lessons written wrong");

const THING_GAP: &str = " ";
because!(THING_GAP, ThingChanges, "the mark between the words of a new truth");

pub(crate) fn choice_segments(asked: &[String], mark: &str) -> Vec<Vec<String>> {
    let starts: Vec<usize> = (0..asked.len()).filter(|place| asked[*place] == mark).collect();
    starts.iter().enumerate().map(|(number, from)| asked[from + 1..starts.get(number + 1).copied().unwrap_or(asked.len())].to_vec()).collect()
}
because!(choice_segments, ThingChanges, "the choices of a question: the words after each place of the word that starts a choice, up to the next such place or the end");

pub(crate) fn choices_mark(lessons: &[Case]) -> Option<String> {
    let mut hits: Vec<(String, usize)> = Vec::new();
    for case in lessons {
        let asked = words(&case.question);
        let answer = answer_words(&case.answer);
        let mut done: Vec<&String> = Vec::new();
        for word in &asked {
            if done.contains(&word) || asked.iter().filter(|other| *other == word).count() < THING_LEAST_CHOICES {
                continue;
            }
            done.push(word);
            if choice_segments(&asked, word).contains(&answer) {
                match hits.iter_mut().find(|(held, _)| held == word) {
                    Some((_, count)) => *count += 1,
                    None => hits.push((word.clone(), 1)),
                }
            }
        }
    }
    let mut best: Option<&(String, usize)> = None;
    for row in &hits {
        if best.is_none_or(|top| row.1 > top.1) {
            best = Some(row);
        }
    }
    best.filter(|(_, count)| (*count as f64) >= THING_SHARE * lessons.len() as f64).map(|(mark, _)| mark.clone())
}
because!(choices_mark, ThingChanges, "the word that starts the choices: of the words that stand two times or more in a question, the one with which the answer is a choice in the most lessons, the first of equals; only when that holds for nearly all lessons");

fn exchanged(text: &str, one: &[String], other: &[String]) -> String {
    let runs = byte_runs(text, u8::is_ascii_alphanumeric);
    let low: Vec<String> = runs.iter().map(|(start, end)| text[*start..*end].to_ascii_lowercase()).collect();
    let mut out = String::with_capacity(text.len());
    let mut from = 0usize;
    let mut at = 0usize;
    while at < runs.len() {
        let swap = if low[at..].starts_with(one) { Some((one.len(), other)) } else if low[at..].starts_with(other) { Some((other.len(), one)) } else { None };
        let Some((size, into)) = swap.filter(|(size, _)| *size > 0) else {
            at += 1;
            continue;
        };
        let (start, end) = (runs[at].0, runs[at + size - 1].1);
        out.push_str(&text[from..start]);
        let capital = text.as_bytes()[start].is_ascii_uppercase();
        let mut put = into.join(THING_GAP);
        if capital && let Some(first) = put.get(..1) {
            put = format!("{}{}", first.to_ascii_uppercase(), &put[1..]);
        }
        out.push_str(&put);
        from = end;
        at += size;
    }
    out.push_str(&text[from..]);
    out
}
because!(exchanged, ThingChanges, "a text with the two things changed for each other wherever their words stand, a capital letter kept at the start of a changed name");

pub fn thing_changes(lessons: &[Case], whole: bool) -> Vec<ChangedCase> {
    let Some(mark) = choices_mark(lessons) else { return Vec::new() };
    let last: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let case = &lessons[place];
        let choices = choice_segments(&words(&case.question), &mark);
        let Some(things) = things_of(&choices) else { continue };
        let Some(answer) = choices.iter().position(|choice| *choice == answer_words(&case.answer)) else { continue };
        for other in (0..things.len()).filter(|other| *other != answer) {
            let sentences: Vec<String> = case.sentences.iter().map(|sentence| exchanged(sentence, &things[answer], &things[other])).collect();
            out.push(ChangedCase { place: Some(place), sort: THING_TURNS.to_string(), sentences, question: case.question.clone(), truth: choices[other].join(THING_GAP) });
        }
        let rest: Vec<usize> = (0..things.len()).filter(|other| *other != answer).collect();
        if let [one, two, ..] = rest.as_slice() {
            let sentences: Vec<String> = case.sentences.iter().map(|sentence| exchanged(sentence, &things[*one], &things[*two])).collect();
            out.push(ChangedCase { place: Some(place), sort: THING_STAYS.to_string(), sentences, question: case.question.clone(), truth: choices[answer].join(THING_GAP) });
        }
    }
    out
}
because!(thing_changes, ThingChanges, "the changed stories of the lessons that the model does not learn, or of all lessons for a model of all lessons: the thing of the answer changed for each other thing, with the choice of that thing as the new truth, and two other things changed for each other, with the old truth");

fn things_of(choices: &[Vec<String>]) -> Option<Vec<Vec<String>>> {
    let first = choices.first()?;
    let shortest = choices.iter().map(Vec::len).min()?;
    let mut head = 0usize;
    while head < shortest && choices.iter().all(|choice| choice[head] == first[head]) {
        head += 1;
    }
    let mut tail = 0usize;
    while head + tail < shortest && choices.iter().all(|choice| choice[choice.len() - 1 - tail] == first[first.len() - 1 - tail]) {
        tail += 1;
    }
    let things: Vec<Vec<String>> = choices.iter().map(|choice| choice[head..choice.len() - tail].to_vec()).collect();
    let distinct = things.iter().enumerate().all(|(at, thing)| !thing.is_empty() && !things[..at].contains(thing));
    distinct.then_some(things)
}
because!(things_of, ThingChanges, "the things of a case: the part of each choice between the words that every choice starts with and the words that every choice ends with; nothing when a choice has no such part or two choices have the same one");
