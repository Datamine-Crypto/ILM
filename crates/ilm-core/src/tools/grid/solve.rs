use patterns::{because, source};

pub struct PlacedKinds;
source!(PlacedKinds, "each kind of things fills the places once, one thing in each place, and each statement form means a test of the places of its things: a thing at a place or not at it, or two things in one place, in different places, one before or after the other, one right before or right after the other, next to each other, or a given number of places apart. The arrangements that pass every test are found by backtracking (Golomb and Baumert), one thing after another, each test made as soon as its things have places");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlaceMeaning {
    At(usize),
    NotAt(usize),
    Same,
    Apart,
    Before,
    After,
    RightBefore,
    RightAfter,
    Next,
    Gap(usize),
}
because!(PlaceMeaning, PlacedKinds, "what a statement form means for the places of its things: for one thing, at a place or not at it; for two things, one place, different places, the first before or after the second, right before or right after it, next to it, or a given number of places apart");

fn place_holds(meaning: PlaceMeaning, first: usize, second: usize) -> bool {
    match meaning {
        PlaceMeaning::At(place) => first == place,
        PlaceMeaning::NotAt(place) => first != place,
        PlaceMeaning::Same => first == second,
        PlaceMeaning::Apart => first != second,
        PlaceMeaning::Before => first < second,
        PlaceMeaning::After => first > second,
        PlaceMeaning::RightBefore => first + 1 == second,
        PlaceMeaning::RightAfter => first == second + 1,
        PlaceMeaning::Next => first.abs_diff(second) == 1,
        PlaceMeaning::Gap(gap) => first.abs_diff(second) == gap,
    }
}
because!(place_holds, PlacedKinds, "whether a meaning holds for the place of its first thing and the place of its second thing, which a meaning about one thing does not look at");

pub(super) fn single_meanings(places: usize) -> Vec<PlaceMeaning> {
    (0..places).map(PlaceMeaning::At).chain((0..places).map(PlaceMeaning::NotAt)).collect()
}
because!(single_meanings, PlacedKinds, "every meaning of a form about one thing for a number of places");

pub(super) fn pair_meanings(places: usize) -> Vec<PlaceMeaning> {
    let mut out = vec![PlaceMeaning::Same, PlaceMeaning::Apart, PlaceMeaning::Before, PlaceMeaning::After, PlaceMeaning::RightBefore, PlaceMeaning::RightAfter, PlaceMeaning::Next];
    out.extend((PLACE_GAP_LEAST..places).map(PlaceMeaning::Gap));
    out
}
because!(pair_meanings, PlacedKinds, "every meaning of a form about two things for a number of places");

const PLACE_GAP_LEAST: usize = 2;
because!(PLACE_GAP_LEAST, PlacedKinds, "the smallest gap of places that is its own meaning: a gap of one is next to each other");

#[derive(Clone, Debug)]
pub(super) struct PlaceTest {
    pub(super) things: Vec<usize>,
    fits: Vec<u32>,
}
because!(PlaceTest, PlacedKinds, "one test of an arrangement: the things that it is about and where it passes, as bits: for one thing, one number with a bit for each place where the test passes; for two things, for each place of the first thing, a number with a bit for each place of the second thing where the test passes");

impl PlaceTest {
    pub(super) fn new(meaning: PlaceMeaning, things: &[usize], wanted: bool, places: usize) -> PlaceTest {
        let bits = |kept: &mut dyn Iterator<Item = usize>| kept.fold(0u32, |bits, place| bits | 1 << place);
        let fits = match things.len() {
            1 => vec![bits(&mut (0..places).filter(|first| place_holds(meaning, *first, *first) == wanted))],
            _ => (0..places).map(|first| bits(&mut (0..places).filter(|second| place_holds(meaning, first, *second) == wanted))).collect(),
        };
        PlaceTest { things: things.to_vec(), fits }
    }

    fn within_kinds(&self, places: usize) -> PlaceTest {
        let mut kept = self.clone();
        if let [first, second] = self.things.as_slice()
            && first / places == second / places
        {
            kept.fits.iter_mut().enumerate().for_each(|(place, bits)| *bits &= !(1 << place));
        }
        kept
    }

    pub(super) fn passes(&self, at: &[usize]) -> bool {
        match self.things.as_slice() {
            [one] => self.fits[0] & 1 << at[*one] != 0,
            [first, second] => self.fits[at[*first]] & 1 << at[*second] != 0,
            _ => true,
        }
    }
}

pub struct DomainPropagation;
source!(DomainPropagation, "the arc consistency of Mackworth over the places that each thing may still take: a test about one thing keeps the places where it holds, and a test about two things keeps the places of each that some place of the other supports; in each kind, a set of places that as many things of the kind must take as it has places is closed to the other things of the kind, and a set that more must take has no arrangement, by the marriage theorem of Hall; the search then gives a place to the thing with the fewest places left for the tests that name it, as Bessiere and Regin advise, so that the things of no test, which the kinds alone place, come last");

type Domains = Vec<u32>;
because!(Domains, DomainPropagation, "the places that each thing may still take, as the bits of one number for each thing");

fn propagate(domains: &mut Domains, places: usize, kinds: usize, tests: &[PlaceTest]) -> bool {
    let every = (1u32 << places) - 1;
    loop {
        let mut changed = false;
        for test in tests {
            match test.things.as_slice() {
                [one] => {
                    let kept = domains[*one] & test.fits[0];
                    if kept != domains[*one] {
                        domains[*one] = kept;
                        changed = true;
                    }
                }
                [first, second] => {
                    let mut kept = 0u32;
                    let mut reach = 0u32;
                    for place in 0..places {
                        if domains[*first] & 1 << place != 0 && test.fits[place] & domains[*second] != 0 {
                            kept |= 1 << place;
                            reach |= test.fits[place];
                        }
                    }
                    let partners = domains[*second] & reach;
                    if kept != domains[*first] || partners != domains[*second] {
                        domains[*first] = kept;
                        domains[*second] = partners;
                        changed = true;
                    }
                }
                _ => {}
            }
        }
        for kind in 0..kinds {
            let members = &mut domains[kind * places..(kind + 1) * places];
            for set in 1..every {
                let mut inside = 0u32;
                for (item, domain) in members.iter().enumerate() {
                    if *domain & !set == 0 {
                        inside |= 1 << item;
                    }
                }
                let size = set.count_ones();
                if inside.count_ones() > size {
                    return false;
                }
                if inside.count_ones() < size {
                    continue;
                }
                for (item, domain) in members.iter_mut().enumerate() {
                    if inside & 1 << item == 0 && *domain & set != 0 {
                        *domain &= !set;
                        changed = true;
                    }
                }
            }
        }
        if domains.contains(&0) {
            return false;
        }
        if !changed {
            return true;
        }
    }
}
because!(propagate, DomainPropagation, "the places that each thing may still take after every test and every kind is applied until nothing changes; false when a thing is left with no place or a set of places must hold more things of a kind than it has places");

struct DomainSearch<'a> {
    places: usize,
    kinds: usize,
    tests: &'a [PlaceTest],
    named: Vec<usize>,
    found: usize,
    most: usize,
}
because!(DomainSearch, DomainPropagation, "the state of one search for arrangements: the size of the story, the tests, how many tests name each thing, how many arrangements were found, and how many are wanted");

impl DomainSearch<'_> {
    fn search(&mut self, mut domains: Domains, seen: &mut dyn FnMut(&[usize])) {
        if self.found >= self.most || !propagate(&mut domains, self.places, self.kinds, self.tests) {
            return;
        }
        let mut open: Option<usize> = None;
        for thing in (0..domains.len()).filter(|thing| domains[*thing].count_ones() > 1) {
            let left = domains[thing].count_ones() as usize;
            if open.is_none_or(|best| left * self.named[best] < domains[best].count_ones() as usize * self.named[thing]) {
                open = Some(thing);
            }
        }
        let Some(thing) = open else {
            self.found += 1;
            let at: Vec<usize> = domains.iter().map(|domain| domain.trailing_zeros() as usize).collect();
            seen(&at);
            return;
        };
        for place in 0..self.places {
            if domains[thing] & (1 << place) == 0 {
                continue;
            }
            let mut tried = domains.clone();
            tried[thing] = 1 << place;
            self.search(tried, seen);
            if self.found >= self.most {
                return;
            }
        }
    }
}

pub(super) fn kind_arrangements(places: usize, kinds: usize, tests: &[PlaceTest], most: usize, seen: &mut dyn FnMut(&[usize])) -> usize {
    let every = (1u32 << places) - 1;
    let mut named = vec![0usize; places * kinds];
    tests.iter().flat_map(|test| test.things.iter()).for_each(|thing| named[*thing] += 1);
    let kept: Vec<PlaceTest> = tests.iter().map(|test| test.within_kinds(places)).collect();
    let mut search = DomainSearch { places, kinds, tests: &kept, named, found: 0, most };
    search.search(vec![every; places * kinds], seen);
    search.found
}
because!(kind_arrangements, DomainPropagation, "the arrangements of the things of every kind that pass every test, each given to seen as the place of each thing, up to the most that are wanted; the count of those found; a test about two things of one kind passes only where they stand apart, since a kind has one thing in each place");
