use crate::base::symbols::Symbols;
use crate::tools::pattern::codes::{ASKED_SLOT, EARLIER, Point, Scope, Step, name_step};
use crate::tools::pattern::counting::Counting;
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::table::{PLACE_BITS, PLACE_MASK};
use crate::tools::pattern::texts::{asked_text, step_names};
use patterns::{because, source};
use std::cmp::Ordering;

pub struct CoveringList;
source!(CoveringList, "sequential covering, the method of the AQ programs of Michalski and of CN2 of Clark and Niblett: the surest rule over all cases is taken first, the cases that it fits are set aside, the counts are taken again on the rest, and so on; a new case is answered by the first rule of the list that fits it");

pub const BEFORE_READINGS: u32 = 3;
because!(BEFORE_READINGS, CoveringList, "the readings that every rule has before the lessons, one for it and two against it, in the way of the rule of succession of Laplace: a rule with few readings is then less sure than a rule with many readings at the same share of hits");

pub const BITS_WORD: usize = 64;
because!(BITS_WORD, CoveringList, "the cases in one machine word of a set of cases: the sets of the covering are joined and counted a word at a time");

#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub pattern: u32,
    pub place: u32,
    pub hit: u32,
    pub readings: u32,
}
because!(Rule, CoveringList, "one rule of the list: the pattern, the place of the answer, and its readings and hits on the cases that the rules before it left; share gives the share of its hits with one reading added for it and two against it");

impl Rule {
    pub fn share(&self) -> f64 {
        f64::from(self.hit + 1) / f64::from(self.readings + BEFORE_READINGS)
    }
}

pub struct Candidate {
    pattern: u32,
    depth: u8,
    fits: Vec<u64>,
    tops: Vec<(u32, Vec<u64>)>,
}
because!(Candidate, CoveringList, "a pattern that can be a rule of the list of one question form: its number, its sentences, the cases of the form that it fits as a set of bits, and for each place with enough readings the cases where the answer stands there");

pub fn bits_and_count(a: &[u64], b: &[u64]) -> u32 {
    a.iter().zip(b).map(|(x, y)| (x & y).count_ones()).sum()
}
because!(bits_and_count, CoveringList, "how many cases two sets of cases share, a machine word at a time");

impl<'a> Counting<'a> {
    pub fn looks_up_all(&self, pattern: u32) -> bool {
        let root = self.table.root(pattern);
        let asked = self.asked.row(self.table.step[root as usize]);
        let slots = asked.iter().filter(|cell| *cell & ASKED_SLOT != 0).count();
        let chain: Vec<Step> = self.table.chain(pattern).into_iter().map(|step| self.steps.row(step)).collect();
        if slots == 0 || chain.first().is_some_and(|step| step.point == Point::Split) {
            return true;
        }
        let nodes = slots + chain.len() + 1;
        let mut group: Vec<usize> = (0..nodes).collect();
        let mut touched: Vec<bool> = vec![false; slots];
        let top = |group: &mut Vec<usize>, mut node: usize| {
            while group[node] != node {
                node = group[node];
            }
            node
        };
        for (place, step) in chain.iter().enumerate() {
            let here = slots + place + 1;
            let form = if matches!(step.point, Point::Any | Point::Last | Point::End | Point::Near) { self.forms.row(step.item) } else { Vec::new() };
            for name in step_names(step, &form) {
                let other = if name & EARLIER == 0 {
                    let slot = (name as usize).saturating_sub(1);
                    if slot >= slots {
                        continue;
                    }
                    touched[slot] = true;
                    slot
                } else {
                    slots + name_step(name) as usize
                };
                let (a, b) = (top(&mut group, here), top(&mut group, other.min(nodes - 1)));
                group[a] = b;
            }
            if matches!(step.scope, Scope::Next | Scope::Prev) {
                let (a, b) = (top(&mut group, here), top(&mut group, here - 1));
                group[a] = b;
            }
        }
        let first = top(&mut group, 0);
        touched.iter().all(|seen| *seen) && (0..slots).all(|slot| top(&mut group, slot) == first)
    }

    pub fn open_forms(&self, names: &Symbols) -> Vec<(u32, u32)> {
        let mut open: CheapMap<u32, u32> = CheapMap::default();
        let mut order: Vec<u32> = Vec::new();
        for (number, ready) in self.ready.iter().enumerate() {
            if self.alive[number] {
                if let Some(asked) = self.asked.find(ready.case.form.as_slice()) {
                    let count = open.entry(asked).or_insert(0);
                    if *count == 0 {
                        order.push(asked);
                    }
                    *count += 1;
                }
            }
        }
        let mut out: Vec<(String, u32, u32)> = order.iter().filter(|asked| open[*asked] >= self.sure_n).map(|asked| (asked_text(names, &self.asked.row(*asked)), *asked, open[asked])).collect();
        out.sort();
        out.into_iter().map(|(_, asked, count)| (asked, count)).collect()
    }

    pub fn candidates(&self, by_form: &CheapMap<u32, Vec<u32>>, local: &[u32]) -> CheapMap<u32, Vec<Candidate>> {
        let need = f64::from(self.sure_n) * self.sure_share;
        let mut out: CheapMap<u32, Vec<Candidate>> = CheapMap::default();
        let mut count: Vec<u32> = vec![0; PLACE_MASK as usize + 1];
        let mut touched: Vec<u32> = Vec::new();
        for pattern in 0..self.table.parent.len() as u32 {
            let readings = self.table.of(pattern);
            if (readings.len() as u32) < self.sure_n {
                continue;
            }
            touched.clear();
            for reading in readings {
                let place = (reading & PLACE_MASK) as usize;
                if count[place] == 0 {
                    touched.push(place as u32);
                }
                count[place] += 1;
            }
            touched.sort_unstable();
            let tops: Vec<u32> = touched.iter().copied().filter(|place| f64::from(count[*place as usize]) >= need).collect();
            for place in &touched {
                count[*place as usize] = 0;
            }
            if tops.is_empty() {
                continue;
            }
            let asked = self.table.step[self.table.root(pattern) as usize];
            let size = by_form.get(&asked).map_or(0, Vec::len).div_ceil(BITS_WORD);
            let mut fits: Vec<u64> = vec![0; size];
            let mut rights: Vec<(u32, Vec<u64>)> = tops.iter().map(|place| (*place, vec![0; size])).collect();
            for reading in readings {
                let at = local[(reading >> PLACE_BITS) as usize] as usize;
                fits[at / BITS_WORD] |= 1 << (at % BITS_WORD);
                if let Some((_, right)) = rights.iter_mut().find(|(place, _)| *place == reading & PLACE_MASK) {
                    right[at / BITS_WORD] |= 1 << (at % BITS_WORD);
                }
            }
            out.entry(asked).or_default().push(Candidate { pattern, depth: self.table.depth[pattern as usize], fits, tops: rights });
        }
        out
    }

    pub fn make_list(&mut self, names: &Symbols) {
        let mut by_form: CheapMap<u32, Vec<u32>> = CheapMap::default();
        let mut local: Vec<u32> = vec![0; self.ready.len()];
        let mut forms: Vec<(String, u32)> = Vec::new();
        for (number, ready) in self.ready.iter().enumerate() {
            let Some(asked) = self.asked.find(ready.case.form.as_slice()) else { continue };
            let row = by_form.entry(asked).or_default();
            if row.is_empty() {
                forms.push((asked_text(names, &ready.case.form), asked));
            }
            local[number] = row.len() as u32;
            row.push(number as u32);
        }
        forms.sort();
        let mut candidates = self.candidates(&by_form, &local);
        let mut tested: CheapMap<u32, bool> = CheapMap::default();
        let mut texts: CheapMap<u32, String> = CheapMap::default();
        let mut order: Vec<Rule> = Vec::new();
        let mut open: Vec<bool> = vec![false; self.ready.len()];
        for (_, asked) in &forms {
            let cases = &by_form[asked];
            let mut mine: Vec<Candidate> = candidates.remove(asked).unwrap_or_default();
            let mut alive: Vec<u64> = vec![0; cases.len().div_ceil(BITS_WORD)];
            for at in 0..cases.len() {
                alive[at / BITS_WORD] |= 1 << (at % BITS_WORD);
            }
            while !mine.is_empty() {
                let mut sure: Vec<(usize, Rule)> = Vec::new();
                let mut live: Vec<bool> = Vec::with_capacity(mine.len());
                for (index, candidate) in mine.iter().enumerate() {
                    let readings = bits_and_count(&candidate.fits, &alive);
                    live.push(readings >= self.sure_n);
                    if readings < self.sure_n {
                        continue;
                    }
                    for (place, right) in &candidate.tops {
                        let hit = bits_and_count(right, &alive);
                        if f64::from(hit) >= self.sure_share * f64::from(readings) {
                            sure.push((index, Rule { pattern: candidate.pattern, place: *place, hit, readings }));
                        }
                    }
                }
                let mut best: Option<(usize, Rule)> = None;
                for found in &sure {
                    if best.is_none_or(|top| self.surer(names, &mut texts, &mine, found, &top)) {
                        best = Some(*found);
                    }
                }
                let Some(mut taken) = best else { break };
                if !*tested.entry(taken.1.pattern).or_insert_with(|| self.looks_up_all(taken.1.pattern)) {
                    let mut wide: Option<(usize, Rule)> = None;
                    for found in &sure {
                        let enough = f64::from(found.1.readings) >= self.sure_share * f64::from(taken.1.readings);
                        if enough && *tested.entry(found.1.pattern).or_insert_with(|| self.looks_up_all(found.1.pattern)) && wide.is_none_or(|top| self.surer(names, &mut texts, &mine, found, &top)) {
                            wide = Some(*found);
                        }
                    }
                    taken = wide.unwrap_or(taken);
                }
                order.push(taken.1);
                for (word, fits) in alive.iter_mut().zip(&mine[taken.0].fits) {
                    *word &= !fits;
                }
                let mut keep = live.into_iter();
                mine.retain(|_| keep.next().unwrap_or(false));
            }
            for (at, number) in cases.iter().enumerate() {
                if alive[at / BITS_WORD] >> (at % BITS_WORD) & 1 == 1 {
                    open[*number as usize] = true;
                }
            }
        }
        self.order = order;
        self.alive = open;
    }

    pub fn surer(&self, names: &Symbols, texts: &mut CheapMap<u32, String>, mine: &[Candidate], found: &(usize, Rule), top: &(usize, Rule)) -> bool {
        let by_share = found.1.share().partial_cmp(&top.1.share()).unwrap_or(Ordering::Equal);
        let by_count = by_share.then(found.1.hit.cmp(&top.1.hit)).then(mine[top.0].depth.cmp(&mine[found.0].depth));
        if by_count != Ordering::Equal {
            return by_count == Ordering::Greater;
        }
        for pattern in [found.1.pattern, top.1.pattern] {
            if !texts.contains_key(&pattern) {
                texts.insert(pattern, self.text(names, pattern));
            }
        }
        texts[&found.1.pattern] > texts[&top.1.pattern]
    }

    pub fn left(&self) -> usize {
        self.alive.iter().filter(|open| **open).count()
    }
}
