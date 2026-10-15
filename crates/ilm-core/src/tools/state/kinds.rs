use crate::base::symbols::Symbols;
use crate::tools::pattern::classes::{Classes, sorted_texts, word_classes};
use crate::tools::pattern::hash::{CheapMap, CheapSet};
use crate::tools::pattern::holes::run_contexts;
use crate::base::first_sight::FirstSightTable;
use crate::tools::pattern::names::{WordTexts};
use crate::tools::state::split::{STATE_PAIR_SIZE, join_ranges, token_of};
use patterns::{because, source};
use std::collections::HashSet;

pub struct UnitKinds;
source!(UnitKinds, "the kinds of words found again on the units of a story, the sentences and their parts, by the distributional method of Harris; a run of words that stands where one word stands is one word when it holds no slot, so that a name of several words is one thing of the state");

pub const STATE_RUN: usize = 6;
because!(STATE_RUN, UnitKinds, "the longest run of words that is one word, and the longest span of an answer: the names of things in the stories of states are longer than in other stories, as a room or a thing with a colour");

pub const RUN_REST: usize = 2;
because!(RUN_REST, UnitKinds, "a run is looked for only where the text holds this many other words, so that a short unit of a list still gives its runs");

pub const LEAST_GLUED: u32 = 3;
because!(LEAST_GLUED, UnitKinds, "the places that a run with a word of the kind of the single word must stand in before it is a run, and the units that the rarer of the two forms of a glued unit needs: a few, so that chance does not make a run or a cut");

const SLOT_ROUNDS: usize = 3;
because!(SLOT_ROUNDS, UnitKinds, "the rounds in which runs that hold a slot are dropped: each round drops the runs that the runs of the round before covered");

const ALONE_ROUNDS: usize = 2;
because!(ALONE_ROUNDS, UnitKinds, "the rounds in which a run is dropped because one of its words stands alone in an answer: the second round sees the answers with the runs of the first one gone");

pub type FoundRuns = (CheapSet<Vec<u32>>, Vec<(u32, Vec<u32>)>, Classes);
because!(FoundRuns, UnitKinds, "the runs of words that are one word, the pairs of a single word and a run that stand in one place, and the classes of single words");

pub fn find_runs(symbols: &Symbols, texts: &[Vec<u32>], most: usize, rest: usize) -> FoundRuns {
    let open_words = word_classes(symbols, texts, &[]);
    let slice = |(number, from, to): (u32, u32, u32)| &texts[number as usize][from as usize..to as usize];
    let mut runs: CheapSet<Vec<u32>> = CheapSet::default();
    let mut pairs: Vec<(u32, Vec<u32>)> = Vec::new();
    let mut places: FirstSightTable<Vec<u32>, u32> = FirstSightTable::new();
    for found in run_contexts(texts, most, rest) {
        let singles: Vec<u32> = found.iter().map(|held| slice(*held)).filter(|one| one.len() == 1).map(|one| one[0]).collect();
        if singles.is_empty() {
            continue;
        }
        for run in found.iter().map(|held| slice(*held)) {
            if run.len() <= 1 {
                continue;
            }
            let with: Vec<u32> = singles.iter().copied().filter(|one| !run.contains(one)).collect();
            if with.is_empty() {
                continue;
            }
            pairs.extend(with.iter().map(|one| (*one, run.to_vec())));
            let other_kind = with.iter().any(|one| {
                let kind = open_words.class(*one);
                kind.is_none() || run.iter().all(|word| open_words.class(*word) != kind)
            });
            if other_kind {
                runs.insert(run.to_vec());
            } else {
                places.add(run.to_vec(), 1);
            }
        }
    }
    runs.extend(places.rows().iter().filter(|(_, count)| *count >= LEAST_GLUED).map(|(run, _)| run.clone()));
    for _ in 0..SLOT_ROUNDS {
        let mut free: CheapSet<u32> = CheapSet::default();
        for text in texts {
            let size = text.len();
            let mut covered = vec![false; size];
            for from in 0..size {
                for length in STATE_PAIR_SIZE..=most.min(size - from) {
                    if runs.contains(&text[from..from + length]) {
                        covered[from..from + length].iter_mut().for_each(|place| *place = true);
                    }
                }
            }
            free.extend(text.iter().zip(&covered).filter(|(_, covered)| !**covered).map(|(word, _)| *word));
        }
        let mut bad: CheapSet<Vec<u32>> = CheapSet::default();
        for run in &runs {
            for other in &runs {
                if other == run {
                    continue;
                }
                let shorter = run.len().min(other.len());
                let mut head = 0;
                while head < shorter && run[head] == other[head] {
                    head += 1;
                }
                let mut tail = 0;
                while tail < shorter - head && run[run.len() - 1 - tail] == other[other.len() - 1 - tail] {
                    tail += 1;
                }
                if head + tail > 0 && other.len() > head + tail && run[head..run.len() - tail].iter().any(|word| open_words.known(*word) && free.contains(word)) {
                    bad.insert(run.clone());
                    break;
                }
            }
        }
        if bad.is_empty() {
            break;
        }
        runs.retain(|run| !bad.contains(run));
    }
    pairs.retain(|(_, run)| runs.contains(run));
    (runs, pairs, open_words)
}
because!(find_runs, UnitKinds, "runs of words that stand where one word stands, with the pairs of a word and a run in one place and the classes of single words; a run with a word of the kind of the single word only when it stands so in a few places; a run that holds a slot is dropped: another run has its words at the start or at the end and other words between, and a word between is of a kind and stands outside of runs too");

pub struct KindTables {
    pub runs: CheapSet<Vec<u32>>,
    pub kind: CheapMap<u32, u32>,
    pub kind_names: Vec<String>,
    pub vocab: CheapSet<u32>,
    pub shapes: CheapMap<usize, FirstSightTable<Vec<u32>, u32>>,
}
because!(KindTables, UnitKinds, "the runs and kinds of the units of the lessons: the runs of words, the kind of each token that has one, the names of the kinds in their order, the tokens of the lessons, and the forms of the lesson units by length with their counts");

pub fn unit_tokens_with<N: WordTexts>(names: &mut N, runs: &CheapSet<Vec<u32>>, more: &HashSet<Vec<u32>>, norm: &[u32]) -> Vec<u32> {
    join_ranges(norm, &|run: &[u32]| runs.contains(run) || more.contains(run), STATE_RUN).iter().map(|(from, to)| token_of(names, &norm[*from..*to])).collect()
}
because!(unit_tokens_with, UnitKinds, "the tokens of a unit in normal form: each known run of words as one token, the longest run first; the runs of the lessons and the runs of new words of one case");

pub fn learn_kinds(symbols: &mut Symbols, read: &[(CheapSet<Vec<u32>>, Vec<u32>, Vec<u32>)]) -> KindTables {
    let mut all: CheapSet<Vec<u32>> = CheapSet::default();
    for (story, q, _) in read {
        all.extend(story.iter().cloned());
        all.insert(q.clone());
    }
    let texts = sorted_texts(symbols, &all);
    let (mut runs, pairs, open_words) = find_runs(symbols, &texts, STATE_RUN, RUN_REST);
    for _ in 0..ALONE_ROUNDS {
        let mut bad: CheapSet<Vec<u32>> = CheapSet::default();
        let mut held: CheapMap<Vec<u32>, Vec<Vec<u32>>> = CheapMap::default();
        for (story, _, answer) in read {
            let mut here: CheapSet<Vec<u32>> = CheapSet::default();
            for sentence in story {
                let found = held.entry(sentence.clone()).or_insert_with(|| join_ranges(sentence, &|run: &[u32]| runs.contains(run), STATE_RUN).iter().filter(|(from, to)| to - from > 1).map(|(from, to)| sentence[*from..*to].to_vec()).collect());
                here.extend(found.iter().cloned());
            }
            if here.is_empty() {
                continue;
            }
            let size = answer.len();
            if size > 1 && story.iter().any(|sentence| sentence.windows(size).any(|window| window == answer.as_slice())) {
                continue;
            }
            let alone: CheapSet<u32> = join_ranges(answer, &|run: &[u32]| runs.contains(run), STATE_RUN).iter().filter(|(from, to)| to - from == 1).map(|(from, _)| answer[*from]).collect();
            bad.extend(here.into_iter().filter(|run| run.iter().any(|word| alone.contains(word) && open_words.known(*word))));
        }
        runs.retain(|run| !bad.contains(run));
    }
    let none: HashSet<Vec<u32>> = HashSet::new();
    let joined: CheapSet<Vec<u32>> = texts.iter().map(|text| unit_tokens_with(symbols, &runs, &none, text)).collect();
    let joined_texts = sorted_texts(symbols, &joined);
    let unions: Vec<(u32, u32)> = pairs
        .iter()
        .filter(|(_, run)| runs.contains(run))
        .map(|(one, run)| {
            let token = token_of(symbols, run);
            (*one, token)
        })
        .collect();
    let classes = word_classes(symbols, &joined_texts, &unions);
    let mut often: CheapMap<u32, u32> = CheapMap::default();
    for word in joined_texts.iter().flatten() {
        *often.entry(*word).or_insert(0) += 1;
    }
    let mut members = classes.members();
    members.sort_by(|a, b| symbols.name(*a).cmp(symbols.name(*b)));
    let mut name: CheapMap<u32, u32> = CheapMap::default();
    for word in &members {
        let Some(root) = classes.class(*word) else { continue };
        let count = often.get(word).copied().unwrap_or(0);
        if name.get(&root).is_none_or(|best| count > often.get(best).copied().unwrap_or(0)) {
            name.insert(root, *word);
        }
    }
    let mut kind_names: Vec<String> = name.values().map(|word| symbols.name(*word).to_uppercase()).collect();
    kind_names.sort();
    kind_names.dedup();
    let mut kind: CheapMap<u32, u32> = CheapMap::default();
    for word in &members {
        if let Some(root) = classes.class(*word) {
            let text = symbols.name(name[&root]).to_uppercase();
            kind.insert(*word, kind_names.binary_search(&text).unwrap_or(0) as u32);
        }
    }
    let mut vocab: CheapSet<u32> = CheapSet::default();
    let mut shapes: CheapMap<usize, FirstSightTable<Vec<u32>, u32>> = CheapMap::default();
    for text in &joined_texts {
        vocab.extend(text.iter().copied());
        let shape: Vec<u32> = text.iter().map(|token| kind.get(token).map_or(*token, |found| crate::tools::pattern::classes::KIND_MARK | found)).collect();
        shapes.entry(text.len()).or_default().add(shape, 1);
    }
    KindTables { runs, kind, kind_names, vocab, shapes }
}
because!(learn_kinds, UnitKinds, "the runs and kinds from the units of the lessons: the runs, less those with a word that stands alone in an answer of a story that holds the run (unless the answer stands in the story as one span), then the kinds of the tokens with the pairs of a word and a run joined, each kind named for its most frequent token, and the forms of the units");
