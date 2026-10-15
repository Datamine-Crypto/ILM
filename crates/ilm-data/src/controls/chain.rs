use crate::controls::file::ChangedCase;
use crate::sets::readers::common::{python_words, stripped};
use ilm_core::choice::inner_parts;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct ChainChanges;
source!(ChainChanges, "changed stories of tasks that pass by a chain: a told sentence is needed when no way is left from the first thing of the question to the thing of the answer once the sentence is taken out, by a search in breadth over the links of the story. Each needed sentence is taken out in turn, and every sentence that links the first thing is taken out at once, which leaves no way in a dense graph; the new truth is not known, so it is written as no answer, and the count that says something is how often a model keeps its old answer. The first sentence that is not on the way is taken out with the old answer. In a graph question whose answer is no, an edge that joins the two sides is added, and the answer becomes yes. The links are read for three sets: a class and the class above it, an edge between two numbered nodes, and a relation between two persons");

const CHAIN_CLASSES: &str = "src_prosqa";
because!(CHAIN_CLASSES, ChainChanges, "the set whose stories link a thing to a class and a class to the class above it");

const CHAIN_GRAPHS: &str = "src_nlgraph";
because!(CHAIN_GRAPHS, ChainChanges, "the set whose stories are edges between numbered nodes");

const CHAIN_FAMILIES: &str = "src_clutrr_graph";
because!(CHAIN_FAMILIES, ChainChanges, "the set whose stories are relations between persons, each a name, a relation and a name");

const CHAIN_EVERY: &str = "Every";
because!(CHAIN_EVERY, ChainChanges, "the first word of a sentence that puts a class under another class");

const CHAIN_IS: &str = "is";
because!(CHAIN_IS, ChainChanges, "the verb of a sentence that puts a thing in a class");

const CHAIN_CLASS_WORDS: usize = 5;
because!(CHAIN_CLASS_WORDS, ChainChanges, "the words of a sentence that puts a class under another class: every, the class, is, a, the class above");

const CHAIN_THING_WORDS: usize = 4;
because!(CHAIN_THING_WORDS, ChainChanges, "the words of a sentence that puts a thing in a class: the thing, is, a, the class");

const CHAIN_ABOVE: usize = 4;
because!(CHAIN_ABOVE, ChainChanges, "the place of the class above in a sentence that puts a class under another class");

const CHAIN_CLASS_OF_THING: usize = 3;
because!(CHAIN_CLASS_OF_THING, ChainChanges, "the place of the class in a sentence that puts a thing in a class");

const CHAIN_END: char = '.';
because!(CHAIN_END, ChainChanges, "the mark at the end of a told sentence and of an answer");

const CHAIN_YES: &str = "yes";
because!(CHAIN_YES, ChainChanges, "the answer of a graph question whose two nodes are joined");

const CHAIN_NO: &str = "no";
because!(CHAIN_NO, ChainChanges, "the answer of a graph question whose two nodes are not joined");

const CHAIN_NODE: &str = "node ";
because!(CHAIN_NODE, ChainChanges, "the word before the number of a node in a question");

const CHAIN_NODE_AND: &str = " and node ";
because!(CHAIN_NODE_AND, ChainChanges, "the words between the two nodes of a question");

const CHAIN_OPEN: char = '(';
because!(CHAIN_OPEN, ChainChanges, "the bracket that opens an edge");

const CHAIN_CLOSE: char = ')';
because!(CHAIN_CLOSE, ChainChanges, "the bracket that closes an edge");

const CHAIN_COMMA: char = ',';
because!(CHAIN_COMMA, ChainChanges, "the mark between the two nodes of an edge");

const CHAIN_PAIR_OPEN: &str = "('";
because!(CHAIN_PAIR_OPEN, ChainChanges, "the start of the pair of persons that a question asks about");

const CHAIN_PAIR_MIDDLE: &str = "', '";
because!(CHAIN_PAIR_MIDDLE, ChainChanges, "the marks between the two persons of the pair of a question");

const CHAIN_PAIR_CLOSE: &str = "')";
because!(CHAIN_PAIR_CLOSE, ChainChanges, "the end of the pair of persons that a question asks about");

const CHAIN_QUOTE: char = '\'';
because!(CHAIN_QUOTE, ChainChanges, "the mark on both sides of a person of the pair of a question, which a name does not hold");

const SORT_ALL_LINKS: &str = "every link of the first thing taken out";
because!(SORT_ALL_LINKS, ChainChanges, "the sort of the change where every sentence that links the first thing is taken out");

const SORT_NEEDED: &str = "a needed sentence taken out";
because!(SORT_NEEDED, ChainChanges, "the sort of the change where a sentence without which no way is left is taken out");

const SORT_OFF_WAY: &str = "a sentence that is not on the way taken out";
because!(SORT_OFF_WAY, ChainChanges, "the sort of the change where the first sentence that is not on the way is taken out");

const SORT_JOINED: &str = "an edge that joins the two sides added";
because!(SORT_JOINED, ChainChanges, "the sort of the change where an edge between the two parts of a graph is added");

type Link = (String, String, usize);
because!(Link, ChainChanges, "a link of a story: its two ends and the place of its sentence");

struct ChainStory {
    links: Vec<Link>,
    directed: bool,
    start: String,
    goal: String,
}
because!(ChainStory, ChainChanges, "the links of a story, whether they lead one way only, and the first and the last thing of the way that the question asks for");

fn graph_edge(sentence: &str) -> Option<(String, String)> {
    let inner = stripped(sentence).strip_prefix(CHAIN_OPEN)?.strip_suffix(CHAIN_CLOSE)?;
    let (one, other) = inner.split_once(CHAIN_COMMA)?;
    let other = other.trim_start_matches(ilm_core::know::signs::is_space);
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit());
    (digits(one) && digits(other)).then(|| (one.to_string(), other.to_string()))
}
because!(graph_edge, ChainChanges, "the two nodes of a told edge: a bracket, a number, a comma, space, a number and a bracket, and nothing more");

fn digit_run(text: &str) -> &str {
    let length = text.bytes().take_while(u8::is_ascii_digit).count();
    &text[..length]
}
because!(digit_run, ChainChanges, "the digits at the start of a text");

fn asked_nodes(question: &str) -> Option<(String, String)> {
    question.match_indices(CHAIN_NODE).find_map(|(at, _)| {
        let rest = &question[at + CHAIN_NODE.len()..];
        let one = digit_run(rest);
        let other = digit_run(rest[one.len()..].strip_prefix(CHAIN_NODE_AND)?);
        (!one.is_empty() && !other.is_empty()).then(|| (one.to_string(), other.to_string()))
    })
}
because!(asked_nodes, ChainChanges, "the two nodes that a graph question names, searched from the left");

fn asked_pair(question: &str) -> Option<(String, String)> {
    question.match_indices(CHAIN_PAIR_OPEN).find_map(|(at, _)| {
        let rest = &question[at + CHAIN_PAIR_OPEN.len()..];
        let one = &rest[..rest.find(CHAIN_QUOTE)?];
        let rest = rest[one.len()..].strip_prefix(CHAIN_PAIR_MIDDLE)?;
        let other = &rest[..rest.find(CHAIN_QUOTE)?];
        rest[other.len()..].starts_with(CHAIN_PAIR_CLOSE).then_some(())?;
        (!one.is_empty() && !other.is_empty()).then(|| (one.to_string(), other.to_string()))
    })
}
because!(asked_pair, ChainChanges, "the pair of persons that a question names, searched from the left");

fn chain_story(set: &str, case: &Case) -> Option<ChainStory> {
    let mut links: Vec<Link> = Vec::new();
    match set {
        CHAIN_CLASSES => {
            for (at, sentence) in case.sentences.iter().enumerate() {
                let words = python_words(sentence.trim_end_matches(CHAIN_END));
                if words.len() == CHAIN_CLASS_WORDS && words[0] == CHAIN_EVERY {
                    links.push((words[1].to_string(), words[CHAIN_ABOVE].to_string(), at));
                } else if words.len() == CHAIN_THING_WORDS && words[1] == CHAIN_IS {
                    links.push((words[0].to_string(), words[CHAIN_CLASS_OF_THING].to_string(), at));
                }
            }
            let said = python_words(case.answer.trim_end_matches(CHAIN_END));
            Some(ChainStory { links, directed: true, start: said.first()?.to_string(), goal: said.last()?.to_string() })
        }
        CHAIN_GRAPHS => {
            for (at, sentence) in case.sentences.iter().enumerate() {
                if let Some((one, other)) = graph_edge(sentence) {
                    links.push((one, other, at));
                }
            }
            let (start, goal) = asked_nodes(&case.question)?;
            case.answer.contains(CHAIN_YES).then_some(ChainStory { links, directed: false, start, goal })
        }
        CHAIN_FAMILIES => {
            for (at, sentence) in case.sentences.iter().enumerate() {
                let words = python_words(sentence);
                links.push((words.first()?.to_string(), words.last()?.to_string(), at));
            }
            let (start, goal) = asked_pair(&case.question)?;
            Some(ChainStory { links, directed: false, start, goal })
        }
        _ => None,
    }
}
because!(chain_story, ChainChanges, "the links of a case and the way that its question asks for; nothing for a graph question whose answer is no, or a case of another form");

fn shortest_way(story: &ChainStory, skip: Option<usize>) -> Option<Vec<usize>> {
    let mut near: HashMap<&str, Vec<(&str, usize)>> = HashMap::new();
    for (one, other, at) in &story.links {
        if Some(*at) == skip {
            continue;
        }
        near.entry(one.as_str()).or_default().push((other.as_str(), *at));
        if !story.directed {
            near.entry(other.as_str()).or_default().push((one.as_str(), *at));
        }
    }
    let mut back: HashMap<&str, Option<(&str, usize)>> = HashMap::new();
    back.insert(story.start.as_str(), None);
    let mut queue: VecDeque<&str> = VecDeque::from([story.start.as_str()]);
    while let Some(place) = queue.pop_front() {
        if place == story.goal {
            let mut out: Vec<usize> = Vec::new();
            let mut at = place;
            while let Some(Some((before, sentence))) = back.get(at).copied() {
                out.push(sentence);
                at = before;
            }
            out.reverse();
            return Some(out);
        }
        for (next, sentence) in near.get(place).cloned().unwrap_or_default() {
            if let std::collections::hash_map::Entry::Vacant(entry) = back.entry(next) {
                entry.insert(Some((place, sentence)));
                queue.push_back(next);
            }
        }
    }
    None
}
because!(shortest_way, ChainChanges, "the places of the sentences on a shortest way from the first thing to the last, found in breadth with the links in the order of the story, or nothing when no way is left; one sentence may be left out");

fn sides_joined(sentences: &[String], start: &str, goal: &str) -> Option<String> {
    let edges: Vec<(String, String)> = sentences.iter().filter_map(|sentence| graph_edge(sentence)).collect();
    let side = |from: &str| -> String {
        let mut seen: HashSet<String> = HashSet::from([from.to_string()]);
        let mut queue: VecDeque<String> = VecDeque::from([from.to_string()]);
        while let Some(place) = queue.pop_front() {
            for (one, other) in &edges {
                let next = if *one == place { Some(other) } else if *other == place { Some(one) } else { None };
                if let Some(next) = next
                    && seen.insert(next.clone())
                {
                    queue.push_back(next.clone());
                }
            }
        }
        let mut nodes: Vec<String> = seen.into_iter().collect();
        nodes.sort_by(|left, right| (left == from).cmp(&(right == from)).then_with(|| left.cmp(right)));
        nodes.into_iter().next().unwrap_or_default()
    };
    (!edges.is_empty()).then(|| format!("{CHAIN_OPEN}{}{CHAIN_COMMA}{}{CHAIN_CLOSE}", side(start), side(goal)))
}
because!(sides_joined, ChainChanges, "a new edge between the two parts of a graph: from each node of the question the first node of its part by the order of their texts, the node itself last");

pub fn chain_changes(lessons: &[Case], set: &str) -> Vec<ChangedCase> {
    let part = inner_parts(lessons).1;
    let yes = if set == CHAIN_GRAPHS { lessons.iter().find(|case| case.answer.contains(CHAIN_YES)).map(|case| case.answer.clone()) } else { None };
    let mut out: Vec<ChangedCase> = Vec::new();
    for place in part {
        let case = &lessons[place];
        let without = |taken: &dyn Fn(usize) -> bool| -> Vec<String> { case.sentences.iter().enumerate().filter(|(at, _)| !taken(*at)).map(|(_, sentence)| sentence.clone()).collect() };
        let mut add = |sort: &str, sentences: Vec<String>, truth: String| out.push(ChangedCase { place: Some(place), sort: sort.to_string(), sentences, question: case.question.clone(), truth });
        if let Some(story) = chain_story(set, case)
            && let Some(way) = shortest_way(&story, None)
        {
            let linked: HashSet<usize> = story.links.iter().filter(|(one, other, _)| *one == story.start || *other == story.start).map(|(_, _, at)| *at).collect();
            add(SORT_ALL_LINKS, without(&|at| linked.contains(&at)), String::new());
            let needed: Vec<usize> = way.iter().copied().filter(|at| shortest_way(&story, Some(*at)).is_none()).collect();
            if !needed.is_empty() {
                for at in &needed {
                    add(SORT_NEEDED, without(&|other| other == *at), String::new());
                }
                if let Some(free) = (0..case.sentences.len()).find(|at| !way.contains(at)) {
                    add(SORT_OFF_WAY, without(&|other| other == free), case.answer.clone());
                }
            }
        }
        if let (Some(yes), Some((start, goal))) = (&yes, asked_nodes(&case.question))
            && case.answer.contains(CHAIN_NO)
            && let Some(edge) = sides_joined(&case.sentences, &start, &goal)
        {
            let last = case.sentences.iter().rposition(|sentence| graph_edge(sentence).is_some()).unwrap_or_default();
            let mut sentences = case.sentences.clone();
            sentences.insert(last + 1, edge);
            add(SORT_JOINED, sentences, yes.clone());
        }
    }
    out
}
because!(chain_changes, ChainChanges, "the changed stories of the lessons that the model does not learn: the links of the first thing, each needed sentence and a sentence off the way taken out, and in a graph with the answer no an edge that joins the two sides added");
