use crate::base::symbols::Symbols;
use crate::base::words::{answer_words, words};
use crate::cases::Case;
use crate::tools::pattern::classes::{ABSENT, KIND_MARK};
use crate::tools::pattern::hash::{CheapMap, CheapSet};
use crate::tools::pattern::kinds::{JOINT, PatternWords};
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::{WordTexts};
use crate::tools::state::kinds::{KindTables, LEAST_GLUED, STATE_RUN, learn_kinds};
use crate::tools::state::split::{Cut, StatePiece, cut, join_ranges, split, token_of};
use patterns::{because, source};
use std::collections::HashSet;

pub struct StoryUnits;
source!(StoryUnits, "a story is read into units, the sentences and the parts of sentences that tell one step each, so that a program of state changes reads one step at a time; the reading is counted from the lessons and holds no word of a set");

pub struct GluedUnits;
source!(GluedUnits, "a unit that is two lesson forms one after the other is read as these two units, as a text that lost its full stop: one form is read often and more often than the whole, the other a few times");

const SURE_UNITS: u32 = 20;
because!(SURE_UNITS, StoryUnits, "the sentences that a word between repeats needs, the units that the more frequent form of a glued unit needs, and the units that a form needs to be a tested form: the sure count of the learners");

const SURE_PART_SHARE: f64 = 0.98;
because!(SURE_PART_SHARE, StoryUnits, "the share of the sentences with parts in which a word must end the part before the last one to be read as a word between repeats: nearly all, so that a rare other sentence does not stop it");

const KIND_TURNS: usize = 4;
because!(KIND_TURNS, StoryUnits, "the turns in which runs and kinds are found on the units: on all sentences and parts, on what the sentences are read as, and twice more with glued units read as their two units, after which the units settle");

#[derive(Clone, Debug)]
pub struct Unit {
    pub fid: u32,
    pub tokens: Vec<u32>,
    pub raws: Vec<Vec<u32>>,
}
because!(Unit, StoryUnits, "one unit of a story: the number of its form, its tokens, and the words as written of each token");

pub trait FormIds {
    fn id(&mut self, shape: &[u32]) -> u32;
    fn shape(&self, id: u32) -> &[u32];
}
because!(FormIds, StoryUnits, "the numbers of the forms of units: a table that grows while the lessons are read, and the same table with the new forms of one case on top while a case is answered, so that an answer changes nothing in the model");

#[derive(Clone, Debug, Default)]
pub struct Forms {
    pub ids: CheapMap<Vec<u32>, u32>,
    pub shapes: Vec<Vec<u32>>,
}
because!(Forms, StoryUnits, "the forms of the units of the lessons, each with its number in the order of first sight");

impl FormIds for Forms {
    fn id(&mut self, shape: &[u32]) -> u32 {
        if let Some(found) = self.ids.get(shape) {
            return *found;
        }
        let number = self.shapes.len() as u32;
        self.shapes.push(shape.to_vec());
        self.ids.insert(shape.to_vec(), number);
        number
    }

    fn shape(&self, id: u32) -> &[u32] {
        &self.shapes[id as usize]
    }
}

pub struct MoreForms<'a> {
    pub base: &'a Forms,
    more: Vec<Vec<u32>>,
    at: CheapMap<Vec<u32>, u32>,
}
because!(MoreForms, StoryUnits, "the forms of the lessons with the forms of one case on top: a form that no lesson had gets a number after the last form of the lessons, and it leads to no step of a program");

impl<'a> From<&'a Forms> for MoreForms<'a> {
    fn from(base: &'a Forms) -> MoreForms<'a> {
        MoreForms { base, more: Vec::new(), at: CheapMap::default() }
    }
}

impl FormIds for MoreForms<'_> {
    fn id(&mut self, shape: &[u32]) -> u32 {
        if let Some(found) = self.base.ids.get(shape) {
            return *found;
        }
        if let Some(found) = self.at.get(shape) {
            return *found;
        }
        let number = (self.base.shapes.len() + self.more.len()) as u32;
        self.more.push(shape.to_vec());
        self.at.insert(shape.to_vec(), number);
        number
    }

    fn shape(&self, id: u32) -> &[u32] {
        let place = id as usize;
        if place < self.base.shapes.len() { &self.base.shapes[place] } else { &self.more[place - self.base.shapes.len()] }
    }
}

pub struct StateReader {
    pub first: PatternWords,
    pub between: CheapSet<u32>,
    pub counts: Option<CheapMap<Vec<u32>, u32>>,
    pub tables: KindTables,
    pub forms: Forms,
    pub tested: Vec<(Vec<u32>, u32)>,
    pub longest: CheapMap<u32, usize>,
}
because!(StateReader, StoryUnits, GluedUnits, "the reader of stories into units: the first step of the pattern learner, the words between repeats, the counts of the forms of units for glued units, the runs and kinds of the units, the forms, the forms that enough lesson units have, and the longest token of each kind. learn makes it from lessons; unit, chosen and unglue read pieces into units; case reads a story and a question, with the kinds of words that no lesson held");

pub struct ReadCase {
    pub units: Vec<Unit>,
    pub question: Unit,
}
because!(ReadCase, StoryUnits, "a case as the state tool reads it: the units of the story in their order, and the unit of the question");

impl StateReader {
    pub fn shape_text<N: WordTexts>(&self, names: &N, entry: u32) -> String {
        if entry & KIND_MARK != 0 { self.tables.kind_names[(entry & !KIND_MARK) as usize].clone() } else { names.name(entry).to_string() }
    }

    pub fn shape_line<N: WordTexts>(&self, names: &N, shape: &[u32]) -> String {
        let parts: Vec<String> = shape.iter().map(|entry| self.shape_text(names, *entry)).collect();
        parts.join(" ")
    }

    pub fn shape_order<N: WordTexts>(&self, names: &N, a: &[u32], b: &[u32]) -> std::cmp::Ordering {
        let left = a.iter().map(|entry| self.shape_text(names, *entry));
        let right = b.iter().map(|entry| self.shape_text(names, *entry));
        left.cmp(right)
    }

    pub fn norm(&self, raw: &[u32]) -> Vec<u32> {
        raw.iter().map(|word| self.first.norm(*word)).collect()
    }

    pub fn unit<N: WordTexts, F: FormIds>(&self, names: &mut N, forms: &mut F, piece: &StatePiece, extra: &[(u32, u32)], runs: &HashSet<Vec<u32>>) -> Unit {
        let (raw, norm) = piece;
        let spans = join_ranges(norm, &|run: &[u32]| self.tables.runs.contains(run) || runs.contains(run), STATE_RUN);
        let tokens: Vec<u32> = spans.iter().map(|(from, to)| token_of(names, &norm[*from..*to])).collect();
        let raws: Vec<Vec<u32>> = spans.iter().map(|(from, to)| raw[*from..*to].to_vec()).collect();
        let shape: Vec<u32> = tokens
            .iter()
            .map(|token| match crate::tools::pattern::kinds::new_kind(extra, *token) {
                Some(kind) => KIND_MARK | kind,
                None => self.tables.kind.get(token).map_or(*token, |kind| KIND_MARK | kind),
            })
            .collect();
        Unit { fid: forms.id(&shape), tokens, raws }
    }

    pub fn unglue<N: WordTexts, F: FormIds>(&self, names: &mut N, forms: &mut F, piece: &StatePiece, extra: &[(u32, u32)], runs: &HashSet<Vec<u32>>, deep: bool) -> Vec<StatePiece> {
        let Some(counts) = &self.counts else { return vec![piece.clone()] };
        let unit = self.unit(names, forms, piece, extra, runs);
        let shape = forms.shape(unit.fid).to_vec();
        let count_of = |part: &[u32]| counts.get(part).copied().unwrap_or(0);
        let whole = count_of(&shape);
        let (raw, norm) = piece;
        let mut best: Option<(u32, Vec<StatePiece>)> = None;
        for place in 1..shape.len() {
            let head = count_of(&shape[..place]);
            let mut tail = count_of(&shape[place..]);
            let until: usize = unit.raws[..place].iter().map(Vec::len).sum();
            let mut rest: Vec<StatePiece> = vec![(raw[until..].to_vec(), norm[until..].to_vec())];
            if tail < LEAST_GLUED && deep && head >= LEAST_GLUED {
                rest = self.unglue(names, forms, &rest[0], extra, runs, false);
                if rest.len() > 1 {
                    tail = SURE_UNITS + whole;
                }
            }
            let (low, high) = (head.min(tail), head.max(tail));
            if low >= LEAST_GLUED && high >= SURE_UNITS && high > whole && best.as_ref().is_none_or(|(top, _)| low > *top) {
                let mut found: Vec<StatePiece> = vec![(raw[..until].to_vec(), norm[..until].to_vec())];
                found.extend(rest);
                best = Some((low, found));
            }
        }
        best.map_or_else(|| vec![piece.clone()], |(_, found)| found)
    }

    pub fn chosen<N: WordTexts, F: FormIds>(&self, names: &mut N, forms: &mut F, cut: &Cut, extra: &[(u32, u32)], runs: &HashSet<Vec<u32>>) -> Vec<StatePiece> {
        if !cut.parts.is_empty() {
            let mut counted: FirstSightTable<u32, u32> = FirstSightTable::new();
            for part in &cut.parts {
                let unit = self.unit(names, forms, part, extra, runs);
                counted.add(unit.fid, 1);
            }
            let most = counted.first_most().map_or(0, |(_, count)| *count) as usize;
            if most * PAIR_SHARE > cut.parts.len() {
                let mut out: Vec<StatePiece> = cut.lead.iter().cloned().collect();
                out.extend(cut.parts.iter().cloned());
                return out;
            }
        }
        self.unglue(names, forms, &cut.whole, extra, runs, true)
    }

    pub fn units<N: WordTexts, F: FormIds>(&self, names: &mut N, forms: &mut F, cut: &Cut, extra: &[(u32, u32)], runs: &HashSet<Vec<u32>>) -> Vec<Unit> {
        let pieces = self.chosen(names, forms, cut, extra, runs);
        pieces.iter().map(|piece| self.unit(names, forms, piece, extra, runs)).collect()
    }

    pub fn cut_of<N: WordTexts>(&self, names: &mut N, sentence: &str, extra: &[(u32, u32)]) -> Cut {
        cut(&self.first, names, &|token| self.between.contains(&token), sentence, extra)
    }

    pub fn learn(cases: &[Case], symbols: &mut Symbols) -> StateReader {
        let first = PatternWords::learn(cases, symbols);
        let mut before_last: FirstSightTable<u32, u32> = FirstSightTable::new();
        let mut with_parts = 0u32;
        for case in cases {
            for sentence in &case.sentences {
                let found = split(&first, symbols, sentence, &[]);
                if found.parts.len() > 1 {
                    with_parts += 1;
                    let (from, to) = found.parts[found.parts.len() - PAIR_SHARE];
                    let token = found.tokens[to - 1];
                    if to - from > 1 && first.kind_of(token) == ABSENT {
                        before_last.add(token, 1);
                    }
                }
            }
        }
        let between: CheapSet<u32> = before_last.rows().iter().filter(|(_, count)| *count >= SURE_UNITS && f64::from(*count) >= SURE_PART_SHARE * f64::from(with_parts)).map(|(token, _)| *token).collect();
        let empty = KindTables { runs: CheapSet::default(), kind: CheapMap::default(), kind_names: Vec::new(), vocab: CheapSet::default(), shapes: CheapMap::default() };
        let mut reader = StateReader { first, between, counts: None, tables: empty, forms: Forms::default(), tested: Vec::new(), longest: CheapMap::default() };
        let answers: Vec<(Vec<u32>, Vec<u32>)> = cases
            .iter()
            .map(|case| {
                let q: Vec<u32> = words(&case.question).iter().map(|word| symbols.number(word)).collect();
                let a: Vec<u32> = answer_words(&case.answer).iter().map(|word| symbols.number(word)).collect();
                (reader.norm(&q), reader.norm(&a))
            })
            .collect();
        let none: HashSet<Vec<u32>> = HashSet::new();
        let cuts: Vec<Vec<Cut>> = cases.iter().map(|case| case.sentences.iter().map(|sentence| reader.cut_of(symbols, sentence, &[])).collect()).collect();
        for turn in 0..KIND_TURNS {
            let mut read: Vec<(CheapSet<Vec<u32>>, Vec<u32>, Vec<u32>)> = Vec::with_capacity(cases.len());
            let mut forms = std::mem::take(&mut reader.forms);
            for (story_cuts, (q, answer)) in cuts.iter().zip(&answers) {
                let mut story: CheapSet<Vec<u32>> = CheapSet::default();
                for found in story_cuts {
                    if turn == 0 {
                        story.insert(found.whole.1.clone());
                        if let Some(lead) = &found.lead {
                            story.insert(lead.1.clone());
                        }
                        story.extend(found.parts.iter().map(|part| part.1.clone()));
                    } else {
                        story.extend(reader.chosen(symbols, &mut forms, found, &[], &none).into_iter().map(|piece| piece.1));
                    }
                }
                read.push((story, q.clone(), answer.clone()));
            }
            reader.tables = learn_kinds(symbols, &read);
            reader.counts = None;
            let mut forms = Forms::default();
            let mut counts: CheapMap<Vec<u32>, u32> = CheapMap::default();
            let mut order: Vec<Vec<u32>> = Vec::new();
            for story_cuts in &cuts {
                for found in story_cuts {
                    for piece in reader.chosen(symbols, &mut forms, found, &[], &none) {
                        let unit = reader.unit(symbols, &mut forms, &piece, &[], &none);
                        let shape = forms.shape(unit.fid).to_vec();
                        let count = counts.entry(shape.clone()).or_insert(0);
                        if *count == 0 {
                            order.push(shape);
                        }
                        *count += 1;
                    }
                }
            }
            reader.forms = forms;
            if turn > 0 {
                reader.counts = Some(counts.clone());
            }
            if turn + 1 == KIND_TURNS {
                let mut often: FirstSightTable<Vec<u32>, u32> = FirstSightTable::new();
                for shape in order {
                    let count = counts[&shape];
                    often.add(shape, count);
                }
                let mut forms = std::mem::take(&mut reader.forms);
                for case in cases {
                    let raw: Vec<u32> = words(&case.question).iter().map(|word| symbols.number(word)).collect();
                    let norm = reader.norm(&raw);
                    let unit = reader.unit(symbols, &mut forms, &(raw, norm), &[], &none);
                    often.add(forms.shape(unit.fid).to_vec(), 1);
                }
                reader.forms = forms;
                let mut tested: Vec<(Vec<u32>, u32)> = often.rows().iter().filter(|(_, count)| *count >= SURE_UNITS).cloned().collect();
                tested.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| reader.shape_order(symbols, &a.0, &b.0)));
                reader.tested = tested;
            }
        }
        let mut longest: CheapMap<u32, usize> = CheapMap::default();
        for (token, kind) in reader.tables.kind.iter() {
            let size = symbols.name(*token).matches(JOINT).count() + 1;
            let held = longest.entry(*kind).or_insert(1);
            *held = (*held).max(size);
        }
        reader.longest = longest;
        reader
    }
}

pub const PAIR_SHARE: usize = 2;
because!(PAIR_SHARE, StoryUnits, "a sentence is read as its parts when more than half of the parts have one form; and the part before the last one is the second part from the end");
