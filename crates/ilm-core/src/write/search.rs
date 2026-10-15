use crate::base::keys::NO_NUMBER;
use crate::base::sums::compensated;
use crate::write::desk::{MatchedRuns, UsableSentences, WRITE_CHOICE_RUN, WRITE_RARE_COST, WritingDesk};
use crate::write::forms::WRITE_HEAD_AND_NEXT;
use crate::write::lessons::{GivenFacts, WRITE_EMPTY_VALUE, slot_bit};
use crate::write::nearest::{RunShares, WRITE_RUN_WEIGHT, WRITE_SHARE_FLOOR};
use crate::base::first_sight::{FirstSightTable, NumberTable};
use patterns::{because, source};

pub struct BeamSearch;
source!(BeamSearch, "sequences of sentences are searched with a beam, as Lowerre describes it for the search of speech: for each set of said slots only the best few sequences are kept, and each step adds a sentence that says a new slot");

pub const WRITE_SENTENCES_MOST: usize = 4;
because!(WRITE_SENTENCES_MOST, BeamSearch, "a text of a person in the lessons has a few sentences; a search that may add sentences without end would say a fact again and again");

pub const WRITE_TOP_ROWS: usize = 4;
because!(WRITE_TOP_ROWS, BeamSearch, "the filled forms of one set of slots that are tried in one place of a text when the whole text is counted");

pub const WRITE_BEAM_KEPT: usize = 3;
because!(WRITE_BEAM_KEPT, BeamSearch, "the sequences of sentences that are kept for one set of said slots in the search");

pub const WRITE_EXACT_KEPT: usize = 3;
because!(WRITE_EXACT_KEPT, BeamSearch, "the sequences with one number of sentences whose whole text is counted for the choice");

impl<'a> WritingDesk<'a> {
    fn largest(entries: &[(f64, Vec<u32>)], most: usize) -> Vec<(f64, Vec<u32>)> {
        let mut order: Vec<usize> = (0..entries.len()).collect();
        order.sort_by(|one, other| entries[*other].0.partial_cmp(&entries[*one].0).unwrap_or(std::cmp::Ordering::Equal));
        order.into_iter().take(most).map(|at| entries[at].clone()).collect()
    }

    fn whole(&mut self, counted: &mut NumberTable<Vec<u32>, (f64, MatchedRuns)>, sequence: &[u32], shares: &RunShares) -> (f64, MatchedRuns) {
        if let Some(known) = counted.get(sequence) {
            return *known;
        }
        let lessons = self.filler.lessons;
        let mut toks: Vec<u32> = vec![lessons.start];
        for line in sequence {
            toks.extend_from_slice(&self.lines[*line as usize].toks);
        }
        toks.push(lessons.end);
        let (value, parts) = WritingDesk::score_text(&toks, shares);
        let cost: i64 = sequence.iter().map(|line| self.lines[*line as usize].cost).sum();
        let scored = (value - WRITE_RARE_COST * (cost as f64), parts);
        counted.insert(sequence.to_vec(), scored);
        scored
    }

    pub(crate) fn candidates(&mut self, given: &GivenFacts, names: &[(u32, Vec<u32>)], strict: u32, part: bool, loose: u32, more: usize, lost: Option<u64>) -> Option<Vec<(u32, Vec<u32>, Vec<(u32, u32)>)>> {
        let reader = self.filler.reader;
        let forms = self.filler.forms;
        let want = given.mask();
        let part_key: Vec<u32> = reader.carried_order.iter().map(|slot| given.value(*slot).unwrap_or(NO_NUMBER)).collect();
        let mut free = forms.again | reader.carried_in.get(&part_key).copied().unwrap_or(0);
        for fact in &given.facts {
            if fact.value == WRITE_EMPTY_VALUE || reader.carried.get(fact.slot as usize).copied().unwrap_or(false) {
                free |= slot_bit(fact.slot);
            }
        }
        if self.shares.is_none() {
            let made = self.nearest_runs(given);
            self.shares = Some(made);
        }
        let (shares, length) = self.shares.take().unwrap_or_default();
        let changed = match lost {
            Some(lost) if lost != 0 => self.changed_forms(given, names, lost, loose),
            _ => Vec::new(),
        };
        self.lines.clear();
        let mut found = self.usable(given, names, &shares, strict, loose, &changed);
        let result = self.choose(&mut found, &shares, length, want, free, part, more);
        self.shares = Some((shares, length));
        result.map(|(_, sequence)| sequence.iter().map(|line| &self.lines[*line as usize]).map(|line| (line.form, line.key.clone(), line.items_at.clone())).collect())
    }

    fn choose(&mut self, found: &mut UsableSentences, shares: &RunShares, length: f64, want: u64, free: u64, part: bool, more: usize) -> Option<(f64, Vec<u32>)> {
        if found.first.rows().is_empty() {
            return None;
        }
        let rows: Vec<u32> = found.first.rows().iter().chain(found.later.rows().iter()).flat_map(|(_, rows)| rows.iter().copied()).collect();
        let mut levels: [(f64, f64); WRITE_CHOICE_RUN] = [(f64::default(), f64::default()); WRITE_CHOICE_RUN];
        for (size, entry) in levels.iter_mut().enumerate() {
            let got = compensated(rows.iter().map(|row| self.lines[*row as usize].matched[size].0));
            let every: i64 = rows.iter().map(|row| self.lines[*row as usize].matched[size].1).sum();
            let share = got / (every.max(1) as f64);
            let floor = if WRITE_SHARE_FLOOR >= share { WRITE_SHARE_FLOOR } else { share };
            *entry = (floor * length, length);
        }
        let mut best: Option<(f64, Vec<u32>, MatchedRuns)> = None;
        let mut counted: NumberTable<Vec<u32>, (f64, MatchedRuns)> = NumberTable::default();
        for _ in 0..WRITE_HEAD_AND_NEXT {
            let mut sequences: Vec<(f64, Vec<u32>)> = Vec::new();
            for (again, partly) in [(free, false), (want, part)] {
                sequences = self.search_at(found, want, again, &levels, partly, more);
                if !sequences.is_empty() {
                    break;
                }
            }
            if sequences.is_empty() {
                return None;
            }
            for (_, sequence) in sequences {
                let (value, parts) = self.whole(&mut counted, &sequence, shares);
                if best.as_ref().is_none_or(|(top, _, _)| value > *top) {
                    best = Some((value, sequence, parts));
                }
            }
            if let Some((_, _, parts)) = &best {
                for (size, entry) in levels.iter_mut().enumerate() {
                    let (got, every) = parts[size];
                    *entry = (if WRITE_SHARE_FLOOR >= got { WRITE_SHARE_FLOOR } else { got }, every.max(1) as f64);
                }
            }
        }
        let (mut value, mut sequence, _) = best?;
        for at in 0..sequence.len() {
            let table = if at == 0 { &found.first } else { &found.later };
            let others: Vec<u32> = table.get(&self.lines[sequence[at] as usize].said).map(|rows| rows.iter().take(WRITE_TOP_ROWS).copied().collect()).unwrap_or_default();
            for other in others {
                if other == sequence[at] {
                    continue;
                }
                let mut trial = sequence.clone();
                trial[at] = other;
                let (new, _) = self.whole(&mut counted, &trial, shares);
                if new > value {
                    sequence = trial;
                    value = new;
                }
            }
        }
        Some((value, sequence))
    }

    fn search_at(&mut self, found: &mut UsableSentences, want: u64, again: u64, levels: &[(f64, f64); WRITE_CHOICE_RUN], part: bool, more: usize) -> Vec<(f64, Vec<u32>)> {
        let lessons = self.filler.lessons;
        let reader = self.filler.reader;
        for at in 0..self.lines.len() {
            let line = &self.lines[at];
            let terms = line.matched.iter().zip(levels.iter()).map(|((got, every), (matched, all))| WRITE_RUN_WEIGHT * (*got - *matched / *all * (*every as f64)) / *matched);
            let gain = compensated(terms) - WRITE_RARE_COST * (line.cost as f64);
            self.lines[at].gain = gain;
        }
        let forms = self.filler.forms;
        let local = &self.local;
        let lines = &self.lines;
        for table in [&mut found.first, &mut found.later] {
            for (_, rows) in table.rows_mut() {
                rows.sort_by(|one, other| {
                    let (a, b) = (&lines[*one as usize], &lines[*other as usize]);
                    let (form_a, form_b) = (WritingDesk::form_of(forms, local, a.form), WritingDesk::form_of(forms, local, b.form));
                    b.gain.partial_cmp(&a.gain).unwrap_or(std::cmp::Ordering::Equal).then(form_b.count.cmp(&form_a.count)).then_with(|| lessons.compare_units(&form_a.key, &form_b.key))
                });
            }
        }
        let mut layer: FirstSightTable<u64, Vec<(f64, Vec<u32>)>> = FirstSightTable::new();
        for (said, rows) in found.first.rows() {
            layer.put(*said, vec![(lines[rows[0] as usize].gain, vec![rows[0]])]);
        }
        let mut out: Vec<(f64, Vec<u32>)> = Vec::new();
        let mut most: Option<(u32, f64, Vec<u32>)> = None;
        for _ in 0..WRITE_SENTENCES_MOST + more {
            if let Some(entries) = layer.get(&want) {
                out.extend(WritingDesk::largest(entries, WRITE_EXACT_KEPT));
            }
            for (covered, entries) in layer.rows() {
                for (value, sequence) in entries {
                    let size = covered.count_ones();
                    if most.as_ref().is_none_or(|(top_size, top_value, _)| size > *top_size || (size == *top_size && *value > *top_value)) {
                        most = Some((size, *value, sequence.clone()));
                    }
                }
            }
            let mut wider: FirstSightTable<u64, Vec<(f64, Vec<u32>)>> = FirstSightTable::new();
            for (covered, entries) in layer.rows() {
                if *covered == want {
                    continue;
                }
                for (said, rows) in found.later.rows() {
                    if said & !covered == 0 || (said & covered) & !again != 0 {
                        continue;
                    }
                    let row = &lines[rows[0] as usize];
                    if row.other && reader.plain_needs & !covered != 0 {
                        continue;
                    }
                    let longer = wider.at(covered | said);
                    for (value, sequence) in entries {
                        let mut next = sequence.clone();
                        next.push(rows[0]);
                        longer.push((*value + row.gain, next));
                    }
                }
            }
            let mut next: FirstSightTable<u64, Vec<(f64, Vec<u32>)>> = FirstSightTable::new();
            for (covered, entries) in wider.rows() {
                next.put(*covered, WritingDesk::largest(entries, WRITE_BEAM_KEPT));
            }
            layer = next;
        }
        if !out.is_empty() {
            return out;
        }
        match most {
            Some((_, value, sequence)) if part => vec![(value, sequence)],
            _ => Vec::new(),
        }
    }
}
