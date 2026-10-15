use crate::base::first_sight::FirstSightTable;
use crate::tools::space::read::{FIRST_PLACE, SECOND_PLACE, SPACE_BETWEEN};
use patterns::{because, source};
use std::collections::HashMap;

pub struct FormAnalogy;
source!(FormAnalogy, "the step of a form is guessed by proportional analogy between texts (Lepage): where one change of a word (a word put in, taken out, or put in the place of another) turns a form of known step into the form, the pairs of forms of known steps that the same change joins show, for each axis, which change of the step it brings, and their votes give the guess");

pub struct DirectionWords;
source!(DirectionWords, "where no analogy joins a form to a form of known step, its step is guessed from its signs: a sign is a word alone, or a word with the open places that stand nearest before and after it; the mean change along an axis of the forms that hold a sign is counted, a sign whose mean is near a whole step points along that axis, and the guess along the axis is the sign of the sum of the means of the pointing signs that the form holds");

pub(super) type Offset = [i32; AXES];
because!(Offset, DirectionWords, "a step between two points: the change along each axis");

pub(super) const AXES: usize = 2;
because!(AXES, DirectionWords, "the axes of the space of the things: a page has a left and right and an up and down");

pub(super) const AXIS_VALUES: [i32; 3] = [-1, 0, 1];
because!(AXIS_VALUES, DirectionWords, "the change of one step along an axis: back, none or forward; a told link joins things that stand next to each other or at one point");

const WORD_EDGE: &str = "^";
because!(WORD_EDGE, DirectionWords, "the mark in a sign of a word that has no open place before it or after it");

const POINTING_MEAN: f64 = 0.7;
because!(POINTING_MEAN, DirectionWords, "a sign points along an axis when the mean change of its forms is at least this far from none: most of its forms change the same way, few the other");

const POINTING_LEAST: u32 = 2;
because!(POINTING_LEAST, DirectionWords, "a sign points only when this many forms hold it or more: the mean of one form is that form alone");

const EDIT_PUT: &str = "+";
because!(EDIT_PUT, FormAnalogy, "the mark of a change that puts a word in, in the text that names a change");

const EDIT_OUT: &str = "-";
because!(EDIT_OUT, FormAnalogy, "the mark of a change that takes a word out");

const EDIT_SWAP: &str = ">";
because!(EDIT_SWAP, FormAnalogy, "the mark between the old and the new word of a change that puts one word in the place of another");

fn one_change(from: &[&str], to: &[&str]) -> Option<String> {
    if from.len() == to.len() {
        let mut differ = (0..from.len()).filter(|at| from[*at] != to[*at]);
        let at = differ.next()?;
        return differ.next().is_none().then(|| [from[at], EDIT_SWAP, to[at]].concat());
    }
    let (short, long, mark) = if from.len() + 1 == to.len() { (from, to, EDIT_PUT) } else if to.len() + 1 == from.len() { (to, from, EDIT_OUT) } else { return None };
    let at = (0..short.len()).find(|at| short[*at] != long[*at]).unwrap_or(short.len());
    (short[at..] == long[at + 1..]).then(|| [mark, long[at]].concat())
}
because!(one_change, FormAnalogy, "the one change of a word that turns a form into another, named by its mark and its words; nothing when the forms differ in more than one word");

fn word_signs(form: &str) -> Vec<String> {
    let said: Vec<&str> = form.split(SPACE_BETWEEN).collect();
    let is_place = |word: &str| word == FIRST_PLACE || word == SECOND_PLACE;
    let mut signs: Vec<String> = Vec::new();
    for (at, word) in said.iter().enumerate() {
        if is_place(word) {
            continue;
        }
        let before = said[..at].iter().rev().find(|word| is_place(word)).copied().unwrap_or(WORD_EDGE);
        let after = said[at + 1..].iter().find(|word| is_place(word)).copied().unwrap_or(WORD_EDGE);
        for sign in [word.to_string(), [*word, before, after].join(SPACE_BETWEEN)] {
            if !signs.contains(&sign) {
                signs.push(sign);
            }
        }
    }
    signs
}
because!(word_signs, DirectionWords, "the signs of a form: each word alone and each word with the open places nearest before and after it, each sign once");

type AxisVotes = [FirstSightTable<(i32, i32), u32>; AXES];
because!(AxisVotes, FormAnalogy, "for each axis, how many pairs of forms that a change joins go from one change of the step to another");

pub(super) struct WordGuess {
    known: Vec<(Vec<String>, Offset)>,
    changes: HashMap<String, AxisVotes>,
    sums: HashMap<String, ([i32; AXES], u32)>,
}
because!(WordGuess, FormAnalogy, "the forms of known step with their words, the changes of step that each change of a word brings, and for each sign the sums of the changes of its forms and how many forms hold it");

impl WordGuess {
    pub(super) fn new(known: &[(String, Offset)]) -> WordGuess {
        let mut guess = WordGuess { known: known.iter().map(|(form, step)| (form.split(SPACE_BETWEEN).map(str::to_string).collect(), *step)).collect(), changes: HashMap::new(), sums: HashMap::new() };
        for (from, from_step) in &guess.known {
            let from_words: Vec<&str> = from.iter().map(String::as_str).collect();
            for (to, to_step) in &guess.known {
                let to_words: Vec<&str> = to.iter().map(String::as_str).collect();
                if from_words.len().abs_diff(to_words.len()) > 1 || from_words == to_words {
                    continue;
                }
                let Some(change) = one_change(&from_words, &to_words) else { continue };
                let votes = guess.changes.entry(change).or_insert_with(|| [FirstSightTable::new(), FirstSightTable::new()]);
                for axis in 0..AXES {
                    votes[axis].add((from_step[axis], to_step[axis]), 1);
                }
            }
        }
        for (form, step) in known {
            guess.count_signs(form, *step, true);
        }
        guess
    }

    fn count_signs(&mut self, form: &str, step: Offset, add: bool) {
        for sign in word_signs(form) {
            let (sums, count) = self.sums.entry(sign).or_insert(([0; AXES], 0));
            for axis in 0..AXES {
                sums[axis] += if add { step[axis] } else { -step[axis] };
            }
            *count = if add { *count + 1 } else { count.saturating_sub(1) };
        }
    }

    fn by_analogy(&self, form: &str) -> Option<Offset> {
        let words: Vec<&str> = form.split(SPACE_BETWEEN).collect();
        let mut votes: [FirstSightTable<i32, u32>; AXES] = [FirstSightTable::new(), FirstSightTable::new()];
        for (known, step) in &self.known {
            let known_words: Vec<&str> = known.iter().map(String::as_str).collect();
            if known_words.len().abs_diff(words.len()) > 1 || known_words == words {
                continue;
            }
            let Some(change) = one_change(&known_words, &words) else { continue };
            let Some(pairs) = self.changes.get(&change) else { continue };
            for axis in 0..AXES {
                for ((from, to), count) in pairs[axis].rows() {
                    if *from == step[axis] {
                        votes[axis].add(*to, *count);
                    }
                }
            }
        }
        let across = votes[0].first_most()?.0;
        let along = votes[1].first_most()?.0;
        Some([across, along])
    }

    fn by_signs(&self, form: &str) -> Offset {
        let signs = word_signs(form);
        let mut step: Offset = [0; AXES];
        for (axis, value) in step.iter_mut().enumerate() {
            let mut sum = f64::default();
            for sign in &signs {
                let Some((sums, count)) = self.sums.get(sign) else { continue };
                if *count < POINTING_LEAST {
                    continue;
                }
                let mean = f64::from(sums[axis]) / f64::from(*count);
                if mean.abs() >= POINTING_MEAN {
                    sum += mean;
                }
            }
            *value = i32::from(sum > f64::default()) - i32::from(sum < f64::default());
        }
        step
    }

    pub(super) fn guess(&self, form: &str) -> Offset {
        self.by_analogy(form).unwrap_or_else(|| self.by_signs(form))
    }

    pub(super) fn signs_left_out_right(&mut self, known: &[(String, Offset)]) -> usize {
        let mut right = 0usize;
        for (form, step) in known {
            self.count_signs(form, *step, false);
            if self.by_signs(form) == *step {
                right += 1;
            }
            self.count_signs(form, *step, true);
        }
        right
    }
}

pub(super) fn guess_agrees(guess: Offset, step: Offset) -> usize {
    (0..AXES).filter(|axis| guess[*axis] == step[*axis]).count()
}
because!(guess_agrees, FormAnalogy, "how many axes of a step agree with the guess from the words of its form");
