use crate::cases::Case;
use crate::base::cheap::CheapMap;
use patterns::{because, source};

pub struct HeldParts;
source!(HeldParts, "the cut of the cases of a kind into a learned part, a read part and a sealed part: whole stories go to one part, so a story that comes again with another question is never learned and read; the stories are taken in the order of their first case; the sealed part is read by one final run only");

const ONE_STORY: f64 = 0.2;
because!(ONE_STORY, HeldParts, "when one told text holds more than this share of all cases, the told text is the same for the kind and is no story: every case is then a story of its own, or no part could be read");

const LEARN_PARTS: usize = 6;
because!(LEARN_PARTS, HeldParts, "of ten parts of the cases of a kind at most this many are learned from, so that cases are left to read");

const OF_PARTS: usize = 10;
because!(OF_PARTS, HeldParts, "the parts that the share of the learned cases is counted in");

pub const SEALED_AFTER: usize = 1000;
because!(SEALED_AFTER, HeldParts, "the sealed part lies before this many last cases of the stories that are not learned: a run of development may read up to that many cases from the end, and the sealed part must lie outside every such run");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parts {
    pub learn: Vec<usize>,
    pub read: Vec<usize>,
    pub sealed: Vec<usize>,
}
because!(Parts, HeldParts, "the places of the cases of each part in the order of the file");

pub fn told_texts(cases: &[Case]) -> (Vec<usize>, Vec<usize>) {
    let mut seen: CheapMap<&[String], usize> = CheapMap::default();
    let mut sizes: Vec<usize> = Vec::new();
    let mut keys: Vec<usize> = Vec::with_capacity(cases.len());
    for case in cases {
        if case.sentences.is_empty() {
            keys.push(sizes.len());
            sizes.push(1);
            continue;
        }
        let fresh = sizes.len();
        let key = *seen.entry(case.sentences.as_slice()).or_insert(fresh);
        if key == fresh {
            sizes.push(0);
        }
        sizes[key] += 1;
        keys.push(key);
    }
    (keys, sizes)
}
because!(told_texts, HeldParts, "the told text of each case as a number in the order of first sight, and the cases of each number: cases with the same told sentences have one number, a case with no told sentence has a number of its own");

fn story_keys(cases: &[Case]) -> Vec<usize> {
    let (keys, sizes) = told_texts(cases);
    let largest = sizes.iter().copied().max().unwrap_or(0);
    if (largest as f64) > ONE_STORY * (cases.len() as f64) {
        return (0..cases.len()).collect();
    }
    keys
}
because!(story_keys, HeldParts, "the story of each case as a number: its told text, or the case alone when one told text holds too large a share of the kind");

fn last_of(mut places: Vec<usize>, held: usize) -> Vec<usize> {
    let from = places.len().saturating_sub(held);
    places.drain(..from);
    places
}
because!(last_of, HeldParts, "the last cases of a part, as many as are held");

pub fn parts(cases: &[Case], fit: usize, held: usize, start: Option<usize>, stories: bool) -> Parts {
    let keys = story_keys(cases);
    let stories_seen = keys.iter().copied().max().map_or(0, |key| key + 1);
    let first = start.unwrap_or(cases.len());
    let mut size = vec![0usize; stories_seen];
    for key in &keys[..first.min(keys.len())] {
        size[*key] += 1;
    }
    let most = match start {
        None => fit.min(cases.len() * LEARN_PARTS / OF_PARTS),
        Some(_) => fit,
    };
    let mut mine = vec![false; stories_seen];
    let mut count = 0usize;
    for (place, key) in keys.iter().enumerate() {
        if place >= first || count >= most {
            break;
        }
        if !mine[*key] {
            mine[*key] = true;
            count += if stories { 1 } else { size[*key] };
        }
    }
    let learn: Vec<usize> = (0..cases.len()).filter(|place| mine[keys[*place]] && *place < first).collect();
    let pool: Vec<usize> = (0..cases.len()).filter(|place| !mine[keys[*place]] && start.is_none_or(|from| *place >= from)).collect();
    let mut in_pool = vec![0usize; stories_seen];
    let mut later: Vec<usize> = Vec::new();
    for place in &pool {
        if in_pool[keys[*place]] == 0 {
            later.push(keys[*place]);
        }
        in_pool[keys[*place]] += 1;
    }
    let mut read_key = vec![false; stories_seen];
    let mut sealed_key = vec![false; stories_seen];
    let skip = held.max(SEALED_AFTER);
    let mut count = 0usize;
    for key in later.iter().rev() {
        if count < held {
            read_key[*key] = true;
        } else if skip <= count && count < skip + held {
            sealed_key[*key] = true;
        } else if count >= skip + held {
            break;
        }
        count += in_pool[*key];
    }
    let read: Vec<usize> = pool.iter().copied().filter(|place| read_key[keys[*place]]).collect();
    let sealed: Vec<usize> = pool.iter().copied().filter(|place| sealed_key[keys[*place]]).collect();
    Parts { learn, read: last_of(read, held), sealed: last_of(sealed, held) }
}
because!(parts, HeldParts, "the three parts of a kind: stories are learned in the order of their first case until the learned part holds the asked number of cases (or of stories), the read part is the last held cases of the other stories, and the sealed part lies before the last cases that development may have read; with a start, the learned part comes from the cases before it and the other parts from the cases after it");
