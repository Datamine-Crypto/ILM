pub mod counts;
pub mod facts;
pub mod said;
pub mod sides;
pub mod slots;
pub mod steps;
pub mod story;
pub mod texts;

use crate::base::first_sight::{FirstSightCount, FirstSightTable};
use crate::base::lanes::{on_lanes, shards};
use crate::base::sums::compensated;
use crate::base::words::answer_words;
use crate::cases::Case;
use crate::tools::call::{Learned, Learner, Lessons, Said, Tool};
use crate::tools::frames::counts::{AtCount, LessonStory};
use crate::tools::frames::said::first_meanings;
use crate::tools::frames::sides::Sides;
use crate::tools::frames::slots::{SlotKey, Who};
use crate::tools::frames::steps::{STEP_BETWEEN, chain_of};
use crate::tools::frames::story::{ThingPair, ThingWords};
use crate::tools::frames::texts::{READINGS, TextReading, Readings, TextCounts};
use crate::tools::frequent::frequent_answer;
use crate::tools::link::chains::ChainJoins;
use crate::tools::pattern::hash::CheapMap;
use patterns::{because, source};

pub struct FramesTool;
source!(FramesTool, "a reader of told facts in sentences that people wrote, with the join of the link tool: a person writes one relation in many ways, and a new story holds sentences that no lesson held, so the label of a link is read from the frames of the relation words of its sentence, counted on the labels of the lesson stories, which say what each link of the way of a lesson is; the labels of the way are then joined by the join table that the lessons teach, as the link tool joins the links of a way. Nothing is written by hand, and the tool holds no word of a set: what it reads is a label of the lessons or a word that the lessons taught to mean one");

pub struct FramesAnswer;
source!(FramesAnswer, "the answer of a story: the way from the first thing of the question to the second over the links of the story; the way is one link when the lessons hold a text of that link as a step; else the kind of way that gave the chain of the steps of the lesson stories more often, the shortest way or the way with the best scores. One link gives its best label; for two links or more the sets of readings, one for each link, are tried from the best sum of scores down, and the first set that the join table joins gives the answer, so that a reading that fits no join is given up for the next one");

const WAY_SAMPLE: usize = 2000;
because!(WAY_SAMPLE, FramesAnswer, "the most lesson stories on which the two kinds of way are compared: more change nothing");

const MOST_READING_SETS: usize = 300;
because!(MOST_READING_SETS, FramesAnswer, "the most sets of readings that the check by the join tries for one way");

const WIDE_WAY: usize = 6;
because!(WIDE_WAY, FramesAnswer, "the most links of a way whose sets of readings take all kept readings of each link; a longer way takes fewer, so the sets stay few");

const NARROW_READINGS: usize = 2;
because!(NARROW_READINGS, FramesAnswer, "the readings of each link that the sets of a longer way take");

const FRAMES_NOT_TWO: &str = "not two things of the story in the question";
because!(FRAMES_NOT_TWO, FramesAnswer, "the note of an answer to a question that does not name two things of its story");

const FRAMES_NO_WAY: &str = "no way between the two things";
because!(FRAMES_NO_WAY, FramesAnswer, "the note of an answer to a question whose things no chain of links joins");

const FRAMES_ONE_LINK: &str = "one link read";
because!(FRAMES_ONE_LINK, FramesAnswer, "the note of an answer that the best label of the only link of the way gives");

const FRAMES_JOINED: &str = "readings joined";
because!(FRAMES_JOINED, FramesAnswer, "the note of an answer that the join of a set of readings gives");

const FRAMES_NO_JOIN: &str = "no set of readings joins";
because!(FRAMES_NO_JOIN, FramesAnswer, "the note of an answer to a way whose sets of readings the join table joins in no way");

const NO_CHAIN: &str = "no lesson story has labels that make a chain of steps between the things of its question";
because!(NO_CHAIN, FramesTool, "what the tool says for a kind whose lessons give it nothing to learn from");

#[derive(Default)]
struct FramesModel {
    things: ThingWords,
    texts: TextCounts,
    means: FirstSightTable<String, Vec<String>>,
    side: Sides,
    pair: FirstSightTable<(String, String), String>,
    mark_group: CheapMap<String, String>,
    frames: CheapMap<SlotKey, AtCount>,
    unread: CheapMap<SlotKey, i64>,
    second: f64,
    scored: bool,
    join: Option<ChainJoins>,
    default: Vec<String>,
}
because!(FramesModel, FramesTool, "what the tool learned: the thing words and the signs around them, what the texts of the lesson steps taught with the groups of labels and names, the labels that each word means, the other sides of the words, the labels that are one relation for two groups, the groups that tokens before a slot go with, the places of owners and persons under each key of a frame, the share of a second person after a plural slot, the kind of way, the join table, and the most frequent answer");

pub(super) struct Chain {
    pub(super) labels: Vec<String>,
    pub(super) answer: String,
    pub(super) ends: ThingPair,
    pub(super) pairs: Vec<ThingPair>,
    pub(super) case: usize,
}
because!(Chain, FramesTool, "a lesson whose steps make a chain: the labels of its steps, its answer, the things of its question, the pairs of names of its steps, and its place among the lessons");

impl FramesModel {
    fn learn(cases: &[Case], labels: &[Vec<String>]) -> Result<FramesModel, String> {
        let things = ThingWords::counted(cases);
        let mut texts = TextCounts::default();
        let mut chains: Vec<Chain> = Vec::new();
        let mut stories: Vec<LessonStory> = Vec::new();
        let mut often: FirstSightCount<String> = FirstSightCount::new();
        for (place, case) in cases.iter().enumerate() {
            let Some(lines) = labels.get(place) else { continue };
            let Some(ends) = things.ends(&case.sentences, &case.question) else { continue };
            let Some(steps) = chain_of(lines, &ends) else { continue };
            let story = things.links(&case.sentences, &ends);
            for step in &steps {
                texts.add(step, &story);
            }
            let answer = answer_words(&case.answer).join(STEP_BETWEEN);
            often.add(answer.clone(), 1);
            chains.push(Chain { labels: steps.iter().map(|(_, label, _)| label.clone()).collect(), answer, ends, pairs: steps.iter().map(|(first, _, second)| (first.clone(), second.clone())).collect(), case: place });
            stories.push((story.read, steps));
        }
        if chains.is_empty() {
            return Err(NO_CHAIN.to_string());
        }
        texts.grouped();
        for (label, count) in texts.labels.rows() {
            often.add(label.clone(), *count);
        }
        let join = ChainJoins::learn(&chains.iter().map(|chain| (chain.labels.clone(), chain.answer.clone())).collect::<Vec<_>>(), &often);
        let means = first_meanings(&texts.labels);
        let mut model = FramesModel { things, texts, means, join: Some(join), default: frequent_answer(cases).0, ..FramesModel::default() };
        model.scored = model.way_kind(cases, &chains);
        model.first_sides(&stories);
        model.count_frames(&stories, true);
        model.count_sides(&stories);
        model.count_frames(&stories, false);
        model.count_marks(&stories);
        if !model.learn_words(&stories).is_empty() {
            model.count_frames(&stories, false);
        }
        Ok(model)
    }

    fn way_kind(&self, cases: &[Case], chains: &[Chain]) -> bool {
        let groups: CheapMap<Who, String> = self.texts.group_of.iter().map(|(name, group)| (Who::Name(name.clone()), group.clone())).collect();
        let (mut shortest, mut best) = (0usize, 0usize);
        for chain in chains.iter().step_by((chains.len() / WAY_SAMPLE).max(1)) {
            let story = self.things.links(&cases[chain.case].sentences, &chain.ends);
            for scored in [false, true] {
                let way = ThingWords::way(&story, &chain.ends, scored, &mut |pair| {
                    let texts = story.texts.get(pair).map(Vec::as_slice).unwrap_or_default();
                    let (got, how) = self.texts.read(texts, pair, &groups);
                    (got.first().map(|(_, score)| *score).unwrap_or_default(), how == TextReading::Lessons)
                });
                if way.as_ref() == Some(&chain.pairs) {
                    if scored {
                        best += 1;
                    } else {
                        shortest += 1;
                    }
                }
            }
        }
        best > shortest
    }

    fn said(&self, note: String) -> Said {
        Said { words: self.default.clone(), note }
    }

    fn way_of(&self, view: &facts::StoryView, ends: &ThingPair) -> Option<Vec<ThingPair>> {
        let held = view.story.texts.get(ends).is_some_and(|texts| texts.iter().any(|toks| self.texts.whole.contains_key(toks)));
        if held {
            return Some(vec![ends.clone()]);
        }
        ThingWords::way(&view.story, ends, self.scored, &mut |pair| {
            let (got, how) = self.readings(view, pair);
            (got.first().map(|(_, score)| *score).unwrap_or_default(), how == TextReading::Lessons)
        })
    }

    fn joined(&self, read: &[Readings]) -> Option<(String, usize)> {
        let most = if read.len() <= WIDE_WAY { READINGS } else { NARROW_READINGS };
        let mut sets: Vec<(Vec<usize>, f64)> = vec![(Vec::new(), f64::default())];
        for got in read {
            let width = got.len().min(most);
            sets = sets.into_iter().flat_map(|(chosen, _)| (0..width).map(move |pick| {
                let mut more = chosen.clone();
                more.push(pick);
                (more, f64::default())
            })).collect();
        }
        for (chosen, sum) in sets.iter_mut() {
            *sum = compensated(chosen.iter().zip(read).map(|(pick, got)| got[*pick].1));
        }
        sets.sort_by(|left, right| right.1.partial_cmp(&left.1).unwrap_or(std::cmp::Ordering::Equal));
        let join = self.join.as_ref()?;
        for lent in [false, true] {
            for (number, (chosen, _)) in sets.iter().take(MOST_READING_SETS).enumerate() {
                let labels: Vec<String> = chosen.iter().zip(read).map(|(pick, got)| got[*pick].0.clone()).collect();
                if let Some(got) = join.joined(&labels, lent).filter(|got| !got.is_empty()) {
                    return Some((got, number + 1));
                }
            }
        }
        None
    }
}

impl Learner for FramesModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let Some(ends) = self.things.ends(sentences, question) else { return self.said(FRAMES_NOT_TWO.to_string()) };
        let view = self.view(sentences, &ends);
        let Some(way) = self.way_of(&view, &ends).filter(|way| !way.is_empty()) else { return self.said(FRAMES_NO_WAY.to_string()) };
        let read: Vec<(Readings, TextReading)> = way.iter().map(|pair| self.readings(&view, pair)).collect();
        let new = read.iter().filter(|(_, how)| *how != TextReading::Lessons).count();
        if let [(got, _)] = read.as_slice() {
            let Some((label, _)) = got.first() else { return self.said(FRAMES_NO_WAY.to_string()) };
            return Said { words: label.split_whitespace().map(str::to_string).collect(), note: format!("{FRAMES_ONE_LINK}, new {new}") };
        }
        let labels: Vec<Readings> = read.into_iter().map(|(got, _)| got).collect();
        match self.joined(&labels) {
            Some((got, number)) => Said { words: got.split_whitespace().map(str::to_string).collect(), note: format!("{FRAMES_JOINED}: links {}, new {new}, set {number}", way.len()) },
            None => self.said(format!("{FRAMES_NO_JOIN}: links {}, new {new}", way.len())),
        }
    }
}

pub(super) fn learn_frames(lessons: Lessons, _lanes: usize) -> Learned {
    Ok(Box::new(FramesModel::learn(lessons.cases, lessons.labels)?))
}
because!(learn_frames, FramesTool, "the reader of told facts as a tool of the one call: it reads a kind only when some lesson story has labels that make a chain of steps");

pub const FRAMES_TOOL: Tool = Tool { name: "frames", needs_labels: true, probe: false, asks: false, over: None, learn: learn_frames };
because!(FRAMES_TOOL, FramesTool, "the reader of told facts in the list of the tools: it learns from labels of the lesson stories, it has no rule by the form of the question alone, and it asks for no answer");

pub struct InnerFrames;
source!(InnerFrames, "the design check of the reader on its lessons alone: every third lesson is read by a model of the other lessons, in three parts, so that each lesson is read once by a model that did not learn from it; for each step of a read lesson whose link has a text that the model's lessons never held, the first reading and the kept readings are compared with the label of the step, and the answers of the stories with their answers");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FramesCheck {
    pub steps: usize,
    pub first_right: usize,
    pub among: usize,
    pub stories: usize,
    pub stories_right: usize,
    pub chains: usize,
    pub way_right: usize,
    pub read_right: usize,
    pub join_right: usize,
    pub by_way: Vec<(String, usize, usize)>,
    pub wrong: Vec<String>,
    pub right: Vec<(usize, bool)>,
}
because!(FramesCheck, InnerFrames, "the counts of the design check: the steps with a new text, those whose first reading is right, those whose label is among the kept readings, the stories and those answered right, for each way of reading its steps and its right ones, and the wrong steps written out for a person to read");

const INNER_PARTS: usize = 3;
because!(INNER_PARTS, InnerFrames, "the parts of the lessons: each part is read by a model of the others");

pub fn frames_check(cases: &[Case], labels: &[Vec<String>], lanes: usize) -> Result<FramesCheck, String> {
    let mut out = FramesCheck::default();
    for phase in 0..INNER_PARTS {
        let learn_places: Vec<usize> = (0..cases.len()).filter(|place| place % INNER_PARTS != phase).collect();
        let read_places: Vec<usize> = (0..cases.len()).filter(|place| place % INNER_PARTS == phase).collect();
        check_phase(cases, labels, &learn_places, &read_places, lanes, &mut out)?;
    }
    out.by_way.sort();
    out.right.sort();
    Ok(out)
}
because!(frames_check, InnerFrames, "the design check of the reader on some lessons with their label lines, on a given number of threads; the counts do not depend on the number of threads");

pub fn frames_check_split(cases: &[Case], labels: &[Vec<String>], read: &[bool], lanes: usize) -> Result<FramesCheck, String> {
    let learn_places: Vec<usize> = (0..cases.len()).filter(|place| !read.get(*place).copied().unwrap_or(false)).collect();
    let read_places: Vec<usize> = (0..cases.len()).filter(|place| read.get(*place).copied().unwrap_or(false)).collect();
    let mut out = FramesCheck::default();
    check_phase(cases, labels, &learn_places, &read_places, lanes, &mut out)?;
    out.by_way.sort();
    out.right.sort();
    Ok(out)
}
because!(frames_check_split, InnerFrames, "the design check with the lessons to read named by the caller, for a split that the three parts do not give, as lessons whose sentences have forms that no lesson of the model holds");

fn check_phase(cases: &[Case], labels: &[Vec<String>], learn_places: &[usize], read_places: &[usize], lanes: usize, out: &mut FramesCheck) -> Result<(), String> {
    {
        let learn_cases: Vec<Case> = learn_places.iter().map(|place| cases[*place].clone()).collect();
        let learn_labels: Vec<Vec<String>> = learn_places.iter().map(|place| labels.get(*place).cloned().unwrap_or_default()).collect();
        let model = FramesModel::learn(&learn_cases, &learn_labels)?;
        let runs = shards(read_places.len(), lanes);
        let found = on_lanes(runs.len(), lanes, &|run| {
            let mut part = FramesCheck::default();
            for place in &read_places[runs[run].clone()] {
                let case = &cases[*place];
                part.stories += 1;
                let right = model.answer(&case.sentences, &case.question).words == answer_words(&case.answer);
                part.stories_right += usize::from(right);
                part.right.push((*place, right));
                let Some(ends) = model.things.ends(&case.sentences, &case.question) else { continue };
                let Some(steps) = labels.get(*place).and_then(|lines| steps::label_steps(lines)) else { continue };
                let view = model.view(&case.sentences, &ends);
                if let Some(chain) = labels.get(*place).and_then(|lines| chain_of(lines, &ends)) {
                    part.chains += 1;
                    let pairs: Vec<ThingPair> = chain.iter().map(|(first, _, second)| (first.clone(), second.clone())).collect();
                    let said: Vec<String> = chain.iter().map(|(_, label, _)| label.clone()).collect();
                    let answer = answer_words(&case.answer).join(STEP_BETWEEN);
                    let joined = if let [one] = said.as_slice() { Some(one.clone()) } else { model.join.as_ref().and_then(|join| join.joined(&said, true)) };
                    part.join_right += usize::from(joined.as_ref() == Some(&answer));
                    let found_way = model.way_of(&view, &ends);
                    if found_way.as_ref() != Some(&pairs) {
                        part.wrong.push(format!("Way: {} | true {pairs:?} | found {found_way:?}", case.sentences.join(" / ")));
                    }
                    if found_way.as_ref() == Some(&pairs) {
                        part.way_right += 1;
                        let all = pairs.iter().zip(&said).all(|(pair, label)| model.readings(&view, pair).0.first().is_some_and(|(held, _)| held == label));
                        part.read_right += usize::from(all);
                    }
                }
                for (first, label, second) in &steps {
                    let pair = (first.clone(), second.clone());
                    let Some(texts) = view.story.texts.get(&pair) else { continue };
                    if texts.is_empty() || texts.iter().any(|toks| model.texts.whole.contains_key(toks)) {
                        continue;
                    }
                    let (got, how) = model.readings(&view, &pair);
                    let right = got.first().is_some_and(|(held, _)| held == label);
                    part.steps += 1;
                    part.first_right += usize::from(right);
                    if !right {
                        let shown: Vec<String> = texts.iter().map(|toks| toks.join(STEP_BETWEEN)).collect();
                        let facts = if how == TextReading::Fact { model.facts_said(&view, &pair).join("; ") } else { String::new() };
                        part.wrong.push(format!("{how:?}: {} | step {first} {label} {second} | read {:?} | facts {facts}", shown.join(" / "), got.iter().map(|(held, _)| held.as_str()).collect::<Vec<&str>>()));
                    }
                    part.among += usize::from(got.iter().any(|(held, _)| held == label));
                    let name = format!("{how:?}");
                    match part.by_way.iter_mut().find(|(held, _, _)| *held == name) {
                        Some(row) => {
                            row.1 += 1;
                            row.2 += usize::from(right);
                        }
                        None => part.by_way.push((name, 1, usize::from(right))),
                    }
                }
            }
            part
        });
        for part in found {
            out.steps += part.steps;
            out.first_right += part.first_right;
            out.among += part.among;
            out.stories += part.stories;
            out.stories_right += part.stories_right;
            out.chains += part.chains;
            out.way_right += part.way_right;
            out.read_right += part.read_right;
            out.join_right += part.join_right;
            out.right.extend(part.right);
            out.wrong.extend(part.wrong);
            for (name, all, right) in part.by_way {
                match out.by_way.iter_mut().find(|(held, _, _)| *held == name) {
                    Some(row) => {
                        row.1 += all;
                        row.2 += right;
                    }
                    None => out.by_way.push((name, all, right)),
                }
            }
        }
    }
    Ok(())
}
because!(check_phase, InnerFrames, "one part of the design check: a model of the lessons at some places reads the lessons at the other places, and its counts are added to the counts so far");
