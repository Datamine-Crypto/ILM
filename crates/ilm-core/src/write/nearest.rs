use crate::base::keys::{NO_NUMBER, padded};
use crate::base::sums::compensated;
use crate::write::desk::{ChoiceRun, MatchedRuns, WRITE_CHOICE_RUN, WritingDesk};
use crate::write::forms::{WRITE_HEAD_AND_NEXT, WRITE_ONLY_ITEM};
use crate::write::lessons::{GivenFacts, WRITE_EMPTY_VALUE, slot_bit};
use crate::write::numbers::{natural_log, to_power};
use crate::base::first_sight::{FirstSightTable, NumberTable};
use patterns::{because, source};

pub struct NearestRuns;
source!(NearestRuns, "the texts of the lessons with the nearest lists of facts say how people write such a list: the share of a run of tokens is the chance that one of a few such texts holds it, with the given values in the place of theirs");

pub const WRITE_RUN_WEIGHT: f64 = 0.25;
because!(WRITE_RUN_WEIGHT, NearestRuns, "the weight of the runs of one length in the count of a text: each of the lengths counts the same");

pub const WRITE_SHARE_FLOOR: f64 = 0.01;
because!(WRITE_SHARE_FLOOR, NearestRuns, "what is added to a matched count and to the count of all runs before the logarithm is taken, and the least level of a text, so that a text with no matched run of one length still has a count");

pub const WRITE_NEAREST_LESSONS: usize = 40;
because!(WRITE_NEAREST_LESSONS, NearestRuns, "the fewest lessons with the nearest lists of facts in whose texts the choice counts runs; with many lessons the choice counts the square root of their number, as a vote of nearest neighbours grows with its data, since more near texts show more of the ways in which persons write a list (on the part for development of E2E, all lessons: the texts of the fewest lessons give fewer of the runs that persons use than those of the square root of all lessons)");

pub const WRITE_PERSONS: f64 = 5.0;
because!(WRITE_PERSONS, NearestRuns, "the texts by persons that a written text is expected to be compared with; the share of a run is the chance that one of them holds it");

pub const WRITE_RUNS_OF_TEXT: usize = 128;
because!(WRITE_RUNS_OF_TEXT, NearestRuns, "the room for the runs of one near text that the table of shares takes at once: a lesson text of a few sentences has about this many runs of one to a few tokens, so the table rarely grows while it is filled");

pub const WRITE_FAR_STEP: f64 = 0.5;
because!(WRITE_FAR_STEP, NearestRuns, "the weight that a lesson loses for each slot or bound value by which its list of facts differs from the given one");

#[derive(Clone, Debug, Default)]
pub struct RunLevels {
    pub first: f64,
    pub more: Vec<f64>,
}
because!(RunLevels, NearestRuns, "the shares of one run of tokens: the share for the first time that a text holds it, kept in place, and the shares for a second time and later, which few runs have; so most runs of the near texts take no room of their own. take gives the first so many shares in their order");

impl RunLevels {
    pub fn take(&self, many: usize) -> impl Iterator<Item = f64> + '_ {
        std::iter::once(self.first).chain(self.more.iter().copied()).take(many)
    }

    fn levels_mut(&mut self) -> impl Iterator<Item = &mut f64> {
        std::iter::once(&mut self.first).chain(self.more.iter_mut())
    }
}

pub type RunShares = NumberTable<ChoiceRun, RunLevels>;
because!(RunShares, NearestRuns, "for each run of tokens of the near texts its shares, from the first time on");

impl<'a> WritingDesk<'a> {
    pub(crate) fn nearest_runs(&mut self, given: &GivenFacts) -> (RunShares, f64) {
        let lessons = self.filler.lessons;
        let reader = self.filler.reader;
        let forms = self.filler.forms;
        let want = given.mask();
        let tied: Vec<u32> = given.facts.iter().filter(|fact| fact.value == WRITE_EMPTY_VALUE || reader.carried.get(fact.slot as usize).copied().unwrap_or(false)).map(|fact| fact.slot).collect();
        let mut ranked: Vec<(usize, i64, u32, u32)> = Vec::with_capacity(lessons.rows.len());
        for (slots, rows) in forms.near.rows() {
            let differs = (want ^ slots).count_ones() as usize;
            for (place, row) in rows.iter().enumerate() {
                let held = |slot: u32| row.values.get(slot as usize).copied().filter(|value| *value != NO_NUMBER);
                let more = tied.iter().filter(|slot| slots & slot_bit(**slot) != 0 && held(**slot) != given.value(**slot)).count();
                let same = given.facts.iter().filter(|fact| slots & slot_bit(fact.slot) != 0 && held(fact.slot) == Some(fact.value)).count();
                ranked.push((differs + more, -(same as i64), row.lesson, place as u32));
            }
        }
        let most = WRITE_NEAREST_LESSONS.max((lessons.rows.len() as f64).sqrt() as usize);
        if ranked.len() > most {
            ranked.select_nth_unstable(most);
            ranked.truncate(most);
        }
        ranked.sort_unstable();
        let nearest: Vec<(usize, u32, u32)> = ranked.iter().map(|(far, _, lesson, place)| (*far, *lesson, *place)).collect();
        let mut weight: RunShares = NumberTable::with_capacity_and_hasher(most * WRITE_RUNS_OF_TEXT, Default::default());
        if nearest.is_empty() {
            return (weight, f64::default());
        }
        let mut every = f64::default();
        let mut length = f64::default();
        let mut runs: Vec<ChoiceRun> = Vec::new();
        for (far, number, _) in nearest {
            let lesson = &lessons.rows[number as usize];
            let have = &lesson.given;
            let weigh = to_power(WRITE_FAR_STEP, far as f64);
            every += weigh;
            let mut toks: Vec<u32> = Vec::with_capacity(lesson.units.len() + WRITE_HEAD_AND_NEXT);
            toks.push(lessons.start);
            toks.extend(lesson.units.iter().map(|unit| unit.sym));
            toks.push(lessons.end);
            for fact in &given.facts {
                let slot = fact.slot;
                let Some(was) = have.value(slot) else {
                    continue;
                };
                if lessons.is_name(slot) || lessons.is_list(slot) || tied.contains(&slot) || was == WRITE_EMPTY_VALUE || was == fact.value {
                    continue;
                }
                let old = self.filler.value_tokens(was).to_vec();
                let new = self.filler.value_tokens(fact.value).to_vec();
                let mut out: Vec<u32> = Vec::with_capacity(toks.len() + new.len());
                let mut at = 0usize;
                while at < toks.len() {
                    if !old.is_empty() && !out.is_empty() && toks[at..].starts_with(&old) {
                        let after = toks.get(at + old.len()).copied().unwrap_or(lessons.end);
                        let before = out[out.len() - 1];
                        if self.filler.fit_at(slot, fact.value, before, after, WRITE_ONLY_ITEM) {
                            out.extend_from_slice(&new);
                            at += old.len();
                            continue;
                        }
                        let earlier = if out.len() > 1 { out[out.len() - WRITE_HEAD_AND_NEXT] } else { lessons.start };
                        let other = self.filler.other_lead(slot, fact.value, earlier, before);
                        if other != NO_NUMBER && self.filler.fit_at(slot, fact.value, other, after, WRITE_ONLY_ITEM) {
                            let last = out.len() - 1;
                            out[last] = other;
                            out.extend_from_slice(&new);
                            at += old.len();
                            continue;
                        }
                    }
                    out.push(toks[at]);
                    at += 1;
                }
                toks = out;
            }
            length += weigh * (toks.len() as f64);
            runs.clear();
            for size in 1..=WRITE_CHOICE_RUN {
                runs.extend(toks.windows(size).map(padded::<WRITE_CHOICE_RUN>));
            }
            runs.sort_unstable();
            let mut at = 0usize;
            while at < runs.len() {
                let mut end = at + 1;
                while end < runs.len() && runs[end] == runs[at] {
                    end += 1;
                }
                let many = end - at;
                let levels = weight.entry(runs[at]).or_default();
                while levels.more.len() + 1 < many {
                    levels.more.push(f64::default());
                }
                for level in levels.levels_mut().take(many) {
                    *level += weigh;
                }
                at = end;
            }
        }
        let one = f64::from(true);
        for levels in weight.values_mut() {
            for level in levels.levels_mut() {
                *level = one - to_power(one - *level / every, WRITE_PERSONS);
            }
        }
        (weight, length / every)
    }

    pub(crate) fn matched(toks: &[u32], shares: &RunShares) -> MatchedRuns {
        let mut out: MatchedRuns = [(f64::default(), 0); WRITE_CHOICE_RUN];
        for size in 1..=WRITE_CHOICE_RUN {
            let mut got = f64::default();
            for run in toks.windows(size) {
                if let Some(levels) = shares.get(&padded(run)) {
                    got += levels.first;
                }
            }
            out[size - 1] = (got, (toks.len() + 1).saturating_sub(size) as i64);
        }
        out
    }

    pub(crate) fn score_text(toks: &[u32], shares: &RunShares) -> (f64, MatchedRuns) {
        let mut total = f64::default();
        let mut parts: MatchedRuns = [(f64::default(), 0); WRITE_CHOICE_RUN];
        for size in 1..=WRITE_CHOICE_RUN {
            let mut times: FirstSightTable<ChoiceRun, usize> = FirstSightTable::new();
            for run in toks.windows(size) {
                *times.at(padded(run)) += 1;
            }
            let mut got = f64::default();
            for (run, many) in times.rows() {
                if let Some(levels) = shares.get(run) {
                    got += compensated(levels.take(*many));
                }
            }
            let every = (toks.len() + 1).saturating_sub(size) as i64;
            parts[size - 1] = (got, every);
            total += WRITE_RUN_WEIGHT * natural_log((got + WRITE_SHARE_FLOOR) / ((every as f64) + WRITE_SHARE_FLOOR));
        }
        (total, parts)
    }
}
