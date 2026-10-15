use crate::base::lanes::{on_lanes, shards};
use crate::base::symbols::Symbols;
use crate::base::words::answer_words;
use crate::choice::inner_parts;
use crate::cases::Case;
use crate::tools::call::{Learned, Learner, Lessons, Said};
use crate::tools::frequent::frequent_answer;
use crate::tools::order::counted::{OrderCount, OrderRanks};
use crate::tools::order::runs::{OrderPlace, OrderTwoRuns, order_items, order_run_at, order_run_place, order_two_runs};
use patterns::{because, source};
use std::cmp::Ordering;

pub struct OrderWays;
source!(OrderWays, "two ways to answer with an order that no case tells and that is the same in all cases. Put in order: the answer is a run of items of the text in another order, sorted by keys of symbols whose order is counted from the answers, from the left end of an item or from its right end. Choose: the text holds two runs of the same items in two orders, and the run whose pairs agree more with the order that was counted from the right runs of the lessons is taken. A way is a rule when it gives the whole true answer for nearly all of enough lessons of the part that is held back, with its order counted on the first part alone; with no rule the way with the best share there is used, when that share is above the share of the most frequent answer");

const ORDER_SURE_LESSONS: usize = 20;
because!(ORDER_SURE_LESSONS, OrderWays, "the least lessons of the part that is held back on which a way must be tried to be a rule: fewer readings do not show that an outcome is certain");

const ORDER_SURE_SHARE: f64 = 0.98;
because!(ORDER_SURE_SHARE, OrderWays, "the least share of the lessons that a way must answer right to be a rule, and the least share of all lessons whose answer must stand at one place or be one of two runs for a way to be tried: a fault of a set may break a rare lesson, and not more");

const ORDER_END: char = '\n';
because!(ORDER_END, OrderWays, "the symbol after the last symbol of an item in its key, so a shorter item comes before a longer one that begins with it when the counts say so; no item holds it, because a blank parts items");

const ORDER_BETWEEN: &str = " ";
because!(ORDER_BETWEEN, OrderWays, "the mark between the items of an answer that a way writes");

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum OrderWay {
    Choose,
    FromLeft,
    FromRight,
}
because!(OrderWay, OrderWays, "the ways of the tool; the order of the list is the order of their names, by which ways with equal shares are tried");

const ORDER_WAY_NAMES: &[(OrderWay, &str)] = &[(OrderWay::Choose, "choose"), (OrderWay::FromLeft, "from the left"), (OrderWay::FromRight, "from the right")];
because!(ORDER_WAY_NAMES, OrderWays, "the name of each way in the note of an answer");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OrderHow {
    Run,
    ItemBefore,
}
because!(OrderHow, OrderWays, "how the lessons of a choice write their answer: as the run that is taken, or as the item that stands before it");

struct OrderLesson {
    lines: Vec<Vec<u32>>,
    said: Vec<u32>,
    late: bool,
}
because!(OrderLesson, OrderWays, "one lesson as items: the items of each told sentence and of the question, the items of its answer, and whether it lies in the part that is held back");

#[derive(Clone, Copy, Debug)]
struct OrderFound {
    way: OrderWay,
    share: f64,
    tried: usize,
    right: usize,
}
because!(OrderFound, OrderWays, "how a way did on the part that is held back: its share of right answers, the lessons that it was tried on and those it answered right");

#[derive(Debug)]
pub(super) struct OrderModel {
    symbols: Symbols,
    keys: Vec<Vec<char>>,
    default: Vec<String>,
    place: Option<OrderPlace>,
    sides: [Option<OrderRanks>; ORDER_SIDES],
    choice: Option<(OrderHow, OrderCount)>,
    rules: Vec<OrderFound>,
    best: Vec<OrderFound>,
}
because!(OrderModel, OrderWays, "what the tool learned, as plain tables: the place of the runs to put in order with the order of the symbols from each side, the order of the items of a choice with how its answer is written, the ways that are rules and the other ways that are better than the most frequent answer, each in the order of its share on the part that is held back, and the most frequent answer");

const ORDER_SIDES: usize = 2;
because!(ORDER_SIDES, OrderWays, "the ends of an item from which its key is read: the left end and the right end");

const ORDER_PAIR: usize = 2;
because!(ORDER_PAIR, OrderWays, "the items of an answer that are compared at a time, an item and the next one; an answer with fewer items has no order to count");

fn order_key(text: &str, way: OrderWay) -> Vec<char> {
    let mut key: Vec<char> = text.chars().collect();
    if way == OrderWay::FromRight {
        key.reverse();
    }
    key.push(ORDER_END);
    key
}
because!(order_key, OrderWays, "the key of an item: its symbols from its left end or from its right end, and the end symbol");

fn order_named(runs: &OrderTwoRuns, said: &[u32]) -> Option<(usize, OrderHow)> {
    if let Some(which) = runs.runs.iter().position(|run| run.as_slice() == said) {
        return Some((which, OrderHow::Run));
    }
    let [first, second] = runs.before;
    match said {
        [item] if first != second => runs.before.iter().position(|before| *before == Some(*item)).map(|which| (which, OrderHow::ItemBefore)),
        _ => None,
    }
}
because!(order_named, OrderWays, "which of the two runs an answer names, and how: as the run itself, or as the one item that stands before it when the two runs have different items before them; nothing when it names neither");

fn order_chosen(runs: &OrderTwoRuns, order: &OrderCount) -> usize {
    let marks: Vec<i64> = runs
        .runs
        .iter()
        .map(|run| {
            let mut mark = 0i64;
            for (place, first) in run.iter().enumerate() {
                for second in &run[place + 1..] {
                    mark += order.before(*first, *second).map_or(0, |ahead| if ahead { 1 } else { -1 });
                }
            }
            mark
        })
        .collect();
    usize::from(marks[1] > marks[0])
}
because!(order_chosen, OrderWays, "the run whose pairs agree more with the counted order: each pair of a run that the order puts first counts for it, each pair that the order puts the other way counts against it; of equal runs the first");

impl OrderModel {
    fn name(&self, item: u32, strangers: &Symbols) -> String {
        match (item as usize).checked_sub(self.symbols.len()) {
            Some(place) => strangers.name(place as u32).to_string(),
            None => self.symbols.name(item).to_string(),
        }
    }

    fn key(&self, item: u32, way: OrderWay, strangers: &Symbols) -> Vec<char> {
        match self.keys.get(item as usize) {
            Some(key) if way == OrderWay::FromLeft => key.clone(),
            _ => order_key(&self.name(item, strangers), way),
        }
    }

    fn sorted_run(&self, lines: &[Vec<u32>], way: OrderWay, ranks: &OrderRanks, strangers: &Symbols) -> Option<Vec<u32>> {
        let run = order_run_at(lines, self.place?)?;
        let mut keyed: Vec<(Vec<usize>, u32)> = run.iter().map(|item| (self.key(*item, way, strangers).iter().map(|symbol| ranks.doubled(u32::from(*symbol))).collect(), *item)).collect();
        keyed.sort_by(|left, right| left.0.cmp(&right.0));
        Some(keyed.into_iter().map(|(_, item)| item).collect())
    }

    fn symbol_order(lessons: &[OrderLesson], keys: &[Vec<char>], way: OrderWay, late: Option<bool>) -> OrderCount {
        let mut order = OrderCount::default();
        for lesson in lessons.iter().filter(|lesson| late.is_none_or(|late| lesson.late == late)) {
            for pair in lesson.said.windows(ORDER_PAIR) {
                let [first, second] = pair else { continue };
                let (one, other) = (order_side_key(&keys[*first as usize], way), order_side_key(&keys[*second as usize], way));
                if let Some((left, right)) = one.iter().zip(&other).find(|(left, right)| left != right) {
                    order.add(u32::from(*left), u32::from(*right));
                }
            }
        }
        order.settle();
        order
    }

    fn learn(cases: &[Case], lanes: usize) -> Learned {
        let mut symbols = Symbols::new();
        let (default, default_share) = frequent_answer(cases);
        let (_, last) = inner_parts(cases);
        let mut late = vec![false; cases.len()];
        for place in last {
            late[place] = true;
        }
        let mut name = |text: &str| symbols.number(text);
        let lessons: Vec<OrderLesson> = cases
            .iter()
            .zip(&late)
            .map(|(case, late)| {
                let mut lines: Vec<Vec<u32>> = case.sentences.iter().map(|text| order_items(text, &mut name)).collect();
                lines.push(order_items(&case.question, &mut name));
                OrderLesson { lines, said: order_items(&case.answer, &mut name), late: *late }
            })
            .collect();
        let keys: Vec<Vec<char>> = symbols.names().iter().map(|text| order_key(text, OrderWay::FromLeft)).collect();
        let mut model = OrderModel { symbols, keys, default, place: None, sides: [None, None], choice: None, rules: Vec::new(), best: Vec::new() };
        let mut found: Vec<OrderFound> = Vec::new();
        model.put_in_order(&lessons, &mut found);
        model.choose(&lessons, lanes, &mut found);
        found.sort_by(|left, right| right.share.partial_cmp(&left.share).unwrap_or(Ordering::Equal).then(left.way.cmp(&right.way)));
        for way in found {
            if way.tried >= ORDER_SURE_LESSONS && (way.right as f64) >= ORDER_SURE_SHARE * (way.tried as f64) {
                model.rules.push(way);
            } else if way.share > default_share {
                model.best.push(way);
            }
        }
        Ok(Box::new(model))
    }

    fn put_in_order(&mut self, lessons: &[OrderLesson], found: &mut Vec<OrderFound>) {
        let mut places: Vec<(OrderPlace, usize)> = Vec::new();
        for lesson in lessons.iter().filter(|lesson| lesson.said.len() >= ORDER_PAIR) {
            let Some(place) = order_run_place(&lesson.lines, &lesson.said) else { continue };
            match places.iter_mut().find(|(held, _)| *held == place) {
                Some((_, count)) => *count += 1,
                None => places.push((place, 1)),
            }
        }
        let mut top: Option<(OrderPlace, usize)> = None;
        for (place, count) in &places {
            if top.is_none_or(|(_, held)| *count > held) {
                top = Some((*place, *count));
            }
        }
        let Some((place, count)) = top else { return };
        if (count as f64) < ORDER_SURE_SHARE * (lessons.len() as f64) {
            return;
        }
        self.place = Some(place);
        let strangers = Symbols::new();
        for (side, way) in [OrderWay::FromLeft, OrderWay::FromRight].into_iter().enumerate() {
            let early = OrderModel::symbol_order(lessons, &self.keys, way, Some(false)).ranks();
            let (mut tried, mut right) = (0usize, 0usize);
            for lesson in lessons.iter().filter(|lesson| lesson.late) {
                tried += 1;
                right += usize::from(self.sorted_run(&lesson.lines, way, &early, &strangers).as_deref() == Some(lesson.said.as_slice()));
            }
            self.sides[side] = Some(OrderModel::symbol_order(lessons, &self.keys, way, None).ranks());
            found.push(OrderFound { way, share: (right as f64) / (tried.max(1) as f64), tried, right });
        }
    }

    fn choose(&mut self, lessons: &[OrderLesson], lanes: usize, found: &mut Vec<OrderFound>) {
        let runs = shards(lessons.len(), lanes);
        let seen: Vec<Option<OrderTwoRuns>> = on_lanes(runs.len(), lanes, &|run| lessons[runs[run].clone()].iter().map(|lesson| order_two_runs(&lesson.lines)).collect::<Vec<Option<OrderTwoRuns>>>()).concat();
        let usable: Vec<(&OrderTwoRuns, (usize, OrderHow), bool)> = seen.iter().zip(lessons).filter_map(|(runs, lesson)| runs.as_ref().and_then(|runs| order_named(runs, &lesson.said).map(|named| (runs, named, lesson.late)))).collect();
        if usable.is_empty() || (usable.len() as f64) < ORDER_SURE_SHARE * (lessons.len() as f64) {
            return;
        }
        let mut hows: Vec<(OrderHow, usize)> = Vec::new();
        for (_, (_, how), _) in &usable {
            match hows.iter_mut().find(|(held, _)| held == how) {
                Some((_, count)) => *count += 1,
                None => hows.push((*how, 1)),
            }
        }
        let how = hows.iter().fold(None, |top: Option<(OrderHow, usize)>, (how, count)| if top.is_none_or(|(_, held)| *count > held) { Some((*how, *count)) } else { top }).map(|(how, _)| how).unwrap_or(OrderHow::Run);
        let mut early = OrderCount::default();
        let mut all = OrderCount::default();
        for (runs, (which, _), late) in &usable {
            let right = &runs.runs[*which];
            for (place, first) in right.iter().enumerate() {
                for second in &right[place + 1..] {
                    if !late {
                        early.add(*first, *second);
                    }
                    all.add(*first, *second);
                }
            }
        }
        early.settle();
        all.settle();
        let (mut tried, mut right) = (0usize, 0usize);
        for (runs, (which, _), late) in &usable {
            if *late {
                tried += 1;
                right += usize::from(order_chosen(runs, &early) == *which);
            }
        }
        self.choice = Some((how, all));
        found.push(OrderFound { way: OrderWay::Choose, share: (right as f64) / (tried.max(1) as f64), tried, right });
    }

    fn by(&self, way: OrderWay, lines: &[Vec<u32>], strangers: &Symbols) -> Option<String> {
        let items: Vec<u32> = match way {
            OrderWay::Choose => {
                let runs = order_two_runs(lines)?;
                let (how, order) = self.choice.as_ref()?;
                let which = order_chosen(&runs, order);
                match how {
                    OrderHow::Run => runs.runs[which].clone(),
                    OrderHow::ItemBefore => vec![runs.before[which]?],
                }
            }
            OrderWay::FromLeft | OrderWay::FromRight => {
                let side = usize::from(way == OrderWay::FromRight);
                self.sorted_run(lines, way, self.sides[side].as_ref()?, strangers)?
            }
        };
        let words: Vec<String> = items.iter().map(|item| self.name(*item, strangers)).collect();
        Some(words.join(ORDER_BETWEEN)).filter(|text| !text.is_empty())
    }
}

fn order_side_key(key: &[char], way: OrderWay) -> Vec<char> {
    if way == OrderWay::FromLeft {
        return key.to_vec();
    }
    let mut turned: Vec<char> = key[..key.len().saturating_sub(1)].to_vec();
    turned.reverse();
    turned.push(ORDER_END);
    turned
}
because!(order_side_key, OrderWays, "the key of an item from the side of a way, made from its key from the left end: the same key, or its symbols turned round with the end symbol kept at the end");

impl Learner for OrderModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let mut strangers = Symbols::new();
        let known = self.symbols.len() as u32;
        let mut name = |text: &str| self.symbols.find(text).unwrap_or_else(|| known + strangers.number(text));
        let mut lines: Vec<Vec<u32>> = sentences.iter().map(|text| order_items(text, &mut name)).collect();
        lines.push(order_items(question, &mut name));
        for (ways, by) in [(&self.rules, "list"), (&self.best, "best")] {
            for found in ways {
                if let Some(text) = self.by(found.way, &lines, &strangers) {
                    let way = ORDER_WAY_NAMES.iter().find(|(held, _)| *held == found.way).map_or("", |(_, name)| *name);
                    return Said { words: answer_words(&text), note: format!("{way}: right for {} of {} lessons of the part held back; {by}", found.right, found.tried) };
                }
            }
        }
        Said { words: self.default.clone(), note: String::new() }
    }
}

pub(super) fn learn_order(lessons: Lessons, lanes: usize) -> Learned {
    OrderModel::learn(lessons.cases, lanes)
}
because!(learn_order, OrderWays, "the order tool as a tool of the one call: it reads every kind, since a kind with no order in its answers gets the most frequent answer");
