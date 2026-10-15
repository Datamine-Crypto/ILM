use crate::base::symbols::Symbols;
use crate::base::words::answer_words;
use crate::cases::Case;
use crate::tools::call::{Learned, Learner, Lessons, Said};
use crate::tools::pattern::hash::CheapMap;
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::{Fresh, WordTexts};
use crate::tools::state::choices::{empty_result, readings_of};
use crate::tools::state::read::{Codes, StateOut, StateReading, read};
use crate::tools::state::reader::{MoreForms, StateReader};
use crate::tools::state::search::{End, FormSearch, MOST_PAIR_CASES, MOST_PAIR_FORMS, StateRow, SURE_CASES, SURE_SHARE_SEARCH, StateStart};
use crate::tools::state::table::{Prog, run_story};
use patterns::{because, source};
use std::collections::HashSet;

pub struct StateRules;
source!(StateRules, "a program of the state is a rule when it is right for nearly all of enough cases of the search and again for nearly all of the last third of the cases that the search did not see, both counted over the cases whose units have forms that the search could test; with no rule a case gets the most frequent answer");

const LEAST_SPAN: u32 = 3;
because!(LEAST_SPAN, StateRules, "a span of tokens where the answer of a lesson stood becomes an argument of its form when this many lessons show it");

const SEARCH_PARTS: usize = 3;
because!(SEARCH_PARTS, StateRules, "the cases of a form of question are cut in parts of this size: the search sees all but the last, the rule is tested again on the last");

const SEARCH_SHARE: usize = 2;
because!(SEARCH_SHARE, StateRules, "the parts of the cases of a form of question that the search sees");

pub struct FormRule {
    pub reading: StateReading,
    pub table: Vec<(StateOut, Vec<u32>)>,
    pub normal: bool,
}
because!(FormRule, StateRules, "the rule of a form of question: its reading, the answer of each result that is no word, and whether a result of words is given in its normal form");

pub struct StateModel {
    pub symbols: Symbols,
    pub reader: StateReader,
    pub spans: CheapMap<u32, Vec<(u8, u8)>>,
    pub default: Vec<u32>,
    pub prog: Prog,
    pub found: CheapMap<u32, FormRule>,
    pub q_default: CheapMap<u32, Vec<u32>>,
    pub work: u64,
    pub notes: Vec<String>,
    pub lanes: usize,
}
because!(StateModel, StateRules, "the state tool after its lessons: the words, the reader of stories into units, the spans of answers of each form, the most frequent answer, the program, the rule of each form of question, the sure answer of a form of question that has one, the work of the search, and notes of the search for the stage checks, and the threads that the search runs on. learn makes it; said answers a case");

fn first_most_of(truths: &[u32]) -> (u32, u32) {
    let mut counted: FirstSightTable<u32, u32> = FirstSightTable::new();
    for truth in truths {
        counted.add(*truth, 1);
    }
    counted.first_most().copied().unwrap_or_default()
}
because!(first_most_of, StateRules, "the most frequent truth of some cases with its count, the first of equals in their order");

impl StateModel {
    pub fn learn(cases: &[Case], lanes: usize) -> StateModel {
        let mut symbols = Symbols::new();
        let mut reader = StateReader::learn(cases, &mut symbols);
        let mut codes = Codes::default();
        let mut forms = std::mem::take(&mut reader.forms);
        let mut rows: Vec<StateRow> = Vec::with_capacity(cases.len());
        let mut qids: Vec<u32> = Vec::with_capacity(cases.len());
        let mut spans: FirstSightTable<(u32, u8, u8), u32> = FirstSightTable::new();
        for case in cases {
            let asked = reader.case(&mut symbols, &mut forms, &case.sentences, &case.question);
            let truth_words: Vec<u32> = answer_words(&case.answer).iter().map(|word| symbols.number(word)).collect();
            let truth = codes.phrase(&reader.first, &truth_words);
            let units: Vec<_> = asked.units.iter().map(|unit| codes.code(&reader.first, &mut symbols, unit)).collect();
            for unit in &asked.units {
                for from in 0..unit.tokens.len() {
                    let mut words: Vec<u32> = Vec::new();
                    for to in from + 1..=unit.tokens.len().min(from + crate::tools::state::kinds::STATE_RUN) {
                        words.extend_from_slice(&unit.raws[to - 1]);
                        if words == truth_words {
                            spans.add((unit.fid, from as u8, to as u8), 1);
                        }
                    }
                }
            }
            let q = codes.code(&reader.first, &mut symbols, &asked.question);
            qids.push(asked.question.fid);
            rows.push(StateRow { units, q, truth });
        }
        let mut counted: Vec<((u32, u8, u8), u32)> = spans.rows().to_vec();
        counted.sort_by_key(|(key, _)| *key);
        let mut by_form: CheapMap<u32, Vec<(u8, u8)>> = CheapMap::default();
        for ((form, from, to), count) in counted {
            if count >= LEAST_SPAN {
                by_form.entry(form).or_default().push((from, to));
            }
        }
        let truths: Vec<u32> = rows.iter().map(|row| row.truth).collect();
        let default = codes.words[first_most_of(&truths).0 as usize].clone();
        reader.forms = forms;
        let mut model = StateModel { symbols, reader, spans: by_form, default: if rows.is_empty() { Vec::new() } else { default }, prog: Prog::default(), found: CheapMap::default(), q_default: CheapMap::default(), work: 0, notes: Vec::new(), lanes };
        let mut by_q: FirstSightTable<u32, Vec<usize>> = FirstSightTable::new();
        for (place, qid) in qids.iter().enumerate() {
            by_q.entry(*qid, Vec::new()).push(place);
        }
        let mut order: Vec<(u32, Vec<usize>)> = by_q.rows().to_vec();
        order.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
        for (qid, mine) in order {
            let picked: Vec<u32> = mine.iter().map(|place| rows[*place].truth).collect();
            let (answer, count) = first_most_of(&picked);
            if mine.len() >= SURE_CASES {
                if f64::from(count) >= SURE_SHARE_SEARCH * mine.len() as f64 {
                    model.q_default.insert(qid, codes.words[answer as usize].clone());
                }
                let chosen: Vec<&StateRow> = mine.iter().map(|place| &rows[*place]).collect();
                model.learn_form(qid, &chosen, &codes);
            }
        }
        model
    }

    fn learn_form(&mut self, qid: u32, mine: &[&StateRow], codes: &Codes) {
        let mut cut = (SEARCH_SHARE * mine.len() + SEARCH_SHARE) / SEARCH_PARTS;
        if cut < SURE_CASES {
            cut = mine.len();
        }
        let (search, second) = mine.split_at(cut);
        let truths: Vec<u32> = search.iter().map(|row| row.truth).collect();
        let count = search.len();
        let mut present: FirstSightTable<u32, u32> = FirstSightTable::new();
        for row in search {
            let mut seen: Vec<u32> = Vec::new();
            for unit in &row.units {
                if !seen.contains(&unit.fid) {
                    seen.push(unit.fid);
                    present.add(unit.fid, 1);
                }
            }
        }
        let frozen: HashSet<u32> = self.prog.ops.keys().copied().collect();
        let mut ranked: Vec<(u32, u32)> = present.rows().to_vec();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let forms: Vec<u32> = ranked.into_iter().filter(|(form, seen)| *seen as usize >= SURE_CASES && !frozen.contains(form)).map(|(form, _)| form).collect();
        let rows_of: Vec<Vec<usize>> = forms.iter().map(|form| (0..count).filter(|case| search[*case].units.iter().any(|unit| unit.fid == *form)).collect()).collect();
        let shape = self.reader.forms.shapes[qid as usize].clone();
        let readings = readings_of(&shape);
        let mut watch: Vec<(u8, u8)> = readings.iter().filter(|reading| reading.name.history()).map(|reading| reading.args[0]).collect();
        watch.sort_unstable();
        watch.dedup();
        let head = format!("question\t{}\tcases {}\tsearch {}", self.reader.shape_line(&self.symbols, &shape), mine.len(), count);
        if readings.is_empty() || self.q_default.contains_key(&qid) {
            self.notes.push(format!("{head}\tend"));
            return;
        }
        let mut groups: Vec<(Vec<(u8, u8)>, Vec<(usize, StateOut)>)> = Vec::new();
        for (place, reading) in readings.iter().enumerate() {
            let entry = (place, empty_result(reading.name));
            match groups.iter_mut().find(|(args, _)| *args == reading.args) {
                Some((_, members)) => members.push(entry),
                None => groups.push((reading.args.clone(), vec![entry])),
            }
        }
        let first = &self.reader.first;
        let frozen_prog = self.prog.clone();
        let mut search_of = FormSearch {
            first,
            codes,
            shapes: &self.reader.forms.shapes,
            spans: &self.spans,
            rows: search.to_vec(),
            truths: truths.clone(),
            readings: readings.clone(),
            watch,
            groups,
            work: self.work,
            in_table: false,
            lanes: self.lanes,
        };
        let plain = search_of.all_results(&frozen_prog, None, None);
        let sure_line = SURE_SHARE_SEARCH * count as f64;
        let mut best: Option<End> = None;
        let ends = |search_of: &mut FormSearch, found: &[StateStart], best: &mut Option<End>, names: &Symbols| {
            for start in found {
                if best.as_ref().is_some_and(|held| held.score as f64 >= sure_line) || search_of.late() {
                    break;
                }
                let end = search_of.climb(names, start, &forms, &rows_of, &frozen_prog);
                if best.as_ref().is_none_or(|held| end.score > held.score) {
                    *best = Some(end);
                }
            }
        };
        if !frozen.is_empty() {
            let (gain, _) = search_of.starts(vec![(None, vec![frozen_prog.clone()])], None, &plain);
            ends(&mut search_of, &gain, &mut best, &self.symbols);
        }
        let (mut gain, mut held) = (Vec::new(), Vec::new());
        if best.as_ref().is_none_or(|held| (held.score as f64) < sure_line) {
            let mut groups: Vec<(Option<Vec<usize>>, Vec<Prog>)> = Vec::new();
            for (place, form) in forms.iter().enumerate() {
                let programs = search_of.set_programs(&frozen_prog, *form);
                groups.push((Some(rows_of[place].clone()), programs));
            }
            (gain, held) = search_of.starts(groups, None, &plain);
            ends(&mut search_of, &gain, &mut best, &self.symbols);
            ends(&mut search_of, &held, &mut best, &self.symbols);
        }
        if best.as_ref().is_none_or(|held| (held.score as f64) < sure_line) && (!gain.is_empty() || !held.is_empty() || search_of.in_table) {
            let few: Vec<usize> = (0..forms.len().min(MOST_PAIR_FORMS)).collect();
            let mut groups: Vec<(Option<Vec<usize>>, Vec<Prog>)> = Vec::new();
            for (place, first_place) in few.iter().enumerate() {
                for second_place in &few[place + 1..] {
                    let mut rows: Vec<usize> = rows_of[*first_place].clone();
                    rows.extend(rows_of[*second_place].iter().copied());
                    rows.sort_unstable();
                    rows.dedup();
                    let programs = search_of.pair_programs(&frozen_prog, forms[*first_place], forms[*second_place]);
                    groups.push((Some(rows), programs));
                }
            }
            let (gain, held) = search_of.starts(groups, Some(MOST_PAIR_CASES), &plain);
            ends(&mut search_of, &gain, &mut best, &self.symbols);
            ends(&mut search_of, &held, &mut best, &self.symbols);
        }
        let Some(End { reading, prog, results, score: cur }) = best else {
            self.work = search_of.work;
            self.notes.push(format!("{head}\tend"));
            return;
        };
        let every = search_of.all_results(&prog, None, None);
        let mut reading = reading;
        let mut least: Option<(usize, u8)> = None;
        for (place, found) in every.out.iter().enumerate() {
            if *found == results {
                let key = (readings[place].args.len(), readings[place].args.iter().map(|(from, to)| to - from).sum::<u8>());
                if least.is_none_or(|held| key < held) {
                    least = Some(key);
                    reading = place;
                }
            }
        }
        let table = search_of.table(&results, &truths);
        let plain_right = results.iter().zip(&truths).filter(|(result, truth)| matches!(result, StateOut::Phrase(found) if found == *truth)).count();
        let normal_right = results.iter().zip(&truths).filter(|(result, truth)| result.is_words() && codes.normal_is(first, result, **truth)).count();
        let normal = normal_right > plain_right;
        let say = |result: &StateOut| -> Option<u32> { say_of(codes, first, result, &table, normal) };
        search_of.results_rows(&prog, reading, second);
        let known: HashSet<u32> = forms.iter().copied().chain(frozen.iter().copied()).collect();
        let read1: Vec<usize> = (0..count).filter(|case| search[*case].units.iter().all(|unit| known.contains(&unit.fid))).collect();
        let read2: Vec<&StateRow> = second.iter().copied().filter(|row| row.units.iter().all(|unit| known.contains(&unit.fid))).collect();
        let right_read1 = read1.iter().filter(|case| say(&results[**case]) == Some(truths[**case])).count();
        let found_read2 = search_of.results_rows(&prog, reading, &read2);
        let right_read2 = found_read2.iter().zip(&read2).filter(|(result, row)| say(result) == Some(row.truth)).count();
        let sure = read1.len() >= SURE_CASES && right_read1 as f64 >= SURE_SHARE_SEARCH * read1.len() as f64 && right_read2 as f64 >= SURE_SHARE_SEARCH * read2.len() as f64;
        let found = &readings[reading];
        let args: Vec<String> = found.args.iter().map(|(from, to)| format!("{from}-{to}")).collect();
        let sure_text = if sure { "True" } else { "False" };
        self.notes.push(format!("{head}\treading {} {}\tright {cur}\tread ({right_read1}, {}, {right_read2}, {})\tsure {sure_text}", found.name.text(), args.join(" "), read1.len(), read2.len()));
        let mut entries: Vec<(String, String)> = Vec::new();
        for (key, choice) in prog.said.iter() {
            if frozen_prog.said.contains_key(key) {
                continue;
            }
            let (form, word, order) = match key {
                crate::tools::state::table::Key::Form(form) => (*form, None, form.to_string()),
                crate::tools::state::table::Key::Word(form, word) => (*form, Some(*word), format!("({form}, '{}')", self.symbols.name(*word))),
            };
            let shape = &self.reader.forms.shapes[form as usize];
            let mut place = self.reader.shape_line(&self.symbols, shape);
            if let Some(word) = word {
                place = format!("{place} with {}", self.symbols.name(word));
            }
            let said = match choice {
                None => "nothing".to_string(),
                Some(choice) => choice.text(&self.reader, shape),
            };
            entries.push((order, format!("   {place}: {said}")));
        }
        entries.sort();
        self.notes.extend(entries.into_iter().map(|(_, line)| line));
        if sure {
            let all: Vec<StateOut> = search_of.results_rows(&prog, reading, mine);
            let all_truths: Vec<u32> = mine.iter().map(|row| row.truth).collect();
            let table = search_of.table(&all, &all_truths);
            self.prog = prog;
            let table = table.into_iter().map(|(result, truth)| (result, codes.words[truth as usize].clone())).collect();
            self.found.insert(qid, FormRule { reading: readings[reading].clone(), table, normal });
        }
        self.work = search_of.work;
    }

    pub fn said(&self, sentences: &[String], question: &str) -> (Vec<String>, String) {
        let mut names = Fresh::over(&self.symbols);
        let mut forms = MoreForms::from(&self.reader.forms);
        let asked = self.reader.case(&mut names, &mut forms, sentences, question);
        let qid = asked.question.fid;
        let Some(found) = self.found.get(&qid) else {
            let words = match self.q_default.get(&qid) {
                Some(answer) => answer.clone(),
                None => self.default.clone(),
            };
            return (words.iter().map(|word| names.name(*word).to_string()).collect(), String::new());
        };
        let mut codes = Codes::default();
        let units: Vec<_> = asked.units.iter().map(|unit| codes.code(&self.reader.first, &mut names, unit)).collect();
        let q = codes.code(&self.reader.first, &mut names, &asked.question);
        let watch: Vec<u32> = if found.reading.name.history() { vec![q.span(found.reading.args[0].0, found.reading.args[0].1)] } else { Vec::new() };
        let (state, hist) = run_story(&units, &self.prog, &watch);
        let history: Vec<(u32, &[u32])> = watch.iter().zip(&hist).map(|(key, values)| (*key, values.as_slice())).collect();
        let result = read(&found.reading, &q, &state, &history, &codes);
        let words: Option<Vec<u32>> = match &result {
            StateOut::Phrase(_) | StateOut::Many(_) => {
                let words = codes.words_of(&result).to_vec();
                Some(if found.normal { words.iter().map(|word| self.reader.first.norm(*word)).collect() } else { words })
            }
            StateOut::Unknown => None,
            other => found.table.iter().find(|(held, _)| held == other).map(|(_, truth)| truth.clone()),
        };
        let words = words.unwrap_or_else(|| self.default.clone());
        (words.iter().map(|word| names.name(*word).to_string()).collect(), String::new())
    }
}

pub fn say_of(codes: &Codes, first: &crate::tools::pattern::kinds::PatternWords, result: &StateOut, table: &[(StateOut, u32)], normal: bool) -> Option<u32> {
    match result {
        StateOut::Phrase(found) => Some(if normal { codes.normal[*found as usize] } else { *found }),
        StateOut::Many(words) => {
            let words: Vec<u32> = if normal { words.iter().map(|word| first.norm(*word)).collect() } else { words.to_vec() };
            codes.phrases.get(&words).copied().or(Some(crate::tools::pattern::classes::ABSENT))
        }
        StateOut::Unknown => None,
        other => table.iter().find(|(held, _)| held == other).map(|(_, truth)| *truth),
    }
}
because!(say_of, StateRules, "what a result says, as the number of its words: words as they stand or in their normal form, the answer of the table for a result that is no word, and nothing when the table does not know");

impl Learner for StateModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let said = self.said(sentences, question);
        Said { words: said.0, note: said.1 }
    }
}

pub fn learn_state(lessons: Lessons, lanes: usize) -> Learned {
    let cases = lessons.cases;
    Ok(Box::new(StateModel::learn(cases, lanes)))
}
because!(learn_state, StateRules, "the state tool as a tool of the one call: it learns a program for each form of question and gives the most frequent answer where it found no rule");
