use crate::tools::parts::counted::FORCED_LESSONS;
use crate::tools::parts::found::{CELLS_WITH_FORMS, Chart, ChartCell, CutWay, FormsFound, Grouped, RunValue, TURNS_MOST, Work};
use crate::tools::parts::language::{WordRow, Word};
use crate::tools::parts::ordered::{Few, Kept};
use crate::tools::parts::places::{Bit, FormKey, PartKind, Lexicon, Out, Place, WHOLE, laid_out};
use crate::tools::parts::programs::{FoundRun, every_program};
use patterns::{because, source};
use std::cell::Cell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;
use crate::base::lanes::on_lanes;

pub struct VersionSpace;
source!(VersionSpace, "the version space of Mitchell for the program of each text of a form: every program that gives as many of its runs as the best one is kept, and a lesson counts as a run only when every program of the form of each of its parts gives that part the same answer, so no answer in doubt is counted; at the end each text keeps its simplest program");

const PARTS_MOST: usize = 4;
because!(PARTS_MOST, VersionSpace, "the parts of the text of a form of these ways at most: the runs of a text with more parts are too many to lay side by side");

const STALL_TURNS: usize = 3;
because!(STALL_TURNS, VersionSpace, "the growth ends when this many turns bring nearly no lesson more that is read as one part: such a way is not for these lessons");

const STALL_SHARE: f64 = 0.01;
because!(STALL_SHARE, VersionSpace, "the share of the lessons below which the gain of the last turns counts as nearly none");

const FIXED_WORDS: usize = 2;
because!(FIXED_WORDS, VersionSpace, "a part of one word becomes a fixed word only when its groups have programs for more than half of all runs: the runs times this number must pass all runs");

fn kinds_of(text: &[Bit]) -> Vec<PartKind> {
    let mut kinds: Vec<PartKind> = text.iter().filter_map(Bit::kind).collect();
    kinds.sort_unstable();
    kinds.dedup();
    kinds
}
because!(kinds_of, VersionSpace, "the different kinds of the parts of a text of a form, the smallest first");

impl FormsFound {
    fn touched(&self, work: &Work, text: &[Bit]) -> Vec<usize> {
        let mut found: Option<Vec<usize>> = None;
        for word in text.iter().filter_map(Bit::word) {
            let mine: &[usize] = work.with_word.get(&word).map(Vec::as_slice).unwrap_or_default();
            found = Some(match found {
                None => mine.to_vec(),
                Some(held) => held.into_iter().filter(|lesson| mine.binary_search(lesson).is_ok()).collect(),
            });
        }
        found.unwrap_or_else(|| (0..work.lessons.len()).collect())
    }

    pub fn chart_sure(&self, text: &[Word]) -> Chart {
        let n = text.len();
        self.count_cells(n);
        let words: HashSet<Word> = text.iter().copied().collect();
        let usable: HashMap<Option<Word>, Vec<&Grouped>> = self
            .grouped
            .pairs()
            .filter(|(first, _)| first.is_none_or(|word| words.contains(&word)))
            .map(|(first, forms)| (*first, forms.iter().filter(|form| form.fixed.iter().all(|word| words.contains(word))).collect()))
            .collect();
        let free = usable.get(&None);
        let row_at = self.rows(text, None);
        let mut table = Chart::new(n);
        for size in 1..=n {
            for x in 0..=n - size {
                let y = x + size;
                let mut found: ChartCell = Few::default();
                if let Some((kind, row)) = row_at.get(&(x, y)) {
                    found.put(*kind, Few::of(row.clone(), 1));
                }
                if size > 1 {
                    for forms in [usable.get(&Some(text[x])), free] {
                        for form in forms.into_iter().flatten() {
                            if form.text.len() > size {
                                continue;
                            }
                            for (filled, _) in self.laid(text, x, y, &form.text, 0, &table) {
                                let mut given: Vec<Option<WordRow>> = Vec::new();
                                for out in form.programs.iter() {
                                    let row = laid_out(out, &filled);
                                    if !given.contains(&row) {
                                        given.push(row);
                                    }
                                }
                                if let [Some(row)] = given.as_slice() {
                                    found.or_put(form.kind, Few::default).put(row.clone(), 1);
                                }
                            }
                        }
                    }
                }
                if !found.is_empty() {
                    table.set(x, y, found);
                }
            }
        }
        table
    }

    pub fn by_word(&self, work: &Work, pattern: &[Bit], items: &[&RunValue], banned: &HashSet<Word>, lexicon: &Lexicon, searched: &Cell<u64>) -> Vec<(Vec<Bit>, Out, usize)> {
        let places: Vec<usize> = pattern.iter().enumerate().filter(|(_, bit)| bit.kind().is_some()).map(|(place, _)| place).collect();
        let mut best: (usize, Vec<(Vec<Bit>, Out, usize)>) = (0, Vec::new());
        for part in 0..places.len() {
            if items.iter().any(|(_, _, words)| words[part].len() != 1) {
                continue;
            }
            let mut groups: Kept<Word, Vec<FoundRun>> = Kept::default();
            for (rows, target, words) in items {
                groups.or_put(words[part][0], Vec::new).push((rows.clone(), target.clone()));
            }
            let mut made: Vec<(Vec<Bit>, Out, usize)> = Vec::new();
            let mut total = 0;
            for (word, group) in groups.pairs() {
                if group.len() < FORCED_LESSONS {
                    continue;
                }
                let mine: HashSet<Word> = banned.iter().copied().filter(|answer| work.owner.get(answer).is_none_or(|words| !words.contains(word))).collect();
                let (out, explained) = self.program(group, &mine, lexicon, searched);
                let Some(out) = out else { continue };
                if explained.len() < FORCED_LESSONS || explained.len() * FIXED_WORDS <= group.len() {
                    continue;
                }
                if out.iter().any(|place| matches!(place, Place::Look(held, _, _) | Place::Looked(held, _) if *held == part)) {
                    continue;
                }
                let said = &group[0].0[part];
                let shift = |held: usize| held - usize::from(held > part);
                let mut program: Out = Vec::new();
                for place in &out {
                    match place {
                        Place::Word(_) => program.push(place.clone()),
                        Place::Part(held) if *held == part => program.extend(said.iter().map(|word| Place::Word(*word))),
                        Place::Seen(held, view) if *held == part => program.extend(view.on(said).into_iter().map(Place::Word)),
                        Place::Look(held, view, table) => program.push(Place::Look(shift(*held), *view, table.clone())),
                        Place::Looked(held, view) => program.push(Place::Looked(shift(*held), *view)),
                        Place::Part(held) => program.push(Place::Part(shift(*held))),
                        Place::Seen(held, view) => program.push(Place::Seen(shift(*held), *view)),
                    }
                }
                let mut text = pattern.to_vec();
                text[places[part]] = Bit::Word(*word);
                made.push((text, program, explained.len()));
                total += explained.len();
            }
            if total > best.0 {
                best = (total, made);
            }
        }
        if best.0 * FIXED_WORDS > items.len() { best.1 } else { Vec::new() }
    }

    pub fn grow_table(&mut self, work: &mut Work, lexicon: &Lexicon, searched: &Cell<u64>) {
        let n = work.lessons.len();
        let mut space: Kept<Vec<Bit>, Arc<Vec<Out>>> = Kept::default();
        let mut cache: HashMap<Vec<Bit>, (HashMap<Vec<Word>, RunValue>, Kept<Vec<Bit>, Arc<Vec<Out>>>)> = HashMap::new();
        let mut states: Vec<HashMap<Vec<Bit>, Arc<Vec<Out>>>> = Vec::new();
        let mut gains: Vec<isize> = Vec::new();
        let mut charts: Vec<Chart> = vec![Chart::default(); n];
        let mut dirty: BTreeSet<usize> = (0..n).collect();
        let mut runs: Kept<Vec<Bit>, Kept<Vec<Word>, RunValue>> = Kept::default();
        for _ in 0..TURNS_MOST {
            if self.cells() > CELLS_WITH_FORMS {
                break;
            }
            self.grouped = Kept::default();
            for (text, programs) in space.pairs() {
                if text.len() > 1 {
                    let fixed: Vec<Word> = text.iter().filter_map(Bit::word).collect();
                    for kind in kinds_of(text) {
                        self.grouped.or_put(text[0].word(), Vec::new).push(Grouped { text: text.clone(), kind, programs: programs.clone(), fixed: fixed.clone() });
                    }
                }
            }
            let todo: Vec<usize> = std::mem::take(&mut dirty).into_iter().collect();
            let made: Vec<Chart> = {
                let (me, lessons) = (&*self, &work.lessons);
                on_lanes(todo.len(), self.lanes, &|piece| me.chart_sure(&lessons[todo[piece]].0))
            };
            for (lesson, chart) in todo.into_iter().zip(made) {
                charts[lesson] = chart;
            }
            runs = Kept::default();
            for (lesson, (text, answer)) in work.lessons.iter().enumerate() {
                let found: Vec<CutWay> = self.cut(text, &charts[lesson], 0, text.len(), None).into_iter().filter(|way| !way.rows.is_empty()).collect();
                if let [way] = found.as_slice() {
                    runs.or_put(way.pattern.clone(), Kept::default).put(text.clone(), (way.rows.clone(), answer.clone(), way.words.clone()));
                }
                if let Some(cell) = charts[lesson].get(0, text.len()) {
                    for (kind, rows) in cell.iter() {
                        if rows.holds(answer) {
                            runs.or_put(vec![Bit::Part(*kind)], Kept::default).put(text.clone(), (vec![answer.clone()], answer.clone(), vec![text.clone()]));
                        }
                    }
                }
            }
            let mut changed = false;
            let mut fresh: Kept<Vec<Bit>, Arc<Vec<Out>>> = Kept::default();
            for (pattern, found) in runs.pairs() {
                if found.len() < FORCED_LESSONS || pattern.iter().filter(|bit| bit.kind().is_some()).count() > PARTS_MOST {
                    continue;
                }
                let content: HashMap<Vec<Word>, RunValue> = found.pairs().map(|(text, value)| (text.clone(), value.clone())).collect();
                if cache.get(pattern).is_none_or(|(mark, _)| *mark != content) {
                    let banned = work.owner.iter().filter(|(_, words)| !words.iter().any(|word| pattern.contains(&Bit::Word(*word)))).map(|(answer, _)| *answer).collect::<HashSet<Word>>();
                    let items: Vec<FoundRun> = found.values().map(|(rows, target, _)| (rows.clone(), target.clone())).collect();
                    let programs = every_program(&items, &banned, searched, lexicon);
                    let mut made: Kept<Vec<Bit>, Arc<Vec<Out>>> = Kept::default();
                    if programs.is_empty() {
                        let values: Vec<&RunValue> = found.values().collect();
                        for (text, out, _) in self.by_word(work, pattern, &values, &banned, lexicon, searched) {
                            made.put(text, Arc::new(vec![out]));
                        }
                    } else {
                        made.put(pattern.clone(), Arc::new(programs));
                    }
                    if let Some((_, old)) = cache.get(pattern) {
                        for text in old.keys() {
                            if !made.holds(text) && space.holds(text) {
                                space.take(text);
                                dirty.extend(self.touched(work, text));
                                changed = true;
                            }
                        }
                    }
                    cache.insert(pattern.clone(), (content, made));
                }
                let Some((_, made)) = cache.get(pattern) else { continue };
                for (text, programs) in made.pairs() {
                    match space.get(text) {
                        None => {
                            fresh.put(text.clone(), programs.clone());
                        }
                        Some(held) if held != programs => {
                            space.put(text.clone(), programs.clone());
                            dirty.extend(self.touched(work, text));
                            changed = true;
                        }
                        Some(_) => {}
                    }
                }
            }
            if let Some(shortest) = fresh.keys().map(Vec::len).min() {
                for (text, programs) in fresh.pairs() {
                    if text.len() == shortest {
                        space.put(text.clone(), programs.clone());
                        dirty.extend(self.touched(work, text));
                        changed = true;
                    }
                }
            }
            self.turns += 1;
            let state: HashMap<Vec<Bit>, Arc<Vec<Out>>> = space.pairs().map(|(text, programs)| (text.clone(), programs.clone())).collect();
            if !changed || states.contains(&state) {
                break;
            }
            states.push(state);
            gains.push(runs.pairs().filter(|(text, _)| text.len() == 1).map(|(_, found)| found.len()).sum::<usize>() as isize);
            let last = gains.len() - 1;
            if gains.len() > STALL_TURNS && ((gains[last] - gains[last - STALL_TURNS]) as f64) < STALL_SHARE * (n as f64) {
                break;
            }
        }
        self.grouped = Kept::default();
        for (text, programs) in space.pairs() {
            let mut kinds = vec![WHOLE];
            if text.len() > 1 {
                kinds.extend(kinds_of(text));
            }
            let count = runs.get(text).map_or(0, Kept::len);
            for kind in kinds {
                let form = Arc::new(FormKey { pattern: text.clone(), kind, out: programs[0].clone() });
                self.forms.put(form, if count == 0 { 1 } else { count });
            }
        }
        self.index();
    }
}
