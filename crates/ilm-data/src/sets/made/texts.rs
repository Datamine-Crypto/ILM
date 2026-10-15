use ilm_core::base::twister::TwisterDraws;
use crate::sets::made::family::{Family, MadeCase, MadeTree};
use crate::sets::made::reader::made_answers;
use patterns::{because, source};
use std::collections::HashSet;

pub struct MadeTexts;
source!(MadeTexts, "the texts of a family: trees grown at random from the functions of the family to a drawn depth, one branch as deep as it may go, written out as texts with their answers; the lessons and the read parts hold no text twice, and the read parts of a family whose items say themselves also hold items that no lesson holds. A family whose first texts do not each have one reading gives no texts, and the next draw of the family is taken");

const TEXT_LESSON_DEPTHS: &[usize] = &[0, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 4, 4];
because!(TEXT_LESSON_DEPTHS, MadeTexts, "the depths of the lesson texts, drawn one of these: most often two functions inside each other, sometimes an item alone");

const TEXT_READ_DEPTHS: &[usize] = &[1, 2, 2, 3, 3, 4, 4];
because!(TEXT_READ_DEPTHS, MadeTexts, "the depths of the texts of the read part, drawn one of these");

const TEXT_DEEP_DEPTHS: &[usize] = &[5, 6];
because!(TEXT_DEEP_DEPTHS, MadeTexts, "the depths of the texts of the deep read part, deeper than any lesson");

const TEXT_LONGEST: (usize, usize) = (36, 48);
because!(TEXT_LONGEST, MadeTexts, "the most words of a text and of its answer, for the lessons and the read part");

const TEXT_LONGEST_DEEP: (usize, usize) = (64, 96);
because!(TEXT_LONGEST_DEEP, MadeTexts, "the most words of a text and of its answer, for the deep read part");

const TEXT_TRIES: usize = 60000;
because!(TEXT_TRIES, MadeTexts, "the draws of trees for the texts of one part before the family is drawn again: enough for a family that can make its texts");

const TEXT_STOP: f64 = 0.3;
because!(TEXT_STOP, MadeTexts, "the share of the places of a tree that stop at an item, for a function of one part");

const TEXT_STOP_PART: f64 = 0.15;
because!(TEXT_STOP_PART, MadeTexts, "how much more often a place stops at an item for each more part of its function, so that wide functions do not make every text long");

const TEXT_RENAMED: f64 = 0.6;
because!(TEXT_RENAMED, MadeTexts, "the share of the texts of a read part whose items may be changed into items that no lesson holds");

const TEXT_NEW_ITEM: f64 = 0.3;
because!(TEXT_NEW_ITEM, MadeTexts, "the share of the items of such a text that become items that no lesson holds");

const TEXT_CHECKED: (usize, usize, usize) = (60, 30, 30);
because!(TEXT_CHECKED, MadeTexts, "the first lessons, texts of the read part and texts of the deep read part whose one reading is checked by the interpreter before the family is taken");

const TEXT_BLANK: &str = " ";
because!(TEXT_BLANK, MadeTexts, "the mark between the words of a text, and between the words of an answer that is no row of signs");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MadeAgain;
because!(MadeAgain, MadeTexts, "a draw of a family that gives no texts: the next draw is taken");

impl Family {
    fn grow(&self, rng: &mut TwisterDraws, budget: usize, level: Option<u32>, must: bool) -> MadeTree {
        let ways: Vec<(usize, usize)> = self.functions.iter().enumerate().flat_map(|(number, group)| group.iter().enumerate().filter(move |(_, way)| level.is_none_or(|top| way.level <= top)).map(move |(way_number, _)| (number, way_number))).collect();
        if budget == 0 || ways.is_empty() {
            return MadeTree::Item(self.items[rng.choice(self.items.len())].clone());
        }
        let mut numbers: Vec<usize> = ways.iter().map(|(number, _)| *number).collect();
        numbers.sort_unstable();
        numbers.dedup();
        let number = numbers[rng.choice(numbers.len())];
        let own: Vec<usize> = ways.iter().filter(|(held, _)| *held == number).map(|(_, way_number)| *way_number).collect();
        let way_number = own[rng.choice(own.len())];
        let way = &self.functions[number][way_number];
        if !must && rng.unit() < TEXT_STOP + TEXT_STOP_PART * ((way.parts - 1) as f64) {
            return MadeTree::Item(self.items[rng.choice(self.items.len())].clone());
        }
        let deep = must.then(|| rng.choice(way.parts));
        let kids: Vec<MadeTree> = (0..way.parts).map(|part| self.grow(rng, budget - 1, way.accept[part], deep == Some(part))).collect();
        MadeTree::Node(number, way_number, kids)
    }

    fn renamed(&self, rng: &mut TwisterDraws, tree: &MadeTree) -> MadeTree {
        match tree {
            MadeTree::Item(item) => {
                if rng.unit() < TEXT_NEW_ITEM {
                    MadeTree::Item(self.new_items[rng.choice(self.new_items.len())].clone())
                } else {
                    MadeTree::Item(item.clone())
                }
            }
            MadeTree::Node(number, way, kids) => MadeTree::Node(*number, *way, kids.iter().map(|kid| self.renamed(rng, kid)).collect()),
        }
    }

    fn drawn_texts(&self, rng: &mut TwisterDraws, depths: &[usize], count: usize, longest: (usize, usize), taken: &mut HashSet<String>, new: bool) -> Result<Vec<MadeCase>, MadeAgain> {
        let low = depths.iter().copied().min().unwrap_or(0);
        let high = depths.iter().copied().max().unwrap_or(0);
        let mut cases: Vec<MadeCase> = Vec::with_capacity(count);
        for _ in 0..TEXT_TRIES {
            if cases.len() == count {
                return Ok(cases);
            }
            let budget = depths[rng.choice(depths.len())];
            let mut tree = self.grow(rng, budget, None, true);
            if new && rng.unit() < TEXT_RENAMED {
                tree = self.renamed(rng, &tree);
            }
            let depth = Family::depth(&tree);
            if depth < low || depth > high {
                continue;
            }
            let text = self.text(&tree);
            let value = self.value(&tree);
            let key = text.join(TEXT_BLANK);
            if taken.contains(&key) || text.len() > longest.0 || value.is_empty() || value.len() > longest.1 {
                continue;
            }
            taken.insert(key.clone());
            cases.push(MadeCase { text: key, answer: value.join(self.between()), tree });
        }
        Err(MadeAgain)
    }

    pub(super) fn make(&mut self) -> Result<(), MadeAgain> {
        let mut rng = TwisterDraws::seeded(&format!("made family {}, texts", self.number));
        let mut taken: HashSet<String> = HashSet::new();
        let new = !self.new_items.is_empty();
        self.lessons = self.drawn_texts(&mut rng, TEXT_LESSON_DEPTHS, MADE_LESSONS_COUNT, TEXT_LONGEST, &mut taken, false)?;
        self.read = self.drawn_texts(&mut rng, TEXT_READ_DEPTHS, MADE_READ_COUNT, TEXT_LONGEST, &mut taken, new)?;
        self.deep = self.drawn_texts(&mut rng, TEXT_DEEP_DEPTHS, MADE_READ_COUNT, TEXT_LONGEST_DEEP, &mut taken, new)?;
        let (lessons, read, deep) = TEXT_CHECKED;
        let checked = self.lessons.iter().take(lessons).chain(self.read.iter().take(read)).chain(self.deep.iter().take(deep));
        for case in checked {
            let words: Vec<String> = case.text.split(TEXT_BLANK).map(str::to_string).collect();
            if made_answers(self, &words) != [self.value(&case.tree)] {
                return Err(MadeAgain);
            }
        }
        Ok(())
    }
}

pub(super) const MADE_LESSONS_COUNT: usize = 1000;
because!(MADE_LESSONS_COUNT, MadeTexts, "the lessons of a family: as many as the most lessons of every kind of the board");

pub(super) const MADE_READ_COUNT: usize = 400;
because!(MADE_READ_COUNT, MadeTexts, "the texts of each read part of a family: as many as the read part of a kind of the board");
