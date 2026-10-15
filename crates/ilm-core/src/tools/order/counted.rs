use crate::base::keys::{KeyNumbers, fields_of, packed};
use patterns::{because, source};

pub struct PairOrder;
source!(PairOrder, "an order of things counted from pairs, read as the order of a tournament: x is before y when x came first more often than y; with equal counts, when a chain of such wins leads from x to y and none from y to x; else when x won against a larger share of the things that it met. Counts add, so the order does not depend on the order in which the pairs came");

const ORDER_MOST_THINGS: usize = 3000;
because!(ORDER_MOST_THINGS, PairOrder, "with more things in an order its chains are not followed: the chains of all things cost the square of the things in time and in memory");

const ORDER_MIDDLE: f64 = 0.5;
because!(ORDER_MIDDLE, PairOrder, "the share of wins of a thing that won and lost nothing: it stands in the middle");

const ORDER_WORD_BITS: usize = 64;
because!(ORDER_WORD_BITS, PairOrder, "the things of the chains of one thing are kept as bits in words of this width");

#[derive(Clone, Debug, Default)]
pub(super) struct OrderCount {
    pairs: KeyNumbers,
    counts: Vec<u32>,
    things: KeyNumbers,
    names: Vec<u32>,
    shares: Vec<f64>,
    reach: Vec<Vec<u64>>,
}
because!(OrderCount, PairOrder, "the counted pairs of things and what is read from them: for each pair how often the first came first; for each thing its share of wins and, when the things are few enough, every thing that a chain of wins leads to. add counts a pair; settle reads the wins, the shares and the chains once all pairs are counted; before says whether one thing is before another, and nothing when the counts do not say; ranks gives the place of each thing in the order, a thing earlier when it is before more of the others, of equals the smaller thing first");

impl OrderCount {
    pub(super) fn add(&mut self, first: u32, second: u32) {
        if first == second {
            return;
        }
        let number = self.pairs.number(packed(&[first, second])) as usize;
        if number == self.counts.len() {
            self.counts.push(0);
        }
        self.counts[number] += 1;
    }

    fn count(&self, first: u32, second: u32) -> u32 {
        self.pairs.find(packed(&[first, second])).map_or(0, |number| self.counts[number as usize])
    }

    fn place(&self, thing: u32) -> Option<usize> {
        self.things.find(packed(&[thing])).map(|place| place as usize)
    }

    pub(super) fn settle(&mut self) {
        for key in self.pairs.keys() {
            let [first, second, ..] = fields_of(*key);
            for thing in [first, second] {
                let fresh = self.names.len() as u32;
                if self.things.number(packed(&[thing])) == fresh {
                    self.names.push(thing);
                }
            }
        }
        let size = self.names.len();
        let mut wins: Vec<Vec<usize>> = vec![Vec::new(); size];
        let mut lost = vec![0usize; size];
        for (number, key) in self.pairs.keys().iter().enumerate() {
            let [first, second, ..] = fields_of(*key);
            if self.counts[number] > self.count(second, first) {
                let (Some(winner), Some(loser)) = (self.place(first), self.place(second)) else { continue };
                wins[winner].push(loser);
                lost[loser] += 1;
            }
        }
        self.shares = (0..size).map(|thing| if wins[thing].len() + lost[thing] > 0 { (wins[thing].len() as f64) / ((wins[thing].len() + lost[thing]) as f64) } else { ORDER_MIDDLE }).collect();
        if size > ORDER_MOST_THINGS {
            return;
        }
        let words = size.div_ceil(ORDER_WORD_BITS);
        let mut stack: Vec<usize> = Vec::new();
        for thing in 0..size {
            let mut seen = vec![0u64; words];
            stack.clear();
            stack.extend_from_slice(&wins[thing]);
            while let Some(next) = stack.pop() {
                let (word, bit) = (next / ORDER_WORD_BITS, next % ORDER_WORD_BITS);
                if seen[word] >> bit & 1 == 0 {
                    seen[word] |= 1 << bit;
                    stack.extend_from_slice(&wins[next]);
                }
            }
            self.reach.push(seen);
        }
    }

    fn leads(&self, from: Option<usize>, to: Option<usize>) -> bool {
        match (from, to) {
            (Some(from), Some(to)) => self.reach.get(from).is_some_and(|seen| seen[to / ORDER_WORD_BITS] >> (to % ORDER_WORD_BITS) & 1 == 1),
            _ => false,
        }
    }

    pub(super) fn before(&self, first: u32, second: u32) -> Option<bool> {
        let (ahead, behind) = (self.count(first, second), self.count(second, first));
        if ahead != behind {
            return Some(ahead > behind);
        }
        let (one, other) = (self.place(first), self.place(second));
        let (forth, back) = (self.leads(one, other), self.leads(other, one));
        if forth != back {
            return Some(forth);
        }
        let (one, other) = (self.shares[one?], self.shares[other?]);
        (one != other).then_some(one > other)
    }

    pub(super) fn ranks(&self) -> OrderRanks {
        let size = self.names.len();
        let ahead: Vec<usize> = (0..size).map(|one| (0..size).filter(|other| *other != one && self.before(self.names[one], self.names[*other]) == Some(true)).count()).collect();
        let mut order: Vec<usize> = (0..size).collect();
        order.sort_by(|left, right| ahead[*right].cmp(&ahead[*left]).then_with(|| self.names[*left].cmp(&self.names[*right])));
        let mut ranks = OrderRanks { places: self.things.clone(), ranks: vec![0; size] };
        for (rank, thing) in order.iter().enumerate() {
            ranks.ranks[*thing] = rank;
        }
        ranks
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct OrderRanks {
    places: KeyNumbers,
    ranks: Vec<usize>,
}
because!(OrderRanks, PairOrder, "the place of each counted thing in the order. doubled gives twice the place of a thing, and for a thing that was never counted the number of the things, which is twice the middle place, so that a key of places compares as whole numbers");

impl OrderRanks {
    pub(super) fn doubled(&self, thing: u32) -> usize {
        match self.places.find(packed(&[thing])) {
            Some(place) => self.ranks[place as usize] * ORDER_DOUBLE,
            None => self.ranks.len(),
        }
    }
}

const ORDER_DOUBLE: usize = 2;
because!(ORDER_DOUBLE, PairOrder, "the factor of a place in a key of places: a thing that was never counted stands in the middle of the order, which can fall between places; with this factor the middle is a whole number too, and keys compare as whole numbers");
