use crate::base::exact::Amount;
use crate::base::lanes::{on_lanes, shards};
use crate::tools::amount::calcs::{AMOUNT_PAIR, AMOUNT_TRIPLE};
use crate::tools::amount::learn::AmountLesson;
use crate::tools::amount::model::AMOUNT_SURE_LESSONS;
use crate::tools::amount::shared::{AmountBase, AmountHand};
use crate::tools::amount::tokens::AmountCut;
use patterns::{because, source};

pub struct AmountSteps;
source!(AmountSteps, "calculation in steps, as a person works out a written sum: two numbers with one token between them become one number, the token says which calculation and how early its step comes, and a number alone between two brackets becomes that number. Which way of steps a kind of task means is found by trying every way on the lessons");

pub(super) const AMOUNT_STEP_TOKENS: usize = 3;
because!(AMOUNT_STEP_TOKENS, AmountSteps, "the search for steps takes the most frequent tokens between two numbers, up to this many: the ways to try grow by a large factor with each token");

pub(super) const AMOUNT_FIRST_LESSONS: usize = 12;
because!(AMOUNT_FIRST_LESSONS, AmountSteps, "a way of steps is dropped when it is wrong for one of this many first lessons: enough to drop nearly every wrong way before the count over all lessons");

pub(super) const AMOUNT_BATCHES: usize = 3;
because!(AMOUNT_BATCHES, AmountSteps, "the groups of first lessons that are tried one after the other: when a fault of a set among the first lessons drops every way, the next group is taken");

pub(super) const AMOUNT_MOST_LENGTH: usize = 80;
because!(AMOUNT_MOST_LENGTH, AmountSteps, "a longer case is not calculated in steps: a sum that a set writes is short, and the cost of steps grows fast with the length");

pub(super) const AMOUNT_KEPT_WAYS: usize = 200;
because!(AMOUNT_KEPT_WAYS, AmountSteps, "of the ways that fit the first lessons, this many are counted on all lessons: the ways that fit are few when the lessons say enough, and the count of each costs a pass over the lessons");

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum AmountItem<V> {
    Word(u32),
    Value(V),
}
because!(AmountItem, AmountSteps, "one item of a written sum while it is worked out: a token, or a value that a number or an earlier step gave");

pub(super) fn amount_unwrap<V: Copy>(items: &mut Vec<AmountItem<V>>, brackets: &[(u32, u32)]) -> bool {
    let found = items.windows(AMOUNT_TRIPLE).position(|trio| matches!(trio, [AmountItem::Word(open), AmountItem::Value(_), AmountItem::Word(close)] if brackets.contains(&(*open, *close))));
    let Some(place) = found else { return false };
    items.remove(place);
    items.remove(place + 1);
    true
}
because!(amount_unwrap, AmountSteps, "a value alone between an opening bracket and its closing bracket becomes that value, at the first such place; says if it was done");

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct AmountWay {
    pub(super) count: usize,
    pub(super) tokens: [u32; AMOUNT_STEP_TOKENS],
    pub(super) choices: [u32; AMOUNT_STEP_TOKENS],
    pub(super) levels: [u32; AMOUNT_STEP_TOKENS],
}
because!(AmountWay, AmountSteps, "one way of steps: for each of a few tokens the calculation or the table that it stands for, and its level, where the step of the highest level comes first. of gives what the way says for a token");

impl AmountWay {
    pub(super) fn of(&self, token: u32) -> Option<(usize, u32)> {
        (0..self.count).find(|place| self.tokens[*place] == token).map(|place| (self.choices[place] as usize, self.levels[place]))
    }
}

pub(super) fn amount_orders(count: usize) -> Vec<[u32; AMOUNT_STEP_TOKENS]> {
    let mut orders = Vec::new();
    for index in 0..count.pow(count as u32) {
        let mut levels = [0u32; AMOUNT_STEP_TOKENS];
        let mut rest = index;
        for place in (0..count).rev() {
            levels[place] = (rest % count) as u32;
            rest /= count;
        }
        let top = levels[..count].iter().copied().max().unwrap_or(0);
        if (0..=top).all(|level| levels[..count].contains(&level)) {
            orders.push(levels);
        }
    }
    orders
}
because!(amount_orders, AmountSteps, "every way to give each token a level so that no level below the highest is left out, in a fixed order, because of ways that are right for equally many lessons the first one is taken");

pub(super) fn amount_way(tokens: &[u32], orders: &[[u32; AMOUNT_STEP_TOKENS]], choices: usize, index: usize) -> AmountWay {
    let mut way = AmountWay { count: tokens.len(), levels: orders[index % orders.len()], ..AmountWay::default() };
    let mut rest = index / orders.len();
    for place in (0..tokens.len()).rev() {
        way.tokens[place] = tokens[place];
        way.choices[place] = (rest % choices) as u32;
        rest /= choices;
    }
    way
}
because!(amount_way, AmountSteps, "the way of steps with a given number: the choice of the first token changes last, and for each choice of all tokens the orders of levels follow one another; with a number for each way the ways are tried side by side on the threads and the first of equal ways is still the same");

pub(super) fn amount_tables() -> Vec<Vec<bool>> {
    let both = [false, true];
    let mut tables: Vec<Vec<bool>> = Vec::new();
    for first in both {
        for second in both {
            for third in both {
                for fourth in both {
                    tables.push(vec![first, second, third, fourth]);
                }
            }
        }
    }
    for first in both {
        for second in both {
            tables.push(vec![first, second]);
        }
    }
    tables
}
because!(amount_tables, AmountSteps, "every table that a token can stand for in steps over the answer words of a kind: the tables of a token between two values, then the tables of a token before one value, in a fixed order");

impl AmountBase {
    pub(super) fn steps_value(&self, cut: &AmountCut, way: &AmountWay, items: &mut Vec<AmountItem<Amount>>) -> Option<Amount> {
        if cut.words.len() > AMOUNT_MOST_LENGTH {
            return None;
        }
        items.clear();
        let mut numbers = cut.values.iter();
        for word in &cut.words {
            items.push(if *word == self.marks.number { AmountItem::Value((*numbers.next()?)?) } else { AmountItem::Word(*word) });
        }
        loop {
            if amount_unwrap(items, &self.marks.brackets) {
                continue;
            }
            let mut first: Option<(u32, usize, usize)> = None;
            for (place, trio) in items.windows(AMOUNT_TRIPLE).enumerate() {
                if let [AmountItem::Value(_), AmountItem::Word(token), AmountItem::Value(_)] = trio
                    && let Some((calc, level)) = way.of(*token)
                    && first.is_none_or(|(top, _, _)| level > top)
                {
                    first = Some((level, place, calc));
                }
            }
            let Some((_, place, calc)) = first else { break };
            let [AmountItem::Value(left), _, AmountItem::Value(right)] = items[place..place + AMOUNT_TRIPLE] else { return None };
            items[place] = AmountItem::Value(self.calcs[calc].run(&[Some(left), Some(right)])?);
            items.drain(place + 1..place + AMOUNT_TRIPLE);
        }
        let mut left = items.iter().filter_map(|item| if let AmountItem::Value(value) = item { Some(*value) } else { None });
        left.next().filter(|_| left.next().is_none())
    }

    pub(super) fn word_value(&self, words: &[u32], both: &[u32], way: &AmountWay, items: &mut Vec<AmountItem<bool>>) -> Option<bool> {
        if words.len() > AMOUNT_MOST_LENGTH {
            return None;
        }
        items.clear();
        items.extend(words.iter().map(|word| both.iter().position(|held| held == word).map_or(AmountItem::Word(*word), |place| AmountItem::Value(place == 1))));
        loop {
            if amount_unwrap(items, &self.marks.brackets) {
                continue;
            }
            let mut first: Option<(u32, usize, usize)> = None;
            for place in 0..items.len().saturating_sub(1) {
                if let (AmountItem::Word(token), AmountItem::Value(_)) = (items[place], items[place + 1])
                    && let Some((table, level)) = way.of(token)
                    && (self.tables[table].len() == AMOUNT_PAIR || (place > 0 && matches!(items[place - 1], AmountItem::Value(_))))
                    && first.is_none_or(|(top, _, _)| level > top)
                {
                    first = Some((level, place, table));
                }
            }
            let Some((_, place, table)) = first else { break };
            let table = &self.tables[table];
            let AmountItem::Value(right) = items[place + 1] else { return None };
            if table.len() == AMOUNT_PAIR {
                items[place] = AmountItem::Value(table[usize::from(right)]);
                items.remove(place + 1);
            } else {
                let AmountItem::Value(left) = items[place - 1] else { return None };
                items[place - 1] = AmountItem::Value(table[(usize::from(left) << 1) | usize::from(right)]);
                items.drain(place..=place + 1);
            }
        }
        let mut left = items.iter().filter_map(|item| if let AmountItem::Value(value) = item { Some(*value) } else { None });
        left.next().filter(|_| left.next().is_none())
    }

    pub(super) fn frequent(&self, counts: &[u32]) -> Vec<u32> {
        let mut marks: Vec<u32> = (0..counts.len() as u32).filter(|token| counts[*token as usize] >= AMOUNT_SURE_LESSONS).collect();
        marks.sort_by(|left, right| counts[*right as usize].cmp(&counts[*left as usize]).then_with(|| self.symbols.name(*left).cmp(self.symbols.name(*right))));
        marks.truncate(AMOUNT_STEP_TOKENS);
        marks
    }

    pub(super) fn best_way(&self, tokens: &[u32], choices: usize, first: &[&AmountLesson], may_be_empty: bool, lanes: usize, fits: &(dyn Fn(&AmountWay, &AmountLesson, &mut AmountHand) -> bool + Sync)) -> Option<AmountWay> {
        let orders = amount_orders(tokens.len());
        let ranges = shards(choices.pow(tokens.len() as u32) * orders.len(), lanes);
        let way_at = |index: usize| amount_way(tokens, &orders, choices, index);
        let mut kept: Vec<usize> = Vec::new();
        for batch in 0..AMOUNT_BATCHES {
            let start = (batch * AMOUNT_FIRST_LESSONS).min(first.len());
            let tried = &first[start..(start + AMOUNT_FIRST_LESSONS).min(first.len())];
            if tried.is_empty() && !may_be_empty {
                break;
            }
            let found = on_lanes(ranges.len(), lanes, &|piece| {
                let mut hand = AmountHand::default();
                ranges[piece].clone().filter(|index| tried.iter().all(|lesson| fits(&way_at(*index), lesson, &mut hand))).collect::<Vec<usize>>()
            });
            kept = found.concat();
            if !kept.is_empty() {
                break;
            }
        }
        kept.truncate(AMOUNT_KEPT_WAYS);
        let right = on_lanes(kept.len(), lanes, &|piece| {
            let mut hand = AmountHand::default();
            first.iter().filter(|lesson| fits(&way_at(kept[piece]), lesson, &mut hand)).count()
        });
        let mut best: Option<usize> = None;
        for (place, count) in right.iter().enumerate() {
            if best.is_none_or(|held| *count > right[held]) {
                best = Some(place);
            }
        }
        best.map(|place| way_at(kept[place]))
    }

    pub(super) fn find_steps(&self, lessons: &[AmountLesson], lanes: usize) -> Option<AmountWay> {
        let first: Vec<&AmountLesson> = lessons.iter().filter(|lesson| lesson.counted && !lesson.late && lesson.cut.words.len() <= AMOUNT_MOST_LENGTH).collect();
        let mut between = vec![0u32; self.symbols.len()];
        for lesson in &first {
            for trio in lesson.cut.words.windows(AMOUNT_TRIPLE) {
                if let [left, token, right] = trio
                    && *left == self.marks.number
                    && *right == self.marks.number
                    && ![self.marks.number, self.marks.end, self.marks.ask].contains(token)
                {
                    between[*token as usize] += 1;
                }
            }
        }
        let tokens = self.frequent(&between);
        if tokens.is_empty() || first.len() < AMOUNT_SURE_LESSONS as usize {
            return None;
        }
        self.best_way(&tokens, self.pairs, &first, true, lanes, &|way, lesson, hand| {
            let got = self.steps_value(&lesson.cut, way, &mut hand.items);
            got.is_some() && got == lesson.truth
        })
    }

    pub(super) fn find_word_steps(&self, lessons: &[AmountLesson], both: &[u32], lanes: usize) -> Option<AmountWay> {
        let first: Vec<&AmountLesson> = lessons.iter().filter(|lesson| lesson.word.is_some() && !lesson.late && lesson.cut.words.len() <= AMOUNT_MOST_LENGTH).collect();
        let mut before = vec![0u32; self.symbols.len()];
        for lesson in &first {
            for pair in lesson.cut.words.windows(AMOUNT_PAIR) {
                if let [token, value] = pair
                    && !both.contains(token)
                    && both.contains(value)
                    && ![self.marks.end, self.marks.ask].contains(token)
                    && !self.marks.is_bracket(*token)
                {
                    before[*token as usize] += 1;
                }
            }
        }
        let tokens = self.frequent(&before);
        if tokens.is_empty() || first.len() < AMOUNT_SURE_LESSONS as usize {
            return None;
        }
        self.best_way(&tokens, self.tables.len(), &first, false, lanes, &|way, lesson, hand| {
            let got = self.word_value(&lesson.cut.words, both, way, &mut hand.word_items);
            got.is_some() && got.map(|value| both[usize::from(value)]) == lesson.word
        })
    }
}
