use crate::base::keys::packed;
use crate::base::sums::compensated;
use crate::tools::requests::shares::SHARE_SMOOTH;
use patterns::{because, source};
use std::cmp::Ordering;

pub struct NearestRequests;
source!(NearestRequests, "the nearest lesson requests to a request, by the marks that they share: each word and each pair of words side by side is a mark, and a mark counts with what it says about the answer, the divergence of Kullback and Leibler of the answers of the lesson requests with the mark from the answers of all lesson requests, with the answers of all as a few more readings under the mark; two requests are near when the squared weights of their shared marks are large beside the lengths of the two, as the cosine of two rows of weights");

pub struct LabelProduct;
source!(LabelProduct, "naive Bayes, the independence model of Maron, for the label of a whole request: the label with the best product of its own share and of the shares of the marks of the request among its lesson requests, where the share of a mark among all lesson requests counts as one more request");

const WHOLE_SURE: f64 = 5.0;
because!(WHOLE_SURE, NearestRequests, "the readings that the answers of all lesson requests count for under a mark: a mark that a few requests hold says little about the answer");

const WHOLE_HOLDERS: usize = 10;
because!(WHOLE_HOLDERS, NearestRequests, "the lesson requests that vote for the slots of a request: enough that one odd request does not decide, few enough that they are near");

const WHOLE_NEAREST: usize = 5;
because!(WHOLE_NEAREST, NearestRequests, "the lesson requests that vote for the label of a whole request");

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct MarkEdges {
    pub(super) start: u32,
    pub(super) end: u32,
}
because!(MarkEdges, NearestRequests, "the marks before the first word and after the last word of a request, which stand in the first and the last pair of words");

pub(super) fn request_marks(seen: &[u32], edges: MarkEdges, number: &mut dyn FnMut(u128) -> Option<u32>) -> Vec<u32> {
    let mut found: Vec<u32> = Vec::new();
    let mut one = |key: u128, found: &mut Vec<u32>| {
        if let Some(mark) = number(key)
            && !found.contains(&mark)
        {
            found.push(mark);
        }
    };
    for word in seen {
        one(packed(&[*word]), &mut found);
    }
    let before = std::iter::once(edges.start).chain(seen.iter().copied());
    let after = seen.iter().copied().chain(std::iter::once(edges.end));
    for (left, right) in before.zip(after) {
        one(packed(&[left, right]), &mut found);
    }
    found
}
because!(request_marks, NearestRequests, "the marks of a request as numbers, each once: its words as a frame writes them, then each pair of words side by side with the edges; the given table gives the number of a mark, a new number while learning and nothing for a mark that no lesson had while reading, which is then left out");

fn whole_spread(lessons: &[Vec<u32>], marks: usize) -> Vec<Vec<u32>> {
    let mut holds: Vec<Vec<u32>> = vec![Vec::new(); marks];
    for (number, lesson) in lessons.iter().enumerate() {
        for mark in lesson {
            holds[*mark as usize].push(number as u32);
        }
    }
    holds
}
because!(whole_spread, NearestRequests, "for each mark the lesson requests that hold it, in their order");

fn whole_nearest(holds: &[Vec<u32>], weights: &[f64], norms: &[f64], marks: &[u32], most: usize) -> Vec<(f64, u32)> {
    let mut shared: Vec<f64> = vec![f64::default(); norms.len()];
    let mut seen = vec![false; norms.len()];
    let mut touched: Vec<u32> = Vec::new();
    for mark in marks {
        let mark = *mark as usize;
        if holds.get(mark).is_none_or(Vec::is_empty) || weights[mark] <= f64::default() {
            continue;
        }
        let square = weights[mark] * weights[mark];
        for number in &holds[mark] {
            if !seen[*number as usize] {
                seen[*number as usize] = true;
                touched.push(*number);
            }
            shared[*number as usize] += square;
        }
    }
    let mine = compensated(marks.iter().filter(|mark| holds.get(**mark as usize).is_some_and(|held| !held.is_empty())).map(|mark| weights[*mark as usize] * weights[*mark as usize])).sqrt();
    let mine = if mine == f64::default() { f64::from(1u8) } else { mine };
    let mut near: Vec<(f64, u32)> = touched.iter().map(|number| (shared[*number as usize] / (mine * norms[*number as usize]), *number)).collect();
    near.sort_by(|left, right| right.0.partial_cmp(&left.0).unwrap_or(Ordering::Equal).then(right.1.cmp(&left.1)));
    near.truncate(most);
    near
}
because!(whole_nearest, NearestRequests, "the nearest lesson requests to the marks of a request with their nearness, the nearest first and of equals the later request first: the summed squared weights of the shared marks over the length of the request and the length of the lesson request");

fn whole_norms(lessons: &[Vec<u32>], weights: &[f64]) -> Vec<f64> {
    lessons
        .iter()
        .map(|marks| {
            let norm = compensated(marks.iter().map(|mark| weights[*mark as usize] * weights[*mark as usize])).sqrt();
            if norm == f64::default() { f64::from(1u8) } else { norm }
        })
        .collect()
}
because!(whole_norms, NearestRequests, "the length of each lesson request: the root of the sum of the squared weights of its marks, and one for a request whose marks say nothing");

#[derive(Clone, Debug, Default)]
pub(super) struct HeldSlots {
    slots: Vec<Vec<u32>>,
    base: Vec<(u32, u32)>,
    total: f64,
    holds: Vec<Vec<u32>>,
    weights: Vec<f64>,
    norms: Vec<f64>,
}
because!(HeldSlots, NearestRequests, "which slots a request has, by the vote of the nearest lesson requests: the slots of each lesson request, how many lesson requests hold each slot in the order of first sight, for each mark the lesson requests with it and its weight, and the length of each lesson request. lifts gives for each slot its share among the nearest lesson requests, with its share among all as a few more votes, over its share among all");

impl HeldSlots {
    pub(super) fn new(lessons: Vec<(Vec<u32>, Vec<u32>)>, marks: usize) -> HeldSlots {
        let (slots, marked): (Vec<Vec<u32>>, Vec<Vec<u32>>) = lessons.into_iter().unzip();
        let total = slots.len() as f64;
        let mut base: Vec<(u32, u32)> = Vec::new();
        for lesson in &slots {
            for slot in lesson {
                match base.iter_mut().find(|(held, _)| held == slot) {
                    Some((_, count)) => *count += 1,
                    None => base.push((*slot, 1)),
                }
            }
        }
        let holds = whole_spread(&marked, marks);
        let mut weights = vec![f64::default(); marks];
        for (mark, held) in holds.iter().enumerate() {
            if held.is_empty() {
                continue;
            }
            let readings = held.len() as f64;
            let mut said = f64::default();
            for (slot, count) in &base {
                let share = f64::from(*count) / total;
                if share <= f64::default() || share >= f64::from(1u8) {
                    continue;
                }
                let with = held.iter().filter(|number| slots[**number as usize].contains(slot)).count() as f64;
                let under = (with + WHOLE_SURE * share) / (readings + WHOLE_SURE);
                let other = f64::from(1u8) - under;
                said += under * (under / share).ln() + other * (other / (f64::from(1u8) - share)).ln();
            }
            weights[mark] = if said > f64::default() { said } else { f64::default() };
        }
        let norms = whole_norms(&marked, &weights);
        HeldSlots { slots, base, total, holds, weights, norms }
    }

    pub(super) fn lifts(&self, marks: &[u32], slots: usize) -> Vec<Option<f64>> {
        let mut out: Vec<Option<f64>> = vec![None; slots];
        if self.slots.is_empty() {
            return out;
        }
        let near = whole_nearest(&self.holds, &self.weights, &self.norms, marks, WHOLE_HOLDERS);
        let mut votes = vec![f64::default(); slots];
        let mut total = f64::default();
        for (share, number) in &near {
            let square = share * share;
            total += square;
            for slot in &self.slots[*number as usize] {
                votes[*slot as usize] += square;
            }
        }
        for (slot, count) in &self.base {
            let share = f64::from(*count) / self.total;
            out[*slot as usize] = Some((votes[*slot as usize] + SHARE_SMOOTH * f64::from(*count) / self.total) / ((total + SHARE_SMOOTH) * share));
        }
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LabelWay {
    Product,
    Vote,
}
because!(LabelWay, NearestRequests, LabelProduct, "the two ways to read the label of a whole request: the product of the shares of its marks, and the vote of the nearest lesson requests");

#[derive(Clone, Debug)]
pub(super) struct LabelReader {
    answers: Vec<u32>,
    labels: Vec<(u32, u32)>,
    holds: Vec<Vec<u32>>,
    under: Vec<Vec<(u32, u32)>>,
    weights: Vec<f64>,
    norms: Vec<f64>,
    pub(super) way: LabelWay,
}
because!(LabelReader, NearestRequests, LabelProduct, "the label of a whole request from its marks, counted from the lesson requests that were asked the question of the label: the label of each lesson request, how often each label stands in the order of first sight, for each mark the lesson requests with it, its labels and its weight, the length of each lesson request, and the way that read the held part of the lessons better. read gives the label by a way: no label with no lessons; by the product the label with the best product, of equal products the label whose words come last; by the vote the label with the most votes, of equals the label whose words come last, and with no vote the most frequent label");

impl LabelReader {
    pub(super) fn new(lessons: &[(u32, Vec<u32>)], marks: usize, way: LabelWay) -> LabelReader {
        let answers: Vec<u32> = lessons.iter().map(|(answer, _)| *answer).collect();
        let marked: Vec<Vec<u32>> = lessons.iter().map(|(_, marks)| marks.clone()).collect();
        let mut labels: Vec<(u32, u32)> = Vec::new();
        for answer in &answers {
            match labels.iter_mut().find(|(held, _)| held == answer) {
                Some((_, count)) => *count += 1,
                None => labels.push((*answer, 1)),
            }
        }
        let holds = whole_spread(&marked, marks);
        let total = lessons.len() as f64;
        let mut under: Vec<Vec<(u32, u32)>> = vec![Vec::new(); marks];
        let mut weights = vec![f64::default(); marks];
        for (mark, held) in holds.iter().enumerate() {
            if held.is_empty() {
                continue;
            }
            for number in held {
                let answer = answers[*number as usize];
                match under[mark].iter_mut().find(|(label, _)| *label == answer) {
                    Some((_, count)) => *count += 1,
                    None => under[mark].push((answer, 1)),
                }
            }
            let mut said = f64::default();
            for (label, count) in &labels {
                let with = under[mark].iter().find(|(held, _)| held == label).map_or(0, |(_, count)| *count);
                let share = (f64::from(with) + WHOLE_SURE * f64::from(*count) / total) / (held.len() as f64 + WHOLE_SURE);
                said += share * (share / (f64::from(*count) / total)).ln();
            }
            weights[mark] = if said > f64::default() { said } else { f64::default() };
        }
        let norms = whole_norms(&marked, &weights);
        LabelReader { answers, labels, holds, under, weights, norms, way }
    }

    fn product(&self, marks: &[u32], order: &dyn Fn(u32, u32) -> Ordering) -> Option<u32> {
        let total = self.answers.len() as f64;
        let mut best: Option<(f64, u32)> = None;
        for (answer, count) in &self.labels {
            let mut score = (f64::from(*count) / total).ln();
            for mark in marks {
                let Some(held) = self.holds.get(*mark as usize).filter(|held| !held.is_empty()) else { continue };
                let with = self.under[*mark as usize].iter().find(|(label, _)| label == answer).map_or(0, |(_, count)| *count);
                score += ((f64::from(with) + held.len() as f64 / total) / (f64::from(*count) + f64::from(1u8))).ln();
            }
            let better = match best {
                None => true,
                Some((top, held)) => score > top || (score == top && order(*answer, held) == Ordering::Greater),
            };
            if better {
                best = Some((score, *answer));
            }
        }
        best.map(|(_, answer)| answer)
    }

    fn vote(&self, marks: &[u32], order: &dyn Fn(u32, u32) -> Ordering) -> Option<u32> {
        let near = whole_nearest(&self.holds, &self.weights, &self.norms, marks, WHOLE_NEAREST);
        let mut votes: Vec<(u32, f64)> = Vec::new();
        for (share, number) in &near {
            let answer = self.answers[*number as usize];
            match votes.iter_mut().find(|(held, _)| *held == answer) {
                Some((_, sum)) => *sum += share * share,
                None => votes.push((answer, share * share)),
            }
        }
        let mut best: Option<(u32, f64)> = None;
        for (answer, sum) in &votes {
            let better = match best {
                None => true,
                Some((held, top)) => *sum > top || (*sum == top && order(*answer, held) == Ordering::Greater),
            };
            if better {
                best = Some((*answer, *sum));
            }
        }
        match best {
            Some((answer, _)) => Some(answer),
            None => {
                let mut most: Option<(u32, u32)> = None;
                for (answer, count) in &self.labels {
                    if most.is_none_or(|(_, top)| *count > top) {
                        most = Some((*answer, *count));
                    }
                }
                most.map(|(answer, _)| answer)
            }
        }
    }

    pub(super) fn read(&self, marks: &[u32], way: LabelWay, order: &dyn Fn(u32, u32) -> Ordering) -> Option<u32> {
        if self.answers.is_empty() {
            return None;
        }
        match way {
            LabelWay::Product => self.product(marks, order),
            LabelWay::Vote => self.vote(marks, order),
        }
    }
}
