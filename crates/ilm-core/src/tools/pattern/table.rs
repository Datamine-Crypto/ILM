use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::grouping::{CLUSTER_COUNT, cluster_of};
use crate::tools::pattern::hash::{CheapMap, mix};
use crate::tools::pattern::story::Query;
use patterns::{because, source};

pub struct CountedPatterns;
source!(CountedPatterns, "counting in place of fitting: every pattern that a lesson shows gets one reading with the place of the answer, a pattern is widened by one more sentence only where it is read often and is not sure, and readings add, so the lessons are counted in runs on all threads and the runs are joined in the order of the lessons");

pub const PLACE_BITS: u32 = 12;
because!(PLACE_BITS, CountedPatterns, "the bits of a reading that hold the place of the answer; the bits above them hold the number of the lesson, so a reading is one number and a pattern with a million readings is a flat array");

pub const PLACE_MASK: u32 = 0x0000_0FFF;
because!(PLACE_MASK, CountedPatterns, "the bits of the place in a reading; the highest value is the one number of every place that comes after the table of places is full");

pub const LOCAL_PLACE: u32 = 0x8000_0000;
because!(LOCAL_PLACE, CountedPatterns, "the bit of the number of a place that one run of lessons saw first: the place gets its number in the table of all runs when the runs are joined, in the order of the lessons");

pub const FEW_READINGS: usize = 40;
because!(FEW_READINGS, CountedPatterns, "up to this many readings the most frequent place of a pattern is found by a walk over the readings, where the place that comes first wins among equals; above it by a count for each place, where the place with the lowest number wins among equals");

#[derive(Clone, Debug, Default)]
pub struct Places {
    at: CheapMap<Vec<u32>, u32>,
    pub rows: Vec<Vec<u32>>,
}
because!(Places, CountedPatterns, "the places of the answer that the lessons showed, each with a number in the order of first sight: number gives the number of a place and takes a new place in while the table has room, and the one last number to every place after that; find gives the number of a place that is there; row gives the parts of a place, and nothing for the last number, which names no place");

impl Places {
    pub fn number(&mut self, row: &[u32]) -> u32 {
        if let Some(found) = self.at.get(row) {
            return *found;
        }
        if self.rows.len() >= PLACE_MASK as usize {
            return PLACE_MASK;
        }
        let number = self.rows.len() as u32;
        self.rows.push(row.to_vec());
        self.at.insert(row.to_vec(), number);
        number
    }

    pub fn find(&self, row: &[u32]) -> Option<u32> {
        self.at.get(row).copied()
    }

    pub fn row(&self, number: u32) -> Option<&[u32]> {
        self.rows.get(number as usize).map(Vec::as_slice)
    }
}

#[derive(Clone, Debug)]
pub struct Ready {
    pub case: Query,
    pub answer: Vec<u32>,
}
because!(Ready, CountedPatterns, "one lesson as it is counted: the case as it was read, and the words of its answer");

#[derive(Clone, Debug, Default)]
pub struct Table {
    pub parent: Vec<u32>,
    pub step: Vec<u32>,
    pub depth: Vec<u8>,
    pub start: Vec<usize>,
    pub count: Vec<u32>,
    pub wide: Vec<bool>,
    pub sure: Vec<bool>,
    pub has_child: Vec<bool>,
    pub has_wide: Vec<bool>,
    pub child: ChildTable,
    pub readings: Vec<u32>,
}
because!(Table, CountedPatterns, "the counted patterns as arrays by pattern number: the pattern with one sentence fewer, the last step (for a question form its number), the number of sentences, where its readings lie, how many they are, if it gets one more sentence, if it is sure, if a longer pattern starts with it, if a longer pattern that gets one more sentence starts with it; one table from a shorter pattern and a step to the longer pattern; and all readings in one array, each the number of a lesson above the number of a place. add puts one pattern in, add_unlinked one pattern that the table from the shorter pattern does not hold yet, and add_with one pattern with its readings, reserve_room makes room for many patterns and readings at once, of gives the readings of a pattern, root gives the question form of a pattern, chain its steps from the first sentence on");

pub struct OpenAddressing;
source!(OpenAddressing, "a table of keys in one flat array with linear probing, as Knuth describes it, where a key is looked for at the place that its hash gives and at the places after it: a key that is there is found with one read of memory in most cases, which matters for a large table that no cache holds, and the reads of many keys do not wait for each other");

const ROOM_PART: usize = 2;
because!(ROOM_PART, OpenAddressing, "the flat table has the places of its keys and this part of them more again, so that most keys stand at the place of their hash and a probe for a missing key ends after a short walk");

#[derive(Clone, Debug)]
pub struct ChildTable {
    parts: Vec<CheapMap<(u32, u32), u32>>,
    flat: Vec<(u32, u32, u32)>,
}
because!(ChildTable, CountedPatterns, OpenAddressing, "the table from a shorter pattern and a step to the longer pattern, in parts by the cluster of its key: the threads of the clusters of a count fill their parts side by side, and a part stays small while it is filled. When the counts of a model are done, freeze puts every key into one flat table, which the answers of the model read. get gives the longer pattern, insert puts one in, take lends the parts to the threads of a count and put gives them back");

impl Default for ChildTable {
    fn default() -> ChildTable {
        ChildTable { parts: vec![CheapMap::default(); CLUSTER_COUNT], flat: Vec::new() }
    }
}

fn flat_place(parent: u32, step: u32, room: usize) -> usize {
    let hash = mix(mix(0, u64::from(parent)), u64::from(step));
    ((u128::from(hash) * room as u128) >> u64::BITS) as usize
}
because!(flat_place, OpenAddressing, "the first place of a key in the flat table: the high bits of the product of its hash and the size of the table, so that the table can have any size");

impl ChildTable {
    pub fn get(&self, parent: u32, step: u32) -> Option<u32> {
        if self.flat.is_empty() {
            return self.parts[cluster_of(parent, step)].get(&(parent, step)).copied();
        }
        let room = self.flat.len();
        let mut at = flat_place(parent, step, room);
        loop {
            let (held_parent, held_step, child) = self.flat[at];
            if child == ABSENT {
                return None;
            }
            if held_parent == parent && held_step == step {
                return Some(child);
            }
            at += 1;
            if at == room {
                at = 0;
            }
        }
    }

    pub fn get_all(&self, parent: u32, steps: &[u32], out: &mut Vec<Option<u32>>) {
        out.clear();
        if !self.flat.is_empty() {
            let room = self.flat.len();
            let mut read = 0u32;
            for step in steps {
                read ^= self.flat[flat_place(parent, *step, room)].1;
            }
            std::hint::black_box(read);
        }
        out.extend(steps.iter().map(|step| self.get(parent, *step)));
    }

    pub fn freeze(&mut self) {
        let count: usize = self.parts.iter().map(|part| part.len()).sum();
        if count == 0 {
            return;
        }
        let room = count + count / ROOM_PART + 1;
        let mut flat: Vec<(u32, u32, u32)> = vec![(0, 0, ABSENT); room];
        for part in &self.parts {
            for ((parent, step), child) in part {
                let mut at = flat_place(*parent, *step, room);
                while flat[at].2 != ABSENT {
                    at += 1;
                    if at == room {
                        at = 0;
                    }
                }
                flat[at] = (*parent, *step, *child);
            }
        }
        self.flat = flat;
        self.parts = Vec::new();
    }

    pub fn insert(&mut self, parent: u32, step: u32, number: u32) {
        self.parts[cluster_of(parent, step)].insert((parent, step), number);
    }

    pub fn take(&mut self) -> Vec<CheapMap<(u32, u32), u32>> {
        std::mem::take(&mut self.parts)
    }

    pub fn put(&mut self, parts: Vec<CheapMap<(u32, u32), u32>>) {
        self.parts = parts;
    }
}

impl Table {
    pub fn add(&mut self, parent: u32, step: u32, readings: usize) -> u32 {
        let number = self.add_unlinked(parent, step, readings);
        self.child.insert(parent, step, number);
        number
    }

    pub fn add_unlinked(&mut self, parent: u32, step: u32, readings: usize) -> u32 {
        let number = self.parent.len() as u32;
        let depth = if parent == ABSENT { 0 } else { self.depth[parent as usize] + 1 };
        self.parent.push(parent);
        self.step.push(step);
        self.depth.push(depth);
        self.start.push(self.readings.len());
        self.count.push(readings as u32);
        self.wide.push(false);
        self.sure.push(false);
        self.has_child.push(false);
        self.has_wide.push(false);
        self.readings.resize(self.readings.len() + readings, 0);
        if parent != ABSENT {
            self.has_child[parent as usize] = true;
        }
        number
    }

    pub fn reserve_room(&mut self, patterns: usize, readings: usize) {
        self.parent.reserve(patterns);
        self.step.reserve(patterns);
        self.depth.reserve(patterns);
        self.start.reserve(patterns);
        self.count.reserve(patterns);
        self.wide.reserve(patterns);
        self.sure.reserve(patterns);
        self.has_child.reserve(patterns);
        self.has_wide.reserve(patterns);
        self.readings.reserve(readings);
    }

    pub fn add_with(&mut self, parent: u32, step: u32, readings: &[u32]) -> u32 {
        let pattern = self.add(parent, step, readings.len());
        let from = self.start[pattern as usize];
        self.readings[from..from + readings.len()].copy_from_slice(readings);
        pattern
    }

    pub fn of(&self, pattern: u32) -> &[u32] {
        let from = self.start[pattern as usize];
        &self.readings[from..from + self.count[pattern as usize] as usize]
    }

    pub fn root(&self, pattern: u32) -> u32 {
        let mut at = pattern;
        while self.parent[at as usize] != ABSENT {
            at = self.parent[at as usize];
        }
        at
    }

    pub fn chain(&self, pattern: u32) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        let mut at = pattern;
        while self.parent[at as usize] != ABSENT {
            out.push(self.step[at as usize]);
            at = self.parent[at as usize];
        }
        out.reverse();
        out
    }
}

#[derive(Clone, Debug, Default)]
pub struct Tops {
    seen: Vec<(u32, u32)>,
    count: Vec<u32>,
    touched: Vec<u32>,
}
because!(Tops, CountedPatterns, "the most frequent place of the readings of one pattern, with arrays that are kept between calls. top gives the place and its readings; with few readings the place that comes first wins among equals, with many the place with the lowest number, and then a table of all places is counted where only the places that the readings name are touched and set back");

impl Tops {
    pub fn top(&mut self, readings: &[u32]) -> (u32, u32) {
        if readings.len() > FEW_READINGS {
            if self.count.is_empty() {
                self.count.resize(PLACE_MASK as usize + 1, 0);
            }
            self.touched.clear();
            for reading in readings {
                let place = (reading & PLACE_MASK) as usize;
                if self.count[place] == 0 {
                    self.touched.push(place as u32);
                }
                self.count[place] += 1;
            }
            let mut best = (0, 0);
            for place in &self.touched {
                let count = self.count[*place as usize];
                if count > best.1 || (count == best.1 && *place < best.0) {
                    best = (*place, count);
                }
                self.count[*place as usize] = 0;
            }
            return best;
        }
        self.seen.clear();
        for reading in readings {
            let place = reading & PLACE_MASK;
            match self.seen.iter_mut().find(|(held, _)| *held == place) {
                Some(row) => row.1 += 1,
                None => self.seen.push((place, 1)),
            }
        }
        let mut best = (0, 0);
        for row in &self.seen {
            if row.1 > best.1 {
                best = *row;
            }
        }
        best
    }
}

impl Places {
    pub fn number_wide(&mut self, row: &[u32]) -> u32 {
        if let Some(found) = self.at.get(row) {
            return *found;
        }
        let number = self.rows.len() as u32;
        self.rows.push(row.to_vec());
        self.at.insert(row.to_vec(), number);
        number
    }
}
