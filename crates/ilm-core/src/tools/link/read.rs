use crate::base::first_sight::{FirstSightCount, FirstSightTable, NumberBag};
use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::kinds::{JOINT, PatternWords};
use crate::tools::pattern::names::WordTexts;
use crate::tools::pattern::story::Query;
use patterns::{because, source};
use std::collections::{HashMap, VecDeque};

pub struct LinkWays;
source!(LinkWays, "links between the things of a story: a sentence with two things or more links each pair of them, labelled by its sentence with the thing nearer to the start as S, the other as T and any third thing as N; the way from one thing of the question to the other is the shortest chain of links, found by a breadth first search over the links in the order of the sentences");

pub struct LinkNewKinds;
source!(LinkNewKinds, "a word of a story that the lessons never held, and that has no kind, takes the kind of its place in the most frequent lesson sentence with the same single words and things in all its other places");

pub(super) const LINK_MARK: u32 = 0x8000_0000;
because!(LINK_MARK, LinkWays, "the bit that marks a thing kind in the form of a sentence, where every other place holds its word: word numbers stay below it");

pub(super) const LINK_START: &str = "S";
because!(LINK_START, LinkWays, "the mark in a label of the thing of a link that is nearer to the start of the way");

pub(super) const LINK_GOAL: &str = "T";
because!(LINK_GOAL, LinkWays, "the mark in a label of the other thing of a link");

pub(super) const LINK_OTHER: &str = "N";
because!(LINK_OTHER, LinkWays, "the mark in a label of a third thing of the sentence of a link");

const LINK_BETWEEN: &str = " ";
because!(LINK_BETWEEN, LinkWays, "the mark between the parts of a label");

pub(super) const LINK_ENDS: usize = 2;
because!(LINK_ENDS, LinkWays, "the things of a question that a way joins: a way runs from one of them to the other, and a question with fewer names no way");

pub(super) const LINK_LABEL: &str = "L:";
because!(LINK_LABEL, LinkWays, "the start of the text of a label of a link, so that a label is never taken for an answer");

pub(super) type LinkNear = FirstSightTable<u32, Vec<(u32, u32)>>;
because!(LinkNear, LinkWays, "the links of a story: for each thing, in the order of first sight, the things that a sentence links it with and the label of each link, in the order of the sentences");

#[derive(Clone, Debug, Default)]
pub(super) struct LinkRead {
    pub(super) ends: Vec<u32>,
    pub(super) near: LinkNear,
    pub(super) told: NumberBag<u32>,
}
because!(LinkRead, LinkWays, "a case as the links read it: the things of the question in its order, the links of the story, and the single words of the story");

#[derive(Clone, Debug, Default)]
pub(super) struct LinkWords {
    pub(super) thing_kinds: Vec<u32>,
    pub(super) known: NumberBag<u32>,
    pub(super) forms: HashMap<usize, FirstSightTable<Vec<u32>, u32>>,
}
because!(LinkWords, LinkWays, LinkNewKinds, "what the links count of words: the kinds of the words that a question of the lessons shares with its story, which are thing kinds; the single words of the lesson stories; and the forms of the lesson sentences by their length, with a thing kind in the place of each thing, each with its count. apart gives the single words of a run of words; kind_of gives the kind of a word in a case; mark gives a word as a form writes it; new_kinds gives the kinds of the new words of a case; read gives the things and links of a case");

impl LinkWords {
    pub(super) fn apart<N: WordTexts>(names: &mut N, words: &[u32]) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::with_capacity(words.len());
        for word in words {
            let text = names.name(*word).to_string();
            if !text.contains(JOINT) {
                out.push(*word);
                continue;
            }
            for part in text.split(JOINT) {
                out.push(names.number(part));
            }
        }
        out
    }

    pub(super) fn kind_of(words: &PatternWords, case: &Query, new: &[(u32, u32)], word: u32) -> u32 {
        new.iter().find(|(held, _)| *held == word).map_or_else(|| words.kind_with(word, &case.story.extra), |(_, kind)| *kind)
    }

    pub(super) fn mark(&self, kind: u32, word: u32) -> u32 {
        if kind != ABSENT && self.thing_kinds.contains(&kind) { LINK_MARK | kind } else { word }
    }

    pub(super) fn new_kinds<N: WordTexts>(&self, words: &PatternWords, names: &mut N, case: &Query) -> Vec<(u32, u32)> {
        let mut votes: FirstSightTable<u32, FirstSightCount<u32>> = FirstSightTable::new();
        for told in &case.story.sents {
            let sent = LinkWords::apart(names, &told.words);
            let new: Vec<usize> = (0..sent.len()).filter(|at| !self.known.contains(&sent[*at]) && LinkWords::kind_of(words, case, &[], sent[*at]) == ABSENT).collect();
            if new.is_empty() {
                continue;
            }
            let marks: Vec<u32> = sent.iter().map(|word| self.mark(LinkWords::kind_of(words, case, &[], *word), *word)).collect();
            let Some(forms) = self.forms.get(&sent.len()) else { continue };
            let mut best: Option<(&Vec<u32>, u32)> = None;
            for (form, count) in forms.rows() {
                if best.is_some_and(|(_, top)| *count <= top) {
                    continue;
                }
                let fits = (0..sent.len()).all(|at| if new.contains(&at) { form[at] & LINK_MARK != 0 && self.thing_kinds.contains(&(form[at] & !LINK_MARK)) } else { form[at] == marks[at] });
                let mut pairs: Vec<(u32, u32)> = new.iter().map(|at| (sent[*at], form[*at])).collect();
                pairs.sort_unstable();
                pairs.dedup();
                let mut alone: Vec<u32> = new.iter().map(|at| sent[*at]).collect();
                alone.sort_unstable();
                alone.dedup();
                if fits && pairs.len() == alone.len() {
                    best = Some((form, *count));
                }
            }
            if let Some((form, count)) = best {
                for at in &new {
                    votes.at(sent[*at]).add(form[*at] & !LINK_MARK, i64::from(count));
                }
            }
        }
        votes.rows().iter().filter_map(|(word, counts)| counts.first_most().map(|(kind, _)| (*word, *kind))).collect()
    }

    pub(super) fn read<N: WordTexts>(&self, words: &PatternWords, names: &mut N, case: &Query, labels: &mut dyn FnMut(&str) -> u32) -> Option<LinkRead> {
        let new = self.new_kinds(words, names, case);
        let kind = |word: u32| LinkWords::kind_of(words, case, &new, word);
        let mut ends: Vec<u32> = Vec::new();
        for word in LinkWords::apart(names, &case.q) {
            let found = kind(word);
            if found != ABSENT && self.thing_kinds.contains(&found) && !ends.contains(&word) {
                ends.push(word);
            }
        }
        if ends.len() < LINK_ENDS {
            return None;
        }
        let kinds: Vec<u32> = ends.iter().map(|word| kind(*word)).collect();
        let mut read = LinkRead { ends, ..LinkRead::default() };
        for told in &case.story.sents {
            let sent = LinkWords::apart(names, &told.words);
            read.told.extend(sent.iter().copied());
            let mut things: Vec<u32> = Vec::new();
            for word in &sent {
                if kinds.contains(&kind(*word)) && !things.contains(word) {
                    things.push(*word);
                }
            }
            for start in &things {
                for goal in &things {
                    if start == goal {
                        continue;
                    }
                    let parts: Vec<String> = sent
                        .iter()
                        .map(|word| {
                            if word == start {
                                LINK_START.to_string()
                            } else if word == goal {
                                LINK_GOAL.to_string()
                            } else if kinds.contains(&kind(*word)) {
                                LINK_OTHER.to_string()
                            } else {
                                names.name(*word).to_string()
                            }
                        })
                        .collect();
                    let label = labels(&format!("{LINK_LABEL}{}", parts.join(LINK_BETWEEN)));
                    read.near.at(*start).push((*goal, label));
                }
            }
        }
        Some(read)
    }
}

pub(super) fn link_search(near: &LinkNear, start: u32, goal: u32, shut: &[u32]) -> Option<Vec<u32>> {
    let mut before: FirstSightTable<u32, Option<(u32, u32)>> = FirstSightTable::new();
    before.put(start, None);
    let mut queue: VecDeque<u32> = VecDeque::from([start]);
    while !before.holds(&goal) {
        let Some(thing) = queue.pop_front() else { break };
        for (next, label) in near.get(&thing).map_or(&[][..], Vec::as_slice) {
            if !before.holds(next) && !shut.contains(label) {
                before.put(*next, Some((thing, *label)));
                queue.push_back(*next);
            }
        }
    }
    before.get(&goal)?;
    let mut labels: Vec<u32> = Vec::new();
    let mut at = goal;
    while let Some(Some((back, label))) = before.get(&at) {
        labels.push(*label);
        at = *back;
    }
    labels.reverse();
    Some(labels)
}
because!(link_search, LinkWays, "the labels of the links on the shortest way from one thing to another, the first such way in the order of the sentences, with no link whose label is shut; nothing when no way leads there");
