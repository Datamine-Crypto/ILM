use crate::sets::made::family::{Family, MadeSlot};
use patterns::{because, source};
use std::collections::HashMap;

pub struct MadeInterpreter;
source!(MadeInterpreter, "the interpreter of a family: the answers of a row of words by every reading of it as an item or as a function of the family with its parts, found by a chart of the spans of the row, as a parser of a grammar with parts inside parts fills its table; a text of the family has one reading, and a row with no reading or with readings of other answers is no text of the family");

const MADE_MOST_ANSWERS: usize = 3;
because!(MADE_MOST_ANSWERS, MadeInterpreter, "the answers of one span that are kept: more than one already says that the span has two readings");

const MADE_MOST_ROWS: usize = 4;
because!(MADE_MOST_ROWS, MadeInterpreter, "the readings of the rest of a way from one place that are kept: enough to see a second answer, few enough that a long row is read quickly");

type MadeAnswers = Vec<Vec<String>>;
because!(MadeAnswers, MadeInterpreter, "the different answers of a span, in the order in which they were found");

type MadeRows = Vec<Vec<Vec<String>>>;
because!(MadeRows, MadeInterpreter, "the readings of the rest of a way: for each, the answers of the parts that it holds");

struct MadeChart<'a> {
    family: &'a Family,
    words: &'a [String],
    spans: HashMap<(usize, usize, Option<u32>, bool), MadeAnswers>,
    rows: HashMap<(usize, usize, usize, usize, usize), MadeRows>,
}
because!(MadeChart, MadeInterpreter, "the chart of one row of words: the answers of each span as an item or a function of a level at most, and the readings of the rest of each way from each place");

fn made_add(out: &mut MadeAnswers, answer: Vec<String>) {
    if !out.contains(&answer) {
        out.push(answer);
    }
}
because!(made_add, MadeInterpreter, "one more answer of a span, kept once");

impl MadeChart<'_> {
    fn read(&mut self, first: usize, end: usize, level: Option<u32>, function: bool) -> MadeAnswers {
        let key = (first, end, level, function);
        if let Some(found) = self.spans.get(&key) {
            return found.clone();
        }
        self.spans.insert(key, Vec::new());
        let mut out: MadeAnswers = Vec::new();
        let family = self.family;
        if end - first == 1 && !function && family.is_item(&self.words[first]) {
            made_add(&mut out, family.said(&self.words[first]));
        }
        for (number, ways) in family.functions.iter().enumerate() {
            for (way_number, way) in ways.iter().enumerate() {
                if level.is_none_or(|top| way.level <= top) && end - first >= way.layout.len() {
                    for parts in self.row(number, way_number, 0, first, end) {
                        if out.len() < MADE_MOST_ANSWERS {
                            made_add(&mut out, family.joined(&way.says, &parts));
                        }
                    }
                }
            }
        }
        self.spans.insert(key, out.clone());
        out
    }

    fn part(&mut self, first: usize, end: usize, level: Option<u32>) -> MadeAnswers {
        let Some((open, close)) = self.family.wrap.clone() else { return self.read(first, end, level, false) };
        if end - first == 1 {
            return if self.family.is_item(&self.words[first]) { vec![self.family.said(&self.words[first])] } else { Vec::new() };
        }
        if self.words[first] == open && self.words[end - 1] == close {
            return self.read(first + 1, end - 1, level, true);
        }
        Vec::new()
    }

    fn row(&mut self, number: usize, way_number: usize, at: usize, first: usize, end: usize) -> MadeRows {
        let key = (number, way_number, at, first, end);
        if let Some(found) = self.rows.get(&key) {
            return found.clone();
        }
        self.rows.insert(key, Vec::new());
        let family = self.family;
        let way = &family.functions[number][way_number];
        let length = way.layout.len();
        let mut out: MadeRows = Vec::new();
        if at == length {
            if first == end {
                out.push(Vec::new());
            }
        } else if let MadeSlot::Fixed(word) = &way.layout[at] {
            if first < end && self.words[first] == *word {
                out = self.row(number, way_number, at + 1, first + 1, end);
            }
        } else if let MadeSlot::Part(part) = way.layout[at] {
            let after = length - at - 1;
            let accept = way.accept[part];
            let next_fixed = way.layout.get(at + 1).and_then(|slot| match slot {
                MadeSlot::Fixed(word) => Some(word.clone()),
                MadeSlot::Part(_) => None,
            });
            let last = (end + 1).saturating_sub(after);
            for stop in first + 1..last {
                if after > 0
                    && let Some(word) = &next_fixed
                    && (stop >= end || self.words[stop] != *word)
                {
                    continue;
                }
                if after == 0 && stop != end {
                    continue;
                }
                let answers = self.part(first, stop, accept);
                if answers.is_empty() {
                    continue;
                }
                for rest in self.row(number, way_number, at + 1, stop, end) {
                    for answer in &answers {
                        if out.len() < MADE_MOST_ROWS {
                            let mut joined = vec![answer.clone()];
                            joined.extend(rest.iter().cloned());
                            out.push(joined);
                        }
                    }
                }
            }
        }
        self.rows.insert(key, out.clone());
        out
    }
}

pub fn made_answers(family: &Family, words: &[String]) -> Vec<Vec<String>> {
    let mut chart = MadeChart { family, words, spans: HashMap::new(), rows: HashMap::new() };
    chart.read(0, words.len(), None, false)
}
because!(made_answers, MadeInterpreter, "the answers of all readings of a row of words, at most a few: one for a text of the family");
