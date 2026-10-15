use crate::tools::parts::language::{WordRow, Word};
use crate::tools::pattern::hash::CheapSet;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct CountedWords;
source!(CountedWords, "what a single word says, counted from texts with their answers in the way of the word translation tables of Brown and others, with a sure test in place of expectation maximization: a text word says an answer word when the answer word stands in nearly no answer without the text word in the text; the other direction is not asked, because a language can leave a part out");

pub const SURE_LESSONS: usize = 20;
because!(SURE_LESSONS, CountedWords, "the lessons that a count needs before it is taken as sure: with fewer, a share of nearly all can come by chance");

pub const COUNT_SURE_SHARE: f64 = 0.98;
because!(COUNT_SURE_SHARE, CountedWords, "the share of the lessons that must agree for a count to be sure: nearly all, with room for a lesson in fifty that is wrong in the set");

pub const FORCED_LESSONS: usize = 2;
because!(FORCED_LESSONS, CountedWords, "the least lessons that must show a thing where the lessons cannot tell it apart from another one: a single lesson shows nothing that chance does not show");

pub struct Counted {
    pub table: HashMap<Word, WordRow>,
    pub out: CheapSet<Word>,
    pub copies: bool,
}
because!(Counted, CountedWords, "the counted table of some lessons: what a word says, the words that stand in nearly every lesson and are left out, and whether the lessons say that words say themselves");

pub fn sure_of(count: usize, all: usize) -> bool {
    (count as f64) >= COUNT_SURE_SHARE * (all as f64)
}
because!(sure_of, CountedWords, "whether a count is the sure share of all or more; the product is made in numbers with a point, the same way wherever a share is asked, so the same count gives the same answer everywhere");

pub fn counted_table(lessons: &[(Vec<Word>, WordRow)], rank: &dyn Fn(Word) -> u32) -> Counted {
    let mut in_text: HashMap<Word, usize> = HashMap::new();
    let mut in_answer: HashMap<Word, usize> = HashMap::new();
    let mut both: HashMap<(Word, Word), (usize, usize)> = HashMap::new();
    for (words, target) in lessons {
        let told: HashSet<Word> = words.iter().copied().collect();
        let mut said: Vec<(Word, usize)> = Vec::new();
        for (place, word) in target.iter().enumerate() {
            if !said.iter().any(|(seen, _)| seen == word) {
                said.push((*word, place));
            }
        }
        for word in &told {
            *in_text.entry(*word).or_default() += 1;
            for (answer, place) in &said {
                let pair = both.entry((*word, *answer)).or_default();
                pair.0 += 1;
                pair.1 += place;
            }
        }
        for (answer, _) in &said {
            *in_answer.entry(*answer).or_default() += 1;
        }
    }
    let count_of = |word: Word, answer: Word| both.get(&(word, answer)).map_or(0, |pair| pair.0);
    let out: CheapSet<Word> = in_text.iter().filter(|(_, count)| lessons.len() >= SURE_LESSONS && sure_of(**count, lessons.len())).map(|(word, _)| *word).collect();
    let mut source: HashMap<Word, Vec<(f64, Word)>> = HashMap::new();
    for ((word, answer), (count, _)) in &both {
        if !out.contains(word) && in_text[word] >= SURE_LESSONS && in_answer[answer] >= SURE_LESSONS && sure_of(*count, in_answer[answer]) {
            source.entry(*answer).or_default().push(((*count as f64) / (in_text[word] as f64), *word));
        }
    }
    let mut says: HashMap<Word, Vec<(f64, Word)>> = HashMap::new();
    for (answer, found) in &source {
        let mut top: Option<f64> = None;
        for (share, _) in found {
            if top.is_none_or(|best| *share > best) {
                top = Some(*share);
            }
        }
        let best: Vec<Word> = found.iter().filter(|(share, _)| Some(*share) == top).map(|(_, word)| *word).collect();
        if let [word] = best.as_slice() {
            let (count, first) = both[&(*word, *answer)];
            says.entry(*word).or_default().push(((first as f64) / (count as f64), *answer));
        }
    }
    let mut table: HashMap<Word, WordRow> = HashMap::new();
    for (word, mut found) in says {
        found.sort_by(|one, other| one.0.total_cmp(&other.0).then(rank(one.1).cmp(&rank(other.1))));
        table.insert(word, found.into_iter().map(|(_, answer)| answer).collect());
    }
    let several: Vec<Word> = table.iter().filter(|(_, row)| row.len() > 1).map(|(word, _)| *word).collect();
    if !several.is_empty() {
        let mut some: HashMap<Word, usize> = HashMap::new();
        for (words, target) in lessons {
            let told: HashSet<Word> = words.iter().copied().collect();
            for word in several.iter().filter(|word| told.contains(word)) {
                if table[word].iter().any(|answer| target.contains(answer)) {
                    *some.entry(*word).or_default() += 1;
                }
            }
        }
        for word in &several {
            let with_one = some.get(word).copied().unwrap_or_default();
            if table[word].iter().any(|answer| (count_of(*word, *answer) as f64) < COUNT_SURE_SHARE * (with_one as f64)) {
                table.remove(word);
            }
        }
    }
    let in_both: usize = in_answer.keys().map(|answer| count_of(*answer, *answer)).sum();
    let in_all: usize = in_answer.values().sum();
    let copies = in_all >= SURE_LESSONS && sure_of(in_both, in_all);
    if copies {
        for word in in_text.keys() {
            let itself = count_of(*word, *word);
            if !table.contains_key(word) && !out.contains(word) && itself > 0 && sure_of(itself, in_answer.get(word).copied().unwrap_or_default()) {
                table.insert(*word, vec![*word]);
            }
        }
    }
    let mut kept: HashMap<Word, usize> = HashMap::new();
    let mut shown: HashMap<Word, usize> = HashMap::new();
    let mut plain = 0;
    for (words, target) in lessons {
        if target.is_empty() || !words.iter().filter(|word| target.contains(word)).eq(target.iter()) {
            continue;
        }
        plain += 1;
        let told: HashSet<Word> = words.iter().copied().collect();
        for word in told {
            *shown.entry(word).or_default() += 1;
            *kept.entry(word).or_default() += usize::from(target.contains(&word));
        }
    }
    if plain >= SURE_LESSONS {
        for (word, count) in &kept {
            if !table.contains_key(word) && !out.contains(word) && *count >= FORCED_LESSONS && sure_of(*count, shown[word]) {
                table.insert(*word, vec![*word]);
            }
        }
    }
    Counted { table, out, copies }
}
because!(counted_table, CountedWords, "the counted table of some lessons. A word that stands in the sure share of all lessons is left out. A text word says an answer word when the answer word stands in the sure share of its answers with the text word in the text, with enough lessons of each; of several such text words the one whose lessons hold the answer word most often, and none when two are equal. Several answer words of one text word are put in the order of their middle places in the answers and must stand together. Words that say themselves are counted over all answer words together, and from the lessons whose answer is their text with some words left out");
