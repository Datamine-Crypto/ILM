use crate::controls::file::ChangedCase;
use ilm_core::base::words::answer_words;
use ilm_core::cases::Case;
use ilm_core::choice::inner_parts;
use patterns::{because, source};

pub struct LoopChanges;
source!(LoopChanges, "changed stories of a graph question about a loop: the told edges between numbered nodes are read, and whether they close a loop is found exactly with disjoint sets. In a graph with a loop, an edge whose removal leaves no loop is taken out, and the truth turns; an edge whose removal leaves a loop is taken out, and the truth stays. In a graph with no loop, an edge between two joined nodes is added, and the truth turns; an edge between two nodes that no chain joins is added, and the truth stays. The answer words of each truth are those of the lessons");

const LOOP_OPEN: char = '(';
because!(LOOP_OPEN, LoopChanges, "the bracket that opens an edge");

const LOOP_CLOSE: char = ')';
because!(LOOP_CLOSE, LoopChanges, "the bracket that closes an edge");

const LOOP_COMMA: char = ',';
because!(LOOP_COMMA, LoopChanges, "the mark between the two nodes of an edge");

const EDGE_OUT_TURNS: &str = "an edge of the only loops taken out";
because!(EDGE_OUT_TURNS, LoopChanges, "the sort of the change where an edge is taken out and no loop is left");

const EDGE_OUT_STAYS: &str = "an edge taken out and a loop left";
because!(EDGE_OUT_STAYS, LoopChanges, "the sort of the change where an edge is taken out and a loop is left");

const EDGE_IN_TURNS: &str = "an edge added between two joined nodes";
because!(EDGE_IN_TURNS, LoopChanges, "the sort of the change where an added edge closes a loop");

const EDGE_IN_STAYS: &str = "an edge added between two nodes that no chain joins";
because!(EDGE_IN_STAYS, LoopChanges, "the sort of the change where an added edge closes no loop");

const LOOP_GAP: &str = " ";
because!(LOOP_GAP, LoopChanges, "the mark between the words of a new truth");

fn edge(sentence: &str) -> Option<(usize, usize)> {
    let inner = sentence.trim().strip_prefix(LOOP_OPEN)?.strip_suffix(LOOP_CLOSE)?;
    let (one, other) = inner.split_once(LOOP_COMMA)?;
    Some((one.trim().parse().ok()?, other.trim().parse().ok()?))
}
because!(edge, LoopChanges, "the two nodes of a told sentence that is an edge, and nothing for any other sentence");

fn top(parent: &mut Vec<usize>, at: usize) -> usize {
    if parent.len() <= at {
        parent.extend(parent.len()..=at);
    }
    let mut at = at;
    while parent[at] != at {
        at = parent[at];
    }
    at
}
because!(top, LoopChanges, "the root of the set of a node, the sets growing to the node when it is new");

fn looped(edges: &[(usize, usize)]) -> (bool, Vec<usize>) {
    let mut parent: Vec<usize> = Vec::new();
    let mut found = false;
    for (one, other) in edges {
        let (first, second) = (top(&mut parent, *one), top(&mut parent, *other));
        if first == second {
            found = true;
        } else {
            parent[first] = second;
        }
    }
    (found, parent)
}
because!(looped, LoopChanges, "whether the edges close a loop, with the sets of the nodes at the end");

pub fn loop_changes(lessons: &[Case], whole: bool) -> Vec<ChangedCase> {
    let read: Vec<Option<(Vec<usize>, Vec<(usize, usize)>, bool)>> = lessons
        .iter()
        .map(|case| {
            let places: Vec<usize> = (0..case.sentences.len()).filter(|at| edge(&case.sentences[*at]).is_some()).collect();
            let edges: Vec<(usize, usize)> = places.iter().filter_map(|at| edge(&case.sentences[*at])).collect();
            (!edges.is_empty()).then(|| {
                let (found, _) = looped(&edges);
                (places, edges, found)
            })
        })
        .collect();
    let answer_of = |wanted: bool| lessons.iter().zip(&read).find(|(_, read)| read.as_ref().is_some_and(|(_, _, found)| *found == wanted)).map(|(case, _)| answer_words(&case.answer).join(LOOP_GAP));
    let (Some(yes), Some(no)) = (answer_of(true), answer_of(false)) else { return Vec::new() };
    let last: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let (case, Some((places, edges, found))) = (&lessons[place], &read[place]) else { continue };
        if answer_words(&case.answer).join(LOOP_GAP) != if *found { yes.clone() } else { no.clone() } {
            continue;
        }
        let change = |sort: &str, sentences: Vec<String>, truth: bool| ChangedCase { place: Some(place), sort: sort.to_string(), sentences, question: case.question.clone(), truth: if truth { yes.clone() } else { no.clone() } };
        if *found {
            let mut turned = false;
            let mut stayed = false;
            for (number, at) in places.iter().enumerate() {
                let rest: Vec<(usize, usize)> = edges.iter().enumerate().filter(|(other, _)| *other != number).map(|(_, pair)| *pair).collect();
                let (still, _) = looped(&rest);
                let taken: Vec<String> = case.sentences.iter().enumerate().filter(|(other, _)| other != at).map(|(_, sentence)| sentence.clone()).collect();
                if !still && !turned {
                    out.push(change(EDGE_OUT_TURNS, taken, false));
                    turned = true;
                } else if still && !stayed {
                    out.push(change(EDGE_OUT_STAYS, taken, true));
                    stayed = true;
                }
            }
            continue;
        }
        let (_, mut parent) = looped(edges);
        let nodes: Vec<usize> = edges.iter().flat_map(|(one, other)| [*one, *other]).collect();
        let first = nodes[0];
        let root = top(&mut parent, first);
        let joined = nodes.iter().copied().find(|node| *node != first && top(&mut parent, *node) == root && !edges.contains(&(first, *node)) && !edges.contains(&(*node, first)));
        let apart = nodes.iter().copied().find(|node| top(&mut parent, *node) != root);
        let last_edge = places.last().copied().unwrap_or(0);
        for (other, sort, truth) in [(joined, EDGE_IN_TURNS, true), (apart, EDGE_IN_STAYS, false)] {
            let Some(other) = other else { continue };
            let mut sentences = case.sentences.clone();
            sentences.insert(last_edge + 1, format!("{LOOP_OPEN}{first}{LOOP_COMMA}{other}{LOOP_CLOSE}"));
            out.push(change(sort, sentences, truth));
        }
    }
    out
}
because!(loop_changes, LoopChanges, "the changed stories of the lessons that the model does not learn, or of all lessons for a model of all lessons, with the answer words of each truth as the lessons write them; a lesson whose reading does not give its answer is left out");
