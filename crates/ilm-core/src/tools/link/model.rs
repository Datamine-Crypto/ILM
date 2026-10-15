use crate::base::first_sight::{FirstSightCount, NumberTable};
use crate::base::keys::NO_NUMBER;
use crate::base::lanes::on_lanes;
use crate::base::symbols::Symbols;
use crate::base::words::{answer_words, words as text_words};
use crate::cases::Case;
use crate::tools::call::{Learned, Learner, Lessons, Said};
use crate::tools::frequent::frequent_answer;
use crate::tools::link::joins::{Joins, LINK_LEAST, LINK_SURE_SHARE, LinkLessons, link_order};
use crate::tools::link::pick::{LinkPicker, PICK_SURE_LESSONS, PickItem, PickLesson, learn_pick};
use crate::tools::link::read::{LINK_ENDS, LINK_LABEL, LinkRead, LinkWords, link_search};
use crate::tools::link::trials::LinkTrials;
use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::kinds::PatternWords;
use crate::tools::pattern::shared_words::fresh_words;
use crate::tools::pattern::names::{Fresh, WordTexts};
use crate::tools::pattern::story::{Desk, Query};
use patterns::{because, source};
use std::cmp::Ordering;

pub struct LinkSame;
source!(LinkSame, "the label of a link and the label of an answer can be one label, as the link that says a daughter says what the answer daughter says. For each label of a link each answer is tried: the two are counted as one label and step zero is counted again; the answer with which more lessons come out right, and nearly no lesson is lost, is taken if no other answer gains as much. In a round only the label with the largest gain is made one with its answer, and all is counted again, because before one label is settled nothing speaks against a wrong pairing of another. When nothing can be gained, a label and an answer are one label if no lesson is lost and they give the same joins with the same partners");

const LINK_SHARED: usize = 3;
because!(LINK_SHARED, LinkSame, "two labels are one label when nothing is gained, from this many shared joins with the same answers on: fewer can be chance");

const LINK_ROUNDS: usize = 40;
because!(LINK_ROUNDS, LinkSame, "the search for labels that are one label ends after this many rounds: a set has fewer labels to settle");

const LINK_ANSWER: &str = "A:";
because!(LINK_ANSWER, LinkSame, "the start of the text of an answer as a label, so that an answer is never taken for the label of a link");

const LINK_BETWEEN_WORDS: &str = " ";
because!(LINK_BETWEEN_WORDS, LinkSame, "the mark between the words of an answer in its text as a label");

const LINK_PICKED: usize = 3;
because!(LINK_PICKED, LinkSame, "the least things of a question that holds a choice among things: one fixed thing and more than one thing to pick from");

const NOTE_NOT_TWO: &str = "not two things in the question";
because!(NOTE_NOT_TWO, LinkSame, "the note of an answer to a question that does not name two things");

const NOTE_NO_WAY: &str = "no way between the two things";
because!(NOTE_NO_WAY, LinkSame, "the note of an answer to a question whose things no chain of links joins: the answer of the lessons with no way when they have a sure one, and else no answer, because with no way the story gives no answer and a guess would read nothing");

const NOTE_ONE_LINK: &str = "one link";
because!(NOTE_ONE_LINK, LinkSame, "the note of an answer that the label of the only link of the way gives, read as an answer");

const NOTE_JOINED: &str = "links joined";
because!(NOTE_JOINED, LinkSame, "the note of an answer that a join of links gives");

const NOTE_NO_JOIN: &str = "links with no join";
because!(NOTE_NO_JOIN, LinkSame, "the note of an answer to a question whose links the table does not join to one answer");

const NOTE_PICKED: &str = "the thing of the question that the search reaches";
because!(NOTE_PICKED, LinkSame, "the note of an answer that is picked among the things of the question by a search");

const NOTE_NOT_PICKED: &str = "the search reaches no single thing of the question";
because!(NOTE_NOT_PICKED, LinkSame, "the note of an answer to a choice where the search reaches no thing or more than one");

#[derive(Debug)]
pub(super) struct LinkModel {
    symbols: Symbols,
    words: PatternWords,
    link_words: LinkWords,
    items: Symbols,
    lessons: LinkLessons,
    default: Vec<String>,
    apart: Option<Vec<String>>,
    same: NumberTable<u32, u32>,
    joins: Joins,
    pick: Option<LinkPicker>,
}
because!(LinkModel, LinkSame, "what the tool learned: the words and kinds of the lessons, the thing kinds and the forms of the lesson sentences, the texts of the labels and answers, the lessons with a way, the most frequent answer, the answer of a question whose things no way joins, the labels that are one label with an answer, the join table, and the form of a choice among things");

fn link_way(read: Option<LinkRead>) -> Option<Vec<u32>> {
    let read = read?;
    if read.ends.len() != LINK_ENDS {
        return None;
    }
    if !read.ends.iter().all(|end| read.told.contains(end)) {
        return Some(Vec::new());
    }
    Some(link_search(&read.near, read.ends[0], read.ends[1], &[]).unwrap_or_default())
}
because!(link_way, LinkSame, "the labels of the way from the first thing of the question to the second; no labels when no way leads from one to the other or a thing of the question is not in the story; nothing when the question does not name two things");

impl LinkModel {
    fn same_of(&self, same: &NumberTable<u32, u32>, label: u32) -> u32 {
        same.get(&label).copied().unwrap_or(label)
    }

    fn settle(&mut self, lanes: usize) {
        let label_of = |same: &NumberTable<u32, u32>, label: u32| same.get(&label).copied().unwrap_or(label);
        let (first, _) = self.lessons.count(&|label| label, false, &self.items);
        for (leaf, answer) in first.single.rows() {
            if self.items.name(*leaf).starts_with(LINK_LABEL) {
                self.same.insert(*leaf, *answer);
            }
        }
        for _ in 0..LINK_ROUNDS {
            let same = self.same.clone();
            let (joins, right) = self.lessons.count(&|label| label_of(&same, label), false, &self.items);
            let mut leaves: Vec<u32> = Vec::new();
            for (labels, _) in &self.lessons.lessons {
                for label in labels {
                    let label = label_of(&same, *label);
                    if !self.lessons.results.contains(&label) && !leaves.contains(&label) {
                        leaves.push(label);
                    }
                }
            }
            leaves.sort_by(|left, right| self.items.name(*left).cmp(self.items.name(*right)));
            let rows = self.lessons.rows(&|label| label_of(&same, label), &joins, &self.items);
            let pairs: Vec<(u32, u32)> = leaves.iter().flat_map(|leaf| self.lessons.results.iter().map(move |result| (*leaf, *result))).collect();
            let trials = LinkTrials::over(&self.lessons, &|label| label_of(&same, label), &joins, &right, &self.items);
            let tried: Vec<(usize, usize)> = on_lanes(pairs.len(), lanes, &|place| {
                let (leaf, result) = pairs[place];
                trials.trial(leaf, result)
            });
            let mut found: Vec<(f64, u32, u32)> = Vec::new();
            for (at, leaf) in leaves.iter().enumerate() {
                let mut best: Vec<(f64, u32)> = Vec::new();
                for (place, result) in self.lessons.results.iter().enumerate() {
                    let (gain, lost) = tried[at * self.lessons.results.len() + place];
                    if gain >= LINK_LEAST as usize && (gain as f64) >= LINK_SURE_SHARE * ((gain + lost) as f64) {
                        best.push((gain as f64 - lost as f64, *result));
                    } else if lost == 0 {
                        let (Some(mine), Some(theirs)) = (rows.get(leaf), rows.get(result)) else { continue };
                        let shared: Vec<&(bool, u32)> = mine.rows().iter().map(|(key, _)| key).filter(|key| theirs.holds(key)).collect();
                        if shared.len() >= LINK_SHARED && shared.iter().all(|key| mine.get(key) == theirs.get(key)) {
                            best.push((shared.len() as f64 / (shared.len() as f64 + f64::from(1u8)), *result));
                        }
                    }
                }
                best.sort_by(|left, right| right.0.partial_cmp(&left.0).unwrap_or(Ordering::Equal).then_with(|| self.items.name(right.1).cmp(self.items.name(left.1))));
                let clear = match best.as_slice() {
                    [] => false,
                    [_] => true,
                    [first, second, ..] => first.0 > second.0,
                };
                if clear {
                    found.push((best[0].0, *leaf, best[0].1));
                }
            }
            let mut top: Option<(f64, u32, u32)> = None;
            for candidate in &found {
                if top.is_none_or(|held| link_order(candidate, &held, &self.items) == Ordering::Greater) {
                    top = Some(*candidate);
                }
            }
            let Some((_, leaf, result)) = top else { break };
            self.same.insert(leaf, result);
        }
        let same = self.same.clone();
        self.joins = self.lessons.count(&|label| label_of(&same, label), true, &self.items).0;
    }

    fn from_lessons(cases: &[Case], lanes: usize) -> Learned {
        let (mut symbols, words) = fresh_words(cases);
        let mut items = Symbols::new();
        let mut often: FirstSightCount<u32> = FirstSightCount::new();
        let answers: Vec<u32> = cases.iter().map(|case| items.number(&format!("{LINK_ANSWER}{}", answer_words(&case.answer).join(LINK_BETWEEN_WORDS)))).collect();
        for answer in &answers {
            often.add(*answer, 1);
        }
        let mut results: Vec<u32> = often.rows().iter().map(|(answer, _)| *answer).collect();
        results.sort_by(|left, right| items.name(*left).cmp(items.name(*right)));
        let queries: Vec<Query> = {
            let mut desk = Desk::over(&words, &mut symbols);
            cases.iter().map(|case| desk.case(&case.sentences, &case.question, &|_| 0)).collect()
        };
        let mut link_words = LinkWords::default();
        for query in &queries {
            for word in &query.slots {
                let kind = words.kind_with(*word, &query.story.extra);
                if kind != ABSENT && !link_words.thing_kinds.contains(&kind) {
                    link_words.thing_kinds.push(kind);
                }
            }
        }
        for query in &queries {
            for told in &query.story.sents {
                let sent = LinkWords::apart(&mut symbols, &told.words);
                link_words.known.extend(sent.iter().copied());
                let form: Vec<u32> = sent.iter().map(|word| link_words.mark(LinkWords::kind_of(&words, query, &[], *word), *word)).collect();
                link_words.forms.entry(sent.len()).or_default().add(form, 1);
            }
        }
        let mut model = LinkModel { symbols, words, link_words, items, lessons: LinkLessons::default(), default: frequent_answer(cases).0, apart: None, same: NumberTable::default(), joins: Joins::default(), pick: None };
        model.lessons.results = results;
        for (answer, count) in often.rows() {
            model.lessons.often.insert(*answer, *count as u32);
        }
        let mut no_way: FirstSightCount<Vec<String>> = FirstSightCount::new();
        let mut picks: Vec<PickLesson> = Vec::new();
        for ((case, query), answer) in cases.iter().zip(&queries).zip(&answers) {
            let items = &mut model.items;
            let read = model.link_words.read(&model.words, &mut model.symbols, query, &mut |text| items.number(text));
            if let Some(read) = read.as_ref().filter(|read| read.ends.len() >= LINK_PICKED) {
                let raw = answer_words(&case.answer);
                let joined = model.words.read(&mut model.symbols, &text_words(&raw.join(LINK_BETWEEN_WORDS)));
                let said = LinkWords::apart(&mut model.symbols, &joined);
                if said.len() == raw.len() {
                    let places: Vec<PickItem> = said.iter().zip(&raw).map(|(word, text)| read.ends.iter().position(|end| end == word).map_or_else(|| PickItem::Word(text.clone()), PickItem::Thing)).collect();
                    picks.push((read.ends.clone(), read.near.clone(), places));
                }
            }
            match link_way(read) {
                Some(labels) if !labels.is_empty() => model.lessons.lessons.push((labels, *answer)),
                Some(_) => no_way.add(answer_words(&case.answer), 1),
                None => {}
            }
        }
        let total: i64 = no_way.rows().iter().map(|(_, count)| *count).sum();
        if let Some((answer, hit)) = no_way.first_most()
            && total as usize >= PICK_SURE_LESSONS
            && (*hit as f64) >= LINK_SURE_SHARE * total as f64
        {
            model.apart = Some(answer.clone());
        }
        model.settle(lanes);
        model.pick = learn_pick(picks, &model.items);
        Ok(Box::new(model))
    }

    fn picked<N: WordTexts>(&self, names: &mut N, read: Option<&LinkRead>, sentences: &[String], question: &str) -> Option<Said> {
        let read = read.filter(|read| read.ends.len() >= LINK_PICKED)?;
        let pick = self.pick.as_ref()?;
        if pick.start >= read.ends.len() {
            return None;
        }
        let reached = pick.reached(&read.ends, &read.near, &pick.shut);
        let [one] = reached.as_slice() else { return Some(Said { words: self.default.clone(), note: NOTE_NOT_PICKED.to_string() }) };
        let got: Vec<u32> = pick
            .form
            .iter()
            .map(|item| match item {
                None => read.ends[*one],
                Some(PickItem::Thing(at)) => read.ends[*at],
                Some(PickItem::Word(text)) => names.number(text),
            })
            .collect();
        Some(Said { words: self.words.raw_words(names, &got, sentences, question, &[]), note: NOTE_PICKED.to_string() })
    }

    fn answer_text(&self, item: u32) -> Vec<String> {
        self.items.name(item).strip_prefix(LINK_ANSWER).unwrap_or_default().split_whitespace().map(str::to_string).collect()
    }
}

impl Learner for LinkModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let mut fresh = Fresh::over(&self.symbols);
        let query = Desk::over(&self.words, &mut fresh).case(sentences, question, &|_| 0);
        let read = self.link_words.read(&self.words, &mut fresh, &query, &mut |text| self.items.find(text).unwrap_or(NO_NUMBER));
        if let Some(said) = self.picked(&mut fresh, read.as_ref(), sentences, question) {
            return said;
        }
        let Some(labels) = link_way(read) else { return Said { words: self.default.clone(), note: NOTE_NOT_TWO.to_string() } };
        if labels.is_empty() {
            return Said { words: self.apart.clone().unwrap_or_default(), note: NOTE_NO_WAY.to_string() };
        }
        let labels: Vec<u32> = labels.iter().map(|label| self.same_of(&self.same, *label)).collect();
        if let [one] = labels.as_slice() {
            let got = if self.items.names().get(*one as usize).is_some_and(|text| text.starts_with(LINK_ANSWER)) { Some(*one) } else { self.joins.single.get(one).copied() };
            return match got {
                Some(answer) => Said { words: self.answer_text(answer), note: NOTE_ONE_LINK.to_string() },
                None => Said { words: self.default.clone(), note: NOTE_NO_JOIN.to_string() },
            };
        }
        let span = self.lessons.chart(&labels, &self.joins);
        match self.lessons.best(&span[labels.len()], &self.items) {
            Some(answer) => Said { words: self.answer_text(answer), note: NOTE_JOINED.to_string() },
            None => Said { words: self.default.clone(), note: NOTE_NO_JOIN.to_string() },
        }
    }
}

pub(super) fn learn_link(lessons: Lessons, lanes: usize) -> Learned {
    LinkModel::from_lessons(lessons.cases, lanes)
}
because!(learn_link, LinkSame, "the link tool as a tool of the one call: it reads every kind, since a kind with no two things in its questions gets the most frequent answer");
