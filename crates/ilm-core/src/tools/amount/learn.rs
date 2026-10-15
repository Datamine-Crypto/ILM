use crate::base::exact::Amount;
use crate::base::lanes::{on_lanes, shards};
use crate::base::symbols::Symbols;
use crate::split::told_texts;
use crate::cases::Case;
use crate::tools::amount::calcs::{AMOUNT_PAIRS, amount_calcs};
use crate::tools::amount::model::AmountModel;
use crate::tools::amount::shared::AmountBase;
use crate::tools::amount::steps::amount_tables;
use crate::tools::amount::tokens::{AmountCut, AmountMarks, amount_cut_case};
use crate::tools::call::{Learned, Lessons};
use crate::tools::frequent::frequent_answer;
use patterns::{because, source};

pub struct AmountLessons;
source!(AmountLessons, "the lessons as a learner of calculations takes them: a lesson whose answer holds just one number is counted for the forms, a lesson whose answer is one word can be counted for steps over words, and all lessons with one story are in one of the two parts of the sure test");

pub(super) const AMOUNT_EARLY_PARTS: usize = 2;
because!(AMOUNT_EARLY_PARTS, AmountLessons, "of the equal parts of the lessons, this many are the first part, where the rules are counted; the rest is held back to test them");

pub(super) const AMOUNT_ALL_PARTS: f64 = 3.0;
because!(AMOUNT_ALL_PARTS, AmountLessons, "the equal parts in which the lessons are counted for the cut into a first part and a part that is held back");

#[derive(Clone, Debug, Default)]
pub(super) struct AmountLesson {
    pub(super) cut: AmountCut,
    pub(super) counted: bool,
    pub(super) truth: Option<Amount>,
    pub(super) said: Vec<u32>,
    pub(super) shape: u32,
    pub(super) late: bool,
    pub(super) word: Option<u32>,
}
because!(AmountLesson, AmountLessons, "one lesson as the tool counts it: its tokens, whether its answer holds just one number, that number, the tokens and the form of its answer, whether it lies in the part that is held back, and for an answer of one token with no number that token");

pub(crate) fn amount_late(cases: &[Case]) -> Vec<bool> {
    let (keys, sizes) = told_texts(cases);
    let most = ((cases.len() * AMOUNT_EARLY_PARTS) as f64) / AMOUNT_ALL_PARTS;
    let mut early = vec![false; sizes.len()];
    let mut count = 0usize;
    for (key, size) in sizes.iter().enumerate() {
        if (count as f64) >= most {
            break;
        }
        early[key] = true;
        count += size;
    }
    keys.iter().map(|key| !early[*key]).collect()
}
because!(amount_late, AmountLessons, "for each lesson whether it lies in the part that is held back: the stories are taken in the order of their first lesson into the first part until it holds its share of the lessons; all lessons with one story, the same told sentences, are in one part, and a lesson with no told sentence is a story of its own");

impl AmountModel {
    pub(super) fn learn(cases: &[Case], lanes: usize) -> AmountModel {
        let (calcs, names) = amount_calcs();
        let mut symbols = Symbols::new();
        let marks = AmountMarks::new(&mut symbols, &names);
        let (default, default_share) = frequent_answer(cases);
        let late = amount_late(cases);
        let mut shapes: Vec<Vec<bool>> = vec![vec![true]];
        let mut lessons: Vec<AmountLesson> = Vec::with_capacity(cases.len());
        let mut lower = String::new();
        let mut said = AmountCut::default();
        for (place, case) in cases.iter().enumerate() {
            let mut lesson = AmountLesson { late: late[place], ..AmountLesson::default() };
            said.clear();
            said.scan(&case.answer, marks.number, &mut lower, &mut |word: &str| symbols.number(word));
            if said.spots.len() == 1 {
                let shape: Vec<bool> = said.words.iter().map(|word| *word == marks.number).collect();
                lesson.shape = match shapes.iter().position(|held| *held == shape) {
                    Some(number) => number as u32,
                    None => {
                        shapes.push(shape);
                        shapes.len() as u32 - 1
                    }
                };
                lesson.counted = true;
                lesson.truth = said.values[0];
                lesson.said = said.words.clone();
            } else if said.words.len() == 1 {
                lesson.word = Some(said.words[0]);
            } else {
                lessons.push(lesson);
                continue;
            }
            amount_cut_case(&case.sentences, &case.question, &marks, &mut lesson.cut, &mut lower, &mut |word: &str| symbols.number(word));
            lessons.push(lesson);
        }
        let pairs = AMOUNT_PAIRS.len();
        let base = AmountBase { symbols, marks, calcs, names, pairs, shapes, tables: amount_tables() };
        let ranges = shards(lessons.len(), lanes);
        let counted = on_lanes(ranges.len(), lanes, &|piece| base.count(&lessons, ranges[piece].clone()));
        let mut pieces = counted.into_iter();
        let mut counts = pieces.next().unwrap_or_default();
        for piece in pieces {
            counts.absorb(&piece);
        }
        let mut model = AmountModel::settle(base, &counts, default, default_share);
        model.add_steps(&lessons, lanes);
        model.add_word_steps(&lessons, cases.len(), lanes);
        model
    }
}

pub fn learn_amount(lessons: Lessons, lanes: usize) -> Learned {
    Ok(Box::new(AmountModel::learn(lessons.cases, lanes)))
}
because!(learn_amount, AmountLessons, "the amount tool as a tool of the one call: it learns from the lessons on the given number of threads, and it reads every kind, since a kind with no number in its answers gets the most frequent answer");
