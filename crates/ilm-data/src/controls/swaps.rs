use ilm_core::base::words::answer_words;
use crate::controls::file::ChangedCase;
use crate::controls::text::{capital_names, list_pieces};
use crate::sets::readers::common::{in_one_case, python_words, stripped};
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::HashMap;

pub struct SwapChanges;
source!(SwapChanges, "changed stories of swaps: the first told sentence names the people; the second gives what each of them has after a colon, as pieces of a list that start with a name, where the small words that every piece has after its name are no part of the thing; a later told sentence with two of the names is a swap of what they have; the question names one person. This reading gives the true answer of a story of this form, so each swap sentence of a story is taken out, or said twice in a row, and the new truth is read from the changed story. A learner that keeps the state of who has what follows every change");

const SWAP_OUT: &str = "a swap sentence taken out";
because!(SWAP_OUT, SwapChanges, "the sort of the change where one swap sentence is taken out");

const SWAP_TWICE: &str = "a swap sentence said twice";
because!(SWAP_TWICE, SwapChanges, "the sort of the change where one swap sentence is said again right after itself, which undoes it");

const SWAP_HAS: char = ':';
because!(SWAP_HAS, SwapChanges, "the mark after which the second told sentence lists what each person has");

const SWAP_PIECE_END: char = '.';
because!(SWAP_PIECE_END, SwapChanges, "the mark that may end the last piece of the list and is no part of the thing");

const SWAP_GAP: &str = " ";
because!(SWAP_GAP, SwapChanges, "the mark between the words of a thing and of a new truth");

const SWAP_LEAST: usize = 3;
because!(SWAP_LEAST, SwapChanges, "the told sentences of the shortest story: the people, what they have, and one swap");

const SWAP_FIRST: usize = 2;
because!(SWAP_FIRST, SwapChanges, "the place of the first told sentence that can be a swap, after the people and what they have");

const SWAP_PEOPLE: usize = 2;
because!(SWAP_PEOPLE, SwapChanges, "the people of a swap, and the fewest people with a thing in a story that can be read");

struct SwapStory {
    held: HashMap<String, String>,
    swaps: Vec<(usize, String, String)>,
    asked: String,
}
because!(SwapStory, SwapChanges, "a story as its reading gives it: the thing of each person at the start, the swaps in their order with the place of each swap sentence, and the person of the question");

fn swap_story(sentences: &[String], question: &str) -> Option<SwapStory> {
    if sentences.len() < SWAP_LEAST {
        return None;
    }
    let (_, listed) = sentences[1].split_once(SWAP_HAS)?;
    let names = capital_names(&sentences[0]);
    let pieces: Vec<Vec<&str>> = list_pieces(listed).into_iter().map(stripped).filter(|piece| !piece.is_empty()).map(|piece| python_words(piece.trim_end_matches(SWAP_PIECE_END))).collect();
    let rests: Vec<(&str, &[&str])> = pieces.iter().filter_map(|piece| piece.split_first()).filter(|(first, _)| names.contains(first)).map(|(first, rest)| (*first, rest)).collect();
    if rests.len() < SWAP_PEOPLE {
        return None;
    }
    let mut skip = 0;
    while rests.iter().all(|(_, rest)| rest.len() > skip + 1) && rests.iter().all(|(_, rest)| rest[skip] == rests[0].1[skip]) && in_one_case(rests[0].1[skip], false) {
        skip += 1;
    }
    let held: HashMap<String, String> = rests.iter().map(|(name, rest)| (name.to_string(), rest[skip..].join(SWAP_GAP))).collect();
    let mut swaps: Vec<(usize, String, String)> = Vec::new();
    for (at, sentence) in sentences.iter().enumerate().skip(SWAP_FIRST) {
        let there: Vec<&str> = capital_names(sentence).into_iter().filter(|name| held.contains_key(*name)).collect();
        if let [one, two] = there.as_slice()
            && one != two
        {
            swaps.push((at, one.to_string(), two.to_string()));
        }
    }
    let asked = capital_names(question).into_iter().filter(|name| held.contains_key(*name)).last()?.to_string();
    Some(SwapStory { held, swaps, asked })
}
because!(swap_story, SwapChanges, "the reading of a story, or nothing when the story does not have this form");

fn swap_truth(story: &SwapStory) -> Vec<String> {
    let mut held = story.held.clone();
    for (_, one, two) in &story.swaps {
        let (first, second) = (held[one].clone(), held[two].clone());
        held.insert(one.clone(), second);
        held.insert(two.clone(), first);
    }
    answer_words(&held[&story.asked])
}
because!(swap_truth, SwapChanges, "the words of the thing of the asked person after every swap of the story");

pub fn swap_changes(lessons: &[Case], whole: bool) -> Vec<ChangedCase> {
    let last: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let case = &lessons[place];
        let Some(story) = swap_story(&case.sentences, &case.question) else { continue };
        if swap_truth(&story) != answer_words(&case.answer) {
            continue;
        }
        for (at, _, _) in &story.swaps {
            let taken: Vec<String> = case.sentences.iter().enumerate().filter(|(other, _)| other != at).map(|(_, sentence)| sentence.clone()).collect();
            let mut twice = case.sentences.clone();
            twice.insert(*at, case.sentences[*at].clone());
            for (sort, changed) in [(SWAP_OUT, taken), (SWAP_TWICE, twice)] {
                let Some(truth) = swap_story(&changed, &case.question).map(|changed_story| swap_truth(&changed_story)) else { continue };
                out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences: changed, question: case.question.clone(), truth: truth.join(SWAP_GAP) });
            }
        }
    }
    out
}
because!(swap_changes, SwapChanges, "the changed stories of the lessons that the model does not learn, or of all lessons for a model of all lessons: for each lesson whose reading gives its answer, each swap sentence taken out and said twice, with the new truth of the reading");
