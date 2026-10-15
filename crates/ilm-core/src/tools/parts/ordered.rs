use patterns::{because, source};
use std::borrow::Borrow;
use crate::tools::pattern::hash::CheapMap;
use std::collections::HashMap;
use std::hash::Hash;

pub struct KeptOrder;
source!(KeptOrder, "a table that keeps its entries in the order of their first entry, beside an index by key: a walk over the table is the walk in which the entries came, so a choice among equals that takes the first one is the same in every run; an entry that is taken out and put in again stands at the end, and an entry that gets a new value keeps its place");

#[derive(Clone, Debug)]
pub struct Kept<K, V> {
    slots: Vec<Option<(K, V)>>,
    index: CheapMap<K, usize>,
}
because!(Kept, KeptOrder, "the entries in the order of their first entry, with the place of each key under the cheap hash, because the index is only looked up and never walked. get, holds, put, take and or_put work by key; keys, values and pairs walk the entries in their order; first_most gives the first entry with the highest measure, so that of equals the earlier one is taken; a place that an entry left stays empty until more places are empty than full, then the table is packed again");

impl<K: Clone + Eq + Hash, V> Default for Kept<K, V> {
    fn default() -> Kept<K, V> {
        Kept { slots: Vec::new(), index: CheapMap::default() }
    }
}

impl<K: Clone + Eq + Hash, V> Kept<K, V> {
    pub fn len(&self) -> usize {
        HashMap::len(&self.index)
    }

    pub fn is_empty(&self) -> bool {
        HashMap::is_empty(&self.index)
    }

    pub fn holds(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.index.get(key).and_then(|place| self.slots[*place].as_ref()).map(|(_, value)| value)
    }

    pub fn find<Q: Hash + Eq + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        self.index.get(key).and_then(|place| self.slots[*place].as_ref()).map(|(_, value)| value)
    }

    pub fn retain(&mut self, keep: impl Fn(&K, &V) -> bool) {
        let gone: Vec<K> = self.pairs().filter(|(key, value)| !keep(key, value)).map(|(key, _)| key.clone()).collect();
        for key in gone {
            self.index.remove(&key);
        }
        let full: Vec<Option<(K, V)>> = std::mem::take(&mut self.slots).into_iter().filter(|slot| slot.as_ref().is_some_and(|(key, _)| self.index.contains_key(key))).collect();
        self.slots = full;
        for (place, slot) in self.slots.iter().enumerate() {
            if let Some((key, _)) = slot {
                self.index.insert(key.clone(), place);
            }
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.index.get(key) {
            Some(place) => self.slots[*place].as_mut().map(|(_, value)| value),
            None => None,
        }
    }

    pub fn put(&mut self, key: K, value: V) -> bool {
        if let Some(place) = self.index.get(&key) {
            self.slots[*place] = Some((key, value));
            return false;
        }
        self.index.insert(key.clone(), self.slots.len());
        self.slots.push(Some((key, value)));
        true
    }

    pub fn or_put(&mut self, key: K, fresh: impl FnOnce() -> V) -> &mut V {
        let place = match self.index.get(&key) {
            Some(place) => *place,
            None => {
                let place = self.slots.len();
                self.index.insert(key.clone(), place);
                self.slots.push(Some((key, fresh())));
                place
            }
        };
        match self.slots[place].as_mut() {
            Some((_, value)) => value,
            None => unreachable!(),
        }
    }

    pub fn take(&mut self, key: &K) -> Option<V> {
        let place = self.index.remove(key)?;
        let taken = self.slots[place].take().map(|(_, value)| value);
        if self.slots.len() - self.index.len() > self.index.len() {
            self.pack();
        }
        taken
    }

    fn pack(&mut self) {
        let full: Vec<Option<(K, V)>> = std::mem::take(&mut self.slots).into_iter().filter(Option::is_some).collect();
        self.slots = full;
        for (place, slot) in self.slots.iter().enumerate() {
            if let Some((key, _)) = slot {
                self.index.insert(key.clone(), place);
            }
        }
    }

    pub fn pairs(&self) -> impl Iterator<Item = (&K, &V)> {
        self.slots.iter().filter_map(|slot| slot.as_ref().map(|(key, value)| (key, value)))
    }

    pub fn pairs_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.slots.iter_mut().filter_map(|slot| slot.as_mut().map(|(key, value)| (&*key, value)))
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.pairs().map(|(key, _)| key)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.slots.iter().flatten().map(|pair| &pair.1)
    }

    pub fn first_most<M: PartialOrd>(&self, measure: impl Fn(&K, &V) -> M) -> Option<(&K, &V)> {
        let mut best: Option<(M, (&K, &V))> = None;
        for (key, value) in self.pairs() {
            let size = measure(key, value);
            if best.as_ref().is_none_or(|(top, _)| size > *top) {
                best = Some((size, (key, value)));
            }
        }
        best.map(|(_, pair)| pair)
    }

    pub fn into_pairs(self) -> impl Iterator<Item = (K, V)> {
        self.slots.into_iter().flatten()
    }
}

impl<K: Clone + Eq + Hash> Kept<K, usize> {
    pub fn count(&mut self, key: K, more: usize) {
        *self.or_put(key, usize::default) += more;
    }

    pub fn of(&self, key: &K) -> usize {
        self.get(key).copied().unwrap_or_default()
    }

    pub fn total(&self) -> usize {
        self.values().sum()
    }

    pub fn most(&self) -> Option<(&K, usize)> {
        self.first_most(|_, count| *count).map(|(key, count)| (key, *count))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Few<K, V> {
    pub items: Vec<(K, V)>,
}
because!(Few, KeptOrder, "a small table in the order of first entry with no index: a cell of a chart holds a few kinds and a few readings, and a walk over a short row is quicker than a lookup in a hashed table");

impl<K, V> Default for Few<K, V> {
    fn default() -> Few<K, V> {
        Few { items: Vec::new() }
    }
}

impl<K: PartialEq, V> Few<K, V> {
    pub fn of(key: K, value: V) -> Few<K, V> {
        Few { items: vec![(key, value)] }
    }

    pub fn len(&self) -> usize {
        self.items.iter().count()
    }

    pub fn is_empty(&self) -> bool {
        self.items.first().is_none()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.items.iter().find(|(held, _)| held == key).map(|(_, value)| value)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.items.iter_mut().find(|(held, _)| held == key).map(|(_, value)| value)
    }

    pub fn holds(&self, key: &K) -> bool {
        self.items.iter().any(|(held, _)| held == key)
    }

    pub fn put(&mut self, key: K, value: V) {
        match self.get_mut(&key) {
            Some(held) => *held = value,
            None => self.items.push((key, value)),
        }
    }

    pub fn or_put(&mut self, key: K, fresh: impl FnOnce() -> V) -> &mut V {
        let place = match self.items.iter().position(|(held, _)| *held == key) {
            Some(place) => place,
            None => {
                self.items.push((key, fresh()));
                self.items.len() - 1
            }
        };
        &mut self.items[place].1
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.items.iter().map(|(key, value)| (key, value))
    }
}
