use crate::base::first_sight::FirstSightTable;
use crate::base::symbols::Symbols;
use crate::base::words::{answer_words, words};
use crate::tools::call::{Learned, Learner, Lessons, Said, Tool};
use crate::tools::frequent::frequent_answer;
use crate::tools::pattern::model::SURE_SHARE;
use patterns::{because, source};
use std::collections::VecDeque;

pub struct LinkGraph;
source!(LinkGraph, "a story read as a graph: of the told sentences of all lessons, those of the most frequent shape where exactly two places change from sentence to sentence and every other place keeps its word are links between the two things that stand in those places. A question is answered by a property of the graph: whether the links close a loop, or whether a chain of links joins the two things that the question names, found with the disjoint sets of Galler and Fischer, or the largest flow between them; which property a form of question asks, and which answer each value of the property gives, are counted from the lessons");

pub struct GraphProperty;
source!(GraphProperty, "the properties of a graph that the tool reads, a small fixed set like the calculations of the amount tool: a loop among the links, a chain between two named things, and the largest flow from one named thing to the other");

const GRAPH_SLOT: &str = "_";
because!(GRAPH_SLOT, LinkGraph, "the mark of a place of a question that holds a thing of the graph, in the text of the form of a question; the word cutter never gives it as a word");

const GRAPH_BETWEEN: &str = " ";
because!(GRAPH_BETWEEN, LinkGraph, "the mark between the words of the form of a question and of an answer");

const LINK_PLACES: usize = 2;
because!(LINK_PLACES, LinkGraph, "the places of a link sentence that change from sentence to sentence: a link joins a pair of things");

const LEAST_LINKS: usize = 2;
because!(LEAST_LINKS, LinkGraph, "the fewest sentences of one shape in the lessons that make a shape of links: with fewer, no place of a shape is seen to change");

const NO_GRAPH: &str = "the graph tool does not read this kind: no property of the links of its stories gives the answers of the lessons";
because!(NO_GRAPH, LinkGraph, "what the tool says when no form of question gets a property whose values give the answers");

const NOTE_GRAPH: &str = "a property of the graph of the told links";
because!(NOTE_GRAPH, LinkGraph, "the note of an answer that a property of the graph gives");

const NOTE_NO_GRAPH: &str = "no property of the graph for this question";
because!(NOTE_NO_GRAPH, LinkGraph, "the note of an answer for a question whose form has no property, or whose story has no links");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Property {
    Loop,
    Chain,
    Flow,
}
because!(Property, GraphProperty, "a property of a graph: a loop among its links, a chain of links between the two named things, or the largest flow from the first named thing to the second along the links in their direction, with the number of each link as its capacity");

const TRUTHS: &[Property] = &[Property::Loop, Property::Chain];
because!(TRUTHS, GraphProperty, "the properties that are true or false, in the order in which they are tried; of two that give the answers equally well the first is taken");

const AMOUNTS: &[Property] = &[Property::Flow];
because!(AMOUNTS, GraphProperty, "the properties that are a whole number, tried after the properties that are true or false");

struct Graph {
    links: Vec<(usize, usize, Option<u64>)>,
    things: Symbols,
}
because!(Graph, LinkGraph, "the links of a story as pairs of numbers of things, each with the number that its sentence holds when the shape of a link holds one, and the text of each thing");

#[derive(Clone, Debug, Default)]
struct LinkShape {
    size: usize,
    fixed: Vec<(usize, String)>,
    open: Vec<usize>,
    weight: Option<usize>,
    numbered: bool,
}
because!(LinkShape, LinkGraph, "the shape of a link sentence: its number of words, the word that stands at each place that never changes, the two places that hold things, the place of the number of a link when one more place changes, and whether every thing of the links of the lessons is a number, so that a number of a question is a thing even when no link names it. learn finds it from the told sentences and the questions of all lessons; links reads the links of a story with it");

const WEIGHTED_PLACES: usize = 3;
because!(WEIGHTED_PLACES, LinkGraph, "the changing places of a link sentence that holds a number of its link beside its pair of things");

impl LinkShape {
    fn learn(stories: &[(Vec<Vec<String>>, Vec<String>)]) -> Option<LinkShape> {
        let mut by_size: FirstSightTable<usize, Vec<(&Vec<String>, &Vec<String>)>> = FirstSightTable::new();
        for (told, asked) in stories {
            for said in told {
                by_size.entry(said.len(), Vec::new()).push((said, asked));
            }
        }
        let mut best: Option<(usize, LinkShape)> = None;
        for (size, members) in by_size.rows() {
            if members.len() < LEAST_LINKS || best.as_ref().is_some_and(|(held, _)| members.len() <= *held) {
                continue;
            }
            let first = members[0].0;
            let open: Vec<usize> = (0..*size).filter(|at| members.iter().any(|(member, _)| member[*at] != first[*at])).collect();
            let weight = match open.len() {
                LINK_PLACES => None,
                WEIGHTED_PLACES => {
                    let asked = |at: &usize| members.iter().filter(|(member, question)| question.contains(&member[*at])).count();
                    let Some(place) = open.iter().copied().min_by_key(asked) else { continue };
                    if !members.iter().all(|(member, _)| member[place].parse::<u64>().is_ok()) {
                        continue;
                    }
                    Some(place)
                }
                _ => continue,
            };
            let things: Vec<usize> = open.iter().copied().filter(|at| Some(*at) != weight).collect();
            let numbered = members.iter().all(|(member, _)| things.iter().all(|at| member[*at].parse::<u64>().is_ok()));
            let fixed = (0..*size).filter(|at| !open.contains(at)).map(|at| (at, first[at].clone())).collect();
            best = Some((members.len(), LinkShape { size: *size, fixed, open: things, weight, numbered }));
        }
        best.map(|(_, shape)| shape)
    }

    fn links(&self, sentences: &[Vec<String>]) -> Option<Graph> {
        let mut things = Symbols::new();
        let mut links: Vec<(usize, usize, Option<u64>)> = Vec::new();
        for said in sentences {
            if said.len() == self.size && self.fixed.iter().all(|(at, word)| said[*at] == *word) {
                let weight = self.weight.and_then(|at| said[at].parse::<u64>().ok());
                links.push((things.number(&said[self.open[0]]) as usize, things.number(&said[self.open[1]]) as usize, weight));
            }
        }
        (!links.is_empty()).then_some(Graph { links, things })
    }
}

fn root(parent: &mut [usize], mut at: usize) -> usize {
    while parent[at] != at {
        parent[at] = parent[parent[at]];
        at = parent[at];
    }
    at
}
because!(root, LinkGraph, "the root of the set of a thing, with each step pointed one level higher on the way, the halving of Tarjan and van Leeuwen");

fn truth(graph: &Graph, asked: &[usize], wanted: Property) -> Option<bool> {
    let mut parent: Vec<usize> = (0..graph.things.len()).collect();
    let mut looped = false;
    for (one, other, _) in &graph.links {
        let (top, other_top) = (root(&mut parent, *one), root(&mut parent, *other));
        if top == other_top {
            looped = true;
        } else {
            parent[top] = other_top;
        }
    }
    match (wanted, asked) {
        (Property::Loop, _) => Some(looped),
        (Property::Chain, [one, other]) => Some(root(&mut parent, *one) == root(&mut parent, *other)),
        _ => None,
    }
}
because!(truth, LinkGraph, "the value of a property that is true or false: the links are joined set by set, and a link whose things are in one set already closes a loop; two things are chained when they end in one set; nothing for a chain when the question does not name two things");

pub struct AugmentingPaths;
source!(AugmentingPaths, "the largest flow of Ford and Fulkerson, with each augmenting path the shortest one, found by a search in breadth, as Edmonds and Karp give it");

fn flow(graph: &Graph, asked: &[usize]) -> Option<u64> {
    let [source, sink] = asked else { return None };
    let count = graph.things.len();
    let mut room: Vec<Vec<u64>> = vec![vec![0; count]; count];
    for (from, to, weight) in &graph.links {
        room[*from][*to] += (*weight)?;
    }
    let mut total = 0u64;
    loop {
        let mut came: Vec<Option<usize>> = vec![None; count];
        came[*source] = Some(*source);
        let mut queue: VecDeque<usize> = VecDeque::from([*source]);
        while let Some(at) = queue.pop_front() {
            for next in 0..count {
                if came[next].is_none() && room[at][next] > 0 {
                    came[next] = Some(at);
                    queue.push_back(next);
                }
            }
        }
        if came[*sink].is_none() || source == sink {
            return Some(total);
        }
        let mut least = u64::MAX;
        let mut at = *sink;
        while at != *source {
            let back = came[at]?;
            least = least.min(room[back][at]);
            at = back;
        }
        let mut at = *sink;
        while at != *source {
            let back = came[at]?;
            room[back][at] -= least;
            room[at][back] += least;
            at = back;
        }
        total += least;
    }
}
because!(flow, AugmentingPaths, "the largest flow from the first named thing to the second, the number of each link its capacity in its direction; nothing when the question does not name two things or a link has no number");

fn graph_amount(graph: &Graph, asked: &[usize], wanted: Property) -> Option<u64> {
    match wanted {
        Property::Flow => flow(graph, asked),
        _ => None,
    }
}
because!(graph_amount, GraphProperty, "the value of a property that is a whole number");

struct GraphRead {
    graph: Graph,
    form: String,
    asked: Vec<usize>,
}
because!(GraphRead, LinkGraph, "a case read for the tool: the graph of its story, the form of its question, and the things of the graph that the question names in its order");

fn read_case(shape: &LinkShape, sentences: &[String], question: &str) -> Option<GraphRead> {
    let told: Vec<Vec<String>> = sentences.iter().map(|sentence| words(sentence)).collect();
    let mut graph = shape.links(&told)?;
    let mut asked: Vec<usize> = Vec::new();
    let mut form: Vec<String> = Vec::new();
    for word in words(question) {
        let alone = shape.numbered && word.parse::<u64>().is_ok();
        match graph.things.find(&word).or_else(|| alone.then(|| graph.things.number(&word))) {
            Some(thing) => {
                asked.push(thing as usize);
                form.push(GRAPH_SLOT.to_string());
            }
            None => form.push(word),
        }
    }
    Some(GraphRead { graph, form: form.join(GRAPH_BETWEEN), asked })
}
because!(read_case, LinkGraph, "a case read for the tool, and nothing when its story has no links; where every thing of the links is a number, a number of the question is a thing of the graph with no link");

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum AnswerPiece {
    Word(String),
    Thing(usize),
    Value,
}
because!(AnswerPiece, GraphProperty, "one word of the form of an answer that holds a whole number: a word as it stands, the thing that the question names at a place, or the value of the property");

#[derive(Clone, Debug)]
enum GraphAsk {
    Truth(Property, Vec<(bool, Vec<String>)>),
    Amount(Property, Vec<AnswerPiece>),
}
because!(GraphAsk, GraphProperty, "what a form of question asks: a property that is true or false with the answer of each value, or a property that is a whole number with the form of its answer");

fn pieces(said: &[String], read: &GraphRead, value: u64) -> Vec<AnswerPiece> {
    let text = value.to_string();
    let last = said.iter().rposition(|word| *word == text);
    said.iter()
        .enumerate()
        .map(|(at, word)| {
            if Some(at) == last {
                return AnswerPiece::Value;
            }
            match read.asked.iter().position(|thing| read.graph.things.name(*thing as u32) == word) {
                Some(place) => AnswerPiece::Thing(place),
                None => AnswerPiece::Word(word.clone()),
            }
        })
        .collect()
}
because!(pieces, GraphProperty, "the form of an answer for a value: the last word that writes the value is the value, a word that writes a thing that the question names is that thing, and every other word stands as it is");

fn written(form: &[AnswerPiece], read: &GraphRead, value: u64) -> Option<Vec<String>> {
    form.iter()
        .map(|piece| match piece {
            AnswerPiece::Word(word) => Some(word.clone()),
            AnswerPiece::Thing(place) => read.asked.get(*place).map(|thing| read.graph.things.name(*thing as u32).to_string()),
            AnswerPiece::Value => Some(value.to_string()),
        })
        .collect()
}
because!(written, GraphProperty, "the words of an answer from its form, and nothing when the question names no thing at a place of the form");

pub struct GraphModel {
    shape: LinkShape,
    asks: FirstSightTable<String, GraphAsk>,
    default: Vec<String>,
}
because!(GraphModel, LinkGraph, "what the tool learned: the shape of a link sentence, for each form of question what it asks, and the most frequent answer of the lessons");

impl Learner for GraphModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let none = || Said { words: self.default.clone(), note: NOTE_NO_GRAPH.to_string() };
        let Some(read) = read_case(&self.shape, sentences, question) else { return none() };
        let said = match self.asks.get(&read.form) {
            Some(GraphAsk::Truth(wanted, answers)) => truth(&read.graph, &read.asked, *wanted).and_then(|value| answers.iter().find(|(held, _)| *held == value)).map(|(_, said)| said.clone()),
            Some(GraphAsk::Amount(wanted, form)) => graph_amount(&read.graph, &read.asked, *wanted).and_then(|value| written(form, &read, value)),
            None => None,
        };
        match said {
            Some(words) => Said { words, note: NOTE_GRAPH.to_string() },
            None => none(),
        }
    }
}

fn sure_answers(counts: &FirstSightTable<bool, FirstSightTable<Vec<String>, u32>>) -> Option<(u32, Vec<(bool, Vec<String>)>)> {
    let mut covered = 0u32;
    let mut answers: Vec<(bool, Vec<String>)> = Vec::new();
    for (value, said) in counts.rows() {
        let total: u32 = said.rows().iter().map(|(_, count)| *count).sum();
        let (top, hit) = said.first_most()?;
        if f64::from(*hit) < SURE_SHARE * f64::from(total) {
            return None;
        }
        covered += hit;
        answers.push((*value, top.clone()));
    }
    let distinct = answers.len() < LINK_PLACES || answers[0].1 != answers[1].1;
    distinct.then_some((covered, answers))
}
because!(sure_answers, GraphProperty, "the answer of each value of a property for one form of question, when nearly every lesson with that value has one answer and the two values have different answers; with the lessons that it gives right");

fn sure_form(counts: &FirstSightTable<Vec<AnswerPiece>, u32>, lessons: u32) -> Option<(u32, Vec<AnswerPiece>)> {
    let (form, hit) = counts.first_most()?;
    (f64::from(*hit) >= SURE_SHARE * f64::from(lessons) && form.contains(&AnswerPiece::Value)).then(|| (*hit, form.clone()))
}
because!(sure_form, GraphProperty, "the form of the answers of a property that is a whole number, for one form of question: when nearly every lesson of the question form has it and it holds the value; with the lessons that it gives right");

pub fn learn_graph(lessons: Lessons, _lanes: usize) -> Learned {
    let stories: Vec<(Vec<Vec<String>>, Vec<String>)> = lessons.cases.iter().map(|case| (case.sentences.iter().map(|sentence| words(sentence)).collect(), words(&case.question))).collect();
    let Some(shape) = LinkShape::learn(&stories) else { return Err(NO_GRAPH.to_string()) };
    let mut truths: FirstSightTable<String, Vec<FirstSightTable<bool, FirstSightTable<Vec<String>, u32>>>> = FirstSightTable::new();
    let mut amounts: FirstSightTable<String, Vec<FirstSightTable<Vec<AnswerPiece>, u32>>> = FirstSightTable::new();
    let mut forms: FirstSightTable<String, u32> = FirstSightTable::new();
    for case in lessons.cases {
        let Some(read) = read_case(&shape, &case.sentences, &case.question) else { continue };
        let said = answer_words(&case.answer);
        forms.add(read.form.clone(), 1);
        let row = truths.entry(read.form.clone(), vec![FirstSightTable::new(); TRUTHS.len()]);
        for (place, wanted) in TRUTHS.iter().enumerate() {
            if let Some(value) = truth(&read.graph, &read.asked, *wanted) {
                row[place].entry(value, FirstSightTable::new()).add(said.clone(), 1);
            }
        }
        let row = amounts.entry(read.form.clone(), vec![FirstSightTable::new(); AMOUNTS.len()]);
        for (place, wanted) in AMOUNTS.iter().enumerate() {
            if let Some(value) = graph_amount(&read.graph, &read.asked, *wanted) {
                row[place].add(pieces(&said, &read, value), 1);
            }
        }
    }
    let mut asks: FirstSightTable<String, GraphAsk> = FirstSightTable::new();
    for (form, lessons_of) in forms.rows() {
        let mut best: Option<(u32, GraphAsk)> = None;
        for (place, wanted) in TRUTHS.iter().enumerate() {
            if let Some((covered, answers)) = truths.get(form).and_then(|row| sure_answers(&row[place]))
                && best.as_ref().is_none_or(|(top, _)| covered > *top)
            {
                best = Some((covered, GraphAsk::Truth(*wanted, answers)));
            }
        }
        for (place, wanted) in AMOUNTS.iter().enumerate() {
            if let Some((covered, form_of)) = amounts.get(form).and_then(|row| sure_form(&row[place], *lessons_of))
                && best.as_ref().is_none_or(|(top, _)| covered > *top)
            {
                best = Some((covered, GraphAsk::Amount(*wanted, form_of)));
            }
        }
        if let Some((_, asked)) = best {
            asks.put(form.clone(), asked);
        }
    }
    if asks.rows().is_empty() {
        return Err(NO_GRAPH.to_string());
    }
    Ok(Box::new(GraphModel { shape, asks, default: frequent_answer(lessons.cases).0 }))
}
because!(learn_graph, LinkGraph, "the tool on lessons: the shape of a link sentence from the told sentences and the questions of all lessons, each lesson read as a graph with the form of its question, the answers counted for each value of each property that is true or false and the forms of the answers of each property that is a whole number, and for each form of question the property that gives the answers of the most lessons; the tool says that it does not read a kind where no form gets a property");

pub const GRAPH_TOOL: Tool = Tool { name: "graph", needs_labels: false, probe: false, asks: false, over: None, learn: learn_graph };
because!(GRAPH_TOOL, LinkGraph, "the tool in the list of the tools: it needs no labels and gives no control by the form of the question, because a property of the graph needs the told links");
