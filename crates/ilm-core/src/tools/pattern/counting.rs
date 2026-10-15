use crate::base::lanes::{on_lanes, shards};
use crate::base::symbols::Symbols;
use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::codes::{Point, Step};
use crate::tools::pattern::finder::{Finder, Steps, place_of};
use crate::tools::pattern::grouping::{CLUSTER_COUNT, ClusterKeys, FirstMarks, Logged, RunLog, SharedReadings, cluster_keys, write_readings};
use crate::tools::pattern::groups::Groups;
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::kinds::PatternWords;
use crate::tools::pattern::list::{CoveringList, Rule};
use crate::tools::pattern::story::Query;
use crate::tools::pattern::striped::{Grow, NearNumbers, Shared};
use crate::tools::pattern::table::{CountedPatterns, Places, Ready, Table, Tops};
use crate::tools::pattern::texts::{asked_text, place_text, step_text};
use patterns::{because, source};
use std::sync::{Mutex, PoisonError};

pub struct ShardedCounts;
source!(ShardedCounts, "the map and reduce of Dean and Ghemawat: counts add, so the lessons are counted in runs on all threads and the runs are joined in the order of the lessons; the joined counts and the order of first sight are those of one thread");

pub struct WideningBeam;
source!(WideningBeam, "the beam of the rule search of CN2 of Clark and Niblett: of the patterns that are read often and are not sure, only the most read ones get one more told sentence, so the search from general to specific stays within a bound");

pub const SHARD_FOLD: usize = 4;
because!(SHARD_FOLD, ShardedCounts, "the runs of lessons for each thread when there are several threads: with several runs a thread that got long stories gives its other runs to the threads that are done, and the joined result does not depend on the number of runs; one thread reads all lessons as one run, so that one table of the numbers of forms and steps that it has seen serves all of them");

pub const LEAST_KEPT: u32 = 3;
because!(LEAST_KEPT, WideningBeam, "a pattern with one more sentence stays only with this many readings or more: fewer readings can not make it sure and can not let it be widened, and such patterns are most of what a pass sees");

pub const MOST_WIDE: usize = 400;
because!(MOST_WIDE, WideningBeam, "the most read patterns of each depth that get one more sentence: wide enough that the pattern of every task of the public set is among them, and a bound on the work of the next depth");

pub struct Counting<'a> {
    pub words: &'a PatternWords,
    pub ready: &'a [Ready],
    pub point_kinds: Vec<u32>,
    pub star: u32,
    pub sure_n: u32,
    pub sure_share: f64,
    pub per_form: bool,
    pub lanes: usize,
    pub forms: Shared<Vec<u32>>,
    pub steps: Shared<Step>,
    pub asked: Shared<Vec<u32>>,
    pub places: Places,
    pub table: Table,
    pub order: Vec<Rule>,
    pub alive: Vec<bool>,
    pub near: Mutex<Vec<NearNumbers>>,
}
because!(Counting, CountedPatterns, CoveringList, "the count of the patterns of one set of lessons: the lessons as they were read, the kinds that a pointing may use, the bars of a sure rule, the shared tables of forms, steps and question forms, the places, the patterns with their readings, the list of rules, the lessons that the list leaves open, and the numbers of forms and steps that the threads have seen. count_one counts the patterns with one sentence, count_more those with one sentence more, make_list makes the list, open_forms gives the question forms that the list leaves open, text gives the text of a pattern; near_numbers lends a thread the numbers that a thread kept, keep_numbers takes them back");

impl<'a> Counting<'a> {
    pub fn over(words: &'a PatternWords, ready: &'a [Ready], point_kinds: Vec<u32>, star: u32, bars: (u32, f64), per_form: bool, lanes: usize) -> Counting<'a> {
        Counting {
            words,
            ready,
            point_kinds,
            star,
            sure_n: bars.0,
            sure_share: bars.1,
            per_form,
            lanes,
            forms: Shared::default(),
            steps: Shared::default(),
            asked: Shared::default(),
            places: Places::default(),
            table: Table::default(),
            order: Vec::new(),
            alive: vec![true; ready.len()],
            near: Mutex::new(Vec::new()),
        }
    }

    pub fn near_numbers(&self) -> NearNumbers {
        self.near.lock().unwrap_or_else(PoisonError::into_inner).pop().unwrap_or_default()
    }

    pub fn keep_numbers(&self, near: NearNumbers) {
        self.near.lock().unwrap_or_else(PoisonError::into_inner).push(near);
    }

    pub fn runs(&self) -> Vec<std::ops::Range<usize>> {
        let parts = if self.lanes > 1 { self.lanes * SHARD_FOLD } else { 1 };
        shards(self.ready.len(), parts)
    }

    pub fn join(&mut self, mut logs: Vec<RunLog>, least: u32, roots: bool) -> Vec<u32> {
        let lanes = self.lanes;
        let mut bases: Vec<u64> = Vec::with_capacity(logs.len());
        let mut readings: u64 = 0;
        for log in &logs {
            bases.push(readings);
            readings += u64::from(log.count);
        }
        let place_maps: Vec<Vec<u32>> = logs.iter().map(|log| log.places.rows.iter().map(|row| self.places.number(row)).collect()).collect();
        let held: Vec<Mutex<Vec<Vec<Logged>>>> = (0..CLUSTER_COUNT).map(|cluster| Mutex::new(logs.iter_mut().map(|log| std::mem::take(&mut log.clusters[cluster])).collect())).collect();
        let marks = FirstMarks::over(readings);
        let clusters: Vec<ClusterKeys> = on_lanes(CLUSTER_COUNT, lanes, &|cluster| {
            let rows = held[cluster].lock().map(|mut rows| std::mem::take(&mut *rows)).unwrap_or_default();
            cluster_keys(rows, &bases, least, &marks)
        });
        let ranks = marks.ranks();
        let first = self.table.parent.len() as u32;
        let numbers: Vec<Vec<u32>> = on_lanes(CLUSTER_COUNT, lanes, &|cluster| {
            let keys = &clusters[cluster];
            keys.totals.iter().zip(&keys.firsts).map(|(total, at)| if *total >= least { first + ranks.rank(*at) as u32 } else { ABSENT }).collect()
        });
        let kept = ranks.total() as usize;
        let mut rows: Vec<((u32, u32), u32)> = vec![((ABSENT, ABSENT), 0); kept];
        let mut room: usize = 0;
        for (keys, numbers) in clusters.iter().zip(&numbers) {
            for ((key, total), number) in keys.keys.iter().zip(&keys.totals).zip(numbers) {
                if *number != ABSENT {
                    rows[(number - first) as usize] = (*key, *total);
                    room += *total as usize;
                }
            }
        }
        self.table.reserve_room(kept, room);
        let mut made: Vec<u32> = Vec::with_capacity(kept);
        if roots {
            for (key, total) in rows {
                let (parent, step) = if key.1 == ABSENT { (ABSENT, key.0) } else { (self.table.child.get(ABSENT, key.0).unwrap_or(ABSENT), key.1) };
                made.push(self.table.add(parent, step, total as usize));
            }
        } else {
            for ((parent, step), total) in rows {
                made.push(self.table.add_unlinked(parent, step, total as usize));
            }
            let parts: Vec<Mutex<CheapMap<(u32, u32), u32>>> = self.table.child.take().into_iter().map(Mutex::new).collect();
            on_lanes(CLUSTER_COUNT, lanes, &|cluster| {
                let mut part = parts[cluster].lock().unwrap_or_else(PoisonError::into_inner);
                part.reserve(numbers[cluster].iter().filter(|number| **number != ABSENT).count());
                for (key, number) in clusters[cluster].keys.iter().zip(&numbers[cluster]) {
                    if *number != ABSENT {
                        part.insert(*key, *number);
                    }
                }
            });
            self.table.child.put(parts.into_iter().map(|part| part.into_inner().unwrap_or_else(PoisonError::into_inner)).collect());
        }
        let Table { start, readings: all, .. } = &mut self.table;
        let out = SharedReadings::over(all);
        on_lanes(CLUSTER_COUNT, lanes, &|cluster| unsafe { write_readings(&clusters[cluster], &numbers[cluster], start, &logs, &place_maps, &out) });
        made
    }

    pub fn count_one(&mut self, names: &Symbols) {
        let runs = self.runs();
        let this = &*self;
        let parts: Vec<RunLog> = on_lanes(runs.len(), this.lanes, &|run| {
            let mut finder = Finder::over(this.words, &this.point_kinds, this.star, Grow::on(&this.forms, &this.steps, this.near_numbers()));
            let mut log = RunLog::default();
            let mut steps = Steps::default();
            let mut place: Vec<u32> = Vec::new();
            for number in runs[run].clone() {
                let ready = &this.ready[number];
                let asked = this.asked.number(ready.case.form.as_slice());
                finder.open(&ready.case);
                log.lesson(number as u32);
                place_of(&ready.answer, &ready.case, &[], &[], &mut place);
                let at = log.place(&this.places, &place);
                log.add(asked, ABSENT, at);
                finder.first_steps(&ready.case, &mut steps);
                for found in &steps.rows {
                    if place_of(&ready.answer, &ready.case, steps.slots_of(found), &[], &mut place) {
                        let at = log.place(&this.places, &place);
                        log.add(asked, found.step, at);
                    }
                }
            }
            this.keep_numbers(finder.catalog.kept());
            log
        });
        let made = self.join(parts, 1, true);
        self.mark(names, &made);
    }

    pub fn level_one(&self, finder: &mut Finder<Grow>, case: &Query, steps: &mut Steps, groups: &mut Groups) {
        groups.clear();
        let Some(root) = self.asked.find(case.form.as_slice()).and_then(|asked| self.table.child.get(ABSENT, asked)) else { return };
        if !self.table.has_wide[root as usize] {
            return;
        }
        finder.first_steps(case, steps);
        for found in &steps.rows {
            if let Some(pattern) = self.table.child.get(root, found.step) {
                if self.table.wide[pattern as usize] {
                    groups.add((found.sent, ABSENT), steps.slots_of(found), &[], pattern);
                }
            }
        }
    }

    pub fn level_two(&self, finder: &mut Finder<Grow>, case: &Query, steps: &mut Steps, from: &Groups, groups: &mut Groups) {
        groups.clear();
        for group in from.all() {
            let keys: Vec<u32> = from.keys(group).into_iter().filter(|key| self.table.has_wide[*key as usize]).collect();
            if keys.is_empty() {
                continue;
            }
            let chosen = from.chosen(group);
            finder.next_steps(case, FIRST_MORE, &chosen, from.slots_of(group), steps);
            for key in keys {
                for found in &steps.rows {
                    if let Some(pattern) = self.table.child.get(key, found.step) {
                        if self.table.wide[pattern as usize] {
                            groups.add((chosen[0], found.sent), from.slots_of(group), steps.slots_of(found), pattern);
                        }
                    }
                }
            }
        }
    }

    pub fn count_more(&mut self, names: &Symbols, depth: u32, only: Option<&[u32]>) {
        let runs = self.runs();
        let this = &*self;
        let parts: Vec<RunLog> = on_lanes(runs.len(), this.lanes, &|run| {
            let mut finder = Finder::over(this.words, &this.point_kinds, this.star, Grow::on(&this.forms, &this.steps, this.near_numbers()));
            let mut log = RunLog::default();
            let mut steps = Steps::default();
            let mut one = Groups::default();
            let mut two = Groups::default();
            let mut place: Vec<u32> = Vec::new();
            let mut at: Vec<u32> = Vec::new();
            for number in runs[run].clone() {
                let ready = &this.ready[number];
                let case = &ready.case;
                if only.is_some_and(|forms| !this.asked.find(case.form.as_slice()).is_some_and(|asked| forms.contains(&asked))) {
                    continue;
                }
                finder.open(case);
                log.lesson(number as u32);
                this.level_one(&mut finder, case, &mut steps, &mut one);
                if depth > FIRST_MORE {
                    this.level_two(&mut finder, case, &mut steps, &one, &mut two);
                }
                let groups = if depth > FIRST_MORE { &two } else { &one };
                for group in groups.all() {
                    finder.next_steps(case, depth, &groups.chosen(group), groups.slots_of(group), &mut steps);
                    at.clear();
                    for found in &steps.rows {
                        let fits = place_of(&ready.answer, case, groups.slots_of(group), steps.slots_of(found), &mut place);
                        at.push(if fits { log.place(&this.places, &place) } else { ABSENT });
                    }
                    for key in groups.keys(group) {
                        for (found, place) in steps.rows.iter().zip(&at) {
                            if *place != ABSENT {
                                log.add(key, found.step, *place);
                            }
                        }
                    }
                }
            }
            this.keep_numbers(finder.catalog.kept());
            log
        });
        let made = self.join(parts, LEAST_KEPT, false);
        self.mark(names, &made);
    }

    pub fn mark(&mut self, names: &Symbols, made: &[u32]) {
        let mut tops = Tops::default();
        let mut found: CheapMap<u32, Vec<u32>> = CheapMap::default();
        let mut groups: Vec<u32> = Vec::new();
        for pattern in made {
            let count = self.table.count[*pattern as usize];
            if count < self.sure_n || self.table.depth[*pattern as usize] == 0 {
                continue;
            }
            let (_, hit) = tops.top(self.table.of(*pattern));
            if f64::from(hit) >= self.sure_share * f64::from(count) {
                self.table.sure[*pattern as usize] = true;
                continue;
            }
            let group = if self.per_form { self.table.root(*pattern) } else { ABSENT };
            let row = found.entry(group).or_default();
            if row.is_empty() {
                groups.push(group);
            }
            row.push(*pattern);
        }
        for group in groups {
            let mut row = found.remove(&group).unwrap_or_default();
            row.sort_by(|a, b| self.table.count[*b as usize].cmp(&self.table.count[*a as usize]));
            if row.len() > MOST_WIDE {
                let edge = self.table.count[row[MOST_WIDE - 1] as usize];
                let above = row.iter().take_while(|pattern| self.table.count[**pattern as usize] > edge).count();
                let until = row.iter().take_while(|pattern| self.table.count[**pattern as usize] >= edge).count();
                let mut tied: Vec<(String, u32)> = row[above..until].iter().map(|pattern| (self.text(names, *pattern), *pattern)).collect();
                tied.sort_by(|a, b| b.0.cmp(&a.0));
                row.truncate(above);
                row.extend(tied.iter().take(MOST_WIDE - above).map(|(_, pattern)| *pattern));
            }
            for pattern in row {
                self.table.wide[pattern as usize] = true;
                let parent = self.table.parent[pattern as usize];
                if parent != ABSENT {
                    self.table.has_wide[parent as usize] = true;
                }
            }
        }
    }

    pub fn text(&self, names: &Symbols, pattern: u32) -> String {
        let root = self.table.root(pattern);
        let mut out = asked_text(names, &self.asked.row(self.table.step[root as usize]));
        for step in self.table.chain(pattern) {
            let key = self.steps.row(step);
            let form = if matches!(key.point, Point::Any | Point::Last | Point::End | Point::Near) { self.forms.row(key.item) } else { Vec::new() };
            out.push_str(" || ");
            out.push_str(&step_text(self.words, names, &key, &form));
        }
        out
    }

    pub fn place_name(&self, names: &Symbols, place: u32) -> String {
        self.places.row(place).map_or_else(|| "?".to_string(), |row| place_text(names, row))
    }
}

pub const FIRST_MORE: u32 = 2;
because!(FIRST_MORE, WideningBeam, "the number of the first sentence of a pattern that is found from an earlier sentence and not from the question alone");
