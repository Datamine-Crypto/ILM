use ilm_core::base::keys::NO_NUMBER;
use ilm_core::base::lanes::{hardware_lanes, on_lanes, shards};
use ilm_core::base::symbols::Symbols;
use ilm_core::base::words::words;
use ilm_core::cases::Case;
use patterns::{because, source};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::io::{BufWriter, Write};
use std::path::Path;

pub struct KindByCounts;
source!(KindByCounts, "the kind of a request is found by counting from the lessons of the kinds alone: for each kind, how often each word and each pair of words next to each other stands in the questions of its lessons and in their told sentences; a request goes to the kind with the highest sum of the logarithms of these counts, each count plus one, with the words and pairs of the question weighed as one whole and those of the told sentences as another, so that a long story does not outweigh its question; this is naive Bayes over the parts of a request, and no rule names a kind");

const START_WORD: &str = "<s>";
because!(START_WORD, KindByCounts, "the mark before the first word of a text, so that the first word makes a pair too; no word of a text holds these signs together");

const END_WORD: &str = "</s>";
because!(END_WORD, KindByCounts, "the mark after the last word of a text, so that the last word makes a pair too");

const WHOLE_PART: f64 = 1.0;
because!(WHOLE_PART, KindByCounts, "the weight of all words and pairs of one part of a request together: the question and the told sentences each weigh as one, however long each is");

const SHOWN_PAIRS: usize = 4;
because!(SHOWN_PAIRS, KindByCounts, "the pairs of words of a request that the trace names for the chosen kind: enough to see why it was chosen, and a line that a person still reads");

const ROUTE_KIND_LINE: &str = "kind";
because!(ROUTE_KIND_LINE, KindByCounts, "the mark of the line of a file of the router that starts a kind");

const ASKED_LINE: &str = "q";
because!(ASKED_LINE, KindByCounts, "the mark of a count of a word or pair of the questions of a kind");

const TOLD_LINE: &str = "s";
because!(TOLD_LINE, KindByCounts, "the mark of a count of a word or pair of the told sentences of a kind");

const STORY_WAY: &str = "story";
because!(STORY_WAY, KindByCounts, "the name of the way of a kind whose requests the tools that reason over a told story answer");

const FACT_WAY: &str = "fact";
because!(FACT_WAY, KindByCounts, "the name of the way of the kind whose requests the table of facts answers");

const REQUEST_WAY: &str = "request";
because!(REQUEST_WAY, KindByCounts, "the name of the way of a kind whose requests the reader of requests reads into their facts");

const ROUTE_FIELD: char = '\t';
because!(ROUTE_FIELD, KindByCounts, "the sign between the fields of a line of a file of the router");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Feature {
    pub told: bool,
    pub pair: bool,
    pub first: u32,
    pub second: u32,
}
because!(Feature, KindByCounts, "a word or a pair of words next to each other, of the question or of the told sentences, as numbers; a word that no lesson holds has no number and so matches no count. renumbered gives the same word or pair in other numbers, from a table of the old number to the new one");

impl Feature {
    pub fn renumbered(&self, numbers: &[u32]) -> Feature {
        Feature { told: self.told, pair: self.pair, first: numbers[self.first as usize], second: if self.pair { numbers[self.second as usize] } else { NO_NUMBER } }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Destination {
    Story,
    Fact,
    Request(String),
}
because!(Destination, KindByCounts, "where a request of a kind goes: to the tools that reason over a told story, to the table of facts, or to the reader of requests with the kind of the intents of its set");

#[derive(Clone, Debug)]
pub struct RouteKind {
    pub source: String,
    pub kind: String,
    pub task: String,
    pub fit: usize,
    pub held: usize,
    pub lessons: usize,
    pub way: Destination,
    pub counts: HashMap<Feature, u32>,
}
because!(RouteKind, KindByCounts, "one kind of the router: its set and its name, the task that it belongs to, the sizes of its parts, the number of lessons that were counted, where its requests go, and the counts of its words and pairs");

pub struct ReadOnThreads;
source!(ReadOnThreads, "a file of the router is read on all threads: its lines are taken apart in runs, the words of each kind are numbered in a table of the kind alone and then given their numbers of the whole in the order of the kinds and of first sight within each, which are the numbers that one pass in order gives, and the counts of the kinds are put in their tables side by side");

enum RouteLine<'a> {
    Kind(RouteKind),
    Count { told: bool, first: &'a str, second: &'a str, count: u32 },
    Other,
}
because!(RouteLine, ReadOnThreads, "a line of a file of the router taken apart: the start of a kind with its parts, a count of a word or pair, or a line that is neither");

fn route_line(line: &str) -> RouteLine<'_> {
    let fields: Vec<&str> = line.split(ROUTE_FIELD).collect();
    match fields.as_slice() {
        [ROUTE_KIND_LINE, source, kind, task, fit, held, lessons, way, intent] => {
            let way = match *way {
                FACT_WAY => Destination::Fact,
                REQUEST_WAY => Destination::Request(intent.to_string()),
                _ => Destination::Story,
            };
            let number = |text: &str| text.parse::<usize>().unwrap_or(0);
            RouteLine::Kind(RouteKind { source: source.to_string(), kind: kind.to_string(), task: task.to_string(), fit: number(fit), held: number(held), lessons: number(lessons), way, counts: HashMap::new() })
        }
        [mark, first, second, count] => RouteLine::Count { told: *mark == TOLD_LINE, first, second, count: count.parse::<u32>().unwrap_or(0) },
        _ => RouteLine::Other,
    }
}
because!(route_line, ReadOnThreads, "one line of a file of the router taken apart as the reader takes it: a kind with all its fields, or a count with its mark, its two words and the count");

type KindRows = (Symbols, Vec<(Feature, u32)>);
because!(KindRows, ReadOnThreads, "the counts of one kind with its words numbered in a table of the kind alone: the table, and each word or pair in the numbers of that table with its count");

pub struct Router {
    pub words: Symbols,
    pub kinds: Vec<RouteKind>,
    none: Vec<(f64, f64)>,
}
because!(Router, KindByCounts, "the router: the words of all kinds as numbers, the kinds in the order of their set and name, and for each kind the logarithm of the share that a word or pair has when the kind never holds it, for the question and for the told sentences. read reads it from its file on all threads; route gives every kind with its score, the best first; why names the pairs of a request that a kind holds most often");

fn text_features(numbers: &[u32], told: bool, start: u32, end: u32, into: &mut Vec<Feature>) {
    into.extend(numbers.iter().map(|word| Feature { told, pair: false, first: *word, second: NO_NUMBER }));
    let before = std::iter::once(start).chain(numbers.iter().copied());
    let after = numbers.iter().copied().chain(std::iter::once(end));
    into.extend(before.zip(after).map(|(first, second)| Feature { told, pair: true, first, second }));
}
because!(text_features, KindByCounts, "the words of a text and then its pairs of words next to each other, with the marks of the start and the end, in their order");

pub fn request_features(number: &mut dyn FnMut(&str) -> u32, sentences: &[String], question: &str) -> (Vec<Feature>, Vec<Feature>) {
    let (start, end) = (number(START_WORD), number(END_WORD));
    let mut asked: Vec<Feature> = Vec::new();
    let asked_words: Vec<u32> = words(question).iter().map(|word| number(word)).collect();
    text_features(&asked_words, false, start, end, &mut asked);
    let mut told: Vec<Feature> = Vec::new();
    for sentence in sentences {
        let told_words: Vec<u32> = words(sentence).iter().map(|word| number(word)).collect();
        text_features(&told_words, true, start, end, &mut told);
    }
    (asked, told)
}
because!(request_features, KindByCounts, "the words and pairs of the question and those of the told sentences of a request, each part in its order; the caller gives the numbers of the words, new numbers while lessons are counted and no number for an unknown word while a request is routed");

pub fn count_cases(words_table: &mut Symbols, cases: &[Case], questions_only: bool) -> HashMap<Feature, u32> {
    let mut counts: HashMap<Feature, u32> = HashMap::new();
    for case in cases {
        let sentences: &[String] = if questions_only { &[] } else { &case.sentences };
        let (asked, told) = request_features(&mut |word| words_table.number(word), sentences, &case.question);
        for feature in asked.into_iter().chain(told) {
            *counts.entry(feature).or_insert(0) += 1;
        }
    }
    counts
}
because!(count_cases, KindByCounts, "the counts of the words and pairs of some lessons; for a kind whose requests are questions alone the told sentences are not counted");

pub fn count_requests(words_table: &mut Symbols, cases: &[Case]) -> (HashMap<Feature, u32>, usize) {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut texts: Vec<&str> = Vec::new();
    for sentence in cases.iter().flat_map(|case| case.sentences.iter()) {
        if seen.insert(sentence.as_str()) {
            texts.push(sentence.as_str());
        }
    }
    let mut counts: HashMap<Feature, u32> = HashMap::new();
    for text in &texts {
        let (asked, _) = request_features(&mut |word| words_table.number(word), &[], text);
        for feature in asked {
            *counts.entry(feature).or_insert(0) += 1;
        }
    }
    (counts, texts.len())
}
because!(count_requests, KindByCounts, "the counts of the words and pairs of the requests of a set to a service: each different request of the lessons once, read as a question, because a request comes as one sentence with no question");

impl Router {
    pub fn new(words: Symbols, mut kinds: Vec<RouteKind>) -> Router {
        kinds.sort_by(|one, other| (&one.source, &one.kind).cmp(&(&other.source, &other.kind)));
        let distinct = |told: bool| kinds.iter().flat_map(|kind| kind.counts.keys()).filter(|feature| feature.told == told).collect::<HashSet<&Feature>>().len() + 1;
        let (asked_size, told_size) = (distinct(false), distinct(true));
        let none = kinds
            .iter()
            .map(|kind| {
                let total = |told: bool| kind.counts.iter().filter(|(feature, _)| feature.told == told).map(|(_, count)| *count as usize).sum::<usize>();
                (-((total(false) + asked_size) as f64).ln(), -((total(true) + told_size) as f64).ln())
            })
            .collect();
        Router { words, kinds, none }
    }

    fn find(&self) -> impl FnMut(&str) -> u32 + '_ {
        |word| self.words.find(word).unwrap_or(NO_NUMBER)
    }

    pub fn route(&self, sentences: &[String], question: &str) -> Vec<(f64, usize)> {
        let (asked, told) = request_features(&mut self.find(), sentences, question);
        let asked_weight = WHOLE_PART / (asked.len().max(1) as f64);
        let told_weight = WHOLE_PART / (told.len().max(1) as f64);
        let mut scored: Vec<(f64, usize)> = self
            .kinds
            .iter()
            .enumerate()
            .map(|(place, kind)| {
                let (asked_none, told_none) = self.none[place];
                let mut score = (if asked.is_empty() { f64::default() } else { asked_none }) + (if told.is_empty() { f64::default() } else { told_none });
                for (part, weight) in [(&asked, asked_weight), (&told, told_weight)] {
                    for feature in part.iter() {
                        if let Some(count) = kind.counts.get(feature) {
                            score += f64::from(*count + 1).ln() * weight;
                        }
                    }
                }
                (score, place)
            })
            .collect();
        scored.sort_by(|one, other| other.0.partial_cmp(&one.0).unwrap_or(Ordering::Equal).then_with(|| (&self.kinds[other.1].source, &self.kinds[other.1].kind).cmp(&(&self.kinds[one.1].source, &self.kinds[one.1].kind))));
        scored
    }

    fn feature_words(&self, feature: &Feature) -> (&str, &str) {
        let name = |number: u32| if number == NO_NUMBER { "" } else { self.words.name(number) };
        (name(feature.first), name(feature.second))
    }

    pub fn why(&self, place: usize, sentences: &[String], question: &str) -> String {
        let (asked, told) = request_features(&mut self.find(), sentences, question);
        let pairs: HashSet<Feature> = asked.into_iter().chain(told).filter(|feature| feature.pair).collect();
        let mut rows: Vec<(u32, &Feature)> = pairs.iter().filter_map(|feature| self.kinds[place].counts.get(feature).map(|count| (*count, feature))).collect();
        rows.sort_by(|one, other| {
            other.0.cmp(&one.0).then_with(|| {
                let (mine, theirs) = (self.feature_words(one.1), self.feature_words(other.1));
                (other.1.told, theirs.0, theirs.1).cmp(&(one.1.told, mine.0, mine.1))
            })
        });
        let shown: Vec<String> = rows
            .iter()
            .take(SHOWN_PAIRS)
            .map(|(count, feature)| {
                let (first, second) = self.feature_words(feature);
                format!("\"{first} {second}\" in {} {count} times", if feature.told { "told sentences" } else { "questions" })
            })
            .collect();
        shown.join("; ")
    }

    pub fn write(&self, path: &Path) -> std::io::Result<()> {
        let mut out = BufWriter::new(std::fs::File::create(path)?);
        for kind in &self.kinds {
            let (way, intent) = match &kind.way {
                Destination::Story => (STORY_WAY, ""),
                Destination::Fact => (FACT_WAY, ""),
                Destination::Request(intent) => (REQUEST_WAY, intent.as_str()),
            };
            writeln!(out, "{ROUTE_KIND_LINE}{ROUTE_FIELD}{}{ROUTE_FIELD}{}{ROUTE_FIELD}{}{ROUTE_FIELD}{}{ROUTE_FIELD}{}{ROUTE_FIELD}{}{ROUTE_FIELD}{way}{ROUTE_FIELD}{intent}", kind.source, kind.kind, kind.task, kind.fit, kind.held, kind.lessons)?;
            let mut lines: Vec<String> = kind
                .counts
                .iter()
                .map(|(feature, count)| {
                    let (first, second) = self.feature_words(feature);
                    let mark = if feature.told { TOLD_LINE } else { ASKED_LINE };
                    format!("{mark}{ROUTE_FIELD}{first}{ROUTE_FIELD}{}{ROUTE_FIELD}{count}", if feature.pair { second } else { "" })
                })
                .collect();
            lines.sort_unstable();
            for line in lines {
                writeln!(out, "{line}")?;
            }
        }
        out.flush()
    }

    pub fn read(path: &Path) -> std::io::Result<Router> {
        let text = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = text.lines().collect();
        let lanes = hardware_lanes();
        let pieces = shards(lines.len(), lanes);
        let parsed: Vec<Vec<RouteLine>> = on_lanes(pieces.len(), lanes, &|piece| lines[pieces[piece].clone()].iter().map(|line| route_line(line)).collect());
        let mut kinds: Vec<RouteKind> = Vec::new();
        let mut blocks: Vec<Vec<(bool, &str, &str, u32)>> = Vec::new();
        for line in parsed.into_iter().flatten() {
            match line {
                RouteLine::Kind(kind) => {
                    kinds.push(kind);
                    blocks.push(Vec::new());
                }
                RouteLine::Count { told, first, second, count } => {
                    if let Some(block) = blocks.last_mut() {
                        block.push((told, first, second, count));
                    }
                }
                RouteLine::Other => {}
            }
        }
        let local: Vec<KindRows> = on_lanes(blocks.len(), lanes, &|place| {
            let mut table = Symbols::new();
            let rows = blocks[place]
                .iter()
                .map(|(told, first, second, count)| {
                    let one = table.number(first);
                    let pair = !second.is_empty();
                    (Feature { told: *told, pair, first: one, second: if pair { table.number(second) } else { NO_NUMBER } }, *count)
                })
                .collect();
            (table, rows)
        });
        let mut words = Symbols::new();
        let numbers: Vec<Vec<u32>> = local.iter().map(|(table, _)| (0..table.len() as u32).map(|place| words.number(table.name(place))).collect()).collect();
        let counts: Vec<HashMap<Feature, u32>> = on_lanes(local.len(), lanes, &|place| local[place].1.iter().map(|(feature, count)| (feature.renumbered(&numbers[place]), *count)).collect());
        for (kind, held) in kinds.iter_mut().zip(counts) {
            kind.counts = held;
        }
        Ok(Router::new(words, kinds))
    }
}
