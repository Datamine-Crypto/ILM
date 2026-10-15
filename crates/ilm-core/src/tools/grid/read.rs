use crate::base::first_sight::FirstSightTable;
use crate::base::words::{answer_words, words};
use crate::cases::Case;
use crate::tools::arrange::read::{choice_mark, segments};
use patterns::{because, source};

pub struct ListedThings;
source!(ListedThings, "a story of things in places, given as lists: the question lists the places as its choices, in their order, and asks for the place of one thing; a told sentence that lists as many pieces as there are places, each piece starting with one word, names one kind of things, and a thing is named by each run of words that ends its piece after that word, when no other piece of the story ends with the same run, as a definite description names the one thing that fits it (Russell, On Denoting); the word that joins the last piece to a list is found by counting over all lists of the lessons; every other told sentence is read as a form with an open place for each thing that it names, without the number that may stand before it as its number in a list");

const LIST_MARK: char = ':';
because!(LIST_MARK, ListedThings, "the mark after which a told sentence gives its list of pieces");

const GRID_SLOT: &str = "_";
because!(GRID_SLOT, ListedThings, "the mark of an open place in the text of a form; the word cutter never gives it as a word");

const GRID_BETWEEN: &str = " ";
because!(GRID_BETWEEN, ListedThings, "the mark between the words of the text of a form");

const LEAST_PLACES: usize = 2;
because!(LEAST_PLACES, ListedThings, "a story of fewer places holds no order of things to find");

pub(super) const GRID_MOST_PLACES: usize = 6;
because!(GRID_MOST_PLACES, ListedThings, "the most places of a story that the tool reads: the arrangements grow as a power of the factorial of the places");

pub(super) const MOST_KINDS: usize = 6;
because!(MOST_KINDS, ListedThings, "the most kinds of things of a story that the tool reads, for the same reason as the most places");

pub(super) const STATEMENT_THINGS: usize = 2;
because!(STATEMENT_THINGS, ListedThings, "the most things that a told statement of the tool names: one thing (its place) or a pair of things (how their places stand)");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct GridStatement {
    pub(super) form: Option<u32>,
    pub(super) things: Vec<usize>,
}
because!(GridStatement, ListedThings, "a told statement as the number of its form (none for a form that the lessons never had) and the things that fill its open places, as numbers of things");

#[derive(Clone, Debug, Default)]
pub(super) struct GridPuzzle {
    pub(super) places: usize,
    pub(super) kinds: usize,
    pub(super) told: Vec<GridStatement>,
    pub(super) asked: usize,
    pub(super) choices: Vec<Vec<String>>,
}
because!(GridPuzzle, ListedThings, "a case read for the tool: the number of places, the number of kinds of things (each kind has one thing for each place, and the thing of number t is of kind t over the places), the told statements, the thing that the question asks about, and the choices in the order of the places: choices that are each one number name the places by the order of their numbers, and other choices by their own order");

fn list_pieces(sentence: &str, places: usize) -> Option<Vec<Vec<String>>> {
    let (_, listed) = sentence.rsplit_once(LIST_MARK)?;
    let said = words(listed);
    let head = said.first()?;
    let starts: Vec<usize> = (0..said.len()).filter(|at| said[*at] == *head).collect();
    if starts.len() != places {
        return None;
    }
    Some(starts.iter().enumerate().map(|(number, from)| said[*from..starts.get(number + 1).copied().unwrap_or(said.len())].to_vec()).collect())
}
because!(list_pieces, ListedThings, "the pieces of a told list: after its mark, a piece starts at each place of the word that starts the list, when that word starts as many pieces as there are places");

pub(super) fn joiner(cases: &[Case], mark: &str) -> Option<String> {
    let mut ends: FirstSightTable<String, u32> = FirstSightTable::new();
    for case in cases {
        let places = segments(&words(&case.question), mark).len();
        for sentence in &case.sentences {
            let Some(pieces) = list_pieces(sentence, places) else { continue };
            if let Some(word) = pieces.len().checked_sub(LEAST_PLACES).and_then(|at| pieces[at].last()) {
                ends.add(word.clone(), 1);
            }
        }
    }
    ends.first_most().map(|(word, _)| word.clone())
}
because!(joiner, ListedThings, "the word that joins the last piece to a list: the word that ends the piece before the last one in the most lists of the lessons");

fn grid_statement(said: &[String], names: &[(Vec<String>, usize)], form_of: &mut dyn FnMut(&str) -> Option<u32>) -> GridStatement {
    let mut form: Vec<&str> = Vec::with_capacity(said.len());
    let mut things: Vec<usize> = Vec::new();
    let mut at = 0usize;
    while at < said.len() {
        match names.iter().find(|(name, _)| said[at..].starts_with(name)) {
            Some((name, thing)) => {
                form.push(GRID_SLOT);
                things.push(*thing);
                at += name.len();
            }
            None => {
                form.push(&said[at]);
                at += 1;
            }
        }
    }
    GridStatement { form: form_of(&form.join(GRID_BETWEEN)), things }
}
because!(grid_statement, ListedThings, "a text read as a statement: each thing that it names becomes an open place, the longest name first at each word");

pub(super) fn grid_puzzle(case: &Case, mark: &str, joined: Option<&str>, form_of: &mut dyn FnMut(&str) -> Option<u32>) -> Option<GridPuzzle> {
    let asked_words = words(&case.question);
    let mut choices = segments(&asked_words, mark);
    let numbers: Option<Vec<u64>> = choices.iter().map(|choice| match choice.as_slice() {
        [one] => one.parse::<u64>().ok(),
        _ => None,
    }).collect();
    if let Some(numbers) = numbers {
        let mut order: Vec<usize> = (0..choices.len()).collect();
        order.sort_by_key(|place| numbers[*place]);
        choices = order.iter().map(|place| choices[*place].clone()).collect();
    }
    let places = choices.len();
    if !(LEAST_PLACES..=GRID_MOST_PLACES).contains(&places) {
        return None;
    }
    let mut names: Vec<(Vec<String>, usize)> = Vec::new();
    let mut kinds = 0usize;
    let mut lists: Vec<usize> = Vec::new();
    for (number, sentence) in case.sentences.iter().enumerate() {
        let Some(mut pieces) = list_pieces(sentence, places) else { continue };
        if let (Some(word), Some(piece)) = (joined, pieces.len().checked_sub(LEAST_PLACES).and_then(|at| pieces.get_mut(at))) {
            if piece.last().is_some_and(|last| last == word) {
                piece.pop();
            }
        }
        if pieces.iter().any(|piece| piece.len() <= 1) {
            return None;
        }
        for (item, piece) in pieces.iter().enumerate() {
            let thing = kinds * places + item;
            names.extend((1..piece.len()).map(|from| (piece[from..].to_vec(), thing)));
        }
        kinds += 1;
        lists.push(number);
    }
    if kinds == 0 || kinds > MOST_KINDS {
        return None;
    }
    let mut uses: std::collections::HashMap<Vec<String>, usize> = std::collections::HashMap::new();
    names.iter().for_each(|(name, _)| *uses.entry(name.clone()).or_default() += 1);
    names.retain(|(name, _)| uses.get(name) == Some(&1));
    names.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
    let told: Vec<GridStatement> = case
        .sentences
        .iter()
        .enumerate()
        .filter(|(number, _)| !lists.contains(number))
        .map(|(_, sentence)| {
            let said = words(sentence);
            let start = usize::from(said.first().is_some_and(|word| word.bytes().all(|sign| sign.is_ascii_digit())));
            grid_statement(&said[start..], &names, form_of)
        })
        .filter(|told| (1..=STATEMENT_THINGS).contains(&told.things.len()))
        .collect();
    let start = asked_words.iter().position(|word| word == mark).unwrap_or(asked_words.len());
    let question = grid_statement(&asked_words[..start], &names, &mut |_| None);
    let [asked] = question.things.as_slice() else { return None };
    Some(GridPuzzle { places, kinds, told, asked: *asked, choices })
}
because!(grid_puzzle, ListedThings, "a case read for the tool, and nothing when its question offers no few places, its story lists no kind of things or too many, a list does not name a thing in each piece, or the question does not name one thing");

pub(super) fn answer_place(case: &Case, puzzle: &GridPuzzle) -> Option<usize> {
    let truth = answer_words(&case.answer);
    puzzle.choices.iter().position(|choice| *choice == truth)
}
because!(answer_place, ListedThings, "the place that the answer of a lesson names: the place of the choice with the words of the answer");

pub struct OptionalWords;
source!(OptionalWords, "the substitution test of Harris on the forms of statements: a run of words before an open place is optional when a form with the run and a form without it are both forms of the lessons, as the person who _ and _ stand in the same places; such runs are taken out of every form, so that the forms of one relation are one form");

const FILLER_MOST: usize = 3;
because!(FILLER_MOST, OptionalWords, "the longest run of words before an open place that is tried as optional: a reference to a thing has few words before it");

const FILLER_LEAST: u32 = 2;
because!(FILLER_LEAST, OptionalWords, "a run is optional from this many pairs of forms with and without it on: one pair can be chance");

pub(super) fn fillers(texts: &[String]) -> Vec<Vec<String>> {
    let known: std::collections::HashSet<&str> = texts.iter().map(String::as_str).collect();
    let mut counts: FirstSightTable<Vec<String>, u32> = FirstSightTable::new();
    for text in texts {
        let said: Vec<&str> = text.split(GRID_BETWEEN).collect();
        for (at, word) in said.iter().enumerate() {
            if *word != GRID_SLOT {
                continue;
            }
            for size in 1..=FILLER_MOST.min(at) {
                let run = &said[at - size..at];
                if run.contains(&GRID_SLOT) {
                    break;
                }
                let shorter: Vec<&str> = said[..at - size].iter().chain(&said[at..]).copied().collect();
                if known.contains(shorter.join(GRID_BETWEEN).as_str()) {
                    counts.add(run.iter().map(|word| word.to_string()).collect(), 1);
                }
            }
        }
    }
    let mut kept: Vec<Vec<String>> = counts.rows().iter().filter(|(_, count)| *count >= FILLER_LEAST).map(|(run, _)| run.clone()).collect();
    kept.sort_by_key(|run| std::cmp::Reverse(run.len()));
    kept
}
because!(fillers, OptionalWords, "the optional runs of the forms of the lessons, the longest first");

pub(super) fn without_fillers(text: &str, fillers: &[Vec<String>]) -> String {
    let mut said: Vec<&str> = text.split(GRID_BETWEEN).collect();
    let mut at = 0usize;
    while at < said.len() {
        if said[at] == GRID_SLOT
            && let Some(run) = fillers.iter().find(|run| run.len() <= at && said[at - run.len()..at].iter().zip(run.iter()).all(|(word, held)| *word == held.as_str()))
        {
            let size = run.len();
            said.drain(at - size..at);
            at -= size;
        }
        at += 1;
    }
    said.join(GRID_BETWEEN)
}
because!(without_fillers, OptionalWords, "a form with every optional run before an open place taken out");

pub(super) fn find_mark(cases: &[Case]) -> Option<String> {
    choice_mark(cases)
}
because!(find_mark, ListedThings, "the word that starts the choices of the questions, as the arrange tool counts it");
