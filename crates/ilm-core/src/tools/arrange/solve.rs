use patterns::{because, source};

pub struct OneOrder;
source!(OneOrder, "the things of a story stand in one order, and each statement form means a test of their places: a thing stands at a given place counted from the start or from the end, or one thing stands before or after another. The arrangements that pass every test are found by backtracking, the search of Golomb and Baumert: the things are placed one after another, and each test is made as soon as its things are placed");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Meaning {
    At(usize),
    FromEnd(usize),
    Before,
    After,
}
because!(Meaning, OneOrder, "what a statement form means for the places of its things: the first thing at a place counted from the start, or from the end, or the first thing before or after the second");

pub(super) fn holds(meaning: Meaning, places: &[usize], slots: &[usize], things: usize) -> bool {
    match meaning {
        Meaning::At(place) => places[slots[0]] == place,
        Meaning::FromEnd(place) => places[slots[0]] + place + 1 == things,
        Meaning::Before => places[slots[0]] < places[slots[1]],
        Meaning::After => places[slots[0]] > places[slots[1]],
    }
}
because!(holds, OneOrder, "whether a statement holds in an arrangement, given as the place of each thing");

#[derive(Clone, Debug)]
pub(super) struct Test {
    pub(super) meaning: Meaning,
    pub(super) slots: Vec<usize>,
    pub(super) wanted: bool,
}
because!(Test, OneOrder, "one test of an arrangement: a meaning, the things that it is about, and whether the statement must hold or must fail");

struct PlaceSearch<'a> {
    things: usize,
    tests: &'a [Test],
    by_last: Vec<Vec<usize>>,
    places: Vec<usize>,
    used: Vec<bool>,
    found: usize,
    most: usize,
}
because!(PlaceSearch, OneOrder, "the state of one search for arrangements: the tests grouped by the last thing that they name, the places given so far, the places taken, how many arrangements were found, and how many are wanted");

impl PlaceSearch<'_> {
    fn place(&mut self, thing: usize, seen: &mut dyn FnMut(&[usize])) {
        if self.found >= self.most {
            return;
        }
        if thing == self.things {
            self.found += 1;
            seen(&self.places);
            return;
        }
        for place in 0..self.things {
            if self.used[place] {
                continue;
            }
            self.places[thing] = place;
            let passes = self.by_last[thing].iter().all(|test| {
                let test = &self.tests[*test];
                holds(test.meaning, &self.places, &test.slots, self.things) == test.wanted
            });
            if passes {
                self.used[place] = true;
                self.place(thing + 1, seen);
                self.used[place] = false;
            }
            if self.found >= self.most {
                return;
            }
        }
    }
}

pub(super) fn arrangements(things: usize, tests: &[Test], most: usize, seen: &mut dyn FnMut(&[usize])) -> usize {
    let mut by_last: Vec<Vec<usize>> = vec![Vec::new(); things];
    for (number, test) in tests.iter().enumerate() {
        if let Some(last) = test.slots.iter().copied().max().filter(|last| *last < things) {
            by_last[last].push(number);
        }
    }
    let mut search = PlaceSearch { things, tests, by_last, places: vec![0; things], used: vec![false; things], found: 0, most };
    search.place(0, seen);
    search.found
}
because!(arrangements, OneOrder, "the arrangements of the things that pass every test, each given to seen as the place of each thing, up to the most that are wanted; the count of those found");
