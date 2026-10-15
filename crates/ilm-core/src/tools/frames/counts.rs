use crate::base::sums::compensated;
use crate::tools::frames::FramesModel;
use crate::tools::frames::slots::{ANCHOR_ORDER, Anchor, At, SlotKey, Sentence, Where, Who, plural_forms};
use crate::tools::frames::steps::LabelStep;
use crate::tools::frames::story::ReadSentence;
use crate::tools::pattern::hash::CheapMap;
use patterns::{because, source};
use std::collections::{BTreeMap, HashMap, HashSet};

pub struct FrameCounts;
source!(FrameCounts, "at which anchors the owner and the person of a slot stand, counted on the lesson sentences for each key of its frame. A step of a lesson story says that its second name is the relation of its first: a slot of a sentence with both names whose word is the relation has the first name as owner and the second as person; a slot whose word is the other side of the relation has them the other way round; a slot that fits both ways is counted only when the group of the first name allows one way alone. A slot that no step with both names fits is read with the steps that have one name in the sentence, the other name at the anchor out. A slot that no step of its story fits is counted too, since its person can have no name: the share of a pair of anchors is taken over all lesson slots with the key. The share under a key is its count with the share under the next shorter key as more readings, from the shortest key to the fullest");

pub struct OwnerMarks;
source!(OwnerMarks, "a token before a slot can go with one group of owners, as a word that only a woman or only a man is called by: counted on the lesson slots, a token with enough readings, nearly all with one group of owners, gives that group to the owner of a slot after it");

pub(super) const MANY: i64 = 5;
because!(MANY, FrameCounts, "the lesson slots under a key from which its most frequent place is sure enough to read a word from it, the readings of a new word, and the plural slots from which the share of a second person is taken");

pub(super) const FRAME_CLEAR: f64 = 0.8;
because!(FRAME_CLEAR, FrameCounts, "the share of the readings that makes the place of a slot sure, a new word mean a label, and a name of a new story take a group");

const SMOOTH: f64 = 8.0;
because!(SMOOTH, FrameCounts, "the share under the next shorter key counts as this many readings of a key");

const MARK_READINGS: i64 = 20;
because!(MARK_READINGS, OwnerMarks, "the readings of a token before a slot from which it can go with one group of owners");

const MARK_SHARE: f64 = 0.9;
because!(MARK_SHARE, OwnerMarks, "the share of its readings that must have one group of owners before a token goes with that group");

const MARK_NEAR: usize = 3;
because!(MARK_NEAR, OwnerMarks, "the tokens before a slot where a token that goes with a group is looked for, up to the first thing");

#[derive(Clone, Debug, Default)]
pub(super) struct AtCount {
    pub(super) by: CheapMap<At, i64>,
    pub(super) all: i64,
}
because!(AtCount, FrameCounts, "how often the owner and the person of the lesson slots under one key stood at each pair of anchors, and how often at any");

pub(super) type FrameCandidate = (f64, Who, Who);
because!(FrameCandidate, FrameCounts, "a candidate fact of a slot: its score, its owner and its person");

pub(super) type LessonStory = (Vec<ReadSentence>, Vec<LabelStep>);
because!(LessonStory, FrameCounts, "a lesson story whose steps make a chain: its sentences with the places of their things, and its steps");

pub(super) fn plain(sentence: &ReadSentence) -> Sentence<'_> {
    Sentence { words: &sentence.words, places: &sentence.places, lone: &[] }
}
because!(plain, FrameCounts, "a sentence with no slot that stands as a thing");

pub(super) fn count_in<K: std::hash::Hash + Eq + std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized, S: std::hash::BuildHasher>(table: &HashMap<K, i64, S>, key: &Q) -> i64 {
    table.get(key).copied().unwrap_or_default()
}
because!(count_in, FrameCounts, "the count of a key in a table of counts, and nothing counted for a key that the table does not hold");

pub(super) fn names_of(sentence: &ReadSentence) -> HashSet<&String> {
    sentence.places.iter().map(|place| &sentence.words[*place]).collect()
}
because!(names_of, FrameCounts, "the words at the marked places of a sentence: the things that its links can join");

pub(super) fn both_in<'a>(steps: &'a [LabelStep], names: &HashSet<&String>) -> Vec<&'a LabelStep> {
    steps.iter().filter(|(first, _, second)| names.contains(first) && names.contains(second) && first != second).collect()
}
because!(both_in, FrameCounts, "the steps of a story whose two names both stand in a sentence");

impl FramesModel {
    pub(super) fn slot_word(&self, word: &str) -> Option<(String, bool)> {
        if self.means.holds(&word.to_string()) {
            return Some((word.to_string(), false));
        }
        plural_forms(word).into_iter().find(|single| self.means.holds(single)).map(|single| (single, true))
    }

    pub(super) fn slots(&self, sentence: &Sentence) -> Vec<usize> {
        (0..sentence.words.len()).filter(|place| !sentence.holds(*place) && self.slot_word(&sentence.words[*place]).is_some()).collect()
    }

    pub(super) fn meant(&self, word: &str) -> &[String] {
        self.means.get(&word.to_string()).map(Vec::as_slice).unwrap_or_default()
    }

    pub(super) fn asked_group(&self, word: &str) -> Option<String> {
        let groups: HashSet<Option<&String>> = self.meant(word).iter().map(|label| self.texts.group.get(label)).collect();
        match groups.into_iter().collect::<Vec<_>>().as_slice() {
            [one] => one.cloned(),
            _ => None,
        }
    }

    pub(super) fn ways_of(&self, word: &str, label: &str, first: Option<&String>) -> (bool, bool) {
        let group = self.texts.group.get(label).cloned().unwrap_or_default();
        let meant = self.meant(word);
        let same = meant.iter().any(|held| held == label);
        let mut words: Vec<&str> = vec![word];
        for held in meant {
            if !words.contains(&held.as_str()) {
                words.push(held);
            }
        }
        let mut turned = words.iter().any(|held| self.side.get(&(held.to_string(), group.clone())).is_some_and(|side| side == label));
        if same && turned && let Some(first) = first {
            let asked: HashSet<Option<&String>> = meant.iter().map(|held| self.texts.group.get(held)).collect();
            if asked.len() == 1 && let Some(mine) = self.texts.group_of.get(first) && asked.iter().next().is_some_and(|one| *one != Some(mine)) {
                turned = false;
            }
        }
        (same, turned)
    }

    pub(super) fn count_frames(&mut self, stories: &[LessonStory], first: bool) {
        let mut frames: CheapMap<SlotKey, AtCount> = CheapMap::default();
        let mut unread: CheapMap<SlotKey, i64> = CheapMap::default();
        let mut later: HashMap<Anchor, i64> = HashMap::new();
        let known: HashSet<&String> = self.side.rows().iter().map(|((word, _), _)| word).collect();
        for (read, steps) in stories {
            for sentence in read {
                let names = names_of(sentence);
                let inside = both_in(steps, &names);
                let outside: Vec<&LabelStep> = steps.iter().filter(|(one, _, other)| names.contains(one) != names.contains(other)).collect();
                if inside.is_empty() && outside.is_empty() {
                    continue;
                }
                let seen = plain(sentence);
                for slot in self.slots(&seen) {
                    let Some((word, plural)) = self.slot_word(&sentence.words[slot]) else { continue };
                    let at = seen.anchors(slot);
                    let keys = seen.keys(slot, &at);
                    let mut fitted = false;
                    for pairs in [&inside, &outside] {
                        if fitted {
                            break;
                        }
                        for (one, label, other) in pairs.iter().copied() {
                            let (same, turned) = self.ways_of(&word, label, Some(one));
                            let group = self.texts.group.get(label).cloned().unwrap_or_default();
                            if same == turned || (first && same && known.contains(&word) && !self.side.holds(&(word.clone(), group))) {
                                fitted = fitted || same;
                                continue;
                            }
                            let place_of = |name: &String| -> Anchor {
                                if !names.contains(name) {
                                    return Anchor::Out;
                                }
                                ANCHOR_ORDER.iter().copied().find(|anchor| at.get(*anchor).is_some_and(|place| sentence.words[place] == *name)).unwrap_or(Anchor::Far)
                            };
                            let (owner, person) = if same { (one, other) } else { (other, one) };
                            let mut pair: At = (place_of(owner), place_of(person));
                            if plural {
                                *later.entry(pair.1).or_default() += 1;
                                if pair.1 == Anchor::RightTwo && pair.0 != Anchor::RightOne {
                                    pair = (pair.0, Anchor::RightOne);
                                }
                            }
                            for key in &keys {
                                let count = frames.entry(key.clone()).or_default();
                                *count.by.entry(pair).or_default() += 1;
                                count.all += 1;
                            }
                            fitted = true;
                        }
                    }
                    if !fitted {
                        for key in &keys {
                            *unread.entry(key.clone()).or_default() += 1;
                        }
                    }
                }
            }
        }
        self.frames = frames;
        self.unread = unread;
        let (beyond, next) = (later.get(&Anchor::RightTwo).copied().unwrap_or_default(), later.get(&Anchor::RightOne).copied().unwrap_or_default());
        self.second = if next >= MANY { (beyond as f64 / next as f64).min(f64::from(1u8)) } else { f64::default() };
    }


    pub(super) fn shares(&self, sentence: &Sentence, slot: usize, at: &Where) -> BTreeMap<At, f64> {
        let mut share: BTreeMap<At, f64> = BTreeMap::new();
        for key in sentence.keys(slot, at).iter().rev() {
            let counts = self.frames.get(key);
            let all = counts.map(|counts| counts.all).unwrap_or_default() + count_in(&self.unread, key);
            if all == 0 {
                continue;
            }
            if let Some(counts) = counts {
                for pair in counts.by.keys() {
                    share.entry(*pair).or_default();
                }
            }
            for (pair, value) in share.iter_mut() {
                let count = counts.and_then(|counts| counts.by.get(pair)).copied().unwrap_or_default();
                *value = (count as f64 + SMOOTH * *value) / (all as f64 + SMOOTH);
            }
        }
        share
    }

    pub(super) fn sure(&self, sentence: &Sentence, slot: usize) -> Option<(usize, usize, i64)> {
        let at = sentence.anchors(slot);
        for key in sentence.keys(slot, &at) {
            let Some(counts) = self.frames.get(&key) else { continue };
            let all = counts.all + count_in(&self.unread, &key);
            if !key.has_token() || all < MANY {
                continue;
            }
            let (pair, hit) = counts.by.iter().max_by(|left, right| (left.1, left.0).cmp(&(right.1, right.0)))?;
            if *hit as f64 >= FRAME_CLEAR * all as f64
                && let (Some(owner), Some(person)) = (at.get(pair.0), at.get(pair.1))
            {
                return Some((owner, person, counts.all));
            }
            return None;
        }
        None
    }

    pub(super) fn mark_of(&self, sentence: &Sentence, slot: usize) -> Option<String> {
        for place in (slot.saturating_sub(MARK_NEAR)..slot).rev() {
            if sentence.holds(place) {
                break;
            }
            if let Some(group) = self.mark_group.get(&sentence.words[place]) {
                return Some(group.clone());
            }
        }
        None
    }

    pub(super) fn candidates(&self, sentence: &Sentence, slot: usize, groups: Option<&CheapMap<Who, String>>, out: Option<&String>) -> Vec<FrameCandidate> {
        let Some((word, plural)) = self.slot_word(&sentence.words[slot]) else { return Vec::new() };
        let at = sentence.anchors(slot);
        let name = |anchor: Anchor| -> Option<Who> {
            if anchor == Anchor::Out {
                return Some(out.map_or(Who::NoName(slot), |out| Who::Name(out.clone())));
            }
            at.get(anchor).map(|place| sentence.who(place))
        };
        let asked = self.asked_group(&word);
        let mark = groups.and_then(|_| self.mark_of(sentence, slot));
        let share = self.shares(sentence, slot, &at);
        let rest = (f64::from(1u8) - compensated(share.values().copied())).max(f64::default());
        let mut can: Vec<(f64, Who, Who)> = Vec::new();
        for (pair, value) in &share {
            let (Some(owner), Some(person)) = (name(pair.0), name(pair.1)) else { continue };
            if owner == person {
                continue;
            }
            if let Some(groups) = groups {
                let fits = |who: &Who, wanted: &Option<String>| wanted.is_none() || groups.get(who).is_none_or(|group| Some(group) == wanted.as_ref());
                if !fits(&person, &asked) || !fits(&owner, &mark) {
                    continue;
                }
            }
            can.push((*value, owner, person));
        }
        let mut total = compensated(can.iter().map(|(value, _, _)| *value)) + rest;
        if total == f64::default() {
            total = f64::from(1u8);
        }
        let mut found: Vec<FrameCandidate> = can.into_iter().map(|(value, owner, person)| (value / total, owner, person)).collect();
        if plural
            && self.second != f64::default()
            && let (Some(next), Some(beyond)) = (name(Anchor::RightOne), name(Anchor::RightTwo))
        {
            for (score, owner, person) in found.clone() {
                if person != next || owner == beyond {
                    continue;
                }
                let old: Vec<FrameCandidate> = found.iter().filter(|(_, held, other)| *held == owner && *other == beyond).cloned().collect();
                if old.first().is_none_or(|(value, _, _)| *value < self.second * score) {
                    found.retain(|item| !old.contains(item));
                    found.push((self.second * score, owner.clone(), beyond.clone()));
                }
            }
        }
        found.sort_by(|left, right| right.0.partial_cmp(&left.0).unwrap_or(std::cmp::Ordering::Equal).then_with(|| left.1.text().cmp(&right.1.text())).then_with(|| left.2.text().cmp(&right.2.text())));
        found
    }

    pub(super) fn count_marks(&mut self, stories: &[LessonStory]) {
        let mut marks: HashMap<String, Vec<(String, i64)>> = HashMap::new();
        for (read, steps) in stories {
            for sentence in read {
                let pairs = both_in(steps, &names_of(sentence));
                if pairs.is_empty() {
                    continue;
                }
                let seen = plain(sentence);
                for slot in self.slots(&seen) {
                    let Some((word, _)) = self.slot_word(&sentence.words[slot]) else { continue };
                    for (one, label, _) in &pairs {
                        let (same, turned) = self.ways_of(&word, label, Some(one));
                        if same == turned {
                            continue;
                        }
                        let group = if turned { self.texts.group.get(label) } else { self.texts.group_of.get(one) };
                        let Some(group) = group else { continue };
                        for place in (slot.saturating_sub(MARK_NEAR)..slot).rev() {
                            if seen.holds(place) {
                                break;
                            }
                            let counts = marks.entry(sentence.words[place].clone()).or_default();
                            match counts.iter_mut().find(|(held, _)| held == group) {
                                Some((_, count)) => *count += 1,
                                None => counts.push((group.clone(), 1)),
                            }
                        }
                    }
                }
            }
        }
        self.mark_group = CheapMap::default();
        for (mark, counts) in marks {
            let all: i64 = counts.iter().map(|(_, count)| *count).sum();
            let Some((group, hit)) = counts.iter().max_by(|left, right| (left.1, &left.0).cmp(&(right.1, &right.0))) else { continue };
            if all >= MARK_READINGS && *hit as f64 >= MARK_SHARE * all as f64 {
                self.mark_group.insert(mark, group.clone());
            }
        }
    }
}
