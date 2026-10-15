use crate::base::lanes::on_lanes;
use crate::base::shares::share;
use crate::know::guess::GuessModel;
use crate::know::article::{lowered, word_runs};
use crate::know::reader::{Entry, Lesson};
use crate::know::signs::{is_space, is_word_sign};
use crate::know::store::{Fact, OTHERS_FILE, SUBJECTS_FILE, Texts, article_of, articles};
use crate::know::sure::Knowledge;
use crate::know::titles::names;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

pub struct FactProbe;
source!(FactProbe, "the measure of knowledge at scale, after the probe of Petroni and others for what a model knows: a question is a relation and a subject, the answer is the object of the fact; a model gets lesson facts with the articles of their subjects, then reads articles, then answers the asked facts; it is given no sentence of an asked fact, so every answer comes from text that it read, and every answer names its sentence");

const DEV_PARTS: usize = 3;
because!(DEV_PARTS, FactProbe, "for development, of the parts of the lesson facts of a relation this many are the lessons and the rest is asked, so that a reader is designed with no look at the asked facts of the probe");

const DEV_OF: usize = 5;
because!(DEV_OF, FactProbe, "the parts that the lesson facts of a relation are cut in for development");

const FORGOTTEN: usize = 20;
because!(FORGOTTEN, FactProbe, "the articles with a right answer that are taken out of the table again, to show that an answer leaves with its article");

const BATCH: usize = 16384;
because!(BATCH, FactProbe, "the articles that are read side by side before their entries go into the table: so many that the largest article of a batch is a small part of its work, and few enough that their text is small beside the table");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    Asked,
    Subjects,
    All,
}
because!(Which, FactProbe, "the text that a run reads: only the articles that answer to a name of a lesson fact or of an asked fact, every article of the subjects, or these and the other articles of the store");

#[derive(Clone, Debug, Default)]
pub struct Numbers {
    pub lessons: usize,
    pub asked: usize,
    pub right: usize,
    pub answered: usize,
    pub sourced: usize,
    pub inferred: usize,
    pub gone: usize,
    pub forgotten: usize,
    pub articles: usize,
    pub signs: usize,
    pub entries: usize,
    pub model_seconds: f64,
    pub read_seconds: f64,
    pub answer_seconds: f64,
    pub by_relation: Vec<(String, usize, usize)>,
    pub guessed: usize,
    pub guessed_right: usize,
}
because!(Numbers, FactProbe, "what a run of the probe gives: the lesson facts and the asked facts; the right answers, the answers, the answers whose sentence stands in the named article and holds the answer, the answers that no sentence states; the articles that were forgotten and whose answer was gone after that; the articles and signs that were read and the entries of the table; the seconds for the model, the reading and the answers; for each relation its right answers and its asked facts; and the facts with no answer in the text that the name guess answers, with the right ones among them");

pub fn development(facts: &[Fact]) -> (Vec<&Fact>, Vec<&Fact>) {
    let mut order: Vec<&str> = Vec::new();
    let mut by: HashMap<&str, Vec<&Fact>> = HashMap::new();
    for fact in facts.iter().filter(|fact| fact.lesson) {
        by.entry(fact.relation.as_str())
            .or_insert_with(|| {
                order.push(fact.relation.as_str());
                Vec::new()
            })
            .push(fact);
    }
    let mut lessons: Vec<&Fact> = Vec::new();
    let mut asked: Vec<&Fact> = Vec::new();
    for relation in order {
        let mine = &by[relation];
        let cut = mine.len() * DEV_PARTS / DEV_OF;
        lessons.extend(&mine[..cut]);
        asked.extend(&mine[cut..]);
    }
    (lessons, asked)
}
because!(development, FactProbe, "the lesson facts cut for development: for each relation, in the order of its first fact, the first part of its lesson facts are the lessons and the others are asked; the asked facts of the probe are not touched");

pub fn whole(facts: &[Fact]) -> (Vec<&Fact>, Vec<&Fact>) {
    (facts.iter().filter(|fact| fact.lesson).collect(), facts.iter().filter(|fact| !fact.lesson).collect())
}
because!(whole, FactProbe, "the lesson facts and the asked facts of the probe as the set gives them");

fn same(one: &str, other: &str) -> bool {
    one.trim_matches(is_space).to_lowercase() == other.trim_matches(is_space).to_lowercase()
}
because!(same, FactProbe, "an answer is the object of a fact when both are the same text in any case of letters, with the space around them left out");

fn stands(body: &str, sentence: &str, word: &str) -> bool {
    if sentence.is_empty() || word.is_empty() || !body.contains(sentence) {
        return false;
    }
    let low = sentence.to_lowercase();
    let word = word.to_lowercase();
    let mut from = 0usize;
    while let Some(found) = low[from..].find(&word) {
        let start = from + found;
        let before = low[..start].chars().next_back().is_some_and(is_word_sign);
        let after = low[start + word.len()..].chars().next().is_some_and(is_word_sign);
        if !before && !after {
            return true;
        }
        from = start + low[start..].chars().next().map_or(1, char::len_utf8);
    }
    false
}
because!(stands, FactProbe, "an answer has its source when its sentence stands in the named article and holds the word as a word of its own, in any case of letters");

pub struct Reference {
    pub with_article: f64,
    pub one_answer: f64,
    pub first_word: f64,
    pub in_article: f64,
}
because!(Reference, FactProbe, "the reference points of a probe in hundredths of the asked facts: the subject has one article; the most frequent object of the relation among the lessons is right; the first word of the article that is an object of the relation among the lessons is right; the object stands in the article (the most that a reader of that one article can reach)");

pub fn references(lessons: &[&Fact], asked: &[&Fact], texts: &Texts) -> Reference {
    let mut order: HashMap<&str, Vec<(String, usize)>> = HashMap::new();
    for fact in lessons {
        let mine = order.entry(fact.relation.as_str()).or_default();
        let object = fact.object.to_lowercase();
        match mine.iter_mut().find(|(known, _)| *known == object) {
            Some(count) => count.1 += 1,
            None => mine.push((object, 1)),
        }
    }
    let (mut one, mut first, mut inside, mut have) = (0usize, 0usize, 0usize, 0usize);
    let mut low = String::new();
    for fact in asked {
        let mine = order.get(fact.relation.as_str()).map_or(&[][..], Vec::as_slice);
        let truth = fact.object.to_lowercase();
        let mut most: Option<&(String, usize)> = None;
        for object in mine {
            if most.is_none_or(|top| object.1 > top.1) {
                most = Some(object);
            }
        }
        one += usize::from(most.is_some_and(|(object, _)| *object == truth));
        let Some(text) = texts.title_of(&fact.subject).and_then(|title| texts.get(title)) else { continue };
        have += 1;
        let mut holds = false;
        let mut typed: Option<bool> = None;
        for (start, end) in word_runs(text) {
            lowered(&text[start..end], &mut low);
            holds |= low == truth;
            if typed.is_none() && mine.iter().any(|(object, _)| *object == low) {
                typed = Some(low == truth);
            }
        }
        inside += usize::from(holds);
        first += usize::from(typed == Some(true));
    }
    Reference { with_article: share(have, asked.len()), one_answer: share(one, asked.len()), first_word: share(first, asked.len()), in_article: share(inside, asked.len()) }
}
because!(references, FactProbe, "the reference points of the asked facts, by plain counting with no model");

pub fn lessons_of<'a>(lessons: &[&'a Fact], fit: usize, texts: &'a Texts) -> Vec<Lesson<'a>> {
    let mut count: HashMap<&str, usize> = HashMap::new();
    let mut taken: Vec<Lesson> = Vec::new();
    for fact in lessons {
        let held = count.entry(fact.relation.as_str()).or_insert(0);
        if fit > 0 && *held >= fit {
            continue;
        }
        *held += 1;
        let title = texts.title_of(&fact.subject);
        taken.push(Lesson { relation: &fact.relation, subject: &fact.subject, object: &fact.object, title, article: title.and_then(|title| texts.get(title)) });
    }
    taken
}
because!(lessons_of, FactProbe, "the lesson facts with the article of each subject; with a limit, the first lesson facts of each relation up to it, for the curve by lesson facts");

fn read_batch<T: AsRef<str> + Sync, B: AsRef<str> + Sync>(model: &mut Knowledge, batch: &[(T, B)], lanes: usize, numbers: &mut Numbers) {
    let mut largest: Vec<usize> = (0..batch.len()).collect();
    largest.sort_by_key(|place| std::cmp::Reverse(batch[*place].1.as_ref().len()));
    let done: Vec<(Vec<(u32, Entry)>, usize)> = on_lanes(largest.len(), lanes, &|at| {
        let (title, body) = (&batch[largest[at]].0, &batch[largest[at]].1);
        (model.entries(title.as_ref(), body.as_ref()), body.as_ref().chars().count())
    });
    let mut read: Vec<Option<(Vec<(u32, Entry)>, usize)>> = (0..batch.len()).map(|_| None).collect();
    for (at, found) in done.into_iter().enumerate() {
        read[largest[at]] = Some(found);
    }
    for ((title, _), found) in batch.iter().zip(read) {
        let Some((entries, signs)) = found else { continue };
        numbers.articles += 1;
        numbers.signs += signs;
        numbers.entries += entries.len();
        model.put(title.as_ref(), entries);
    }
}
because!(read_batch, FactProbe, "some articles read side by side on the threads with one model that does not change, the largest given out first so that no thread waits long for the last one, and their entries put into the table in the order of the text, so the table is the same for every number of threads");

pub fn read_files(model: &mut Knowledge, store: &Path, files: &[&str], lanes: usize, numbers: &mut Numbers) -> std::io::Result<()> {
    let (send, take) = std::sync::mpsc::sync_channel::<std::io::Result<Vec<String>>>(1);
    std::thread::scope(|scope| {
        scope.spawn(move || {
            for file in files {
                let mut stream = match articles(&store.join(file)) {
                    Ok(stream) => stream.peekable(),
                    Err(fault) => {
                        send.send(Err(fault)).ok();
                        return;
                    }
                };
                while stream.peek().is_some() {
                    if send.send(Ok(stream.by_ref().take(BATCH).collect())).is_err() {
                        return;
                    }
                }
            }
        });
        for lines in take {
            let lines = lines?;
            let batch: Vec<(String, String)> = on_lanes(lines.len(), lanes, &|place| article_of(&lines[place])).into_iter().flatten().collect();
            read_batch(model, &batch, lanes, numbers);
        }
        Ok(())
    })
}
because!(read_files, FactProbe, "every article of some files of the store into the table of a model, in the order of the files and of their lines; a thread of its own reads the lines of the next batch while the lines of a batch are taken apart and read on all threads, and a file that does not open ends the reading after the batches before it, as one pass in order would");

pub fn probe(lessons: &[&Fact], asked: &[&Fact], texts: &Texts, store: &Path, which: Which, fit: usize, classes: &HashMap<String, String>, lanes: usize, look: &mut dyn FnMut(&Knowledge)) -> std::io::Result<(Knowledge, Numbers)> {
    let taken = lessons_of(lessons, fit, texts);
    let mut numbers = Numbers { lessons: taken.len(), asked: asked.len(), ..Numbers::default() };
    let started = Instant::now();
    let mut model = Knowledge::learn(&taken, classes, lanes);
    let guesses = GuessModel::learn(&taken);
    numbers.model_seconds = started.elapsed().as_secs_f64();
    let started = Instant::now();
    if which == Which::Asked {
        let wanted: HashSet<String> = taken.iter().map(|lesson| lesson.subject.to_lowercase()).chain(asked.iter().map(|fact| fact.subject.to_lowercase())).collect();
        let mut batch: Vec<(&str, &str)> = Vec::new();
        for (title, body) in texts.titles.iter().zip(&texts.bodies) {
            if names(title).iter().any(|name| wanted.contains(name)) {
                batch.push((title.as_str(), body.as_str()));
                if batch.len() >= BATCH {
                    read_batch(&mut model, &batch, lanes, &mut numbers);
                    batch.clear();
                }
            }
        }
        read_batch(&mut model, &batch, lanes, &mut numbers);
    } else {
        let files: &[&str] = if which == Which::All { &[SUBJECTS_FILE, OTHERS_FILE] } else { &[SUBJECTS_FILE] };
        read_files(&mut model, store, files, lanes, &mut numbers)?;
    }
    numbers.read_seconds = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let mut right_titles: Vec<(String, &Fact)> = Vec::new();
    let mut elsewhere: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for fact in asked {
        let place = match numbers.by_relation.iter().position(|(relation, _, _)| *relation == fact.relation) {
            Some(place) => place,
            None => {
                numbers.by_relation.push((fact.relation.clone(), 0, 0));
                numbers.by_relation.len() - 1
            }
        };
        numbers.by_relation[place].2 += 1;
        let Some((entry, title)) = model.answer(&fact.relation, &fact.subject) else {
            if let Some(object) = guesses.guess(&fact.relation, &fact.subject) {
                numbers.guessed += 1;
                numbers.guessed_right += usize::from(same(object, &fact.object));
            }
            continue;
        };
        numbers.answered += 1;
        let holds = model.holds(entry);
        numbers.inferred += usize::from(holds != entry.answer);
        let ok = same(&entry.answer, &fact.object);
        numbers.right += usize::from(ok);
        numbers.by_relation[place].1 += usize::from(ok);
        match texts.get(title) {
            Some(body) => numbers.sourced += usize::from(stands(body, &entry.sentence, holds)),
            None if which == Which::All => elsewhere.entry(title.to_string()).or_default().push((entry.sentence.clone(), holds.to_string())),
            None => {}
        }
        if ok && right_titles.len() < FORGOTTEN && !right_titles.iter().any(|(held, _)| held == title) {
            right_titles.push((title.to_string(), fact));
        }
    }
    numbers.answer_seconds = started.elapsed().as_secs_f64();
    if !elsewhere.is_empty() {
        for line in articles(&store.join(OTHERS_FILE))? {
            let Some((title, body)) = article_of(&line) else { continue };
            for (sentence, word) in elsewhere.get(&title).map_or(&[][..], Vec::as_slice) {
                numbers.sourced += usize::from(stands(&body, sentence, word));
            }
        }
    }
    look(&model);
    numbers.forgotten = right_titles.len();
    for (title, fact) in &right_titles {
        model.forget(title);
        numbers.gone += usize::from(model.answer(&fact.relation, &fact.subject).is_none_or(|(_, again)| again != title));
    }
    Ok((model, numbers))
}
because!(probe, FactProbe, "one run of the probe: the model from the lesson facts, the text read into its table, each asked fact answered from the table, each answer checked against the object and against its source, a fact with no answer in the table guessed from its subject's name and counted apart, and some articles forgotten again; the caller can look at the model with its whole table before articles are forgotten");
