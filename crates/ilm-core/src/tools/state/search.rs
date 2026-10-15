use crate::base::lanes::{on_lanes, shards};
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::kinds::PatternWords;
use crate::base::first_sight::FirstSightTable;
use crate::tools::state::choices::{compile, sets};
use crate::tools::state::read::{Codes, Name, StateOut, StateReading, read};
use crate::tools::state::table::{Coded, Holder, Key, OpChoice, Prog, StoryRun, names_thing};
use patterns::{because, source};
use std::collections::HashSet;
use std::sync::Arc;

pub struct ProgramSearch;
source!(ProgramSearch, "a program is found by local search over whole cases, in the way of hill climbing from many starts: a start is one form with a set and a reading, the climb changes one form at a time and keeps a change that makes more lessons right by more than chance, and the final answer of each case is the only judge");

pub struct ParallelTrials;
source!(ParallelTrials, "the trials of a search that do not depend on each other, the programs of one group of starts and the choices of one entry, are run on all threads, and their results and their work are taken in their order afterwards, so the search stops at the same trial and keeps the same choice for every number of threads");

pub const LEAST_GAIN: f64 = 2.0;
because!(LEAST_GAIN, ProgramSearch, "a change stays only when it makes this many more cases right beyond the gain of chance");

pub const MOST_PAIR_FORMS: usize = 8;
because!(MOST_PAIR_FORMS, ProgramSearch, "the start with two forms is searched among this many forms, the forms in most cases first: the pairs grow with the square of the forms");

pub const MOST_PAIR_CASES: usize = 150;
because!(MOST_PAIR_CASES, ProgramSearch, "a start with two forms is counted on this many cases that hold the forms: enough to see a start, and a bound on the work of the many pairs");

pub const MOST_STARTS: usize = 6;
because!(MOST_STARTS, ProgramSearch, "the search climbs from this many starts, the best ones, and the best end stays");

pub const MOST_WORK: u64 = 800_000_000;
because!(MOST_WORK, ProgramSearch, "the search of a model stops after this much work, counted and not timed, so that the same lessons give the same model on every machine; a few minutes of work at most on a desktop machine");

pub const START_WORK: u64 = 6;
because!(START_WORK, ProgramSearch, "the work of scoring one reading of one case for a start, beside the work of running the case");

pub const SHARE_SCALE: usize = 4;
because!(SHARE_SCALE, ProgramSearch, "the work of reading a table grows with a quarter of its size, the part of it that a reading looks at");

pub const SPLIT_SPREAD: f64 = 0.5;
because!(SPLIT_SPREAD, ProgramSearch, "each word that joins a split adds half a standard deviation of a chance change of the answers that it changes to the gain that the split must reach: a choice among many words finds some gain by chance");

pub const TABLE_SPREADS: f64 = 2.0;
because!(TABLE_SPREADS, ProgramSearch, "a table of answers for results that are no words can fit by chance; a change then needs a gain of this many standard deviations");

const ONE_SPREAD: f64 = 1.0;
because!(ONE_SPREAD, ProgramSearch, "a change of answers with no table of results that are no words needs a gain of one standard deviation of a chance change");

const TABLE_LEAST: usize = 2;
because!(TABLE_LEAST, ProgramSearch, "a table of answers for results that are no words needs at least this many different results to be able to fit by chance");

pub const SURE_CASES: usize = 20;
because!(SURE_CASES, ProgramSearch, "a program is a rule only on this many cases or more, and a start that holds the true answers in the table needs this many cases");

pub const SURE_SHARE_SEARCH: f64 = 0.98;
because!(SURE_SHARE_SEARCH, ProgramSearch, "the share of the cases that a rule must get right, and the share of the cases whose true answer a start must hold in the table");

const TWO_BEST: usize = 2;
because!(TWO_BEST, ProgramSearch, "a group of starts gives the starts of its best readings, as many as this: the second best reading is often the one that a climb makes right");

const CASE_RUNS: usize = 4;
because!(CASE_RUNS, ParallelTrials, "the runs of cases for each thread when many cases are run at once: each run keeps its tables from case to case, and with several runs for each thread a thread with long stories gives its other runs to the threads that are done");

const PARALLEL_CASES: usize = 64;
because!(PARALLEL_CASES, ParallelTrials, "below this many cases a run of the cases is done on one thread: starting the threads costs more than it saves");

pub const BATCH_TRIALS: usize = 4;
because!(BATCH_TRIALS, ParallelTrials, "the trials that are run at once for each thread before the search looks whether its work is used up: enough to keep every thread busy, and few trials run in vain after the end");

pub struct StateRow {
    pub units: Vec<Coded>,
    pub q: Coded,
    pub truth: u32,
}
because!(StateRow, ProgramSearch, "one lesson as the search reads it: its units with the numbers of their spans, its question, and the number of the words of its true answer");

#[derive(Clone)]
pub struct Results {
    pub out: Vec<Vec<StateOut>>,
    pub held: Vec<bool>,
}
because!(Results, ProgramSearch, "for each reading the result of each case, and for each case whether its true answer stands in the table");

pub struct StateStart {
    pub score: usize,
    pub reading: usize,
    pub prog: Prog,
    pub results: Vec<StateOut>,
}
because!(StateStart, ProgramSearch, "a start of the climb: the cases right, the reading, the program, and the results of the reading");

pub struct End {
    pub score: usize,
    pub reading: usize,
    pub prog: Prog,
    pub results: Vec<StateOut>,
}
because!(End, ProgramSearch, "the end of a climb: the cases right, the reading, the program, and the results");

pub struct CaseRun {
    pub out: Vec<StateOut>,
    pub held: bool,
    pub work: u64,
}
because!(CaseRun, ProgramSearch, "one case run under a program: the result of each reading, whether its true answer stands in the table, and the work");

type StateScored = (bool, Vec<i64>, Option<Vec<i64>>);
because!(StateScored, ParallelTrials, "one reading of a trial: whether it passes the margin, its score, and its count of answers of lessons when the table holds the true answers");

struct Trial {
    work: u64,
    held: usize,
    readings: Vec<StateScored>,
}
because!(Trial, ParallelTrials, "one program of a group of starts, tried on the cases of the group: its work, the cases whose true answer stands in the table, and each reading as it is scored");

type Seen = (Vec<i64>, usize, Prog, Option<Vec<usize>>);
because!(Seen, ProgramSearch, "a start as it is ranked: its score, its reading, its program, and the cases of its group");

pub fn margin(results: &[StateOut], changed: Option<usize>) -> f64 {
    let mut others = 0usize;
    let mut distinct: Vec<&StateOut> = Vec::new();
    for result in results.iter().filter(|result| matches!(result, StateOut::Count(_) | StateOut::Truth(_))) {
        others += 1;
        if distinct.len() < TABLE_LEAST && !distinct.contains(&result) {
            distinct.push(result);
        }
    }
    let table = distinct.len() >= TABLE_LEAST;
    match changed {
        None => LEAST_GAIN + if table { TABLE_SPREADS * (others as f64).sqrt() } else { (results.len() as f64).sqrt() },
        Some(changed) => LEAST_GAIN + if table { TABLE_SPREADS } else { ONE_SPREAD } * (changed as f64).sqrt(),
    }
}
because!(margin, ProgramSearch, "the gain that a change must reach: a change of a number of answers makes some cases right by chance, so the gain must be above one standard deviation of that, or above more where a table of answers for results that are no words can also fit by chance; for a start the same over all its cases");

const TRUTH_CODES: u64 = 1 << u32::BITS;
because!(TRUTH_CODES, ProgramSearch, "the code of a result true or false when the results that are no words are put in order: above the code of every count, so that a count and a truth are never one row of the table of answers");

pub struct FormSearch<'a> {
    pub first: &'a PatternWords,
    pub codes: &'a Codes,
    pub shapes: &'a [Vec<u32>],
    pub spans: &'a CheapMap<u32, Vec<(u8, u8)>>,
    pub rows: Vec<&'a StateRow>,
    pub truths: Vec<u32>,
    pub readings: Vec<StateReading>,
    pub watch: Vec<(u8, u8)>,
    pub groups: Vec<(Vec<(u8, u8)>, Vec<(usize, StateOut)>)>,
    pub work: u64,
    pub in_table: bool,
    pub lanes: usize,
}
because!(FormSearch, ProgramSearch, ParallelTrials, "the search for one form of question: the cases of the search with their truths, the forms of units and the spans of answers, the readings with their groups of equal arguments and the results of each group on an empty table, the arguments whose history is watched, the count of work of the model, whether a start held the true answers in the table, and the threads");

impl FormSearch<'_> {
    pub fn late(&self) -> bool {
        self.work > MOST_WORK
    }

    pub fn score(&self, results: &[StateOut], truths: &[u32]) -> usize {
        let mut right = 0;
        let mut other: Vec<(u64, u32)> = Vec::new();
        for (result, truth) in results.iter().zip(truths) {
            match result {
                StateOut::Phrase(found) if *found == *truth => right += 1,
                StateOut::Phrase(_) | StateOut::Many(_) => {
                    if self.codes.normal_is(self.first, result, *truth) {
                        right += 1;
                    }
                }
                StateOut::Unknown => {}
                StateOut::Count(count) => other.push((u64::from(*count), *truth)),
                StateOut::Truth(said) => other.push((TRUTH_CODES | u64::from(*said), *truth)),
            }
        }
        other.sort_unstable();
        right + other.chunk_by(|one, next| one.0 == next.0).map(|result| result.chunk_by(|one, next| one == next).map(<[(u64, u32)]>::len).max().unwrap_or(0)).sum::<usize>()
    }

    pub fn table(&self, results: &[StateOut], truths: &[u32]) -> Vec<(StateOut, u32)> {
        let mut other: FirstSightTable<StateOut, FirstSightTable<u32, u32>> = FirstSightTable::new();
        for (result, truth) in results.iter().zip(truths) {
            if matches!(result, StateOut::Count(_) | StateOut::Truth(_)) {
                other.entry(result.clone(), FirstSightTable::new()).add(*truth, 1);
            }
        }
        other.rows().iter().filter_map(|(result, counts)| counts.first_most().map(|(truth, _)| (result.clone(), *truth))).collect()
    }

    pub fn put(&self, prog: &Prog, key: Key, choice: Option<OpChoice>) -> Prog {
        let form = match key {
            Key::Form(form) | Key::Word(form, _) => form,
        };
        prog.put(key, choice, Arc::new(compile(&self.shapes[form as usize], choice)))
    }

    pub fn case_run(&self, run: &mut StoryRun, prog: &Prog, case: usize, firsts: bool) -> CaseRun {
        let row = self.rows[case];
        let watch: Vec<u32> = self.watch.iter().map(|(from, to)| row.q.span(*from, *to)).collect();
        run.run(&row.units, prog, &watch, firsts);
        let StoryRun { state, hist, .. } = run;
        let mut work = state.work + self.groups.len() as u64;
        let size = 1 + (state.at.len() / SHARE_SCALE) as u64;
        let truth = row.truth;
        let held = state.raw.iter().any(|(_, raw)| *raw == truth || self.codes.normal[*raw as usize] == truth);
        let history: Vec<(u32, &[u32])> = watch.iter().zip(hist.iter()).map(|(key, values)| (*key, values.as_slice())).collect();
        let mut out: Vec<StateOut> = vec![StateOut::Unknown; self.readings.len()];
        for (args, members) in &self.groups {
            let named = args.iter().any(|(from, to)| names_thing(state, row.q.span(*from, *to)));
            let (from, to) = args[0];
            let first = row.q.span(from, to);
            let seen = self.watch.contains(&(from, to)) && history.iter().any(|(key, values)| *key == first && !values.is_empty());
            if named || (!history.is_empty() && seen) {
                for (reading, _) in members {
                    out[*reading] = read(&self.readings[*reading], &row.q, state, &history, self.codes);
                }
                work += members.len() as u64 * size;
            } else {
                for (reading, empty) in members {
                    out[*reading] = empty.clone();
                }
            }
        }
        CaseRun { out, held, work }
    }

    pub fn one_row(&self, run: &mut StoryRun, prog: &Prog, reading: usize, row: &StateRow) -> (StateOut, u64) {
        let found = &self.readings[reading];
        let watch: Vec<u32> = if found.name.history() { vec![row.q.span(found.args[0].0, found.args[0].1)] } else { Vec::new() };
        run.run(&row.units, prog, &watch, found.name.first_order());
        let work = run.state.work + (run.state.at.len() / SHARE_SCALE) as u64;
        let history: Vec<(u32, &[u32])> = watch.iter().zip(&run.hist).map(|(key, values)| (*key, values.as_slice())).collect();
        (read(found, &row.q, &run.state, &history, self.codes), work)
    }

    pub fn cases_run(&self, prog: &Prog, cases: &[usize], lanes: usize) -> Vec<CaseRun> {
        let firsts = self.readings.iter().any(|reading| reading.name.first_order());
        if lanes > 1 && cases.len() >= PARALLEL_CASES {
            let parts = shards(cases.len(), lanes * CASE_RUNS);
            let found: Vec<Vec<CaseRun>> = on_lanes(parts.len(), lanes, &|part| {
                let mut run = StoryRun::default();
                parts[part].clone().map(|place| self.case_run(&mut run, prog, cases[place], firsts)).collect()
            });
            found.into_iter().flatten().collect()
        } else {
            let mut run = StoryRun::default();
            cases.iter().map(|case| self.case_run(&mut run, prog, *case, firsts)).collect()
        }
    }

    pub fn ones(&self, prog: &Prog, reading: usize, cases: &[usize], lanes: usize) -> Vec<(StateOut, u64)> {
        let rows: Vec<&StateRow> = cases.iter().map(|case| self.rows[*case]).collect();
        self.rows_run(prog, reading, &rows, lanes)
    }

    pub fn rows_run(&self, prog: &Prog, reading: usize, rows: &[&StateRow], lanes: usize) -> Vec<(StateOut, u64)> {
        if lanes > 1 && rows.len() >= PARALLEL_CASES {
            let parts = shards(rows.len(), lanes * CASE_RUNS);
            let found: Vec<Vec<(StateOut, u64)>> = on_lanes(parts.len(), lanes, &|part| {
                let mut run = StoryRun::default();
                parts[part].clone().map(|place| self.one_row(&mut run, prog, reading, rows[place])).collect()
            });
            found.into_iter().flatten().collect()
        } else {
            let mut run = StoryRun::default();
            rows.iter().map(|row| self.one_row(&mut run, prog, reading, row)).collect()
        }
    }

    pub fn all_results(&mut self, prog: &Prog, only: Option<&[usize]>, old: Option<&Results>) -> Results {
        let count = self.rows.len();
        let (mut results, every): (Results, Vec<usize>) = match (only, old) {
            (Some(only), Some(old)) => (old.clone(), only.to_vec()),
            _ => (Results { out: vec![vec![StateOut::Unknown; count]; self.readings.len()], held: vec![false; count] }, (0..count).collect()),
        };
        let runs = self.cases_run(prog, &every, self.lanes);
        for (case, found) in every.iter().zip(runs) {
            self.work += found.work;
            results.held[*case] = found.held;
            for (reading, out) in found.out.into_iter().enumerate() {
                results.out[reading][*case] = out;
            }
        }
        results
    }

    pub fn one_result(&mut self, prog: &Prog, reading: usize, case: usize) -> StateOut {
        let (out, work) = self.one_row(&mut StoryRun::default(), prog, reading, self.rows[case]);
        self.work += work;
        out
    }

    pub fn results_of(&mut self, prog: &Prog, reading: usize, cases: &[usize]) -> Vec<StateOut> {
        let found = self.ones(prog, reading, cases, self.lanes);
        let mut out: Vec<StateOut> = Vec::with_capacity(found.len());
        for (result, work) in found {
            self.work += work;
            out.push(result);
        }
        out
    }

    pub fn results_rows(&mut self, prog: &Prog, reading: usize, rows: &[&StateRow]) -> Vec<StateOut> {
        let found: Vec<(StateOut, u64)> = self.rows_run(prog, reading, rows, self.lanes);
        let mut out: Vec<StateOut> = Vec::with_capacity(found.len());
        for (result, work) in found {
            self.work += work;
            out.push(result);
        }
        out
    }

    fn scored(&self, results: &[StateOut], reading: usize, mine: &[u32], least: i64, answers: Option<&HashSet<u32>>) -> StateScored {
        let found = &self.readings[reading];
        let width: i64 = found.args.iter().map(|(from, to)| i64::from(to - from)).sum();
        let length = -(found.args.len() as i64);
        let score = vec![self.score(results, mine) as i64, width, length];
        let passes = (score[0] - least) as f64 >= margin(results, None);
        let words = answers.map(|answers| {
            let named = results.iter().filter(|result| matches!(result, StateOut::Phrase(found) if answers.contains(found))).count() as i64;
            let right = results.iter().zip(mine).filter(|(result, truth)| matches!(result, StateOut::Phrase(found) if found == *truth)).count() as i64;
            vec![named, right, width, length]
        });
        (passes, score, words)
    }

    fn trial(&self, prog: &Prog, part: &[usize], mine: &[u32], least: i64, answers: &HashSet<u32>) -> Trial {
        let runs = self.cases_run(prog, part, 1);
        let mut work: u64 = runs.iter().map(|found| found.work).sum();
        work += START_WORK * part.len() as u64 * self.readings.len() as u64;
        let held = runs.iter().filter(|found| found.held).count();
        let table_held = part.len() >= SURE_CASES && held as f64 >= SURE_SHARE_SEARCH * part.len() as f64;
        let readings: Vec<StateScored> = (0..self.readings.len())
            .map(|reading| {
                let results: Vec<StateOut> = runs.iter().map(|run| run.out[reading].clone()).collect();
                self.scored(&results, reading, mine, least, table_held.then_some(answers))
            })
            .collect();
        Trial { work, held, readings }
    }

    fn plain_trial(&self, plain: &Results, part: &[usize], mine: &[u32], least: i64) -> Trial {
        let readings: Vec<StateScored> = (0..self.readings.len())
            .map(|reading| {
                let results: Vec<StateOut> = part.iter().map(|case| plain.out[reading][*case].clone()).collect();
                self.scored(&results, reading, mine, least, None)
            })
            .collect();
        Trial { work: START_WORK * part.len() as u64 * self.readings.len() as u64, held: part.iter().filter(|case| plain.held[**case]).count(), readings }
    }

    pub fn starts(&mut self, groups: Vec<(Option<Vec<usize>>, Vec<Prog>)>, most: Option<usize>, plain: &Results) -> (Vec<StateStart>, Vec<StateStart>) {
        let count = self.rows.len();
        let answers: HashSet<u32> = self.truths.iter().copied().collect();
        let mut gain: Vec<Seen> = Vec::new();
        let mut held: Vec<Seen> = Vec::new();
        for (rows, group) in groups {
            let mut best: FirstSightTable<Name, Seen> = FirstSightTable::new();
            let mut in_table: FirstSightTable<Name, Seen> = FirstSightTable::new();
            let all: Vec<usize> = rows.clone().unwrap_or_else(|| (0..count).collect());
            let part: Vec<usize> = match most {
                Some(most) => all.iter().copied().take(most).collect(),
                None => all.clone(),
            };
            let mine: Vec<u32> = part.iter().map(|case| self.truths[*case]).collect();
            let least = {
                let mut counted: FirstSightTable<u32, u32> = FirstSightTable::new();
                for truth in &mine {
                    counted.add(*truth, 1);
                }
                counted.first_most().map_or(0, |(_, count)| *count) as i64
            };
            let batch = self.lanes.max(1) * BATCH_TRIALS;
            let mut at = 0;
            'group: while at < group.len() {
                let end = (at + batch).min(group.len());
                let trials: Vec<Trial> = if rows.is_some() {
                    let this = &*self;
                    on_lanes(end - at, this.lanes, &|place| this.trial(&group[at + place], &part, &mine, least, &answers))
                } else {
                    (at..end).map(|_| self.plain_trial(plain, &part, &mine, least)).collect()
                };
                for (place, trial) in trials.into_iter().enumerate() {
                    if self.late() {
                        break 'group;
                    }
                    let prog = &group[at + place];
                    self.work += trial.work;
                    let table_held = rows.is_some() && part.len() >= SURE_CASES && trial.held as f64 >= SURE_SHARE_SEARCH * part.len() as f64;
                    for (reading, (passes, score, words)) in trial.readings.into_iter().enumerate() {
                        let name = self.readings[reading].name;
                        if passes && best.get(&name).is_none_or(|(top, _, _, _)| score > *top) {
                            best.put(name, (score, reading, prog.clone(), rows.clone()));
                        }
                        if table_held {
                            self.in_table = true;
                            if let Some(words) = words {
                                if words[0] > 0 && in_table.get(&name).is_none_or(|(top, _, _, _)| words > *top) {
                                    in_table.put(name, (words, reading, prog.clone(), rows.clone()));
                                }
                            }
                        }
                    }
                }
                at = end;
            }
            let mut ranked: Vec<Seen> = best.rows().iter().map(|(_, entry)| entry.clone()).collect();
            ranked.sort_by(|a, b| b.0.cmp(&a.0));
            gain.extend(ranked.into_iter().take(TWO_BEST));
            let mut ranked: Vec<Seen> = in_table.rows().iter().map(|(_, entry)| entry.clone()).collect();
            ranked.sort_by(|a, b| b.0.cmp(&a.0));
            held.extend(ranked.into_iter().take(TWO_BEST));
        }
        let truths = self.truths.clone();
        let mut out: Vec<Vec<StateStart>> = Vec::new();
        for mut found in [gain, held] {
            found.sort_by(|a, b| b.0.cmp(&a.0));
            let mut starts: Vec<StateStart> = Vec::new();
            for (_, reading, prog, rows) in found.into_iter().take(MOST_STARTS) {
                let results = match &rows {
                    Some(rows) => self.all_results(&prog, Some(rows), Some(plain)).out.swap_remove(reading),
                    None => plain.out[reading].clone(),
                };
                let score = self.score(&results, &truths);
                starts.push(StateStart { score, reading, prog, results });
            }
            out.push(starts);
        }
        let held = out.pop().unwrap_or_default();
        let gain = out.pop().unwrap_or_default();
        (gain, held)
    }

    pub fn sets_of(&self, form: u32) -> Vec<OpChoice> {
        sets(&self.shapes[form as usize], self.spans.get(&form).map_or(&[][..], Vec::as_slice))
    }

    pub fn set_programs(&self, prog: &Prog, form: u32) -> Vec<Prog> {
        self.sets_of(form).into_iter().map(|choice| self.put(prog, Key::Form(form), Some(choice))).collect()
    }

    pub fn pair_programs(&self, prog: &Prog, form: u32, other: u32) -> Vec<Prog> {
        let firsts: Vec<OpChoice> = self.sets_of(form).into_iter().filter(|choice| choice.first != Holder::Last).collect();
        let seconds: Vec<OpChoice> = self.sets_of(other).into_iter().filter(|choice| choice.first != Holder::Last).collect();
        let mut out: Vec<Prog> = Vec::with_capacity(firsts.len() * seconds.len());
        for choice in &firsts {
            let with = self.put(prog, Key::Form(form), Some(*choice));
            for second in &seconds {
                out.push(self.put(&with, Key::Form(other), Some(*second)));
            }
        }
        out
    }
}
