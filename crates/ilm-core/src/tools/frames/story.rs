use crate::base::first_sight::{FirstSightCount, FirstSightTable};
use crate::base::words::words;
use crate::cases::Case;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct StoryLinks;
source!(StoryLinks, "the things of a story and the links between them. A thing is a word that a question shares with its story and that few questions hold; when the things of the lessons stand between the same two signs nearly always, every word between those signs is a thing, a new one too, and the two signs are taken out; the things of a question are then its words that stand between the signs in its story, however many questions hold them. A sentence with two things links each pair of them; two things that no sentence holds together are linked when they stand in two sentences side by side. The text of a link is its words with S for the first thing, T for the second and N for any other thing. The way from one thing of the question to the other is the shortest chain of links, or the chain with the best sum of the scores of its links");

const THING_OFTEN: f64 = 0.2;
because!(THING_OFTEN, StoryLinks, "the share of the questions that a word stands in, from which on it is no thing but a word of every question");

pub(super) const FRAMES_SURE: f64 = 0.98;
because!(FRAMES_SURE, StoryLinks, "the share of the places of the things that must stand between the same two signs before the signs mark every thing, and the share of the labels of a name that must be of one group before the name has the group");

const WRAP_LESSONS: usize = 3000;
because!(WRAP_LESSONS, StoryLinks, "the lessons whose sentences show the signs around the things: more change nothing");

pub(super) const MARK_FIRST: &str = "S";
because!(MARK_FIRST, StoryLinks, "the mark of the first thing of a link in its text");

pub(super) const MARK_SECOND: &str = "T";
because!(MARK_SECOND, StoryLinks, "the mark of the second thing of a link in its text");

const MARK_OTHER: &str = "N";
because!(MARK_OTHER, StoryLinks, "the mark of any other thing in the text of a link");

const MOST_THINGS: usize = 13;
because!(MOST_THINGS, StoryLinks, "the most things of a story for which the way with the best scores is searched over every chain; with more the shortest way is taken");

pub(super) const LINK_BONUS: f64 = 3.0;
because!(LINK_BONUS, StoryLinks, "what each link adds to the score of a way, so that a way through one more thing is better");

pub(super) const NEW_SPAN: f64 = -10.0;
because!(NEW_SPAN, StoryLinks, "the score of a link over two sentences whose text is new: such a link is seldom one");

pub(super) type Toks = Vec<String>;
because!(Toks, StoryLinks, "the text of a link: the words of its sentences with the marks of its things");

pub(super) type ThingPair = (String, String);
because!(ThingPair, StoryLinks, "a link from a first thing to a second thing");

#[derive(Clone, Debug, Default)]
pub(super) struct ReadSentence {
    pub(super) words: Vec<String>,
    pub(super) places: Vec<usize>,
}
because!(ReadSentence, StoryLinks, "a sentence with the signs around its things taken out, and the places of its things");

#[derive(Clone, Debug, Default)]
pub(super) struct StoryRead {
    pub(super) read: Vec<ReadSentence>,
    pub(super) texts: FirstSightTable<ThingPair, Vec<Toks>>,
    pub(super) spans: HashMap<ThingPair, Vec<(usize, usize)>>,
    pub(super) things: Vec<String>,
    pub(super) together: HashSet<ThingPair>,
}
because!(StoryRead, StoryLinks, "a story as the links read it: its sentences, the texts of each link in the order of first sight with the first and the last sentence of each text, the things in their order, and the links of things that one sentence holds together");

#[derive(Clone, Debug, Default)]
pub(super) struct ThingWords {
    pub(super) common: HashSet<String>,
    pub(super) things: HashSet<String>,
    pub(super) wrap: Option<(String, String)>,
}
because!(ThingWords, StoryLinks, "the words of every question, the thing words, and the signs that stand around the things. counted takes them from the lessons; unwrapped reads a sentence; ends gives the two things of a question; links reads a story; way gives the way between the two things");

const WRAP_CLOSE: usize = 2;
because!(WRAP_CLOSE, StoryLinks, "the place of the closing sign of a wrapped thing after its opening sign: the name of the thing stands alone between them");

fn story_words(sentences: &[String]) -> Vec<Vec<String>> {
    sentences.iter().map(|sentence| words(sentence)).collect()
}
because!(story_words, StoryLinks, "each told sentence cut into the words that the measure compares, so that the frames count the same words as the answers");

impl ThingWords {
    pub(super) fn counted(cases: &[Case]) -> ThingWords {
        let mut in_questions: HashMap<String, usize> = HashMap::new();
        let mut shared: HashSet<String> = HashSet::new();
        for case in cases {
            let asked: HashSet<String> = words(&case.question).into_iter().collect();
            let told: HashSet<String> = story_words(&case.sentences).into_iter().flatten().collect();
            for word in &asked {
                *in_questions.entry(word.clone()).or_default() += 1;
            }
            shared.extend(asked.into_iter().filter(|word| told.contains(word)));
        }
        let common: HashSet<String> = in_questions.into_iter().filter(|(_, count)| *count as f64 >= THING_OFTEN * cases.len() as f64).map(|(word, _)| word).collect();
        let things: HashSet<String> = shared.into_iter().filter(|word| !common.contains(word)).collect();
        let mut around: FirstSightCount<(String, String)> = FirstSightCount::new();
        for case in cases.iter().take(WRAP_LESSONS) {
            for sentence in story_words(&case.sentences) {
                for (at, word) in sentence.iter().enumerate() {
                    if things.contains(word) {
                        let before = if at > 0 { sentence[at - 1].clone() } else { String::new() };
                        let after = sentence.get(at + 1).cloned().unwrap_or_default();
                        around.add((before, after), 1);
                    }
                }
            }
        }
        let total: i64 = around.rows().iter().map(|(_, count)| *count).sum();
        let wrap = around.first_most().filter(|((before, after), hit)| !before.is_empty() && !after.is_empty() && *hit as f64 >= FRAMES_SURE * total as f64).map(|(pair, _)| pair.clone());
        ThingWords { common, things, wrap }
    }

    pub(super) fn unwrapped(&self, sentence: &[String], ends: &HashSet<String>) -> ReadSentence {
        let mut out = ReadSentence::default();
        let mut at = 0;
        while at < sentence.len() {
            if let Some((open, close)) = &self.wrap
                && at + WRAP_CLOSE < sentence.len()
                && sentence[at] == *open
                && sentence[at + WRAP_CLOSE] == *close
            {
                out.places.push(out.words.len());
                out.words.push(sentence[at + 1].clone());
                at += LINK_WRAP_STEP;
                continue;
            }
            if self.wrap.is_none() && (self.things.contains(&sentence[at]) || ends.contains(&sentence[at])) {
                out.places.push(out.words.len());
            }
            out.words.push(sentence[at].clone());
            at += 1;
        }
        out
    }

    pub(super) fn ends(&self, sentences: &[String], question: &str) -> Option<ThingPair> {
        let told: Vec<Vec<String>> = story_words(sentences);
        let things: HashSet<String> = match self.wrap {
            Some(_) => told
                .iter()
                .flat_map(|sentence| {
                    let read = self.unwrapped(sentence, &HashSet::new());
                    read.places.iter().map(|place| read.words[*place].clone()).collect::<Vec<String>>()
                })
                .collect(),
            None => told.into_iter().flatten().filter(|word| !self.common.contains(word)).collect(),
        };
        let mut found: Vec<String> = Vec::new();
        for word in words(question) {
            if things.contains(&word) && !found.contains(&word) {
                found.push(word);
            }
        }
        match found.as_slice() {
            [first, second] => Some((first.clone(), second.clone())),
            _ => None,
        }
    }

    pub(super) fn links(&self, sentences: &[String], ends: &ThingPair) -> StoryRead {
        let asked: HashSet<String> = [ends.0.clone(), ends.1.clone()].into_iter().collect();
        let read: Vec<ReadSentence> = story_words(sentences).iter().map(|sentence| self.unwrapped(sentence, &asked)).collect();
        let things: Vec<Vec<String>> = read
            .iter()
            .map(|sentence| {
                let mut names: Vec<String> = Vec::new();
                for place in &sentence.places {
                    if !names.contains(&sentence.words[*place]) {
                        names.push(sentence.words[*place].clone());
                    }
                }
                names
            })
            .collect();
        let thing_words: HashSet<String> = things.iter().flatten().cloned().collect();
        let mut story = StoryRead { read, ..StoryRead::default() };
        let link = |story: &mut StoryRead, first: usize, last: usize, pairs: &[ThingPair]| {
            let text: Vec<String> = (first..=last).flat_map(|at| story.read[at].words.clone()).collect();
            for (one, other) in pairs {
                if one == other {
                    continue;
                }
                let toks: Toks = text
                    .iter()
                    .map(|word| {
                        if word == one {
                            MARK_FIRST.to_string()
                        } else if word == other {
                            MARK_SECOND.to_string()
                        } else if thing_words.contains(word) {
                            MARK_OTHER.to_string()
                        } else {
                            word.clone()
                        }
                    })
                    .collect();
                let pair = (one.clone(), other.clone());
                let held = story.texts.at(pair.clone());
                if !held.contains(&toks) {
                    held.push(toks);
                    story.spans.entry(pair).or_default().push((first, last));
                }
            }
        };
        for (at, names) in things.iter().enumerate() {
            if names.len() >= THING_ENDS {
                let pairs: Vec<ThingPair> = names.iter().flat_map(|one| names.iter().map(move |other| (one.clone(), other.clone()))).collect();
                link(&mut story, at, at, &pairs);
            }
        }
        story.together = story.texts.rows().iter().map(|(pair, _)| pair.clone()).collect();
        let held: Vec<usize> = things.iter().enumerate().filter(|(_, names)| !names.is_empty()).map(|(at, _)| at).collect();
        for side in held.windows(THING_ENDS) {
            let (first, last) = (side[0], side[1]);
            let mut pairs: Vec<ThingPair> = Vec::new();
            for one in &things[first] {
                for other in &things[last] {
                    if one != other && !story.together.contains(&(one.clone(), other.clone())) {
                        pairs.push((one.clone(), other.clone()));
                    }
                }
            }
            let turned: Vec<ThingPair> = pairs.iter().map(|(one, other)| (other.clone(), one.clone())).collect();
            pairs.extend(turned);
            link(&mut story, first, last, &pairs);
        }
        for names in &things {
            for name in names {
                if !story.things.contains(name) {
                    story.things.push(name.clone());
                }
            }
        }
        story
    }

    pub(super) fn way(story: &StoryRead, ends: &ThingPair, scored: bool, link_score: &mut dyn FnMut(&ThingPair) -> (f64, bool)) -> Option<Vec<ThingPair>> {
        let (first, last) = ends;
        let mut near: FirstSightTable<String, Vec<String>> = FirstSightTable::new();
        for ((one, other), _) in story.texts.rows() {
            near.at(one.clone()).push(other.clone());
        }
        if !near.holds(first) {
            return None;
        }
        if scored && story.things.len() <= MOST_THINGS {
            let number: HashMap<&String, usize> = story.things.iter().enumerate().map(|(at, thing)| (thing, at)).collect();
            let mut score: HashMap<(usize, usize), f64> = HashMap::new();
            for (pair, _) in story.texts.rows() {
                let (best, lesson) = link_score(pair);
                let value = if !lesson && !story.together.contains(pair) { NEW_SPAN } else { best + LINK_BONUS };
                score.insert((number[&pair.0], number[&pair.1]), value);
            }
            let (start, goal) = (number[first], number[last]);
            let mut layer: FirstSightTable<(u64, usize), f64> = FirstSightTable::new();
            layer.put((1u64 << start, start), f64::default());
            let mut before: HashMap<(u64, usize), (u64, usize)> = HashMap::new();
            let mut best: Option<(f64, u64, usize)> = None;
            while !layer.rows().is_empty() {
                let mut later: FirstSightTable<(u64, usize), f64> = FirstSightTable::new();
                for ((mask, place), value) in layer.rows() {
                    if *place == goal && *mask != 1u64 << start && best.is_none_or(|top| way_better((*value, *mask, *place), top)) {
                        best = Some((*value, *mask, *place));
                    }
                }
                for ((mask, place), value) in layer.rows() {
                    if *place == goal {
                        continue;
                    }
                    for other in near.get(&story.things[*place]).cloned().unwrap_or_default() {
                        let next = number[&other];
                        if mask & (1u64 << next) != 0 {
                            continue;
                        }
                        let key = (mask | (1u64 << next), next);
                        let total = value + score[&(*place, next)];
                        if later.get(&key).is_none_or(|held| total > *held) {
                            later.put(key, total);
                            before.insert(key, (*mask, *place));
                        }
                    }
                }
                layer = later;
            }
            if let Some((_, mask, place)) = best {
                let mut out: Vec<ThingPair> = Vec::new();
                let mut key = (mask, place);
                while let Some(earlier) = before.get(&key) {
                    out.push((story.things[earlier.1].clone(), story.things[key.1].clone()));
                    key = *earlier;
                }
                out.reverse();
                return Some(out);
            }
        }
        let mut before: HashMap<String, Option<String>> = HashMap::new();
        before.insert(first.clone(), None);
        let mut queue: Vec<String> = vec![first.clone()];
        let mut at = 0;
        while at < queue.len() {
            if before.contains_key(last) {
                break;
            }
            let thing = queue[at].clone();
            at += 1;
            for other in near.get(&thing).cloned().unwrap_or_default() {
                if !before.contains_key(&other) {
                    before.insert(other.clone(), Some(thing.clone()));
                    queue.push(other);
                }
            }
        }
        if !before.contains_key(last) {
            return None;
        }
        let mut out: Vec<ThingPair> = Vec::new();
        let mut thing = last.clone();
        while let Some(Some(earlier)) = before.get(&thing).cloned() {
            out.push((earlier.clone(), thing));
            thing = earlier;
        }
        out.reverse();
        Some(out)
    }
}

const LINK_WRAP_STEP: usize = 3;
because!(LINK_WRAP_STEP, StoryLinks, "the words of a thing with its two signs, which a sentence passes at once");

pub(super) const THING_ENDS: usize = 2;
because!(THING_ENDS, StoryLinks, "the things that one link joins, one at each end, and the sentences side by side that a link over sentences joins");

fn way_better(found: (f64, u64, usize), held: (f64, u64, usize)) -> bool {
    found.0 > held.0 || (found.0 == held.0 && (found.1, found.2) > (held.1, held.2))
}
because!(way_better, StoryLinks, "whether an ended way is better than the best one so far: by its score, then by the things that it passes and its last thing, as tuples compare");
