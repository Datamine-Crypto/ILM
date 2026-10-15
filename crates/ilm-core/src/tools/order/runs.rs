use crate::know::signs::is_space;
use patterns::{because, source};

pub struct RunsOfItems;
source!(RunsOfItems, "a text as items, and where the items of an answer stand in it. An item is a part of the text between blanks, in small letters, with the marks of a sentence taken off its ends, so a word with a mark inside it is one item. A run is a part of one line of the text, a told sentence or the question, that holds the items of the answer in some order; two runs of one line can hold the same items in two orders");

const ORDER_MARKS: &str = ".,?!:;\"";
because!(ORDER_MARKS, RunsOfItems, "the marks that end a sentence or part it, and the quote: they stand at the ends of a word and are no part of it");

const ORDER_MOST_ITEMS: usize = 60;
because!(ORDER_MOST_ITEMS, RunsOfItems, "a longer line is not searched for two runs of the same items: the search grows with the fourth power of the items of a line");

const ORDER_LEAST_RUN: usize = 2;
because!(ORDER_LEAST_RUN, RunsOfItems, "the shortest run whose items can stand in more than one order; runs of one line that do not overlap are at most the length of the line over this number");

pub(super) const ORDER_RUNS: usize = 2;
because!(ORDER_RUNS, RunsOfItems, "the runs that a choice is made between: one with the items in the order of the answer, the other with the same items in another order");

pub(super) fn order_items<N: FnMut(&str) -> u32>(text: &str, name: &mut N) -> Vec<u32> {
    let low = text.to_lowercase();
    low.split(is_space).map(|part| part.trim_matches(|mark: char| ORDER_MARKS.contains(mark))).filter(|item| !item.is_empty()).map(name).collect()
}
because!(order_items, RunsOfItems, "the items of a text in their order, each as the number that the given table has for it");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct OrderPlace {
    pub(super) back: usize,
    pub(super) start: usize,
    pub(super) rest: usize,
}
because!(OrderPlace, RunsOfItems, "where a run stands: its line counted from the end of the text, so the question is the first line; its first item; and how many items of the line stand after it");

pub(super) fn order_run_place(lines: &[Vec<u32>], said: &[u32]) -> Option<OrderPlace> {
    let mut want = said.to_vec();
    want.sort_unstable();
    let mut window: Vec<u32> = Vec::with_capacity(said.len());
    for (back, line) in lines.iter().rev().enumerate() {
        if line.len() < said.len() {
            continue;
        }
        for start in 0..=line.len() - said.len() {
            window.clear();
            window.extend_from_slice(&line[start..start + said.len()]);
            window.sort_unstable();
            if window == want {
                return Some(OrderPlace { back, start, rest: line.len() - start - said.len() });
            }
        }
    }
    None
}
because!(order_run_place, RunsOfItems, "the first place where the items of an answer stand as one run: the lines from the end, the runs of a line from its start");

pub(super) fn order_run_at(lines: &[Vec<u32>], place: OrderPlace) -> Option<&[u32]> {
    let line = lines.len().checked_sub(place.back + 1).map(|at| &lines[at])?;
    let size = line.len() as isize;
    let mut stop = size - place.rest as isize;
    if stop < 0 {
        stop = (stop + size).max(0);
    }
    let start = place.start.min(line.len());
    let run = &line[start..(stop as usize).max(start)];
    (!run.is_empty()).then_some(run)
}
because!(order_run_at, RunsOfItems, "the run at a place in the lines of a new case: the line counted from the end, from the first item of the place to as many items before the end of the line as the place had after it; an end that would fall before the start of the line is counted again from the end of the line, as an index below nothing counts from the end; nothing when the run is empty");

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct OrderTwoRuns {
    pub(super) runs: [Vec<u32>; ORDER_RUNS],
    pub(super) before: [Option<u32>; ORDER_RUNS],
}
because!(OrderTwoRuns, RunsOfItems, "two runs of one line with the same items in two orders, each with the item that stands before it, which the first run of a line need not have");

pub(super) fn order_two_runs(lines: &[Vec<u32>]) -> Option<OrderTwoRuns> {
    let mut sorted: Vec<u32> = Vec::new();
    for line in lines.iter().rev() {
        let size = line.len();
        if size > ORDER_MOST_ITEMS {
            continue;
        }
        for width in (ORDER_LEAST_RUN..=size / ORDER_LEAST_RUN).rev() {
            sorted.clear();
            for start in 0..=size - width {
                let from = sorted.len();
                sorted.extend_from_slice(&line[start..start + width]);
                sorted[from..].sort_unstable();
            }
            let bag = |start: usize| &sorted[start * width..(start + 1) * width];
            for first in 0..=size - ORDER_RUNS * width {
                let one = &line[first..first + width];
                for second in first + width..=size - width {
                    let other = &line[second..second + width];
                    if bag(first) == bag(second) && one != other {
                        return Some(OrderTwoRuns { runs: [one.to_vec(), other.to_vec()], before: [first.checked_sub(1).map(|at| line[at]), Some(line[second - 1])] });
                    }
                }
            }
        }
    }
    None
}
because!(order_two_runs, RunsOfItems, "the first two runs of one line that hold the same items in two orders: the lines from the end, the longest runs first, the first run from the start of the line, the second run after it");
