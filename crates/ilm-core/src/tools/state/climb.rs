use crate::base::lanes::on_lanes;
use crate::tools::pattern::names::WordTexts;
use crate::tools::state::choices::{choices, slots};
use crate::tools::state::read::StateOut;
use crate::tools::state::search::{BATCH_TRIALS, End, FormSearch, SPLIT_SPREAD, SURE_CASES, StateStart, margin};
use crate::tools::state::table::{Key, OpChoice, Prog};
use patterns::{because, source};

pub struct FormClimb;
source!(FormClimb, "the climb from a start: one form of unit at a time, the form in most cases first, its best operation stays when it gives more right answers than chance; a form can be split by the word in one of its slots; then the reading again; at the end an entry that changes no answer goes");

pub struct Better {
    pub score: usize,
    pub prog: Prog,
    pub results: Vec<StateOut>,
}
because!(Better, FormClimb, "a better program for one entry: the cases right, the program, and the results of the reading");

struct ChoiceTry {
    work: u64,
    changed: usize,
    score: usize,
    passes: bool,
    prog: Prog,
    results: Vec<(usize, StateOut)>,
}
because!(ChoiceTry, FormClimb, "one choice for one entry, tried on the cases of the entry: the work, the answers that changed, the cases right, whether the gain passes the margin, the program, and the new results of the cases of the entry");

struct SplitTry {
    work: u64,
    score: usize,
    prog: Prog,
    results: Vec<StateOut>,
    group: Vec<usize>,
    changed: usize,
    choosing: f64,
}
because!(SplitTry, FormClimb, "one choice for the split words of a slot, tried word by word: the work, the cases right at the end, the program, the results, the cases of the words that joined, the answers they changed, and the gain that the many words add to the margin");

impl FormSearch<'_> {
    fn again(&mut self, prog: &Prog, reading: usize, rows: &[usize], results: &[StateOut]) -> (Vec<StateOut>, usize) {
        let found = self.results_of(prog, reading, rows);
        let mut new = results.to_vec();
        let mut changed = 0;
        for (case, out) in rows.iter().zip(found) {
            changed += usize::from(out != results[*case]);
            new[*case] = out;
        }
        (new, changed)
    }

    fn choices_of(&self, form: u32, reading: usize) -> Vec<OpChoice> {
        let spans = self.spans.get(&form).map_or(&[][..], Vec::as_slice);
        choices(&self.shapes[form as usize], spans, self.readings[reading].name.copies())
    }

    fn try_choice(&self, key: Key, choice: Option<OpChoice>, rows: &[usize], prog: &Prog, reading: usize, results: &[StateOut], cur: usize) -> ChoiceTry {
        let new_prog = self.put(prog, key, choice);
        let found = self.ones(&new_prog, reading, rows, 1);
        let mut work = 0u64;
        let mut changed = 0;
        let mut changes: Vec<(usize, StateOut)> = Vec::with_capacity(rows.len());
        for (case, (out, cost)) in rows.iter().zip(found) {
            work += cost;
            changed += usize::from(out != results[*case]);
            changes.push((*case, out));
        }
        let (mut score, mut passes) = (0, false);
        if changed > 0 {
            let mut new = results.to_vec();
            for (case, out) in &changes {
                new[*case] = out.clone();
            }
            score = self.score(&new, &self.truths);
            let part: Vec<StateOut> = rows.iter().map(|case| new[*case].clone()).collect();
            passes = (score as f64 - cur as f64) >= margin(&part, Some(changed));
        }
        ChoiceTry { work, changed, score, passes, prog: new_prog, results: changes }
    }

    fn better(&mut self, key: Key, rows: &[usize], prog: &Prog, reading: usize, results: &[StateOut], cur: usize) -> Option<Better> {
        let form = match key {
            Key::Form(form) | Key::Word(form, _) => form,
        };
        let now: Option<OpChoice> = prog.said.get(&key).copied().flatten();
        let mut tried: Vec<Option<OpChoice>> = vec![None];
        tried.extend(self.choices_of(form, reading).into_iter().map(Some));
        tried.retain(|choice| *choice != now);
        let count = self.rows.len() as u64;
        let this = &*self;
        let found: Vec<ChoiceTry> = on_lanes(tried.len(), this.lanes, &|place| this.try_choice(key, tried[place], rows, prog, reading, results, cur));
        let mut best: Option<ChoiceTry> = None;
        for trial in found {
            self.work += trial.work + count;
            if trial.changed == 0 {
                continue;
            }
            if trial.passes && best.as_ref().is_none_or(|held| trial.score > held.score) {
                best = Some(trial);
            }
        }
        best.map(|trial| {
            let mut new = results.to_vec();
            for (case, out) in trial.results {
                new[case] = out;
            }
            Better { score: trial.score, prog: trial.prog, results: new }
        })
    }

    fn words_at<N: WordTexts>(&self, names: &N, rows_of: &[usize], form: u32, place: usize) -> Vec<(u32, Vec<usize>)> {
        let mut held: Vec<(u32, Vec<usize>)> = Vec::new();
        for case in rows_of {
            for unit in &self.rows[*case].units {
                if unit.fid == form {
                    let word = unit.tokens[place];
                    match held.iter_mut().find(|(seen, _)| *seen == word) {
                        Some((_, cases)) => {
                            if !cases.contains(case) {
                                cases.push(*case);
                            }
                        }
                        None => held.push((word, vec![*case])),
                    }
                }
            }
        }
        for (_, cases) in held.iter_mut() {
            cases.sort_unstable();
        }
        held.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| names.name(a.0).cmp(names.name(b.0))));
        held
    }

    #[allow(clippy::too_many_arguments)]
    fn try_split(&self, form: u32, place: usize, choice: Option<OpChoice>, words: &[(u32, Vec<usize>)], prog: &Prog, reading: usize, results: &[StateOut], cur: usize) -> SplitTry {
        let count = self.rows.len() as u64;
        let mut trial = prog.clone();
        trial.sub_at.insert(form, place);
        let mut t_results = results.to_vec();
        let mut t_cur = cur;
        let mut group: Vec<usize> = Vec::new();
        let mut changed_all = 0usize;
        let mut choosing = f64::default();
        let mut work = 0u64;
        for (word, cases) in words {
            let new_prog = self.put(&trial, Key::Word(form, *word), choice);
            let found = self.ones(&new_prog, reading, cases, 1);
            let mut changed = 0;
            for (case, (out, cost)) in cases.iter().zip(&found) {
                work += cost;
                changed += usize::from(*out != t_results[*case]);
            }
            work += count;
            if changed > 0 {
                let mut new = t_results.clone();
                for (case, (out, _)) in cases.iter().zip(found) {
                    new[*case] = out;
                }
                let score = self.score(&new, &self.truths);
                if score > t_cur {
                    trial = new_prog;
                    t_results = new;
                    t_cur = score;
                    for case in cases {
                        if !group.contains(case) {
                            group.push(*case);
                        }
                    }
                    changed_all += changed;
                    choosing += SPLIT_SPREAD * (changed as f64).sqrt();
                }
            }
        }
        SplitTry { work, score: t_cur, prog: trial, results: t_results, group, changed: changed_all, choosing }
    }

    #[allow(clippy::too_many_arguments)]
    fn split<N: WordTexts>(&mut self, names: &N, rows_of: &[usize], form: u32, prog: &Prog, reading: usize, results: &[StateOut], cur: usize) -> Option<Better> {
        let mut best: Option<Better> = None;
        let now: Option<OpChoice> = prog.said.get(&Key::Form(form)).copied().flatten();
        let places: Vec<usize> = match prog.sub_at.get(&form) {
            Some(place) => vec![*place],
            None => slots(&self.shapes[form as usize]).iter().map(|(from, _)| *from as usize).collect(),
        };
        for place in places {
            let words: Vec<(u32, Vec<usize>)> = self.words_at(names, rows_of, form, place).into_iter().filter(|(word, _)| !prog.subs.contains_key(&(form, *word))).collect();
            if words.len() < WORDS_LEAST {
                continue;
            }
            let mut tried: Vec<Option<OpChoice>> = vec![None];
            tried.extend(self.choices_of(form, reading).into_iter().map(Some));
            tried.retain(|choice| *choice != now);
            let batch = self.lanes.max(1) * BATCH_TRIALS;
            let mut at = 0;
            'choices: while at < tried.len() {
                if self.late() {
                    break;
                }
                let end = (at + batch).min(tried.len());
                let this = &*self;
                let found: Vec<SplitTry> = on_lanes(end - at, this.lanes, &|step| this.try_split(form, place, tried[at + step], &words, prog, reading, results, cur));
                for trial in found {
                    if self.late() {
                        break 'choices;
                    }
                    self.work += trial.work;
                    let part: Vec<StateOut> = trial.group.iter().map(|case| trial.results[*case].clone()).collect();
                    let enough = trial.group.len() >= SURE_CASES && (trial.score as f64 - cur as f64) >= margin(&part, Some(trial.changed)) + trial.choosing;
                    if enough && best.as_ref().is_none_or(|held| trial.score > held.score) {
                        best = Some(Better { score: trial.score, prog: trial.prog, results: trial.results });
                    }
                }
                at = end;
            }
        }
        best
    }

    pub fn climb<N: WordTexts>(&mut self, names: &N, start: &StateStart, forms: &[u32], rows_of: &[Vec<usize>], frozen: &Prog) -> End {
        let mut cur = start.score;
        let mut reading = start.reading;
        let mut prog = start.prog.clone();
        let mut results = start.results.clone();
        let count = self.rows.len();
        let mut moved = true;
        while moved && cur < count && !self.late() {
            moved = false;
            for (place, form) in forms.iter().enumerate() {
                if let Some(found) = self.better(Key::Form(*form), &rows_of[place], &prog, reading, &results, cur) {
                    (cur, prog, results) = (found.score, found.prog, found.results);
                    moved = true;
                }
                if let Some(at) = prog.sub_at.get(form).copied() {
                    for (word, cases) in self.words_at(names, &rows_of[place], *form, at) {
                        let found = if prog.subs.contains_key(&(*form, word)) { self.better(Key::Word(*form, word), &cases, &prog, reading, &results, cur) } else { None };
                        if let Some(found) = found {
                            (cur, prog, results) = (found.score, found.prog, found.results);
                            moved = true;
                        }
                    }
                }
            }
            if moved || cur >= count {
                continue;
            }
            for (place, form) in forms.iter().enumerate() {
                if let Some(found) = self.split(names, &rows_of[place], *form, &prog, reading, &results, cur) {
                    (cur, prog, results) = (found.score, found.prog, found.results);
                    moved = true;
                    break;
                }
            }
            if moved {
                continue;
            }
            let every = self.all_results(&prog, None, None);
            let truths = self.truths.clone();
            for (other, found) in every.out.into_iter().enumerate() {
                let score = self.score(&found, &truths);
                let mut distinct: Vec<&StateOut> = Vec::new();
                for result in &found {
                    if !distinct.contains(&result) {
                        distinct.push(result);
                        if distinct.len() > 1 {
                            break;
                        }
                    }
                }
                let changed = found.iter().zip(&results).filter(|(a, b)| a != b).count();
                if score > cur && distinct.len() > 1 && (score as f64 - cur as f64) >= margin(&found, Some(changed)) {
                    cur = score;
                    reading = other;
                    results = found;
                    moved = true;
                }
            }
        }
        let mut keys: Vec<(bool, String, Key)> = prog
            .said
            .keys()
            .filter(|key| !frozen.said.contains_key(*key))
            .map(|key| match key {
                Key::Form(form) => (true, form.to_string(), *key),
                Key::Word(form, word) => (false, format!("({form}, '{}')", names.name(*word)), *key),
            })
            .collect();
        keys.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        let truths = self.truths.clone();
        for (_, _, key) in keys {
            let form = match key {
                Key::Form(form) | Key::Word(form, _) => form,
            };
            let trial = prog.without(key);
            let rows = forms.iter().position(|held| *held == form).map(|place| rows_of[place].clone()).unwrap_or_default();
            let (new, _) = self.again(&trial, reading, &rows, &results);
            if self.score(&new, &truths) >= cur {
                prog = trial;
                results = new;
            }
        }
        End { score: cur, reading, prog, results }
    }
}

const WORDS_LEAST: usize = 2;
because!(WORDS_LEAST, FormClimb, "a slot is split only when it holds more than one word in the cases of its form");
