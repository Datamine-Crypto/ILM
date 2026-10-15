use crate::base::sums::compensated;
use crate::tools::frames::FramesModel;
use crate::tools::frames::counts::{FRAME_CLEAR, FrameCandidate, names_of, plain};
use crate::tools::frames::slots::{Sentence, Who};
use crate::tools::frames::story::{ThingPair, ReadSentence, StoryRead, Toks};
use crate::tools::frames::texts::{READINGS, TextReading, Readings};
use crate::tools::pattern::hash::CheapMap;
use patterns::{because, source};
use std::collections::HashMap;

pub struct StoryFacts;
source!(StoryFacts, "a sentence that people wrote is read as a small story of facts. Each slot of a new sentence has its pairs of anchors as candidates, each with its share under the keys of its frame, taken over the pairs that the sentence has and whose names agree with the groups: the person of a fact is of the group of its word, and a token before the slot that goes with one group of owners must agree with the owner; a name that the lessons never held gets its group from the facts of its own story. The best candidate is the fact of the slot, and a plural slot can have more persons. The person of a slot can have no name: such a slot is then a thing for the other slots of its sentence when this gives one of them a better fact. The link of two names is read from the facts of its text as a story is read from its sentences: a fact about the two names gives the label, the relation word or its other side; else the facts on the shortest way from the one name to the other are joined by the join table of the lessons. Two sentences side by side are read only when no sentence holds both names, each with the other name of the link at the anchor out");

const FACT_LOWEST: f64 = 0.25;
because!(FACT_LOWEST, StoryFacts, "a candidate under this score is no fact");

const ALSO: f64 = 0.5;
because!(ALSO, StoryFacts, "a plural slot has a second person when its candidate has this share of the score of the first");

const FACT_SCORE: f64 = 4.0;
because!(FACT_SCORE, StoryFacts, "the score of a fact with all readings of its key in one way, below the score of a text of the lessons");

const CHAIN: f64 = 0.8;
because!(CHAIN, StoryFacts, "the factor of the score of a reading that joins facts, below a fact about the two names");

const FACT_LONGEST: usize = 3;
because!(FACT_LONGEST, StoryFacts, "the most facts on a way inside one text");

pub(super) type SlotFact = (Who, String, Who, f64, bool);
because!(SlotFact, StoryFacts, "a fact of a slot: its owner, its word, its person, its score at most one, and whether it is the fact of its slot or a further person of a plural slot");

pub(super) type FactEdge = (Who, String, f64);
because!(FactEdge, StoryFacts, "a link of a name by a fact: the other name, the label of the other name seen from this one, and the score of the fact");

#[derive(Clone, Debug, Default)]
pub(super) struct StoryView {
    pub(super) story: StoryRead,
    pub(super) groups: CheapMap<Who, String>,
}
because!(StoryView, StoryFacts, "a story as the tool reads it for a question: its links and texts, and the group of each name of it");

impl FramesModel {
    pub(super) fn view(&self, sentences: &[String], ends: &ThingPair) -> StoryView {
        let story = self.things.links(sentences, ends);
        let groups = self.story_groups(&story.read);
        StoryView { story, groups }
    }

    fn story_groups(&self, read: &[ReadSentence]) -> CheapMap<Who, String> {
        let mut groups: CheapMap<Who, String> = CheapMap::default();
        for sentence in read {
            for place in &sentence.places {
                let name = &sentence.words[*place];
                if let Some(group) = self.texts.group_of.get(name) {
                    groups.insert(Who::Name(name.clone()), group.clone());
                }
            }
        }
        let mut votes: HashMap<Who, Vec<(String, f64)>> = HashMap::new();
        let mut vote = |who: &Who, group: String, score: f64| {
            let counts = votes.entry(who.clone()).or_default();
            match counts.iter_mut().find(|(held, _)| *held == group) {
                Some((_, value)) => *value += score,
                None => counts.push((group, score)),
            }
        };
        for sentence in read {
            let seen = plain(sentence);
            for slot in self.slots(&seen) {
                let found = self.candidates(&seen, slot, Some(&groups), None);
                let Some((score, owner, person)) = found.first() else { continue };
                if *score < FACT_LOWEST {
                    continue;
                }
                let Some((word, _)) = self.slot_word(&sentence.words[slot]) else { continue };
                if let Some(asked) = self.asked_group(&word)
                    && !groups.contains_key(person)
                    && person.named()
                {
                    vote(person, asked, *score);
                }
                if let Some(mark) = self.mark_of(&seen, slot)
                    && !groups.contains_key(owner)
                    && owner.named()
                {
                    vote(owner, mark, *score);
                }
            }
        }
        for (who, counts) in votes {
            let all = compensated(counts.iter().map(|(_, value)| *value));
            let best = counts.iter().max_by(|left, right| left.1.partial_cmp(&right.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| left.0.cmp(&right.0)));
            if let Some((group, value)) = best
                && *value >= FRAME_CLEAR * all
            {
                groups.insert(who, group.clone());
            }
        }
        groups
    }

    fn facts(&self, sentence: &ReadSentence, out: Option<&String>, view: &StoryView, groups: &mut CheapMap<Who, String>) -> Vec<SlotFact> {
        let seen = plain(sentence);
        let slots = self.slots(&seen);
        let mut read: Vec<Vec<FrameCandidate>> = slots.iter().map(|slot| self.candidates(&seen, *slot, Some(&view.groups), out)).collect();
        let lone: Vec<usize> = slots.iter().zip(&read).filter(|(_, found)| found.first().is_some_and(|(score, _, person)| *score >= FACT_LOWEST && !person.named())).map(|(slot, _)| *slot).collect();
        if !lone.is_empty() && slots.len() > 1 {
            let mut known = view.groups.clone();
            for slot in &lone {
                if let Some((word, _)) = self.slot_word(&sentence.words[*slot])
                    && let Some(asked) = self.asked_group(&word)
                {
                    known.insert(Who::NoName(*slot), asked);
                }
            }
            for (at, slot) in slots.iter().enumerate() {
                let others: Vec<usize> = lone.iter().copied().filter(|held| held != slot).collect();
                if others.is_empty() {
                    continue;
                }
                let mut places: Vec<usize> = sentence.places.iter().chain(&others).copied().collect();
                places.sort_unstable();
                places.dedup();
                let also = Sentence { words: &sentence.words, places: &places, lone: &others };
                let found = self.candidates(&also, *slot, Some(&known), out);
                let Some((score, owner, person)) = found.first() else { continue };
                let before = read[at].first().map(|(score, _, _)| *score).unwrap_or_default();
                let alone = |who: &Who| matches!(who, Who::NoName(place) if others.contains(place));
                if *score > before && (alone(owner) || alone(person)) {
                    read[at] = found;
                }
            }
        }
        let mut facts: Vec<SlotFact> = Vec::new();
        for (slot, found) in slots.iter().zip(&read) {
            let Some((word, plural)) = self.slot_word(&sentence.words[*slot]) else { continue };
            for (number, (score, owner, person)) in found.iter().enumerate() {
                if *score < FACT_LOWEST {
                    break;
                }
                let first = number == 0 || (plural && *owner == found[0].1 && *score >= ALSO * found[0].0);
                facts.push((owner.clone(), word.clone(), person.clone(), score.min(f64::from(1u8)), first));
                if !person.named()
                    && let Some(asked) = self.asked_group(&word)
                {
                    groups.insert(person.clone(), asked);
                }
                if !owner.named()
                    && let Some(mark) = self.mark_of(&seen, *slot)
                {
                    groups.insert(owner.clone(), mark);
                }
            }
        }
        facts
    }

    fn forward(&self, word: &str, person: &Who, groups: &CheapMap<Who, String>) -> Option<String> {
        let meant = self.meant(word);
        if let [one] = meant {
            return Some(one.clone());
        }
        let group = groups.get(person);
        meant.iter().find(|label| self.texts.group.get(*label) == group && group.is_some()).cloned()
    }

    fn backward(&self, word: &str, owner: &Who, groups: &CheapMap<Who, String>) -> Option<String> {
        let group = groups.get(owner);
        let mut words: Vec<&str> = vec![word];
        for held in self.meant(word) {
            if !words.contains(&held.as_str()) {
                words.push(held);
            }
        }
        for held in words {
            let found = group.and_then(|group| self.side.get(&(held.to_string(), group.clone())).cloned());
            if found.is_some() {
                return found;
            }
            if group.is_none() {
                let mut sides: Vec<&String> = self.side.rows().iter().filter(|((other, _), _)| other == held).map(|(_, label)| label).collect();
                sides.sort();
                sides.dedup();
                if let [one] = sides.as_slice() {
                    return Some((*one).clone());
                }
            }
        }
        None
    }

    fn by_facts(&self, read: &[ReadSentence], pair: &ThingPair, view: &StoryView) -> Vec<(String, f64, TextReading)> {
        let (first, last) = pair;
        let mut groups = view.groups.clone();
        let mut found: Vec<SlotFact> = Vec::new();
        for sentence in read {
            let names = names_of(sentence);
            let out = if read.len() == 1 || names.contains(first) == names.contains(last) {
                None
            } else if names.contains(first) {
                Some(last)
            } else {
                Some(first)
            };
            found.extend(self.facts(sentence, out, view, &mut groups));
        }
        let mut edges: HashMap<Who, Vec<FactEdge>> = HashMap::new();
        let mut firsts: HashMap<Who, Vec<FactEdge>> = HashMap::new();
        for (owner, word, person, score, is_first) in &found {
            let ways = [(owner, person, self.forward(word, person, &groups)), (person, owner, self.backward(word, owner, &groups))];
            for (one, other, label) in ways {
                let Some(label) = label else { continue };
                edges.entry(one.clone()).or_default().push((other.clone(), label.clone(), *score));
                if *is_first {
                    firsts.entry(one.clone()).or_default().push((other.clone(), label, *score));
                }
            }
        }
        let (start, goal) = (Who::Name(first.clone()), Who::Name(last.clone()));
        let mut out: Vec<(String, f64, TextReading)> = edges.get(&start).map(Vec::as_slice).unwrap_or_default().iter().filter(|(other, _, _)| *other == goal).map(|(_, label, score)| (label.clone(), FACT_SCORE * score, TextReading::Fact)).collect();
        let mut layer: Vec<(Vec<Who>, Vec<String>, f64)> = vec![(vec![start], Vec::new(), f64::from(1u8))];
        for _ in 0..FACT_LONGEST {
            let mut later: Vec<(Vec<Who>, Vec<String>, f64)> = Vec::new();
            let mut ended = false;
            for (names, labels, score) in &layer {
                let Some(at) = names.last() else { continue };
                for (other, label, value) in firsts.get(at).map(Vec::as_slice).unwrap_or_default() {
                    if names.contains(other) {
                        continue;
                    }
                    if *other == goal {
                        if labels.is_empty() {
                            continue;
                        }
                        let mut chain = labels.clone();
                        chain.push(label.clone());
                        if let Some(got) = self.join.as_ref().and_then(|join| join.joined(&chain, false)).filter(|got| !got.is_empty()) {
                            out.push((got, FACT_SCORE * CHAIN * score * value, TextReading::Joined));
                            ended = true;
                        }
                    } else {
                        let mut more = names.clone();
                        more.push(other.clone());
                        let mut said = labels.clone();
                        said.push(label.clone());
                        later.push((more, said, score * value));
                    }
                }
            }
            if ended {
                break;
            }
            layer = later;
        }
        out.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    pub(super) fn facts_said(&self, view: &StoryView, pair: &ThingPair) -> Vec<String> {
        let spans: &[(usize, usize)] = view.story.spans.get(pair).map(Vec::as_slice).unwrap_or_default();
        let mut said: Vec<String> = Vec::new();
        for (one, other) in spans {
            let mut groups = view.groups.clone();
            for sentence in &view.story.read[*one..=*other] {
                for (owner, word, person, score, first) in self.facts(sentence, None, view, &mut groups) {
                    said.push(format!("{owner:?} {word} {person:?} {score:.2}{}", if first { "" } else { " later" }));
                }
            }
        }
        said
    }

    pub(super) fn readings(&self, view: &StoryView, pair: &ThingPair) -> (Readings, TextReading) {
        let texts: &[Toks] = view.story.texts.get(pair).map(Vec::as_slice).unwrap_or_default();
        let (got, how) = self.texts.read(texts, pair, &view.groups);
        if how == TextReading::Lessons {
            return (got, how);
        }
        let spans: &[(usize, usize)] = view.story.spans.get(pair).map(Vec::as_slice).unwrap_or_default();
        let across = spans.iter().any(|(one, other)| one == other);
        let mut best: Option<(String, f64, TextReading)> = None;
        for (one, other) in spans {
            if one != other && across {
                continue;
            }
            let Some(found) = self.by_facts(&view.story.read[*one..=*other], pair, view).into_iter().next() else { continue };
            if best.as_ref().is_none_or(|held| found.1 > held.1) {
                best = Some(found);
            }
        }
        let Some((label, score, how)) = best else { return (got, how) };
        let mut out: Readings = vec![(label.clone(), score)];
        out.extend(got.into_iter().filter(|(held, _)| *held != label).take(READINGS - 1));
        (out, how)
    }
}
