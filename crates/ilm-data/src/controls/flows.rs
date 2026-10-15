use crate::controls::file::ChangedCase;
use crate::controls::text::number_runs;
use ilm_core::base::words::answer_words;
use ilm_core::cases::Case;
use ilm_core::choice::inner_parts;
use patterns::{because, source};
use std::collections::VecDeque;

pub struct FlowChanges;
source!(FlowChanges, "changed stories of a question about the largest flow: each told sentence with three numbers is an edge, from the first node to the second, with the third number as its capacity; the question names the source and the sink as its first two numbers, and the answer ends with the flow. The flow is found exactly with augmenting paths (Edmonds and Karp). An edge is taken out so that the flow changes, and the truth turns; an edge is taken out so that the flow stays, and the truth stays; the new truth is the answer of the lesson with its last number made the new flow");

const EDGE_NUMBERS: usize = 3;
because!(EDGE_NUMBERS, FlowChanges, "the numbers of an edge sentence: the node it leaves, the node it reaches, and its capacity");

const ASKED_NODES: usize = 2;
because!(ASKED_NODES, FlowChanges, "the numbers of the question that name the source and the sink");

const FLOW_TURNS: &str = "an edge taken out and the flow changes";
because!(FLOW_TURNS, FlowChanges, "the sort of the change where an edge is taken out and the largest flow changes");

const FLOW_STAYS: &str = "an edge taken out and the flow stays";
because!(FLOW_STAYS, FlowChanges, "the sort of the change where an edge is taken out and the largest flow stays");

const FLOW_GAP: &str = " ";
because!(FLOW_GAP, FlowChanges, "the mark between the words of a new truth");

fn numbers(text: &str) -> Vec<u64> {
    number_runs(text, false).iter().filter_map(|(start, end)| text[*start..*end].parse::<u64>().ok()).collect()
}
because!(numbers, FlowChanges, "the whole numbers of a text in their order");

fn largest_flow(edges: &[(usize, usize, u64)], source: usize, sink: usize) -> u64 {
    let count = edges.iter().flat_map(|(from, to, _)| [*from, *to]).chain([source, sink]).max().map_or(0, |most| most + 1);
    let mut room: Vec<Vec<u64>> = vec![vec![0; count]; count];
    for (from, to, size) in edges {
        room[*from][*to] += size;
    }
    let mut total = 0u64;
    loop {
        let mut came: Vec<Option<usize>> = vec![None; count];
        came[source] = Some(source);
        let mut queue: VecDeque<usize> = VecDeque::from([source]);
        while let Some(at) = queue.pop_front() {
            for next in 0..count {
                if came[next].is_none() && room[at][next] > 0 {
                    came[next] = Some(at);
                    queue.push_back(next);
                }
            }
        }
        if source == sink || came[sink].is_none() {
            return total;
        }
        let (mut least, mut at) = (u64::MAX, sink);
        while let Some(back) = came[at].filter(|_| at != source) {
            least = least.min(room[back][at]);
            at = back;
        }
        let mut at = sink;
        while let Some(back) = came[at].filter(|_| at != source) {
            room[back][at] -= least;
            room[at][back] += least;
            at = back;
        }
        total += least;
    }
}
because!(largest_flow, FlowChanges, "the largest flow from the source to the sink along the edges, each with its capacity in its direction");

pub fn flow_changes(lessons: &[Case], whole: bool) -> Vec<ChangedCase> {
    let last: Vec<usize> = if whole { (0..lessons.len()).collect() } else { inner_parts(lessons).1 };
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in last {
        let case = &lessons[place];
        let places: Vec<usize> = (0..case.sentences.len()).filter(|at| numbers(&case.sentences[*at]).len() == EDGE_NUMBERS).collect();
        let edges: Vec<(usize, usize, u64)> = places
            .iter()
            .filter_map(|at| match numbers(&case.sentences[*at]).as_slice() {
                [from, to, size] => Some((*from as usize, *to as usize, *size)),
                _ => None,
            })
            .collect();
        let asked = numbers(&case.question);
        let said = answer_words(&case.answer);
        if edges.is_empty() || asked.len() != ASKED_NODES || said.last().and_then(|word| word.parse::<u64>().ok()) != Some(largest_flow(&edges, asked[0] as usize, asked[1] as usize)) {
            continue;
        }
        let flow = largest_flow(&edges, asked[0] as usize, asked[1] as usize);
        let mut turned = false;
        let mut stayed = false;
        for (number, at) in places.iter().enumerate() {
            let rest: Vec<(usize, usize, u64)> = edges.iter().enumerate().filter(|(other, _)| *other != number).map(|(_, edge)| *edge).collect();
            let new = largest_flow(&rest, asked[0] as usize, asked[1] as usize);
            let (sort, done) = if new == flow { (FLOW_STAYS, &mut stayed) } else { (FLOW_TURNS, &mut turned) };
            if *done {
                continue;
            }
            *done = true;
            let mut truth = said.clone();
            if let Some(end) = truth.last_mut() {
                *end = new.to_string();
            }
            let sentences: Vec<String> = case.sentences.iter().enumerate().filter(|(other, _)| other != at).map(|(_, sentence)| sentence.clone()).collect();
            out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences, question: case.question.clone(), truth: truth.join(FLOW_GAP) });
        }
    }
    out
}
because!(flow_changes, FlowChanges, "the changed stories of the lessons that the model does not learn, or of all lessons for a model of all lessons: the first edge whose removal changes the flow, and the first whose removal keeps it, each with its new truth; a lesson whose reading does not give its answer is left out");
