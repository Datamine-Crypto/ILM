use crate::base::first_sight::{NumberBag, NumberTable};
use crate::base::lanes::on_lanes;
use crate::base::symbols::Symbols;
use crate::know::article::{lowered, sentence_starts, word_runs};
use crate::base::keys::padded;
use crate::know::reader::{CERTAIN, Entry, FRAME_WORDS, Frame, Frames, Lesson, MORE_HITS, MORE_SEEN, NAME_BETWEEN, NO_WORD, Scan, Side, key_of, name_of};
use crate::know::signs::{is_space, is_word_sign};
use crate::know::titles::names;
use patterns::{because, source};
use std::collections::HashMap;

pub struct SureWords;
source!(SureWords, "some facts are stated by no sentence: an article about a woman says she, an article about a writer of a country names the people of that country and not the country; so for each relation and each word with how often an article writes it, the lessons give in how many articles of lesson subjects the word stands so, and how often the fact of the article has one object; a word is sure for an object when that share is high, and the sure words of an article vote for their objects");

pub struct RouteChoice;
source!(RouteChoice, "which route answers first is counted for each relation: a model of the first part of the lesson subjects of each relation reads the articles of the rest of them, and the order with more right answers is taken; with as many, the sentence answers first, because its answer stands in a sentence. The same count measures the joined chance: for each route and each level of the chance, how many answers of the held lesson facts are at this level and how many of them are right");

pub struct ListedObjects;
source!(ListedObjects, "the objects of some relations are a short list that the lessons hold (sorts of music and film, positions in a game), and an article writes the words of its object with no frame around them that the lessons know; so for each relation the objects of its lesson facts are kept, each with the lesson articles that write every word of it and how many of them have it as their object; of the listed objects whose every word an article writes, the one with the highest share of right lesson articles is the answer of this third route");

const SURE: f64 = 0.6;
because!(SURE, SureWords, "the share of the lesson articles with a word that must have one object for the word to be sure: more than half, so the word speaks for that object more than for all others together; on the development lessons a higher share left facts unread that the lessons support, and a lower one took words that speak for no object");

const HALF: f64 = 0.5;
because!(HALF, SureWords, "the share of the votes that an object needs: an object that most sure words of the article speak against is not kept");

const FEW_UNDER: u32 = 5;
because!(FEW_UNDER, SureWords, "a word that an article writes fewer times than this, and more than once, stands there a few times; from this number on it stands there often. A word of the subject of an article comes again and again, a word of a passing mention does not");

const EARLY_PARTS: usize = 2;
because!(EARLY_PARTS, RouteChoice, "of the parts of the lesson subjects of a relation, this many are learned from before the routes are compared, so that every relation has subjects of its own to compare on");

const EARLY_OF: usize = 3;
because!(EARLY_OF, RouteChoice, "the parts that the lesson subjects of a relation are cut in for the choice of the routes");

const TENTHS: f64 = 10.0;
because!(TENTHS, RouteChoice, "the levels of a share of votes are its tenths, and the levels of a joined chance are the powers of a tenth that its miss is under");

const TOP_LEVEL: i64 = 9;
because!(TOP_LEVEL, RouteChoice, "the highest level of a chance: above it too few answers stand for a count");

const CLEAR: f64 = 2.0;
because!(CLEAR, RouteChoice, "an order other than the sentence first is taken only when its extra right answers on the held lesson facts of the relation are more than this many times the spread that chance gives to the facts where the two orders differ, so that one or two facts of a few held ones do not decide");

#[derive(Clone, Copy, Debug, Default)]
struct Apart {
    words: (u32, u32),
    listed_after_sentence: (u32, u32),
    listed_after_words: (u32, u32),
}
because!(Apart, RouteChoice, "for one relation, the held lesson facts where two orders differ: sure words first against the sentence first, and a listed object first against each of the two, each as the facts that the other order gets right and this one wrong, and the reverse");

fn tally(pair: &mut (u32, u32), other: u32, base: u32) {
    pair.0 += u32::from(other > base);
    pair.1 += u32::from(base > other);
}
because!(tally, RouteChoice, "one held fact counted for two orders where they differ: the other order right and the base wrong, or the reverse");

fn clearly_better(pair: (u32, u32)) -> bool {
    let (wins, losses) = pair;
    f64::from(wins) - f64::from(losses) > CLEAR * f64::from(wins + losses).sqrt()
}
because!(clearly_better, RouteChoice, "whether an order beats the base order by more than chance gives: the difference of the facts where they differ against its spread");

const LEAST_MISS: f64 = 1e-12;
because!(LEAST_MISS, RouteChoice, "the least miss that a level is taken from, so that a chance of one has a level too");

#[derive(Clone, Debug)]
pub struct SureWord {
    pub relation: u32,
    pub object: String,
    pub hit: u32,
    pub seen: u32,
}
because!(SureWord, SureWords, "what a sure word gives: the relation, the object as the lessons write it, and the articles behind it (with the object, and all)");

const OFTEN: u32 = 2;
because!(OFTEN, SureWords, "the step of a word that an article writes often, after the step of once and the step of a few times");

fn often(count: u32) -> u32 {
    if count == 1 {
        0
    } else if count < FEW_UNDER {
        1
    } else {
        OFTEN
    }
}
because!(often, SureWords, "how often an article writes a word, in steps: once, a few times, often");

pub const OFTEN_NAMES: &[&str] = &["once", "a few times", "often"];
because!(OFTEN_NAMES, SureWords, "the names of the steps of how often an article writes a word, for the trace of an answer");

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Level {
    pub words: bool,
    pub listed: bool,
    pub level: i64,
}
because!(Level, RouteChoice, "the level of the chance of an entry: its route (by sure words, by a listed object, or by a sentence), and the step of its chance");

#[derive(Clone, Debug)]
pub struct ListedObject {
    pub written: String,
    pub words: Vec<u32>,
    pub lessons: u32,
    pub hit: u32,
    pub seen: u32,
}
because!(ListedObject, ListedObjects, "one object of the list of a relation: as a lesson writes it, its words as numbers in their order (each once), the lesson facts with it, and the lesson articles that write every word of it with those of them whose fact has it as object");

#[derive(Default)]
pub struct Chosen {
    pub words_first: HashMap<String, bool>,
    pub route_counts: HashMap<String, (u32, u32)>,
    pub levels: Vec<(Level, u32, u32)>,
    pub listed_first: HashMap<String, bool>,
    pub listed_counts: HashMap<String, (u32, u32)>,
}
because!(Chosen, RouteChoice, "what the count of the routes on the held lesson facts gives: for each relation whether sure words answer before the sentence and whether a listed object answers before both, the right answers of each order, and the measured levels of the chance");

#[derive(Default)]
pub struct Knowledge {
    pub frames: Frames,
    pub sure: HashMap<(u32, u32), Vec<SureWord>>,
    pub sure_words: NumberBag<u32>,
    pub words_first: HashMap<String, bool>,
    pub route_counts: HashMap<String, (u32, u32)>,
    pub levels: Vec<(Level, u32, u32)>,
    pub listed: NumberTable<u32, Vec<ListedObject>>,
    pub listed_of_word: NumberTable<u32, Vec<(u32, u32)>>,
    pub listed_first: HashMap<String, bool>,
    pub listed_counts: HashMap<String, (u32, u32)>,
    pub by_title: HashMap<String, Vec<(u32, Entry)>>,
    pub by_name: HashMap<String, Vec<String>>,
}
because!(Knowledge, SureWords, RouteChoice, ListedObjects, "the reader of facts with its three routes and its table: the counted frames; the sure words by word and how often it stands; the listed objects of each relation, and for each word the listed objects that hold it; for each relation whether the sure words answer first and whether a listed object answers before both, with the counts of the orders; the measured levels of the chance; and what was read, by the title of the article and by each name of a title. learn counts all of this from the lesson facts; routes gives what each route says of one article; read puts an article into the table; answer gives the entry of a subject for a relation with the title of its article; forget takes one article out, and nothing else in the table depends on that article");

fn counted_marks(numbers: impl Iterator<Item = u32>, skip: u32) -> Vec<(u32, u32)> {
    let mut order: Vec<u32> = Vec::new();
    let mut count: NumberTable<u32, u32> = NumberTable::default();
    for number in numbers {
        if number == NO_WORD || number == skip {
            continue;
        }
        let held = count.entry(number).or_insert(0);
        if *held == 0 {
            order.push(number);
        }
        *held += 1;
    }
    order.into_iter().map(|number| (number, count[&number])).collect()
}
because!(counted_marks, SureWords, "the numbers of the words of an article with the number of times that each stands, in the order of their first place; no word, and the number that the caller skips, are left out");

fn marks(text: &str, skip: u32, known: &mut dyn FnMut(&str) -> u32) -> Vec<(u32, u32)> {
    let mut low = String::new();
    counted_marks(
        word_runs(text).into_iter().map(|(start, end)| {
            lowered(&text[start..end], &mut low);
            known(&low)
        }),
        skip,
    )
}
because!(marks, SureWords, "the words of an article with the number of times that it writes each, in the order of their first place; a word that the caller does not know, and the number that the caller skips, are left out");

fn marks_alone(text: &str) -> (Symbols, Vec<(u32, u32)>) {
    let mut words = Symbols::new();
    let found = marks(text, NO_WORD, &mut |low| words.number(low));
    (words, found)
}
because!(marks_alone, SureWords, "the words of an article with their counts, numbered in a table of the article alone, so that many articles are read at once and the numbers of the whole are given after them in their order");

pub fn level_of(entry: &Entry) -> Level {
    let tenth = ((entry.chance * TENTHS) as i64).min(TOP_LEVEL);
    match entry.frame.side {
        Side::Sure => Level { words: true, listed: false, level: tenth },
        Side::Listed => Level { words: false, listed: true, level: tenth },
        _ => {
            let miss = (CERTAIN - entry.chance).max(LEAST_MISS);
            Level { words: false, listed: false, level: ((-miss.log10()) as i64).min(TOP_LEVEL) }
        }
    }
}
because!(level_of, RouteChoice, "the level of the chance of an entry: for an answer that a sentence states the number of nines of its joined chance, for an answer by sure words the tenth of the share of the votes, for a listed object the tenth of its share of right lesson articles");

fn odds(object: &ListedObject) -> f64 {
    (f64::from(object.hit) + MORE_HITS) / (f64::from(object.seen) + MORE_SEEN)
}
because!(odds, ListedObjects, "the share of right lesson articles of a listed object, pulled toward a low share when few articles stand behind it, as the share of a sure word is");

fn better(one: &ListedObject, other: &ListedObject) -> bool {
    let (mine, theirs) = (odds(one), odds(other));
    mine > theirs || (mine == theirs && one.lessons > other.lessons)
}
because!(better, ListedObjects, "whether a found object beats another: a higher share of right lesson articles, and with the same share more lesson facts");

struct SentencePlaces {
    lower: String,
    in_text: Vec<usize>,
    in_lower: Vec<usize>,
}
because!(SentencePlaces, SureWords, "the text of an article in small letters, and the start of each of its sentences as a place in the text and as a place in the text in small letters: a start is counted in signs, because the text in small letters can have other bytes than the text as written, and the place in small letters is the place of the sign with the same count");

fn sentence_places(text: &str) -> SentencePlaces {
    let in_text = sentence_starts(text);
    let mut signs: Vec<usize> = Vec::with_capacity(in_text.len());
    let mut count = 0usize;
    let mut at = 0usize;
    for start in &in_text {
        count += text[at..*start].chars().count();
        at = *start;
        signs.push(count);
    }
    let lower = text.to_lowercase();
    let mut in_lower: Vec<usize> = Vec::with_capacity(signs.len());
    let mut walk = lower.char_indices().enumerate();
    let mut next = walk.next();
    for sign in &signs {
        while let Some((number, _)) = next {
            if number >= *sign {
                break;
            }
            next = walk.next();
        }
        in_lower.push(match next {
            Some((number, (place, _))) if number == *sign => place,
            _ => lower.len(),
        });
    }
    SentencePlaces { lower, in_text, in_lower }
}
because!(sentence_places, SureWords, "the places of the sentences of an article, made once for an article when a route names the sentence of a word");

fn sentence_with<'a>(text: &'a str, places: &SentencePlaces, word: &str) -> &'a str {
    let at = first_word_place(&places.lower, word).unwrap_or(0);
    let sentence = places.in_lower.partition_point(|start| *start <= at).saturating_sub(1);
    let end = places.in_text.get(sentence + 1).copied().unwrap_or(text.len());
    text[places.in_text[sentence]..end].trim_matches(is_space)
}
because!(sentence_with, SureWords, "the sentence of an article where a word first stands as a word of its own, and the first sentence when it stands nowhere");

impl Knowledge {
    fn learn_sure(&mut self, lessons: &[Lesson], lanes: usize) {
        let mut least = 1u32;
        while (f64::from(least) + MORE_HITS) / (f64::from(least) + MORE_SEEN) < SURE {
            least += 1;
        }
        let mut subjects: Vec<(&str, &str)> = Vec::new();
        let mut place_of: HashMap<&str, usize> = HashMap::new();
        for lesson in lessons {
            let Some(text) = lesson.article.filter(|text| !text.is_empty()) else { continue };
            let name = name_of(lesson);
            place_of.entry(name).or_insert_with(|| {
                subjects.push((name, text));
                subjects.len() - 1
            });
        }
        let alone = on_lanes(subjects.len(), lanes, &|place| marks_alone(subjects[place].1));
        let mut words = std::mem::take(&mut self.frames.words);
        let mut marks_of: HashMap<&str, Vec<(u32, u32)>> = HashMap::new();
        for ((name, _), (table, found)) in subjects.iter().zip(alone) {
            let numbers: Vec<u32> = (0..table.len() as u32).map(|place| words.number(table.name(place))).collect();
            marks_of.insert(name, found.into_iter().map(|(word, count)| (numbers[word as usize], often(count))).collect());
        }
        self.frames.words = words;
        let mut facts: Vec<(u32, u32, &str)> = Vec::new();
        let mut seen: NumberTable<(u32, u32, u32), u32> = NumberTable::default();
        let mut written: HashMap<u32, &str> = HashMap::new();
        for lesson in lessons {
            let Some(_) = lesson.article.filter(|text| !text.is_empty()) else { continue };
            let name = name_of(lesson);
            let found = &marks_of[name];
            let relation = self.frames.relations.number(lesson.relation);
            let object = self.frames.objects.number(&key_of(lesson.object));
            written.entry(object).or_insert(lesson.object);
            facts.push((relation, object, name));
            for (word, step) in found.iter() {
                *seen.entry((relation, *word, *step)).or_insert(0) += 1;
            }
        }
        let mut order: Vec<(u32, u32, u32)> = Vec::new();
        let mut hit: NumberTable<(u32, u32, u32), Vec<(u32, u32)>> = NumberTable::default();
        for (relation, object, name) in &facts {
            for (word, step) in &marks_of[name] {
                let key = (*relation, *word, *step);
                if seen[&key] < least {
                    continue;
                }
                let by = hit.entry(key).or_insert_with(|| {
                    order.push(key);
                    Vec::new()
                });
                match by.iter_mut().find(|(known, _)| known == object) {
                    Some(count) => count.1 += 1,
                    None => by.push((*object, 1)),
                }
            }
        }
        for key in order {
            let mut best: Option<(u32, u32)> = None;
            for (object, count) in &hit[&key] {
                if best.is_none_or(|(_, top)| *count > top) {
                    best = Some((*object, *count));
                }
            }
            let Some((object, count)) = best else { continue };
            let all = seen[&key];
            if (f64::from(count) + MORE_HITS) / (f64::from(all) + MORE_SEEN) >= SURE {
                self.sure.entry((key.1, key.2)).or_default().push(SureWord { relation: key.0, object: written[&object].to_string(), hit: count, seen: all });
                self.sure_words.insert(key.1);
            }
        }
    }

    fn learn_listed(&mut self, lessons: &[Lesson], lanes: usize) {
        let mut subjects: Vec<(&str, &str)> = Vec::new();
        let mut place_of: HashMap<&str, usize> = HashMap::new();
        for lesson in lessons {
            let Some(text) = lesson.article.filter(|text| !text.is_empty()) else { continue };
            let name = name_of(lesson);
            place_of.entry(name).or_insert_with(|| {
                subjects.push((name, text));
                subjects.len() - 1
            });
        }
        let frames = &self.frames;
        let present: Vec<Vec<u32>> = on_lanes(subjects.len(), lanes, &|place| {
            let (name, text) = subjects[place];
            let mut numbers = frames.run_numbers(&frames.scanned(name, text));
            numbers.sort_unstable();
            numbers.dedup();
            numbers
        });
        let mut lists: NumberTable<u32, Vec<ListedObject>> = NumberTable::default();
        let mut place_in: HashMap<(u32, String), u32> = HashMap::new();
        let mut facts: Vec<(u32, u32, usize)> = Vec::new();
        for lesson in lessons {
            let relation = self.frames.relations.number(lesson.relation);
            let key = key_of(lesson.object);
            let mut words: Vec<u32> = Vec::new();
            for word in key.split(NAME_BETWEEN) {
                match self.frames.words.find(word) {
                    Some(number) if !words.contains(&number) => words.push(number),
                    Some(_) => {}
                    None => words.clear(),
                }
                if words.is_empty() {
                    break;
                }
            }
            if words.is_empty() {
                continue;
            }
            let list = lists.entry(relation).or_default();
            let place = *place_in.entry((relation, key)).or_insert_with(|| {
                list.push(ListedObject { written: lesson.object.to_string(), words, lessons: 0, hit: 0, seen: 0 });
                (list.len() - 1) as u32
            });
            list[place as usize].lessons += 1;
            if lesson.article.is_some_and(|text| !text.is_empty()) {
                facts.push((relation, place, place_of[name_of(lesson)]));
            }
        }
        let mut of_word: NumberTable<u32, Vec<(u32, u32)>> = NumberTable::default();
        let mut relations: Vec<u32> = lists.keys().copied().collect();
        relations.sort_unstable();
        for relation in relations {
            for (place, object) in lists[&relation].iter().enumerate() {
                for word in &object.words {
                    of_word.entry(*word).or_default().push((relation, place as u32));
                }
            }
        }
        for (relation, truth, subject) in facts {
            let mut matched: NumberTable<u32, usize> = NumberTable::default();
            for word in &present[subject] {
                for (known, place) in of_word.get(word).map_or(&[][..], Vec::as_slice) {
                    if *known == relation {
                        *matched.entry(*place).or_insert(0) += 1;
                    }
                }
            }
            let Some(list) = lists.get_mut(&relation) else { continue };
            for (place, count) in matched {
                let object = &mut list[place as usize];
                if object.words.len() == count {
                    object.seen += 1;
                    object.hit += u32::from(place == truth);
                }
            }
        }
        self.listed = lists;
        self.listed_of_word = of_word;
    }

    pub fn learn_alone(lessons: &[Lesson], classes: &HashMap<String, String>, lanes: usize) -> Knowledge {
        let mut model = Knowledge { frames: Frames::learn(lessons, classes, lanes), ..Knowledge::default() };
        model.learn_sure(lessons, lanes);
        model.learn_listed(lessons, lanes);
        model
    }

    pub fn learn(lessons: &[Lesson], classes: &HashMap<String, String>, lanes: usize) -> Knowledge {
        let mut subjects_of: HashMap<&str, usize> = HashMap::new();
        let mut place_of: HashMap<(&str, &str), usize> = HashMap::new();
        for lesson in lessons {
            let count = subjects_of.entry(lesson.relation).or_insert(0);
            place_of.entry((lesson.relation, name_of(lesson))).or_insert_with(|| {
                *count += 1;
                *count - 1
            });
        }
        let late = |lesson: &Lesson| place_of[&(lesson.relation, name_of(lesson))] >= subjects_of[lesson.relation] * EARLY_PARTS / EARLY_OF;
        let first: Vec<Lesson> = lessons.iter().filter(|lesson| !late(lesson)).copied().collect();
        let last: Vec<Lesson> = lessons.iter().filter(|lesson| late(lesson)).copied().collect();
        let (mut model, chosen) = std::thread::scope(|scope| {
            let choice = scope.spawn(|| Knowledge::learn_alone(&first, classes, lanes).routes_compared(&last, lanes));
            let model = Knowledge::learn_alone(lessons, classes, lanes);
            (model, choice.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault)))
        });
        model.words_first = chosen.words_first;
        model.route_counts = chosen.route_counts;
        model.levels = chosen.levels;
        model.listed_first = chosen.listed_first;
        model.listed_counts = chosen.listed_counts;
        model
    }

    fn routes_compared(&self, last: &[Lesson], lanes: usize) -> Chosen {
        let mut order: Vec<&str> = Vec::new();
        let mut asked: HashMap<&str, Vec<&Lesson>> = HashMap::new();
        for lesson in last {
            if lesson.article.is_some_and(|text| !text.is_empty()) {
                let name = name_of(lesson);
                asked.entry(name).or_insert_with(|| {
                    order.push(name);
                    Vec::new()
                })
                .push(lesson);
            }
        }
        let mut right: HashMap<String, (u32, u32)> = HashMap::new();
        let mut listed_right: HashMap<String, (u32, u32)> = HashMap::new();
        let mut apart: HashMap<String, Apart> = HashMap::new();
        let mut levels: Vec<(Level, u32, u32)> = Vec::new();
        let routes = on_lanes(order.len(), lanes, &|place| self.routes(order[place], asked[order[place]][0].article.unwrap_or_default()));
        for (name, (read, words, listed)) in order.iter().zip(routes) {
            let facts = &asked[name];
            for fact in facts {
                let truth = key_of(fact.object);
                let relation = self.frames.relations.find(fact.relation);
                let by_sentence = relation.and_then(|relation| read.iter().find(|(known, _)| *known == relation)).map(|(_, entry)| entry);
                let by_words = relation.and_then(|relation| words.iter().find(|(known, _)| *known == relation)).map(|(_, entry)| entry);
                let by_listed = relation.and_then(|relation| listed.iter().find(|(known, _)| *known == relation)).map(|(_, entry)| entry);
                let is_right = |entry: Option<&Entry>| u32::from(entry.is_some_and(|entry| key_of(&entry.answer) == truth));
                let sentence_first = is_right(by_sentence.or(by_words).or(by_listed));
                let words_first = is_right(by_words.or(by_sentence).or(by_listed));
                let listed_after_sentence = is_right(by_listed.or(by_sentence).or(by_words));
                let listed_after_words = is_right(by_listed.or(by_words).or(by_sentence));
                let count = right.entry(fact.relation.to_string()).or_insert((0, 0));
                count.0 += sentence_first;
                count.1 += words_first;
                let count = listed_right.entry(fact.relation.to_string()).or_insert((0, 0));
                count.0 += listed_after_sentence;
                count.1 += listed_after_words;
                let differ = apart.entry(fact.relation.to_string()).or_default();
                tally(&mut differ.words, words_first, sentence_first);
                tally(&mut differ.listed_after_sentence, listed_after_sentence, sentence_first);
                tally(&mut differ.listed_after_words, listed_after_words, words_first);
                for entry in [by_sentence, by_words, by_listed].into_iter().flatten() {
                    let level = level_of(entry);
                    let place = match levels.iter().position(|(known, _, _)| *known == level) {
                        Some(place) => place,
                        None => {
                            levels.push((level, 0, 0));
                            levels.len() - 1
                        }
                    };
                    levels[place].1 += u32::from(key_of(&entry.answer) == truth);
                    levels[place].2 += 1;
                }
            }
        }
        let words_first: HashMap<String, bool> = apart.iter().map(|(relation, differ)| (relation.clone(), clearly_better(differ.words))).collect();
        let listed_first = apart.iter().map(|(relation, differ)| (relation.clone(), clearly_better(if words_first[relation] { differ.listed_after_words } else { differ.listed_after_sentence }))).collect();
        Chosen { words_first, route_counts: right, levels, listed_first, listed_counts: listed_right }
    }

    fn by_listed(&self, scan: &Scan) -> Vec<(u32, Entry)> {
        if self.listed.is_empty() {
            return Vec::new();
        }
        let mut present = self.frames.run_numbers(scan);
        present.sort_unstable();
        present.dedup();
        let mut matched: NumberTable<(u32, u32), usize> = NumberTable::default();
        for word in &present {
            for key in self.listed_of_word.get(word).map_or(&[][..], Vec::as_slice) {
                *matched.entry(*key).or_insert(0) += 1;
            }
        }
        let mut found: Vec<(u32, u32)> = matched.into_iter().filter(|((relation, place), count)| self.listed[relation][*place as usize].words.len() == *count).map(|(key, _)| key).collect();
        found.sort_unstable();
        let mut best: Vec<(u32, u32)> = Vec::new();
        for (relation, place) in found {
            let list = &self.listed[&relation];
            match best.last_mut() {
                Some(top) if top.0 == relation => {
                    if better(&list[place as usize], &list[top.1 as usize]) {
                        top.1 = place;
                    }
                }
                _ => best.push((relation, place)),
            }
        }
        if best.is_empty() {
            return Vec::new();
        }
        let places = sentence_places(scan.text);
        best.into_iter()
            .map(|(relation, place)| {
                let object = &self.listed[&relation][place as usize];
                let first = self.frames.words.name(object.words[0]);
                let shown = &object.words[..object.words.len().min(FRAME_WORDS)];
                (relation, Entry { answer: object.written.clone(), chance: odds(object), sentence: sentence_with(scan.text, &places, first).to_string(), frame: Frame { side: Side::Listed, words: padded(shown) } })
            })
            .collect()
    }

    pub fn listed_of(&self, relation: u32, answer: &str) -> Option<&ListedObject> {
        self.listed.get(&relation)?.iter().find(|object| object.written == answer)
    }

    fn by_sure_words(&self, scan: &Scan) -> Vec<(u32, Entry)> {
        if self.sure.is_empty() {
            return Vec::new();
        }
        let text = scan.text;
        let found = counted_marks(self.frames.run_numbers(scan).into_iter().map(|number| if self.sure_words.contains(&number) { number } else { NO_WORD }), self.frames.title_place);
        let mut order: Vec<u32> = Vec::new();
        let mut votes: HashMap<u32, Vec<(&str, u32, u32, (u32, u32))>> = HashMap::new();
        for (word, count) in found {
            let mark = (word, often(count));
            for sure in self.sure.get(&mark).map_or(&[][..], Vec::as_slice) {
                let by = votes.entry(sure.relation).or_insert_with(|| {
                    order.push(sure.relation);
                    Vec::new()
                });
                match by.iter_mut().find(|(object, _, _, _)| *object == sure.object) {
                    Some(vote) => {
                        vote.1 += count;
                        if count > vote.2 {
                            vote.2 = count;
                            vote.3 = mark;
                        }
                    }
                    None => by.push((sure.object.as_str(), count, count, mark)),
                }
            }
        }
        let mut entries: Vec<(u32, Entry)> = Vec::new();
        let mut places: Option<SentencePlaces> = None;
        for relation in order {
            let by = &votes[&relation];
            let mut best: Option<&(&str, u32, u32, (u32, u32))> = None;
            for vote in by {
                if best.is_none_or(|top| vote.1 > top.1) {
                    best = Some(vote);
                }
            }
            let Some((object, count, _, mark)) = best else { continue };
            let all: u32 = by.iter().map(|vote| vote.1).sum();
            let share = f64::from(*count) / f64::from(all);
            if share < HALF {
                continue;
            }
            let held = places.get_or_insert_with(|| sentence_places(text));
            let word = self.frames.words.name(mark.0);
            entries.push((relation, Entry { answer: object.to_string(), chance: share, sentence: sentence_with(text, held, word).to_string(), frame: Frame { side: Side::Sure, words: [mark.0, mark.1, NO_WORD] } }));
        }
        entries
    }

    pub fn routes(&self, title: &str, text: &str) -> (Vec<(u32, Entry)>, Vec<(u32, Entry)>, Vec<(u32, Entry)>) {
        let scan = self.frames.scanned(title, text);
        (self.frames.entries(&scan), self.by_sure_words(&scan), self.by_listed(&scan))
    }

    pub fn entries(&self, title: &str, text: &str) -> Vec<(u32, Entry)> {
        let (mut entries, words, listed) = self.routes(title, text);
        for (first_of, rows) in [(&self.words_first, words), (&self.listed_first, listed)] {
            for (relation, row) in rows {
                let first = first_of.get(self.frames.relations.name(relation)).copied().unwrap_or(false);
                match entries.iter_mut().find(|(known, _)| *known == relation) {
                    Some(held) if first => held.1 = row,
                    Some(_) => {}
                    None => entries.push((relation, row)),
                }
            }
        }
        entries
    }

    pub fn put(&mut self, title: &str, entries: Vec<(u32, Entry)>) {
        if entries.is_empty() {
            return;
        }
        self.by_title.insert(title.to_string(), entries);
        for name in names(title) {
            self.by_name.entry(name).or_default().push(title.to_string());
        }
    }

    pub fn read(&mut self, title: &str, text: &str) {
        let entries = self.entries(title, text);
        self.put(title, entries);
    }

    pub fn answer(&self, relation: &str, subject: &str) -> Option<(&Entry, &str)> {
        let relation = self.frames.relations.find(relation)?;
        let alone = [subject.to_string()];
        let titles: &[String] = if self.by_title.contains_key(subject) { &alone } else { self.by_name.get(&subject.to_lowercase()).map_or(&[][..], Vec::as_slice) };
        let mut best: Option<(&Entry, &str)> = None;
        for title in titles {
            let Some((held, entries)) = self.by_title.get_key_value(title.as_str()) else { continue };
            let Some((_, entry)) = entries.iter().find(|(known, _)| *known == relation) else { continue };
            if best.is_none_or(|(top, _)| entry.chance > top.chance) {
                best = Some((entry, held.as_str()));
            }
        }
        best
    }

    pub fn holds<'a>(&'a self, entry: &'a Entry) -> &'a str {
        if entry.frame.side == Side::Sure { self.frames.words.name(entry.frame.words[0]) } else { entry.answer.as_str() }
    }

    pub fn forget(&mut self, title: &str) {
        if self.by_title.remove(title).is_some() {
            for name in names(title) {
                if let Some(titles) = self.by_name.get_mut(&name) {
                    if let Some(place) = titles.iter().position(|held| held == title) {
                        titles.remove(place);
                    }
                }
            }
        }
    }

    pub fn measured(&self, entry: &Entry) -> (u32, u32) {
        let level = level_of(entry);
        self.levels.iter().find(|(known, _, _)| *known == level).map_or((0, 0), |(_, right, answers)| (*right, *answers))
    }
}

fn first_word_place(lower: &str, word: &str) -> Option<usize> {
    if word.is_empty() {
        return None;
    }
    let mut from = 0usize;
    while let Some(found) = lower[from..].find(word) {
        let start = from + found;
        let end = start + word.len();
        let before = lower[..start].chars().next_back().is_some_and(is_word_sign);
        let after = lower[end..].chars().next().is_some_and(is_word_sign);
        if !before && !after {
            return Some(start);
        }
        from = start + lower[start..].chars().next().map_or(1, char::len_utf8);
    }
    None
}
because!(first_word_place, SureWords, "the first place of a text where a word stands as a word of its own, with no word sign before it and none after it");

