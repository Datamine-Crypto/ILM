use crate::tools::parts::counted::{FORCED_LESSONS, sure_of};
use crate::tools::parts::language::{FIRST_OF, LAST_OF, NO_OPERATION, ONLY_PLAIN, WordRow, VIEWS, AnswerView, Word};
use crate::tools::parts::ordered::Kept;
use crate::tools::parts::places::{Lexicon, LookTable, Out, Place, laid_out, simple, simple_rank};
use patterns::{because, source};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct RunsSideBySide;
source!(RunsSideBySide, "the program of a form found from its runs by laying them side by side, as in the anti-unification of Plotkin: a place of the program holds the answer of the same part in every run (plain, or under the same operations), or the same word in every run; the programs are counted on all runs, and the simplest of those that give the most runs is kept");

pub const OUTS_MOST: usize = 6;
because!(OUTS_MOST, RunsSideBySide, "the programs that two runs side by side give at most with plain answers: two runs agree on few ways to place their parts");

pub const OUTS_VIEWS: usize = 24;
because!(OUTS_VIEWS, RunsSideBySide, "the programs that runs side by side give at most when the answer of a part can stand under an operation: each operation is one more way to place a part");

const OUTS_LOOK: usize = 48;
because!(OUTS_LOOK, RunsSideBySide, "the programs that runs side by side give at most when one place is a looked-up word: twice as many as with operations alone, since the looked-up word can stand at any place");

const MORE_THAN_HALF: usize = 2;
because!(MORE_THAN_HALF, RunsSideBySide, "the best program of a text of a form must give more than half of its runs: its runs times this factor must pass all runs");

const PAIRS_MOST: usize = 4;
because!(PAIRS_MOST, RunsSideBySide, "the runs of a form that are laid side by side with others, and the others for each of them, those that differ in the most parts first");

const GROUP: usize = 3;
because!(GROUP, RunsSideBySide, "the runs that are laid side by side at once with operations: a pair of runs agrees on a program by chance far more often than a group with one run more");

const GROUPS_MOST: usize = 6;
because!(GROUPS_MOST, RunsSideBySide, "the groups of runs that are tried at once with operations");

const SPACE_RUNS: usize = 12;
because!(SPACE_RUNS, RunsSideBySide, "the runs of a text of a form that are laid side by side at once when every program that gives as many runs as the best one is kept: so many runs leave few programs");

const SPACE_MOST: usize = 400;
because!(SPACE_MOST, RunsSideBySide, "the programs that are kept for one place of such a search: enough for every way that so many runs agree on");

const LOOK_SHARE: f64 = 0.8;
because!(LOOK_SHARE, RunsSideBySide, "a program with a looked-up word is kept when its table gives this share of the runs that do not stand against it: a table counted from few runs leaves some keys open");

pub const SEARCHED_MOST: u64 = 200000;
because!(SEARCHED_MOST, RunsSideBySide, "the places of answers that the search for one program tries at most; then it gives what it has, so a form with many long runs cannot hold the learning");

const ALL: f64 = 1.0;
because!(ALL, RunsSideBySide, "the share of all runs, from which the share of the runs that stand against a table is taken");

pub const LOOK_VIEWS: &[AnswerView] = &[AnswerView { first: FIRST_OF, then: NO_OPERATION }, AnswerView { first: LAST_OF, then: NO_OPERATION }];
because!(LOOK_VIEWS, RunsSideBySide, "the word of a part that is the key of a looked-up word: its first word or its last word");

pub type FoundRun = (Vec<WordRow>, WordRow);
because!(FoundRun, RunsSideBySide, "one run of a form: the answers of its parts and its answer");

fn starts_at(target: &[Word], at: usize, piece: &[Word]) -> bool {
    at + piece.len() <= target.len() && target[at..at + piece.len()] == *piece
}
because!(starts_at, RunsSideBySide, "whether a run of words stands in a row at a place");

fn prefixed(first: Place, rest: &[Out]) -> impl Iterator<Item = Out> + '_ {
    rest.iter().map(move |more| {
        let mut out = Vec::with_capacity(more.len() + 1);
        out.push(first.clone());
        out.extend(more.iter().cloned());
        out
    })
}
because!(prefixed, RunsSideBySide, "the programs of the rest of the runs with one place before each of them");

fn piece_place(part: usize, view: AnswerView) -> Place {
    if view.plain() { Place::Part(part) } else { Place::Seen(part, view) }
}
because!(piece_place, RunsSideBySide, "the place of the answer of a part: the part alone when its answer is plain, else the part with its view");

fn viewed(row: &[Word], view: AnswerView) -> WordRow {
    if view.plain() { row.to_vec() } else { view.on(row) }
}
because!(viewed, RunsSideBySide, "the answer of a part under a view");

struct Pair<'a> {
    rows: (&'a [WordRow], &'a [WordRow]),
    targets: (&'a [Word], &'a [Word]),
    banned: &'a HashSet<Word>,
    views: &'a [AnswerView],
    look: bool,
    most: usize,
    memo: HashMap<(usize, usize, bool), Vec<Out>>,
    searched: &'a Cell<u64>,
}
because!(Pair, RunsSideBySide, "two runs laid side by side: their answers of parts and their answers, the words that are no fixed word of the form, the views, whether one place is a looked-up word, the most programs that a place keeps, and what was found from each pair of places");

impl Pair<'_> {
    fn rest(&mut self, one: usize, other: usize, looked: bool) -> Vec<Out> {
        if let Some(found) = self.memo.get(&(one, other, looked)) {
            return found.clone();
        }
        let mut found: Vec<Out> = Vec::new();
        self.searched.set(self.searched.get() + 1);
        if self.searched.get() > SEARCHED_MOST && self.views.len() > 1 {
            return found;
        }
        let (first, second) = self.targets;
        if one == first.len() && other == second.len() {
            if looked || !self.look {
                found.push(Vec::new());
            }
        } else {
            if self.look && !looked && one < first.len() && other < second.len() {
                for part in 0..self.rows.0.len() {
                    for view in LOOK_VIEWS {
                        let (key1, key2) = (view.on(&self.rows.0[part]), view.on(&self.rows.1[part]));
                        if key1.len() == 1 && key2.len() == 1 && (key1 != key2 || first[one] == second[other]) {
                            let more = self.rest(one + 1, other + 1, true);
                            found.extend(prefixed(Place::Looked(part, *view), &more));
                        }
                    }
                }
            }
            for part in 0..self.rows.0.len() {
                for view in self.views {
                    let (piece1, piece2) = (viewed(&self.rows.0[part], *view), viewed(&self.rows.1[part], *view));
                    if !piece1.is_empty() && !piece2.is_empty() && starts_at(first, one, &piece1) && starts_at(second, other, &piece2) {
                        let more = self.rest(one + piece1.len(), other + piece2.len(), looked);
                        found.extend(prefixed(piece_place(part, *view), &more));
                    }
                }
            }
            if one < first.len() && other < second.len() && first[one] == second[other] && !self.banned.contains(&first[one]) {
                let more = self.rest(one + 1, other + 1, looked);
                found.extend(prefixed(Place::Word(first[one]), &more));
            }
        }
        found.truncate(self.most);
        self.memo.insert((one, other, looked), found.clone());
        found
    }
}

pub fn side_by_side(one: &FoundRun, other: &FoundRun, banned: &HashSet<Word>, views: &[AnswerView], look: bool, searched: &Cell<u64>) -> Vec<Out> {
    let most = if views.len() == 1 { OUTS_MOST } else if look { OUTS_LOOK } else { OUTS_VIEWS };
    let mut pair = Pair { rows: (&one.0, &other.0), targets: (&one.1, &other.1), banned, views, look, most, memo: HashMap::new(), searched };
    pair.rest(0, 0, false)
}
because!(side_by_side, RunsSideBySide, "the programs of a form that give the answers of two runs, each from the answers of its parts; with look every program has one place with a looked-up word, the same word of the part giving the same word of the answer");

struct RunGroup<'a> {
    runs: &'a [&'a FoundRun],
    banned: &'a HashSet<Word>,
    views: &'a [AnswerView],
    look: bool,
    most: usize,
    parts: usize,
    memo: HashMap<(Vec<usize>, bool), Vec<Out>>,
    searched: &'a Cell<u64>,
}
because!(RunGroup, RunsSideBySide, "several runs laid side by side at once, with the same things as two runs");

impl RunGroup<'_> {
    fn rest(&mut self, at: Vec<usize>, looked: bool) -> Vec<Out> {
        if let Some(found) = self.memo.get(&(at.clone(), looked)) {
            return found.clone();
        }
        let mut found: Vec<Out> = Vec::new();
        self.searched.set(self.searched.get() + 1);
        if self.searched.get() > SEARCHED_MOST {
            return found;
        }
        let runs = self.runs;
        if runs.iter().zip(&at).all(|(run, place)| *place == run.1.len()) {
            if looked || !self.look {
                found.push(Vec::new());
            }
        } else {
            let inside = runs.iter().zip(&at).all(|(run, place)| *place < run.1.len());
            let next: Vec<usize> = at.iter().map(|place| place + 1).collect();
            if self.look && !looked && inside {
                let words: Vec<Word> = runs.iter().zip(&at).map(|(run, place)| run.1[*place]).collect();
                for part in 0..self.parts {
                    for view in LOOK_VIEWS {
                        let keys: Vec<WordRow> = runs.iter().map(|run| view.on(&run.0[part])).collect();
                        if keys.iter().all(|key| key.len() == 1) {
                            let pairs: HashSet<(&WordRow, Word)> = keys.iter().zip(&words).map(|(key, word)| (key, *word)).collect();
                            let distinct: HashSet<&WordRow> = keys.iter().collect();
                            if pairs.len() == distinct.len() {
                                let more = self.rest(next.clone(), true);
                                found.extend(prefixed(Place::Looked(part, *view), &more));
                            }
                        }
                    }
                }
            }
            for part in 0..self.parts {
                for view in self.views {
                    let pieces: Vec<WordRow> = runs.iter().map(|run| viewed(&run.0[part], *view)).collect();
                    if pieces.iter().any(|piece| !piece.is_empty()) && pieces.iter().zip(&at).zip(runs).all(|((piece, place), run)| starts_at(&run.1, *place, piece)) {
                        let after: Vec<usize> = pieces.iter().zip(&at).map(|(piece, place)| place + piece.len()).collect();
                        let more = self.rest(after, looked);
                        found.extend(prefixed(piece_place(part, *view), &more));
                    }
                    if found.len() >= self.most {
                        break;
                    }
                }
            }
            if inside {
                let word = runs[0].1[at[0]];
                if runs.iter().zip(&at).all(|(run, place)| run.1[*place] == word) && !self.banned.contains(&word) {
                    let more = self.rest(next, looked);
                    found.extend(prefixed(Place::Word(word), &more));
                }
            }
        }
        found.truncate(self.most);
        self.memo.insert((at, looked), found.clone());
        found
    }
}

pub fn together(runs: &[&FoundRun], banned: &HashSet<Word>, views: &[AnswerView], look: bool, most: Option<usize>, searched: &Cell<u64>) -> Vec<Out> {
    let most = most.unwrap_or(if look { OUTS_LOOK } else { OUTS_VIEWS });
    let parts = runs.first().map_or(0, |run| run.0.len());
    let mut group = RunGroup { runs, banned, views, look, most, parts, memo: HashMap::new(), searched };
    group.rest(vec![0; runs.len()], false)
}
because!(together, RunsSideBySide, "the programs that give the answers of all these runs, each from the answers of its parts: a place holds the answer of the same part in all runs, plain or under the same operations, which may be empty in some runs and not in all, or the same word in all; with look every program has one place with a looked-up word whose key gives one word in all runs");

fn informative(item: &FoundRun) -> (u8, usize) {
    let lengths: Vec<usize> = item.0.iter().map(Vec::len).collect();
    let least = lengths.iter().copied().min().unwrap_or_default();
    let largest = lengths.iter().copied().max().unwrap_or_default();
    let sort = if least >= OUTS_ANSWER_WORDS { 0 } else if largest >= OUTS_ANSWER_WORDS { 1 } else { SORTS_OF_RUNS };
    (sort, lengths.iter().sum())
}
because!(informative, RunsSideBySide, "the order of the runs of a form for a search with operations: the runs whose parts all have answers of two words or more, then those with one such part, then the others; in each the shortest first, because a shallow text has its parts read right while a form of it is still wrong");

const OUTS_ANSWER_WORDS: usize = 2;
because!(OUTS_ANSWER_WORDS, RunsSideBySide, "the answer words of a part from which an operation on it can be seen: a turn or a cut of one word changes nothing that the lessons show");

const TABLE_WORDS: usize = 2;
because!(TABLE_WORDS, RunsSideBySide, "the different words that a table must give at least: a table that gives one word for every key is a fixed word of the program");

const SORTS_OF_RUNS: u8 = 2;
because!(SORTS_OF_RUNS, RunsSideBySide, "the number of the last sort of runs in the order of a search with operations: runs whose parts all have short answers, after the sorts with long answers");

pub fn with_table(out: &[Place], items: &[&FoundRun], full: bool, lexicon: &Lexicon) -> Option<(Out, Vec<usize>, usize)> {
    let place = out.iter().position(|place| matches!(place, Place::Looked(..) | Place::Look(..)))?;
    let (part, view) = match &out[place] {
        Place::Looked(part, view) | Place::Look(part, view, _) => (*part, *view),
        _ => return None,
    };
    let mut shown: Vec<Option<(Word, Word)>> = Vec::with_capacity(items.len());
    let mut counts: Kept<Word, Kept<Word, usize>> = Kept::default();
    for item in items {
        let before = laid_out(&out[..place], &item.0)?;
        let after = laid_out(&out[place + 1..], &item.0)?;
        let key = view.on(&item.0[part]);
        let target = &item.1;
        if key.len() != 1 || target.len() != before.len() + 1 + after.len() || target[..before.len()] != *before || target[before.len() + 1..] != *after {
            shown.push(None);
        } else {
            let word = target[before.len()];
            shown.push(Some((key[0], word)));
            counts.or_put(key[0], Kept::default).count(word, 1);
        }
    }
    let mut table: Vec<(Word, Word)> = Vec::new();
    for (key, found) in counts.pairs() {
        if let Some((word, count)) = found.most() {
            if found.total() >= FORCED_LESSONS && sure_of(count, found.total()) {
                table.push((*key, *word));
            }
        }
    }
    let values: HashSet<Word> = table.iter().map(|(_, word)| *word).collect();
    if values.len() < TABLE_WORDS || table.iter().all(|(key, word)| key == word) {
        return None;
    }
    table.sort_by(|one, other| lexicon.cmp_word(one.0, other.0).then_with(|| lexicon.cmp_word(one.1, other.1)));
    let held: HashMap<Word, Word> = table.iter().copied().collect();
    let mut program: Out = out[..place].to_vec();
    program.push(Place::Look(part, view, Arc::new(LookTable::of(table))));
    program.extend(out[place + 1..].iter().cloned());
    let explained: Vec<usize> = shown.iter().enumerate().filter(|(_, pair)| pair.is_some_and(|(key, word)| held.get(&key) == Some(&word))).map(|(number, _)| number).collect();
    let against = if full { shown.iter().filter(|pair| pair.is_none_or(|(key, word)| held.get(&key).is_some_and(|held| *held != word))).count() } else { 0 };
    Some((program, explained, against))
}
because!(with_table, RunsSideBySide, "a program with a looked-up word, with the table of that place counted from the runs: a key has a word when enough runs show it and nearly all of them agree; nothing when the table gives one word for every key, or every key for itself, since that is a fixed word or the part; with the program come the runs that it gives and, when asked, the runs that stand against it");

pub type Scored = Kept<Out, Vec<usize>>;
because!(Scored, RunsSideBySide, "the programs that the runs were laid into, in the order in which they were found, each with the numbers of the runs that it gives");

pub fn candidates_of(items: &[FoundRun], banned: &HashSet<Word>, views: &[AnswerView], look: bool, searched: &Cell<u64>, lexicon: &Lexicon) -> Scored {
    let mut candidates: Kept<Out, ()> = Kept::default();
    let mut order: Vec<usize> = (0..items.len()).collect();
    if views.len() > 1 {
        order.sort_by_key(|number| (informative(&items[*number]), *number));
        searched.set(0);
        let mut start = 0;
        while start < order.len().min(GROUP * GROUPS_MOST) {
            let group: Vec<&FoundRun> = order[start..(start + GROUP).min(order.len())].iter().map(|number| &items[*number]).collect();
            let sizes: HashSet<usize> = group.iter().map(|item| item.0.len()).collect();
            if group.len() >= FORCED_LESSONS && sizes.len() == 1 {
                for out in together(&group, banned, views, look, None, searched) {
                    candidates.put(out, ());
                }
            }
            start += GROUP;
        }
    }
    for first in order.iter().take(PAIRS_MOST) {
        let rows = &items[*first].0;
        let mut others: Vec<(usize, usize)> = order.iter().enumerate().filter(|(_, number)| *number != first).map(|(place, number)| (place, *number)).collect();
        others.sort_by_key(|(place, number)| (std::cmp::Reverse(rows.iter().zip(&items[*number].0).filter(|(one, other)| one != other).count()), *place));
        for (_, second) in others.into_iter().take(PAIRS_MOST) {
            for out in side_by_side(&items[*first], &items[second], banned, views, look, searched) {
                if !candidates.holds(&out) {
                    candidates.put(out, ());
                }
            }
        }
    }
    let all: Vec<&FoundRun> = items.iter().collect();
    let mut scored: Scored = Kept::default();
    for out in candidates.into_pairs().map(|(out, _)| out) {
        if look {
            let Some((program, explained, _)) = with_table(&out, &all, false, lexicon) else { continue };
            scored.put(program, explained);
        } else {
            let explained: Vec<usize> = items.iter().enumerate().filter(|(_, item)| laid_out(&out, &item.0).as_ref() == Some(&item.1)).map(|(number, _)| number).collect();
            scored.put(out, explained);
        }
    }
    scored
}
because!(candidates_of, RunsSideBySide, "the programs of a form found from its runs laid side by side: with operations first groups of runs, those whose parts have answers of two words or more and the shortest of them first; then pairs of runs, the first runs each with the runs that differ from it in the most parts; each program with the runs that it gives");

pub fn template(items: &[FoundRun], banned: &HashSet<Word>, views: &[AnswerView], look: bool, searched: &Cell<u64>, lexicon: &Lexicon) -> (Option<Out>, Vec<usize>) {
    let mut best: Option<(Out, Vec<usize>, (usize, (bool, isize, isize, isize)))> = None;
    for (out, explained) in candidates_of(items, banned, views, look, searched, lexicon).into_pairs() {
        let rank = (explained.len(), simple_rank(&out));
        if best.as_ref().is_none_or(|(_, _, top)| rank > *top) {
            best = Some((out, explained, rank));
        }
    }
    match best {
        Some((out, explained, _)) => (Some(out), explained),
        None => (None, Vec::new()),
    }
}
because!(template, RunsSideBySide, "the program of a form that gives the answer of the most runs, with the numbers of these runs; of two that give as many runs the simpler one, and of two as simple the one found first");

pub fn every_program(items: &[FoundRun], banned: &HashSet<Word>, searched: &Cell<u64>, lexicon: &Lexicon) -> Vec<Out> {
    if items.len() < FORCED_LESSONS {
        return Vec::new();
    }
    let mut sorted: Vec<&FoundRun> = items.iter().collect();
    sorted.sort_by_key(|item| informative(item));
    sorted.truncate(SPACE_RUNS);
    searched.set(0);
    let mut scored: Kept<Out, usize> = Kept::default();
    for out in together(&sorted, banned, VIEWS, false, Some(SPACE_MOST), searched) {
        let count = items.iter().filter(|item| laid_out(&out, &item.0).as_ref() == Some(&item.1)).count();
        scored.put(out, count);
    }
    let weak = |scored: &Kept<Out, usize>| scored.values().max().is_none_or(|most| most * MORE_THAN_HALF <= items.len());
    if weak(&scored) {
        for views in [ONLY_PLAIN, VIEWS] {
            for (out, explained) in candidates_of(items, banned, views, false, searched, lexicon).into_pairs() {
                scored.put(out, explained.len());
            }
        }
    }
    if weak(&scored) {
        scored = Kept::default();
        searched.set(0);
        let all: Vec<&FoundRun> = items.iter().collect();
        for out in together(&sorted, banned, VIEWS, true, Some(SPACE_MOST), searched) {
            if !out.iter().any(|place| matches!(place, Place::Looked(..))) {
                continue;
            }
            if let Some((program, explained, against)) = with_table(&out, &all, true, lexicon) {
                if (explained.len() as f64) >= LOOK_SHARE * ((items.len() - against) as f64) {
                    scored.put(program, explained.len());
                }
            }
        }
    }
    let Some(most) = scored.values().max().copied() else { return Vec::new() };
    if most * MORE_THAN_HALF <= items.len() || most < FORCED_LESSONS {
        return Vec::new();
    }
    let mut kept: Vec<(Out, crate::tools::parts::places::Simple)> = scored.pairs().filter(|(_, count)| sure_of(**count, most)).map(|(out, _)| (out.clone(), simple(out, lexicon))).collect();
    kept.sort_by(|one, other| other.1.cmp(&one.1));
    kept.into_iter().map(|(out, _)| out).collect()
}
because!(every_program, RunsSideBySide, "every program that gives as many runs of a text of a form as the best one, nearly all of its number, the simplest first; the best one gives more than half of the runs. The runs are laid side by side with no operation and with operations; when no such program gives half of the runs, with one looked-up word whose table gives most of the runs that do not stand against it");

pub fn whole_share(count: usize, all: usize) -> bool {
    (count as f64) > (ALL - crate::tools::parts::counted::COUNT_SURE_SHARE) * (all as f64)
}
because!(whole_share, RunsSideBySide, "whether a count is more than the share of all that the sure share leaves out: some lessons, more than a set may hold by error");
