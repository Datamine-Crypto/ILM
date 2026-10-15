use crate::base::first_sight::FirstSightTable;
use crate::base::words::sign_runs;
use crate::cases::Case;
use patterns::{because, source};
use std::collections::{HashSet, VecDeque};

pub struct NamedPoints;
source!(NamedPoints, "a story of things at points: the question names two things, and the words that most questions share are the form of the question, so the other words of a question are the names of things, kept with their capital letters as names are written; every told sentence that names exactly two things is a link between them, read as a form with an open place for each of the two, the first thing named in the sentence in the first place; the two asked things are joined by the shortest chain of links (breadth first search, Moore)");

pub(super) const FIRST_PLACE: &str = "_";
because!(FIRST_PLACE, NamedPoints, "the mark of the open place of the first thing of a link in the text of its form; the cut of names never gives it as a word");

pub(super) const SECOND_PLACE: &str = "=";
because!(SECOND_PLACE, NamedPoints, "the mark of the open place of the second thing of a link in the text of its form, so that a form with four places tells which thing stands in each");

pub(super) const SPACE_BETWEEN: &str = " ";
because!(SPACE_BETWEEN, NamedPoints, "the mark between the words of the text of a form");

pub(super) const LINK_THINGS: usize = 2;
because!(LINK_THINGS, NamedPoints, "the things of a link and of a question: a link joins a pair of things, and the question asks how one thing stands to another");

pub(super) fn name_runs(text: &str) -> Vec<&str> {
    sign_runs(text, char::is_alphanumeric)
}
because!(name_runs, NamedPoints, "the words of a text with their capital letters: the runs of letters and digits");

pub(super) fn question_form(cases: &[Case]) -> HashSet<String> {
    let mut counts: FirstSightTable<String, usize> = FirstSightTable::new();
    for case in cases {
        let mut seen: Vec<String> = Vec::new();
        for run in name_runs(&case.question) {
            let low = run.to_lowercase();
            if !seen.contains(&low) {
                seen.push(low);
            }
        }
        seen.into_iter().for_each(|word| counts.add(word, 1));
    }
    counts.rows().iter().filter(|(_, count)| count * LINK_THINGS > cases.len()).map(|(word, _)| word.clone()).collect()
}
because!(question_form, NamedPoints, "the words of the form of the question: the words, in small letters, that more than half of the questions hold");

pub(super) fn asked_pair(question: &str, form: &HashSet<String>) -> Option<[String; LINK_THINGS]> {
    let named: Vec<&str> = name_runs(question).into_iter().filter(|run| !form.contains(&run.to_lowercase())).collect();
    match named.as_slice() {
        [one, other] if one != other => Some([one.to_string(), other.to_string()]),
        _ => None,
    }
}
because!(asked_pair, NamedPoints, "the two things that a question names: its words outside the form of the question, when there are exactly two different ones");

#[derive(Clone, Debug)]
pub(super) struct SpaceLink {
    pub(super) form: String,
    pub(super) first: usize,
    pub(super) second: usize,
}
because!(SpaceLink, NamedPoints, "a told link: the text of its form and the numbers of its first and its second thing in the story");

#[derive(Clone, Debug)]
pub(super) struct SpaceStory {
    pub(super) links: Vec<SpaceLink>,
    pub(super) asked: [usize; LINK_THINGS],
}
because!(SpaceStory, NamedPoints, "a story read for the tool: its links and the numbers of the two asked things");

fn thing_number(things: &mut Vec<String>, name: &str) -> usize {
    match things.iter().position(|held| held == name) {
        Some(at) => at,
        None => {
            things.push(name.to_string());
            things.len() - 1
        }
    }
}
because!(thing_number, NamedPoints, "the number of a thing in a story, a new number for a new name");

pub(super) fn space_story(sentences: &[String], asked: &[String; LINK_THINGS], names: &HashSet<String>) -> SpaceStory {
    let mut things: Vec<String> = Vec::new();
    let pair = [thing_number(&mut things, &asked[0]), thing_number(&mut things, &asked[1])];
    let is_name = |run: &str| names.contains(run) || asked.iter().any(|name| name == run);
    let mut links: Vec<SpaceLink> = Vec::new();
    for sentence in sentences {
        let runs = name_runs(sentence);
        let mut named: Vec<&str> = Vec::new();
        for run in runs.iter().filter(|run| is_name(run)) {
            if !named.contains(run) {
                named.push(run);
            }
        }
        let [first, second] = named.as_slice() else { continue };
        let form: Vec<String> = runs
            .iter()
            .map(|run| if run == first { FIRST_PLACE.to_string() } else if run == second { SECOND_PLACE.to_string() } else { run.to_lowercase() })
            .collect();
        links.push(SpaceLink { form: form.join(SPACE_BETWEEN), first: thing_number(&mut things, first), second: thing_number(&mut things, second) });
    }
    SpaceStory { links, asked: pair }
}
because!(space_story, NamedPoints, "the links of a story: each told sentence that names exactly two things, by the names of the lessons and the names of its question");

pub(super) type ChainStep = (usize, bool);
because!(ChainStep, NamedPoints, "one step of a chain: the number of a link, and whether the chain goes from its first thing to its second");

pub(super) fn chain(story: &SpaceStory) -> Option<Vec<ChainStep>> {
    let [start, end] = story.asked;
    let size = story.links.iter().map(|link| link.first.max(link.second)).chain(story.asked).max().unwrap_or(0) + 1;
    let mut reached: Vec<Option<(usize, ChainStep)>> = vec![None; size];
    let mut seen: HashSet<usize> = HashSet::from([start]);
    let mut queue: VecDeque<usize> = VecDeque::from([start]);
    while let Some(thing) = queue.pop_front() {
        if thing == end {
            break;
        }
        for (number, link) in story.links.iter().enumerate() {
            let next = if link.first == thing { (link.second, true) } else if link.second == thing { (link.first, false) } else { continue };
            if seen.insert(next.0) {
                reached[next.0] = Some((thing, (number, next.1)));
                queue.push_back(next.0);
            }
        }
    }
    if !seen.contains(&end) {
        return None;
    }
    let mut steps: Vec<ChainStep> = Vec::new();
    let mut at = end;
    while at != start {
        let (from, step) = reached.get(at).copied().flatten()?;
        steps.push(step);
        at = from;
    }
    steps.reverse();
    Some(steps)
}
because!(chain, NamedPoints, "the shortest chain of links from the first asked thing to the second, the links of each thing tried in the order of the story; nothing when no chain joins them");
