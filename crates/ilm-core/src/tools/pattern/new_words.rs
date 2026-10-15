use crate::tools::pattern::classes::KIND_MARK;
use crate::tools::pattern::kinds::{ENDS_NAMES, IN_LESSONS, MOST_RUN, PAIR_RUN, PatternWords, STARTS_NAMES, join_with, joined_text, new_kind};
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::{WordTexts, UnseenWords};
use patterns::because;
use std::collections::HashMap;

impl PatternWords {
    pub fn lesson_form(&self, tokens: &[u32], known: &[(u32, u32)]) -> Option<(&[u32], u32)> {
        let is_known = |word: u32| new_kind(known, word).is_some();
        let new_at: Vec<usize> = (0..tokens.len()).filter(|at| !self.has(tokens[*at], IN_LESSONS) && !is_known(tokens[*at])).collect();
        let shapes = self.shapes.get(&tokens.len())?;
        if new_at.is_empty() && !tokens.iter().any(|word| is_known(*word)) {
            return None;
        }
        let mut new_words: Vec<u32> = new_at.iter().map(|at| tokens[*at]).collect();
        new_words.sort_unstable();
        new_words.dedup();
        let mut best: Option<(&[u32], u32)> = None;
        for (shape, count) in shapes.rows() {
            if best.is_some_and(|(_, top)| *count <= top) {
                continue;
            }
            let fits = (0..tokens.len()).all(|at| if new_at.contains(&at) { shape[at] & KIND_MARK != 0 } else { shape[at] == self.shape_at(tokens[at], known) });
            if !fits {
                continue;
            }
            let mut paired: Vec<(u32, u32)> = new_at.iter().map(|at| (tokens[*at], shape[*at])).collect();
            paired.sort_unstable();
            paired.dedup();
            if paired.len() == new_words.len() {
                best = Some((shape.as_slice(), *count));
            }
        }
        best
    }

    pub fn new_kinds(&self, texts: &[Vec<u32>], known: &[(u32, u32)]) -> Vec<(u32, u32)> {
        let mut votes: FirstSightTable<u32, FirstSightTable<u32, u32>> = FirstSightTable::new();
        for tokens in texts {
            if let Some((shape, count)) = self.lesson_form(tokens, known) {
                for (at, word) in tokens.iter().enumerate() {
                    if !self.has(*word, IN_LESSONS) && new_kind(known, *word).is_none() {
                        votes.entry(*word, FirstSightTable::new()).add(shape[at] & !KIND_MARK, count);
                    }
                }
            }
        }
        let mut found: Vec<(u32, u32)> = votes.rows().iter().filter_map(|(word, kinds)| kinds.first_most().map(|(kind, _)| (*word, *kind))).collect();
        found.sort_unstable();
        found
    }

    pub fn new_runs(&self, tokens: &[u32], known: &[(u32, u32)]) -> Vec<(usize, usize)> {
        let is_new = |word: u32| !self.has(word, IN_LESSONS) && new_kind(known, word).is_none();
        let mut runs: Vec<(usize, usize)> = Vec::new();
        let mut at = 0;
        while at < tokens.len() {
            let mut end = at;
            while end < tokens.len() && is_new(tokens[end]) && end - at < MOST_RUN {
                end += 1;
            }
            let (mut from, mut to) = (at, end);
            if end > at {
                if at > 0 && self.has(tokens[at - 1], STARTS_NAMES) && runs.last().is_none_or(|(_, last)| *last < at) {
                    from = at - 1;
                }
                if end < tokens.len() && self.has(tokens[end], ENDS_NAMES) {
                    to = end + 1;
                }
            }
            if to - from >= PAIR_RUN {
                runs.push((from, to));
            }
            at = to.max(at + 1);
        }
        runs
    }

    pub fn new_names<N: WordTexts>(&self, names: &mut N, texts: Vec<Vec<u32>>) -> Vec<Vec<u32>> {
        let known = self.new_kinds(&texts, &[]);
        let mut found: HashMap<Vec<u32>, u32> = HashMap::new();
        for tokens in &texts {
            if self.lesson_form(tokens, &known).is_some() {
                continue;
            }
            let runs = self.new_runs(tokens, &known);
            if runs.is_empty() {
                continue;
            }
            let mut cuts: Vec<usize> = runs.iter().map(|(_, to)| *to).collect();
            loop {
                let mut names_now: Vec<(Vec<u32>, u32)> = Vec::new();
                let mut joined: Vec<u32> = Vec::new();
                let mut at = 0;
                for (from, to) in cut_pieces(&runs, &cuts) {
                    joined.extend_from_slice(&tokens[at..from]);
                    let text = joined_text(names, &tokens[from..to]);
                    let number = names.number(&text);
                    joined.push(number);
                    names_now.push((tokens[from..to].to_vec(), number));
                    at = to;
                }
                joined.extend_from_slice(&tokens[at..]);
                if self.lesson_form(&joined, &known).is_some() {
                    found.extend(names_now.into_iter().filter(|(run, _)| run.len() >= PAIR_RUN));
                    break;
                }
                if !next_cuts(&runs, &mut cuts) {
                    break;
                }
            }
        }
        if found.is_empty() {
            return texts;
        }
        texts.iter().map(|tokens| join_with(tokens, &found)).collect()
    }
}

pub fn cut_pieces(runs: &[(usize, usize)], cuts: &[usize]) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for ((from, to), cut) in runs.iter().zip(cuts) {
        if from < cut && cut < to {
            out.push((*from, *cut));
            out.push((*cut, *to));
        } else {
            out.push((*from, *to));
        }
    }
    out
}
because!(cut_pieces, UnseenWords, "the runs of new words of a text as the names that a choice of cuts makes of them: a run with no cut is one name, a run with a cut inside is the name before the cut and the name after it");

pub fn next_cuts(runs: &[(usize, usize)], cuts: &mut [usize]) -> bool {
    for at in (0..runs.len()).rev() {
        let (from, to) = runs[at];
        let next = if cuts[at] == to { from + 1 } else { cuts[at] + 1 };
        if next < to {
            cuts[at] = next;
            return true;
        }
        cuts[at] = to;
    }
    false
}
because!(next_cuts, UnseenWords, "the next choice of cuts of the runs of new words: for each run first no cut, then a cut at each place from its start on; the last run changes fastest, so the choices come in one fixed order and the first one that gives a lesson form is the same in every run");
