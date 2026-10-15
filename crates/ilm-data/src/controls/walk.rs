use ilm_core::base::first_sight::{FirstSightCount, all_counts};
use ilm_core::base::words::answer_words;
use crate::controls::file::ChangedCase;
use crate::controls::text::{number_runs, with_numbers};
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::BTreeSet;

pub struct WalkChanges;
source!(WalkChanges, "changed stories of steps: a told sentence with one number is a step of that amount, and the question asks whether the walker is back at the start, so the lessons have two answers. Which answer says that the walker is not back is read from the form too: a story with one step, or with two steps of different amounts, can not end at the start, whatever its turns are, and these lessons must agree on one answer. A walker that comes back no longer does when one step gets one more or when one step is taken out; doubling every amount and changing the places of two steps that stand side by side keep the answer. A learner that keeps a place and adds the amounts follows all four changes");

const WALK_ONE_MORE: &str = "one more";
because!(WALK_ONE_MORE, WalkChanges, "the sort of the change where the amount of one step gets one more");

const WALK_OUT: &str = "out";
because!(WALK_OUT, WalkChanges, "the sort of the change where one step is taken out");

const WALK_DOUBLED: &str = "doubled";
because!(WALK_DOUBLED, WalkChanges, "the sort of the change where every amount is doubled");

const WALK_SIDE_BY_SIDE: &str = "side by side";
because!(WALK_SIDE_BY_SIDE, WalkChanges, "the sort of the change where two steps that stand side by side change places");

const WALK_GAP: &str = " ";
because!(WALK_GAP, WalkChanges, "the mark between the words of an answer that is written anew");

const WALK_ANSWERS: usize = 2;
because!(WALK_ANSWERS, WalkChanges, "the answers of the lessons of a kind that this control fits: back at the start or not");

const WALK_MOST_STEPS: usize = 2;
because!(WALK_MOST_STEPS, WalkChanges, "the most steps of a story that surely does not end at the start, whatever its turns: a single step can not, and a pair of steps can not when their amounts differ");

const WALK_PAIR: usize = 2;
because!(WALK_PAIR, WalkChanges, "the steps that change places when they stand side by side");

const WALK_TWICE: u128 = 2;
because!(WALK_TWICE, WalkChanges, "the factor of the doubled amounts");

fn walk_steps(sentences: &[String]) -> Vec<(usize, u128)> {
    let mut out: Vec<(usize, u128)> = Vec::new();
    for (at, sentence) in sentences.iter().enumerate() {
        if let [(start, end)] = number_runs(sentence, false)[..]
            && let Ok(amount) = sentence[start..end].parse::<u128>()
        {
            out.push((at, amount));
        }
    }
    out
}
because!(walk_steps, WalkChanges, "the places of the told sentences with one number, and their amounts");

fn walk_amount(sentence: &str, amount: u128) -> String {
    let runs = number_runs(sentence, false);
    with_numbers(sentence, &runs[..runs.len().min(1)], &[amount.to_string()])
}
because!(walk_amount, WalkChanges, "a step sentence with a new amount in place of its number");

fn walk_away(lessons: &[Case]) -> Option<(Vec<String>, i64, i64)> {
    let mut seen: FirstSightCount<Vec<String>> = FirstSightCount::new();
    for case in lessons {
        let steps = walk_steps(&case.sentences);
        if let [(_, first), (_, second)] = steps[..]
            && first == second
        {
            continue;
        }
        if (1..=WALK_MOST_STEPS).contains(&steps.len()) {
            seen.add(answer_words(&case.answer), 1);
        }
    }
    let (word, agree) = seen.first_most()?;
    Some((word.clone(), all_counts(&seen), *agree))
}
because!(walk_away, WalkChanges, "the answer of the lessons that can not end at the start, with the count of such lessons and of those among them that have this answer");

pub fn walk_changes(lessons: &[Case], whole: bool) -> Result<Vec<ChangedCase>, String> {
    let answers: BTreeSet<Vec<String>> = lessons.iter().map(|case| answer_words(&case.answer)).collect();
    let fits = walk_away(lessons).filter(|(_, cannot, agree)| cannot == agree && answers.len() == WALK_ANSWERS);
    let Some((away, _, _)) = fits else { return Err("this control does not fit the kind; nothing written".to_string()) };
    let back = answers.iter().find(|answer| **answer != away).cloned().unwrap_or_default();
    let away_text = away.join(WALK_GAP);
    let last: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let case = &lessons[place];
        let steps = walk_steps(&case.sentences);
        let mut add = |sort: &str, sentences: Vec<String>, truth: &str| out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences, question: case.question.clone(), truth: truth.to_string() });
        if answer_words(&case.answer) == back {
            for (at, amount) in &steps {
                let mut changed = case.sentences.clone();
                changed[*at] = walk_amount(&case.sentences[*at], amount + 1);
                add(WALK_ONE_MORE, changed, &away_text);
                if steps.len() > 1 && *amount > 0 {
                    let taken: Vec<String> = case.sentences.iter().enumerate().filter(|(other, _)| other != at).map(|(_, sentence)| sentence.clone()).collect();
                    add(WALK_OUT, taken, &away_text);
                }
            }
        }
        if !steps.is_empty() {
            let mut doubled = case.sentences.clone();
            for (at, amount) in &steps {
                doubled[*at] = walk_amount(&case.sentences[*at], WALK_TWICE * amount);
            }
            add(WALK_DOUBLED, doubled, &case.answer);
        }
        for pair in steps.windows(WALK_PAIR) {
            let (first, second) = (pair[0].0, pair[1].0);
            if second == first + 1 && case.sentences[first] != case.sentences[second] {
                let mut turned = case.sentences.clone();
                turned.swap(first, second);
                add(WALK_SIDE_BY_SIDE, turned, &case.answer);
            }
        }
    }
    Ok(out)
}
because!(walk_changes, WalkChanges, "the changed stories of the lessons that the model does not learn, or of all lessons for a model of all lessons, with their true answers; nothing when the lessons do not have two answers or the stories that can not end at the start do not agree on one");
