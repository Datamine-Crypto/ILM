use crate::base::keys::NO_NUMBER;
use crate::base::symbols::Symbols;
use crate::base::words::words;
use crate::tools::call::{Learner, Said};
use crate::tools::requests::frames::{FrameKey, FramePart, FrameWord, SIDE_LEFT, SIDE_RIGHT, SIDE_SLOT, SLOT_NONE, SLOT_START, frame_of};
use crate::tools::requests::model::{RequestModel, request_class};
use crate::tools::requests::shares::ShareHand;
use crate::tools::requests::tags::{TAG_OUTSIDE, is_later, tag_first, tag_slot};
use crate::tools::requests::whole::request_marks;
use patterns::{because, source};
use std::cell::RefCell;
use std::cmp::Ordering;

pub struct BeamReading;
source!(BeamReading, "a request is read from its first word to its last with the best rows of tags kept, a beam search: a row is judged as a whole by the product of the shares of its tags; a later word of a run goes on with the slot of the word before it and with no other; a slot that a row holds already has a price for a second run; and the first word of a run of a slot is lifted by the share of the slot among the nearest lesson requests. The answer to a slot question is the words of the request with that slot, or the fixed word");

const BEAM_ROWS: usize = 4;
because!(BEAM_ROWS, BeamReading, "the rows of tags that are kept while a request is read: enough that a row whose first words look worse can win later, few enough to be quick");

const BEAM_TRIED: usize = 3;
because!(BEAM_TRIED, BeamReading, "the tags of a word that are tried after each row: the best few, since a tag with a small share seldom wins");

const SHOWN_PLACES: usize = 2;
because!(SHOWN_PLACES, BeamReading, "the places after the point of a lift in the lines that say how a word was read");

const NAME_OUTSIDE: &str = "outside";
because!(NAME_OUTSIDE, BeamReading, "the name of the tag of no slot in the lines that say how a word was read");

const NAME_SAME: &str = "same, ";
because!(NAME_SAME, BeamReading, "the start of the name of the tag of a later word of a run, before the name of its slot");

const NAME_START: &str = "<";
because!(NAME_START, BeamReading, "the name of the slot before the first word of a request in the lines that say how a word was read");

const WORDS_BETWEEN: &str = " ";
because!(WORDS_BETWEEN, BeamReading, "the mark between the words of a value or of a label that a request gives with no question");

const NOTE_SLOT: &str = "words of the request";
because!(NOTE_SLOT, BeamReading, "the note of an answer that is the words of a slot of the request");

const NOTE_NONE: &str = "no such slot in the request";
because!(NOTE_NONE, BeamReading, "the note of an answer that is the fixed word of a request with no such slot");

const NOTE_LABEL: &str = "a label of the request";
because!(NOTE_LABEL, BeamReading, "the note of an answer that is a label of the whole request");

const NOTE_UNKNOWN: &str = "a question that the lessons do not hold";
because!(NOTE_UNKNOWN, BeamReading, "the note of an answer to a question that no lesson asked");

#[derive(Debug, Default)]
struct BeamKept {
    model: u64,
    texts: Vec<String>,
    tags: Vec<u32>,
    marks: Vec<u32>,
}
because!(BeamKept, BeamReading, "the tags of the last request that one thread read, kept for the next questions of the same request, with the model and the words that they belong to, so that a request with many questions is read once");

thread_local! {
    static BEAM_KEPT: RefCell<BeamKept> = RefCell::new(BeamKept::default());
}

#[derive(Clone, Debug)]
struct BeamRow {
    score: f64,
    tags: Vec<u32>,
    slot: u32,
    used: Vec<u32>,
}
because!(BeamRow, BeamReading, "one row of tags while a request is read: the sum of the logarithms of the shares of its tags, the tags so far, the slot of its last word, and the slots that it holds");

impl RequestModel {
    fn request(&self, texts: &[String], strangers: &mut Symbols) -> Vec<FrameWord> {
        let known = self.symbols.len() as u32;
        texts
            .iter()
            .map(|text| match self.symbols.find(text) {
                Some(word) => self.facts[word as usize],
                None => {
                    let class = request_class(self.classes.as_ref(), text).map(|class| self.class_names.find(&class).unwrap_or(NO_NUMBER));
                    FrameWord { word: known + strangers.number(text), seen: self.marks.new, hidden: self.marks.hidden, new: true, values: None, class }
                }
            })
            .collect()
    }

    fn tags(&self, facts: &[FrameWord], hand: &mut ShareHand) -> Vec<u32> {
        let seen: Vec<u32> = facts.iter().map(|fact| fact.seen).collect();
        let marks = request_marks(&seen, self.edges, &mut |key| self.mark_table.find(key));
        let whole = self.held.lifts(&marks, self.slot_questions.len());
        let mut rows = vec![BeamRow { score: f64::default(), tags: Vec::new(), slot: SLOT_START, used: Vec::new() }];
        for at in 0..facts.len() {
            let frame = frame_of(facts, at, SLOT_START, false, &self.marks, self.classes.is_some(), &|left, right| self.name_order(left, right));
            let (levels, share) = self.table.place(&frame, hand);
            let mut after: Vec<(u32, Vec<f64>)> = Vec::new();
            let mut next: Vec<BeamRow> = Vec::new();
            for row in &rows {
                if !after.iter().any(|(slot, _)| *slot == row.slot) {
                    let values = self.table.shares(&levels, &share, row.slot, hand);
                    after.push((row.slot, values));
                }
                let Some((_, values)) = after.iter().find(|(slot, _)| *slot == row.slot) else { continue };
                let mut options: Vec<(f64, u32)> = Vec::new();
                for tag in &self.table.present {
                    if is_later(*tag) && RequestModel::slot_code(*tag) != row.slot {
                        continue;
                    }
                    let mut value = values[*tag as usize];
                    if let Some(slot) = tag_slot(*tag).filter(|_| !is_later(*tag)) {
                        if row.used.contains(tag)
                            && let Some(price) = self.again[slot]
                        {
                            value *= price;
                        }
                        if let Some(lift) = whole[slot] {
                            value *= lift;
                        }
                    }
                    options.push((value, *tag));
                }
                options.sort_by(|left, right| right.0.partial_cmp(&left.0).unwrap_or(Ordering::Equal).then(self.ranks[left.1 as usize].cmp(&self.ranks[right.1 as usize])));
                for (value, tag) in options.into_iter().take(BEAM_TRIED) {
                    if value <= f64::default() {
                        continue;
                    }
                    let mut tags = row.tags.clone();
                    tags.push(tag);
                    let mut used = row.used.clone();
                    if tag != TAG_OUTSIDE && !is_later(tag) && !used.contains(&tag) {
                        used.push(tag);
                    }
                    next.push(BeamRow { score: row.score + value.ln(), tags, slot: if is_later(tag) { row.slot } else { RequestModel::slot_code(tag) }, used });
                }
            }
            next.sort_by(|left, right| right.score.partial_cmp(&left.score).unwrap_or(Ordering::Equal));
            next.truncate(BEAM_ROWS);
            if next.is_empty() {
                let mut first = rows[0].clone();
                first.tags.push(TAG_OUTSIDE);
                first.slot = SLOT_NONE;
                next.push(first);
            }
            rows = next;
        }
        rows.swap_remove(0).tags
    }

    fn runs(tags: &[u32]) -> Vec<(u32, usize, usize)> {
        let mut out: Vec<(u32, usize, usize)> = Vec::new();
        for (at, tag) in tags.iter().enumerate() {
            if is_later(*tag) {
                if let Some(last) = out.last_mut().filter(|last| last.2 == at) {
                    last.2 = at + 1;
                }
            } else if *tag != TAG_OUTSIDE {
                out.push((*tag, at, at + 1));
            }
        }
        out
    }

    fn named(&self, tag: u32) -> String {
        let Some(slot) = tag_slot(tag) else { return NAME_OUTSIDE.to_string() };
        let name = &self.asked[self.slot_questions[slot] as usize];
        if is_later(tag) { format!("{NAME_SAME}{name}") } else { name.clone() }
    }

    fn question_of(&self, question: &str) -> Option<u32> {
        let found: Option<Vec<u32>> = words(question).iter().map(|word| self.symbols.find(word)).collect();
        self.questions.find(&found?)
    }

    fn label_words(&self, reader: u32, marks: &[u32]) -> Vec<String> {
        let reader = &self.readers[reader as usize];
        self.answer_words(reader.read(marks, reader.way, &|left, right| self.answer_order(left, right)))
    }

    fn read_request(&self, texts: &[String]) -> (Vec<FrameWord>, Vec<u32>, Vec<u32>) {
        let mut strangers = Symbols::new();
        let facts = self.request(texts, &mut strangers);
        let kept = BEAM_KEPT.with_borrow(|kept| (kept.model == self.id && kept.texts == texts).then(|| (kept.tags.clone(), kept.marks.clone())));
        if let Some((tags, marks)) = kept {
            return (facts, tags, marks);
        }
        let mut hand = ShareHand::default();
        let tags = self.tags(&facts, &mut hand);
        let seen: Vec<u32> = facts.iter().map(|fact| fact.seen).collect();
        let marks = request_marks(&seen, self.edges, &mut |key| self.mark_table.find(key));
        BEAM_KEPT.with_borrow_mut(|kept| *kept = BeamKept { model: self.id, texts: texts.to_vec(), tags: tags.clone(), marks: marks.clone() });
        (facts, tags, marks)
    }

    fn shown(&self, key: &FrameKey) -> String {
        let names: Vec<String> = match key.first().copied() {
            Some(part) if part == FramePart::Slot as u32 => vec![match key[1] {
                SLOT_START => NAME_START.to_string(),
                SLOT_NONE => NAME_OUTSIDE.to_string(),
                code => self.named(tag_first((code - SLOT_NONE - 1) as usize)),
            }],
            Some(part) if part == FramePart::ClassLeft as u32 || part == FramePart::ClassRight as u32 => key[1..].iter().zip([true, false]).map(|(value, table)| if table { value.to_string() } else { self.class_names.names().get(*value as usize).cloned().unwrap_or_default() }).collect(),
            _ => key[1..].iter().map(|word| self.symbols.names().get(*word as usize).cloned().unwrap_or_default()).collect(),
        };
        names.join(WORDS_BETWEEN)
    }

    fn trace(&self, text: &str) -> Vec<String> {
        let texts = words(text);
        let (facts, tags, marks) = self.read_request(&texts);
        let whole = self.held.lifts(&marks, self.slot_questions.len());
        let mut hand = ShareHand::default();
        let mut lines: Vec<String> = Vec::new();
        for (at, (word, tag)) in texts.iter().zip(&tags).enumerate() {
            let before = if at == 0 { SLOT_START } else { RequestModel::slot_code(tags[at - 1]) };
            let frame = frame_of(&facts, at, before, false, &self.marks, self.classes.is_some(), &|left, right| self.name_order(left, right));
            let place = frame_of(&facts, at, SLOT_START, false, &self.marks, self.classes.is_some(), &|left, right| self.name_order(left, right));
            let (levels, _) = self.table.place(&place, &mut hand);
            let mut line = format!("{word}: {}", self.named(*tag));
            let own = (!facts[at].new).then(|| self.table.keys.find(&[FramePart::Word as u32, facts[at].word])).flatten();
            match own {
                Some(number) => {
                    let mut counted: Vec<(u32, u32)> = self.table.keys.tags[number].clone();
                    counted.sort_by(|left, right| right.1.cmp(&left.1));
                    let shown: Vec<String> = counted.iter().take(BEAM_TRIED).map(|(tag, count)| format!("{} {count}", self.named(*tag))).collect();
                    line.push_str(&format!("; the word in the lessons: {} of {}", shown.join(", "), self.table.keys.totals[number]));
                }
                None => line.push_str("; a new word"),
            }
            for (label, side) in [("the word before", SIDE_LEFT), ("the word after", SIDE_RIGHT), ("the slot before", SIDE_SLOT)] {
                let keys = &frame.sides[side];
                let Some(key) = keys.iter().rev().find(|key| self.table.keys.find(key).is_some()) else { continue };
                let Some(number) = self.table.keys.find(key) else { continue };
                let count = self.table.keys.tags[number].iter().find(|(held, _)| held == tag).map_or(0, |(_, count)| *count);
                let lift = self.table.lifted(&levels, keys, &mut hand).map_or(f64::from(1u8), |lift| lift[*tag as usize]);
                line.push_str(&format!("; {label} ({}): lift {:.*}, {} {count} of {}", self.shown(key), SHOWN_PLACES, lift, self.named(*tag), self.table.keys.totals[number]));
            }
            if let Some(lift) = tag_slot(*tag).filter(|_| !is_later(*tag)).and_then(|slot| whole[slot]) {
                line.push_str(&format!("; the request as a whole: lift {:.*}", SHOWN_PLACES, lift));
            }
            lines.push(line);
        }
        lines
    }
}

impl Learner for RequestModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let texts: Vec<String> = sentences.iter().flat_map(|text| words(text)).collect();
        let question = self.question_of(question);
        if let Some(slot) = question.and_then(|question| self.slot_of.get(&question)) {
            let (_, tags, _) = self.read_request(&texts);
            let found: Vec<String> = RequestModel::runs(&tags).iter().filter(|(tag, _, _)| tag_slot(*tag) == Some(*slot as usize)).flat_map(|(_, first, end)| texts[*first..*end].iter().cloned()).collect();
            if found.is_empty() {
                return Said { words: self.none.clone(), note: NOTE_NONE.to_string() };
            }
            return Said { words: found, note: NOTE_SLOT.to_string() };
        }
        if let Some(reader) = question.and_then(|question| self.label_of.get(&question)) {
            let mut strangers = Symbols::new();
            let facts = self.request(&texts, &mut strangers);
            let seen: Vec<u32> = facts.iter().map(|fact| fact.seen).collect();
            let marks = request_marks(&seen, self.edges, &mut |key| self.mark_table.find(key));
            return Said { words: self.label_words(*reader, &marks), note: NOTE_LABEL.to_string() };
        }
        Said { words: self.default.clone(), note: NOTE_UNKNOWN.to_string() }
    }

    fn facts(&self, text: &str) -> Option<Vec<(String, String)>> {
        let texts = words(text);
        let (_, tags, marks) = self.read_request(&texts);
        let mut found: Vec<(String, String)> = RequestModel::runs(&tags).iter().map(|(tag, first, end)| (self.named(*tag), texts[*first..*end].join(WORDS_BETWEEN))).collect();
        let mut labels: Vec<(u32, u32)> = self.label_questions.iter().copied().zip(0..).collect();
        labels.sort_by(|left, right| {
            let names = |question: u32| self.questions.run(question).iter().map(|word| self.symbols.name(*word)).collect::<Vec<&str>>();
            names(left.0).cmp(&names(right.0))
        });
        for (question, reader) in labels {
            found.push((self.asked[question as usize].clone(), self.label_words(reader, &marks).join(WORDS_BETWEEN)));
        }
        Some(found)
    }

    fn explain(&self, text: &str) -> Vec<String> {
        self.trace(text)
    }
}
