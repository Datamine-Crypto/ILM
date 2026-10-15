use crate::base::words::words;
use crate::tools::pattern::classes::KIND_MARK;
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::kinds::{JOINT, new_kind, set_new_kind};
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::{WordTexts};
use crate::tools::state::kinds::{STATE_RUN, unit_tokens_with};
use crate::tools::state::reader::{ReadCase, FormIds, StateReader};
use crate::tools::state::split::{Cut, StatePiece};
use patterns::{because, source};
use std::collections::HashSet;

pub struct FormFit;
source!(FormFit, "a word that the lessons never held is read by the place where it stands: it takes the kind that the most frequent lesson form with the same tokens and kinds in all other places has there; a run of words with a new word is one word of a kind when its unit then has a lesson form; a unit with several groups of new words is fitted to the most frequent tested lesson form");

const FIT_ROUNDS: usize = 2;
because!(FIT_ROUNDS, FormFit, "the rounds of fitting units to tested forms: the second round reads the units again with the runs and kinds of the first");

fn pieces_of(cut: &Cut) -> Vec<&StatePiece> {
    let mut out: Vec<&StatePiece> = vec![&cut.whole];
    out.extend(cut.lead.iter());
    out.extend(cut.parts.iter());
    out
}
because!(pieces_of, FormFit, "all pieces of a cut sentence: the whole, the text before the first repeat, and the parts");

impl StateReader {
    fn is_new(&self, token: u32, extra: &[(u32, u32)]) -> bool {
        !self.tables.vocab.contains(&token) && new_kind(extra, token).is_none()
    }

    fn entry_of(&self, token: u32, extra: &[(u32, u32)]) -> u32 {
        match new_kind(extra, token) {
            Some(kind) => KIND_MARK | kind,
            None => self.tables.kind.get(&token).map_or(token, |kind| KIND_MARK | kind),
        }
    }

    pub fn new_kinds(&self, texts: &[Vec<u32>]) -> Vec<(u32, u32)> {
        let mut votes: FirstSightTable<u32, FirstSightTable<u32, u32>> = FirstSightTable::new();
        for tokens in texts {
            let new_at: Vec<usize> = (0..tokens.len()).filter(|at| !self.tables.vocab.contains(&tokens[*at])).collect();
            let Some(shapes) = self.tables.shapes.get(&tokens.len()) else { continue };
            if new_at.is_empty() {
                continue;
            }
            let mut distinct: Vec<u32> = new_at.iter().map(|at| tokens[*at]).collect();
            distinct.sort_unstable();
            distinct.dedup();
            let mut best: Option<(&Vec<u32>, u32)> = None;
            for (shape, count) in shapes.rows() {
                if best.is_some_and(|(_, top)| *count <= top) {
                    continue;
                }
                let known = (0..tokens.len()).filter(|at| !new_at.contains(at)).all(|at| shape[at] == self.tables.kind.get(&tokens[at]).map_or(tokens[at], |kind| KIND_MARK | kind));
                let kinds = new_at.iter().all(|at| shape[*at] & KIND_MARK != 0);
                let mut paired: Vec<(u32, u32)> = new_at.iter().map(|at| (tokens[*at], shape[*at])).collect();
                paired.sort_unstable();
                paired.dedup();
                if known && kinds && paired.len() == distinct.len() {
                    best = Some((shape, *count));
                }
            }
            if let Some((shape, count)) = best {
                for at in &new_at {
                    votes.entry(tokens[*at], FirstSightTable::new()).add(shape[*at] & !KIND_MARK, count);
                }
            }
        }
        let mut found: Vec<(u32, u32)> = votes.rows().iter().filter_map(|(word, kinds)| kinds.first_most().map(|(kind, _)| (*word, *kind))).collect();
        found.sort_unstable();
        found
    }

    pub fn new_runs(&self, texts: &[Vec<u32>], extra: &[(u32, u32)]) -> Vec<(Vec<u32>, u32)> {
        let mut found: Vec<(Vec<u32>, u32)> = Vec::new();
        for tokens in texts {
            let new_at: Vec<usize> = (0..tokens.len()).filter(|at| self.is_new(tokens[*at], extra)).collect();
            let (Some(first_new), Some(last_new)) = (new_at.first(), new_at.last()) else { continue };
            let count = tokens.len();
            let shape: Vec<u32> = tokens.iter().map(|token| self.entry_of(*token, extra)).collect();
            let mut best: Option<(u32, Vec<u32>, u32)> = None;
            for from in 0..=*first_new {
                for to in (from + RUN_LEAST).max(last_new + 1)..=count.min(from + STATE_RUN) {
                    let Some(shapes) = self.tables.shapes.get(&(count - (to - from) + 1)) else { continue };
                    for kind in 0..self.tables.kind_names.len() as u32 {
                        let mut probe: Vec<u32> = shape[..from].to_vec();
                        probe.push(KIND_MARK | kind);
                        probe.extend_from_slice(&shape[to..]);
                        if let Some(seen) = shapes.get(&probe) {
                            if best.as_ref().is_none_or(|(top, _, _)| *seen > *top) {
                                best = Some((*seen, tokens[from..to].to_vec(), kind));
                            }
                        }
                    }
                }
            }
            if let Some((_, run, kind)) = best {
                match found.iter_mut().find(|(held, _)| *held == run) {
                    Some(entry) => entry.1 = kind,
                    None => found.push((run, kind)),
                }
            }
        }
        found
    }

    fn fit_groups(&self, tokens: &[u32], shape: &[u32], extra: &[(u32, u32)]) -> Option<Vec<(usize, usize, u32)>> {
        let mut memo: CheapMap<(usize, usize), Option<Vec<(usize, usize, u32)>>> = CheapMap::default();
        self.fit_from(tokens, shape, extra, 0, 0, &mut memo)
    }

    #[allow(clippy::too_many_arguments)]
    fn fit_from(&self, tokens: &[u32], shape: &[u32], extra: &[(u32, u32)], at: usize, place: usize, memo: &mut CheapMap<(usize, usize), Option<Vec<(usize, usize, u32)>>>) -> Option<Vec<(usize, usize, u32)>> {
        let (count, size) = (tokens.len(), shape.len());
        if place == size {
            return (at == count).then(Vec::new);
        }
        if at == count {
            return None;
        }
        if let Some(found) = memo.get(&(at, place)) {
            return found.clone();
        }
        let token = tokens[at];
        let want = shape[place];
        let new = self.is_new(token, extra);
        let mut out = None;
        if want & KIND_MARK != 0 {
            if !new {
                let own = match new_kind(extra, token) {
                    Some(kind) => Some(KIND_MARK | kind),
                    None => self.tables.kind.get(&token).map(|kind| KIND_MARK | kind),
                };
                if own == Some(want) {
                    out = self.fit_from(tokens, shape, extra, at + 1, place + 1, memo);
                }
            } else {
                let longest = self.longest.get(&(want & !KIND_MARK)).copied().unwrap_or(1);
                for length in 1..=(count - at - (size - place - 1)).min(longest) {
                    if self.is_new(tokens[at + length - 1], extra) {
                        if let Some(rest) = self.fit_from(tokens, shape, extra, at + length, place + 1, memo) {
                            let mut found = vec![(at, at + length, want & !KIND_MARK)];
                            found.extend(rest);
                            out = Some(found);
                            break;
                        }
                    }
                }
            }
        } else if !new && token == want {
            out = self.fit_from(tokens, shape, extra, at + 1, place + 1, memo);
        }
        memo.insert((at, place), out.clone());
        out
    }

    fn fit_new<N: WordTexts>(&self, names: &mut N, norms: &[Vec<u32>], extra: &mut Vec<(u32, u32)>, runs: &mut HashSet<Vec<u32>>) {
        for _ in 0..FIT_ROUNDS {
            let mut found = false;
            let mut waiting: Vec<(usize, Vec<u32>)> = Vec::new();
            for norm in norms {
                let tokens = unit_tokens_with(names, &self.tables.runs, runs, norm);
                let new = tokens.iter().filter(|token| self.is_new(**token, extra)).count();
                if new > 0 {
                    waiting.push((new, tokens));
                }
            }
            waiting.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.iter().map(|token| names.name(*token)).cmp(b.1.iter().map(|token| names.name(*token)))));
            for (_, tokens) in &waiting {
                let mut best: Option<(u32, Vec<(usize, usize, u32)>)> = None;
                for (shape, count) in &self.tested {
                    if shape.len() <= tokens.len() && best.as_ref().is_none_or(|(top, _)| *count > *top) {
                        if let Some(groups) = self.fit_groups(tokens, shape, extra) {
                            if !groups.is_empty() {
                                best = Some((*count, groups));
                            }
                        }
                    }
                }
                let Some((_, groups)) = best else { continue };
                found = true;
                for (from, to, kind) in groups {
                    let mut parts: Vec<u32> = Vec::new();
                    for token in &tokens[from..to] {
                        let text = names.name(*token).to_string();
                        for word in text.split(JOINT) {
                            parts.push(names.number(word));
                        }
                    }
                    let token = crate::tools::state::split::token_of(names, &parts);
                    if parts.len() > 1 {
                        runs.insert(parts);
                    }
                    set_new_kind(extra, token, kind, false);
                }
            }
            if !found {
                break;
            }
        }
    }

    pub fn case<N: WordTexts, F: FormIds>(&self, names: &mut N, forms: &mut F, sentences: &[String], question: &str) -> ReadCase {
        let none: HashSet<Vec<u32>> = HashSet::new();
        let q_raw: Vec<u32> = words(question).iter().map(|word| names.number(word)).collect();
        let q_norm = self.norm(&q_raw);
        let mut cuts: Vec<Cut> = sentences.iter().map(|sentence| self.cut_of(names, sentence, &[])).collect();
        let mut units = Vec::new();
        for found in &cuts {
            units.extend(self.units(names, forms, found, &[], &none));
        }
        let unit_q = self.unit(names, forms, &(q_raw.clone(), q_norm.clone()), &[], &none);
        let known = |tokens: &[u32]| tokens.iter().all(|token| self.tables.vocab.contains(token));
        if units.iter().all(|unit| known(&unit.tokens)) && known(&unit_q.tokens) {
            return ReadCase { units, question: unit_q };
        }
        let mut read: Vec<Vec<u32>> = sentences.iter().map(|sentence| self.first.read(names, &words(sentence))).collect();
        read.push(self.first.read(names, &words(question)));
        let extra1 = self.first.new_kinds(&read, &[]);
        if !extra1.is_empty() {
            cuts = sentences.iter().map(|sentence| self.cut_of(names, sentence, &extra1)).collect();
        }
        let mut texts: Vec<Vec<u32>> = Vec::new();
        let mut norms: Vec<Vec<u32>> = Vec::new();
        for found in &cuts {
            for piece in pieces_of(found) {
                texts.push(unit_tokens_with(names, &self.tables.runs, &none, &piece.1));
                norms.push(piece.1.clone());
            }
        }
        texts.push(unit_tokens_with(names, &self.tables.runs, &none, &q_norm));
        norms.push(q_norm.clone());
        let mut extra2 = self.new_kinds(&texts);
        let mut runs: HashSet<Vec<u32>> = HashSet::new();
        for (run, kind) in self.new_runs(&texts, &extra2) {
            let mut parts: Vec<u32> = Vec::new();
            for token in &run {
                let text = names.name(*token).to_string();
                for word in text.split(JOINT) {
                    parts.push(names.number(word));
                }
            }
            let token = crate::tools::state::split::token_of(names, &parts);
            runs.insert(parts);
            set_new_kind(&mut extra2, token, kind, true);
        }
        self.fit_new(names, &norms, &mut extra2, &mut runs);
        let mut units = Vec::new();
        for found in &cuts {
            units.extend(self.units(names, forms, found, &extra2, &runs));
        }
        let question = self.unit(names, forms, &(q_raw, q_norm), &extra2, &runs);
        ReadCase { units, question }
    }
}

const RUN_LEAST: usize = 2;
because!(RUN_LEAST, FormFit, "a run of new words holds more than one token, else it is one word that takes its kind by its place");
