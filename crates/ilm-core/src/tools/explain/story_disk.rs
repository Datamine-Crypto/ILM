use crate::tools::explain::known_good::store_key;
use crate::base::digest::Sha256;
use crate::base::lanes::{Worked, learned_within};
use crate::cases::Case;
use crate::tools::explain::chain::Facts;
use crate::tools::explain::text::{AskedForm, Condition, LogicPiece, Pattern, Reply, ReplyLine, Tally, ToldRule, Topic};
use crate::tools::explain::words::{WordId, word_text};
use patterns::{because, source};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};

pub struct StoryDisk;
source!(StoryDisk, "memoization of a pure function kept on disk between runs (Michie): the facts of a story depend only on the rules, the facts known before it, the told sentences, the bounds and the code of the engine, so a chain made in one run is read back by a later run whose engine is the same, and made again when any of them changed; the files of a store carry the fingerprint of the code of the engine in their names, and a run reads no entry of another fingerprint. What a learner discovers depends only on the text, its known facts, the lessons and the code of the core, so it is kept the same way, under the fingerprint of every source of the core, as trained weights are kept until their inputs change");

const STORE_ENDING: &str = "chains";
because!(STORE_ENDING, StoryDisk, "the ending of the name of a file of the store, so that a folder that holds other files keeps them");

const STORE_NAME_BETWEEN: &str = "-";
because!(STORE_NAME_BETWEEN, StoryDisk, "the mark between the fingerprint of the engine and the number of the process in the name of a file of the store, so that a run finds the files of its fingerprint by the start of their names");

const ENDING_MARK: &str = ".";
because!(ENDING_MARK, StoryDisk, "the mark before the ending of the name of a file of the store");

const MOST_DISK_BYTES: u64 = 4_000_000_000;
because!(MOST_DISK_BYTES, StoryDisk, "the most bytes that the files of one fingerprint may hold on disk before a run lets them all go and starts the store again: a board writes a few hundred million bytes of chains, and runs with the same engine add only the chains of what changed, so the bound is reached seldom and the disk stays bounded");

const HALF_SHIFT: u32 = u64::BITS;
because!(HALF_SHIFT, StoryDisk, "the bits by which the first hash of a content moves up in the number of two hashes, the width of one hash");

const BUILD_LIST_ENDING: &str = "d";
because!(BUILD_LIST_ENDING, StoryDisk, "the ending of the file that the build writes beside the program, which names every source the program was made from");

const BUILD_TARGET_END: &str = ": ";
because!(BUILD_TARGET_END, StoryDisk, "the mark after the path of the program at the start of the list of its sources, before the sources");

const SOURCE_GAP: char = ' ';
because!(SOURCE_GAP, StoryDisk, "the sign between two sources in the list of a build");

const SOURCE_ESCAPE: char = '\\';
because!(SOURCE_ESCAPE, StoryDisk, "the sign before a space that belongs to the path of a source in the list of a build");

const CORE_ROOT_SOURCE: &str = "crates/ilm-core/src/lib.rs";
because!(CORE_ROOT_SOURCE, StoryDisk, "the root source of the core, by which the folder of the core is found among the sources of a build");

const ENGINE_LIST: &str = "tests/engine_files.txt";
because!(ENGINE_LIST, StoryDisk, "the list of the files of the engine in the folder of the core, which a test keeps equal to every file that the engine reads");

const FINGERPRINT_SIGNS: usize = 16;
because!(FINGERPRINT_SIGNS, StoryDisk, "the hexadecimal signs of the hash of the engine files that make a fingerprint, as the script that builds the program cuts them");

const NO_BUILD_LIST: &str = "the store needs the program where its build left it, beside the list of its sources";
because!(NO_BUILD_LIST, StoryDisk, "the fault of a program that was copied away from its build: no list says which engine it holds, so no store may be read");

const NO_ENGINE_LIST: &str = "the sources of the build do not show the files of the engine";
because!(NO_ENGINE_LIST, StoryDisk, "the fault of a build whose sources hold no core or no list of the engine files");

const CHANGED_AFTER_BUILD: &str = "a file of the engine changed after the program was built";
because!(CHANGED_AFTER_BUILD, StoryDisk, "the fault of a file of the engine that is newer than the program, so that the program may hold another engine than the file shows");

const OTHER_ENGINE: &str = "the program holds another engine than the fingerprint given";
because!(OTHER_ENGINE, StoryDisk, "the fault of a run that names a fingerprint that is not the one of the engine of the program, so that it would read chains that another engine made");

const LEARNED_ENDING: &str = "learned";
because!(LEARNED_ENDING, StoryDisk, "the ending of the name of a file of what the learners discovered, beside the files of chains");

pub const LEARNED_RULES: &str = "rules";
because!(LEARNED_RULES, StoryDisk, "the name of the learner of the rules of a concept in the key of what it discovered");

pub const LEARNED_TOLD: &str = "told";
because!(LEARNED_TOLD, StoryDisk, "the name of the learner of the readings of told sentences in the key of what it discovered");

const SECOND_HALF: u8 = 1;
because!(SECOND_HALF, StoryDisk, "the byte that starts the second of the two hashes of a key, so that the two halves of the number of a key are two different hashes of it");

struct OpenDisk {
    path: PathBuf,
    entries: HashMap<u128, (usize, u64)>,
    readers: Vec<Mutex<File>>,
    writer: Mutex<BufWriter<File>>,
    bases: Mutex<HashMap<u64, (Weak<Facts>, u128)>>,
    rules: Mutex<HashMap<u64, u128>>,
    learned: HashMap<u128, Worked<Vec<String>>>,
    learned_path: PathBuf,
    learned_writer: Mutex<BufWriter<File>>,
}
because!(OpenDisk, StoryDisk, "the store of one run: the file that this run writes, by its name, for the number of each key on disk the file and the place of its record, a reader for each file of the fingerprint, the file that this run writes, and the content of each set of known facts and of each set of rules that the run met, found once for each, since many chains share them; and what the learners discovered in earlier runs of the same core, by the number of its key, with the file of what this run discovers and its writer");

static DISK: OnceLock<OpenDisk> = OnceLock::new();
because!(DISK, StoryDisk, "the store of the run, opened once by the command that asks for it and read by every lane; a run that does not ask for it reads and writes no file");

pub struct Bytes<'a> {
    rest: &'a [u8],
}
because!(Bytes, StoryDisk, "the bytes of a record of the store as they are read, from the start: take gives the next bytes, and the numbers and texts are read from them in the order in which they were written");

impl<'a> Bytes<'a> {
    pub fn of(bytes: &'a [u8]) -> Bytes<'a> {
        Bytes { rest: bytes }
    }

    fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        if self.rest.len() < count {
            return None;
        }
        let (head, tail) = self.rest.split_at(count);
        self.rest = tail;
        Some(head)
    }

    pub(super) fn byte(&mut self) -> Option<u8> {
        self.take(std::mem::size_of::<u8>()).map(|head| head[usize::default()])
    }

    pub(super) fn number(&mut self) -> Option<u64> {
        let mut value = 0u64;
        let mut shift = 0u32;
        loop {
            let byte = self.byte()?;
            value |= u64::from(byte & SEVEN_BITS).checked_shl(shift)?;
            if byte & MORE_BIT == 0 {
                return Some(value);
            }
            shift += SEVEN_BITS.count_ones();
            if shift >= HALF_SHIFT {
                return None;
            }
        }
    }

    fn fixed(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(std::mem::size_of::<u64>())?.try_into().ok()?))
    }

    pub(super) fn text(&mut self) -> Option<&'a str> {
        let length = usize::try_from(self.number()?).ok()?;
        std::str::from_utf8(self.take(length)?).ok()
    }

    pub(super) fn end(&self) -> Option<()> {
        self.rest.is_empty().then_some(())
    }
}

pub(super) fn put_byte(out: &mut Vec<u8>, byte: u8) {
    out.push(byte);
}
because!(put_byte, StoryDisk, "one byte of a record, as a mark of yes or no");

pub(super) fn put_number(out: &mut Vec<u8>, number: u64) {
    let mut rest = number;
    while rest > u64::from(SEVEN_BITS) {
        out.push((rest as u8 & SEVEN_BITS) | MORE_BIT);
        rest >>= SEVEN_BITS.count_ones();
    }
    out.push(rest as u8);
}
because!(put_number, StoryDisk, "a number of a record in as few bytes as it needs, seven bits a byte, the low bits first, and the high bit of a byte set while more bytes follow, so that the many small numbers of a chain (lengths of words, places in a story) take one byte each, the same on every machine");

fn put_fixed(out: &mut Vec<u8>, number: u64) {
    out.extend_from_slice(&number.to_le_bytes());
}
because!(put_fixed, StoryDisk, "a length of a record or of its key in eight bytes, the low byte first, so that the index of a file is read from the starts of its records alone");

const SEVEN_BITS: u8 = 0x7f;
because!(SEVEN_BITS, StoryDisk, "the bits of a number that one byte of a record holds, all but its high bit");

const MORE_BIT: u8 = 0x80;
because!(MORE_BIT, StoryDisk, "the high bit of a byte of a number of a record, set while more bytes of the number follow");

pub(super) fn put_text(out: &mut Vec<u8>, text: &str) {
    put_number(out, text.len() as u64);
    out.extend_from_slice(text.as_bytes());
}
because!(put_text, StoryDisk, "a text of a record with its length before it, so that a word is read back as written, whatever marks it holds");

fn piece_into(logic_piece: &LogicPiece, hasher: &mut DefaultHasher) {
    std::mem::discriminant(logic_piece).hash(hasher);
    match logic_piece {
        LogicPiece::Word(word) => word_text(*word).hash(hasher),
        LogicPiece::Open(place) => place.hash(hasher),
        LogicPiece::OpenWithEnd(place, ending) => {
            place.hash(hasher);
            ending.hash(hasher);
        }
    }
}
because!(piece_into, StoryDisk, "a piece of a pattern into the hash of the content of rules: its kind, and its word as a text, not as the number that this run gave it, or its place and ending, so that the same rules have the same content in every run");

fn pattern_into(pattern: &Pattern, hasher: &mut DefaultHasher) {
    pattern.len().hash(hasher);
    for logic_piece in pattern {
        piece_into(logic_piece, hasher);
    }
}
because!(pattern_into, StoryDisk, "a pattern into the hash of the content of rules, its length first, so that two patterns that follow each other are not read as one");

fn words_into(words: &[WordId], hasher: &mut DefaultHasher) {
    words.len().hash(hasher);
    for word in words {
        word_text(*word).hash(hasher);
    }
}
because!(words_into, StoryDisk, "a list of words into the hash of the content of rules, as their texts");

fn condition_into(condition: &Condition, hasher: &mut DefaultHasher) {
    std::mem::discriminant(condition).hash(hasher);
    match condition {
        Condition::Fact { pattern, when } => {
            pattern_into(pattern, hasher);
            std::mem::discriminant(when).hash(hasher);
        }
        Condition::Compare(comparison, first, second) => {
            std::mem::discriminant(comparison).hash(hasher);
            piece_into(first, hasher);
            piece_into(second, hasher);
        }
        Condition::Alike { first, second } => {
            pattern_into(first, hasher);
            pattern_into(second, hasher);
        }
        Condition::Reckon(reckoning, first, second, result) => {
            std::mem::discriminant(reckoning).hash(hasher);
            piece_into(first, hasher);
            piece_into(second, hasher);
            piece_into(result, hasher);
        }
        Condition::Count { result, counted, pattern, tally } => {
            piece_into(result, hasher);
            counted.hash(hasher);
            pattern_into(pattern, hasher);
            std::mem::discriminant(tally).hash(hasher);
            if let Tally::Flow { source, sink, end, capacity } = tally {
                piece_into(source, hasher);
                piece_into(sink, hasher);
                end.hash(hasher);
                capacity.hash(hasher);
            }
        }
        Condition::Follows { statement, said } => {
            pattern_into(statement, hasher);
            pattern_into(said, hasher);
        }
        Condition::Proves { statement, holds } => {
            pattern_into(statement, hasher);
            holds.hash(hasher);
        }
        Condition::Letter { word, letter, place } => {
            piece_into(word, hasher);
            piece_into(letter, hasher);
            place.is_some().hash(hasher);
            if let Some(place) = place {
                piece_into(place, hasher);
            }
        }
        Condition::Told(sentence) => piece_into(sentence, hasher),
        Condition::Holds { sentence, word } => {
            piece_into(sentence, hasher);
            piece_into(word, hasher);
        }
        Condition::HoldsKind { sentence, kind } => {
            piece_into(sentence, hasher);
            words_into(kind, hasher);
        }
        Condition::Backward { result, run } => {
            piece_into(result, hasher);
            piece_into(run, hasher);
        }
        Condition::After { sentence, earlier, later } => {
            piece_into(sentence, hasher);
            piece_into(earlier, hasher);
            piece_into(later, hasher);
        }
    }
}
because!(condition_into, StoryDisk, "a condition into the hash of the content of rules: the kind of the condition and of each enum in it, and every field, with words as texts; a new kind of condition must be named here before the code builds, so no field is left out");

fn rules_into(rules: &[ToldRule], hasher: &mut DefaultHasher) {
    rules.len().hash(hasher);
    for rule in rules {
        rule.conditions.len().hash(hasher);
        for condition in &rule.conditions {
            condition_into(condition, hasher);
        }
        rule.absent.len().hash(hasher);
        for condition in &rule.absent {
            condition_into(condition, hasher);
        }
        pattern_into(&rule.outcome, hasher);
        std::mem::discriminant(&rule.kind).hash(hasher);
        words_into(&rule.name, hasher);
        rule.learned.hash(hasher);
    }
}
because!(rules_into, StoryDisk, "a set of rules into the hash of its content, rule by rule in their order: conditions, the conditions that must be absent, the outcome, the kind of the line, the name that a story gives the rule, and whether a learner made it, so that the words that stand for any thing in a line that reads told rules are in it too, and a chain kept for learned rules, which fill their places with different things, is never read back for the same lines of the library, which may fill two places with one thing");

fn two_hashes(fill: &dyn Fn(&mut DefaultHasher)) -> u128 {
    let mut first = DefaultHasher::new();
    fill(&mut first);
    let mut second = DefaultHasher::new();
    SECOND_HALF.hash(&mut second);
    fill(&mut second);
    (u128::from(first.finish()) << HALF_SHIFT) | u128::from(second.finish())
}
because!(two_hashes, StoryDisk, "a content as one number of two hashes of it, the second started by another byte, so that two different contents that the keys of one store hold share a number with a chance far too small to meet");

fn rules_content(disk: &OpenDisk, rules: &[ToldRule], rules_hash: u64) -> u128 {
    if let Some(known) = disk.rules.lock().ok().and_then(|known| known.get(&rules_hash).copied()) {
        return known;
    }
    let content = two_hashes(&|hasher| rules_into(rules, hasher));
    if let Ok(mut known) = disk.rules.lock() {
        known.insert(rules_hash, content);
    }
    content
}
because!(rules_content, StoryDisk, "the content of a set of rules, found once for the hash that this run keeps for it, as the kept chains of the run already tell rules apart by that hash");

fn base_content(disk: &OpenDisk, base: &Arc<Facts>) -> u128 {
    let number = base.id();
    if let Some(known) = disk.bases.lock().ok().and_then(|known| known.get(&number).map(|(_, content)| *content)) {
        return known;
    }
    let content = two_hashes(&|hasher| base.content_into(hasher));
    if let Ok(mut known) = disk.bases.lock() {
        known.retain(|_, (held, _)| held.strong_count() > 0);
        known.insert(number, (Arc::downgrade(base), content));
    }
    content
}
because!(base_content, StoryDisk, "the content of the facts known before a story, found once for each set of them that the run made, by the number of the set (FactsId); each entry holds a weak reference to its set, and an entry whose set the run let go is dropped when a new set comes, so that a run that makes a set for each search does not hold them all");

fn topic_into(topic: &Topic, hasher: &mut DefaultHasher) {
    let Topic { name, rules, questions, reply_lines, facts, builds_on, lines, rules_key: _ } = topic;
    name.hash(hasher);
    rules_into(rules, hasher);
    questions.len().hash(hasher);
    for asked in questions {
        let AskedForm { question, replies } = asked;
        pattern_into(question, hasher);
        replies.len().hash(hasher);
        for reply in replies {
            let Reply { words, conditions, absent, text, every } = reply;
            pattern_into(words, hasher);
            conditions.len().hash(hasher);
            for condition in conditions {
                condition_into(condition, hasher);
            }
            absent.len().hash(hasher);
            for condition in absent {
                condition_into(condition, hasher);
            }
            text.hash(hasher);
            every.hash(hasher);
        }
    }
    reply_lines.len().hash(hasher);
    for line in reply_lines {
        let ReplyLine { question, answer, said, sentence } = line;
        pattern_into(question, hasher);
        pattern_into(answer, hasher);
        pattern_into(said, hasher);
        sentence.hash(hasher);
    }
    facts.len().hash(hasher);
    for fact in facts {
        words_into(fact, hasher);
    }
    builds_on.hash(hasher);
    lines.hash(hasher);
}
because!(topic_into, StoryDisk, "a text into the hash of its content: every field of it and of its question forms, answers and reply lines, words as texts; the hash of its rules is left out, as the rules themselves are in it; a new field must be named here before the code builds");

fn lessons_into(lessons: &[&Case], hasher: &mut DefaultHasher) {
    lessons.len().hash(hasher);
    for case in lessons {
        let Case { sentences, question, answer } = case;
        sentences.hash(hasher);
        question.hash(hasher);
        answer.hash(hasher);
    }
}
because!(lessons_into, StoryDisk, "the lessons into the hash of a key: each told sentence, question and answer, in their order");

pub fn learned_content(learner: &str, topic: &Topic, base: &Facts, lessons: &[&Case]) -> u128 {
    two_hashes(&|hasher| {
        learner.hash(hasher);
        topic_into(topic, hasher);
        store_key(&topic.name).hash(hasher);
        base.content_into(hasher);
        lessons_into(lessons, hasher);
    })
}
because!(learned_content, StoryDisk, "the key of what a learner discovers, by content alone: the learner, the text and the lines that the known-good store holds for it, the facts it knows before any story, and the lessons");

fn learned_key(learner: &str, topic: &Topic, base: &Arc<Facts>, lessons: &[&Case]) -> Option<u128> {
    let disk = DISK.get()?;
    let known = base_content(disk, base);
    Some(two_hashes(&|hasher| {
        learner.hash(hasher);
        topic_into(topic, hasher);
        store_key(&topic.name).hash(hasher);
        known.hash(hasher);
        lessons_into(lessons, hasher);
    }))
}
because!(learned_key, StoryDisk, "the key of what a learner discovers in a run that keeps a store, with the lines that the known-good store holds for the text and the content of the known facts found once for each set of them; nothing when the run keeps no store");

pub fn learned_record(key: u128, worked: &Worked<Vec<String>>) -> Vec<u8> {
    let mut record = Vec::new();
    record.extend_from_slice(&key.to_le_bytes());
    put_number(&mut record, worked.work as u64);
    put_byte(&mut record, u8::from(worked.stopped));
    put_number(&mut record, worked.taken.len() as u64);
    for kind in &worked.taken {
        put_text(&mut record, kind);
    }
    put_number(&mut record, worked.value.len() as u64);
    for line in &worked.value {
        put_text(&mut record, line);
    }
    let mut out = Vec::new();
    put_fixed(&mut out, record.len() as u64);
    out.extend_from_slice(&record);
    out
}
because!(learned_record, StoryDisk, "the record of what a learner discovered: its length, the number of its key, the work that it cost and whether it stopped at the bound of work, the kinds whose stored lines it took, the number of lines and each line");

fn learned_entry(record: &[u8]) -> Option<(u128, Worked<Vec<String>>)> {
    let mut bytes = Bytes::of(record);
    let key = u128::from_le_bytes(bytes.take(std::mem::size_of::<u128>())?.try_into().ok()?);
    let work = usize::try_from(bytes.number()?).ok()?;
    let stopped = bytes.byte()? != 0;
    let kinds = usize::try_from(bytes.number()?).ok()?;
    let taken = (0..kinds).map(|_| bytes.text().map(str::to_string)).collect::<Option<Vec<String>>>()?;
    let count = usize::try_from(bytes.number()?).ok()?;
    let value = (0..count).map(|_| bytes.text().map(str::to_string)).collect::<Option<Vec<String>>>()?;
    bytes.end()?;
    Some((key, Worked { value, work, stopped, taken }))
}
because!(learned_entry, StoryDisk, "one record of what a learner discovered read back: the number of its key, its work, its stop, its stored kinds and its lines, or nothing when it does not read whole");

pub fn learned_records(bytes: &[u8]) -> Vec<(u128, Worked<Vec<String>>)> {
    let mut out = Vec::new();
    let mut rest = Bytes::of(bytes);
    while let Some(length) = rest.fixed() {
        let Some(record) = usize::try_from(length).ok().and_then(|length| rest.take(length)) else { break };
        if let Some(found) = learned_entry(record) {
            out.push(found);
        }
    }
    out
}
because!(learned_records, StoryDisk, "the records of a file of what learners discovered, in their order; a record that does not read whole is passed over, and a record cut off at the end of the file, as when a run stopped while it wrote, ends the reading");

pub(super) fn kept_learning(learner: &str, (topic, base, lessons): (&Topic, &Arc<Facts>, &[&Case]), learn: &dyn Fn(&Topic, &Arc<Facts>, &[&Case]) -> Vec<String>) -> Vec<String> {
    let key = learned_key(learner, topic, base, lessons);
    let known = key.and_then(|key| DISK.get()?.learned.get(&key).cloned());
    let (lines, fresh) = learned_within(known.as_ref(), || learn(topic, base, lessons));
    if let Some(key) = key
        && let Some(fresh) = fresh
        && let Some(disk) = DISK.get()
        && let Ok(mut writer) = disk.learned_writer.lock()
    {
        writer.write_all(&learned_record(key, &fresh)).ok();
    }
    lines
}
because!(kept_learning, StoryDisk, "what a learner discovers for a text, its known facts and lessons, under the bound of work of a kept learning (learned_within): read back from the store when an earlier run of the same core discovered it within the work left, else learned and written to the store of the run when it keeps one and the learning did not stop, with its work");

fn key_number(key: &[u8]) -> u128 {
    two_hashes(&|hasher| key.hash(hasher))
}
because!(key_number, StoryDisk, "the number of a key in the index of the store; the record holds the whole key, so a number shared by two keys only costs a read");

pub(super) fn disk_key(told: &[String], base: &Arc<Facts>, (rules, rules_hash): (&[ToldRule], u64), (most, reads): (usize, usize)) -> Option<Vec<u8>> {
    let disk = DISK.get()?;
    let mut key = Vec::new();
    put_number(&mut key, told.len() as u64);
    for sentence in told {
        put_text(&mut key, sentence);
    }
    key.extend_from_slice(&rules_content(disk, rules, rules_hash).to_le_bytes());
    key.extend_from_slice(&base_content(disk, base).to_le_bytes());
    put_number(&mut key, most as u64);
    put_number(&mut key, reads as u64);
    Some(key)
}
because!(disk_key, StoryDisk, "the key of a chain on disk, by content alone: the told sentences, the content of the rules, the content of the facts known before the story, and the two bounds; nothing when the run keeps no store");

pub(super) fn disk_read(key: &[u8], base: &Arc<Facts>) -> Option<Facts> {
    let disk = DISK.get()?;
    let (file, place) = *disk.entries.get(&key_number(key))?;
    let record = {
        let mut reader = disk.readers.get(file)?.lock().ok()?;
        reader.seek(SeekFrom::Start(place)).ok()?;
        let mut length = [0u8; std::mem::size_of::<u64>()];
        reader.read_exact(&mut length).ok()?;
        let mut record = vec![0u8; usize::try_from(u64::from_le_bytes(length)).ok()?];
        reader.read_exact(&mut record).ok()?;
        record
    };
    let mut bytes = Bytes::of(&record);
    let length = usize::try_from(bytes.fixed()?).ok()?;
    if bytes.take(length)? != key {
        return None;
    }
    Facts::from_layer(&mut bytes, base)
}
because!(disk_read, StoryDisk, "the chain of a key read back from the store over the given known facts, or nothing when the store holds no such key, its record holds another key, or the record does not read as a chain over those facts");

pub(super) fn disk_write(key: &[u8], facts: &Facts) {
    let Some(disk) = DISK.get() else { return };
    let mut record = Vec::new();
    put_fixed(&mut record, key.len() as u64);
    record.extend_from_slice(key);
    facts.layer_into(&mut record);
    let mut out = Vec::new();
    put_fixed(&mut out, record.len() as u64);
    out.extend_from_slice(&record);
    if let Ok(mut writer) = disk.writer.lock() {
        writer.write_all(&out).ok();
    }
}
because!(disk_write, StoryDisk, "a new chain written to the file of this run under its key, so that a later run with the same engine reads it back");

fn store_files(folder: &Path, (engine, kind): (&str, &str)) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let start = format!("{engine}{STORE_NAME_BETWEEN}");
    let ending = format!("{ENDING_MARK}{kind}");
    let mut own = Vec::new();
    let mut other = Vec::new();
    for entry in fs::read_dir(folder).into_iter().flatten().flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else { continue };
        if !name.ends_with(&ending) {
            continue;
        }
        if name.starts_with(&start) {
            own.push(path);
        } else {
            other.push(path);
        }
    }
    own.sort();
    (own, other)
}
because!(store_files, StoryDisk, "the files of one kind of the store in a folder (chains, or what learners discovered): those of the given fingerprint in the order of their names, and those of other fingerprints, which a run lets go");

fn file_entries(file: usize, path: &Path, entries: &mut HashMap<u128, (usize, u64)>) -> Option<File> {
    let mut reader = File::open(path).ok()?;
    let total = reader.metadata().ok()?.len();
    let mut place = u64::default();
    let number_bytes = std::mem::size_of::<u64>() as u64;
    while place + number_bytes <= total {
        reader.seek(SeekFrom::Start(place)).ok()?;
        let mut head = [0u8; std::mem::size_of::<u64>()];
        reader.read_exact(&mut head).ok()?;
        let record = u64::from_le_bytes(head);
        let mut key_length = [0u8; std::mem::size_of::<u64>()];
        if record < number_bytes || place + number_bytes + record > total || reader.read_exact(&mut key_length).is_err() {
            break;
        }
        let length = u64::from_le_bytes(key_length);
        let mut key = vec![0u8; usize::try_from(length.min(record - number_bytes)).ok()?];
        reader.read_exact(&mut key).ok()?;
        entries.entry(key_number(&key)).or_insert((file, place));
        place += number_bytes + record;
    }
    Some(reader)
}
because!(file_entries, StoryDisk, "the records of one file of the store put in the index by the number of their keys, the first record of a key kept; a record cut off at the end of the file, as when a run stopped while it wrote, is left out");

pub fn build_sources(listed: &str) -> Vec<PathBuf> {
    let Some((_, rest)) = listed.lines().next().unwrap_or_default().split_once(BUILD_TARGET_END) else { return Vec::new() };
    let mut out = Vec::new();
    let mut path = String::new();
    let mut signs = rest.chars().peekable();
    while let Some(sign) = signs.next() {
        if sign == SOURCE_ESCAPE && signs.peek() == Some(&SOURCE_GAP) {
            path.push(SOURCE_GAP);
            signs.next();
        } else if sign == SOURCE_GAP {
            if !path.is_empty() {
                out.push(PathBuf::from(std::mem::take(&mut path)));
            }
        } else {
            path.push(sign);
        }
    }
    if !path.is_empty() {
        out.push(PathBuf::from(path));
    }
    out
}
because!(build_sources, StoryDisk, "the sources that the list of a build names after the path of the program, each path whole, a space that its sign marks kept in it");

pub fn engine_print(core: &Path, built: std::time::SystemTime) -> Result<String, String> {
    let names = fs::read_to_string(core.join(ENGINE_LIST)).map_err(|_| NO_ENGINE_LIST.to_string())?;
    let mut hash = Sha256::new();
    for name in names.lines().map(str::trim).filter(|name| !name.is_empty()) {
        let source = core.join(name);
        let changed = fs::metadata(&source).and_then(|meta| meta.modified()).map_err(|fault| format!("{}: {fault}", source.display()))?;
        if changed > built {
            return Err(format!("{CHANGED_AFTER_BUILD}: {}", source.display()));
        }
        hash.add(&fs::read(&source).map_err(|fault| format!("{}: {fault}", source.display()))?);
    }
    Ok(hash.hex().chars().take(FINGERPRINT_SIGNS).collect())
}
because!(engine_print, StoryDisk, "the fingerprint of the engine files of a core folder, as the script that builds the program makes it (the files of the list in its order, their bytes one after another, hashed, the first signs of the hash), when no file of the list is newer than the build");

pub fn core_print(core: &Path, sources: &[PathBuf], built: std::time::SystemTime) -> Result<String, String> {
    let mut inner: Vec<(String, &PathBuf)> = sources.iter().filter_map(|source| Some((source.strip_prefix(core).ok()?.to_string_lossy().into_owned(), source))).collect();
    inner.sort();
    let mut hash = Sha256::new();
    for (name, source) in inner {
        let changed = fs::metadata(source).and_then(|meta| meta.modified()).map_err(|fault| format!("{}: {fault}", source.display()))?;
        if changed > built {
            return Err(format!("{CHANGED_AFTER_BUILD}: {}", source.display()));
        }
        put_name_into(&mut hash, &name);
        hash.add(&fs::read(source).map_err(|fault| format!("{}: {fault}", source.display()))?);
    }
    Ok(hash.hex().chars().take(FINGERPRINT_SIGNS).collect())
}
because!(core_print, StoryDisk, "the fingerprint of every source of the core that a build names, each by its name in the core folder and its bytes, in the order of the names, when none is newer than the build: what a learner discovers may rest on any of them");

fn put_name_into(hash: &mut Sha256, name: &str) {
    let mut named = Vec::new();
    put_text(&mut named, name);
    hash.add(&named);
}
because!(put_name_into, StoryDisk, "the name of a source into the hash, its length first, so that a file that moves changes the fingerprint and two names never run into one");

pub fn built_prints() -> Result<(String, String), String> {
    let program = std::env::current_exe().map_err(|fault| fault.to_string())?;
    let listed = fs::read_to_string(program.with_extension(BUILD_LIST_ENDING)).map_err(|_| NO_BUILD_LIST.to_string())?;
    let sources = build_sources(&listed);
    let core = sources.iter().find(|path| path.ends_with(CORE_ROOT_SOURCE)).and_then(|root| root.parent()?.parent()).ok_or_else(|| NO_ENGINE_LIST.to_string())?;
    let built = fs::metadata(&program).and_then(|meta| meta.modified()).map_err(|fault| fault.to_string())?;
    Ok((engine_print(core, built)?, core_print(core, &sources, built)?))
}
because!(built_prints, StoryDisk, "the fingerprints of the engine and of the whole core that this program holds, from the sources that its build names beside it, none newer than the program");

pub fn open_story_disk(folder: &str, engine: Option<&str>) -> Result<(), String> {
    let (built, core) = built_prints()?;
    if let Some(given) = engine
        && given != built
    {
        return Err(format!("{OTHER_ENGINE}: {given} given, {built} built"));
    }
    let engine = built.as_str();
    let folder = PathBuf::from(folder);
    fs::create_dir_all(&folder).map_err(|fault| fault.to_string())?;
    let (mut own, other) = store_files(&folder, (engine, STORE_ENDING));
    for path in other {
        fs::remove_file(path).ok();
    }
    let size: u64 = own.iter().filter_map(|path| fs::metadata(path).ok()).map(|meta| meta.len()).sum();
    if size > MOST_DISK_BYTES {
        for path in own.drain(..) {
            fs::remove_file(path).ok();
        }
    }
    let mut entries: HashMap<u128, (usize, u64)> = HashMap::new();
    let mut readers: Vec<Mutex<File>> = Vec::new();
    for path in &own {
        if let Some(reader) = file_entries(readers.len(), path, &mut entries) {
            readers.push(Mutex::new(reader));
        }
    }
    let name = format!("{engine}{STORE_NAME_BETWEEN}{}{ENDING_MARK}{STORE_ENDING}", std::process::id());
    let path = folder.join(name);
    let file = OpenOptions::new().create(true).append(true).open(&path).map_err(|fault| fault.to_string())?;
    let (learned_files, other_learned) = store_files(&folder, (&core, LEARNED_ENDING));
    for path in other_learned {
        fs::remove_file(path).ok();
    }
    let learned: HashMap<u128, Worked<Vec<String>>> = learned_files.iter().filter_map(|path| fs::read(path).ok()).flat_map(|bytes| learned_records(&bytes)).collect();
    let learned_path = folder.join(format!("{core}{STORE_NAME_BETWEEN}{}{ENDING_MARK}{LEARNED_ENDING}", std::process::id()));
    let learned_file = OpenOptions::new().create(true).append(true).open(&learned_path).map_err(|fault| fault.to_string())?;
    let disk = OpenDisk { path, entries, readers, writer: Mutex::new(BufWriter::new(file)), bases: Mutex::default(), rules: Mutex::default(), learned, learned_path, learned_writer: Mutex::new(BufWriter::new(learned_file)) };
    DISK.set(disk).map_err(|_| String::new())
}
because!(open_story_disk, StoryDisk, "the store of the run in a folder for the fingerprint of the engine of this program, refused when a fingerprint given differs from it: the files of other fingerprints let go, all files of this one let go when they hold more than the bound, the records of the others put in the index, and a new file for the chains that this run makes; beside them, what the learners of earlier runs of the same core discovered, read whole, the files of other cores let go, and a new file for what this run discovers");

pub fn close_story_disk() {
    let Some(disk) = DISK.get() else { return };
    if let Ok(mut writer) = disk.writer.lock() {
        writer.flush().ok();
    }
    if let Ok(mut writer) = disk.learned_writer.lock() {
        writer.flush().ok();
    }
    for path in [&disk.path, &disk.learned_path] {
        if fs::metadata(path).is_ok_and(|meta| meta.len() == 0) {
            fs::remove_file(path).ok();
        }
    }
}
because!(close_story_disk, StoryDisk, "the chains and the discoveries of the run written out to their files before the run ends, and a file let go when the run added nothing to it, so that runs that find everything leave no empty files");
