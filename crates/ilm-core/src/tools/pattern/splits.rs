use crate::base::symbols::Symbols;
use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::codes::{ALL_SPLIT, FREE, Point, Scope, Step, split_slot};
use crate::tools::pattern::counting::Counting;
use crate::tools::pattern::finder::place_of;
use crate::tools::pattern::kinds::PatternWords;
use crate::tools::pattern::learn::learn_counted;
use crate::tools::pattern::model::{Hands, Part, PatternAnswers, PatternModel, SURE_READINGS, SURE_SHARE, Setting};
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::WordTexts;
use crate::tools::pattern::story::Query;
use crate::tools::pattern::striped::Shared;
use crate::tools::pattern::table::{PLACE_BITS, Places, Ready, Table};
use patterns::{because, source};

pub struct QuestionSplit;
source!(QuestionSplit, "a question over several things is asked for each thing alone, the decomposition of a question into simpler questions: a part learner is the same learner taught on the cases with one candidate, and the answer of the whole case is counted under the answers for its candidates");

pub const PART_DEPTH: u32 = 2;
because!(PART_DEPTH, QuestionSplit, "the most told sentences of a pattern of a part learner: the question for one candidate is simpler than the whole question, and a part learner is taught for many pairs of question form and kind");

pub const MOST_SPLITS: usize = 8;
because!(MOST_SPLITS, QuestionSplit, "the most splits of one set of lessons: each split teaches a learner of its own, and the question forms with the most open cases come first");

pub const HALF_PARTS: usize = 2;
because!(HALF_PARTS, QuestionSplit, "a part learner stays only when its list leaves open no more than one of this many of its own cases: a learner that does not close its own lessons gives answers for the candidates that are guesses");

pub fn candidates(case: &Query, kind: u32) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    for sent in &case.hits {
        for word in case.story.sents[*sent as usize].of_kind(kind) {
            if !case.slots.contains(word) && !out.iter().any(|(held, _)| held == word) {
                out.push((*word, *sent));
            }
        }
    }
    out
}
because!(candidates, QuestionSplit, "the candidates of a case for a split: the words of one kind that stand in a sentence with a slot of the question and are no slot themselves, each with its first such sentence, in the order of the story");

pub fn split_name<N: WordTexts>(names: &mut N, outs: &[(u32, u32)], null: &[u32]) -> (u32, Vec<u32>) {
    let named: Vec<u32> = outs.iter().filter(|(_, label)| *label == ABSENT).map(|(word, _)| *word).collect();
    let mut labels: Vec<&str> = outs.iter().filter(|(_, label)| *label != ABSENT && !null.contains(label)).map(|(_, label)| names.name(*label)).collect();
    labels.sort_unstable();
    labels.extend(named.iter().map(|_| "Q"));
    let text = if labels.is_empty() { "-".to_string() } else { labels.join(" ") };
    (names.number(&text), named)
}
because!(split_name, QuestionSplit, "the name of a split of one case from the answers for its candidates: the answers that add something in the order of their texts, then one mark for each candidate that is its own answer; and these candidates, which are the slots of the split");

impl PatternModel {
    pub fn outcomes<N: WordTexts>(&self, words: &PatternWords, names: &mut N, hands: &mut Hands, case: &Query, part: usize, light: bool) -> Option<Vec<(u32, u32)>> {
        let mut found = candidates(case, self.parts[part].kind);
        found.sort_by_key(|(_, sent)| *sent);
        let mut out: Vec<(u32, u32)> = Vec::with_capacity(found.len());
        for (word, _) in found {
            let mut q = case.q.clone();
            q.push(self.hash);
            q.push(word);
            let sub = Query::on(case.story.clone(), q, words);
            let (got, _) = self.parts[part].model.answer_case(words, names, hands, &sub, light);
            if got.is_empty() {
                return None;
            }
            if got == [word] {
                out.push((word, ABSENT));
                continue;
            }
            let text = format!("={}", got.iter().map(|said| names.name(*said)).collect::<Vec<&str>>().join("+="));
            out.push((word, names.number(&text)));
        }
        Some(out)
    }

    pub fn split_keys<N: WordTexts>(&self, names: &mut N, part: usize, outs: &[(u32, u32)]) -> (Vec<Step>, Vec<(u32, u32)>) {
        let (name, named) = split_name(names, outs, &self.parts[part].null);
        let slots: Vec<(u32, u32)> = named.iter().enumerate().map(|(place, word)| (split_slot(place), *word)).collect();
        let lead = Step { scope: Scope::Whole, point: Point::Split, name: FREE, second: FREE, item: self.parts[part].kind, value: name };
        let mut keys: Vec<Step> = vec![lead];
        let only_own = !named.is_empty() && outs.iter().all(|(_, label)| *label == ABSENT || self.parts[part].null.contains(label));
        if only_own {
            keys.push(Step { value: names.number("Q*"), ..lead });
        }
        (keys, slots)
    }
}

pub fn learn_splits(count: &mut Counting, words: &PatternWords, symbols: &mut Symbols, set: Setting) -> Vec<Part> {
    let mut parts: Vec<Part> = Vec::new();
    let mut open = count.open_forms(symbols);
    open.sort_by(|a, b| b.1.cmp(&a.1));
    let ready = count.ready;
    for (asked, _) in open {
        if parts.len() >= MOST_SPLITS {
            break;
        }
        let Some(root) = count.table.child.get(ABSENT, asked) else { continue };
        let asked_row = count.asked.row(asked);
        let cases: Vec<usize> = (0..ready.len()).filter(|number| ready[*number].case.form == asked_row && !ready[*number].case.slots.is_empty()).collect();
        for kind in 0..words.kind_names.len() as u32 {
            let single: Vec<(usize, u32)> = cases.iter().filter_map(|number| {
                let found = candidates(&ready[*number].case, kind);
                (found.len() == 1).then(|| (*number, found[0].0))
            }).collect();
            if (single.len() as u32) < SURE_READINGS {
                continue;
            }
            let sub: Vec<Ready> = single
                .iter()
                .map(|(number, word)| {
                    let case = &ready[*number].case;
                    let mut q = case.q.clone();
                    q.push(set.hash);
                    q.push(*word);
                    Ready { case: Query::on(case.story.clone(), q, words), answer: ready[*number].answer.clone() }
                })
                .collect();
            let model = learn_counted(words, symbols, &sub, Setting { depth: set.depth.min(PART_DEPTH), per_form: false, split: false, ..set });
            if model.left * HALF_PARTS > single.len() {
                continue;
            }
            let mut holder = PatternModel { parts: vec![Part { asked, kind, model, null: Vec::new(), used: false }], ..empty_model(set) };
            let outs: Vec<Option<Vec<(u32, u32)>>> = {
                let mut hands = holder.parts[0].model.hands(words);
                cases.iter().map(|number| holder.outcomes(words, symbols, &mut hands, &ready[*number].case, 0, false)).collect()
            };
            let mut raw: FirstSightTable<u32, FirstSightTable<Vec<u32>, u32>> = FirstSightTable::new();
            let mut places: Vec<Vec<u32>> = Vec::with_capacity(cases.len());
            let mut row: Vec<u32> = Vec::new();
            for (number, out) in cases.iter().zip(&outs) {
                let Some(out) = out else {
                    places.push(Vec::new());
                    continue;
                };
                let (name, named) = split_name(symbols, out, &[]);
                let slots: Vec<(u32, u32)> = named.iter().enumerate().map(|(place, word)| (split_slot(place), *word)).collect();
                place_of(&ready[*number].answer, &ready[*number].case, &slots, &[], &mut row);
                raw.entry(name, FirstSightTable::new()).add(row.clone(), 1);
                places.push(row.clone());
            }
            let mut labels: Vec<u32> = outs.iter().flatten().flatten().map(|(_, label)| *label).filter(|label| *label != ABSENT).collect();
            labels.sort_by(|a, b| symbols.name(*a).cmp(symbols.name(*b)));
            labels.dedup();
            let mut null: Vec<u32> = Vec::new();
            for label in labels {
                let (mut agree, mut total) = (0, 0);
                for (out, place) in outs.iter().zip(&places) {
                    let Some(out) = out else { continue };
                    let Some(at) = out.iter().position(|(_, held)| *held == label) else { continue };
                    if out.len() <= 1 {
                        continue;
                    }
                    let mut rest = out.clone();
                    rest.remove(at);
                    let (name, _) = split_name(symbols, &rest, &[]);
                    if let Some((top, _)) = raw.get(&name).and_then(FirstSightTable::first_most) {
                        total += 1;
                        agree += u32::from(top == place);
                    }
                }
                if total >= SURE_READINGS && f64::from(agree) >= SURE_SHARE * f64::from(total) {
                    null.push(label);
                }
            }
            holder.parts[0].null = null;
            let mut found: FirstSightTable<Step, Vec<u32>> = FirstSightTable::new();
            for (number, out) in cases.iter().zip(&outs) {
                let Some(out) = out else { continue };
                let (keys, slots) = holder.split_keys(symbols, 0, out);
                let named: Vec<u32> = slots.iter().map(|(_, word)| *word).collect();
                for (place, key) in keys.iter().enumerate() {
                    if place > 0 && ready[*number].answer == named {
                        row.clear();
                        row.push(ALL_SPLIT);
                    } else {
                        place_of(&ready[*number].answer, &ready[*number].case, &slots, &[], &mut row);
                    }
                    let reading = ((*number as u32) << PLACE_BITS) | count.places.number(&row);
                    found.entry(*key, Vec::new()).push(reading);
                }
            }
            for (key, readings) in found.rows() {
                let step = count.steps.number(key);
                count.table.add_with(root, step, readings);
            }
            parts.append(&mut holder.parts);
            if parts.len() >= MOST_SPLITS {
                break;
            }
        }
    }
    if !parts.is_empty() {
        count.make_list(symbols);
    }
    parts
}
because!(learn_splits, QuestionSplit, "the splits of a set of lessons: for the question forms that the list leaves open, most open cases first, and for each kind, a part learner is taught on the cases with one candidate; it stays when it closes half of its own cases or more; then every case of the form is counted under the answers of the part learner for its candidates, and an answer that does not change the place of the answer of the whole case is left out of the name of the split");

pub fn empty_model(set: Setting) -> PatternModel {
    PatternModel {
        depth: set.depth,
        star: set.star,
        hash: set.hash,
        forms: Shared::default().freeze(),
        steps: Shared::default().freeze(),
        asked: Shared::default().freeze(),
        places: Places::default(),
        table: Table::default(),
        place: Vec::new(),
        hit: Vec::new(),
        rank: Vec::new(),
        order: Vec::new(),
        left: 0,
        default: Vec::new(),
        point_kinds: Vec::new(),
        parts: Vec::new(),
    }
}
because!(empty_model, PatternAnswers, "a model with no lesson, which holds the parts of a split while they are counted");
