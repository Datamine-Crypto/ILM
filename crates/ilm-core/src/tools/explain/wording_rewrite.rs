use crate::base::words::answer_words;
use crate::cases::Case;
use crate::tools::call::{AnswerMark, Learner, Reading, Said};
use crate::tools::explain::abduction::capitalized;
use crate::tools::explain::wording::WHOLE_SHARE;
use patterns::{because, source};
use std::collections::{BTreeMap, BTreeSet, HashSet};

pub struct LearnedRewrites;
source!(LearnedRewrites, "a sentence written again in another build, learned from lessons that give a sentence and its new form (a sentence into the passive, into a question): the words of the answer are found in the sentence of the question, as whole runs of it that move together or as other forms of its words that the lessons show, with no fact of the library, and the runs that move together teach what a unit of a sentence is (a noun phrase: a determiner and a noun, and the phrases after the noun that move with it), so that a template of units learned from some builds reads builds that no lesson has");

const DETERMINER_SHAPE: &str = "D";
because!(DETERMINER_SHAPE, LearnedRewrites, "the class of a determiner in the shape of a run of words");

const NOUN_SHAPE: &str = "N";
because!(NOUN_SHAPE, LearnedRewrites, "the class of a noun in the shape of a run of words");

const MOVED_SHAPE: &str = "AUX";
because!(MOVED_SHAPE, LearnedRewrites, "the class of a word that moves to the front in a lesson (the main auxiliary of a question)");

const BESIDE_SHAPE: &str = "V";
because!(BESIDE_SHAPE, LearnedRewrites, "the class of a word that stands next to a word that moves to the front in some lesson (a verb)");

const TASK_MARK: char = ':';
because!(TASK_MARK, LearnedRewrites, "the mark that closes the task word with which a question opens (passiv:, quest:, decl:)");

const REWRITE_GAP: &str = " ";
because!(REWRITE_GAP, LearnedRewrites, "the space between the words of a sentence written again");

const HEAD_WORDS: usize = 2;
because!(HEAD_WORDS, LearnedRewrites, "the words at the head of a noun phrase whose forms its case changes, its determiner and its noun; the phrases after them keep the case of their own joining word");

const NEIGHBOURS: usize = 2;
because!(NEIGHBOURS, LearnedRewrites, "the words of a sentence read together to find the words beside a word that moves to the front: the word and the one after it");

const REWRITE_WORDS_PERCENT: usize = 50;
because!(REWRITE_WORDS_PERCENT, LearnedRewrites, "the least share of the words of an answer that stand in its question, as they are or as another form of a word of it, for the answer to be that sentence written again");

const LEAST_DETERMINER_STARTS: usize = 3;
because!(LEAST_DETERMINER_STARTS, LearnedRewrites, "the fewest lessons in which a word opens a sentence that the lesson writes in another build (a passive, or a question whose moved word is not the second word) for it to be read as a determiner: such a sentence opens with its subject, and a subject opens with its determiner. The first word of each run that moves as one is not taken, as a German run opens with its noun when its determiner changes its form (deinen Esel, dein Esel). A word that opens few sentences may open them by chance (a name)");

const DETERMINER_NOUNS: usize = 2;
because!(DETERMINER_NOUNS, LearnedRewrites, "the fewest different nouns that a word must stand before in the questions of the lessons to be read as a determiner that opens no sentence (den, meinem: a form of a determiner that opens no subject), as a determiner stands before a noun; the nouns are the words after the determiners known so far, and nouns and determiners are learned in turn until no new determiner is found");

const MAJORITY: usize = 2;
because!(MAJORITY, LearnedRewrites, "a template gives a proof only when more than half of the lessons of its build teach it (its lessons, times this, above the lessons of the build), and more than one lesson: the lessons of a build whose sentences are written again in many ways (functions on rows, built by the count of their words) agree on no template, and its answers are guesses");

const MOST_SUFFIX_LETTERS: usize = 3;
because!(MOST_SUFFIX_LETTERS, LearnedRewrites, "the most letters of an ending by which two values of a learned choice may differ to be read as forms of one word (s, es, n): an ending of more letters is a part of another word");

const LEAST_SUFFIX_PAIRS: usize = 2;
because!(LEAST_SUFFIX_PAIRS, LearnedRewrites, "the fewest pairs of values of a table that differ by one ending, before the ending is read as a change that the chosen word follows: one pair may agree by chance");

const RELATED_NOTE: &str = " (a piece found through a related word, so the answer is a guess)";
because!(RELATED_NOTE, LearnedRewrites, "the note of an answer of which a piece was found through a word that differs by an ending, or a noun form through nouns of the same ending, and not as a lesson showed it");

const LEAST_CHOICE_LESSONS: usize = 2;
because!(LEAST_CHOICE_LESSONS, LearnedRewrites, "the fewest lessons of a template for each value of a word chosen by the determiner, the noun or the word of a unit, on average: a table of as many values as lessons is no rule that the lessons teach, it only repeats them");

const NOUN_DETERMINERS: usize = 2;
because!(NOUN_DETERMINERS, LearnedRewrites, "the fewest different determiners that a word must follow in the lessons to be read as a noun: a noun takes many determiners, while the word after a German relative pronoun (der, die) is a verb and is mostly also seen after a moved word or at the end of a sentence");

const STEM_LETTERS: usize = 3;
because!(STEM_LETTERS, LearnedRewrites, "the first letters that two forms of one noun share at least (Pfaue, Pfauen), so that a noun phrase written in another case is found at its place in the answer");

const REWRITE_SHARE_PERCENT: usize = 90;
because!(REWRITE_SHARE_PERCENT, LearnedRewrites, "the share of the first lessons whose answer is a sentence made mostly of the words of the question, for a kind to be one of sentences written again: a kind of labels, actions or numbers has few of them, and the learner then costs nothing");

const REWRITE_TEXT: &str = "learned rewrites";
because!(REWRITE_TEXT, LearnedRewrites, "the name of the learned lines of a kind of sentences written again, as the note of an answer names it");

const TEMPLATE_HEAD: &str = "To write ";
because!(TEMPLATE_HEAD, LearnedRewrites, "the words that open the line of a learned template, by which an answer of a template, which the lessons of its build showed (a proof), is told from a sentence written as it is for a build that no lesson of its task showed (a guess)");

const NO_REWRITE: &str = "no learned template of units fits this sentence";
because!(NO_REWRITE, LearnedRewrites, "the note of a question that no template learned from the lessons reads, which gets no answer");

const NO_TEMPLATE: &str = "no lesson has the build of the units";
because!(NO_TEMPLATE, LearnedRewrites, "the note of a question whose units, a noun phrase or a word at each place, make a build that no lesson of its task word has, with the units");

const NO_PIECE: &str = "no words for piece";
because!(NO_PIECE, LearnedRewrites, "the note of a question whose template has a piece that gives no words for its units (a form or a chosen word that the lessons do not give), with the piece, the template and the units");

struct RewriteWords {
    det: HashSet<String>,
}
because!(RewriteWords, LearnedRewrites, "the determiners that the lessons teach: the first words of the sentences that they write again, and the words before the nouns that they teach, learned in turn");

impl RewriteWords {
    fn learned(lessons: &[(String, Vec<String>, Vec<String>)]) -> RewriteWords {
        let mut starts: BTreeMap<String, usize> = BTreeMap::new();
        for (_, tokens, output) in lessons {
            if tokens == output {
                continue;
            }
            if moved_word(tokens, output).is_none_or(|at| at > 1) {
                *starts.entry(tokens[0].to_lowercase()).or_default() += 1;
            }
        }
        let mut words = RewriteWords { det: starts.into_iter().filter(|(word, count)| *count >= LEAST_DETERMINER_STARTS && word.chars().next().is_some_and(char::is_alphabetic)).map(|(word, _)| word).collect() };
        let moved = moved_set(lessons);
        loop {
            let beside = beside_set(&words, lessons, &moved);
            let nouns = noun_set(&words, lessons, &moved, &beside);
            let mut before: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            for (_, tokens, _) in lessons {
                for pair in tokens.windows(NEIGHBOURS) {
                    let (word, next) = (pair[0].to_lowercase(), pair[1].to_lowercase());
                    if nouns.contains(&next) && pair[0].chars().next().is_some_and(char::is_alphabetic) && !words.det.contains(&word) && !nouns.contains(&word) && !moved.contains(&word) {
                        before.entry(word).or_default().insert(next);
                    }
                }
            }
            let found: Vec<String> = before.into_iter().filter(|(_, after)| after.len() >= DETERMINER_NOUNS).map(|(word, _)| word).collect();
            if found.is_empty() {
                return words;
            }
            words.det.extend(found);
        }
    }
}

fn split_question(question: &str) -> (String, Vec<String>) {
    let words: Vec<String> = question.split_whitespace().map(str::to_string).collect();
    match words.split_first() {
        Some((first, rest)) if first.ends_with(TASK_MARK) => (first.clone(), rest.to_vec()),
        _ => (String::new(), words),
    }
}
because!(split_question, LearnedRewrites, "the question of a lesson as the word that names its task (passiv:) and the words of its sentence, or no task word and all its words");

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Unit {
    Phrase(Vec<String>),
    Word(String),
}
because!(Unit, LearnedRewrites, "a unit of a sentence: a noun phrase (its words) or a word on its own");

impl Unit {
    fn words(&self) -> Vec<String> {
        match self {
            Unit::Phrase(words) => words.clone(),
            Unit::Word(word) => vec![word.clone()],
        }
    }
}

struct Units {
    nouns: BTreeSet<String>,
    head: Option<Vec<String>>,
    moved: BTreeSet<String>,
    beside: BTreeSet<String>,
    modifiers: BTreeMap<Vec<String>, usize>,
    groups: BTreeMap<String, BTreeSet<Vec<String>>>,
    longest: usize,
}
because!(Units, LearnedRewrites, "what a unit of a sentence is, learned from the lessons: the nouns (the words that follow determiners of several words in the questions of the lessons, as a determiner stands before a noun: Orang-Utan after mein, dein, der and ein; a verb follows a German relative pronoun, der or die: die gebadet haben), its head (the shortest shape of a run of words that moves as one, a determiner and a noun), the words that move to the front (the main auxiliary of a question) and the verbs (the words after such a word in some lesson, and the last words of sentences that follow no determiner, as a German verb ends its clause), the modifiers that follow a head (a joining word and a head again, from runs that move as one; and what follows the head of a subject before the moved word or of an object after it), and the first words of modifiers that share the rest of their shape as one group (above and behind, who and that), with the longest modifier");

impl Units {
    fn learn(words: &RewriteWords, lessons: &[(String, Vec<String>, Vec<String>)]) -> Units {
        let mut units = Units { nouns: BTreeSet::new(), head: None, moved: moved_set(lessons), beside: BTreeSet::new(), modifiers: BTreeMap::new(), groups: BTreeMap::new(), longest: 0 };
        let mut moved: Vec<(&[String], usize)> = Vec::new();
        let mut reordered: Vec<(&[String], &[String])> = Vec::new();
        for (_, tokens, output) in lessons {
            if tokens == output {
                continue;
            }
            match moved_word(tokens, output) {
                Some(at) => moved.push((tokens, at)),
                None => reordered.push((tokens, output)),
            }
        }
        units.beside = beside_set(words, lessons, &units.moved);
        units.nouns = noun_set(words, lessons, &units.moved, &units.beside);
        let ends: Vec<String> = lessons
            .iter()
            .filter_map(|(_, tokens, _)| {
                let last = tokens.iter().rposition(|word| word.chars().next().is_some_and(char::is_alphabetic))?;
                (last == 0 || !words.det.contains(&tokens[last - 1].to_lowercase())).then(|| tokens[last].to_lowercase())
            })
            .collect();
        units.beside.extend(ends.into_iter().filter(|word| !words.det.contains(word) && !units.nouns.contains(word) && !units.moved.contains(word)));
        let mut shapes: BTreeMap<Vec<String>, usize> = BTreeMap::new();
        for (tokens, output) in &reordered {
            for block in blocks(tokens, output).into_iter().filter(|block| block.len() > 1) {
                *shapes.entry(units.shape(words, &block)).or_default() += 1;
            }
        }
        for (tokens, at) in &moved {
            if *at > 1 {
                *shapes.entry(units.shape(words, &tokens[..*at])).or_default() += 1;
            }
        }
        let shortest = shapes.keys().map(Vec::len).min();
        units.head = shapes.iter().filter(|(shape, _)| Some(shape.len()) == shortest).max_by(|one, other| one.1.cmp(other.1).then_with(|| one.0.cmp(other.0))).map(|(shape, _)| shape.clone());
        let Some(head) = units.head.clone() else { return units };
        let size = head.len();
        for shape in shapes.keys() {
            let rest = &shape[size.min(shape.len())..];
            if shape[..size.min(shape.len())] != head[..] || rest.is_empty() || rest.len() % (size + 1) != 0 {
                continue;
            }
            let steps: Vec<&[String]> = rest.chunks(size + 1).collect();
            if steps.iter().all(|step| step[1..] == head[..] && literal(&step[0])) {
                for step in steps {
                    *units.modifiers.entry(step.to_vec()).or_default() += 1;
                }
            }
        }
        for (tokens, at) in &moved {
            let shape = units.shape(words, tokens);
            if shape.len() >= size && shape[..size] == head[..] && *at > size {
                *units.modifiers.entry(shape[size..*at].to_vec()).or_default() += 1;
            }
            let Some(start) = (at + 1..shape.len().saturating_sub(1)).find(|place| shape.get(*place..place + size) == Some(&head[..])) else { continue };
            let mut end = shape.len() - 1;
            let verb_first = shape.get(at + 1).is_some_and(|word| word == BESIDE_SHAPE);
            while !verb_first && end > start + size && shape[end - 1] == BESIDE_SHAPE {
                end -= 1;
            }
            if end > start + size {
                *units.modifiers.entry(shape[start + size..end].to_vec()).or_default() += 1;
            }
        }
        units.groups = word_groups(&units.modifiers);
        units.longest = units.modifiers.keys().map(Vec::len).max().unwrap_or_default();
        units
    }

    fn shape(&self, words: &RewriteWords, tokens: &[String]) -> Vec<String> {
        tokens
            .iter()
            .map(|word| {
                let low = word.to_lowercase();
                if words.det.contains(&low) {
                    DETERMINER_SHAPE.to_string()
                } else if self.nouns.contains(&low) {
                    NOUN_SHAPE.to_string()
                } else if self.moved.contains(&low) {
                    MOVED_SHAPE.to_string()
                } else if self.beside.contains(&low) {
                    BESIDE_SHAPE.to_string()
                } else {
                    low
                }
            })
            .collect()
    }

    fn units_of(&self, words: &RewriteWords, tokens: &[String]) -> Vec<Unit> {
        let shape = self.shape(words, tokens);
        let mut units = Vec::new();
        let mut at = 0;
        while at < tokens.len() {
            match &self.head {
                Some(head) if shape.get(at..at + head.len()) == Some(&head[..]) => {
                    let mut end = at + head.len();
                    'grow: loop {
                        let Some(rests) = shape.get(end).and_then(|first| self.groups.get(first)) else { break };
                        for length in (1..self.longest).rev() {
                            if shape.get(end + 1..end + 1 + length).is_some_and(|rest| rests.contains(rest)) {
                                end += 1 + length;
                                continue 'grow;
                            }
                        }
                        break;
                    }
                    units.push(Unit::Phrase(tokens[at..end].to_vec()));
                    at = end;
                }
                _ => {
                    units.push(Unit::Word(tokens[at].clone()));
                    at += 1;
                }
            }
        }
        units
    }

    fn line(&self) -> String {
        let head = self.head.clone().unwrap_or_default().join(REWRITE_GAP);
        let mut groups: BTreeMap<Vec<Vec<String>>, BTreeSet<&String>> = BTreeMap::new();
        for (word, rests) in &self.groups {
            groups.entry(rests.iter().cloned().collect()).or_default().insert(word);
        }
        let modifiers: Vec<String> = groups.iter().map(|(rests, firsts)| format!("{} with {}", firsts.iter().map(|word| word.as_str()).collect::<Vec<&str>>().join("/"), rests.iter().map(|rest| rest.join(REWRITE_GAP)).collect::<Vec<String>>().join(" or "))).collect();
        format!("A unit is {head} and any run of modifiers after it: {}.", modifiers.join("; "))
    }
}

fn moved_set(lessons: &[(String, Vec<String>, Vec<String>)]) -> BTreeSet<String> {
    lessons.iter().filter(|(_, tokens, output)| tokens != output).filter_map(|(_, tokens, output)| moved_word(tokens, output).map(|at| tokens[at].to_lowercase())).collect()
}
because!(moved_set, LearnedRewrites, "the words that move to the front in some lesson (the main auxiliaries of the questions)");

fn beside_set(words: &RewriteWords, lessons: &[(String, Vec<String>, Vec<String>)], moved: &BTreeSet<String>) -> BTreeSet<String> {
    let mut beside = BTreeSet::new();
    for (_, tokens, _) in lessons {
        for pair in tokens.windows(NEIGHBOURS) {
            let (first, word) = (pair[0].to_lowercase(), pair[1].to_lowercase());
            if moved.contains(&first) && pair[1].chars().next().is_some_and(char::is_alphabetic) && !words.det.contains(&word) && !moved.contains(&word) {
                beside.insert(word);
            }
        }
    }
    beside
}
because!(beside_set, LearnedRewrites, "the verbs: the words right after a word that moves to the front in some lesson that are no determiner and no moved word");

fn noun_set(words: &RewriteWords, lessons: &[(String, Vec<String>, Vec<String>)], moved: &BTreeSet<String>, beside: &BTreeSet<String>) -> BTreeSet<String> {
    let mut after_determiners: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (_, tokens, _) in lessons {
        for pair in tokens.windows(NEIGHBOURS) {
            let (before, word) = (pair[0].to_lowercase(), pair[1].to_lowercase());
            if words.det.contains(&before) && !words.det.contains(&word) && !beside.contains(&word) && !moved.contains(&word) && word.chars().next().is_some_and(char::is_alphabetic) {
                after_determiners.entry(word).or_default().insert(before);
            }
        }
    }
    let mut nouns: BTreeSet<String> = after_determiners.iter().filter(|(_, lemmas)| lemmas.len() >= NOUN_DETERMINERS).map(|(word, _)| word.clone()).collect();
    let mut endings: BTreeMap<String, usize> = BTreeMap::new();
    for noun in &nouns {
        for letters in 1..=MOST_SUFFIX_LETTERS {
            if let Some((_, ending)) = split_suffix(noun, letters).filter(|(stem, _)| nouns.contains(*stem)) {
                *endings.entry(ending.to_string()).or_default() += 1;
            }
        }
    }
    let related = |word: &str| endings.iter().filter(|(_, pairs)| **pairs >= LEAST_SUFFIX_PAIRS).any(|(ending, _)| nouns.contains(&format!("{word}{ending}")) || word.strip_suffix(ending.as_str()).is_some_and(|stem| nouns.contains(stem)));
    let forms: Vec<String> = after_determiners.keys().filter(|word| !nouns.contains(*word) && related(word)).cloned().collect();
    nouns.extend(forms);
    nouns
}
because!(noun_set, LearnedRewrites, "the nouns that the lessons teach: the words after several determiners that are no determiner, no verb and no moved word, and a word after one determiner that is another form of such a noun by an ending that several pairs of these nouns show (Hunden beside Hunde, as Eseln beside Esel and Molchen beside Molche)");

fn literal(piece: &str) -> bool {
    ![DETERMINER_SHAPE, NOUN_SHAPE, MOVED_SHAPE, BESIDE_SHAPE].contains(&piece)
}
because!(literal, LearnedRewrites, "whether a piece of a shape is a word of its own and not a class (the first word of a modifier is one: above, who)");

fn moved_word(tokens: &[String], output: &[String]) -> Option<usize> {
    if output.is_empty() || output.len() != tokens.len() {
        return None;
    }
    (0..tokens.len() - 1).find(|at| tokens[*at] == output[0] && tokens[..*at].iter().chain(&tokens[at + 1..tokens.len() - 1]).eq(&output[1..output.len() - 1]))
}
because!(moved_word, LearnedRewrites, "the word that moves to the front in a lesson: the answer starts with it and goes on with the sentence without it, the mark at the end aside (the main auxiliary of a question), when one does");

fn blocks(tokens: &[String], output: &[String]) -> Vec<Vec<String>> {
    let mut places: Vec<Option<usize>> = vec![None; tokens.len()];
    let mut used = vec![false; output.len()];
    loop {
        let mut longest = (0, 0, 0);
        for start in 0..tokens.len() {
            for place in 0..output.len() {
                let length = (0..).take_while(|step| start + step < tokens.len() && place + step < output.len() && places[start + step].is_none() && !used[place + step] && tokens[start + step] == output[place + step]).count();
                if length > longest.2 {
                    longest = (start, place, length);
                }
            }
        }
        if longest.2 == 0 {
            break;
        }
        for step in 0..longest.2 {
            places[longest.0 + step] = Some(longest.1 + step);
            used[longest.1 + step] = true;
        }
    }
    let mut out: Vec<Vec<String>> = Vec::new();
    let mut run: Vec<String> = Vec::new();
    for (at, word) in tokens.iter().enumerate() {
        let joined = at > 0 && matches!((places[at - 1], places[at]), (Some(before), Some(now)) if now == before + 1);
        if !joined && !run.is_empty() {
            out.push(std::mem::take(&mut run));
        }
        run.push(word.clone());
    }
    out.push(run);
    out
}
because!(blocks, LearnedRewrites, "the runs of the words of a sentence that its new form keeps together and in order: the longest run of words that the answer holds as they are is found first, then the longest of the words left, and so on, so that a word said twice (the quail accepted the peacocks) is found in its own run and not at the first place that holds it; neighbours that stay neighbours form one run");

fn word_groups(modifiers: &BTreeMap<Vec<String>, usize>) -> BTreeMap<String, BTreeSet<Vec<String>>> {
    let mut by_rest: BTreeMap<Vec<String>, BTreeSet<String>> = BTreeMap::new();
    for modifier in modifiers.keys().filter(|modifier| modifier.first().is_some_and(|first| literal(first))) {
        by_rest.entry(modifier[1..].to_vec()).or_default().insert(modifier[0].clone());
    }
    let root = joined_roots(by_rest.values());
    let mut rests: BTreeMap<String, BTreeSet<Vec<String>>> = BTreeMap::new();
    for (rest, firsts) in &by_rest {
        for word in firsts {
            rests.entry(root_of(&root, word)).or_default().insert(rest.clone());
        }
    }
    root.keys().map(|word| (word.clone(), rests.get(&root_of(&root, word)).cloned().unwrap_or_default())).collect()
}
because!(word_groups, LearnedRewrites, "the first words of modifiers that are followed by the same rest of a shape are one group, joined over all the rests that any of them takes, so that a word seen with one rest is read with the others (that before a subject's clause, as who was seen there)");

fn root_of(root: &BTreeMap<String, String>, word: &str) -> String {
    let mut now = word.to_string();
    while let Some(next) = root.get(&now).filter(|next| **next != now) {
        now = next.clone();
    }
    now
}
because!(root_of, LearnedRewrites, "the word that stands for the group of a word: the word it was joined to, followed to the end");

fn joined_roots<'a>(sets: impl Iterator<Item = &'a BTreeSet<String>>) -> BTreeMap<String, String> {
    let mut root: BTreeMap<String, String> = BTreeMap::new();
    for set in sets {
        let members: Vec<&String> = set.iter().collect();
        for word in &members {
            root.entry((*word).clone()).or_insert_with(|| (*word).clone());
        }
        let Some(first) = members.first().map(|word| root_of(&root, word)) else { continue };
        for word in &members[1..] {
            let other = root_of(&root, word);
            if other != first {
                root.insert(other, first.clone());
            }
        }
    }
    root
}
because!(joined_roots, LearnedRewrites, "the groups of words that the sets join: the words of one set are one group, and a word in two sets joins their groups");

fn stem_like(one: &str, other: &str) -> bool {
    one == other || one.chars().zip(other.chars()).take_while(|(first, second)| first == second).count() >= STEM_LETTERS
}
because!(stem_like, LearnedRewrites, "whether two words are forms of one noun by their first letters (Pfaue, Pfauen)");

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Item {
    Phrase(usize, Vec<String>),
    Copy(usize),
    Fixed(String),
}
because!(Item, LearnedRewrites, "a piece of the answer of a lesson aligned to the units of its sentence: a noun phrase at its place with the words that the answer gives it, a unit written as it is, or a word of the answer that no unit gives");

#[derive(Clone, Debug, PartialEq, Eq)]
enum Slot {
    Copy(usize),
    Fixed(String),
    Choice { unit: usize, key: ChoiceKey, table: BTreeMap<String, String> },
    Formed { unit: usize, dets: BTreeMap<String, String>, nouns: Option<BTreeMap<(String, String), String>>, classes: BTreeMap<String, String> },
    Unknown,
}
because!(Slot, LearnedRewrites, "a piece of a learned template: a unit written as it is; a fixed word; a word chosen by a unit (was or were by the noun of a noun phrase, wurde or wurden by its determiner, a participle by its verb); a noun phrase with its determiner and noun in the forms that the lessons give them (der as dem, Hunde as Hunden after a determiner of the class of den); or a piece that the lessons do not tell, which gives no answer");

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ChoiceKey {
    Determiner,
    Noun,
    Word,
}
because!(ChoiceKey, LearnedRewrites, "what of a unit chooses a word of the answer: the determiner or the noun of a noun phrase as the lessons pair them with the word (wurde after der, were after zebras), or the word of a unit that is no noun phrase (bewunderten, which chooses bewundert)");

fn choice_value(key: ChoiceKey, said: &[String]) -> Option<String> {
    match key {
        ChoiceKey::Determiner => said.first().map(|word| word.to_lowercase()),
        ChoiceKey::Noun => said.get(1).map(|word| word.to_lowercase()),
        ChoiceKey::Word => (said.len() == 1).then(|| said[0].to_lowercase()),
    }
}
because!(choice_value, LearnedRewrites, "the value of a unit by which a word is chosen: its determiner, its noun, or its one word");

type TemplateKey = (String, Vec<bool>);
because!(TemplateKey, LearnedRewrites, "the task word of a question and the units of its sentence, each a noun phrase or not, which pick a template");

pub struct RewriteModel {
    words: RewriteWords,
    units: Units,
    templates: BTreeMap<TemplateKey, (Vec<Slot>, usize, usize)>,
    copying: BTreeSet<String>,
    seen: HashSet<String>,
}
because!(RewriteModel, LearnedRewrites, "the learned rewrites of a kind: the determiners and the units learned from the lessons, a template for each task word and build of units with the lessons that teach it and the lessons of that build, the task words whose templates all write the sentence as it is, which write any sentence as it is, and the words of the questions of the lessons");

impl RewriteModel {
    pub fn learn(cases: &[&Case]) -> Option<RewriteModel> {
        let lessons: Vec<(String, Vec<String>, Vec<String>)> = cases.iter().map(|case| {
            let (task, tokens) = split_question(&case.question);
            (task, tokens, case.answer.split_whitespace().map(str::to_string).collect())
        }).collect();
        let words = RewriteWords::learned(&lessons);
        let units = Units::learn(&words, &lessons);
        let mut seen: BTreeMap<TemplateKey, Vec<(Vec<Unit>, Vec<Item>)>> = BTreeMap::new();
        for (task, tokens, output) in &lessons {
            let read = units.units_of(&words, tokens);
            let key = (task.clone(), read.iter().map(|unit| matches!(unit, Unit::Phrase(_))).collect());
            let items = aligned_answer(&words, &read, output);
            seen.entry(key).or_default().push((read, items));
        }
        let mut templates = BTreeMap::new();
        for (key, rows) in &seen {
            let mut shapes: BTreeMap<Vec<(ItemKind, usize)>, Vec<&(Vec<Unit>, Vec<Item>)>> = BTreeMap::new();
            for row in rows {
                shapes.entry(row.1.iter().map(item_key).collect()).or_default().push(row);
            }
            let Some((shape, members)) = shapes.into_iter().max_by(|one, other| one.1.len().cmp(&other.1.len()).then_with(|| one.0.cmp(&other.0))) else { continue };
            let slots: Vec<Slot> = shape.iter().enumerate().map(|(slot, piece)| slot_rule((key, slot, *piece), &members)).collect();
            templates.insert(key.clone(), (slots, members.len(), rows.len()));
        }
        let copying: BTreeSet<String> = templates.keys().map(|key| key.0.clone()).filter(|task| templates.iter().filter(|(key, _)| key.0 == *task).all(|(key, (slots, _, _))| *slots == (0..key.1.len()).map(Slot::Copy).collect::<Vec<Slot>>())).collect();
        let seen: HashSet<String> = lessons.iter().flat_map(|(_, tokens, _)| tokens.iter().map(|word| word.to_lowercase())).collect();
        (!templates.is_empty()).then_some(RewriteModel { words, units, templates, copying, seen })
    }

    fn all_known(&self, question: &str) -> bool {
        let (_, tokens) = split_question(question);
        tokens.iter().all(|word| {
            let low = word.to_lowercase();
            self.seen.contains(&low) || self.words.det.contains(&low) || self.units.moved.contains(&low) || self.units.beside.contains(&low)
        })
    }

    fn rewritten(&self, question: &str) -> Result<(String, String, bool), String> {
        let (task, tokens) = split_question(question);
        let units = self.units.units_of(&self.words, &tokens);
        let key: TemplateKey = (task.clone(), units.iter().map(|unit| matches!(unit, Unit::Phrase(_))).collect());
        let read: Vec<String> = units.iter().map(|unit| format!("[{}]", unit.words().join(REWRITE_GAP))).collect();
        let Some((slots, _, _)) = self.templates.get(&key) else {
            if self.copying.contains(&task) {
                return Ok((tokens.join(REWRITE_GAP), format!("{task} writes the sentence as it is"), false));
            }
            return Err(format!("{NO_TEMPLATE} {}", read.join(REWRITE_GAP)));
        };
        let mut out: Vec<String> = Vec::new();
        let mut related = false;
        for (place, slot) in slots.iter().enumerate() {
            match self.slot_words(slot, &units) {
                Some((words, by_relation)) => {
                    out.extend(words);
                    related |= by_relation;
                }
                None => return Err(format!("{NO_PIECE} {} of \"{}\", units {}", place + 1, template_line(&key, slots), read.join(REWRITE_GAP))),
            }
        }
        Ok((out.join(REWRITE_GAP), template_line(&key, slots), related))
    }

    fn backed(&self, question: &str) -> bool {
        let (task, tokens) = split_question(question);
        let key: TemplateKey = (task, self.units.units_of(&self.words, &tokens).iter().map(|unit| matches!(unit, Unit::Phrase(_))).collect());
        self.templates.get(&key).is_some_and(|(_, taught, built)| taught * MAJORITY > *built && *taught > 1)
    }

    fn slot_words(&self, slot: &Slot, units: &[Unit]) -> Option<(Vec<String>, bool)> {
        match slot {
            Slot::Copy(unit) => Some((units.get(*unit)?.words(), false)),
            Slot::Fixed(word) => Some((vec![word.clone()], false)),
            Slot::Choice { unit, key, table } => {
                let said = units.get(*unit)?.words();
                let value = choice_value(*key, &said)?;
                match table.get(&value) {
                    Some(word) => Some((vec![word.clone()], false)),
                    None => related_word(table, &value).map(|word| (vec![word], true)),
                }
            }
            Slot::Formed { unit, dets, nouns, classes } => {
                let said = units.get(*unit)?.words();
                let det = dets.get(&said[0].to_lowercase())?;
                let det_form = if said[0].chars().next().is_some_and(char::is_uppercase) { capitalized(det) } else { det.clone() };
                let (noun_form, by_ending) = match nouns {
                    None => (said[1].clone(), false),
                    Some(table) => {
                        let class = classes.get(det)?;
                        match table.get(&(said[1].clone(), class.clone())) {
                            Some(form) => (form.clone(), false),
                            None => (ending_form(table, &said[1], class)?, true),
                        }
                    }
                };
                Some(([det_form, noun_form].into_iter().chain(said[HEAD_WORDS..].iter().cloned()).collect(), by_ending))
            }
            Slot::Unknown => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ItemKind {
    Phrase,
    Copy,
    Fixed,
}
because!(ItemKind, LearnedRewrites, "the kind of a piece of an aligned answer, with no words and no unit");

fn item_key(item: &Item) -> (ItemKind, usize) {
    match item {
        Item::Phrase(unit, _) => (ItemKind::Phrase, *unit),
        Item::Copy(unit) => (ItemKind::Copy, *unit),
        Item::Fixed(_) => (ItemKind::Fixed, usize::MAX),
    }
}
because!(item_key, LearnedRewrites, "the shape of a piece of an aligned answer, with no words: its kind and its unit, so that answers of one build that differ only in their words fall together");

fn aligned_answer(words: &RewriteWords, units: &[Unit], output: &[String]) -> Vec<Item> {
    let mut items = Vec::new();
    let mut used: HashSet<usize> = HashSet::new();
    let mut last: Option<usize> = None;
    let mut at = 0;
    while at < output.len() {
        let mut best: Option<(Item, (usize, bool, bool, bool))> = None;
        for (place, unit) in units.iter().enumerate() {
            let said = unit.words();
            let Some(piece) = output.get(at..at + said.len()) else { continue };
            let (item, exact) = if piece == said.as_slice() {
                (if matches!(unit, Unit::Phrase(_)) { Item::Phrase(place, piece.to_vec()) } else { Item::Copy(place) }, true)
            } else if matches!(unit, Unit::Phrase(_)) && piece[HEAD_WORDS..] == said[HEAD_WORDS..] && words.det.contains(&piece[0].to_lowercase()) && stem_like(&said[1], &piece[1]) {
                (Item::Phrase(place, piece.to_vec()), false)
            } else {
                continue;
            };
            let score = (said.len(), !used.contains(&place), last.map_or(place == 0, |before| place == before + 1), exact);
            if best.as_ref().is_none_or(|(_, known)| score > *known) {
                best = Some((item, score));
            }
        }
        match best {
            Some((item, (length, _, _, _))) => {
                if let Item::Phrase(place, _) | Item::Copy(place) = &item {
                    used.insert(*place);
                    last = Some(*place);
                }
                items.push(item);
                at += length;
            }
            None => {
                items.push(Item::Fixed(output[at].clone()));
                at += 1;
            }
        }
    }
    items
}
because!(aligned_answer, LearnedRewrites, "the answer of a lesson as pieces of the units of its sentence, from left to right: at each place the longest unit that the answer holds there (as it is, or a noun phrase with another determiner and another form of its noun), of those alike first a unit not used yet, then the unit after the one before, then one written exactly, else a fixed word");

fn slot_rule((key, slot, piece): (&TemplateKey, usize, (ItemKind, usize)), members: &[&(Vec<Unit>, Vec<Item>)]) -> Slot {
    match piece {
        (ItemKind::Phrase, unit) => {
            let pieces: Vec<(Vec<String>, Vec<String>)> = members.iter().filter_map(|(units, items)| match (&units[unit], &items[slot]) {
                (Unit::Phrase(said), Item::Phrase(_, written)) => Some((said.clone(), written.clone())),
                _ => None,
            }).collect();
            if pieces.iter().all(|(said, written)| said == written) {
                return Slot::Copy(unit);
            }
            formed_rule(unit, &pieces).unwrap_or(Slot::Unknown)
        }
        (ItemKind::Copy, unit) => Slot::Copy(unit),
        _ => {
            let said: BTreeSet<String> = members.iter().filter_map(|(_, items)| if let Item::Fixed(word) = &items[slot] { Some(word.clone()) } else { None }).collect();
            if said.len() == 1 {
                return Slot::Fixed(said.into_iter().next().unwrap_or_default());
            }
            let phrases: Vec<usize> = key.1.iter().enumerate().filter(|(_, phrase)| **phrase).map(|(place, _)| place).collect();
            let plain_words: Vec<usize> = key.1.iter().enumerate().filter(|(_, phrase)| !**phrase).map(|(place, _)| place).collect();
            let tried = phrases.iter().flat_map(|place| [(*place, ChoiceKey::Determiner), (*place, ChoiceKey::Noun)]).chain(plain_words.iter().map(|place| (*place, ChoiceKey::Word)));
            for (place, choice) in tried {
                let mut table: BTreeMap<String, String> = BTreeMap::new();
                let clash = members.iter().any(|(units, items)| {
                    let Item::Fixed(word) = &items[slot] else { return true };
                    match choice_value(choice, &units[place].words()) {
                        Some(value) => *table.entry(value).or_insert_with(|| word.clone()) != *word,
                        None => true,
                    }
                });
                if !clash && table.len() * LEAST_CHOICE_LESSONS <= members.len() {
                    return Slot::Choice { unit: place, key: choice, table };
                }
            }
            Slot::Unknown
        }
    }
}
because!(slot_rule, LearnedRewrites, "a piece of a template from the lessons that teach it: a noun phrase written as it is in every lesson, or in the forms that the lessons give its determiner and noun; a unit as it is; a word of the answer that every lesson gives, or that one unit of the build chooses in every lesson: the determiner or the noun of a noun phrase first, then the word of a unit that is no noun phrase (was or were, wurde or wurden, a participle); else a piece that gives no answer");

fn formed_rule(unit: usize, pieces: &[(Vec<String>, Vec<String>)]) -> Option<Slot> {
    let mut dets: BTreeMap<String, String> = BTreeMap::new();
    for (said, written) in pieces {
        if *dets.entry(said[0].to_lowercase()).or_insert_with(|| written[0].to_lowercase()) != written[0].to_lowercase() {
            return None;
        }
    }
    if pieces.iter().all(|(said, written)| said[1] == written[1]) {
        return Some(Slot::Formed { unit, dets, nouns: None, classes: BTreeMap::new() });
    }
    let mut alike: BTreeMap<(String, String), BTreeSet<String>> = BTreeMap::new();
    for (said, written) in pieces {
        alike.entry((said[1].clone(), written[1].clone())).or_default().insert(written[0].to_lowercase());
    }
    let root = joined_roots(alike.values());
    let classes: BTreeMap<String, String> = root.keys().map(|det| (det.clone(), root_of(&root, det))).collect();
    let mut nouns: BTreeMap<(String, String), String> = BTreeMap::new();
    for (said, written) in pieces {
        let class = classes.get(&written[0].to_lowercase())?.clone();
        if *nouns.entry((said[1].clone(), class)).or_insert_with(|| written[1].clone()) != written[1] {
            return None;
        }
    }
    Some(Slot::Formed { unit, dets, nouns: Some(nouns), classes })
}
because!(formed_rule, LearnedRewrites, "a noun phrase in the forms that the lessons give it: each determiner in the one form that the lessons write for it (der as dem, den as der), and the noun as it is, or in the form that the lessons write for it after a determiner of one class; two determiners are one class when the lessons write some noun alike after both (Esel after dem and einem, Eseln after den and einigen), so that a noun seen after one determiner of a class is written after the others; no piece when a determiner or a noun is written in two ways");

fn template_line(key: &TemplateKey, slots: &[Slot]) -> String {
    let units: Vec<String> = key.1.iter().enumerate().map(|(at, phrase)| format!("{}{}", if *phrase { "P" } else { "W" }, at + 1)).collect();
    let pieces: Vec<String> = slots
        .iter()
        .map(|slot| match slot {
            Slot::Copy(unit) => units.get(*unit).cloned().unwrap_or_default(),
            Slot::Fixed(word) => word.clone(),
            Slot::Choice { unit, key, table, .. } => format!("({} by {}{})", table.iter().map(|(value, word)| format!("{word} if {value}")).collect::<Vec<String>>().join(", "), choice_words(*key), units.get(*unit).cloned().unwrap_or_default()),
            Slot::Formed { unit, dets, nouns, classes } => {
                let dets_line = dets.iter().map(|(said, written)| format!("{said} as {written}")).collect::<Vec<String>>().join(", ");
                let noun_line = match nouns {
                    None => "as it is".to_string(),
                    Some(table) => {
                        let mut groups: BTreeMap<&String, Vec<&String>> = BTreeMap::new();
                        for (det, class) in classes {
                            groups.entry(class).or_default().push(det);
                        }
                        let class_line = groups.values().map(|group| group.iter().map(|det| det.as_str()).collect::<Vec<&str>>().join(" ")).collect::<Vec<String>>().join("; ");
                        format!("in the form of the lessons after a determiner of its class ({} forms; classes {class_line})", table.len())
                    }
                };
                format!("{} with its determiner in the form of the lessons ({dets_line}) and its noun {noun_line}", units.get(*unit).cloned().unwrap_or_default())
            }
            Slot::Unknown => "?".to_string(),
        })
        .collect();
    format!("{TEMPLATE_HEAD}\"{}{}\" again: {}.", if key.0.is_empty() { String::new() } else { format!("{} ", key.0) }, units.join(REWRITE_GAP), pieces.join(REWRITE_GAP))
}
fn split_suffix(word: &str, letters: usize) -> Option<(&str, &str)> {
    let at = word.char_indices().rev().nth(letters.checked_sub(1)?)?.0;
    (at > 0).then(|| word.split_at(at))
}
because!(split_suffix, LearnedRewrites, "a word cut before its last letters into a stem and an ending, when a stem is left");

fn related_word(table: &BTreeMap<String, String>, value: &str) -> Option<String> {
    let mut found: BTreeSet<String> = BTreeSet::new();
    for letters in 1..=MOST_SUFFIX_LETTERS {
        let mut pairs: BTreeMap<String, Vec<(&String, &String)>> = BTreeMap::new();
        for (long, written) in table {
            if let Some((stem, ending)) = split_suffix(long, letters)
                && let Some(short) = table.get(stem)
            {
                pairs.entry(ending.to_string()).or_default().push((short, written));
            }
        }
        let read = |ending: &str, from_short: bool, known: &String| -> Option<String> {
            let seen = pairs.get(ending).filter(|seen| seen.len() >= LEAST_SUFFIX_PAIRS)?;
            if seen.iter().all(|(short, long)| short == long) {
                return Some(known.clone());
            }
            let targets: BTreeSet<&String> = seen.iter().filter(|(short, long)| if from_short { *short == known } else { *long == known }).map(|(short, long)| if from_short { *long } else { *short }).collect();
            let sources: BTreeSet<&String> = seen.iter().map(|(short, long)| if from_short { *short } else { *long }).collect();
            let functional = sources.iter().all(|source| seen.iter().filter(|(short, long)| if from_short { short == source } else { long == source }).map(|(short, long)| if from_short { *long } else { *short }).collect::<BTreeSet<&String>>().len() == 1);
            (functional && targets.len() == 1).then(|| targets.into_iter().next().cloned()).flatten()
        };
        if let Some((stem, ending)) = split_suffix(value, letters)
            && let Some(known) = table.get(stem)
            && let Some(word) = read(ending, true, known)
        {
            found.insert(word);
        }
        for (ending, _) in &pairs {
            if let Some(known) = table.get(&format!("{value}{ending}"))
                && let Some(word) = read(ending, false, known)
            {
                found.insert(word);
            }
        }
    }
    (found.len() == 1).then(|| found.into_iter().next()).flatten()
}
because!(related_word, LearnedRewrites, "the word that a table of a learned choice gives a value that no lesson showed, through a value of the table that differs from it by an ending (xylophones beside xylophone, verwirrte beside verwirrten): the pairs of values of the table that differ by that ending show how the chosen word changes with it, the same word (verwirrte and verwirrten both give verwirrt) or another word in the same way in every pair (zebra gives was and zebras were); no word when the pairs are too few, when they do not agree, or when two endings give two words");

fn ending_form(table: &BTreeMap<(String, String), String>, noun: &str, class: &str) -> Option<String> {
    let edits: Vec<(&String, (usize, String))> = table
        .iter()
        .filter(|((_, of_class), _)| of_class == class)
        .map(|((said, _), written)| {
            let kept = said.chars().zip(written.chars()).take_while(|(one, other)| one == other).count();
            (said, (said.chars().count() - kept, written.chars().skip(kept).collect()))
        })
        .collect();
    let letters: Vec<char> = noun.chars().collect();
    for length in (1..letters.len()).rev() {
        let ending: String = letters[letters.len() - length..].iter().collect();
        let alike: BTreeSet<&(usize, String)> = edits.iter().filter(|(said, _)| said.ends_with(&ending)).map(|(_, edit)| edit).collect();
        match alike.into_iter().collect::<Vec<&(usize, String)>>().as_slice() {
            [] => continue,
            [(cut, added)] if *cut < letters.len() => return Some(letters[..letters.len() - cut].iter().collect::<String>() + added),
            _ => return None,
        }
    }
    None
}
because!(ending_form, LearnedRewrites, "the form of a noun after a determiner of a class, when no lesson wrote that noun after that class: the change of the noun of the lessons whose word ends as this one does, the longest ending first (Ziesel as Esel: Zieseln, as Esel was written Eseln after den), when the nouns of that ending all change alike; no form when they do not");

fn choice_words(key: ChoiceKey) -> &'static str {
    match key {
        ChoiceKey::Determiner => "the determiner of ",
        ChoiceKey::Noun => "the noun of ",
        ChoiceKey::Word => "",
    }
}
because!(choice_words, LearnedRewrites, "the words that name what of a unit chooses a word, in the line of a template");

because!(template_line, LearnedRewrites, "a learned template as a line that a person reads: the task word and the units of the build (P for a noun phrase, W for a word, by their places), then the pieces of the answer");

impl Learner for RewriteModel {
    fn answer(&self, _sentences: &[String], question: &str) -> Said {
        match self.rewritten(question) {
            Ok((text, line, related)) => Said { words: answer_words(&text), mark: if line.starts_with(TEMPLATE_HEAD) && !related && self.backed(question) && self.all_known(question) { AnswerMark::Proof } else { AnswerMark::Guess }, note: format!("{REWRITE_TEXT}: {line}") },
            Err(why) => Said { words: Vec::new(), note: format!("{NO_REWRITE}: {why}"), mark: AnswerMark::NoAnswer },
        }
    }

    fn reading(&self) -> Option<Reading> {
        Some(Reading::Learned)
    }

    fn rules_used(&self, _sentences: &[String], question: &str) -> Vec<(String, bool)> {
        self.rewritten(question).map(|(_, line, _)| vec![(self.units.line(), false), (line, false)]).unwrap_or_default()
    }

    fn facts_used(&self, _sentences: &[String], question: &str) -> Vec<(String, bool)> {
        let (_, tokens) = split_question(question);
        let mut out: Vec<(String, bool)> = Vec::new();
        for word in &tokens {
            let low = word.to_lowercase();
            let fact = if self.words.det.contains(&low) {
                Some((format!("{low} is a determiner"), false))
            } else {
                self.units.nouns.contains(&low).then(|| (format!("{low} is a noun"), false))
            };
            if let Some(fact) = fact.filter(|(fact, _)| !out.iter().any(|(known, _)| known == fact)) {
                out.push(fact);
            }
        }
        out
    }

    fn proof(&self, _sentences: &[String], question: &str) -> Vec<String> {
        let (_, tokens) = split_question(question);
        let units: Vec<String> = self.units.units_of(&self.words, &tokens).iter().map(|unit| format!("[{}]", unit.words().join(REWRITE_GAP))).collect();
        match self.rewritten(question) {
            Ok((text, line, related)) => vec![format!("{REWRITE_TEXT}: {text}{}", if related { RELATED_NOTE } else { "" }), format!("  units: {}", units.join(REWRITE_GAP)), format!("  {}", self.units.line()), format!("  {line}")],
            Err(why) => vec![format!("{NO_REWRITE}: {why}"), format!("  {}", self.units.line())],
        }
    }

    fn lines_from_lessons(&self) -> Vec<String> {
        std::iter::once(self.units.line()).chain(self.templates.iter().map(|(key, (slots, _, _))| template_line(key, slots))).collect()
    }
}

pub fn rewrite_kind(cases: &[&Case]) -> bool {
    let fitting = cases
        .iter()
        .filter(|case| {
            let (_, tokens) = split_question(&case.question);
            let said: Vec<&str> = case.answer.split_whitespace().collect();
            case.sentences.is_empty() && said.len() > 1 && said.iter().filter(|word| tokens.iter().any(|token| stem_like(token, word))).count() * WHOLE_SHARE >= said.len() * REWRITE_WORDS_PERCENT
        })
        .count();
    !cases.is_empty() && fitting * WHOLE_SHARE >= cases.len() * REWRITE_SHARE_PERCENT
}
because!(rewrite_kind, LearnedRewrites, "whether the lessons of a kind give sentences written again: they tell no story, and nearly all their answers are sentences of several words of which at least half stand in the question, as they are or as another form of a word of it (meinem for mein, by its first letters)");
