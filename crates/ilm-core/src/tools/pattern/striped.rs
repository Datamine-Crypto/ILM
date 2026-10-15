use crate::tools::pattern::codes::Step;
use crate::tools::pattern::finder::Catalog;
use crate::tools::pattern::hash::{CheapHasher, CheapMap};
use crate::tools::pattern::table::CountedPatterns;
use patterns::{because, source};
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

pub struct StripedTable;
source!(StripedTable, "lock striping for a table that many threads fill, as Herlihy and Shavit describe it: the table is cut into parts by some bits of the hash and each part has its own lock, so two threads wait for each other only when they need the same part");

pub const STRIPES: usize = 64;
because!(STRIPES, StripedTable, "the parts of a striped table: more than the hardware threads of a large desktop machine, so that two threads seldom need one part, and a number that a few bits of a hash select");

pub const STRIPE_SHIFT: u32 = 40;
because!(STRIPE_SHIFT, StripedTable, "the bits of a hash below the bits that select the part: the table inside a part uses the low bits for the place and the highest bits for its marks, so the part is taken from bits between them");

pub fn hash_of<Q: Hash + ?Sized>(key: &Q) -> u64 {
    let mut hasher = CheapHasher::default();
    key.hash(&mut hasher);
    hasher.finish()
}
because!(hash_of, StripedTable, "the cheap hash of a key, from which a striped table takes its part");

#[derive(Clone, Debug)]
pub struct Stripe<K> {
    at: CheapMap<K, u32>,
    rows: Vec<K>,
}
because!(Stripe, StripedTable, "one part of a striped table: the number of each key, and the keys in the order of their numbers");

impl<K> Default for Stripe<K> {
    fn default() -> Stripe<K> {
        Stripe { at: CheapMap::default(), rows: Vec::new() }
    }
}

pub struct Shared<K> {
    parts: Vec<Mutex<Stripe<K>>>,
}
because!(Shared, StripedTable, "a table that gives each key a number once, filled by many threads: number gives the number of a key and takes a new key in, find gives the number of a key that is there, row gives the key of a number, freeze ends the filling and gives the same table with no locks. The number of a key holds its part, so a key is found again from its number");

pub struct Frozen<K> {
    parts: Vec<Stripe<K>>,
}
because!(Frozen, StripedTable, "a striped table after its filling: the same parts and numbers with no locks, so that many threads read it at the same time. find gives the number of a key, row the key of a number");

impl<K: Eq + Hash + Clone> Default for Shared<K> {
    fn default() -> Shared<K> {
        Shared { parts: (0..STRIPES).map(|_| Mutex::new(Stripe::default())).collect() }
    }
}

pub fn stripe_of(hash: u64) -> usize {
    (hash >> STRIPE_SHIFT) as usize % STRIPES
}
because!(stripe_of, StripedTable, "the part of a striped table for a hash");

impl<K: Eq + Hash + Clone> Shared<K> {
    pub fn number<Q>(&self, key: &Q) -> u32
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ToOwned<Owned = K> + ?Sized,
    {
        let part = stripe_of(hash_of(key));
        let mut stripe = self.parts[part].lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(found) = stripe.at.get(key) {
            return *found;
        }
        let number = (stripe.rows.len() * STRIPES + part) as u32;
        stripe.rows.push(key.to_owned());
        stripe.at.insert(key.to_owned(), number);
        number
    }

    pub fn find<Q>(&self, key: &Q) -> Option<u32>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let stripe = self.parts[stripe_of(hash_of(key))].lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        stripe.at.get(key).copied()
    }

    pub fn row(&self, number: u32) -> K {
        let stripe = self.parts[number as usize % STRIPES].lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        stripe.rows[number as usize / STRIPES].clone()
    }

    pub fn freeze(self) -> Frozen<K> {
        Frozen { parts: self.parts.into_iter().map(|part| part.into_inner().unwrap_or_else(std::sync::PoisonError::into_inner)).collect() }
    }
}

impl<K: Eq + Hash + Clone> Frozen<K> {
    pub fn seek<Q>(&self, key: &Q) -> Option<u32>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.parts[stripe_of(hash_of(key))].at.get(key).copied()
    }

    pub fn held(&self, number: u32) -> &K {
        &self.parts[number as usize % STRIPES].rows[number as usize / STRIPES]
    }
}

#[derive(Default)]
pub struct NearNumbers {
    forms: CheapMap<Vec<u32>, u32>,
    steps: CheapMap<Step, u32>,
}
because!(NearNumbers, StripedTable, CountedPatterns, "the numbers of forms and steps that one thread has seen in the shared tables of one model, kept from one count of the model to the next, so that a key is looked up in the shared tables once");

pub struct Grow<'t> {
    forms: &'t Shared<Vec<u32>>,
    steps: &'t Shared<Step>,
    near: NearNumbers,
}
because!(Grow, StripedTable, CountedPatterns, "the numbers of forms and steps for one thread while a model learns: the thread keeps the numbers that it has seen in a table of its own and goes to the shared table, which has locks, only for a key that is new to it. on starts it with the numbers that a thread kept, kept gives them back");

impl<'t> Grow<'t> {
    pub fn on(forms: &'t Shared<Vec<u32>>, steps: &'t Shared<Step>, near: NearNumbers) -> Grow<'t> {
        Grow { forms, steps, near }
    }

    pub fn kept(self) -> NearNumbers {
        self.near
    }
}

impl Catalog for Grow<'_> {
    fn form(&mut self, row: &[u32]) -> Option<u32> {
        if let Some(found) = self.near.forms.get(row) {
            return Some(*found);
        }
        let number = self.forms.number(row);
        self.near.forms.insert(row.to_vec(), number);
        Some(number)
    }

    fn step(&mut self, step: &Step) -> Option<u32> {
        if let Some(found) = self.near.steps.get(step) {
            return Some(*found);
        }
        let number = self.steps.number(step);
        self.near.steps.insert(*step, number);
        Some(number)
    }
}

pub struct Seek<'t> {
    pub forms: &'t Frozen<Vec<u32>>,
    pub steps: &'t Frozen<Step>,
}
because!(Seek, CountedPatterns, "the numbers of forms and steps while a model answers: a form or a step that no lesson had has no number, and nothing is written, so many threads answer with one model");

impl Catalog for Seek<'_> {
    fn form(&mut self, row: &[u32]) -> Option<u32> {
        self.forms.seek(row)
    }

    fn step(&mut self, step: &Step) -> Option<u32> {
        self.steps.seek(step)
    }
}
