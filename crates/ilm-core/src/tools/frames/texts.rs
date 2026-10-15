use crate::tools::frames::counts::count_in;
use crate::base::first_sight::{FirstSightCount, all_counts};
use crate::base::sums::compensated;
use crate::tools::frames::slots::Who;
use crate::tools::frames::steps::LabelStep;
use crate::tools::frames::story::{MARK_FIRST, MARK_SECOND, ThingPair, StoryRead, Toks};
use crate::tools::pattern::hash::CheapMap;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct TextReadings;
source!(TextReadings, "the label of a link read from its text, counted on the steps of the lesson stories: for the whole text, for parts of it from the most exact to the least (the words from two before the one thing to two after the other; the words between the two things; the three, two and one words before and after each thing; the word before each thing; the words around the second thing), and for single marks (each word with its place from either thing, each word between them, each word at all). A text that the lessons hold has their labels, the most frequent first; a new text has the labels of its two most exact parts that count, the second with half the weight, and with no such part those of its single marks; a label of another group than the one of the second thing loses points, and the best three are kept with their scores");

pub struct NameGroups;
source!(NameGroups, "a name comes with some relations and not with others: a woman is never a son. Two groups of labels are joined when a fifth of the names of the smaller group, and two names or more, have labels of both, the pair with the largest share first, so that one name with labels of both groups cannot join them; a name is of a group when nearly all of its steps as the second name have labels of that group. Nothing else is taken from a name: which label a name had in the lessons is no vote, because a new story is about other people with the same names");

pub(super) const TEXT_LEAST: i64 = 2;
because!(TEXT_LEAST, TextReadings, NameGroups, "the readings from which a part of a text counts, a label counts for a name, and a pair of groups shares enough names; a single reading can be chance");

pub(super) const READINGS: usize = 3;
because!(READINGS, TextReadings, "the labels of a link that are kept with their scores for the check by the join");

const LESSON_SCALE: f64 = 10.0;
because!(LESSON_SCALE, TextReadings, "the score of a label of a text that the lessons hold, times the share of its readings: above every score of a new text");

const PART_SMOOTH: f64 = 0.2;
because!(PART_SMOOTH, TextReadings, "the readings that every label gets beside its count under a part of a text, so that a label that the part never showed is not ruled out");

const MARK_SMOOTH: f64 = 0.1;
because!(MARK_SMOOTH, TextReadings, "the readings that every label gets beside its count under a single mark");

const MARK_SPREAD: f64 = 2.0;
because!(MARK_SPREAD, TextReadings, "the readings added to the count of a label under which the count of a mark is taken, so that a rare label does not take every mark");

const MOST_PARTS: usize = 2;
because!(MOST_PARTS, TextReadings, "the parts of a text that count, the most exact ones that the lessons show; a second one is taken with half the weight");

const PART_REACH: usize = 2;
because!(PART_REACH, TextReadings, "the words before the first thing and after the last in the fullest part of a text: enough for the word of the relation beside a name, few enough that the part keeps to one relation");

const PART_WIDEST: usize = 3;
because!(PART_WIDEST, TextReadings, "the most words on each side of each thing in a part of a text; parts with fewer words follow");

const MARK_WINDOW: isize = 3;
because!(MARK_WINDOW, TextReadings, "the places on each side of a thing that give single marks");

const OTHER_GROUP: f64 = -8.0;
because!(OTHER_GROUP, TextReadings, NameGroups, "the vote against a label of another group than the one of the second thing");

pub(super) const LOWEST: f64 = -20.0;
because!(LOWEST, TextReadings, "the least score of a reading of a new text, whose score is never above nought");

const SHARED: f64 = 0.2;
because!(SHARED, NameGroups, "two groups of labels are joined when this share of the names of the smaller one have labels of both");

pub(super) const GROUP_SHARE: f64 = 0.98;
because!(GROUP_SHARE, NameGroups, "the share of the steps of a name whose labels must be of one group before the name is of that group");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct TextPart {
    kind: usize,
    order: bool,
    pieces: Vec<Toks>,
}
because!(TextPart, TextReadings, "a part of a text: its kind as its place in the list of parts, whether the first thing stands before the second, and its runs of words");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum TextMark {
    Order(bool),
    Near(bool, bool, isize, String),
    Between(bool, String),
    Any(bool, String),
}
because!(TextMark, TextReadings, "a single mark of a text: the order of its two things; a word with its place from the first or the second thing; a word between them; a word anywhere in it; each with the order of the things");

#[derive(Clone, Debug, Default)]
pub(super) struct LabelCount {
    pub(super) by: CheapMap<String, i64>,
    pub(super) all: i64,
}
because!(LabelCount, TextReadings, "how often each label stands under one part or one mark, and how often any label does");

impl LabelCount {
    fn add(&mut self, label: &str) {
        *self.by.entry(label.to_string()).or_default() += 1;
        self.all += 1;
    }

}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TextReading {
    Lessons,
    Parts,
    Marks,
    Fact,
    Joined,
}
because!(TextReading, TextReadings, "what gave the readings of a link: a text of the lessons, parts of a new text, single marks, a fact of its sentence, or facts of its sentence joined");

pub(super) type Readings = Vec<(String, f64)>;
because!(Readings, TextReadings, "the labels of a link with their scores, the best first");

#[derive(Clone, Debug, Default)]
pub(super) struct TextCounts {
    pub(super) whole: HashMap<Toks, FirstSightCount<String>>,
    pub(super) parts: HashMap<TextPart, LabelCount>,
    pub(super) marks: HashMap<TextMark, LabelCount>,
    pub(super) names: HashMap<String, FirstSightCount<String>>,
    pub(super) labels: FirstSightCount<String>,
    pub(super) total: i64,
    pub(super) group: CheapMap<String, String>,
    pub(super) group_of: CheapMap<String, String>,
    pub(super) all_groups: Vec<String>,
}
because!(TextCounts, TextReadings, NameGroups, "what the steps of the lesson stories taught about texts: the labels of each whole text, of each part and of each single mark, the labels of each name as the second name of a step, how often each label stands, the group of each label and of each name, and the groups in the order of their texts. add counts one step with the texts of its link; grouped makes the groups; read gives the readings of a link from its texts");

fn text_place(toks: &[String], mark: &str) -> usize {
    toks.iter().position(|word| word == mark).unwrap_or_default()
}
because!(text_place, TextReadings, "where a text holds the sign that stands for a thing, and the first place when it holds none");

fn piece(toks: &[String], from: usize, to: usize) -> Toks {
    let end = to.min(toks.len());
    toks[from.min(end)..end].to_vec()
}
because!(piece, TextReadings, "the words of a text from one place to the place before another, cut at the end of the text");

fn text_parts(toks: &[String]) -> Vec<TextPart> {
    let (first, second) = (text_place(toks, MARK_FIRST), text_place(toks, MARK_SECOND));
    let order = first < second;
    let (low, high) = (first.min(second), first.max(second));
    let mut pieces: Vec<Vec<Toks>> = vec![vec![piece(toks, low.saturating_sub(PART_REACH), high + PART_REACH + 1)], vec![piece(toks, low + 1, high)]];
    for width in (1..=PART_WIDEST).rev() {
        pieces.push(vec![piece(toks, first.saturating_sub(width), first), piece(toks, first + 1, first + 1 + width), piece(toks, second.saturating_sub(width), second), piece(toks, second + 1, second + 1 + width)]);
    }
    let before = |at: usize| at.checked_sub(1).map(|from| piece(toks, from, at)).unwrap_or_default();
    pieces.push(vec![before(first), before(second)]);
    pieces.push(vec![before(second), piece(toks, second + 1, second + 1 + 1)]);
    pieces.into_iter().enumerate().map(|(kind, pieces)| TextPart { kind, order, pieces }).collect()
}
because!(text_parts, TextReadings, "the parts of a text with its two things, from the most exact part to the least");

fn text_marks(toks: &[String]) -> HashSet<TextMark> {
    let (first, second) = (text_place(toks, MARK_FIRST), text_place(toks, MARK_SECOND));
    let order = first < second;
    let mut out: HashSet<TextMark> = HashSet::new();
    out.insert(TextMark::Order(order));
    for (is_second, at) in [(false, first), (true, second)] {
        for step in -MARK_WINDOW..=MARK_WINDOW {
            let place = at as isize + step;
            if step != 0 && place >= 0 && (place as usize) < toks.len() {
                out.insert(TextMark::Near(order, is_second, step, toks[place as usize].clone()));
            }
        }
    }
    for word in &toks[first.min(second) + 1..first.max(second)] {
        out.insert(TextMark::Between(order, word.clone()));
    }
    for word in toks {
        out.insert(TextMark::Any(order, word.clone()));
    }
    out
}
because!(text_marks, TextReadings, "the single marks of a text with its two things");

impl TextCounts {
    pub(super) fn add(&mut self, step: &LabelStep, story: &StoryRead) {
        let (first, label, second) = step;
        self.labels.add(label.clone(), 1);
        self.names.entry(second.clone()).or_default().add(label.clone(), 1);
        for toks in story.texts.get(&(first.clone(), second.clone())).map(Vec::as_slice).unwrap_or_default() {
            self.whole.entry(toks.clone()).or_default().add(label.clone(), 1);
            for part in text_parts(toks) {
                self.parts.entry(part).or_default().add(label);
            }
            for mark in text_marks(toks) {
                self.marks.entry(mark).or_default().add(label);
            }
        }
    }

    pub(super) fn grouped(&mut self) {
        self.total = all_counts(&self.labels);
        let taken: Vec<HashSet<&String>> = self.names.values().map(|counts| counts.rows().iter().filter(|(_, count)| *count >= TEXT_LEAST).map(|(label, _)| label).collect()).collect();
        let mut members: Vec<(String, Vec<String>)> = self.labels.rows().iter().map(|(label, _)| (label.clone(), vec![label.clone()])).collect();
        members.sort();
        loop {
            let have: Vec<HashSet<usize>> = members.iter().map(|(_, labels)| (0..taken.len()).filter(|name| labels.iter().any(|label| taken[*name].contains(label))).collect()).collect();
            let mut best: Option<(f64, usize, usize)> = None;
            for one in 0..members.len() {
                for other in one + 1..members.len() {
                    let both = have[one].intersection(&have[other]).count();
                    let small = have[one].len().min(have[other].len());
                    if both as i64 >= TEXT_LEAST && both as f64 >= SHARED * small as f64 && best.is_none_or(|(share, _, _)| both as f64 / small as f64 > share) {
                        best = Some((both as f64 / small as f64, one, other));
                    }
                }
            }
            let Some((_, one, other)) = best else { break };
            let (_, joined) = members.remove(other);
            members[one].1.extend(joined);
        }
        self.group = members.iter().flat_map(|(head, labels)| labels.iter().map(move |label| (label.clone(), head.clone()))).collect();
        self.group_of = CheapMap::default();
        for (name, counts) in &self.names {
            let mut by_group: FirstSightCount<String> = FirstSightCount::new();
            for (label, count) in counts.rows() {
                by_group.add(self.group[label].clone(), *count);
            }
            if let Some((top, hit)) = by_group.first_most()
                && *hit as f64 >= GROUP_SHARE * all_counts(counts) as f64
            {
                self.group_of.insert(name.clone(), top.clone());
            }
        }
        let mut heads: Vec<String> = members.into_iter().map(|(head, _)| head).collect();
        heads.sort();
        self.all_groups = heads;
    }

    pub(super) fn vote(&self, groups: &CheapMap<Who, String>, name: &str, label: &str) -> f64 {
        match groups.get(&Who::Name(name.to_string())) {
            Some(mine) if self.group.get(label) != Some(mine) => OTHER_GROUP,
            _ => f64::default(),
        }
    }

    pub(super) fn read(&self, texts: &[Toks], pair: &ThingPair, groups: &CheapMap<Who, String>) -> (Readings, TextReading) {
        let known = texts.iter().filter_map(|toks| self.whole.get(toks).map(|counts| (all_counts(counts), toks))).max();
        if let Some((all, toks)) = known {
            let counts = &self.whole[toks];
            let got = counts.by_count().into_iter().take(READINGS).map(|(label, hit)| (label.clone(), LESSON_SCALE * *hit as f64 / all as f64)).collect();
            return (got, TextReading::Lessons);
        }
        let Some(toks) = texts.iter().min_by_key(|toks| toks.len()) else { return (Vec::new(), TextReading::Marks) };
        let size = self.labels.rows().len() as f64;
        let mut score: Vec<(String, f64)> = self.labels.rows().iter().map(|(label, _)| (label.clone(), f64::default())).collect();
        let mut used = 0usize;
        for part in text_parts(toks) {
            let Some(counts) = self.parts.get(&part).filter(|counts| counts.all >= TEXT_LEAST) else { continue };
            for (label, value) in score.iter_mut() {
                *value += ((count_in(&counts.by, label) as f64 + PART_SMOOTH) / (counts.all as f64 + PART_SMOOTH * size)).ln() / (used as f64 + f64::from(1u8));
            }
            used += 1;
            if used == MOST_PARTS {
                break;
            }
        }
        if used == 0 {
            let marks: Vec<&LabelCount> = text_marks(toks).iter().filter_map(|mark| self.marks.get(mark)).collect();
            for ((label, count), (_, value)) in self.labels.rows().iter().zip(score.iter_mut()) {
                *value += (*count as f64 / self.total as f64).ln() + compensated(marks.iter().map(|counts| ((count_in(&counts.by, label) as f64 + MARK_SMOOTH) / (*count as f64 + MARK_SPREAD)).ln()));
            }
        }
        for (label, value) in score.iter_mut() {
            *value += self.vote(groups, &pair.1, label);
        }
        score.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| left.0.cmp(&right.0)));
        score.truncate(READINGS);
        let got = score.into_iter().map(|(label, value)| (label, value.min(f64::default()).max(LOWEST))).collect();
        (got, if used > 0 { TextReading::Parts } else { TextReading::Marks })
    }
}
