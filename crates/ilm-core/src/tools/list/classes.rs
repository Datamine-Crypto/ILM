use crate::base::exact::Amount;
use crate::tools::list::amounts::{LIST_ROUNDS, above_nought};
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct ThingClasses;
source!(ThingClasses, "for each question and each thing of a list: does the thing belong to what the question counts. A lesson whose sum of all amounts is the amount of its answer says that all its things belong; a lesson with a larger sum says that some do not, and when just one group of its undecided things has the amount that is too much, these do not belong and the others do; counted again until nothing changes. A thing that no lesson of the question decides does not belong when it belongs to the class of another question that shares no thing with this one; else it goes with the larger number of decided things of the question");

const LIST_MOST_OPEN: usize = 12;
because!(LIST_MOST_OPEN, ThingClasses, "the things of one lesson that are not known to belong, at most, for every group of them to be tried; more would take too long and say little");

const LIST_SHARED: usize = 5;
because!(LIST_SHARED, ThingClasses, "the things that are decided for two questions, at least, before the classes of the two are said to share no thing");

const LIST_DOUBLE: usize = 2;
because!(LIST_DOUBLE, ThingClasses, "an undecided thing belongs when the things of its question that belong, counted twice, reach the decided ones: the larger part of the decided things wins, and a tie counts for belonging");

pub type Decided = HashMap<Vec<u32>, HashMap<Vec<u32>, (i64, i64)>>;
because!(Decided, ThingClasses, "for each question as its words, for each thing as its words: the lessons that say that the thing belongs and the lessons that say that it does not");

#[derive(Clone, Debug, Default)]
pub struct ListClasses {
    pub decided: Decided,
    pub apart: HashSet<(Vec<u32>, Vec<u32>)>,
}
because!(ListClasses, ThingClasses, "the classes of the things for each question, and the pairs of questions whose classes share no thing. learn counts them from the lessons that have an amount for every item; belongs gives yes, no, or nothing when the lessons do not decide; guess gives what an undecided thing does");

fn count_of(decided: &Decided, question: &[u32], thing: &[u32]) -> (i64, i64) {
    decided.get(question).and_then(|things| things.get(thing)).copied().unwrap_or((0, 0))
}
because!(count_of, ThingClasses, "the lessons for and against a thing in a question, nought for both when no lesson spoke of it");

impl ListClasses {
    pub fn belongs(&self, question: &[u32], thing: &[u32]) -> Option<bool> {
        let (yes, no) = count_of(&self.decided, question, thing);
        if yes == no { None } else { Some(yes > no) }
    }

    pub fn guess(&self, question: &[u32], thing: &[u32]) -> bool {
        for other in self.decided.keys() {
            if self.apart.contains(&(question.to_vec(), other.clone())) && self.belongs(other, thing) == Some(true) {
                return false;
            }
        }
        let things = self.decided.get(question);
        let decided: Vec<bool> = things.map(|found| found.keys().filter_map(|thing| self.belongs(question, thing)).collect()).unwrap_or_default();
        LIST_DOUBLE * decided.iter().filter(|belongs| **belongs).count() >= decided.len()
    }

    pub fn learn(lessons: &[(Vec<u32>, Vec<(Vec<u32>, Amount)>, Amount)]) -> ListClasses {
        let mut made = ListClasses::default();
        for _ in 0..LIST_ROUNDS {
            let mut next: Decided = HashMap::new();
            for (question, things, total) in lessons {
                let Some(all) = things.iter().try_fold(Amount::whole(0), |sum, (_, value)| sum.plus(*value)) else { continue };
                let Some(more) = all.minus(*total) else { continue };
                if !above_nought(more) && more != Amount::whole(0) {
                    continue;
                }
                let out: Vec<usize> = (0..things.len()).filter(|at| {
                    let (yes, no) = count_of(&made.decided, question, &things[*at].0);
                    no > yes
                }).collect();
                let open: Vec<usize> = (0..things.len()).filter(|at| {
                    let (yes, no) = count_of(&made.decided, question, &things[*at].0);
                    yes == no
                }).collect();
                let Some(taken) = out.iter().try_fold(Amount::whole(0), |sum, at| sum.plus(things[*at].1)) else { continue };
                let Some(need) = more.minus(taken) else { continue };
                if (!above_nought(need) && need != Amount::whole(0)) || open.len() > LIST_MOST_OPEN {
                    continue;
                }
                let mut found: Option<u64> = None;
                let mut groups = 0usize;
                for mask in 0u64..(1u64 << open.len()) {
                    let sum = (0..open.len()).filter(|bit| mask >> bit & 1 == 1).try_fold(Amount::whole(0), |sum, bit| sum.plus(things[open[bit]].1));
                    if sum == Some(need) {
                        groups += 1;
                        found = Some(mask);
                    }
                }
                let Some(mask) = found.filter(|_| groups == 1) else { continue };
                let row = next.entry(question.clone()).or_default();
                for (at, (thing, _)) in things.iter().enumerate() {
                    let left_out = out.contains(&at) || open.iter().position(|place| *place == at).is_some_and(|bit| mask >> bit & 1 == 1);
                    let entry = row.entry(thing.clone()).or_insert((0, 0));
                    if left_out {
                        entry.1 += 1;
                    } else {
                        entry.0 += 1;
                    }
                }
            }
            if next == made.decided {
                break;
            }
            made.decided = next;
        }
        let questions: Vec<&Vec<u32>> = made.decided.keys().collect();
        let mut apart: HashSet<(Vec<u32>, Vec<u32>)> = HashSet::new();
        for (place, one) in questions.iter().enumerate() {
            for other in &questions[place + 1..] {
                let both: Vec<&Vec<u32>> = made.decided[*one].keys().filter(|thing| made.belongs(one, thing).is_some() && made.belongs(other, thing).is_some()).collect();
                if both.len() >= LIST_SHARED && !both.iter().any(|thing| made.belongs(one, thing) == Some(true) && made.belongs(other, thing) == Some(true)) {
                    apart.insert(((*one).clone(), (*other).clone()));
                    apart.insert(((*other).clone(), (*one).clone()));
                }
            }
        }
        made.apart = apart;
        made
    }
}
