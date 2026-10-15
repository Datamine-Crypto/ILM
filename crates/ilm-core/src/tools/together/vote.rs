use crate::base::exact::Amount;
use crate::base::first_sight::{FirstSightTable, NumberBag};
use crate::base::symbols::Symbols;
use crate::cases::Case;
use crate::tools::pattern::kinds::PatternWords;
use crate::tools::pattern::names::WordTexts;
use crate::tools::together::marks::{LEAST_MARK_LESSONS, MOST_TEXTS_SHARE, MarkReader, by_size_order};
use crate::tools::together::ratio::Share;
use crate::tools::together::roles::{MOST_NUMBERS, UNUSED, role_order, role_shapes, role_ways};
use crate::tools::together::tokens::{Problem, RESULT_PLACES, TokenMarks, answer_amount, places_after_point};
use crate::tools::together::trees::{Forest, ForestRows};
use patterns::{because, source};

pub struct RoleVote;
source!(RoleVote, "the roles of a new problem in two steps: step one asks of each number alone whether it is used, step two asks which way of roles the used numbers take, once with the numbers in the order of the text and once by size; a set of used numbers counts by the share of the trees of step one, the four most likely sets are asked, and the value with the largest sum of shares is the answer; a value that has not the form of a result of the lessons is dropped");

pub const OWN_SHARE: f64 = 0.5;
because!(OWN_SHARE, RoleVote, "the roles read a kind when the answers of this share of its lessons or more are numbers and a way of roles gives the true number of this share of its lessons or more");

const WAY_WEIGHT: u64 = 720_720;
because!(WAY_WEIGHT, RoleVote, "a lesson with several ways of roles counts this weight shared out among its ways: the least common multiple of the numbers one to sixteen, so a lesson with up to sixteen ways gives each a whole weight");

const WHOLE_SHARES: usize = 16;
because!(WHOLE_SHARES, RoleVote, "a lesson with up to this many ways of roles shares its weight in whole parts; a lesson with more ways gives each the rounded part");

const MOST_SETS: usize = 4;
because!(MOST_SETS, RoleVote, "the most likely sets of used numbers whose ways of roles step two is asked for; a less likely set changes a vote by less than its share");

const USED_SALT: u64 = 9;
because!(USED_SALT, RoleVote, "the number of the forest of step one in the hash of its trees, apart from the two forests of step two");

#[derive(Clone, Debug, Default)]
pub struct RoleModel {
    pub reader: MarkReader,
    pub used: Forest,
    pub forests: Vec<Forest>,
    pub ways: Vec<Vec<Vec<u8>>>,
    pub shapes: Vec<Vec<Vec<u8>>>,
    pub whole: bool,
    pub above_zero: bool,
    pub not_below: bool,
    pub digits: usize,
}
because!(RoleModel, RoleVote, "the roles of a kind: the reader of marks, the forest of step one, the two forests of step two with the ways of roles that are their answers, the ways of roles of each count of numbers, and the form of the results of the lessons (whole, above nought, not below nought, the most places after the point). learn makes it from lessons when the roles read the kind; fits tells whether a value has the form of the results of the lessons; uses gives for each number of a problem the share of the trees of step one that use it; vote gives the value, the way of roles in the order of the text and the share of the vote of a problem");

pub type WeightedWays = Vec<(Vec<u8>, u64)>;
because!(WeightedWays, RoleVote, "the ways of roles of one row of a forest of step two, each with the weight of its lesson");

pub type ValueShares = Vec<(Amount, Share)>;
because!(ValueShares, RoleVote, "the values of a vote, each with the sum of its shares");

pub type WayShares = Vec<(Vec<u8>, Amount, Share)>;
because!(WayShares, RoleVote, "the ways of roles of a vote in the order of the text, each with its value and the sum of its shares");

pub type Vote = (Amount, Vec<u8>, f64);
because!(Vote, RoleVote, "the answer of the roles for a problem: the value, the role of each number in the order of the text, and the share of the vote that the value has");

fn marks_by_number(rows: &[Vec<String>]) -> (Vec<String>, Vec<Vec<u32>>) {
    let mut names: Vec<String> = rows.iter().flatten().cloned().collect();
    names.sort_unstable();
    names.dedup();
    let marks: Vec<Vec<u32>> = rows.iter().map(|row| row.iter().filter_map(|mark| names.binary_search(mark).ok()).map(|at| at as u32).collect()).collect();
    (names, marks)
}
because!(marks_by_number, RoleVote, "the marks of the rows of a forest by their texts in their order, and each row as the numbers of its marks");

fn present(names: &[String], marks: &[String]) -> Vec<bool> {
    let mut has = vec![false; names.len()];
    for mark in marks {
        if let Ok(at) = names.binary_search(mark) {
            has[at] = true;
        }
    }
    has
}
because!(present, RoleVote, "for each mark of a forest whether a problem has it");

fn ways_with_added(ways: &[Vec<u8>], own: &[Amount], amounts: &[Amount]) -> Option<Vec<Vec<u8>>> {
    let added = amounts.len().checked_sub(own.len())?;
    let start = (0..=own.len()).find(|start| amounts[..*start] == own[..*start] && amounts[start + added..] == own[*start..])?;
    Some(ways.iter().map(|roles| roles[..start].iter().copied().chain(std::iter::repeat_n(UNUSED, added)).chain(roles[start..].iter().copied()).collect()).collect())
}
because!(ways_with_added, RoleVote, "the ways of roles of a changed lesson from those of its own lesson: the numbers of the added sentence stand in one block among its own numbers and are not used; none when the numbers do not line up so");

fn kept_only(values: &[Option<Amount>], at: &[usize], kept: &[usize]) -> Vec<Option<Amount>> {
    let keep: Vec<usize> = kept.iter().map(|place| at[*place]).collect();
    values.iter().enumerate().map(|(place, value)| if keep.contains(&place) { *value } else { None }).collect()
}
because!(kept_only, RoleVote, "the amounts of a problem with the numbers that are not used read as plain words");

impl RoleModel {
    pub fn learn(cases: &[Case], changed: &[(usize, Case)], symbols: &mut Symbols, tokens: &TokenMarks) -> Option<RoleModel> {
        let truths: Vec<Option<Amount>> = cases.iter().map(|case| answer_amount(&case.answer)).collect();
        let numeric = truths.iter().filter(|truth| truth.is_some()).count();
        if cases.is_empty() || (numeric as f64) < OWN_SHARE * cases.len() as f64 {
            return None;
        }
        let shapes: Vec<Vec<Vec<u8>>> = (0..=MOST_NUMBERS).map(role_shapes).collect();
        let mut answers: Vec<Amount> = Vec::new();
        let mut read: Vec<(Problem, Vec<usize>, Vec<Vec<u8>>)> = Vec::new();
        let mut fit_of: Vec<Option<usize>> = vec![None; cases.len()];
        let mut problems: Vec<Problem> = Vec::with_capacity(cases.len());
        for case in cases {
            problems.push(tokens.read(symbols, &case.sentences, &case.question));
        }
        for (lesson, (problem, truth)) in problems.iter().zip(&truths).enumerate() {
            let Some(truth) = truth else { continue };
            answers.push(*truth);
            let at: Vec<usize> = (0..problem.values.len()).filter(|place| problem.values[*place].is_some()).collect();
            if problem.numbers > MOST_NUMBERS || at.is_empty() || at.len() > MOST_NUMBERS {
                continue;
            }
            let amounts: Vec<Amount> = at.iter().filter_map(|place| problem.values[*place]).collect();
            let fit = role_ways(&shapes[at.len()], &amounts, *truth);
            if !fit.is_empty() {
                fit_of[lesson] = Some(read.len());
                read.push((problem.clone(), at, fit));
            }
        }
        if (read.len() as f64) < OWN_SHARE * cases.len() as f64 {
            return None;
        }
        let problems_read = read.len();
        for (lesson, case) in changed {
            let Some(own) = fit_of.get(*lesson).copied().flatten() else { continue };
            let problem = tokens.read(symbols, &case.sentences, &case.question);
            let at: Vec<usize> = (0..problem.values.len()).filter(|place| problem.values[*place].is_some()).collect();
            if problem.numbers > MOST_NUMBERS || at.len() > MOST_NUMBERS {
                continue;
            }
            let amounts: Vec<Amount> = at.iter().filter_map(|place| problem.values[*place]).collect();
            let (_, own_at, own_fit) = &read[own];
            let own_amounts: Vec<Amount> = own_at.iter().filter_map(|place| read[own].0.values[*place]).collect();
            let Some(fit) = ways_with_added(own_fit, &own_amounts, &amounts) else { continue };
            read.push((problem, at, fit));
        }
        let all_cases: Vec<Case> = cases.iter().cloned().chain(changed.iter().map(|(_, case)| case.clone())).collect();
        let words = PatternWords::learn(&all_cases, symbols);
        let mut held: FirstSightTable<u32, u32> = FirstSightTable::new();
        for problem in problems.iter().chain(read[problems_read..].iter().map(|(problem, _, _)| problem)) {
            let mut seen: Vec<u32> = problem.words.clone();
            seen.sort_unstable();
            seen.dedup();
            for word in seen {
                held.add(word, 1);
            }
        }
        let is_word = |word: u32| symbols.name(word).as_bytes().first().is_some_and(u8::is_ascii_alphabetic);
        let as_mark: NumberBag<u32> = held.rows().iter().filter(|(word, count)| *count >= LEAST_MARK_LESSONS && is_word(*word)).map(|(word, _)| *word).collect();
        let most_texts: NumberBag<u32> = held.rows().iter().filter(|(word, count)| f64::from(*count) >= MOST_TEXTS_SHARE * all_cases.len() as f64 && is_word(*word)).map(|(word, _)| *word).collect();
        let reader = MarkReader { words, as_mark, most_texts, end: tokens.end, ask: tokens.ask };
        let mut one_marks: Vec<Vec<String>> = Vec::new();
        let mut one_weights: Vec<Vec<(u32, u64)>> = Vec::new();
        let mut view_marks: Vec<Vec<Vec<String>>> = vec![Vec::new(), Vec::new()];
        let mut view_ways: Vec<Vec<WeightedWays>> = vec![Vec::new(), Vec::new()];
        for (problem, at, fit) in &read {
            let amounts: Vec<Amount> = at.iter().filter_map(|place| problem.values[*place]).collect();
            let share = if fit.len() <= WHOLE_SHARES { WAY_WEIGHT / fit.len() as u64 } else { (WAY_WEIGHT as f64 / fit.len() as f64).round_ties_even() as u64 };
            for own in 0..amounts.len() {
                let used = fit.iter().filter(|roles| roles[own] != UNUSED).count() as u64;
                one_marks.push(reader.marks_one(&*symbols, &problem.words, &problem.values, at, own));
                one_weights.push(vec![(0, (fit.len() as u64 - used) * share), (1, used * share)]);
            }
            let mut sets: FirstSightTable<Vec<usize>, Vec<&Vec<u8>>> = FirstSightTable::new();
            for roles in fit {
                let kept: Vec<usize> = (0..roles.len()).filter(|place| roles[*place] != UNUSED).collect();
                sets.entry(kept, Vec::new()).push(roles);
            }
            for (kept, ways) in sets.rows() {
                let left = kept_only(&problem.values, at, kept);
                for (view, by_size) in [false, true].into_iter().enumerate() {
                    let kept_amounts: Vec<Amount> = kept.iter().map(|place| amounts[*place]).collect();
                    let order: Vec<usize> = if by_size { by_size_order(&kept_amounts) } else { (0..kept.len()).collect() };
                    view_marks[view].push(reader.marks(&*symbols, &problem.words, &left, by_size));
                    view_ways[view].push(ways.iter().map(|roles| (order.iter().map(|place| roles[kept[*place]]).collect::<Vec<u8>>(), share)).collect());
                }
            }
        }
        let whole = answers.iter().all(Amount::is_whole);
        let above_zero = answers.iter().all(|answer| *answer > Amount::whole(0));
        let not_below = answers.iter().all(|answer| *answer >= Amount::whole(0));
        let digits = answers.iter().map(|answer| answer.written(RESULT_PLACES).map_or(0, |text| places_after_point(&text))).max().unwrap_or_default();
        let (one_names, one_rows) = marks_by_number(&one_marks);
        let used = Forest::grow(one_names.clone(), &ForestRows { marks: &one_rows, weights: &one_weights, mark_count: one_names.len(), answers: one_weights.first().map_or(0, Vec::len) }, USED_SALT);
        let mut forests: Vec<Forest> = Vec::new();
        let mut all_ways: Vec<Vec<Vec<u8>>> = Vec::new();
        for (view, (marks, ways)) in view_marks.iter().zip(&view_ways).enumerate() {
            let mut answers: Vec<Vec<u8>> = ways.iter().flatten().map(|(roles, _)| roles.clone()).collect();
            answers.sort_by(|one, other| role_order(one, other));
            answers.dedup();
            let weights: Vec<Vec<(u32, u64)>> = ways
                .iter()
                .map(|row| {
                    let mut weights: Vec<(u32, u64)> = row.iter().map(|(roles, weight)| (answers.binary_search_by(|held| role_order(held, roles)).map_or(0, |at| at as u32), *weight)).collect();
                    weights.sort_unstable();
                    weights
                })
                .collect();
            let (names, rows) = marks_by_number(marks);
            forests.push(Forest::grow(names.clone(), &ForestRows { marks: &rows, weights: &weights, mark_count: names.len(), answers: answers.len() }, view as u64));
            all_ways.push(answers);
        }
        Some(RoleModel { reader, used, forests, ways: all_ways, shapes, whole, above_zero, not_below, digits })
    }

    pub fn fits(&self, value: Amount) -> bool {
        let zero = Amount::whole(0);
        if (self.whole && !value.is_whole()) || (self.above_zero && value <= zero) || (self.not_below && value < zero) {
            return false;
        }
        value.is_whole() || value.written(RESULT_PLACES).is_some_and(|text| places_after_point(&text) <= self.digits)
    }

    fn allowed(&self, ways: &[Vec<u8>], amounts: &[Amount]) -> Vec<Option<Amount>> {
        ways.iter()
            .map(|roles| {
                if roles.len() != amounts.len() {
                    return None;
                }
                let value = crate::tools::together::roles::role_value(roles, amounts)?;
                self.fits(value).then_some(value)
            })
            .collect()
    }

    fn ways_of<N: WordTexts>(&self, names: &N, problem: &Problem, values: &[Option<Amount>]) -> (ValueShares, WayShares) {
        let amounts: Vec<Amount> = values.iter().flatten().copied().collect();
        let mut by_value: ValueShares = Vec::new();
        let mut by_way: WayShares = Vec::new();
        let mut trees = 0u64;
        for (view, by_size) in [false, true].into_iter().enumerate() {
            let order: Vec<usize> = if by_size { by_size_order(&amounts) } else { (0..amounts.len()).collect() };
            let ordered: Vec<Amount> = order.iter().map(|place| amounts[*place]).collect();
            let can = self.allowed(&self.ways[view], &ordered);
            if can.iter().all(Option::is_none) {
                continue;
            }
            let good: Vec<bool> = can.iter().map(Option::is_some).collect();
            let marks = self.reader.marks(names, &problem.words, values, by_size);
            let has = present(&self.forests[view].names, &marks);
            let leaves = self.forests[view].leaves(&has, Some(&good));
            trees += self.forests[view].roots.len() as u64;
            for leaf in leaves {
                let all: u64 = leaf.iter().map(|(_, weight)| *weight).sum();
                for (answer, weight) in leaf {
                    let way = &self.ways[view][answer as usize];
                    let Some(value) = can[answer as usize] else { continue };
                    let mut roles = vec![UNUSED; order.len()];
                    for (place, role) in order.iter().zip(way) {
                        roles[*place] = *role;
                    }
                    match by_value.iter_mut().find(|(held, _)| *held == value) {
                        Some((_, share)) => share.add_part(weight, all),
                        None => by_value.push((value, Share::part(weight, all))),
                    }
                    match by_way.iter_mut().find(|(held, held_value, _)| *held == roles && *held_value == value) {
                        Some((_, _, share)) => share.add_part(weight, all),
                        None => by_way.push((roles, value, Share::part(weight, all))),
                    }
                }
            }
        }
        if trees > 0 {
            for (_, share) in by_value.iter_mut() {
                *share = share.over_small(trees);
            }
            for (_, _, share) in by_way.iter_mut() {
                *share = share.over_small(trees);
            }
        }
        (by_value, by_way)
    }

    pub fn uses<N: WordTexts>(&self, names: &N, problem: &Problem, at: &[usize]) -> Vec<Share> {
        let mut uses: Vec<Share> = Vec::with_capacity(at.len());
        for own in 0..at.len() {
            let marks = self.reader.marks_one(names, &problem.words, &problem.values, at, own);
            let has = present(&self.used.names, &marks);
            let leaves = self.used.leaves(&has, None);
            let mut sum = Share::default();
            for leaf in &leaves {
                let all: u64 = leaf.iter().map(|(_, weight)| *weight).sum();
                if let Some((_, weight)) = leaf.iter().find(|(answer, _)| *answer == 1) {
                    sum.add_part(*weight, all);
                }
            }
            uses.push(sum.over_small(leaves.len().max(1) as u64));
        }
        uses
    }

    pub fn vote<N: WordTexts>(&self, names: &N, problem: &Problem) -> Option<Vote> {
        let at: Vec<usize> = (0..problem.values.len()).filter(|place| problem.values[*place].is_some()).collect();
        if at.is_empty() || at.len() > MOST_NUMBERS {
            return None;
        }
        let uses = self.uses(names, problem, &at);
        let mut sets: Vec<(Share, Vec<usize>)> = Vec::new();
        let count = at.len();
        for mask in 1u32..(1u32 << count) {
            let kept: Vec<usize> = (0..count).filter(|place| mask & (1u32 << place) != 0).collect();
            let mut share = Share::whole(1);
            for (place, used) in uses.iter().enumerate() {
                share = if kept.contains(&place) { share.times(used) } else { share.times(&used.complement()) };
            }
            if !share.is_zero() {
                sets.push((share, kept));
            }
        }
        sets.sort_by(|one, other| other.0.order(&one.0).then_with(|| one.1.cmp(&other.1)));
        let mut by_value: ValueShares = Vec::new();
        let mut by_way: WayShares = Vec::new();
        for (share, kept) in sets.iter().take(MOST_SETS) {
            let left = kept_only(&problem.values, &at, kept);
            let (values, ways) = self.ways_of(names, problem, &left);
            for (value, count) in values {
                let add = share.times(&count);
                match by_value.iter_mut().find(|(held, _)| *held == value) {
                    Some((_, held)) => *held = held.plus(&add),
                    None => by_value.push((value, add)),
                }
            }
            for (roles, value, count) in ways {
                let mut full = vec![UNUSED; at.len()];
                for (place, role) in kept.iter().zip(&roles) {
                    full[*place] = *role;
                }
                let full_places: Vec<u8> = full;
                let add = share.times(&count);
                match by_way.iter_mut().find(|(held, held_value, _)| *held == full_places && *held_value == value) {
                    Some((_, _, held)) => *held = held.plus(&add),
                    None => by_way.push((full_places, value, add)),
                }
            }
        }
        let (value, top) = by_value.iter().max_by(|one, other| one.1.order(&other.1).then_with(|| other.0.cmp(&one.0)))?.clone();
        let roles = by_way.iter().filter(|(_, held, _)| *held == value).max_by(|one, other| one.2.order(&other.2).then_with(|| role_order(&one.0, &other.0)))?.0.clone();
        let mut all = Share::default();
        for (_, share) in &by_value {
            all = all.plus(share);
        }
        Some((value, roles, top.over(&all).float()))
    }
}
