use crate::base::runs::RunNumbers;
use crate::base::sums::compensated;
use crate::tools::requests::frames::{FrameKey, FramePart, PlaceKeys, SIDE_LEFT, SIDE_RIGHT, frame_joined, frame_near, frame_short};
use patterns::{because, source};

pub struct ShareLift;
source!(ShareLift, "the shares of the tags of a word, changed by each side of its frame by itself. The share under a key of the word is its count with the share under the shorter key as a few more readings. What a side does is counted in steps: the share of each tag is multiplied by its count under the side over the count that the share before expects, each with a half more, so one reading that was not expected lifts a tag a few times and not a hundred times, and a side that says what the shares say changes nothing. The word is the pivot of a place, and the shares of a row of tags are not made to sum to one, so a row is judged as a whole");

pub(super) const SHARE_SMOOTH: f64 = 2.0;
because!(SHARE_SMOOTH, ShareLift, "the readings that a share known before counts for when a share is taken from counts: under a key of the word the share under the shorter key, for the price of a second run the share of all slots, and for the slots of a whole request the share among all requests; so few counts move a share little");

const SHARE_MORE: f64 = 0.5;
because!(SHARE_MORE, ShareLift, "what is added to the count of a tag under a side and to the count that the share before expects: one reading of a tag that was not expected then lifts it a few times, not without end");

pub(super) const SHARE_LEAST: u32 = 2;
because!(SHARE_LEAST, ShareLift, "the least readings under a key for a step of the lift, and under a key that is kept: one reading says nothing about a side");

const SHARE_NOTHING: f64 = 0.01;
because!(SHARE_NOTHING, ShareLift, "a tag with no count under a side and a share that expects fewer readings than this there is not changed by the side: the side could not have shown it");

const SHARE_NEAR: f64 = 1.0;
because!(SHARE_NEAR, ShareLift, "the weight of the words a few places away together, as a number of views of a place: each of them alone is a weak view, so together they count as much as one side");

#[derive(Clone, Debug, Default)]
pub(super) struct KeyCounts {
    pub(super) runs: RunNumbers,
    pub(super) tags: Vec<Vec<(u32, u32)>>,
    pub(super) totals: Vec<u32>,
    pub(super) kept: Vec<bool>,
}
because!(KeyCounts, ShareLift, "the counts of the tags under each key: the tags with their counts in the order in which the lessons first showed them, the readings of the key, and whether the key is kept. add counts one reading; keep keeps a key with enough readings, or a short key of the word; find gives a kept key; total gives the readings of a kept key, and none for any other key");

impl KeyCounts {
    pub(super) fn add(&mut self, key: &[u32], tag: u32) {
        let number = self.runs.number(key) as usize;
        if number == self.totals.len() {
            self.tags.push(Vec::new());
            self.totals.push(0);
            self.kept.push(true);
        }
        self.totals[number] += 1;
        match self.tags[number].iter_mut().find(|(held, _)| *held == tag) {
            Some((_, count)) => *count += 1,
            None => self.tags[number].push((tag, 1)),
        }
    }

    pub(super) fn keep(&mut self) {
        for number in 0..self.totals.len() {
            self.kept[number] = self.totals[number] >= SHARE_LEAST || frame_short(self.runs.run(number as u32));
        }
    }

    pub(super) fn find(&self, key: &[u32]) -> Option<usize> {
        self.runs.find(key).map(|number| number as usize).filter(|number| self.kept[*number])
    }

    pub(super) fn total(&self, key: &[u32]) -> u32 {
        self.find(key).map_or(0, |number| self.totals[number])
    }
}

#[derive(Debug, Default)]
pub(super) struct ShareHand {
    counts: Vec<u32>,
    joined: Vec<u32>,
    fresh: Vec<f64>,
}
because!(ShareHand, ShareLift, "the working memory of one reading: the counts of one key spread over all tags, a joined key, and the new shares of a step; kept from place to place");

pub(super) type ShareLevel = (FrameKey, Vec<f64>);
because!(ShareLevel, ShareLift, "a key of the row of a word with the shares under it");

#[derive(Clone, Debug, Default)]
pub(super) struct ShareTable {
    pub(super) keys: KeyCounts,
    pub(super) present: Vec<u32>,
    pub(super) all: Vec<f64>,
}
because!(ShareTable, ShareLift, "the counted keys with the tags that the lessons hold, in the order of first sight, and the share of each tag in all places. under gives the shares under one more key of the word; step gives what one side does to the shares of a word in one step; lifted gives what a side does in two steps, with all words and then with the fullest key of the word that the lessons hold with it; place gives the shares of a word after its sides of words and the words a few places away; shares gives them after the slot before too");

impl ShareTable {
    fn spread(&self, number: usize, hand: &mut ShareHand, on: bool) {
        if hand.counts.len() < self.all.len() {
            hand.counts.resize(self.all.len(), 0);
        }
        for (tag, count) in &self.keys.tags[number] {
            hand.counts[*tag as usize] = if on { *count } else { 0 };
        }
    }

    fn normalized(&self, est: &mut [f64]) {
        let total = compensated(self.present.iter().map(|tag| est[*tag as usize]));
        let total = if total == f64::default() { f64::from(1u8) } else { total };
        for tag in &self.present {
            est[*tag as usize] /= total;
        }
    }

    pub(super) fn under(&self, number: usize, share: &[f64], hand: &mut ShareHand) -> Vec<f64> {
        self.spread(number, hand, true);
        let readings = f64::from(self.keys.totals[number]);
        let mut out = vec![f64::default(); self.all.len()];
        for tag in &self.present {
            let tag = *tag as usize;
            out[tag] = (f64::from(hand.counts[tag]) + SHARE_SMOOTH * share[tag]) / (readings + SHARE_SMOOTH);
        }
        self.spread(number, hand, false);
        out
    }

    pub(super) fn step(&self, pivot: &[u32], by_pivot: &[f64], keys: &[FrameKey], lift: Option<&[f64]>, hand: &mut ShareHand) -> Option<Vec<f64>> {
        let mut est = by_pivot.to_vec();
        if let Some(lift) = lift {
            for tag in &self.present {
                let tag = *tag as usize;
                est[tag] = by_pivot[tag] * lift[tag];
            }
            self.normalized(&mut est);
        }
        let mut moved = false;
        for key in keys {
            frame_joined(pivot, key, &mut hand.joined);
            let Some(number) = self.keys.find(&hand.joined) else { continue };
            let total = self.keys.totals[number];
            if total < SHARE_LEAST {
                continue;
            }
            let readings = f64::from(total);
            self.spread(number, hand, true);
            hand.fresh.clear();
            hand.fresh.extend_from_slice(&est);
            let mut changed = false;
            for tag in &self.present {
                let tag = *tag as usize;
                let (expected, count) = (est[tag], hand.counts[tag]);
                if count > 0 || expected * readings > SHARE_NOTHING {
                    hand.fresh[tag] = expected * (f64::from(count) + SHARE_MORE) / (readings * expected + SHARE_MORE);
                    changed = true;
                }
            }
            self.spread(number, hand, false);
            if changed {
                est.copy_from_slice(&hand.fresh);
                self.normalized(&mut est);
                moved = true;
            }
        }
        if !moved && lift.is_none() {
            return None;
        }
        let mut out = vec![f64::from(1u8); self.all.len()];
        for tag in &self.present {
            let tag = *tag as usize;
            if by_pivot[tag] > f64::default() {
                out[tag] = est[tag] / by_pivot[tag];
            }
        }
        Some(out)
    }

    pub(super) fn lifted(&self, levels: &[ShareLevel], keys: &[FrameKey], hand: &mut ShareHand) -> Option<Vec<f64>> {
        let wide = self.step(&[], &self.all, keys, None, hand);
        for (pivot, by_pivot) in levels.iter().rev() {
            let held = keys.iter().any(|key| {
                frame_joined(pivot, key, &mut hand.joined);
                self.keys.total(&hand.joined) >= SHARE_LEAST
            });
            if held {
                return self.step(pivot, by_pivot, keys, wide.as_deref(), hand);
            }
        }
        match (levels.last(), wide.as_deref()) {
            (Some((pivot, by_pivot)), Some(lift)) => self.step(pivot, by_pivot, keys, Some(lift), hand),
            _ => wide,
        }
    }

    pub(super) fn place(&self, frame: &PlaceKeys, hand: &mut ShareHand) -> (Vec<ShareLevel>, Vec<f64>) {
        let mut levels: Vec<ShareLevel> = Vec::new();
        let mut share = self.all.clone();
        for key in &frame.word {
            if let Some(number) = self.keys.find(key) {
                share = self.under(number, &share, hand);
                levels.push((key.clone(), share.clone()));
            }
        }
        for side in [SIDE_LEFT, SIDE_RIGHT] {
            if let Some(lift) = self.lifted(&levels, &frame.sides[side], hand) {
                for tag in &self.present {
                    share[*tag as usize] *= lift[*tag as usize];
                }
            }
        }
        let near_key = frame_near(&frame.word);
        let near: Vec<ShareLevel> = levels.iter().filter(|(key, _)| Some(key) == near_key).cloned().collect();
        let lifts: Vec<Vec<f64>> = frame.around.iter().filter(|key| self.keys.find(key).is_some()).filter_map(|key| self.lifted(&near, std::slice::from_ref(key), hand)).collect();
        if !lifts.is_empty() {
            let power = SHARE_NEAR / (lifts.len() as f64);
            for lift in &lifts {
                for tag in &self.present {
                    let tag = *tag as usize;
                    share[tag] *= lift[tag].powf(power);
                }
            }
        }
        (levels, share)
    }

    pub(super) fn shares(&self, levels: &[ShareLevel], share: &[f64], before: u32, hand: &mut ShareHand) -> Vec<f64> {
        let mut out = share.to_vec();
        if let Some(lift) = self.lifted(levels, &[vec![FramePart::Slot as u32, before]], hand) {
            for tag in &self.present {
                out[*tag as usize] *= lift[*tag as usize];
            }
        }
        out
    }
}
