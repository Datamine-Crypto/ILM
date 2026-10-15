use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::hash::{CheapMap, mix};
use crate::tools::pattern::table::CountedPatterns;
use patterns::because;

#[derive(Clone, Copy, Debug, Default)]
pub struct GroupRow {
    first: u32,
    second: u32,
    from: u32,
    len: u32,
    head: u32,
    tail: u32,
    same: u32,
}
because!(GroupRow, CountedPatterns, "the patterns of a case that found the same sentences and have the same free slots: the one or two sentences, where the slots lie, the first and the last of its patterns in the chain of patterns, and the next group with the same hash");

#[derive(Default)]
pub struct Groups {
    rows: Vec<GroupRow>,
    slots: Vec<(u32, u32)>,
    links: Vec<(u32, u32)>,
    at: CheapMap<u64, u32>,
}
because!(Groups, CountedPatterns, "the patterns of one case in groups with the same sentences and the same free slots, in the order in which the groups came: the next sentence is searched one time for a group and not for each of its patterns. add puts a pattern into its group; all gives the numbers of the groups; chosen, slots_of and keys give the parts of a group");

impl Groups {
    pub fn clear(&mut self) {
        self.rows.clear();
        self.slots.clear();
        self.links.clear();
        self.at.clear();
    }

    pub fn all(&self) -> std::ops::Range<usize> {
        0..self.rows.len()
    }

    pub fn add(&mut self, sents: (u32, u32), slots: &[(u32, u32)], more: &[(u32, u32)], pattern: u32) {
        let mut hash = mix(u64::from(sents.0), u64::from(sents.1));
        for (name, word) in slots.iter().chain(more) {
            hash = mix(mix(hash, u64::from(*name)), u64::from(*word));
        }
        let link = self.links.len() as u32;
        self.links.push((pattern, ABSENT));
        let mut at = self.at.get(&hash).copied().unwrap_or(ABSENT);
        while at != ABSENT {
            let row = self.rows[at as usize];
            let held = &self.slots[row.from as usize..(row.from + row.len) as usize];
            if row.first == sents.0 && row.second == sents.1 && held.iter().eq(slots.iter().chain(more)) {
                self.links[row.tail as usize].1 = link;
                self.rows[at as usize].tail = link;
                return;
            }
            at = row.same;
        }
        let from = self.slots.len() as u32;
        self.slots.extend(slots.iter().chain(more));
        let same = self.at.insert(hash, self.rows.len() as u32).unwrap_or(ABSENT);
        self.rows.push(GroupRow { first: sents.0, second: sents.1, from, len: self.slots.len() as u32 - from, head: link, tail: link, same });
    }

    pub fn chosen(&self, group: usize) -> Vec<u32> {
        let row = &self.rows[group];
        [row.first, row.second].into_iter().filter(|sent| *sent != ABSENT).collect()
    }

    pub fn slots_of(&self, group: usize) -> &[(u32, u32)] {
        let row = &self.rows[group];
        &self.slots[row.from as usize..(row.from + row.len) as usize]
    }

    pub fn keys(&self, group: usize) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        let mut at = self.rows[group].head;
        while at != ABSENT {
            out.push(self.links[at as usize].0);
            at = self.links[at as usize].1;
        }
        out
    }
}
