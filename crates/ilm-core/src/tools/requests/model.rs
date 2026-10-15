use crate::base::first_sight::{FirstSightCount, FirstSightTable, NumberTable};
use crate::base::keys::KeyNumbers;
use crate::base::runs::RunNumbers;
use crate::base::symbols::Symbols;
use crate::base::words::{answer_words, words};
use crate::classes::WordClasses;
use crate::tools::call::{Learned, Lessons};
use crate::tools::frequent::frequent_answer;
use crate::tools::requests::frames::{FrameMarks, FrameWord, SLOT_NONE, SLOT_START, frame_joined, frame_near, frame_of};
use crate::tools::requests::shares::{KeyCounts, SHARE_SMOOTH, ShareTable};
use crate::tools::requests::tags::{TAG_OUTSIDE, is_later, request_located, request_placed, tag_count, tag_first, tag_later, tag_slot};
use crate::tools::requests::whole::{HeldSlots, LabelReader, LabelWay, MarkEdges, request_marks};
use patterns::{because, source};
use std::cmp::Ordering;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

pub struct RequestSlots;
source!(RequestSlots, "a reader of requests that people said or wrote. A lesson is a request, a question and an answer; a question whose answer is always a run of words of the request, or one fixed word for a request that has no such thing, asks for a slot, and any other question asks for a label of the whole request. Values group by the places that they share: two slots are of one class when half of the places of the smaller one, and a few words, hold words that fill both, and a word that fills slots of a class in half of its places or more is a word of that class; so a city seen once as the end of a trip is read after a word of departure as the other cities are read there");

const REQUEST_RARE: u32 = 1;
because!(REQUEST_RARE, RequestSlots, "a word that the lesson requests hold this often or less is a new word: it has no key of its own and is read by its frame");

const REQUEST_SHARED: f64 = 0.5;
because!(REQUEST_SHARED, RequestSlots, "two classes of slots are joined when at least this share of the places of the smaller one hold words that fill both");

const REQUEST_SHARED_WORDS: usize = 2;
because!(REQUEST_SHARED_WORDS, RequestSlots, "classes of slots are joined only when more than one word fills both: one word in more than one slot can be chance");

const REQUEST_HALF: f64 = 0.5;
because!(REQUEST_HALF, RequestSlots, "the share that is most: a word counts for the classes when it fills slots in at least this share of its places; a kind is no kind of requests when more than this share of its lessons have a question of their own, or of the lessons of a question an answer of their own");

const REQUEST_SURE_LESSONS: usize = 20;
because!(REQUEST_SURE_LESSONS, RequestSlots, "the least lessons, or lessons of one question, before the tool says that a kind is no kind of requests: fewer do not show it");

const REQUEST_CHECKED: usize = 500;
because!(REQUEST_CHECKED, RequestSlots, "of a long held part of the lessons of a label, this many are read to choose between the two ways of reading a label: enough to tell them apart, and the choice costs a reading of each");

const REQUEST_EARLY: usize = 2;
because!(REQUEST_EARLY, RequestSlots, "of the equal parts of the lessons of a label, this many are the first part, from which each way of reading a label is counted before it reads the rest");

const REQUEST_THIRDS: usize = 3;
because!(REQUEST_THIRDS, RequestSlots, "the equal parts in which the lessons of a label are counted to choose its way of reading");

const NO_ONE_SENTENCE: &str = "no requests: a case of this kind tells more or less than one sentence";
because!(NO_ONE_SENTENCE, RequestSlots, "what the tool says for a kind whose cases are no single requests");

const NO_SLOT_QUESTIONS: &str = "no requests: most lessons have a question of their own";
because!(NO_SLOT_QUESTIONS, RequestSlots, "what the tool says for a kind whose questions do not come again, so that no question can be a slot");

const NO_LABEL_ANSWERS: &str = "no requests: most lessons of a question have an answer of their own";
because!(NO_LABEL_ANSWERS, RequestSlots, "what the tool says for a kind where a question that is no slot has an answer of its own for most lessons, so that it is no label either");

const TEXT_QUOTE: &str = "'";
because!(TEXT_QUOTE, RequestSlots, "the mark around a word in the written form of a tag, by which tags of equal shares are put in order");

const TEXT_OPEN: &str = "(";
because!(TEXT_OPEN, RequestSlots, "the mark that opens the written form of a question in a tag");

const TEXT_CLOSE: &str = ")";
because!(TEXT_CLOSE, RequestSlots, "the mark that closes the written form of a question in a tag");

const TEXT_BETWEEN: &str = ", ";
because!(TEXT_BETWEEN, RequestSlots, "the mark between the words of the written form of a question in a tag");

const TEXT_ALONE: &str = ",";
because!(TEXT_ALONE, RequestSlots, "the mark after the one word of a written question of one word, so that it is a run and not a word");

const TEXT_NONE: &str = "None";
because!(TEXT_NONE, RequestSlots, "the written form of the tag of no slot");

const TEXT_SAME: &str = "('=', ";
because!(TEXT_SAME, RequestSlots, "the written form of the start of the tag of a later word of a run, before its question");

static REQUEST_MODELS: AtomicU64 = AtomicU64::new(0);
because!(REQUEST_MODELS, RequestSlots, "the models that this run of the program made: each model gets the next number, so that the tags that a thread keeps for the questions of one request are never taken for the tags of another model");

#[derive(Debug)]
pub(super) struct RequestModel {
    pub(super) id: u64,
    pub(super) symbols: Symbols,
    pub(super) marks: FrameMarks,
    pub(super) facts: Vec<FrameWord>,
    pub(super) classes: Option<WordClasses>,
    pub(super) class_names: Symbols,
    pub(super) questions: RunNumbers,
    pub(super) asked: Vec<String>,
    pub(super) slot_of: NumberTable<u32, u32>,
    pub(super) slot_questions: Vec<u32>,
    pub(super) label_of: NumberTable<u32, u32>,
    pub(super) label_questions: Vec<u32>,
    pub(super) readers: Vec<LabelReader>,
    pub(super) answers: RunNumbers,
    pub(super) none: Vec<String>,
    pub(super) default: Vec<String>,
    pub(super) again: Vec<Option<f64>>,
    pub(super) table: ShareTable,
    pub(super) ranks: Vec<u32>,
    pub(super) held: HeldSlots,
    pub(super) mark_table: KeyNumbers,
    pub(super) edges: MarkEdges,
}
because!(RequestModel, RequestSlots, "what the tool learned, as plain tables: the words with their counts, classes of values and classes in plain text; the questions with their texts, which of them ask for a slot and which for a label; the fixed word of a request with no such slot; the price of a second run of each slot; the counted keys with the share of each tag in all places; the place of each tag in the order of their written forms; which slots the lesson requests have, with their marks; and for each label its reader");

fn request_text(names: &[&str]) -> String {
    let mut text = String::from(TEXT_OPEN);
    let quoted: Vec<String> = names.iter().map(|name| format!("{TEXT_QUOTE}{name}{TEXT_QUOTE}")).collect();
    text.push_str(&quoted.join(TEXT_BETWEEN));
    if names.len() == 1 {
        text.push_str(TEXT_ALONE);
    }
    text.push_str(TEXT_CLOSE);
    text
}
because!(request_text, RequestSlots, "the written form of a question as a run of its words between quotes; tags of equal shares are put in order by these forms, so the same request is read the same way in every run");

fn request_count(counts: &mut Vec<u32>, at: usize) -> &mut u32 {
    if counts.len() <= at {
        counts.resize(at + 1, 0);
    }
    &mut counts[at]
}
because!(request_count, RequestSlots, "the count of one place of a flat array of counts, which grows to the place when a new word got its number");

pub(super) fn request_class(classes: Option<&WordClasses>, text: &str) -> Option<String> {
    let table = classes?;
    table.get(text).or_else(|| (!text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())).then(|| table.get(&u32::default().to_string())).flatten()).cloned()
}
because!(request_class, RequestSlots, "the class of a word in the table of word classes: the class of the word, or for a word of digits the class of the word nought, as the table writes every number, and nothing with no table or for a word that the table does not hold");

type RequestStories = FirstSightTable<Vec<u32>, FirstSightTable<u32, u32>>;
because!(RequestStories, RequestSlots, "the lesson requests in the order of first sight, each with its questions and their answers; a later lesson with the same request and question gives its answer in place of the earlier one");

impl RequestModel {
    pub(super) fn slot_code(tag: u32) -> u32 {
        tag_slot(tag).map_or(SLOT_NONE, |slot| SLOT_NONE + 1 + slot as u32)
    }

    pub(super) fn answer_order(&self, left: u32, right: u32) -> Ordering {
        let names = |answer: u32| self.answers.run(answer).iter().map(|word| self.symbols.name(*word)).collect::<Vec<&str>>();
        names(left).cmp(&names(right))
    }

    pub(super) fn name_order(&self, left: u32, right: u32) -> Ordering {
        self.symbols.name(left).cmp(self.symbols.name(right))
    }

    pub(super) fn answer_words(&self, answer: Option<u32>) -> Vec<String> {
        answer.map(|answer| self.answers.run(answer).iter().map(|word| self.symbols.name(*word).to_string()).collect()).unwrap_or_default()
    }

    fn fact(&self, word: u32, counts: &[u32], values: &[Option<u32>], classes: &[Option<u32>]) -> FrameWord {
        let count = counts.get(word as usize).copied().unwrap_or(0);
        let new = count <= REQUEST_RARE;
        let of_values = values.get(word as usize).copied().flatten();
        FrameWord {
            word,
            seen: if new { self.marks.new } else { word },
            hidden: if new || of_values.is_some() { self.marks.hidden } else { word },
            new,
            values: if new { None } else { of_values },
            class: classes.get(word as usize).copied().flatten(),
        }
    }

    fn grouped(fills: &FirstSightTable<u32, FirstSightCount<u32>>, counts: &[u32], texts: &[String], slots: usize) -> Vec<u32> {
        let usable: Vec<&FirstSightCount<u32>> = fills.rows().iter().filter(|(word, filled)| (filled.rows().iter().map(|(_, count)| *count).sum::<i64>() as f64) >= REQUEST_HALF * f64::from(counts[*word as usize])).map(|(_, filled)| filled).collect();
        let mut members: Vec<(u32, Vec<u32>)> = Vec::new();
        for filled in &usable {
            for (slot, _) in filled.rows() {
                if !members.iter().any(|(head, _)| head == slot) {
                    members.push((*slot, vec![*slot]));
                }
            }
        }
        loop {
            let have: Vec<Vec<(usize, i64)>> = members
                .iter()
                .map(|(_, held)| usable.iter().enumerate().filter_map(|(word, filled)| {
                    let count: i64 = held.iter().filter_map(|slot| filled.get(slot)).sum();
                    (count != 0).then_some((word, count))
                }).collect())
                .collect();
            let mut heads: Vec<usize> = (0..members.len()).collect();
            heads.sort_by(|left, right| texts[members[*left].0 as usize].cmp(&texts[members[*right].0 as usize]));
            let mut best: Option<(f64, usize, usize)> = None;
            for (place, one) in heads.iter().enumerate() {
                for other in &heads[place + 1..] {
                    let (mine, theirs) = (&have[*one], &have[*other]);
                    let small = mine.iter().map(|(_, count)| *count).sum::<i64>().min(theirs.iter().map(|(_, count)| *count).sum::<i64>());
                    let mut both = 0usize;
                    let mut common = 0i64;
                    let mut at = 0usize;
                    for (word, count) in mine {
                        while at < theirs.len() && theirs[at].0 < *word {
                            at += 1;
                        }
                        if at < theirs.len() && theirs[at].0 == *word {
                            both += 1;
                            common += (*count).min(theirs[at].1);
                        }
                    }
                    let share = common as f64 / (if small == 0 { 1 } else { small }) as f64;
                    if both >= REQUEST_SHARED_WORDS && share >= REQUEST_SHARED && best.is_none_or(|(top, _, _)| share > top) {
                        best = Some((share, *one, *other));
                    }
                }
            }
            let Some((_, one, other)) = best else { break };
            let (_, taken) = members.remove(other);
            let keep = if other < one { one - 1 } else { one };
            members[keep].1.extend(taken);
        }
        let mut heads: Vec<u32> = (0..slots as u32).collect();
        for (head, held) in &members {
            for slot in held {
                heads[*slot as usize] = *head;
            }
        }
        heads
    }

    fn learn(lessons: Lessons, lanes: usize) -> Learned {
        let cases = lessons.cases;
        if cases.iter().any(|case| case.sentences.len() != 1) {
            return Err(NO_ONE_SENTENCE.to_string());
        }
        let mut questions_seen: FirstSightCount<&str> = FirstSightCount::new();
        for case in cases {
            questions_seen.add(case.question.as_str(), 1);
        }
        if cases.len() >= REQUEST_SURE_LESSONS && (questions_seen.rows().len() as f64) > REQUEST_HALF * (cases.len() as f64) {
            return Err(NO_SLOT_QUESTIONS.to_string());
        }
        let mut symbols = Symbols::new();
        let marks = FrameMarks::new(&mut symbols);
        let mut questions = RunNumbers::default();
        let mut answers = RunNumbers::default();
        let mut asked: Vec<String> = Vec::new();
        let mut stories: RequestStories = FirstSightTable::new();
        for case in cases {
            let request: Vec<u32> = case.sentences.iter().flat_map(|text| words(text)).map(|word| symbols.number(&word)).collect();
            let question: Vec<u32> = words(&case.question).iter().map(|word| symbols.number(word)).collect();
            let fresh = questions.count() as u32;
            let question = questions.number(&question);
            if question == fresh {
                asked.push(case.question.clone());
            }
            let said: Vec<u32> = answer_words(&case.answer).iter().map(|word| symbols.number(word)).collect();
            let answer = answers.number(&said);
            stories.at(request).put(question, answer);
        }
        let mut other: FirstSightTable<u32, FirstSightCount<u32>> = FirstSightTable::new();
        for (request, told) in stories.rows() {
            let free = vec![false; request.len()];
            for (question, answer) in told.rows() {
                let found = !request_located(request, answers.run(*answer), &free, 0).is_empty();
                let counts = other.at(*question);
                if !found {
                    counts.add(*answer, 1);
                }
            }
        }
        let mut slot_of: NumberTable<u32, u32> = NumberTable::default();
        let mut slot_questions: Vec<u32> = Vec::new();
        for (question, counts) in other.rows() {
            if counts.rows().len() <= 1 {
                slot_of.insert(*question, slot_questions.len() as u32);
                slot_questions.push(*question);
            }
        }
        for (question, counts) in other.rows() {
            let total: i64 = counts.rows().iter().map(|(_, count)| *count).sum();
            if !slot_of.contains_key(question) && total as usize >= REQUEST_SURE_LESSONS && (counts.rows().len() as f64) > REQUEST_HALF * total as f64 {
                return Err(NO_LABEL_ANSWERS.to_string());
            }
        }
        let mut fixed: FirstSightCount<u32> = FirstSightCount::new();
        for question in &slot_questions {
            if let Some(counts) = other.get(question) {
                for (answer, count) in counts.rows() {
                    fixed.add(*answer, *count);
                }
            }
        }
        let none = fixed.first_most().map(|(answer, _)| *answer);
        let none_run: Vec<u32> = none.map(|answer| answers.run(answer).to_vec()).unwrap_or_default();
        let slots = slot_questions.len();
        let names = |answer: u32| answers.run(answer).iter().map(|word| symbols.name(*word)).collect::<Vec<&str>>();
        let mut tagged: Vec<(usize, Vec<u32>)> = Vec::new();
        for (story, (request, told)) in stories.rows().iter().enumerate() {
            if !told.rows().iter().any(|(question, _)| slot_of.contains_key(question)) {
                continue;
            }
            let has: Vec<(u32, u32)> = told.rows().iter().filter_map(|(question, answer)| slot_of.get(question).filter(|_| answers.run(*answer) != none_run.as_slice()).map(|slot| (*slot, *answer))).collect();
            let mut distinct: Vec<u32> = Vec::new();
            for (_, answer) in &has {
                if !distinct.contains(answer) {
                    distinct.push(*answer);
                }
            }
            if distinct.len() != has.len() {
                continue;
            }
            distinct.sort_by(|left, right| names(*left).cmp(&names(*right)));
            let runs: Vec<&[u32]> = distinct.iter().map(|answer| answers.run(*answer)).collect();
            let Some(placed) = request_placed(request, &runs) else { continue };
            let mut tags = vec![TAG_OUTSIDE; request.len()];
            for (slot, answer) in &has {
                let Some(at) = distinct.iter().position(|held| held == answer) else { continue };
                for (first, end) in &placed[at] {
                    tags[*first] = tag_first(*slot as usize);
                    for tag in &mut tags[*first + 1..*end] {
                        *tag = tag_later(*slot as usize);
                    }
                }
            }
            tagged.push((story, tags));
        }
        let mut held: FirstSightCount<u32> = FirstSightCount::new();
        let mut twice = vec![0u32; slots];
        for (_, tags) in &tagged {
            let mut here: FirstSightCount<u32> = FirstSightCount::new();
            for tag in tags.iter().filter(|tag| **tag != TAG_OUTSIDE && !is_later(**tag)) {
                here.add(*tag, 1);
            }
            for (tag, count) in here.rows() {
                let Some(slot) = tag_slot(*tag) else { continue };
                held.add(slot as u32, 1);
                twice[slot] += u32::from(*count > 1);
            }
        }
        let all_held: i64 = held.rows().iter().map(|(_, count)| *count).sum();
        let often = f64::from(twice.iter().sum::<u32>()) / (if all_held == 0 { 1 } else { all_held }) as f64;
        let mut again: Vec<Option<f64>> = vec![None; slots];
        for (slot, count) in held.rows() {
            again[*slot as usize] = Some((f64::from(twice[*slot as usize]) + SHARE_SMOOTH * often) / (*count as f64 + SHARE_SMOOTH));
        }
        let mut counts: Vec<u32> = Vec::new();
        for (request, _) in stories.rows() {
            for word in request {
                *request_count(&mut counts, *word as usize) += 1;
            }
        }
        counts.resize(symbols.len(), 0);
        let mut fills: FirstSightTable<u32, FirstSightCount<u32>> = FirstSightTable::new();
        for (story, tags) in &tagged {
            for (word, tag) in stories.rows()[*story].0.iter().zip(tags) {
                if let Some(slot) = tag_slot(*tag) {
                    fills.at(*word).add(slot as u32, 1);
                }
            }
        }
        let slot_texts: Vec<String> = slot_questions.iter().map(|question| request_text(&questions.run(*question).iter().map(|word| symbols.name(*word)).collect::<Vec<&str>>())).collect();
        let heads = RequestModel::grouped(&fills, &counts, &slot_texts, slots);
        let mut values: Vec<Option<u32>> = vec![None; symbols.len()];
        for (word, filled) in fills.rows() {
            let mut by_class: FirstSightCount<u32> = FirstSightCount::new();
            for (slot, count) in filled.rows() {
                by_class.add(heads[*slot as usize], *count);
            }
            let mut top: Option<(u32, i64)> = None;
            for (head, count) in by_class.rows() {
                let better = match top {
                    None => true,
                    Some((held, most)) => *count > most || (*count == most && slot_texts[*head as usize] > slot_texts[held as usize]),
                };
                if better {
                    top = Some((*head, *count));
                }
            }
            let count = counts[*word as usize];
            if let Some((head, most)) = top
                && (most as f64) >= REQUEST_HALF * f64::from(count)
                && count > REQUEST_RARE
            {
                values[*word as usize] = Some(head);
            }
        }
        let mut class_names = Symbols::new();
        let classes: Vec<Option<u32>> = symbols.names().iter().map(|name| request_class(lessons.classes, name).map(|class| class_names.number(&class))).collect();
        let mut model = RequestModel {
            id: REQUEST_MODELS.fetch_add(1, AtomicOrdering::Relaxed) + 1,
            symbols,
            marks,
            facts: Vec::new(),
            classes: lessons.classes.cloned(),
            class_names,
            questions,
            asked,
            slot_of,
            slot_questions,
            label_of: NumberTable::default(),
            label_questions: Vec::new(),
            readers: Vec::new(),
            answers,
            none: Vec::new(),
            default: frequent_answer(cases).0,
            again,
            table: ShareTable::default(),
            ranks: Vec::new(),
            held: HeldSlots::default(),
            mark_table: KeyNumbers::default(),
            edges: MarkEdges::default(),
        };
        model.none = model.answer_words(none);
        model.facts = (0..model.symbols.len() as u32).map(|word| model.fact(word, &counts, &values, &classes)).collect();
        model.edges = MarkEdges { start: model.marks.start, end: model.marks.end };
        let mut texts: Vec<String> = vec![TEXT_NONE.to_string()];
        for text in &slot_texts {
            texts.push(text.clone());
            texts.push(format!("{TEXT_SAME}{text}{TEXT_CLOSE}"));
        }
        let mut order: Vec<usize> = (0..texts.len()).collect();
        order.sort_by(|left, right| texts[*left].cmp(&texts[*right]));
        model.ranks = vec![0; texts.len()];
        for (rank, tag) in order.iter().enumerate() {
            model.ranks[*tag] = rank as u32;
        }
        model.count_keys(&stories, &tagged, lanes);
        let story_marks: Vec<Vec<u32>> = stories.rows().iter().map(|(request, _)| {
            let seen: Vec<u32> = request.iter().map(|word| model.facts[*word as usize].seen).collect();
            let table = &mut model.mark_table;
            request_marks(&seen, model.edges, &mut |key| Some(table.number(key)))
        }).collect();
        let mark_count = model.mark_table.keys().len();
        let held_lessons: Vec<(Vec<u32>, Vec<u32>)> = tagged.iter().map(|(story, tags)| {
            let mut slots_here: Vec<u32> = Vec::new();
            for tag in tags.iter().filter(|tag| **tag != TAG_OUTSIDE && !is_later(**tag)) {
                if let Some(slot) = tag_slot(*tag)
                    && !slots_here.contains(&(slot as u32))
                {
                    slots_here.push(slot as u32);
                }
            }
            (slots_here, story_marks[*story].clone())
        }).collect();
        model.held = HeldSlots::new(held_lessons, mark_count);
        let mut stored: FirstSightTable<u32, Vec<(u32, Vec<u32>)>> = FirstSightTable::new();
        for (story, (_, told)) in stories.rows().iter().enumerate() {
            for (question, answer) in told.rows() {
                if !model.slot_of.contains_key(question) {
                    stored.at(*question).push((*answer, story_marks[story].clone()));
                }
            }
        }
        for (question, held) in stored.rows() {
            let cut = held.len() * REQUEST_EARLY / REQUEST_THIRDS;
            let small = LabelReader::new(&held[..cut], mark_count, LabelWay::Vote);
            let tried = &held[cut..held.len().min(cut + REQUEST_CHECKED)];
            let right = |way: LabelWay| tried.iter().filter(|(answer, marks)| match small.read(marks, way, &|left, right| model.answer_order(left, right)) {
                Some(said) => said == *answer,
                None => model.answers.run(*answer).is_empty(),
            }).count();
            let way = if right(LabelWay::Vote) >= right(LabelWay::Product) { LabelWay::Vote } else { LabelWay::Product };
            model.label_of.insert(*question, model.readers.len() as u32);
            model.label_questions.push(*question);
            model.readers.push(LabelReader::new(held, mark_count, way));
        }
        Ok(Box::new(model))
    }

    fn count_keys(&mut self, stories: &RequestStories, tagged: &[(usize, Vec<u32>)], _lanes: usize) {
        let tags = tag_count(self.slot_questions.len());
        let mut keys = KeyCounts::default();
        let mut every: FirstSightCount<u32> = FirstSightCount::new();
        let mut found: Vec<Vec<u32>> = Vec::new();
        let mut joined: Vec<u32> = Vec::new();
        let classes = self.classes.is_some();
        for (story, tagging) in tagged {
            let request = &stories.rows()[*story].0;
            let facts: Vec<FrameWord> = request.iter().map(|word| self.facts[*word as usize]).collect();
            let mut before = SLOT_START;
            for (at, tag) in tagging.iter().enumerate() {
                every.add(*tag, 1);
                let frame = frame_of(&facts, at, before, true, &self.marks, classes, &|left, right| self.name_order(left, right));
                found.clear();
                found.extend(frame.word.iter().cloned());
                found.extend(frame.around.iter().cloned());
                for side in &frame.sides {
                    found.extend(side.iter().cloned());
                }
                for pivot in &frame.word {
                    for side in &frame.sides {
                        for key in side {
                            frame_joined(pivot, key, &mut joined);
                            found.push(joined.clone());
                        }
                    }
                }
                if let Some(pivot) = frame_near(&frame.word) {
                    for key in &frame.around {
                        frame_joined(pivot, key, &mut joined);
                        found.push(joined.clone());
                    }
                }
                found.sort_unstable();
                found.dedup();
                for key in &found {
                    keys.add(key, *tag);
                }
                before = RequestModel::slot_code(*tag);
            }
        }
        keys.keep();
        let total: i64 = every.rows().iter().map(|(_, count)| *count).sum();
        let mut all = vec![f64::default(); tags];
        let mut present: Vec<u32> = Vec::new();
        for (tag, count) in every.rows() {
            all[*tag as usize] = *count as f64 / total as f64;
            present.push(*tag);
        }
        self.table = ShareTable { keys, present, all };
    }
}

pub(super) fn learn_requests(lessons: Lessons, lanes: usize) -> Learned {
    RequestModel::learn(lessons, lanes)
}
because!(learn_requests, RequestSlots, "the reader of requests as a tool of the one call: it learns from lessons that are single requests, with the table of word classes of the run when there is one, and it says that it does not read a kind of other cases");
