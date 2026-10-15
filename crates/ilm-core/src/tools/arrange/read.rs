use crate::base::first_sight::FirstSightTable;
use crate::base::words::{answer_words, words};
use crate::cases::Case;
use crate::tools::pattern::model::SURE_SHARE;
use patterns::{because, source};

pub struct ChoiceStatements;
source!(ChoiceStatements, "a question that lists its choices, each a statement about one thing of the story: the word that starts each choice is found by counting, as the learner of choices of the pattern learner finds it; the things are the parts in which the choices differ; every told sentence and every choice is read as a form, its words with each thing that it names as an open place, and the things that fill the places");

pub(super) const ARRANGE_SLOT: &str = "_";
because!(ARRANGE_SLOT, ChoiceStatements, "the mark of an open place in the text of a form; the word cutter never gives it as a word, so a form is never taken for a text");

const ARRANGE_BETWEEN: &str = " ";
because!(ARRANGE_BETWEEN, ChoiceStatements, "the mark between the words of the text of a form");

const LEAST_CHOICES: usize = 2;
because!(LEAST_CHOICES, ChoiceStatements, "a question with fewer choices offers no choice, and a word that starts the choices stands this many times or more in the question");

pub(super) const MOST_PLACES: usize = 2;
because!(MOST_PLACES, ChoiceStatements, "the most things that a statement of the tool names: a place is said of a single thing and an order of a pair of things; a sentence that names more of them lists the things");

pub(super) const ARRANGE_MOST_THINGS: usize = 8;
because!(ARRANGE_MOST_THINGS, ChoiceStatements, "the most things of a case that the tool arranges: the arrangements grow as the factorial of the things, and with more things a search of them takes too long");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ChoiceStatement {
    pub(super) form: Option<u32>,
    pub(super) slots: Vec<usize>,
}
because!(ChoiceStatement, ChoiceStatements, "a told sentence or a choice as the number of its form (none for a form that the lessons never had) and the things that fill the open places of the form, in their order");

#[derive(Clone, Debug, Default)]
pub(super) struct Puzzle {
    pub(super) things: usize,
    pub(super) told: Vec<ChoiceStatement>,
    pub(super) choices: Vec<(Option<ChoiceStatement>, Vec<String>)>,
}
because!(Puzzle, ChoiceStatements, "a case read for the tool: the number of its things, its told statements, and each choice as a statement (none when it names no thing or more than the tool reads) with its words");

pub(crate) fn segments(asked: &[String], mark: &str) -> Vec<Vec<String>> {
    let starts: Vec<usize> = (0..asked.len()).filter(|place| asked[*place] == mark).collect();
    let mut found: Vec<Vec<String>> = Vec::new();
    for (number, from) in starts.iter().enumerate() {
        let to = starts.get(number + 1).copied().unwrap_or(asked.len());
        found.push(asked[from + 1..to].to_vec());
    }
    found
}
because!(segments, ChoiceStatements, "the choices of a question: the words after each place of the word that starts a choice, up to the next such place or the end");

pub(crate) fn choice_mark(cases: &[Case]) -> Option<String> {
    let need = SURE_SHARE * cases.len() as f64;
    let mut hits: FirstSightTable<String, u32> = FirstSightTable::new();
    for case in cases {
        let asked = words(&case.question);
        let answer = answer_words(&case.answer);
        let mut counted: FirstSightTable<&String, u32> = FirstSightTable::new();
        for word in &asked {
            counted.add(word, 1);
        }
        for (word, count) in counted.rows() {
            if *count as usize >= LEAST_CHOICES && segments(&asked, word).contains(&answer) {
                hits.add((*word).clone(), 1);
            }
        }
    }
    let (mark, count) = hits.first_most()?;
    (f64::from(*count) >= need).then(|| mark.clone())
}
because!(choice_mark, ChoiceStatements, "the word that starts the choices of the questions: of the words that stand two times or more in a question, the one with which the answer is one of the choices in the most lessons, the first of equals; only when that holds for nearly all lessons");

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
because!(things_of, ChoiceStatements, "the things of a case: the part of each choice between the words that every choice starts with and the words that every choice ends with; nothing when a choice has no such part or two choices have the same one");

fn statement(said: &[String], things: &[Vec<String>], form_of: &mut dyn FnMut(&str) -> Option<u32>) -> Option<ChoiceStatement> {
    let mut longest: Vec<usize> = (0..things.len()).collect();
    longest.sort_by_key(|thing| std::cmp::Reverse(things[*thing].len()));
    let mut form: Vec<&str> = Vec::with_capacity(said.len());
    let mut slots: Vec<usize> = Vec::new();
    let mut at = 0usize;
    while at < said.len() {
        match longest.iter().find(|thing| said[at..].starts_with(&things[**thing])) {
            Some(thing) => {
                form.push(ARRANGE_SLOT);
                slots.push(*thing);
                at += things[*thing].len();
            }
            None => {
                form.push(&said[at]);
                at += 1;
            }
        }
    }
    let distinct = slots.iter().enumerate().all(|(place, thing)| !slots[..place].contains(thing));
    if slots.is_empty() || slots.len() > MOST_PLACES || !distinct {
        return None;
    }
    Some(ChoiceStatement { form: form_of(&form.join(ARRANGE_BETWEEN)), slots })
}
because!(statement, ChoiceStatements, "a text read as a statement: each thing that it names becomes an open place, the longest thing first at each word; nothing when the text names no thing, names a thing twice, or names more things than a statement of the tool");

pub(super) fn puzzle(sentences: &[String], question: &str, mark: &str, form_of: &mut dyn FnMut(&str) -> Option<u32>) -> Option<Puzzle> {
    let choices = segments(&words(question), mark);
    if choices.len() < LEAST_CHOICES {
        return None;
    }
    let things = things_of(&choices)?;
    if things.len() > ARRANGE_MOST_THINGS {
        return None;
    }
    let told: Vec<ChoiceStatement> = sentences.iter().filter_map(|sentence| statement(&words(sentence), &things, form_of)).collect();
    let choices = choices.into_iter().map(|choice| (statement(&choice, &things, form_of), choice)).collect();
    Some(Puzzle { things: things.len(), told, choices })
}
because!(puzzle, ChoiceStatements, "a case read for the tool: its choices, its things, its told statements and each choice as a statement; nothing when the question offers no choice among few enough things");
