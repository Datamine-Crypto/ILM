use crate::base::first_sight::FirstSightTable;
use crate::base::symbols::Symbols;
use crate::base::words::{answer_words, words};
use crate::tools::call::{Learned, Learner, Lessons, Said, Tool};
use crate::tools::frequent::frequent_answer;
use crate::tools::pattern::model::SURE_SHARE;
use patterns::{because, source};

pub struct LinkGraph;
source!(LinkGraph, "a story read as a graph: of the told sentences of all lessons, those of the most frequent shape where exactly two places change from sentence to sentence and every other place keeps its word are links between the two things that stand in those places. A question is answered by a property of the graph, found with the disjoint sets of Galler and Fischer: whether the links close a loop, or whether a chain of links joins the two things that the question names; which property a form of question asks, and which answer each value of the property gives, are counted from the lessons");

pub struct GraphProperty;
source!(GraphProperty, "the properties of a graph that the tool reads, a small fixed set like the calculations of the amount tool: a loop among the links, and a chain between two named things");

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
}
because!(Property, GraphProperty, "a property of a graph: a loop among its links, or a chain of links between the two named things");

const PROPERTIES: &[Property] = &[Property::Loop, Property::Chain];
because!(PROPERTIES, GraphProperty, "the properties in the order in which they are tried; of two that give the answers equally well the first is taken");

struct Graph {
    links: Vec<(usize, usize)>,
    things: Symbols,
}
because!(Graph, LinkGraph, "the links of a story as pairs of numbers of things, and the text of each thing");

#[derive(Clone, Debug, Default)]
struct LinkShape {
    size: usize,
    fixed: Vec<(usize, String)>,
    open: Vec<usize>,
}
because!(LinkShape, LinkGraph, "the shape of a link sentence: its number of words, the word that stands at each place that never changes, and the two places that hold things. learn finds it from the told sentences of all lessons; links reads the links of a story with it");

impl LinkShape {
    fn learn(stories: &[Vec<Vec<String>>]) -> Option<LinkShape> {
        let mut by_size: FirstSightTable<usize, Vec<&Vec<String>>> = FirstSightTable::new();
        for said in stories.iter().flatten() {
            by_size.entry(said.len(), Vec::new()).push(said);
        }
        let mut best: Option<(usize, LinkShape)> = None;
        for (size, members) in by_size.rows() {
            if members.len() < LEAST_LINKS {
                continue;
            }
            let first = members[0];
            let open: Vec<usize> = (0..*size).filter(|at| members.iter().any(|member| member[*at] != first[*at])).collect();
            if open.len() == LINK_PLACES && best.as_ref().is_none_or(|(held, _)| members.len() > *held) {
                let fixed = (0..*size).filter(|at| !open.contains(at)).map(|at| (at, first[at].clone())).collect();
                best = Some((members.len(), LinkShape { size: *size, fixed, open }));
            }
        }
        best.map(|(_, shape)| shape)
    }

    fn links(&self, sentences: &[Vec<String>]) -> Option<Graph> {
        let mut things = Symbols::new();
        let mut links: Vec<(usize, usize)> = Vec::new();
        for said in sentences {
            if said.len() == self.size && self.fixed.iter().all(|(at, word)| said[*at] == *word) {
                links.push((things.number(&said[self.open[0]]) as usize, things.number(&said[self.open[1]]) as usize));
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

fn property(graph: &Graph, asked: &[usize], wanted: Property) -> Option<bool> {
    let mut parent: Vec<usize> = (0..graph.things.len()).collect();
    let mut looped = false;
    for (one, other) in &graph.links {
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
        (Property::Chain, _) => None,
    }
}
because!(property, LinkGraph, "the value of a property of a graph: the links are joined set by set, and a link whose things are in one set already closes a loop; two things are chained when they end in one set; nothing for a chain when the question does not name two things");

struct GraphRead {
    graph: Graph,
    form: String,
    asked: Vec<usize>,
}
because!(GraphRead, LinkGraph, "a case read for the tool: the graph of its story, the form of its question, and the things of the graph that the question names in its order");

fn read_case(shape: &LinkShape, sentences: &[String], question: &str) -> Option<GraphRead> {
    let told: Vec<Vec<String>> = sentences.iter().map(|sentence| words(sentence)).collect();
    let graph = shape.links(&told)?;
    let mut asked: Vec<usize> = Vec::new();
    let mut form: Vec<String> = Vec::new();
    for word in words(question) {
        match graph.things.find(&word) {
            Some(thing) => {
                asked.push(thing as usize);
                form.push(GRAPH_SLOT.to_string());
            }
            None => form.push(word),
        }
    }
    Some(GraphRead { graph, form: form.join(GRAPH_BETWEEN), asked })
}
because!(read_case, LinkGraph, "a case read for the tool, and nothing when its story has no links");

pub struct GraphModel {
    shape: LinkShape,
    asks: FirstSightTable<String, (Property, Vec<(bool, Vec<String>)>)>,
    default: Vec<String>,
}
because!(GraphModel, LinkGraph, "what the tool learned: the shape of a link sentence, for each form of question the property that it asks and the answer of each value, and the most frequent answer of the lessons");

impl Learner for GraphModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let none = || Said { words: self.default.clone(), note: NOTE_NO_GRAPH.to_string() };
        let Some(read) = read_case(&self.shape, sentences, question) else { return none() };
        let Some((wanted, answers)) = self.asks.get(&read.form) else { return none() };
        let Some(value) = property(&read.graph, &read.asked, *wanted) else { return none() };
        match answers.iter().find(|(held, _)| *held == value) {
            Some((_, said)) => Said { words: said.clone(), note: NOTE_GRAPH.to_string() },
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

pub fn learn_graph(lessons: Lessons, _lanes: usize) -> Learned {
    let stories: Vec<Vec<Vec<String>>> = lessons.cases.iter().map(|case| case.sentences.iter().map(|sentence| words(sentence)).collect()).collect();
    let Some(shape) = LinkShape::learn(&stories) else { return Err(NO_GRAPH.to_string()) };
    let mut counts: FirstSightTable<String, Vec<FirstSightTable<bool, FirstSightTable<Vec<String>, u32>>>> = FirstSightTable::new();
    for case in lessons.cases {
        let Some(read) = read_case(&shape, &case.sentences, &case.question) else { continue };
        let row = counts.entry(read.form.clone(), vec![FirstSightTable::new(); PROPERTIES.len()]);
        for (place, wanted) in PROPERTIES.iter().enumerate() {
            if let Some(value) = property(&read.graph, &read.asked, *wanted) {
                row[place].entry(value, FirstSightTable::new()).add(answer_words(&case.answer), 1);
            }
        }
    }
    let mut asks: FirstSightTable<String, (Property, Vec<(bool, Vec<String>)>)> = FirstSightTable::new();
    for (form, by_property) in counts.rows() {
        let mut best: Option<(u32, Property, Vec<(bool, Vec<String>)>)> = None;
        for (place, wanted) in PROPERTIES.iter().enumerate() {
            if let Some((covered, answers)) = sure_answers(&by_property[place])
                && best.as_ref().is_none_or(|(top, _, _)| covered > *top)
            {
                best = Some((covered, *wanted, answers));
            }
        }
        if let Some((_, wanted, answers)) = best {
            asks.put(form.clone(), (wanted, answers));
        }
    }
    if asks.rows().is_empty() {
        return Err(NO_GRAPH.to_string());
    }
    Ok(Box::new(GraphModel { shape, asks, default: frequent_answer(lessons.cases).0 }))
}
because!(learn_graph, LinkGraph, "the tool on lessons: the shape of a link sentence from the told sentences of all lessons, each lesson read as a graph with the form of its question, the answers counted for each value of each property, and for each form the property whose values give the answers of the most lessons; the tool says that it does not read a kind where no form gets a property");

pub const GRAPH_TOOL: Tool = Tool { name: "graph", needs_labels: false, probe: false, asks: false, over: None, learn: learn_graph };
because!(GRAPH_TOOL, LinkGraph, "the tool in the list of the tools: it needs no labels and gives no control by the form of the question, because a property of the graph needs the told links");
