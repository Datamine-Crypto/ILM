use crate::base::words::words;
use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::codes::{ASKED_SLOT, SLOT_LETTERS, SlotPatterns};
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::kinds::{IN_LESSONS, PatternWords, new_kind, set_new_kind};
use crate::tools::pattern::names::WordTexts;
use patterns::{because, source};
use std::sync::Arc;

pub struct StandIns;
source!(StandIns, "a new word that stands in the question alone is replaced by a lesson word of its kind that the story does not hold, the one whose question form the lessons hold most often, and the new word is put back in the answer");

pub const MOST_STAND_INS: usize = 30;
because!(MOST_STAND_INS, StandIns, "the lesson words of a kind that are tried in the place of a new word of the question: enough to find a word that the story does not hold, and a bound on the work for a kind with thousands of words");

#[derive(Clone, Debug, Default)]
pub struct Told {
    pub words: Vec<u32>,
    pub labels: Vec<u32>,
    pub held: Vec<u32>,
    pub fix: u32,
    pub kinds: Vec<(u32, Vec<u32>)>,
}
because!(Told, SlotPatterns, "one told sentence as it is read: its words; its words that have a kind, each under a label (the name of the kind and its number among the words of that kind), in the order of the texts of the labels; the number of the text of its fixed words; and its words by kind in their order. of_kind gives the words of one kind");

impl Told {
    pub fn of_kind(&self, kind: u32) -> &[u32] {
        self.kinds.iter().find(|(held, _)| *held == kind).map_or(&[], |(_, found)| found.as_slice())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Story {
    pub sents: Vec<Arc<Told>>,
    spots: Vec<(u32, u32, u32)>,
    rows: Vec<u32>,
    pub extra: Vec<(u32, u32)>,
}
because!(Story, SlotPatterns, "the told sentences of a case with the numbers of the sentences that hold each word, in one flat array, and the kinds of its words that no lesson held. build makes it; at gives the rising sentence numbers of a word, and nothing for a word that the story does not hold; told_words gives its different words in the order of their first place");

impl Story {
    pub fn build(sents: Vec<Arc<Told>>, extra: Vec<(u32, u32)>) -> Story {
        let mut pairs: Vec<(u32, u32)> = Vec::new();
        for (number, told) in sents.iter().enumerate() {
            pairs.extend(told.words.iter().map(|word| (*word, number as u32)));
        }
        pairs.sort_unstable();
        pairs.dedup();
        let mut spots: Vec<(u32, u32, u32)> = Vec::new();
        let mut rows: Vec<u32> = Vec::with_capacity(pairs.len());
        for (word, number) in pairs {
            rows.push(number);
            match spots.last_mut() {
                Some(spot) if spot.0 == word => spot.2 = rows.len() as u32,
                _ => spots.push((word, rows.len() as u32 - 1, rows.len() as u32)),
            }
        }
        let mut extra = extra;
        extra.sort_unstable();
        Story { sents, spots, rows, extra }
    }

    pub fn at(&self, word: u32) -> &[u32] {
        match self.spots.binary_search_by_key(&word, |spot| spot.0) {
            Ok(place) => &self.rows[self.spots[place].1 as usize..self.spots[place].2 as usize],
            Err(_) => &[],
        }
    }

    pub fn told_words(&self) -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        for word in self.sents.iter().flat_map(|told| told.words.iter()) {
            if !out.contains(word) {
                out.push(*word);
            }
        }
        out
    }
}

#[derive(Clone, Debug, Default)]
pub struct Query {
    pub story: Arc<Story>,
    pub q: Vec<u32>,
    pub slots: Vec<u32>,
    pub form: Vec<u32>,
    pub hits: Vec<u32>,
    pub back: Vec<(u32, u32)>,
    pub local: Vec<(u32, u32)>,
}
because!(Query, SlotPatterns, "one question on a story: its words; the words of it that stand in the story and have a kind, which are its slots in their order; the question form, a slot in the place of each such word; the sentences that hold a slot; and for a case with new words the lesson words that stand in for new words of the question, and the new forms that are read as their other form. on makes it for a story and the words of a question");

impl Query {
    pub fn on(story: Arc<Story>, q: Vec<u32>, words: &PatternWords) -> Query {
        let mut slots: Vec<u32> = Vec::new();
        for word in &q {
            let fits = !story.at(*word).is_empty() && words.kind_with(*word, &story.extra) != ABSENT;
            if fits && !slots.contains(word) && slots.len() < SLOT_LETTERS.len() {
                slots.push(*word);
            }
        }
        let form: Vec<u32> = q.iter().map(|word| slots.iter().position(|slot| slot == word).map_or(*word, |place| ASKED_SLOT | place as u32)).collect();
        let mut hits: Vec<u32> = slots.iter().flat_map(|word| story.at(*word).iter().copied()).collect();
        hits.sort_unstable();
        hits.dedup();
        Query { story, q, slots, form, hits, back: Vec::new(), local: Vec::new() }
    }
}

pub struct Desk<'a, N: WordTexts> {
    pub words: &'a PatternWords,
    pub names: &'a mut N,
    seen: CheapMap<String, Arc<Told>>,
    labels: CheapMap<(u32, u32), u32>,
}
because!(Desk, SlotPatterns, "the reader of texts into cases for one model: the words and kinds of the lessons, the table of texts, the sentences that were read before, and the numbers of the labels. told reads the words of a sentence into its parts; sentence reads a told text, once for each different text; case reads a story and a question, with the kinds, the names and the stand-in words of words that no lesson held");

impl<'a, N: WordTexts> Desk<'a, N> {
    pub fn over(words: &'a PatternWords, names: &'a mut N) -> Desk<'a, N> {
        Desk { words, names, seen: CheapMap::default(), labels: CheapMap::default() }
    }

    pub fn label(&mut self, kind: u32, count: u32) -> u32 {
        if let Some(found) = self.labels.get(&(kind, count)) {
            return *found;
        }
        let text = format!("{}{}", self.words.kind_names[kind as usize], count);
        let number = self.names.number(&text);
        self.labels.insert((kind, count), number);
        number
    }

    pub fn told(&mut self, sent: &[u32], extra: &[(u32, u32)]) -> Told {
        let mut pairs: Vec<(u32, u32)> = Vec::new();
        let mut fixed: Vec<u32> = Vec::new();
        let mut kinds: Vec<(u32, Vec<u32>)> = Vec::new();
        for word in sent {
            let kind = self.words.kind_with(*word, extra);
            if kind == ABSENT {
                fixed.push(*word);
                continue;
            }
            let place = kinds.iter().position(|(held, _)| *held == kind).unwrap_or(kinds.len());
            if place == kinds.len() {
                kinds.push((kind, Vec::new()));
            }
            let label = self.label(kind, kinds[place].1.len() as u32);
            kinds[place].1.push(*word);
            pairs.push((label, *word));
        }
        let names = &*self.names;
        pairs.sort_by(|a, b| names.name(a.0).cmp(names.name(b.0)).then_with(|| names.name(a.1).cmp(names.name(b.1))));
        let fix_text = fixed.iter().map(|word| names.name(*word)).collect::<Vec<&str>>().join(" ");
        let fix = self.names.number(&fix_text);
        Told { words: sent.to_vec(), labels: pairs.iter().map(|pair| pair.0).collect(), held: pairs.iter().map(|pair| pair.1).collect(), fix, kinds }
    }

    pub fn sentence(&mut self, text: &str) -> Arc<Told> {
        if let Some(found) = self.seen.get(text) {
            return found.clone();
        }
        let row = self.words.read(self.names, &words(text));
        let told = Arc::new(self.told(&row, &[]));
        self.seen.insert(text.to_string(), told.clone());
        told
    }

    pub fn local_forms(&mut self, texts: &[Vec<u32>]) -> Vec<(u32, u32)> {
        let mut here: Vec<u32> = Vec::new();
        let mut seen: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for word in texts.iter().flatten() {
            if seen.insert(*word) {
                here.push(*word);
            }
        }
        let mut local: Vec<(u32, u32)> = Vec::new();
        for word in &here {
            let text = self.names.name(*word).to_string();
            for ending in ["s", "es"] {
                let base = text.strip_suffix(ending).and_then(|stem| self.names.find(stem)).filter(|base| seen.contains(base));
                let Some(base) = base else { continue };
                let new_word = !self.words.has(*word, IN_LESSONS);
                if new_word || !self.words.has(base, IN_LESSONS) {
                    let (from, to) = if new_word { (*word, base) } else { (base, *word) };
                    set_new_kind(&mut local, from, to, true);
                    break;
                }
            }
        }
        local
    }

    pub fn case(&mut self, sentences: &[String], question: &str, readings: &dyn Fn(&[u32]) -> u32) -> Query {
        let told: Vec<Arc<Told>> = sentences.iter().map(|text| self.sentence(text)).collect();
        let q = self.words.read(self.names, &words(question));
        let words_of = self.words;
        let known = |word: &u32| words_of.has(*word, IN_LESSONS);
        if q.iter().all(known) && told.iter().all(|sent| sent.words.iter().all(known)) {
            return Query::on(Arc::new(Story::build(told, Vec::new())), q, words_of);
        }
        let mut texts: Vec<Vec<u32>> = told.iter().map(|sent| sent.words.clone()).collect();
        texts.push(q);
        let local = self.local_forms(&texts);
        for word in texts.iter_mut().flatten() {
            if let Some((_, base)) = local.iter().find(|(from, _)| from == word) {
                *word = *base;
            }
        }
        let mut texts = words_of.new_names(self.names, texts);
        let extra = words_of.new_kinds(&texts, &[]);
        let mut q = texts.pop().unwrap_or_default();
        let sents: Vec<Arc<Told>> = texts.iter().map(|sent| Arc::new(self.told(sent, &extra))).collect();
        let story = Arc::new(Story::build(sents, extra.clone()));
        let mut back: Vec<(u32, u32)> = Vec::new();
        for word in q.clone() {
            let Some(kind) = new_kind(&extra, word) else { continue };
            if !story.at(word).is_empty() || back.iter().any(|(_, new)| *new == word) {
                continue;
            }
            let mut best: Option<(u32, u32)> = None;
            for stand in words_of.members[kind as usize].iter().take(MOST_STAND_INS) {
                if story.at(*stand).is_empty() && !q.contains(stand) && !back.iter().any(|(held, _)| held == stand) {
                    let tried: Vec<u32> = q.iter().map(|held| if *held == word { *stand } else { *held }).collect();
                    let count = readings(&Query::on(story.clone(), tried, words_of).form);
                    if best.is_none_or(|(top, _)| count > top) {
                        best = Some((count, *stand));
                    }
                }
            }
            if let Some((_, stand)) = best {
                back.push((stand, word));
                for held in q.iter_mut().filter(|held| **held == word) {
                    *held = stand;
                }
            }
        }
        let mut case = Query::on(story, q, words_of);
        case.back = back;
        case.local = local;
        case
    }
}
