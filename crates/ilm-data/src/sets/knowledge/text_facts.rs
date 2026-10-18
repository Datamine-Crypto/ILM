use ilm_core::base::lanes::on_lanes;
use ilm_core::base::signs::{is_space, is_word_sign};
use ilm_core::base::words::answer_words;
use ilm_core::cases::Case;
use ilm_core::classes::WordClasses;
use ilm_core::tools::call::Lessons;
use ilm_core::tools::explain::learn::{LIBRARY_FOLDER, LINES_PRINT_MARK, Library, learn_explain_with, lines_print, read_library};
use ilm_core::tools::explain::text::fact_numbers;
use ilm_core::tools::explain::words::word_texts;
use crate::sets::knowledge::letters::letters;
use patterns::{because, source};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub struct TextFacts;
source!(TextFacts, "the facts of a knowledge text of the library found in a large text that people wrote, such as an encyclopedia, with no sentence shape that a person wrote: the text's own form of its facts finds the first facts, the facts that the proofs of the lessons confirm become seeds, the sentences that hold both names of a seed give the shapes of the relation, a shape is kept by how it does on the seeds, the kept shapes find more facts, and the proofs of the lessons judge them again; each fact keeps the files and lines of the sentences that gave it");

pub struct PatternRelation;
source!(PatternRelation, "a relation and the sentence shapes that state it are found from each other over a large text: a few known pairs give the shapes of the sentences that hold both of their names, and the shapes give new pairs (the dual iterative pattern relation expansion of Brin)");

pub struct MentionMargin;
source!(MentionMargin, "a value of a relation that holds one partner (the capital of a country) is taken from a large text only when the text names one partner far more often than any other: a partner named at least twice, and at least twice as often as the next");

pub struct DiscriminatingSeeds;
source!(DiscriminatingSeeds, "a shape that only follows the name a text writes most beside a key (a city named beside its country) is precise on most seeds, as the most named partner is often the true one; it fails on the seeds whose true partner is not the most named, so a shape is judged on those seeds too");

const MOST_BETWEEN: usize = 7;
because!(MOST_BETWEEN, PatternRelation, "the most words between the two names of a sentence that a shape holds: a shape states the relation in a few words, and two names far apart in a sentence are seldom one fact");

const LEAST_SEED_PAIRS: usize = 2;
because!(LEAST_SEED_PAIRS, PatternRelation, "the seeds whose sentences a shape must hold before it is tried: a shape of one seed alone is the wording of one sentence, not of the relation");

const LEAST_KNOWN: usize = 3;
because!(LEAST_KNOWN, PatternRelation, "the sentences of seeds that a shape must find right before it is kept, so that its share right is not one or two sentences by chance");

const SHAPE_PERCENT: usize = 90;
because!(SHAPE_PERCENT, PatternRelation, "the share of its sentences on seeds that a kept shape must find right, in hundredths: a shape of the relation is nearly always right, a shape of names that stand side by side for any reason is often wrong");

const WHOLE_PERCENT: usize = 100;
because!(WHOLE_PERCENT, PatternRelation, "shares are counted in hundredths, so that a share and its bar are compared in whole numbers");

const LEAST_MENTIONS: usize = 2;
because!(LEAST_MENTIONS, MentionMargin, "the sentences that must name a partner before it is taken: one sentence can be a slip");

const MARGIN_TIMES: usize = 2;
because!(MARGIN_TIMES, MentionMargin, "how many times more often the text must name the partner taken than the next partner");

const ROUNDS: usize = 2;
because!(ROUNDS, PatternRelation, "the rounds of shapes after the first facts: the second round learns the shapes that the seeds of the first round did not hold");

const ARTICLE: &str = "the";
because!(ARTICLE, TextFacts, "the word that a text may write before a name (the Netherlands), which a lesson leaves out (lives in Netherlands)");

const ARTICLE_PERCENT: usize = 50;
because!(ARTICLE_PERCENT, TextFacts, "a name takes the word before it as its own when more than half of the times the text names it, in hundredths: then a shape reads the name with that word, and other names without it");

const SOURCE_PLACES: usize = 3;
because!(SOURCE_PLACES, TextFacts, "the sentences of a fact whose file and line its note names: enough to find the fact in the text again, few enough for one line");

const READ_ROOM: usize = 1_048_576;
because!(READ_ROOM, TextFacts, "the bytes that the reader of a file of the text takes at a time");

const SENTENCE_ENDS: &[char] = &['.', '!', '?'];
because!(SENTENCE_ENDS, TextFacts, "a sentence of the text ends with one of these signs when space and a capital letter follow");

const WORD_INSIDE: &[char] = &['\'', '-'];
because!(WORD_INSIDE, TextFacts, "the signs that may stand inside a word of the text, after its first letter (Port-au-Prince, Côte d'Ivoire)");

const SENTENCE_EDGE: &str = "";
because!(SENTENCE_EDGE, TextFacts, "what stands for the word before the first name or after the second when the sentence starts or ends there");

const SHAPE_GAP: &str = "\t";
because!(SHAPE_GAP, TextFacts, "the sign between the parts of the key of a shape, a sign that no word of the text holds");

const LINE_END: u8 = b'\n';
because!(LINE_END, TextFacts, "the byte that ends a line of a file of the text");

const TOKEN_GAP: &str = " ";
because!(TOKEN_GAP, TextFacts, "the mark between the words of a name and between the words of a shape as they are kept and shown");

const EDGE_SHOWN: &str = "|";
because!(EDGE_SHOWN, TextFacts, "how a shape shows the start or the end of its sentence");

const NOTE_START: &str = "Note: ";
because!(NOTE_START, TextFacts, "the start of a note in the learned text, which the library reads as no fact");

const FACT_END: &str = ".";
because!(FACT_END, TextFacts, "the point that ends a fact of the learned text");

const FIRST_PLACE: usize = 0;
because!(FIRST_PLACE, TextFacts, "the place of the first name of a fact of the form, as the form writes it");

const SECOND_PLACE: usize = 1;
because!(SECOND_PLACE, TextFacts, "the place of the second name of a fact of the form");

const PLACES: usize = 2;
because!(PLACES, TextFacts, "the names that a fact of the form holds, one for each place of the relation that the form states");

const SHAPES_SHOWN: usize = 12;
because!(SHAPES_SHOWN, TextFacts, "the kept shapes that the report of a round names, the most used first");

type Pair = (String, String);
because!(Pair, TextFacts, "the two names of a fact, in the order of the form");

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Shape {
    pub first: usize,
    pub before: String,
    pub middle: String,
    pub after: String,
}
because!(Shape, PatternRelation, "the shape of a sentence that states a fact: which name of the form comes first, the word before the first name, the words between the two names, and the word after the second; a shape from its key and the name that comes first, its key again, and the shape as a person reads it: A and B for the first and second names of the form, a bar for the start or end of the sentence");

pub struct FactForm {
    pub head: Vec<String>,
    pub middle: Vec<String>,
    pub tail: Vec<String>,
}
because!(FactForm, TextFacts, "the own form of the facts of a text: the words that every fact holds between its two names (is the capital of), and the words before the first name and after the second that every fact holds, often none; a fact of the form written as a line of the library, its first letter a capital, ended by a point; and the form as a shape whose words before and after any sentence may hold");

pub fn fact_form(facts: &[Vec<String>]) -> Option<FactForm> {
    let shortest = facts.iter().min_by_key(|fact| fact.len())?;
    let holds = |words: &[String]| facts.iter().all(|fact| fact.windows(words.len()).any(|stretch| stretch == words));
    let mut best: Option<&[String]> = None;
    for from in 0..shortest.len() {
        for to in from + 1..=shortest.len() {
            let words = &shortest[from..to];
            if best.is_none_or(|known| words.len() > known.len()) && holds(words) {
                best = Some(words);
            }
        }
    }
    let middle = best?.to_vec();
    let split = |fact: &Vec<String>| -> Option<(Vec<String>, Vec<String>)> {
        let at = fact.windows(middle.len()).position(|stretch| stretch == middle.as_slice())?;
        Some((fact[..at].to_vec(), fact[at + middle.len()..].to_vec()))
    };
    let parts: Vec<(Vec<String>, Vec<String>)> = facts.iter().map(split).collect::<Option<Vec<_>>>()?;
    if parts.iter().any(|(left, right)| left.is_empty() || right.is_empty()) {
        return None;
    }
    Some(FactForm { head: Vec::new(), middle, tail: Vec::new() })
}
because!(fact_form, TextFacts, "the own form of the facts of a text of two names: the longest run of words that every fact holds, with a name on each side of it in every fact; nothing when the facts share no such run");

impl FactForm {
    pub fn line(&self, pair: &Pair) -> String {
        let mut words: Vec<&str> = self.head.iter().map(String::as_str).collect();
        words.push(&pair.0);
        words.extend(self.middle.iter().map(String::as_str));
        words.push(&pair.1);
        words.extend(self.tail.iter().map(String::as_str));
        let mut line = words.join(TOKEN_GAP);
        if let Some(first) = line.chars().next() {
            line = first.to_uppercase().chain(line.chars().skip(1)).collect();
        }
        format!("{line}{FACT_END}")
    }

    fn shape(&self) -> Shape {
        Shape { first: FIRST_PLACE, before: String::new(), middle: self.middle.join(TOKEN_GAP), after: String::new() }
    }
}

pub fn corpus_sentences(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut from = 0usize;
    let mut previous: Option<char> = None;
    let mut signs = line.char_indices().peekable();
    while let Some((at, sign)) = signs.next() {
        if is_space(sign) && previous.is_some_and(|last| SENTENCE_ENDS.contains(&last)) {
            let mut next_from = at + sign.len_utf8();
            while let Some(&(later, next)) = signs.peek() {
                if !is_space(next) {
                    break;
                }
                next_from = later + next.len_utf8();
                signs.next();
            }
            if signs.peek().is_some_and(|(_, next)| next.is_uppercase()) {
                out.push(&line[from..at]);
                from = next_from;
            }
            previous = Some(sign);
            continue;
        }
        previous = Some(sign);
    }
    out.push(&line[from..]);
    out.into_iter().map(|piece| piece.trim_matches(is_space)).filter(|piece| !piece.is_empty()).collect()
}
because!(corpus_sentences, TextFacts, "the sentences of a line of the text: it is cut where space follows the end of a sentence and a capital letter follows the space");

pub fn corpus_tokens(sentence: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (at, sign) in sentence.char_indices() {
        if is_word_sign(sign) || (start.is_some() && WORD_INSIDE.contains(&sign)) {
            start.get_or_insert(at);
            continue;
        }
        if let Some(from) = start.take() {
            out.push(&sentence[from..at]);
        }
        if !is_space(sign) {
            out.push(&sentence[at..at + sign.len_utf8()]);
        }
    }
    if let Some(from) = start {
        out.push(&sentence[from..]);
    }
    out
}
because!(corpus_tokens, TextFacts, "the words and the marks of a sentence in order: a word starts with a letter or a digit and goes on with letters, digits and the signs inside a word; every other sign that is not space is a token of its own");

fn capital(token: &str) -> bool {
    token.chars().next().is_some_and(char::is_uppercase)
}
because!(capital, TextFacts, "whether a token starts with a capital letter, as a name does");

pub fn name_runs(tokens: &[&str], joins: &HashSet<String>) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut at = 0usize;
    while at < tokens.len() {
        if !capital(tokens[at]) {
            at += 1;
            continue;
        }
        let mut end = at + 1;
        loop {
            if end < tokens.len() && capital(tokens[end]) {
                end += 1;
            } else if end + 1 < tokens.len() && joins.contains(tokens[end]) && capital(tokens[end + 1]) {
                end += PLACES;
            } else {
                break;
            }
        }
        runs.push((at, end));
        at = end;
    }
    runs
}
because!(name_runs, TextFacts, "the names of a sentence: each run of words with a capital letter, with the small words that the names of the lessons hold between two such words (United States of America), as the first and the end of its tokens");

pub fn lesson_joins(cases: &[&Case]) -> HashSet<String> {
    let mut joins = HashSet::new();
    for case in cases {
        for text in case.sentences.iter().chain(std::iter::once(&case.question)) {
            let tokens = corpus_tokens(text);
            for at in 1..tokens.len().saturating_sub(1) {
                if !capital(tokens[at]) && tokens[at].chars().all(char::is_alphabetic) && capital(tokens[at - 1]) && capital(tokens[at + 1]) {
                    joins.insert(tokens[at].to_string());
                }
            }
        }
    }
    joins
}
because!(lesson_joins, TextFacts, "the small words that stand between two words with a capital letter in the told sentences and questions of the lessons, as the names of the lessons join their words with them");

struct Mention {
    name: String,
    article: bool,
}
because!(Mention, TextFacts, "a name of a sentence: its words, and whether the word before it is the article");

fn mentions(tokens: &[&str], runs: &[(usize, usize)]) -> Vec<Mention> {
    runs.iter().map(|(start, end)| Mention { name: tokens[*start..*end].join(TOKEN_GAP), article: *start > 0 && tokens[*start - 1] == ARTICLE }).collect()
}
because!(mentions, TextFacts, "the names of a sentence, each with whether the article stands before it");

struct Framed {
    key: String,
    first_bare: bool,
    second_bare: bool,
}
because!(Framed, TextFacts, "the key of the words around two names of a sentence, and for each name whether the article before it was read as part of the name");

fn frames(tokens: &[&str], runs: &[(usize, usize)], one: usize, two: usize, bare: &dyn Fn(&str) -> bool) -> Vec<Framed> {
    let (first, second) = (runs[one], runs[two]);
    let between = &tokens[first.1..second.0];
    if between.is_empty() || between.len() > MOST_BETWEEN {
        return Vec::new();
    }
    let first_name = tokens[first.0..first.1].join(TOKEN_GAP);
    let second_name = tokens[second.0..second.1].join(TOKEN_GAP);
    let after = if second.1 < tokens.len() { tokens[second.1] } else { SENTENCE_EDGE };
    let mut out = Vec::new();
    for first_bare in [false, true] {
        let before = match (first_bare, first.0) {
            (false, start) if start > 0 => tokens[start - 1],
            (false, _) => SENTENCE_EDGE,
            (true, start) if start > 0 && tokens[start - 1] == ARTICLE && bare(&first_name) => {
                if start > 1 { tokens[start - PLACES] } else { SENTENCE_EDGE }
            }
            (true, _) => continue,
        };
        for second_bare in [false, true] {
            let middle = if second_bare {
                match between.split_last() {
                    Some((last, rest)) if *last == ARTICLE && !rest.is_empty() && bare(&second_name) => rest,
                    _ => continue,
                }
            } else {
                between
            };
            out.push(Framed { key: frame_key(before, &middle.join(TOKEN_GAP), after), first_bare, second_bare });
        }
    }
    out
}
because!(frames, TextFacts, "the keys of the words around two names of a sentence that stand a few words apart, at most as many as a shape may hold: the plain key, and when the article stands before a name that takes it, the key that reads the article as part of that name");

fn frame_key(before: &str, middle: &str, after: &str) -> String {
    [before, middle, after].join(SHAPE_GAP)
}
because!(frame_key, TextFacts, "the key of a shape's words: the word before, the words between and the word after, with a sign between that no word holds");

fn middle_key(middle: &str) -> String {
    format!("{SHAPE_GAP}{middle}{SHAPE_GAP}")
}
because!(middle_key, TextFacts, "the key of the words between the two names alone, by which the own form of the facts is found in any sentence");

fn key_middle(key: &str) -> &str {
    key.split(SHAPE_GAP).nth(1).unwrap_or_default()
}
because!(key_middle, TextFacts, "the words between the two names of a key");

impl Shape {
    fn from_key(first: usize, key: &str) -> Shape {
        let mut parts = key.split(SHAPE_GAP);
        let before = parts.next().unwrap_or_default().to_string();
        let middle = parts.next().unwrap_or_default().to_string();
        let after = parts.next().unwrap_or_default().to_string();
        Shape { first, before, middle, after }
    }

    fn key(&self) -> String {
        frame_key(&self.before, &self.middle, &self.after)
    }

    pub fn shown(&self) -> String {
        let edge = |word: &str| if word.is_empty() { EDGE_SHOWN.to_string() } else { word.to_string() };
        let (one, two) = if self.first == FIRST_PLACE { ("A", "B") } else { ("B", "A") };
        format!("{} {one} {} {two} {}", edge(&self.before), self.middle, edge(&self.after))
    }
}

struct ShapeTable {
    framed: HashMap<String, Vec<(usize, usize)>>,
    open: HashMap<String, Vec<(usize, usize)>>,
}
because!(ShapeTable, PatternRelation, "the shapes that a pass looks for, by the key of their words, each with its number and the name of the form that comes first; the own form by the words between alone; the shapes that the key of two names of a sentence fits, the own form among them when the words between fit it");

impl ShapeTable {
    fn of(shapes: &[Shape], open: Option<&Shape>) -> ShapeTable {
        let mut table = ShapeTable { framed: HashMap::new(), open: HashMap::new() };
        for (at, shape) in shapes.iter().enumerate() {
            table.framed.entry(shape.key()).or_default().push((at, shape.first));
        }
        if let Some(form) = open {
            table.open.entry(middle_key(&form.middle)).or_default().push((shapes.len(), form.first));
        }
        table
    }

    fn found(&self, key: &str) -> impl Iterator<Item = &(usize, usize)> {
        let open = self.open.get(&middle_key(key_middle(key))).into_iter().flatten();
        self.framed.get(key).into_iter().flatten().chain(open)
    }
}

fn corpus_files(folder: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(folder)
        .map_err(|error| format!("{}: {error}", folder.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    Ok(files)
}
because!(corpus_files, TextFacts, "the files of the large text, in the order of their names");

fn each_sentence(place: &Path, visit: &mut dyn FnMut(&[&str], usize)) -> Result<(), String> {
    let file = std::fs::File::open(place).map_err(|error| format!("{}: {error}", place.display()))?;
    let mut reader = BufReader::with_capacity(READ_ROOM, file);
    let mut bytes: Vec<u8> = Vec::new();
    let mut line_number = 0usize;
    loop {
        bytes.clear();
        let read = reader.read_until(LINE_END, &mut bytes).map_err(|error| format!("{}: {error}", place.display()))?;
        if read == 0 {
            return Ok(());
        }
        line_number += 1;
        let line = String::from_utf8_lossy(&bytes);
        for sentence in corpus_sentences(&line) {
            visit(&corpus_tokens(sentence), line_number);
        }
    }
}
because!(each_sentence, TextFacts, "every sentence of a file of the text as its tokens, with the number of its line; a sign that is not plain text is read as the sign of a fault, as the text is the work of many hands");

fn file_name(path: &Path) -> String {
    path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}
because!(file_name, TextFacts, "the name of a file of the text, as a note of a fact names it");

#[derive(Default)]
struct SeedShapes {
    held: HashMap<(usize, String, bool, bool), BTreeSet<usize>>,
    named: HashMap<String, (usize, usize)>,
}
because!(SeedShapes, PatternRelation, "what the sentences of one file give the shapes: for each key of the words around two names of a seed, with the name that comes first and whether the article was read as part of each name, the seeds whose sentences hold it; and for each name of a seed, how often the file names it and how often with the article before it");

fn seed_shapes(path: &Path, seeds: &[Pair], joins: &HashSet<String>) -> Result<SeedShapes, String> {
    let mut by_name: HashMap<&str, Vec<(usize, usize)>> = HashMap::new();
    for (at, (one, two)) in seeds.iter().enumerate() {
        by_name.entry(one.as_str()).or_default().push((at, FIRST_PLACE));
        by_name.entry(two.as_str()).or_default().push((at, SECOND_PLACE));
    }
    let mut found = SeedShapes::default();
    each_sentence(path, &mut |tokens, _| {
        let runs = name_runs(tokens, joins);
        if runs.len() < PLACES {
            return;
        }
        let names = mentions(tokens, &runs);
        let seen: Vec<Option<&Vec<(usize, usize)>>> = names.iter().map(|mention| by_name.get(mention.name.as_str())).collect();
        for (mention, known) in names.iter().zip(&seen) {
            if known.is_some() {
                let counts = found.named.entry(mention.name.clone()).or_default();
                counts.0 += 1;
                counts.1 += usize::from(mention.article);
            }
        }
        for one in 0..runs.len() {
            let Some(left) = seen[one] else { continue };
            for two in one + 1..runs.len() {
                let Some(right) = seen[two] else { continue };
                for (seed, place) in left {
                    if !right.iter().any(|(other, other_place)| other == seed && other_place != place) {
                        continue;
                    }
                    for framed in frames(tokens, &runs, one, two, &|_| true) {
                        found.held.entry((*place, framed.key, framed.first_bare, framed.second_bare)).or_default().insert(*seed);
                    }
                }
            }
        }
    })?;
    Ok(found)
}
because!(seed_shapes, PatternRelation, "the keys of the sentences of one file that hold both names of a seed, each with the seeds that hold it, read both with and without the article as part of a name; and how often the file names each name of a seed, and how often with the article");

fn takes_article(counts: Option<&(usize, usize)>) -> bool {
    counts.is_some_and(|(all, with)| *with * WHOLE_PERCENT > *all * ARTICLE_PERCENT)
}
because!(takes_article, TextFacts, "whether a name takes the article as its own: the text writes the article before it more than half of the times that it names it");

#[derive(Default)]
struct ShapeCounts {
    seeds: HashMap<(usize, usize), [usize; PLACES + 1]>,
    named: HashMap<(usize, usize), HashMap<String, usize>>,
}
because!(ShapeCounts, DiscriminatingSeeds, "what the sentences of one file give the judging of the shapes: for each shape and seed, the sentences where the shape finds the seed's pair, those where it finds another name beside the seed's first name, and those where it finds another name beside its second name; and for each seed and place of the form, the names that the shapes find beside the seed's name in the other place");

const KNOWN_AT: usize = 0;
because!(KNOWN_AT, DiscriminatingSeeds, "the place of the count of the sentences where a shape finds the seed's own pair");

fn shape_counts(path: &Path, (shapes, seeds): (&[Shape], &[Pair]), (joins, bare): (&HashSet<String>, &HashSet<String>)) -> Result<ShapeCounts, String> {
    let table = ShapeTable::of(shapes, None);
    let mut by_place: [HashMap<&str, usize>; PLACES] = [HashMap::new(), HashMap::new()];
    for (at, (one, two)) in seeds.iter().enumerate() {
        by_place[FIRST_PLACE].insert(one.as_str(), at);
        by_place[SECOND_PLACE].insert(two.as_str(), at);
    }
    let mut found = ShapeCounts::default();
    each_sentence(path, &mut |tokens, _| {
        let runs = name_runs(tokens, joins);
        if runs.len() < PLACES {
            return;
        }
        for one in 0..runs.len() {
            for two in one + 1..runs.len() {
                let names = [tokens[runs[one].0..runs[one].1].join(TOKEN_GAP), tokens[runs[two].0..runs[two].1].join(TOKEN_GAP)];
                for framed in frames(tokens, &runs, one, two, &|name| bare.contains(name)) {
                    for (shape, first) in table.found(&framed.key) {
                        let pair = if *first == FIRST_PLACE { [&names[0], &names[1]] } else { [&names[1], &names[0]] };
                        for place in [FIRST_PLACE, SECOND_PLACE] {
                            let Some(seed) = by_place[place].get(pair[place].as_str()) else { continue };
                            let other = PLACES - 1 - place;
                            let counts = found.seeds.entry((*shape, *seed)).or_default();
                            let own = if other == FIRST_PLACE { &seeds[*seed].0 } else { &seeds[*seed].1 };
                            if pair[other] == own {
                                if place == SECOND_PLACE {
                                    counts[KNOWN_AT] += 1;
                                }
                            } else {
                                counts[place + 1] += 1;
                            }
                            *found.named.entry((*seed, place)).or_default().entry(pair[other].clone()).or_default() += 1;
                        }
                    }
                }
            }
        }
    })?;
    Ok(found)
}
because!(shape_counts, DiscriminatingSeeds, "the counts of the shapes on the seeds in one file: every two names of a sentence that a shape fits, with the article read as part of a name only for a seed's name that takes it; a seed's pair counts once as found right, and another name beside a seed's name counts against the shape in the place of that name");

#[derive(Default)]
struct Harvest {
    counts: HashMap<(Pair, bool, bool), usize>,
    places: HashMap<Pair, Vec<(usize, usize)>>,
}
because!(Harvest, TextFacts, "what the kept shapes find in one file: for each pair and whether the article was read as part of each name, its sentences; and for each pair, the files and lines of its first sentences");

fn harvest(path: &Path, file: usize, (shapes, form): (&[Shape], &Shape), joins: &HashSet<String>) -> Result<Harvest, String> {
    let table = ShapeTable::of(shapes, Some(form));
    let mut found = Harvest::default();
    each_sentence(path, &mut |tokens, line| {
        let runs = name_runs(tokens, joins);
        if runs.len() < PLACES {
            return;
        }
        for one in 0..runs.len() {
            for two in one + 1..runs.len() {
                for framed in frames(tokens, &runs, one, two, &|_| true) {
                    let Some((_, first)) = table.found(&framed.key).next() else { continue };
                    let names = (tokens[runs[one].0..runs[one].1].join(TOKEN_GAP), tokens[runs[two].0..runs[two].1].join(TOKEN_GAP));
                    let (pair, bare) = if *first == FIRST_PLACE { (names, (framed.first_bare, framed.second_bare)) } else { ((names.1, names.0), (framed.second_bare, framed.first_bare)) };
                    let places = found.places.entry(pair.clone()).or_default();
                    if places.len() < SOURCE_PLACES {
                        places.push((file, line));
                    }
                    *found.counts.entry((pair, bare.0, bare.1)).or_default() += 1;
                }
            }
        }
    })?;
    Ok(found)
}
because!(harvest, TextFacts, "the pairs that the kept shapes and the own form find in one file: two names of a sentence count once for each way of reading the article that a shape fits, the first shape that fits deciding which name comes first");

fn article_counts(path: &Path, names: &HashSet<String>, joins: &HashSet<String>) -> Result<HashMap<String, (usize, usize)>, String> {
    let mut found: HashMap<String, (usize, usize)> = HashMap::new();
    each_sentence(path, &mut |tokens, _| {
        let runs = name_runs(tokens, joins);
        for mention in mentions(tokens, &runs) {
            if names.contains(&mention.name) {
                let counts = found.entry(mention.name).or_default();
                counts.0 += 1;
                counts.1 += usize::from(mention.article);
            }
        }
    })?;
    Ok(found)
}
because!(article_counts, TextFacts, "how often one file names each of some names, and how often with the article before it");

fn margin_pairs(counts: &BTreeMap<Pair, usize>, key: usize) -> BTreeSet<Pair> {
    let mut by_key: BTreeMap<String, Vec<(usize, &Pair)>> = BTreeMap::new();
    for (pair, count) in counts {
        let name = if key == FIRST_PLACE { &pair.0 } else { &pair.1 };
        by_key.entry(letters(name)).or_default().push((*count, pair));
    }
    let mut kept = BTreeSet::new();
    for partners in by_key.values_mut() {
        partners.sort_by(|one, other| other.0.cmp(&one.0).then_with(|| one.1.cmp(other.1)));
        let top = partners[0].0;
        let next = partners.get(1).map_or(0, |(count, _)| *count);
        if top >= LEAST_MENTIONS && top >= MARGIN_TIMES * next {
            kept.insert(partners[0].1.clone());
        }
    }
    kept
}
because!(margin_pairs, MentionMargin, "the pairs taken by the margin with the names of one place as keys: for each key, its partner named most often, when the text names it at least twice and at least twice as often as the next; names that differ in accents or capitals are one key");

fn folded_counts(counts: &BTreeMap<Pair, usize>) -> BTreeMap<Pair, usize> {
    let mut spelled: BTreeMap<Pair, Vec<(usize, &Pair)>> = BTreeMap::new();
    for (pair, count) in counts {
        spelled.entry((letters(&pair.0), letters(&pair.1))).or_default().push((*count, pair));
    }
    spelled
        .into_values()
        .map(|mut ways| {
            let all: usize = ways.iter().map(|(count, _)| *count).sum();
            ways.sort_by(|one, other| other.0.cmp(&one.0).then_with(|| one.1.cmp(other.1)));
            (ways[0].1.clone(), all)
        })
        .collect()
}
because!(folded_counts, MentionMargin, "the counts of pairs whose names differ only in accents or capitals summed, under the spelling that the text uses most");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum KeyPlace {
    One(usize),
    Both,
}
because!(KeyPlace, MentionMargin, "the place of the form whose names are the keys of the margin: the one where the seeds keep their pairs best, or both when the two keep as many");

fn margin_of(counts: &BTreeMap<Pair, usize>, key: KeyPlace) -> BTreeSet<Pair> {
    match key {
        KeyPlace::One(place) => margin_pairs(counts, place),
        KeyPlace::Both => margin_pairs(counts, FIRST_PLACE).intersection(&margin_pairs(counts, SECOND_PLACE)).cloned().collect(),
    }
}
because!(margin_of, MentionMargin, "the pairs that the margin takes with the keys of one place, or with the keys of both places at once");

fn key_place(counts: &BTreeMap<Pair, usize>, seeds: &[Pair]) -> KeyPlace {
    let folded: HashSet<Pair> = seeds.iter().map(|(one, two)| (letters(one), letters(two))).collect();
    let kept = |place: usize| margin_pairs(counts, place).iter().filter(|(one, two)| folded.contains(&(letters(one), letters(two)))).count();
    let (first, second) = (kept(FIRST_PLACE), kept(SECOND_PLACE));
    if first > second {
        KeyPlace::One(FIRST_PLACE)
    } else if second > first {
        KeyPlace::One(SECOND_PLACE)
    } else {
        KeyPlace::Both
    }
}
because!(key_place, MentionMargin, "the place of the keys of the margin: the place with which the margin keeps more of the seeds, both places when the two keep as many");

pub struct KindLessons<'a> {
    pub name: String,
    pub cases: Vec<&'a Case>,
    pub labels: Vec<Vec<String>>,
}
because!(KindLessons, TextFacts, "the lessons of a kind on which the facts are judged: its name, its cases and their label lines");

#[derive(Clone, Default)]
struct Judgment {
    right: usize,
    wrong: usize,
}
because!(Judgment, TextFacts, "how the proofs of the lessons use a fact: in how many answers that are right, and in how many that are wrong");

fn covered(case: &Case, (found, taken): (&HashSet<String>, &HashSet<String>), joins: &HashSet<String>) -> bool {
    let mut named = 0usize;
    for text in case.sentences.iter().chain(std::iter::once(&case.question)) {
        let tokens = corpus_tokens(text);
        for (start, end) in name_runs(&tokens, joins) {
            let name = letters(&tokens[start..end].join(TOKEN_GAP));
            if taken.contains(&name) {
                named += 1;
            } else if found.contains(&name) {
                return false;
            }
        }
    }
    named >= PLACES
}
because!(covered, TextFacts, "whether a lesson can judge the facts of a round: every name of it that the text's shapes found is a name of a fact that the margin took, and it holds two such names at least; a lesson that names a place whose fact the round lacks would be answered wrong for that lack alone, and the names that no shape found (the persons of a story) are left alone");

fn spelled_pair(pair: &Pair, spelled: &HashMap<String, String>) -> Pair {
    let way = |name: &String| spelled.get(&letters(name)).cloned().unwrap_or_else(|| name.clone());
    (way(&pair.0), way(&pair.1))
}
because!(spelled_pair, TextFacts, "a pair in the spelling of the lessons where they name its names (Bogota as Bogotá), else as the text writes it");

fn judged(topic: &str, form: &FactForm, candidates: &[Pair], (kinds, found, joins, spelled): (&[KindLessons], &HashSet<String>, &HashSet<String>, &HashMap<String, String>), (classes, lanes, kept): (Option<&WordClasses>, usize, &mut Vec<Arc<Library>>)) -> Result<(Vec<Judgment>, Vec<String>, usize), String> {
    let mut topics = read_library(Path::new(LIBRARY_FOLDER))?;
    let place = topics.iter().position(|text| text.name == topic).ok_or_else(|| format!("no text of the library has the name {topic}"))?;
    let lines: Vec<String> = candidates.iter().map(|pair| form.line(&spelled_pair(pair, spelled))).collect();
    topics[place].facts = lines.iter().map(|line| fact_numbers(line)).collect();
    let library = Arc::new(Library::of(topics)?);
    kept.push(Arc::clone(&library));
    let by_fact: HashMap<String, usize> = lines.iter().enumerate().map(|(at, line)| (word_texts(&fact_numbers(line)).join(TOKEN_GAP), at)).collect();
    let taken: HashSet<String> = candidates.iter().flat_map(|(one, two)| [letters(one), letters(two)]).collect();
    let mut judgments = vec![Judgment::default(); candidates.len()];
    let mut unread: Vec<String> = Vec::new();
    let mut used = 0usize;
    for kind in kinds {
        let chosen: Vec<usize> = (0..kind.cases.len()).filter(|at| covered(kind.cases[*at], (found, &taken), joins)).collect();
        used += chosen.len();
        let cases: Vec<Case> = chosen.iter().map(|at| kind.cases[*at].clone()).collect();
        let labels: Vec<Vec<String>> = if kind.labels.is_empty() { Vec::new() } else { chosen.iter().map(|at| kind.labels.get(*at).cloned().unwrap_or_default()).collect() };
        let lessons = Lessons { cases: &cases, labels: &labels, ask: None, classes, before: None };
        let model = match learn_explain_with(Arc::clone(&library), lessons, lanes) {
            Ok(model) => model,
            Err(fault) => {
                unread.push(format!("{} ({fault})", kind.name));
                continue;
            }
        };
        let read: Vec<(bool, Vec<String>)> = on_lanes(cases.len(), lanes, &|at| {
            let case = &cases[at];
            let right = model.answer(&case.sentences, &case.question).words == answer_words(&case.answer);
            (right, model.facts_used(&case.sentences, &case.question).into_iter().map(|(fact, _)| fact).collect())
        });
        for (right, used) in read {
            for fact in used {
                if let Some(at) = by_fact.get(&fact) {
                    if right {
                        judgments[*at].right += 1;
                    } else {
                        judgments[*at].wrong += 1;
                    }
                }
            }
        }
    }
    Ok((judgments, unread, used))
}
because!(judged, TextFacts, "the candidate facts judged by the proofs of the lessons: the library with the facts of the text in place of its own, each kind learned with it from its lessons that can judge the round, and for each fact the right and the wrong answers of those lessons whose proofs rest on it; each fact written as the lessons spell its names, so that a proof can rest on it; the kinds that cannot be learned with these facts, which judge none; and how many lessons judged; each library made stays kept until the step ends, as the engine keeps what it reads of a library's known facts by the place of those facts in memory, for a run whose library does not change");

fn lesson_spelling(cases: &[&Case], joins: &HashSet<String>) -> HashMap<String, String> {
    let mut spelled: HashMap<String, String> = HashMap::new();
    for case in cases {
        for text in case.sentences.iter().chain(std::iter::once(&case.question)) {
            let tokens = corpus_tokens(text);
            for (start, end) in name_runs(&tokens, joins) {
                let name = tokens[start..end].join(TOKEN_GAP);
                spelled.entry(letters(&name)).or_insert(name);
            }
        }
    }
    spelled
}
because!(lesson_spelling, TextFacts, "the names of the lessons by their plain letters, so that a fact found in the text is written as the lessons write its names (Bogota as Bogotá) when they name it");

pub struct LearnedFact {
    pub pair: Pair,
    pub sentences: usize,
    pub places: Vec<(String, usize)>,
    pub right: usize,
    pub round: usize,
}
because!(LearnedFact, TextFacts, "a fact that the step keeps: its names, the sentences of the text that gave it, the files and lines of its first sentences, the right answers of the lessons whose proofs rest on it, and the round that found it");

pub struct FactsReport {
    pub lines: Vec<String>,
    pub facts: Vec<LearnedFact>,
    pub form: FactForm,
    pub shapes: Vec<(Shape, usize, usize)>,
    pub seeds: usize,
    pub seeds_judged: bool,
    pub corpus: String,
    pub files: usize,
}
because!(FactsReport, TextFacts, "what the step reports: its lines, the facts it keeps, the own form, the shapes of the last round with their sentences right and wrong on the seeds, the seeds it started from and whether the lessons judged them before their shapes were learned, and the name and files of the text");

struct RoundHarvest {
    counts: BTreeMap<Pair, usize>,
    places: HashMap<Pair, Vec<(usize, usize)>>,
}
because!(RoundHarvest, TextFacts, "the pairs of one round with their sentences, the article read as part of a name only where the name takes it, and the first places of each pair");

fn round_harvest(files: &[PathBuf], (shapes, form): (&[Shape], &Shape), joins: &HashSet<String>, lanes: usize) -> Result<RoundHarvest, String> {
    let found: Vec<Harvest> = on_lanes(files.len(), lanes, &|at| harvest(&files[at], at, (shapes, form), joins)).into_iter().collect::<Result<Vec<Harvest>, String>>()?;
    let mut bare_names: HashSet<String> = HashSet::new();
    for file in &found {
        for ((pair, first_bare, second_bare), _) in &file.counts {
            if *first_bare {
                bare_names.insert(pair.0.clone());
            }
            if *second_bare {
                bare_names.insert(pair.1.clone());
            }
        }
    }
    let rates: Vec<HashMap<String, (usize, usize)>> = on_lanes(files.len(), lanes, &|at| article_counts(&files[at], &bare_names, joins)).into_iter().collect::<Result<Vec<_>, String>>()?;
    let mut named: HashMap<String, (usize, usize)> = HashMap::new();
    for file in rates {
        for (name, (all, with)) in file {
            let counts = named.entry(name).or_default();
            counts.0 += all;
            counts.1 += with;
        }
    }
    let mut counts: BTreeMap<Pair, usize> = BTreeMap::new();
    let mut places: HashMap<Pair, Vec<(usize, usize)>> = HashMap::new();
    for file in found {
        for ((pair, first_bare, second_bare), count) in file.counts {
            if (first_bare && !takes_article(named.get(&pair.0))) || (second_bare && !takes_article(named.get(&pair.1))) {
                continue;
            }
            *counts.entry(pair).or_default() += count;
        }
        for (pair, found_places) in file.places {
            let kept = places.entry(pair).or_default();
            kept.extend(found_places.into_iter().take(SOURCE_PLACES.saturating_sub(kept.len())));
        }
    }
    Ok(RoundHarvest { counts, places })
}
because!(round_harvest, TextFacts, "the pairs that the kept shapes find in all files, one file on each lane, summed in the order of the files: a pair read with the article as part of a name counts only when the text writes the article before that name more than half of the times");

fn bare_fits(frame: &str, left: bool, bare: bool, takes: bool) -> bool {
    let mut parts = frame.split(SHAPE_GAP);
    let before = parts.next().unwrap_or_default();
    let middle = parts.next().unwrap_or_default();
    let shown = if left { before == ARTICLE } else { middle.rsplit(TOKEN_GAP).next() == Some(ARTICLE) };
    if takes { bare || !shown } else { !bare }
}
because!(bare_fits, TextFacts, "whether a key reads the article before a name as the name takes it: a name that takes the article is read with it wherever the text writes it, and no other name is");

fn learned_shapes(files: &[PathBuf], seeds: &[Pair], (joins, key): (&HashSet<String>, KeyPlace), lanes: usize) -> Result<(Vec<(Shape, usize, usize)>, Vec<String>), String> {
    let found: Vec<SeedShapes> = on_lanes(files.len(), lanes, &|at| seed_shapes(&files[at], seeds, joins)).into_iter().collect::<Result<Vec<SeedShapes>, String>>()?;
    let mut named: HashMap<String, (usize, usize)> = HashMap::new();
    let mut held: BTreeMap<(usize, String, bool, bool), BTreeSet<usize>> = BTreeMap::new();
    for file in found {
        for (name, (all, with)) in file.named {
            let counts = named.entry(name).or_default();
            counts.0 += all;
            counts.1 += with;
        }
        for (shape_key, held_by) in file.held {
            held.entry(shape_key).or_default().extend(held_by);
        }
    }
    let bare: HashSet<String> = named.iter().filter(|(_, counts)| takes_article(Some(counts))).map(|(name, _)| name.clone()).collect();
    let keys_held = held.len();
    let mut candidates: BTreeMap<Shape, BTreeSet<usize>> = BTreeMap::new();
    for ((first, frame, first_bare, second_bare), held_by) in held {
        let fitting: BTreeSet<usize> = held_by
            .into_iter()
            .filter(|seed| {
                let (one, two) = &seeds[*seed];
                let (left, right) = if first == FIRST_PLACE { (one, two) } else { (two, one) };
                bare_fits(&frame, true, first_bare, bare.contains(left)) && bare_fits(&frame, false, second_bare, bare.contains(right))
            })
            .collect();
        if !fitting.is_empty() {
            candidates.entry(Shape::from_key(first, &frame)).or_default().extend(fitting);
        }
    }
    let tried: Vec<Shape> = candidates.into_iter().filter(|(_, held_by)| held_by.len() >= LEAST_SEED_PAIRS).map(|(shape, _)| shape).collect();
    let counted: Vec<ShapeCounts> = on_lanes(files.len(), lanes, &|at| shape_counts(&files[at], (&tried, seeds), (joins, &bare))).into_iter().collect::<Result<Vec<ShapeCounts>, String>>()?;
    let mut by_seed: HashMap<(usize, usize), [usize; PLACES + 1]> = HashMap::new();
    let mut partners: HashMap<(usize, usize), HashMap<String, usize>> = HashMap::new();
    for file in counted {
        for (shape_seed, counts) in file.seeds {
            let total = by_seed.entry(shape_seed).or_default();
            for (sum, add) in total.iter_mut().zip(counts) {
                *sum += add;
            }
        }
        for (seed_place, names) in file.named {
            let total = partners.entry(seed_place).or_default();
            for (name, count) in names {
                *total.entry(name).or_default() += count;
            }
        }
    }
    let top_partner = |seed: usize, place: usize| -> Option<String> {
        let names = partners.get(&(seed, place))?;
        names.iter().max_by(|one, other| one.1.cmp(other.1).then_with(|| other.0.cmp(one.0))).map(|(name, _)| name.clone())
    };
    let telling: BTreeSet<usize> = (0..seeds.len())
        .filter(|seed| top_partner(*seed, SECOND_PLACE).is_some_and(|name| name != seeds[*seed].0) || top_partner(*seed, FIRST_PLACE).is_some_and(|name| name != seeds[*seed].1))
        .collect();
    let against = |counts: &[usize; PLACES + 1]| match key {
        KeyPlace::One(place) => counts[place + 1],
        KeyPlace::Both => counts[FIRST_PLACE + 1] + counts[SECOND_PLACE + 1],
    };
    let precise = |(right, wrong): (usize, usize)| right * WHOLE_PERCENT >= (right + wrong) * SHAPE_PERCENT;
    let mut kept: Vec<(Shape, usize, usize)> = Vec::new();
    for (at, shape) in tried.iter().enumerate() {
        let mut all = (0usize, 0usize);
        let mut told = (0usize, 0usize);
        for seed in 0..seeds.len() {
            let Some(counts) = by_seed.get(&(at, seed)) else { continue };
            all.0 += counts[KNOWN_AT];
            all.1 += against(counts);
            if telling.contains(&seed) {
                told.0 += counts[KNOWN_AT];
                told.1 += against(counts);
            }
        }
        if all.0 >= LEAST_KNOWN && precise(all) && told.0 >= 1 && precise(told) {
            kept.push((shape.clone(), all.0, all.1));
        }
    }
    kept.sort_by(|one, other| other.1.cmp(&one.1).then_with(|| one.0.cmp(&other.0)));
    let telling_names: Vec<String> = telling.iter().map(|seed| format!("{} {}", seeds[*seed].0, seeds[*seed].1)).collect();
    let mut report = vec![format!(
        "shapes: {keys_held} keys hold the two names of a seed, {} of them held by {LEAST_SEED_PAIRS} or more seeds; seeds whose true partner is not the name most often found beside them: {} of {} ({}); names that take the article: {}; kept {}",
        tried.len(),
        telling.len(),
        seeds.len(),
        telling_names.join(", "),
        bare.len(),
        kept.len()
    )];
    report.extend(kept.iter().take(SHAPES_SHOWN).map(|(shape, right, wrong)| format!("  shape {} (seed sentences right {right}, wrong {wrong})", shape.shown())));
    Ok((kept, report))
}
because!(learned_shapes, PatternRelation, DiscriminatingSeeds, "the shapes of one round: the keys of the sentences that hold both names of a seed, the article read as part of a name only where the name takes it, kept as candidates when two seeds or more hold them; each candidate counted on the seeds over all files, and kept when it finds enough seed sentences right and nearly all of its seed sentences right, both on all seeds and on the seeds whose true partner is not the name most often found beside them, of which it must find one right at least");

struct Round {
    pairs: Vec<(Pair, usize, Judgment)>,
    taken: usize,
    unread: Vec<String>,
    lessons: usize,
}
because!(Round, TextFacts, "the pairs of a round that the margin takes, each with its sentences and how the proofs of the lessons use it, the kinds that could not be learned with them, and the lessons that judged them");

fn judged_round(topic: &str, form: &FactForm, harvest: &RoundHarvest, (key, seeds, joins, spelled): (KeyPlace, &[Pair], &HashSet<String>, &HashMap<String, String>), (kinds, classes, lanes, kept): (&[KindLessons], Option<&WordClasses>, usize, &mut Vec<Arc<Library>>)) -> Result<Round, String> {
    let folded = folded_counts(&harvest.counts);
    let mut chosen: BTreeSet<Pair> = margin_of(&folded, key);
    let seed_keys: HashSet<Pair> = seeds.iter().map(|(one, two)| (letters(one), letters(two))).collect();
    for pair in folded.keys() {
        let keys = (letters(&pair.0), letters(&pair.1));
        if seed_keys.contains(&keys) || (spelled.contains_key(&keys.0) && spelled.contains_key(&keys.1)) {
            chosen.insert(pair.clone());
        }
    }
    let candidates: Vec<Pair> = chosen.into_iter().collect();
    let found: HashSet<String> = harvest.counts.keys().flat_map(|(one, two)| [letters(one), letters(two)]).collect();
    let (judgments, unread, lessons) = judged(topic, form, &candidates, (kinds, &found, joins, spelled), (classes, lanes, kept))?;
    let taken = candidates.len();
    let pairs = candidates
        .into_iter()
        .zip(judgments)
        .map(|(pair, judgment)| {
            let sentences = folded.get(&pair).copied().unwrap_or_default();
            (pair, sentences, judgment)
        })
        .collect();
    Ok(Round { pairs, taken, unread, lessons })
}
because!(judged_round, TextFacts, MentionMargin, "the pairs of a round that the margin takes, with the seeds that the text still names and the pairs whose two names the lessons name, which the lessons judge whatever the margin says (the text names Delhi beside India more often than New Delhi, which the lessons name), judged by the proofs of the lessons, each with its sentences");

fn confirmed(round: &Round) -> Vec<Pair> {
    round.pairs.iter().filter(|(_, _, judgment)| judgment.right > 0 && judgment.wrong == 0).map(|(pair, _, _)| pair.clone()).collect()
}
because!(confirmed, TextFacts, "the pairs of a round that the proof of a right answer of a lesson rests on, and the proof of no wrong answer");

fn judges(round: &Round) -> bool {
    round.unread.is_empty() && round.pairs.iter().any(|(_, _, judgment)| judgment.right + judgment.wrong > 0)
}
because!(judges, TextFacts, "whether the lessons judged the pairs of a round: every kind was learned with them and the proof of some answer rests on one of them; a model learned from a few lessons may answer by words alone and rest on no fact");

fn said(line: String, lines: &mut Vec<String>) {
    eprintln!("{line}");
    lines.push(line);
}
because!(said, TextFacts, "a line of the report, kept for the end and shown at once on the error stream, as a run over a large text takes minutes");

fn dumped_shapes(dump: Option<&Path>, round: usize, shapes: &[(Shape, usize, usize)]) -> Result<(), String> {
    let Some(place) = dump else { return Ok(()) };
    let mut text = std::fs::read_to_string(place).unwrap_or_default();
    for (shape, right, wrong) in shapes {
        text.push_str(&format!("shape of round {round}{SHAPE_GAP}{}{SHAPE_GAP}{right}{SHAPE_GAP}{wrong}\n", shape.shown()));
    }
    std::fs::write(place, text).map_err(|error| format!("{}: {error}", place.display()))
}
because!(dumped_shapes, PatternRelation, "the kept shapes of a round with their seed sentences right and wrong, added to the dump file when the run names one: the shapes hold words of the text, so they stay out of the library");

fn dumped(dump: Option<&Path>, round: usize, judged: &Round) -> Result<(), String> {
    let Some(place) = dump else { return Ok(()) };
    let mut text = std::fs::read_to_string(place).unwrap_or_default();
    for (pair, sentences, judgment) in &judged.pairs {
        text.push_str(&format!("round {round}{SHAPE_GAP}{}{SHAPE_GAP}{}{SHAPE_GAP}{sentences}{SHAPE_GAP}{}{SHAPE_GAP}{}\n", pair.0, pair.1, judgment.right, judgment.wrong));
    }
    std::fs::write(place, text).map_err(|error| format!("{}: {error}", place.display()))
}
because!(dumped, TextFacts, "the pairs of a round with their sentences and the right and wrong answers that rest on each, added to a file when the run names one, so that a person sees what each round took and why it kept it");

pub fn learn_text_facts((corpus, most_files): (&Path, usize), topic: &str, kinds: &[KindLessons], (classes, lanes, dump): (Option<&WordClasses>, usize, Option<&Path>)) -> Result<FactsReport, String> {
    let library = read_library(Path::new(LIBRARY_FOLDER))?;
    let text = library.iter().find(|text| text.name == topic).ok_or_else(|| format!("no text of the library has the name {topic}"))?;
    let facts: Vec<Vec<String>> = text.facts.iter().map(|fact| word_texts(fact)).collect();
    let form = fact_form(&facts).ok_or_else(|| format!("the facts of {topic} share no form of two names"))?;
    let files: Vec<PathBuf> = corpus_files(corpus)?.into_iter().take(most_files).collect();
    let all_cases: Vec<&Case> = kinds.iter().flat_map(|kind| kind.cases.iter().copied()).collect();
    let joins = lesson_joins(&all_cases);
    let spelled = lesson_spelling(&all_cases, &joins);
    let own = form.shape();
    let mut lines: Vec<String> = Vec::new();
    let join_words: Vec<String> = joins.iter().cloned().collect::<BTreeSet<String>>().into_iter().collect();
    said(format!("the own form of {topic}: A {} B; {} files of {}; small words inside the names of the lessons: {}", form.middle.join(TOKEN_GAP), files.len(), corpus.display(), join_words.join(", ")), &mut lines);
    let first = round_harvest(&files, (&[], &own), &joins, lanes)?;
    if let Some(place) = dump {
        std::fs::write(place, "").map_err(|error| format!("{}: {error}", place.display()))?;
    }
    let mut libraries: Vec<Arc<Library>> = Vec::new();
    let start = judged_round(topic, &form, &first, (KeyPlace::Both, &[], &joins, &spelled), (kinds, classes, lanes, &mut libraries))?;
    dumped(dump, 0, &start)?;
    let mut seeds: Vec<Pair> = confirmed(&start);
    if !judges(&start) {
        seeds = start.pairs.iter().map(|(pair, _, _)| pair.clone()).collect();
        said(format!("round 0, the own form alone: {} pairs in the text, {} taken by the margin, {} lessons that can judge them; no proof rests on them ({}), so the seeds are the pairs of the own form that the margin takes, judged in the next round: {}", first.counts.len(), start.taken, start.lessons, start.unread.join(", "), seeds.len()), &mut lines);
    } else {
        said(format!("round 0, the own form alone: {} pairs in the text, {} taken by the margin, {} lessons that can judge them, seeds (in a proof of a right answer and in none of a wrong one): {}", first.counts.len(), start.taken, start.lessons, seeds.len()), &mut lines);
    }
    let seed_count = seeds.len();
    let seeds_judged = judges(&start);
    let mut found_in: HashMap<Pair, usize> = seeds.iter().map(|pair| (pair.clone(), 0usize)).collect();
    let mut places = first.places;
    let mut key = key_place(&first.counts, &seeds);
    let mut shapes: Vec<(Shape, usize, usize)> = Vec::new();
    let mut last: Option<Round> = None;
    for round in 1..=ROUNDS {
        said(format!("round {round}, {} seeds:", seeds.len()), &mut lines);
        let (kept, shape_lines) = learned_shapes(&files, &seeds, (&joins, key), lanes)?;
        dumped_shapes(dump, round, &kept)?;
        for line in shape_lines {
            said(line, &mut lines);
        }
        let only: Vec<Shape> = kept.iter().map(|(shape, _, _)| shape.clone()).collect();
        let harvest = round_harvest(&files, (&only, &own), &joins, lanes)?;
        key = key_place(&harvest.counts, &seeds);
        let judged_now = judged_round(topic, &form, &harvest, (key, &seeds, &joins, &spelled), (kinds, classes, lanes, &mut libraries))?;
        dumped(dump, round, &judged_now)?;
        if !judges(&judged_now) {
            if round == ROUNDS {
                return Err(format!("round {round}: no proof of the lessons rests on the {} facts that the margin takes, on the {} lessons that can judge them ({})", judged_now.taken, judged_now.lessons, judged_now.unread.join(", ")));
            }
            said(format!("  harvest: {} pairs, keys of the margin {key:?}, {} taken, {} lessons that can judge them; no proof rests on them ({}), so all of them are the seeds of the next round, judged there", harvest.counts.len(), judged_now.taken, judged_now.lessons, judged_now.unread.join(", ")), &mut lines);
            seeds = judged_now.pairs.iter().map(|(pair, _, _)| pair.clone()).collect();
            for (pair, found_places) in harvest.places {
                places.insert(pair, found_places);
            }
            for pair in &seeds {
                found_in.entry(pair.clone()).or_insert(round);
            }
            shapes = kept;
            continue;
        }
        let now = confirmed(&judged_now);
        let denied = judged_now.pairs.iter().filter(|(_, _, judgment)| judgment.wrong > 0).count();
        said(format!("  harvest: {} pairs, keys of the margin {key:?}, {} taken, {} lessons that can judge them, confirmed {}, in a proof of a wrong answer {denied}", harvest.counts.len(), judged_now.taken, judged_now.lessons, now.len()), &mut lines);
        for pair in &now {
            found_in.entry(pair.clone()).or_insert(round);
        }
        for (pair, found_places) in harvest.places {
            places.insert(pair, found_places);
        }
        seeds = now;
        shapes = kept;
        last = Some(judged_now);
    }
    let last = last.ok_or_else(|| "no round of shapes ran".to_string())?;
    let mut kept_facts: Vec<LearnedFact> = last
        .pairs
        .iter()
        .filter(|(_, _, judgment)| judgment.right > 0 && judgment.wrong == 0)
        .map(|(pair, sentences, judgment)| {
            let found_places = places.get(pair).cloned().unwrap_or_default().into_iter().map(|(file, line)| (file_name(&files[file]), line)).collect();
            LearnedFact { pair: spelled_pair(pair, &spelled), sentences: *sentences, places: found_places, right: judgment.right, round: found_in.get(pair).copied().unwrap_or_default() }
        })
        .collect();
    kept_facts.sort_by(|one, other| one.pair.cmp(&other.pair));
    said(format!("facts kept (confirmed in the last round): {}", kept_facts.len()), &mut lines);
    let corpus_name = corpus.file_name().map_or_else(|| corpus.display().to_string(), |name| name.to_string_lossy().into_owned());
    Ok(FactsReport { lines, facts: kept_facts, form, shapes, seeds: seed_count, seeds_judged, corpus: corpus_name, files: files.len() })
}
because!(learn_text_facts, TextFacts, PatternRelation, "the facts of one text of the library from a large text: the own form of its facts found in the text and judged by the proofs of the lessons gives the seeds; each round learns shapes from the seeds, finds pairs with them, takes them by the margin and keeps those that the proofs confirm as the seeds of the next round; a round whose facts are too few for the lessons to be learned with them passes all the pairs that the margin takes to the next round, to be judged there, and the last round must be judged; the facts are those that the last round confirms, each in the spelling of the lessons where they name it, with the sentences and places that gave it; each round's pairs go to a file when the run names one; a run may read only the first files of the text, in the order of their names, to see how the facts grow with the text");

pub fn learned_text(report: &FactsReport, topic: &str, kinds: &[String]) -> String {
    let mut body: Vec<String> = Vec::new();
    for fact in &report.facts {
        body.push(report.form.line(&fact.pair));
        body.push(format!("{NOTE_START}{} sentences, round {}, right answers of lessons that rest on it {}; first sentences: {}", fact.sentences, fact.round, fact.right, fact.places.iter().map(|(file, line)| format!("{file} line {line}")).collect::<Vec<String>>().join(", ")));
    }
    let print = lines_print(&body.join("\n"));
    let (right, wrong) = report.shapes.iter().fold((0usize, 0usize), |(right, wrong), (_, one, other)| (right + one, wrong + other));
    let mut out = vec![
        format!(
            "{NOTE_START}Facts of the text {topic} learned by the step that reads facts from a large text (ilm facts), not written by a person, {LINES_PRINT_MARK}{print}: the text {} ({} files), its own form A {} B, sentence shapes learned from the {} pairs of that form that {}, and each fact kept only when the proof of a right answer of the lessons of {} rests on it and the proof of no wrong answer does.",
            report.corpus,
            report.files,
            report.form.middle.join(TOKEN_GAP),
            report.seeds,
            if report.seeds_judged { "the proofs of the lessons confirmed" } else { "the margin took (too few for the lessons to judge, so the proofs judged them in the next round)" },
            kinds.join(", ")
        ),
        format!("{NOTE_START}{} shapes in the last round, which together find {right} seed sentences right and {wrong} wrong; the shapes themselves stay in the step's dump, out of the library, as each holds words of the text.", report.shapes.len()),
    ];
    out.extend(body);
    out.join("\n") + "\n"
}
because!(learned_text, TextFacts, "the learned text of the facts: a first note that says how the facts were learned and from what, with the print of the lines that are not notes, by which the library refuses a hand edit; a note with the number of shapes and their sentences right and wrong on the seeds, as the shapes hold words of the text; then each fact with a note of its sentences, its round, the right answers that rest on it and the files and lines of its first sentences, never the words of the text");
