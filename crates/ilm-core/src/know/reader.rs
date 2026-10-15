use crate::base::first_sight::{NumberBag, NumberHashes, NumberTable};
use crate::base::lanes::on_lanes;
use crate::base::keys::{NO_NUMBER, padded};
use crate::base::symbols::Symbols;
use crate::know::article::{Article, lowered, parse, sentence_of};
use crate::know::signs::is_word_sign;
use crate::know::titles::head;
use patterns::{because, source};
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct CountedFrames;
source!(CountedFrames, "a reader of facts by counted frames: a frame is what stands around a candidate in an article (the one to three words before it, the one or two words after it, or both sides, with the title of the article as one place); for each frame and each relation the lessons give how often the frame stood around a candidate in the article of a subject with a fact of the relation, and how often that candidate was the object of the fact; no relation and no word is named in the program");

pub struct NameCandidates;
source!(NameCandidates, "the candidates of an article for the object of a fact: a name, which is the longest run of words with a capital letter next to each other in one sentence, and an object of the lesson facts wherever the article writes it; so an answer can be a name of several words that no lesson holds");

pub struct SplitCounts;
source!(SplitCounts, "the frames of the lesson articles are counted on all threads: each article is cut and its words are numbered in a table of its own, and the numbers of the whole are then given in the order of the articles and of first sight within each, so every word gets the number that one pass in order gives; the counts are whole sums, so each thread counts its articles into tables split by the hash of the key, and the parts of one hash are summed apart; the same counts on every number of threads");

pub type CountKey = (Frame, u32);
because!(CountKey, SplitCounts, "the key of a count: a frame and a relation");

pub type CountTable = NumberTable<CountKey, (u32, u32)>;
because!(CountTable, SplitCounts, "the hits and the readings of each frame and relation, as one thread counts them in one part of the hash");

pub type KeptCount = (Frame, u32, u32, u32);
because!(KeptCount, SplitCounts, "a frame and relation that is kept, with its hits and its readings");

pub type LessonArticle<'a> = (&'a str, &'a str, &'a [(u32, u32)]);
because!(LessonArticle, SplitCounts, "one lesson article as it is counted: its name, its first signs, and the relations of its lesson facts with their objects");

pub struct JoinedChance;
source!(JoinedChance, "the shares of the frames of one candidate are joined as chances that do not depend on each other, the noisy-or of Pearl: one minus the product of the misses; the candidate with the highest joined chance is kept for a relation, with the sentence of its best frame, so every answer names where it was read");

const LEAST: u32 = 3;
because!(LEAST, CountedFrames, "a frame is kept for a relation only with this many readings or more: fewer readings are the chance of one article");

const LEAST_CHANCE: f64 = 0.5;
because!(LEAST_CHANCE, JoinedChance, "a candidate is kept for a relation at this joined chance or more: under it an answer is wrong more often than right, and the table grows by a third with nearly no right answer more");

pub const MORE_HITS: f64 = 1.0;
because!(MORE_HITS, CountedFrames, "what is added to the hits of a frame before its share: with the readings that are added, a frame that was right every time of a few readings stands under a frame that was right every time of many");

pub const MORE_SEEN: f64 = 3.0;
because!(MORE_SEEN, CountedFrames, "what is added to the readings of a frame before its share: more than is added to the hits, so that the share of a frame with few readings is drawn down and no share is a certainty");

pub const CERTAIN: f64 = 1.0;
because!(CERTAIN, JoinedChance, "the chance of a sure thing; a miss is what is left of it");

const NAME_MOST: usize = 6;
because!(NAME_MOST, NameCandidates, "the most words of one name: longer runs of capital words are headings and lists, not names");

const LESSON_SIGNS: usize = 12000;
because!(LESSON_SIGNS, CountedFrames, "the frames are counted in this many first signs of a lesson article: the whole article gives the same share of right answers and takes twice the time");

const JOINS: &[&str] = &[" ", "-", ". ", ".", "'", "\u{2019}"];
because!(JOINS, NameCandidates, "what can stand between two words of one name: a space, a hyphen, the point of a short form with or without a space, and the two forms of the apostrophe");

const BEFORE: usize = 3;
because!(BEFORE, CountedFrames, "the most words before a candidate that a frame holds: longer frames come again too seldom to be counted");

const AFTER: usize = 2;
because!(AFTER, CountedFrames, "the most words after a candidate that a frame holds");

const TITLE_MARK: &str = "<x>";
because!(TITLE_MARK, CountedFrames, "the word of a frame that stands for the title of the article; a word of a text never holds these signs");

pub const NAME_BETWEEN: &str = " ";
because!(NAME_BETWEEN, NameCandidates, "the mark between the words of a candidate of several words when it is compared with an object");

pub const FRAME_WORDS: usize = 3;
because!(FRAME_WORDS, CountedFrames, "the places for words in a frame: as many as the longest frame holds");

pub const NO_WORD: u32 = NO_NUMBER;
because!(NO_WORD, CountedFrames, "the number of a word that no lesson article holds, and of an empty place of a frame; no kept frame holds such a word");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Side {
    Left,
    Right,
    Both,
    Class,
    First,
    Later,
    Sure,
    Listed,
}
because!(Side, CountedFrames, "the sort of a frame: words before the candidate, words after it, words on both sides, the class of the candidate, the two frames with no word (the first candidate of the article that is an object of the relation in the lessons, and a later one), a sure word of the article, and an object of the lessons whose every word the article writes");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Frame {
    pub side: Side,
    pub words: [u32; FRAME_WORDS],
}
because!(Frame, CountedFrames, "a frame as numbers: its sort, and its words in the order of the text");

fn known_frame(side: Side, words: &[u32]) -> Option<Frame> {
    (!words.contains(&NO_WORD)).then(|| Frame { side, words: padded(words) })
}
because!(known_frame, CountedFrames, "a frame when every word of it is a word of the lesson articles: a frame with another word was never counted, and with an empty place for that word it would be taken for a shorter frame");

pub fn key_of(text: &str) -> String {
    let mut key = String::new();
    let mut word = String::new();
    let mut low = String::new();
    for sign in text.chars().chain(std::iter::once(char::default())) {
        if is_word_sign(sign) {
            word.push(sign);
            continue;
        }
        if !word.is_empty() {
            if !key.is_empty() {
                key.push_str(NAME_BETWEEN);
            }
            lowered(&word, &mut low);
            key.push_str(&low);
            word.clear();
        }
    }
    key
}
because!(key_of, NameCandidates, "a name as its words in small letters with one space between them, so that a name is compared without its case and its marks");

#[derive(Clone, Copy, Debug)]
pub struct Lesson<'a> {
    pub relation: &'a str,
    pub subject: &'a str,
    pub object: &'a str,
    pub title: Option<&'a str>,
    pub article: Option<&'a str>,
}
because!(Lesson, CountedFrames, "a lesson fact: the relation, the subject, the object, and the article of the subject with its title when the text holds one");

pub fn name_of<'a>(lesson: &Lesson<'a>) -> &'a str {
    lesson.title.filter(|title| !title.is_empty()).unwrap_or(lesson.subject)
}
because!(name_of, CountedFrames, "the name under which the article of a lesson fact is counted: its title, or the subject when it has no title");

#[derive(Clone, Debug)]
pub struct Entry {
    pub answer: String,
    pub chance: f64,
    pub sentence: String,
    pub frame: Frame,
}
because!(Entry, JoinedChance, "what is kept of one article for one relation: the candidate as the article writes it, its joined chance, the sentence where its best frame stands, and that frame");

pub struct Scan<'a> {
    pub text: &'a str,
    pub article: Article,
    pub numbers: Vec<u32>,
}
because!(Scan, CountedFrames, "an article that was cut into tokens, with the number of each token: the number of its word in small letters, the number of the title mark, or no word; the two routes of an article read one scan, so that each word is looked up once");

#[derive(Clone, Debug)]
pub struct Span {
    pub first: usize,
    pub end: usize,
    pub candidate: u32,
    pub object: Option<u32>,
}
because!(Span, NameCandidates, "one candidate of an article: its first token and the token after its last, the number of its words in small letters within the article (the same words have the same number), and the number of the object of the lessons that it is, when it is one");

#[derive(Default)]
pub struct Frames {
    pub relations: Symbols,
    pub words: Symbols,
    pub objects: Symbols,
    pub facts_of: NumberTable<(u32, u32), u32>,
    pub of_object: Vec<Vec<u32>>,
    pub object_of_word: NumberTable<u32, u32>,
    pub longer: NumberTable<u32, Vec<usize>>,
    pub classes: NumberTable<String, u32>,
    pub share: NumberTable<Frame, Vec<(u32, f64)>>,
    pub place_share: NumberTable<(Side, u32), f64>,
    pub counts: NumberTable<(Frame, u32), (u32, u32)>,
    pub title_place: u32,
}
because!(Frames, CountedFrames, "what the reader counted from the lesson facts: the relations, the words of the lesson articles and the objects as numbers; for each relation and object the lesson facts; for each object its relations; the objects of one word by the number of the word; for the first word of an object of several words their lengths, the longest first; the classes of words when a table of them is given; for each kept frame the share of each relation, and the hits and readings behind it, with the shares of the two frames of the places by relation for a quick look up; the counts are held for the kept frames only, which are all that an entry can name");

impl Frames {
    fn scan<'a>(&self, title: &str, text: &'a str, number: &mut dyn FnMut(&str) -> u32) -> Scan<'a> {
        let article = parse(title, text);
        let mut low = String::new();
        let numbers = article
            .tokens
            .iter()
            .map(|token| {
                if token.title {
                    return self.title_place;
                }
                lowered(&text[token.start..token.end], &mut low);
                number(&low)
            })
            .collect();
        Scan { text, article, numbers }
    }

    pub fn scanned<'a>(&self, title: &str, text: &'a str) -> Scan<'a> {
        self.scan(head(title), text, &mut |low| self.words.find(low).unwrap_or(NO_WORD))
    }

    pub fn run_numbers(&self, scan: &Scan) -> Vec<u32> {
        let tokens = &scan.article.tokens;
        let mut low = String::new();
        let mut at = 0usize;
        scan.article
            .runs
            .iter()
            .map(|(start, end)| {
                while at < tokens.len() && tokens[at].title && tokens[at].end <= *start {
                    at += 1;
                }
                if at < tokens.len() && !tokens[at].title && tokens[at].start == *start {
                    at += 1;
                    return scan.numbers[at - 1];
                }
                lowered(&scan.text[*start..*end], &mut low);
                self.words.find(&low).unwrap_or(NO_WORD)
            })
            .collect()
    }

    fn low_of(&self, scan: &Scan, token: usize, into: &mut String) {
        if scan.article.tokens[token].title {
            into.clear();
            into.push_str(TITLE_MARK);
        } else {
            lowered(&scan.text[scan.article.tokens[token].start..scan.article.tokens[token].end], into);
        }
    }

    fn key_into(&self, scan: &Scan, first: usize, end: usize, key: &mut String, low: &mut String) {
        key.clear();
        for token in first..end {
            if token > first {
                key.push_str(NAME_BETWEEN);
            }
            self.low_of(scan, token, low);
            key.push_str(low);
        }
    }

    pub fn spans(&self, scan: &Scan) -> Vec<Span> {
        let tokens = &scan.article.tokens;
        let text = scan.text;
        let begins = |place: usize| scan.article.starts.binary_search(&place).is_ok();
        let capital = |token: usize| text[tokens[token].start..].chars().next().is_some_and(char::is_uppercase);
        let mut small_known: NumberBag<u32> = NumberBag::default();
        let mut small_new = Symbols::new();
        let mut low = String::new();
        for (token, place) in tokens.iter().enumerate() {
            if text[place.start..].chars().next().is_some_and(char::is_lowercase) {
                if scan.numbers[token] == NO_WORD {
                    self.low_of(scan, token, &mut low);
                    small_new.number(&low);
                } else {
                    small_known.insert(scan.numbers[token]);
                }
            }
        }
        let mut found: Vec<Span> = Vec::new();
        let mut names = Symbols::new();
        let mut key = String::new();
        let mut at = 0usize;
        while at < tokens.len() {
            if tokens[at].title {
                at += 1;
                continue;
            }
            let mut taken = 0usize;
            for size in self.longer.get(&scan.numbers[at]).map_or(&[][..], Vec::as_slice) {
                if at + size <= tokens.len() {
                    self.key_into(scan, at, at + size, &mut key, &mut low);
                    if self.objects.find(&key).is_some() {
                        taken = *size;
                        break;
                    }
                }
            }
            let written_small = if scan.numbers[at] == NO_WORD {
                self.low_of(scan, at, &mut low);
                small_new.find(&low).is_some()
            } else {
                small_known.contains(&scan.numbers[at])
            };
            if capital(at) && (!begins(tokens[at].start) || !written_small) {
                let mut end = at + 1;
                while end < tokens.len() && end - at < NAME_MOST && !tokens[end].title && capital(end) && !begins(tokens[end].start) && JOINS.contains(&&text[tokens[end - 1].end..tokens[end].start]) {
                    end += 1;
                }
                taken = taken.max(end - at);
            }
            if taken == 0 {
                if let Some(object) = self.object_of_word.get(&scan.numbers[at]) {
                    self.key_into(scan, at, at + 1, &mut key, &mut low);
                    found.push(Span { first: at, end: at + 1, candidate: names.number(&key), object: Some(*object) });
                }
                at += 1;
                continue;
            }
            self.key_into(scan, at, at + taken, &mut key, &mut low);
            let object = self.objects.find(&key);
            found.push(Span { first: at, end: at + taken, candidate: names.number(&key), object });
            if taken > 1 {
                for inner in at..at + taken {
                    if let Some(object) = self.object_of_word.get(&scan.numbers[inner]) {
                        self.key_into(scan, inner, inner + 1, &mut key, &mut low);
                        found.push(Span { first: inner, end: inner + 1, candidate: names.number(&key), object: Some(*object) });
                    }
                }
            }
            at += taken;
        }
        found
    }

    pub fn frames_into(&self, scan: &Scan, first: usize, end: usize, found: &mut Vec<Frame>, word: &mut String) {
        let numbers = &scan.numbers;
        let low = &numbers[first.saturating_sub(BEFORE)..first];
        let after = &numbers[end..(end + AFTER).min(numbers.len())];
        found.clear();
        for size in 1..=low.len() {
            found.extend(known_frame(Side::Left, &low[low.len() - size..]));
        }
        if !after.is_empty() {
            found.extend(known_frame(Side::Right, &after[..1]));
            if after.len() > 1 {
                found.extend(known_frame(Side::Right, after));
            }
            if let Some(last) = low.last() {
                found.extend(known_frame(Side::Both, &[*last, after[0]]));
                if low.len() > 1 {
                    found.extend(known_frame(Side::Both, &[low[low.len() - 1 - 1], *last, after[0]]));
                }
            }
        }
        if end - first == 1 && !self.classes.is_empty() {
            self.low_of(scan, first, word);
            if let Some(class) = self.classes.get(word.as_str()) {
                found.push(Frame { side: Side::Class, words: padded(&[*class]) });
                if let Some(last) = low.last() {
                    found.extend(known_frame(Side::Class, &[*class, *last]));
                }
            }
        }
    }

    fn read_alone(&self, name: &str, text: &str) -> (Article, Vec<u32>, Symbols) {
        let article = parse(head(name), text);
        let mut words = Symbols::new();
        let title = words.number(TITLE_MARK);
        let mut low = String::new();
        let local = article
            .tokens
            .iter()
            .map(|token| {
                if token.title {
                    return title;
                }
                lowered(&text[token.start..token.end], &mut low);
                words.number(&low)
            })
            .collect();
        (article, local, words)
    }

    fn count_article(&self, scan: &Scan, mine: &[(u32, u32)], parts: &mut [CountTable]) {
        let hashes = NumberHashes::default();
        let split = parts.len() as u64;
        let mut add = |key: CountKey, hit: bool| {
            let part = if split > 1 { (hashes.hash_one(key) % split) as usize } else { 0 };
            let count = parts[part].entry(key).or_insert((0, 0));
            count.0 += u32::from(hit);
            count.1 += 1;
        };
        let mut first = vec![false; self.relations.len()];
        let mut found: Vec<Frame> = Vec::new();
        let mut word = String::new();
        for span in self.spans(scan) {
            self.frames_into(scan, span.first, span.end, &mut found, &mut word);
            for (relation, truth) in mine {
                let is_truth = span.object == Some(*truth);
                for one in &found {
                    add((*one, *relation), is_truth);
                }
                let others = span.object.and_then(|object| self.facts_of.get(&(*relation, object))).copied().unwrap_or(0);
                if others > u32::from(is_truth) {
                    let side = if first[*relation as usize] { Side::Later } else { Side::First };
                    first[*relation as usize] = true;
                    add((Frame { side, words: padded(&[]) }, *relation), is_truth);
                }
            }
        }
    }

    fn counted(&self, scans: &[Scan], facts: &[&[(u32, u32)]], lanes: usize) -> Vec<KeptCount> {
        let parts = lanes.max(1);
        let workers = lanes.clamp(1, scans.len().max(1));
        let next = AtomicUsize::new(0);
        let tables: Vec<Vec<CountTable>> = std::thread::scope(|scope| {
            let hands: Vec<_> = (0..workers)
                .map(|_| {
                    scope.spawn(|| {
                        let mut mine: Vec<CountTable> = (0..parts).map(|_| CountTable::default()).collect();
                        loop {
                            let place = next.fetch_add(1, Ordering::Relaxed);
                            if place >= scans.len() {
                                break;
                            }
                            self.count_article(&scans[place], facts[place], &mut mine);
                        }
                        mine
                    })
                })
                .collect();
            hands.into_iter().map(|hand| hand.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault))).collect()
        });
        let mut by_part: Vec<Vec<CountTable>> = (0..parts).map(|_| Vec::with_capacity(workers)).collect();
        for table in tables {
            for (part, counts) in table.into_iter().enumerate() {
                by_part[part].push(counts);
            }
        }
        let by_part: Vec<std::sync::Mutex<Vec<CountTable>>> = by_part.into_iter().map(std::sync::Mutex::new).collect();
        let kept = on_lanes(parts, lanes, &|part| {
            let mut held = by_part[part].lock().map(|mut held| std::mem::take(&mut *held)).unwrap_or_default();
            held.sort_by_key(|counts| std::cmp::Reverse(counts.len()));
            let mut rest = held.into_iter();
            let mut whole = rest.next().unwrap_or_default();
            for table in rest {
                for (key, (hit, seen)) in table {
                    let count = whole.entry(key).or_insert((0, 0));
                    count.0 += hit;
                    count.1 += seen;
                }
            }
            whole.into_iter().filter(|(_, (hit, seen))| *seen >= LEAST && *hit > 0).map(|((one, relation), (hit, seen))| (one, relation, hit, seen)).collect::<Vec<KeptCount>>()
        });
        kept.into_iter().flatten().collect()
    }

    pub fn learn(lessons: &[Lesson], classes: &HashMap<String, String>, lanes: usize) -> Frames {
        let mut model = Frames::default();
        model.title_place = model.words.number(TITLE_MARK);
        for (word, class) in classes {
            let number = model.words.number(class);
            model.classes.insert(word.clone(), number);
        }
        for lesson in lessons {
            let relation = model.relations.number(lesson.relation);
            let object = model.objects.number(&key_of(lesson.object));
            *model.facts_of.entry((relation, object)).or_insert(0) += 1;
        }
        model.of_object = vec![Vec::new(); model.objects.len()];
        let mut pairs: Vec<(u32, u32)> = model.facts_of.keys().copied().collect();
        pairs.sort_unstable();
        for (relation, object) in pairs {
            model.of_object[object as usize].push(relation);
        }
        for object in 0..model.objects.len() as u32 {
            let key = model.objects.name(object).to_string();
            let size = key.split(NAME_BETWEEN).count();
            let first = model.words.number(key.split(NAME_BETWEEN).next().unwrap_or_default());
            if size == 1 {
                model.object_of_word.insert(first, object);
            } else {
                let sizes = model.longer.entry(first).or_default();
                if !sizes.contains(&size) {
                    sizes.push(size);
                    sizes.sort_unstable_by(|one, other| other.cmp(one));
                }
            }
        }
        let mut order: Vec<&str> = Vec::new();
        let mut by_subject: HashMap<&str, (&str, Vec<(u32, u32)>)> = HashMap::new();
        for lesson in lessons {
            let Some(text) = lesson.article.filter(|text| !text.is_empty()) else { continue };
            let name = name_of(lesson);
            let relation = model.relations.number(lesson.relation);
            let truth = model.objects.number(&key_of(lesson.object));
            let mine = by_subject.entry(name).or_insert_with(|| {
                order.push(name);
                (text, Vec::new())
            });
            mine.0 = text;
            match mine.1.iter_mut().find(|(known, _)| *known == relation) {
                Some(fact) => fact.1 = truth,
                None => mine.1.push((relation, truth)),
            }
        }
        let articles: Vec<LessonArticle> = order
            .iter()
            .map(|name| {
                let (whole, mine) = &by_subject[name];
                (*name, whole.char_indices().nth(LESSON_SIGNS).map_or(*whole, |(end, _)| &whole[..end]), mine.as_slice())
            })
            .collect();
        let read = on_lanes(articles.len(), lanes, &|place| model.read_alone(articles[place].0, articles[place].1));
        let mut scans: Vec<Scan> = Vec::with_capacity(read.len());
        for ((article, local, words), (_, text, _)) in read.into_iter().zip(&articles) {
            let numbers: Vec<u32> = (0..words.len() as u32).map(|place| model.words.number(words.name(place))).collect();
            scans.push(Scan { text, article, numbers: local.iter().map(|place| numbers[*place as usize]).collect() });
        }
        let facts: Vec<&[(u32, u32)]> = articles.iter().map(|(_, _, mine)| *mine).collect();
        let mut kept = model.counted(&scans, &facts, lanes);
        kept.sort_unstable_by(|one, other| (one.0, one.1).cmp(&(other.0, other.1)));
        for (one, relation, hit, seen) in kept {
            let share = (f64::from(hit) + MORE_HITS) / (f64::from(seen) + MORE_SEEN);
            model.counts.insert((one, relation), (hit, seen));
            model.share.entry(one).or_default().push((relation, share));
            if matches!(one.side, Side::First | Side::Later) {
                model.place_share.insert((one.side, relation), share);
            }
        }
        model
    }

    pub fn entries(&self, scan: &Scan) -> Vec<(u32, Entry)> {
        struct Slot {
            relation: u32,
            miss: f64,
            best: f64,
            span: usize,
            frame: Frame,
        }
        let text = scan.text;
        let spans = self.spans(scan);
        let relations = self.relations.len();
        let candidates = spans.iter().map(|span| span.candidate as usize + 1).max().unwrap_or(0);
        let mut slot_of: Vec<u32> = vec![NO_NUMBER; candidates * relations];
        let mut found: Vec<Frame> = Vec::new();
        let mut word = String::new();
        let mut slots: Vec<Slot> = Vec::new();
        let mut first = vec![false; self.relations.len()];
        let mut vote_share = vec![f64::default(); self.relations.len()];
        let mut vote_frame = vec![Frame { side: Side::Left, words: padded(&[]) }; self.relations.len()];
        let mut voted = vec![false; self.relations.len()];
        let mut touched: Vec<u32> = Vec::new();
        for (place, span) in spans.iter().enumerate() {
            let candidate = span.candidate as usize;
            for relation in touched.drain(..) {
                voted[relation as usize] = false;
            }
            self.frames_into(scan, span.first, span.end, &mut found, &mut word);
            for one in found.iter().copied() {
                for (relation, share) in self.share.get(&one).map_or(&[][..], Vec::as_slice) {
                    let at = *relation as usize;
                    if !voted[at] {
                        voted[at] = true;
                        touched.push(*relation);
                        vote_share[at] = *share;
                        vote_frame[at] = one;
                    } else if *share > vote_share[at] {
                        vote_share[at] = *share;
                        vote_frame[at] = one;
                    }
                }
            }
            for relation in span.object.map_or(&[][..], |object| self.of_object[object as usize].as_slice()) {
                let at = *relation as usize;
                let side = if first[at] { Side::Later } else { Side::First };
                first[at] = true;
                let one = Frame { side, words: padded(&[]) };
                let Some(share) = self.place_share.get(&(side, *relation)).copied() else { continue };
                if !voted[at] {
                    voted[at] = true;
                    touched.push(*relation);
                    vote_share[at] = CERTAIN - (CERTAIN - share);
                    vote_frame[at] = one;
                    continue;
                }
                if share > vote_share[at] {
                    vote_frame[at] = one;
                }
                vote_share[at] = CERTAIN - (CERTAIN - vote_share[at]) * (CERTAIN - share);
            }
            for relation in &touched {
                let (relation, share, one) = (*relation, vote_share[*relation as usize], vote_frame[*relation as usize]);
                let key = candidate * relations + relation as usize;
                if slot_of[key] == NO_NUMBER {
                    slot_of[key] = slots.len() as u32;
                    slots.push(Slot { relation, miss: CERTAIN, best: f64::default(), span: place, frame: one });
                }
                let slot = slot_of[key] as usize;
                slots[slot].miss *= CERTAIN - share;
                if share > slots[slot].best {
                    slots[slot].best = share;
                    slots[slot].span = place;
                    slots[slot].frame = one;
                }
            }
        }
        let mut best_of: Vec<Option<usize>> = vec![None; self.relations.len()];
        let mut order: Vec<u32> = Vec::new();
        for (place, slot) in slots.iter().enumerate() {
            match best_of[slot.relation as usize] {
                None => {
                    best_of[slot.relation as usize] = Some(place);
                    order.push(slot.relation);
                }
                Some(top) => {
                    let (held, new) = (&slots[top], slot);
                    if new.miss < held.miss || (new.miss == held.miss && spans[new.span].first < spans[held.span].first) {
                        best_of[slot.relation as usize] = Some(place);
                    }
                }
            }
        }
        let mut entries: Vec<(u32, Entry)> = Vec::new();
        for relation in order {
            let Some(slot) = best_of[relation as usize].map(|place| &slots[place]) else { continue };
            let chance = CERTAIN - slot.miss;
            if chance < LEAST_CHANCE {
                continue;
            }
            let span = &spans[slot.span];
            let tokens = &scan.article.tokens;
            entries.push((
                relation,
                Entry { answer: text[tokens[span.first].start..tokens[span.end - 1].end].to_string(), chance, sentence: sentence_of(&scan.article, text, scan.article.sentence[span.first]).to_string(), frame: slot.frame },
            ));
        }
        entries
    }

    pub fn frame_text(&self, one: &Frame) -> String {
        let mark = match one.side {
            Side::Left => "L",
            Side::Right => "R",
            Side::Both => "B",
            Side::Class => "C",
            Side::First => "<first>",
            Side::Later => "<later>",
            Side::Sure => "W",
            Side::Listed => "O",
        };
        let mut parts = vec![mark.to_string()];
        if one.side == Side::Sure {
            parts.push(self.words.name(one.words[0]).to_string());
            parts.push(one.words[1].to_string());
        } else {
            parts.extend(one.words.iter().filter(|word| **word != NO_WORD).map(|word| self.words.name(*word).to_string()));
        }
        parts.join("|")
    }

    pub fn kept_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        for (one, kept) in &self.share {
            for (relation, _) in kept {
                let (hit, seen) = self.counts[&(*one, *relation)];
                lines.push(format!("{}\t{}\t{hit}\t{seen}", self.frame_text(one), self.relations.name(*relation)));
            }
        }
        lines.sort_unstable();
        lines
    }
}
