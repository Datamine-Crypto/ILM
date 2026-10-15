use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::hash::{CheapMap, mix};
use patterns::{because, source};

pub struct SliceKeys;
source!(SliceKeys, "the fingerprints of Karp and Rabin: a text with one place open, or the words around a run, is known by a hash made in a few steps from the running hashes of the two ends of its text, and texts with equal hashes are compared word by word; so no such text is built as a list of its own");

pub struct TextEnds {
    left: Vec<u64>,
    right: Vec<u64>,
}
because!(TextEnds, SliceKeys, "the running hashes of one text: from its start up to each place, and from each place to its end. over reads a text, holed gives the hash of the text with one place open, around the hash of the words before a run and after it");

impl TextEnds {
    pub fn over(text: &[u32]) -> TextEnds {
        let mut left: Vec<u64> = Vec::with_capacity(text.len() + 1);
        let mut hash: u64 = 0;
        left.push(hash);
        for word in text {
            hash = mix(hash, u64::from(*word));
            left.push(hash);
        }
        let mut right: Vec<u64> = vec![0; text.len() + 1];
        let mut hash: u64 = 0;
        for (place, word) in text.iter().enumerate().rev() {
            hash = mix(hash, u64::from(*word));
            right[place] = hash;
        }
        TextEnds { left, right }
    }

    pub fn holed(&self, place: usize) -> u64 {
        let size = self.left.len() - 1;
        mix(mix(mix(self.left[place], self.right[place + 1]), place as u64), size as u64)
    }

    pub fn around(&self, from: usize, to: usize) -> u64 {
        let size = self.left.len() - 1;
        mix(mix(mix(self.left[from], from as u64), self.right[to]), (size - to) as u64)
    }
}

pub fn holed_groups(keys: &[Vec<u32>], words: &[Vec<u32>]) -> Vec<Vec<u32>> {
    let mut at: CheapMap<u64, u32> = CheapMap::default();
    let mut next: Vec<u32> = Vec::new();
    let mut seen: Vec<(u32, u32)> = Vec::new();
    let mut first: Vec<u32> = Vec::new();
    let mut more: Vec<Vec<u32>> = Vec::new();
    for (number, key) in keys.iter().enumerate() {
        let ends = TextEnds::over(key);
        for place in 0..key.len() {
            let hash = ends.holed(place);
            let word = words[number][place];
            let mut group = at.get(&hash).copied().unwrap_or(ABSENT);
            while group != ABSENT {
                let (held, held_place) = seen[group as usize];
                let other = &keys[held as usize];
                if held_place as usize == place && other.len() == key.len() && other[..place] == key[..place] && other[place + 1..] == key[place + 1..] {
                    break;
                }
                group = next[group as usize];
            }
            if group == ABSENT {
                let fresh = seen.len() as u32;
                next.push(at.insert(hash, fresh).unwrap_or(ABSENT));
                seen.push((number as u32, place as u32));
                first.push(word);
                more.push(Vec::new());
            } else if first[group as usize] != word && !more[group as usize].contains(&word) {
                more[group as usize].push(word);
            }
        }
    }
    let mut out: Vec<Vec<u32>> = Vec::new();
    for (word, others) in first.into_iter().zip(more) {
        if !others.is_empty() {
            let mut group: Vec<u32> = Vec::with_capacity(others.len() + 1);
            group.push(word);
            group.extend(others);
            out.push(group);
        }
    }
    out
}
because!(holed_groups, SliceKeys, "the words that fill one place of texts that are the same in every other place: for each text and place, the key is the text of keys with that place open and the word is the word of words at that place; the groups with two different words or more, each word once, in the order in which the groups were first seen");

pub fn run_contexts(texts: &[Vec<u32>], most: usize, rest: usize) -> Vec<Vec<(u32, u32, u32)>> {
    let mut at: CheapMap<u64, u32> = CheapMap::default();
    let mut next: Vec<u32> = Vec::new();
    let mut seen: Vec<(u32, u32, u32)> = Vec::new();
    let mut more: Vec<Vec<(u32, u32, u32)>> = Vec::new();
    let same = |a: (u32, u32, u32), b: (u32, u32, u32)| texts[a.0 as usize][a.1 as usize..a.2 as usize] == texts[b.0 as usize][b.1 as usize..b.2 as usize];
    for (number, text) in texts.iter().enumerate() {
        let size = text.len();
        let ends = TextEnds::over(text);
        for from in 0..size {
            for to in from + 1..=size.min(from + most) {
                if size - (to - from) < rest {
                    continue;
                }
                let hash = ends.around(from, to);
                let run = (number as u32, from as u32, to as u32);
                let mut context = at.get(&hash).copied().unwrap_or(ABSENT);
                while context != ABSENT {
                    let (held, held_from, held_to) = seen[context as usize];
                    let other = &texts[held as usize];
                    if held_from as usize == from && other.len() - held_to as usize == size - to && other[..from] == text[..from] && other[held_to as usize..] == text[to..] {
                        break;
                    }
                    context = next[context as usize];
                }
                if context == ABSENT {
                    let fresh = seen.len() as u32;
                    next.push(at.insert(hash, fresh).unwrap_or(ABSENT));
                    seen.push(run);
                    more.push(Vec::new());
                } else if !same(seen[context as usize], run) && !more[context as usize].iter().any(|held| same(*held, run)) {
                    more[context as usize].push(run);
                }
            }
        }
    }
    let mut out: Vec<Vec<(u32, u32, u32)>> = Vec::new();
    for (run, others) in seen.into_iter().zip(more) {
        if !others.is_empty() {
            let mut context: Vec<(u32, u32, u32)> = Vec::with_capacity(others.len() + 1);
            context.push(run);
            context.extend(others);
            out.push(context);
        }
    }
    out
}
because!(run_contexts, SliceKeys, "the runs of words that stand between the same words before and after them: for each run of at most most words that leaves at least rest words of its text around it, the context is the words before and after it; the contexts with two different runs or more, each run once as its text and its first and last place, in the order in which the contexts and their runs were first seen");
