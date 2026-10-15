use crate::base::exact::Amount;
use crate::base::first_sight::NumberBag;
use crate::base::sums::compensated;
use crate::tools::pattern::names::WordTexts;
use crate::tools::together::marks::{MarkReader, is_letter_token};
use crate::tools::together::tokens::Problem;
use patterns::{because, source};
use std::collections::HashMap;

pub struct AskedNumbers;
source!(AskedNumbers, "the numbers that a question names: each word of a told sentence that is no word of most texts belongs to one number of its sentence, the nearest one, where a word before a number counts as farther than a word after it, because a number stands before the words that say what it counts; a word of the question that only one number holds names that number at its place in the question");

pub struct AskedPrograms;
source!(AskedPrograms, "a question that names two numbers on the two sides of one of its words asks for one step between them: a program is an operation on the last number named before a word of the question and the first other number named after it, and the word is found from the lessons with the operation; a new problem takes the program that is right for most of the lessons whose questions share the most rare words with its question, when it is right for enough of them, and else the roles answer");

const BACK_COST: usize = 5;
because!(BACK_COST, AskedNumbers, "a word before a number costs this many times its distance from it, a word after a number its distance: in seven hours on learning english, two hours on learning chinese, the word english belongs to seven; chosen on the development part of SVAMP");

const NEAREST_LESSONS: usize = 30;
because!(NEAREST_LESSONS, AskedPrograms, "the lessons nearest to a new question that vote on its program: enough that one odd lesson does not decide, few enough that they share the form of the question; chosen on the development part of SVAMP");

const ASKED_SURE_PART: f64 = 0.6;
because!(ASKED_SURE_PART, AskedPrograms, "a program answers only when it is right for at least this share of the nearest lessons, and else the roles answer; chosen on the development part of SVAMP");

const LEAST_NAMED: usize = 2;
because!(LEAST_NAMED, AskedPrograms, "a program takes a number named before its word and another one named after it, so a problem or a lesson whose question names fewer numbers has no program");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AskedStep {
    Less,
    Reversed,
    Plus,
    Times,
    Over,
    Under,
}
because!(AskedStep, AskedPrograms, "the operations of a program on the number named before its word and the number named after it: the first less the second, the second less the first, their sum, their product, the first over the second, the second over the first");

const STEPS: &[AskedStep] = &[AskedStep::Less, AskedStep::Reversed, AskedStep::Plus, AskedStep::Times, AskedStep::Over, AskedStep::Under];
because!(STEPS, AskedPrograms, "every operation of a program, in the order in which equal votes are settled");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AskedProgram {
    pub step: AskedStep,
    pub word: u32,
}
because!(AskedProgram, AskedPrograms, "a program: an operation and the word of the question that parts the two named numbers");

#[derive(Clone, Debug, Default)]
pub struct AskedRead {
    pub form: Vec<Option<u32>>,
    pub named: Vec<(usize, usize)>,
    pub values: Vec<Amount>,
    letters: Vec<u32>,
}
because!(AskedRead, AskedNumbers, AskedPrograms, "a problem as the programs read it: the tokens of its question, each a word of the form or a place that names the story (none), the places of the question that name one number with that number, the numbers of the told sentences in their order, and the words of the form that are words and no signs. of reads a problem: the owner of each word of a told sentence, and the places of the question that name one number; run gives the value of a program, the last number named before the last place of its word and the first other number named after it; programs gives every program that runs on the problem, around a word of the form");

#[derive(Clone, Debug)]
struct LessonRow {
    words: Vec<u32>,
    right: Vec<AskedProgram>,
}
because!(LessonRow, AskedPrograms, "a lesson as the vote counts it: the words of the form of its question, sorted, and the programs that give its answer");

#[derive(Clone, Debug, Default)]
pub struct AskedModel {
    rows: Vec<LessonRow>,
    rarity: HashMap<u32, f64>,
}
because!(AskedModel, AskedPrograms, "the programs of a kind: the lessons whose questions name two numbers or more, with their programs, and the rarity of each word of the forms of the questions of all lessons (the logarithm of the lessons over the lessons that hold it). learn counts the rarity over all lessons and keeps the lessons that name two numbers or more with the programs that give their answers, or gives none when no lesson has a program; answer takes the lessons nearest to the question of a problem by the summed rarity of the words that the two forms share, the earlier lesson first among equals, and the program that most of them have, the first in the order of programs among equals; it answers when that program is right for enough of them and runs on the problem");

impl AskedRead {
    pub fn of<N: WordTexts>(reader: &MarkReader, names: &N, problem: &Problem) -> AskedRead {
        let words = &problem.words;
        let ask = words.iter().rposition(|word| *word == reader.ask).unwrap_or_default();
        let local: NumberBag<u32> = words.iter().copied().filter(|word| is_letter_token(names, *word)).collect();
        let numbers: Vec<usize> = (0..ask).filter(|place| problem.values[*place].is_some()).collect();
        let values: Vec<Amount> = numbers.iter().filter_map(|place| problem.values[*place]).collect();
        let is_end = |word: u32| word == reader.end || word == reader.ask;
        let spans: Vec<(usize, usize)> = numbers
            .iter()
            .map(|place| {
                let start = (0..*place).rev().find(|at| is_end(words[*at])).map_or(0, |at| at + 1);
                let end = (*place..words.len()).find(|at| is_end(words[*at])).unwrap_or(words.len());
                (start, end)
            })
            .collect();
        let mut owners: HashMap<u32, Vec<usize>> = HashMap::new();
        let mut told: NumberBag<u32> = NumberBag::default();
        for (place, word) in words.iter().enumerate().take(ask) {
            if !is_letter_token(names, *word) {
                continue;
            }
            let seen = reader.norm(names, *word, &local);
            told.insert(seen);
            if reader.most_texts.contains(word) {
                continue;
            }
            let mut best: Option<(usize, usize)> = None;
            for (number, token) in numbers.iter().enumerate() {
                let (start, end) = spans[number];
                if place < start || place >= end {
                    continue;
                }
                let cost = if place > *token { place - token } else { BACK_COST * (token - place) };
                if best.is_none_or(|(top, _)| cost < top) {
                    best = Some((cost, number));
                }
            }
            if let Some((_, number)) = best {
                let held = owners.entry(seen).or_default();
                if !held.contains(&number) {
                    held.push(number);
                }
            }
        }
        let mut form: Vec<Option<u32>> = Vec::new();
        let mut named: Vec<(usize, usize)> = Vec::new();
        let mut letters: Vec<u32> = Vec::new();
        for (place, word) in words.iter().enumerate().skip(ask + 1) {
            if problem.values[place].is_some() {
                form.push(None);
                continue;
            }
            let seen = reader.norm(names, *word, &local);
            if is_letter_token(names, *word) && !reader.most_texts.contains(word) && told.contains(&seen) {
                if let Some([number]) = owners.get(&seen).map(Vec::as_slice) {
                    named.push((form.len(), *number));
                }
                form.push(None);
            } else {
                if is_letter_token(names, *word) {
                    letters.push(*word);
                }
                form.push(Some(*word));
            }
        }
        letters.sort_unstable();
        letters.dedup();
        AskedRead { form, named, values, letters }
    }

    fn distinct(&self) -> usize {
        let mut seen: Vec<usize> = self.named.iter().map(|(_, number)| *number).collect();
        seen.sort_unstable();
        seen.dedup();
        seen.len()
    }

    fn words(&self) -> Vec<u32> {
        let mut out: Vec<u32> = self.form.iter().flatten().copied().collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    pub fn run(&self, program: &AskedProgram) -> Option<Amount> {
        let at = self.form.iter().rposition(|word| *word == Some(program.word))?;
        let before = self.named.iter().filter(|(place, _)| *place < at).map(|(_, number)| *number).next_back()?;
        let after = self.named.iter().filter(|(place, number)| *place > at && *number != before).map(|(_, number)| *number).next()?;
        let (first, second) = (*self.values.get(before)?, *self.values.get(after)?);
        match program.step {
            AskedStep::Less => first.minus(second),
            AskedStep::Reversed => second.minus(first),
            AskedStep::Plus => first.plus(second),
            AskedStep::Times => first.times(second),
            AskedStep::Over => first.over(second),
            AskedStep::Under => second.over(first),
        }
    }

    fn programs(&self) -> Vec<AskedProgram> {
        let mut out: Vec<AskedProgram> = self.letters.iter().flat_map(|word| STEPS.iter().map(move |step| AskedProgram { step: *step, word: *word })).collect();
        out.retain(|program| self.run(program).is_some());
        out
    }
}

impl AskedModel {
    pub fn learn(reads: &[(AskedRead, Amount)]) -> Option<AskedModel> {
        let mut held: HashMap<u32, usize> = HashMap::new();
        for (read, _) in reads {
            for word in read.words() {
                *held.entry(word).or_default() += 1;
            }
        }
        let all = reads.len() as f64;
        let rarity: HashMap<u32, f64> = held.into_iter().map(|(word, count)| (word, (all / count as f64).ln())).collect();
        let rows: Vec<LessonRow> = reads
            .iter()
            .filter(|(read, _)| read.distinct() >= LEAST_NAMED)
            .map(|(read, truth)| LessonRow { words: read.words(), right: read.programs().into_iter().filter(|program| read.run(program) == Some(*truth)).collect() })
            .collect();
        if rows.iter().all(|row| row.right.is_empty()) {
            return None;
        }
        Some(AskedModel { rows, rarity })
    }

    pub fn answer(&self, read: &AskedRead) -> Option<(Amount, AskedProgram)> {
        if read.distinct() < LEAST_NAMED {
            return None;
        }
        let mine = read.words();
        let mut scored: Vec<(f64, usize)> = self
            .rows
            .iter()
            .enumerate()
            .map(|(place, row)| (compensated(mine.iter().filter(|word| row.words.binary_search(word).is_ok()).map(|word| self.rarity.get(word).copied().unwrap_or_default())), place))
            .collect();
        scored.sort_by(|one, other| other.0.total_cmp(&one.0).then(one.1.cmp(&other.1)));
        let near: Vec<usize> = scored.iter().take(NEAREST_LESSONS).map(|(_, place)| *place).collect();
        let mut votes: HashMap<AskedProgram, usize> = HashMap::new();
        for place in &near {
            for program in &self.rows[*place].right {
                *votes.entry(*program).or_default() += 1;
            }
        }
        let (program, count) = votes.into_iter().max_by(|one, other| one.1.cmp(&other.1).then(other.0.cmp(&one.0)))?;
        if (count as f64) < ASKED_SURE_PART * near.len() as f64 {
            return None;
        }
        Some((read.run(&program)?, program))
    }
}
