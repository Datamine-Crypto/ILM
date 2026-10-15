use crate::tools::parts::counted::{FORCED_LESSONS, sure_of};
use crate::tools::parts::grammar::{Grammar, Item, Meter, ROW_KEY, FormShape, ShapeKind, State};
use crate::tools::parts::language::{Program, WordRow, Word};
use crate::tools::parts::ordered::Kept;
use crate::tools::parts::search::{SEARCH_CHANGES_MOST, SEARCH_LESSONS_MOST, SEARCH_WORK_EACH, SEARCH_WORK_FIRST, SEARCH_WORK_MOST};
use crate::tools::pattern::hash::CheapMap;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct WhatIsLeft;
source!(WhatIsLeft, "the class of a text as what is left of it when no form fits any more, in the way of nearest neighbour classification with leave one out: a lesson counts for a proposed state only when its answer is the most frequent answer of the other lessons of its class, so a state that gives every lesson a class of its own gains nothing");

const CLASS_WORDS: usize = 30;
because!(CLASS_WORDS, WhatIsLeft, "the words that a round tries as fixed words: those that stand in the most lessons, since a word of few lessons tells the class of few");

const CLASS_PAIRS: usize = 12;
because!(CLASS_PAIRS, WhatIsLeft, "of those words, the ones that a round tries as pairs of two different words that say nothing; pairs grow with the square of the words, so fewer words are paired");

const CLASS_SAMPLE: usize = 150;
because!(CLASS_SAMPLE, WhatIsLeft, "the lessons of all lengths on which a round compares its proposals first: enough to rank them, few enough to try every proposal");

const CLASS_TRIED: usize = 8;
because!(CLASS_TRIED, WhatIsLeft, "the best proposals of the sample that are then counted on all lessons");

pub type Class = Option<Vec<Item>>;
because!(Class, WhatIsLeft, "what is left of a text when no form fits any more, or nothing when the text cannot be read");

pub fn alone_right(keys: &[Class], answers: &[WordRow]) -> usize {
    let mut by_key: CheapMap<&Class, CheapMap<&WordRow, usize>> = CheapMap::default();
    for (key, answer) in keys.iter().zip(answers) {
        *by_key.entry(key).or_default().entry(answer).or_default() += 1;
    }
    let mut single: CheapMap<&WordRow, usize> = CheapMap::default();
    for found in by_key.values() {
        if found.values().sum::<usize>() == 1 {
            for (answer, count) in found {
                *single.entry(answer).or_default() += count;
            }
        }
    }
    let mut right = 0;
    for found in by_key.values() {
        let pool = if found.values().sum::<usize>() == 1 { &single } else { found };
        for (answer, count) in found {
            let others = pool.iter().filter(|(other, _)| *other != answer).map(|(_, held)| *held).max().unwrap_or_default();
            if pool.get(answer).copied().unwrap_or_default() > others + 1 {
                right += count;
            }
        }
    }
    right
}
because!(alone_right, WhatIsLeft, "the lessons whose answer is the most frequent answer of the other lessons with the same class; a lesson with a class of its own is counted against the other such lessons");

pub struct ClassTable {
    pub table: CheapMap<Class, WordRow>,
    pub default: WordRow,
}
because!(ClassTable, WhatIsLeft, "the answer of each class that has an answer of its own, and the answer of every other text");

pub fn class_table(grammar: &Grammar, lessons: &[(Vec<Word>, WordRow)], meter: &Meter) -> ClassTable {
    let mut by_key: Kept<Class, Kept<WordRow, usize>> = Kept::default();
    for (words, answer) in lessons {
        by_key.or_put(grammar.left(words, meter), Kept::default).count(answer.clone(), 1);
    }
    let mut table: CheapMap<Class, WordRow> = CheapMap::default();
    let mut rest: Kept<WordRow, usize> = Kept::default();
    for (key, found) in by_key.pairs() {
        let Some((answer, count)) = found.most() else { continue };
        if found.total() >= FORCED_LESSONS && sure_of(count, found.total()) {
            table.insert(key.clone(), answer.clone());
        } else {
            for (answer, count) in found.pairs() {
                rest.count(answer.clone(), *count);
            }
        }
    }
    let mut every: Kept<WordRow, usize> = Kept::default();
    for (_, answer) in lessons {
        every.count(answer.clone(), 1);
    }
    let pool = if rest.is_empty() { &every } else { &rest };
    let default = pool.most().map(|(answer, _)| answer.clone()).unwrap_or_default();
    ClassTable { table, default }
}
because!(class_table, WhatIsLeft, "the table of the classes: a class has an answer of its own when enough lessons have it and nearly all of them give one answer; every other text gets the most frequent answer of the lessons with no such class, the first of equals in the order of the lessons");

pub fn class_search(lessons: &[(Vec<Word>, WordRow)], start: Option<&Grammar>, meter: &Meter, rank: &dyn Fn(Word) -> u32) -> Grammar {
    let mut g = match start {
        None => {
            let mut grammar = Grammar::default();
            grammar.settle();
            grammar
        }
        Some(start) => start.with_state(ROW_KEY, &start.state(ROW_KEY)),
    };
    let mut unique: Kept<(Vec<Word>, WordRow), ()> = Kept::default();
    for lesson in lessons {
        unique.put(lesson.clone(), ());
    }
    let mut seen: Vec<(Vec<Word>, WordRow)> = unique.into_pairs().map(|(lesson, _)| lesson).collect();
    seen.sort_by_key(|(words, _)| words.len());
    seen.truncate(SEARCH_LESSONS_MOST);
    let answers: Vec<WordRow> = seen.iter().map(|(_, answer)| answer.clone()).collect();
    let mut by_word: HashMap<Word, Vec<usize>> = HashMap::new();
    let mut in_text: HashMap<Word, usize> = HashMap::new();
    let mut times: HashMap<Word, HashSet<usize>> = HashMap::new();
    for (number, (words, _)) in seen.iter().enumerate() {
        let mut counts: Kept<Word, usize> = Kept::default();
        for word in words {
            counts.count(*word, 1);
        }
        for (word, count) in counts.pairs() {
            by_word.entry(*word).or_default().push(number);
            *in_text.entry(*word).or_default() += 1;
            times.entry(*word).or_default().insert(*count);
        }
    }
    let mut vocab: Vec<(Word, usize)> = in_text.iter().map(|(word, count)| (*word, *count)).collect();
    vocab.sort_by_key(|(word, count)| (std::cmp::Reverse(*count), rank(*word)));
    let vocab: Vec<Word> = vocab.into_iter().filter(|(word, count)| *count >= FORCED_LESSONS && (*count < seen.len() || times[word].len() > 1)).map(|(word, _)| word).collect();
    let mut keys: Vec<Class> = seen.iter().map(|(words, _)| g.left(words, meter)).collect();
    let base = alone_right(&keys, &answers);
    let mut score = base;
    let began = meter.words.get();
    let step = (seen.len() / CLASS_SAMPLE).max(1);
    let sample: Vec<usize> = (0..seen.len()).step_by(step).take(CLASS_SAMPLE).collect();
    let over = |score: usize| meter.words.get() - began >= SEARCH_WORK_MOST.min(SEARCH_WORK_FIRST + SEARCH_WORK_EACH * (score - base) as u64);
    let sample_answers: Vec<WordRow> = sample.iter().map(|lesson| answers[*lesson].clone()).collect();
    for _ in 0..SEARCH_CHANGES_MOST {
        let free: Vec<Word> = vocab.iter().copied().filter(|word| !g.forms.holds(word) && !g.marks.contains(word)).take(CLASS_WORDS).collect();
        let mut proposals: Vec<(Word, State, Vec<Word>)> = free.iter().map(|word| (*word, State::Mark, vec![*word])).collect();
        proposals.extend(free.iter().map(|word| (*word, State::form(FormShape::two(ShapeKind::Pair, *word), Some(Program::nothing()), 0, None), vec![*word])));
        for word in free.iter().take(CLASS_PAIRS) {
            for other in vocab.iter().take(CLASS_PAIRS) {
                if other != word && !g.forms.holds(other) {
                    proposals.push((*word, State::form(FormShape::two(ShapeKind::Pair, *other), Some(Program::nothing()), 0, None), vec![*word, *other]));
                }
            }
        }
        let now = alone_right(&sample.iter().map(|lesson| keys[*lesson].clone()).collect::<Vec<Class>>(), &sample_answers);
        let mut first: Vec<(usize, usize, usize, Grammar, HashSet<usize>)> = Vec::new();
        for (word, state, things) in &proposals {
            if over(score) {
                break;
            }
            let other = g.with_state(*word, state);
            let touched: HashSet<usize> = things.iter().filter_map(|thing| by_word.get(thing)).flatten().copied().collect();
            let probe: Vec<Class> = sample.iter().map(|lesson| if touched.contains(lesson) { other.left(&seen[*lesson].0, meter) } else { keys[*lesson].clone() }).collect();
            let found = alone_right(&probe, &sample_answers);
            if found > now {
                let order = first.len();
                first.push((found - now, touched.len(), order, other, touched));
            }
        }
        first.sort_by_key(|(gain, with, order, _, _)| (std::cmp::Reverse(*gain), *with, *order));
        let mut best: Option<((usize, std::cmp::Reverse<usize>, std::cmp::Reverse<usize>), Grammar, Vec<Class>)> = None;
        for (_, with, _, other, touched) in first.into_iter().take(CLASS_TRIED) {
            let next: Vec<Class> = (0..seen.len()).map(|lesson| if touched.contains(&lesson) { other.left(&seen[lesson].0, meter) } else { keys[lesson].clone() }).collect();
            let right = alone_right(&next, &answers);
            let classes: HashSet<&Class> = next.iter().collect();
            let rank_of = (right, std::cmp::Reverse(classes.len()), std::cmp::Reverse(with));
            if right > score && best.as_ref().is_none_or(|(top, _, _)| rank_of > *top) {
                best = Some((rank_of, other, next));
            }
        }
        let Some((rank_of, other, next)) = best else { break };
        score = rank_of.0;
        g = other;
        keys = next;
        if over(score) {
            break;
        }
    }
    g.right = score;
    g.seen = seen.len();
    g.stopped = over(score);
    g.base = base;
    g
}
because!(class_search, WhatIsLeft, "the fixed words, and the pairs of fixed words that say nothing, with which what is left of a text tells its answer: a round proposes each of the words of the most lessons as a fixed word and as a pair with itself, and pairs of the first of them; each proposal is counted first on a sample of the lessons of all lengths, then the best on all lessons; the proposal with the highest count is taken, of equals the one with fewer classes, then the one whose words stand in fewer lessons, until no proposal gains or the words read pass the limit");
