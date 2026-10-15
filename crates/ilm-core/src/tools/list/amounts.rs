use crate::base::exact::Amount;
use crate::base::keys::NO_NUMBER;
use crate::base::symbols::Symbols;
use crate::base::words::{answer_words, words};
use crate::cases::Case;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct ListAmounts;
source!(ListAmounts, "a told list is cut at its commas and semicolons into parts, and a part is an item: its first word and the words of a thing. The amount of each first word and of each answer is counted from the lessons in one table: the most frequent first word counts once, an answer has the most frequent sum of its lessons whose items all have an amount, and another first word has the most frequent rest of the lessons where it is the only unknown; both counts are made again until nothing changes");

pub(crate) const LIST_SURE_N: i64 = 20;
because!(LIST_SURE_N, ListAmounts, "the readings of the sure test of the tools: a joining word must start this many parts, and a model must give back this many answers of its lessons");

pub(crate) const LIST_SURE_SHARE: f64 = 0.98;
because!(LIST_SURE_SHARE, ListAmounts, "the share of the sure test of the tools, for the lead of a list, a joining word, the answers that two amounts must not share, and the answers that a model gives back");

const LIST_OWN: f64 = 0.5;
because!(LIST_OWN, ListAmounts, "the tool reads a kind when this share of its lessons have a sum of amounts that is the amount of their answer; with fewer the lessons are no lists with amounts");

pub(crate) const LIST_ROUNDS: usize = 20;
because!(LIST_ROUNDS, ListAmounts, "the counts of the amounts and of the classes are made again at most this often; they settle in a few rounds");

const LIST_LEAST_READINGS: i64 = 2;
because!(LIST_LEAST_READINGS, ListAmounts, "the lessons that must give one amount to a word or an answer, at least: one lesson can be wrong");

const LIST_HALVES: i64 = 2;
because!(LIST_HALVES, ListAmounts, "an amount is taken when its lessons are more than the half of the lessons that give the word an amount, and the lessons answer in digits when more than the half of them do");

const LIST_JOINED_PART: usize = 2;
because!(LIST_JOINED_PART, ListAmounts, "a part that a joining word starts must hold more words than the joining word and the word that starts parts after it, so that a thing is left when the joining word goes");

const LIST_CUT_SIGNS: [char; 2] = [',', ';'];
because!(LIST_CUT_SIGNS, ListAmounts, "the signs between the parts of a told list");

pub(crate) const LIST_MOST_DIGITS: usize = 12;
because!(LIST_MOST_DIGITS, ListAmounts, "the places after the point of an amount that is written as a decimal, at most; an amount whose decimal does not end there has no written form");

const LIST_NUMBER_SIGNS: &str = "-+*/=<>()[]{}^%$";
because!(LIST_NUMBER_SIGNS, ListAmounts, "the signs that are tokens of their own when a text is read for one number: the signs of arithmetic, the brackets, and the signs of a power, a share and money; a text with one of them beside its number holds more than one number token");

const LIST_JOINT_SIGNS: &str = "^/";
because!(LIST_JOINT_SIGNS, ListAmounts, "the signs that join the parts of a unit into one word, so that a unit of area or of speed is one token");

const LIST_BEFORE_MINUS: &str = " \t\n([{=+-*/<>:,";
because!(LIST_BEFORE_MINUS, ListAmounts, "a minus sign after one of these signs, or at the start of a text, belongs to the number after it; after a letter, a digit or a closing bracket it is a subtraction");

const LIST_MINUS: u8 = b'-';
because!(LIST_MINUS, ListAmounts, "the sign that makes the number after it negative where no subtraction can stand");

const LIST_POINT: u8 = b'.';
because!(LIST_POINT, ListAmounts, "the mark between the whole part of a number and its digits below one");

const LIST_LOW_LINE: u8 = b'_';
because!(LIST_LOW_LINE, ListAmounts, "the mark that a text of a program writes inside a name, a letter of a word");

pub fn one_number(text: &str) -> Option<Amount> {
    let bytes = text.as_bytes();
    let mut at = 0;
    let mut tokens = 0usize;
    let mut found: Option<Amount> = None;
    let mut minus_end: Option<usize> = None;
    while at < bytes.len() {
        let start = at;
        if bytes[at].is_ascii_digit() {
            while at < bytes.len() && bytes[at].is_ascii_digit() {
                at += 1;
            }
            let whole_end = at;
            if at + 1 < bytes.len() && bytes[at] == LIST_POINT && bytes[at + 1].is_ascii_digit() {
                at += 1;
                while at < bytes.len() && bytes[at].is_ascii_digit() {
                    at += 1;
                }
            }
            let after = if at > whole_end { &text[whole_end + 1..at] } else { "" };
            let mut value = Amount::from_digits(&text[start..whole_end], after);
            if minus_end == Some(start) {
                tokens -= 1;
                value = value.and_then(Amount::negated);
            }
            tokens += 1;
            found = if tokens == 1 { value } else { None };
            minus_end = None;
        } else if bytes[at].is_ascii_alphabetic() || bytes[at] == LIST_LOW_LINE {
            while at < bytes.len() && (bytes[at].is_ascii_alphabetic() || bytes[at] == LIST_LOW_LINE) {
                at += 1;
            }
            while at + 1 < bytes.len() && LIST_JOINT_SIGNS.as_bytes().contains(&bytes[at]) && bytes[at + 1].is_ascii_alphanumeric() {
                at += 1;
                while at < bytes.len() && bytes[at].is_ascii_alphanumeric() {
                    at += 1;
                }
            }
            tokens += 1;
            found = None;
            minus_end = None;
        } else if bytes[at].is_ascii() && LIST_NUMBER_SIGNS.as_bytes().contains(&bytes[at]) {
            at += 1;
            let may_join = bytes[start] == LIST_MINUS && (start == 0 || (bytes[start - 1].is_ascii() && LIST_BEFORE_MINUS.as_bytes().contains(&bytes[start - 1])));
            minus_end = may_join.then_some(at);
            tokens += 1;
            found = None;
        } else {
            at += 1;
        }
    }
    if tokens == 1 { found } else { None }
}
because!(one_number, ListAmounts, "the amount of a text that is one number and nothing else, read as a learner of calculations reads numbers: digits with or without a decimal part, and a minus sign joined to them where no subtraction can stand; nothing for a text with a word, a sign or a second number beside it");

pub fn above_nought(value: Amount) -> bool {
    value.written(LIST_MOST_DIGITS).is_some_and(|text| !text.starts_with(char::from(LIST_MINUS)) && value != Amount::whole(0))
}
because!(above_nought, ListAmounts, "is an amount above nought: the rest of a lesson is an amount of a first word only then; the amounts here are sums and differences of numbers that a text writes, so their decimals end and the written form shows the sign");

pub fn parts_of(sentences: &[String]) -> Vec<Vec<String>> {
    sentences.iter().flat_map(|sentence| sentence.split(LIST_CUT_SIGNS).map(words)).filter(|part| !part.is_empty()).collect()
}
because!(parts_of, ListAmounts, "the parts of the told sentences between their cut signs, each as its words, the parts with no word left out");

#[derive(Clone, Debug, Default)]
pub struct ListCounts {
    pub symbols: Symbols,
    pub lead: Vec<u32>,
    pub joins: HashSet<u32>,
    pub amount: HashMap<Vec<u32>, Amount>,
    pub text_of: HashMap<Amount, Vec<String>>,
    pub digits: bool,
    pub whole: usize,
    pub reads: bool,
}
because!(ListCounts, ListAmounts, "what the tool counts in the lists of its lessons: the lead that starts nearly every first part, the joining words that start a part before a word that starts parts, the amount of each first word and of each answer, the words of the answer of each amount, whether the lessons answer in digits, how many lessons have a sum that is the amount of their answer, and whether the lessons are lists with amounts at all. learn makes it from the lessons; items cuts a list into first words and things; amount_of and answer_amount give an amount or nothing");

impl ListCounts {
    pub fn numbered(&self, found: &[String], fresh: &mut Vec<String>) -> Vec<u32> {
        found
            .iter()
            .map(|word| match self.symbols.find(word) {
                Some(number) => number,
                None => match fresh.iter().position(|seen| seen == word) {
                    Some(place) => (self.symbols.len() + place) as u32,
                    None => {
                        fresh.push(word.clone());
                        (self.symbols.len() + fresh.len() - 1) as u32
                    }
                },
            })
            .collect()
    }

    pub fn name<'a>(&'a self, number: u32, fresh: &'a [String]) -> &'a str {
        let known = self.symbols.len();
        if (number as usize) < known { self.symbols.name(number) } else { fresh[number as usize - known].as_str() }
    }

    fn strip_lead(&self, parts: &[Vec<u32>]) -> Vec<Vec<u32>> {
        let Some(first) = parts.first() else {
            return Vec::new();
        };
        let first = if first.starts_with(&self.lead) { first[self.lead.len()..].to_vec() } else { first.clone() };
        std::iter::once(first).chain(parts[1..].iter().cloned()).filter(|part| !part.is_empty()).collect()
    }

    pub fn items(&self, parts: &[Vec<u32>]) -> Vec<(Option<u32>, Vec<u32>)> {
        let mut out = Vec::new();
        for mut part in self.strip_lead(parts) {
            while part.len() > 1 && self.joins.contains(&part[0]) {
                part.remove(0);
            }
            if part.len() > 1 {
                out.push((Some(part[0]), part[1..].to_vec()));
            } else {
                out.push((None, part));
            }
        }
        out
    }

    pub fn amount_of(&self, first: Option<u32>, fresh: &[String]) -> Option<Amount> {
        let Some(first) = first else {
            return Some(Amount::whole(1));
        };
        match self.amount.get(&vec![first]) {
            Some(value) => Some(*value),
            None => one_number(self.name(first, fresh)),
        }
    }

    pub fn answer_amount(&self, answer: &str) -> Option<Amount> {
        let key: Vec<u32> = answer_words(answer).iter().map(|word| self.symbols.find(word).unwrap_or(NO_NUMBER)).collect();
        one_number(answer).or_else(|| self.amount.get(&key).copied())
    }

    fn sum_of(values: &[Option<Amount>]) -> Option<Amount> {
        values.iter().flatten().try_fold(Amount::whole(0), |total, value| total.plus(*value))
    }

    pub fn learn(rows: &[(Vec<Vec<String>>, &Case)], symbols: Symbols) -> ListCounts {
        let mut made = ListCounts { symbols, ..ListCounts::default() };
        let lists: Vec<Vec<Vec<u32>>> = rows.iter().map(|(parts, _)| parts.iter().map(|part| made.symbols.numbers(part)).collect()).collect();
        loop {
            let size = made.lead.len() + 1;
            let mut starts: Vec<(Vec<u32>, usize)> = Vec::new();
            for parts in &lists {
                if parts[0].len() > size {
                    let start = parts[0][..size].to_vec();
                    match starts.iter_mut().find(|(seen, _)| *seen == start) {
                        Some((_, count)) => *count += 1,
                        None => starts.push((start, 1)),
                    }
                }
            }
            let mut best: Option<&(Vec<u32>, usize)> = None;
            for entry in &starts {
                if best.is_none_or(|top| entry.1 > top.1) {
                    best = Some(entry);
                }
            }
            match best {
                Some((start, count)) if (*count as f64) >= LIST_SURE_SHARE * (lists.len() as f64) => made.lead = start.clone(),
                _ => break,
            }
        }
        let cut: Vec<Vec<Vec<u32>>> = lists.iter().map(|parts| made.strip_lead(parts)).collect();
        let mut heads: Vec<(u32, i64)> = Vec::new();
        for part in cut.iter().flatten() {
            match heads.iter_mut().find(|(word, _)| *word == part[0]) {
                Some((_, count)) => *count += 1,
                None => heads.push((part[0], 1)),
            }
        }
        let starts: HashSet<u32> = heads.iter().map(|(word, _)| *word).collect();
        for (word, count) in &heads {
            let joined = cut.iter().flatten().filter(|part| part[0] == *word && part.len() > LIST_JOINED_PART && starts.contains(&part[1]) && part[1] != *word).count() as i64;
            if *count >= LIST_SURE_N && (joined as f64) >= LIST_SURE_SHARE * (*count as f64) {
                made.joins.insert(*word);
            }
        }
        made.learn_amounts(rows, &lists);
        made
    }

    fn learn_amounts(&mut self, rows: &[(Vec<Vec<String>>, &Case)], lists: &[Vec<Vec<u32>>]) {
        let items: Vec<Vec<(Option<u32>, Vec<u32>)>> = lists.iter().map(|parts| self.items(parts)).collect();
        let keys: Vec<Vec<u32>> = rows.iter().map(|(_, case)| answer_words(&case.answer).iter().map(|word| self.symbols.find(word).unwrap_or(NO_NUMBER)).collect()).collect();
        let mut firsts: Vec<(u32, i64)> = Vec::new();
        for (first, _) in items.iter().flatten() {
            let Some(first) = first else { continue };
            if one_number(self.symbols.name(*first)).is_some() {
                continue;
            }
            match firsts.iter_mut().find(|(word, _)| word == first) {
                Some((_, count)) => *count += 1,
                None => firsts.push((*first, 1)),
            }
        }
        let mut most: Option<(u32, i64)> = None;
        for (word, count) in &firsts {
            if most.is_none_or(|(_, top)| *count > top) {
                most = Some((*word, *count));
            }
        }
        if let Some((word, _)) = most {
            self.amount.insert(vec![word], Amount::whole(1));
        }
        for _ in 0..LIST_ROUNDS {
            let mut new = self.amount.clone();
            let mut sums: Vec<(Vec<u32>, Vec<(Amount, i64)>)> = Vec::new();
            let mut rests: Vec<(Vec<u32>, Vec<(Amount, i64)>)> = Vec::new();
            let bump = |table: &mut Vec<(Vec<u32>, Vec<(Amount, i64)>)>, key: Vec<u32>, value: Amount| {
                let at = match table.iter().position(|(seen, _)| *seen == key) {
                    Some(at) => at,
                    None => {
                        table.push((key, Vec::new()));
                        table.len() - 1
                    }
                };
                match table[at].1.iter_mut().find(|(seen, _)| *seen == value) {
                    Some((_, count)) => *count += 1,
                    None => table[at].1.push((value, 1)),
                }
            };
            for ((found, (_, case)), key) in items.iter().zip(rows).zip(&keys) {
                let amounts: Vec<Option<Amount>> = found.iter().map(|(first, _)| self.amount_of(*first, &[])).collect();
                let mut open: Vec<Option<u32>> = Vec::new();
                for ((first, _), value) in found.iter().zip(&amounts) {
                    if value.is_none() && !open.contains(first) {
                        open.push(*first);
                    }
                }
                let total = self.answer_amount(&case.answer);
                if open.is_empty() {
                    if total.is_none() && !key.is_empty() {
                        if let Some(sum) = ListCounts::sum_of(&amounts) {
                            bump(&mut sums, key.clone(), sum);
                        }
                    }
                } else if open.len() == 1 {
                    let (Some(total), Some(word)) = (total, open[0]) else { continue };
                    let Some(known) = ListCounts::sum_of(&amounts) else { continue };
                    let times = amounts.iter().filter(|value| value.is_none()).count() as i128;
                    let Some(rest) = total.minus(known) else { continue };
                    if above_nought(rest) {
                        if let Some(each) = rest.over(Amount::whole(times)).filter(Amount::is_whole) {
                            bump(&mut rests, vec![word], each);
                        }
                    }
                }
            }
            for table in [&sums, &rests] {
                for (key, counts) in table {
                    let mut best: Option<(Amount, i64)> = None;
                    for (value, count) in counts {
                        if best.is_none_or(|(_, top)| *count > top) {
                            best = Some((*value, *count));
                        }
                    }
                    let all: i64 = counts.iter().map(|(_, count)| *count).sum();
                    if let Some((value, count)) = best {
                        if !new.contains_key(key) && count >= LIST_LEAST_READINGS && LIST_HALVES * count > all {
                            new.insert(key.clone(), value);
                        }
                    }
                }
            }
            if new == self.amount {
                break;
            }
            self.amount = new;
        }
        let mut texts: Vec<(Amount, Vec<(Vec<String>, i64)>)> = Vec::new();
        let mut digits = 0usize;
        for (_, case) in rows {
            let said = answer_words(&case.answer);
            digits += usize::from(one_number(&case.answer).is_some());
            let Some(total) = self.answer_amount(&case.answer) else { continue };
            let at = match texts.iter().position(|(seen, _)| *seen == total) {
                Some(at) => at,
                None => {
                    texts.push((total, Vec::new()));
                    texts.len() - 1
                }
            };
            match texts[at].1.iter_mut().find(|(seen, _)| *seen == said) {
                Some((_, count)) => *count += 1,
                None => texts[at].1.push((said, 1)),
            }
        }
        let mut agree = 0i64;
        let mut every = 0i64;
        for (total, counts) in &texts {
            let mut best: Option<&(Vec<String>, i64)> = None;
            for entry in counts {
                if best.is_none_or(|top| entry.1 > top.1) {
                    best = Some(entry);
                }
            }
            if let Some((said, count)) = best {
                self.text_of.insert(*total, said.clone());
                agree += count;
            }
            every += counts.iter().map(|(_, count)| *count).sum::<i64>();
        }
        self.digits = (LIST_HALVES * digits as i64) > rows.len() as i64;
        self.whole = items
            .iter()
            .zip(rows)
            .filter(|(found, (_, case))| {
                let total = self.answer_amount(&case.answer);
                let amounts: Vec<Option<Amount>> = found.iter().map(|(first, _)| self.amount_of(*first, &[])).collect();
                total.is_some() && amounts.iter().all(Option::is_some) && ListCounts::sum_of(&amounts) == total
            })
            .count();
        self.reads = (self.whole as f64) >= LIST_OWN * (rows.len() as f64) && (agree as f64) >= LIST_SURE_SHARE * (every as f64);
    }
}
