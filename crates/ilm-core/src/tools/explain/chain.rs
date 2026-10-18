use crate::tools::explain::costs::{chain_ended, learning_clock, reply_ended};
use std::borrow::Cow;
use crate::base::lanes::{WorkBudget, add_scope_work};
use std::cell::Cell;
use crate::base::exact::Amount;
use crate::tools::explain::worlds::{announced_sentence, follows};
use crate::tools::explain::clauses::{any_sentence, opposite, refuted};
use crate::tools::explain::positions::{first_letter, held_words, later_pairs, placed_letters};
use crate::tools::explain::story_disk::{Bytes, StoryDisk, disk_key, disk_read, disk_write, put_byte, put_number, put_text};
use crate::tools::explain::text::{NUMBER_BELOW_NOTHING, NUMBER_OVER, NUMBER_POINT, OPEN_PLACES, QUESTION_TOLD, USUALLY, Tally, place_letter, place_takes, PlaceTakes, group_turn, LineKind, told_rule, Pattern, LogicPiece, AskedForm, Comparison, Condition, Moment, Reckoning, Reply, ReplyLine, ToldRule, condition_text, fact_numbers, text_words};
use crate::tools::explain::words::{WORD_IS_BRACKET, WORD_IS_DIGITS, WORD_IS_TEXT, WordId, word_has, word_number, word_text, word_texts};
use patterns::{because, source};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher, Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};

pub struct ChainOfRules;
source!(ChainOfRules, "forward chaining of rules over sentences, as in production systems and Datalog: the told sentences are facts; a round fills the open places of a rule with words of facts so that every condition is a fact, and adds its outcome as a new fact; a round tries only the fillings that use a fact that the round before added (semi-naive evaluation, Bancilhon and Ramakrishnan), so that no filling is tried twice; the chain_facts ends when a round adds nothing. Rules with a condition that must be absent, or that only the latest fact may hold, come in a later layer, as stratified negation does it (Apt, Blair and Walker): they read the facts only when the other rules add nothing more. Each fact keeps the place in the story of the latest told sentence that it rests on, as a truth maintenance system keeps the support of a belief (Doyle): a told sentence its own place, an outcome the latest place of the facts that its conditions read, or, for a rule that reads 'at that time' or 'before that time', the place of the clause that the first such condition points to, since the outcome happened then, and a fact that comes again with a later place takes that place, so that its outcomes are read again; in a topic that reads 'at that time' or 'before that time', a fact keeps every place at which it came, as a history of events, and its outcomes are read again at each new place");

const MOST_FACTS: usize = 50_000;
because!(MOST_FACTS, ChainOfRules, "the most facts of one story: a rule whose outcome feeds its own condition, as a route made of routes, would add facts without end; the bound stops the chain with every fact that it reached");

const MOST_KNOWN_FACTS: usize = 500_000;
because!(MOST_KNOWN_FACTS, ChainOfRules, "the most facts that a topic may know before any story: made once for a topic and shared by every story, so it may be far larger than the facts of one story; more than ten times the largest knowledge of the library (the meanings of words read both ways), at about a hundred million bytes, so that a list that grows does not cut what the topic knows");

const MOST_FACT_WORDS: usize = 80;
because!(MOST_FACT_WORDS, ChainOfRules, "the words that the facts of a chain may hold on average under its bound of facts: the facts of a story or of a topic hold some words each, and the longest few times more (the actions of a long command), while a rule whose outcome holds its own condition with more words makes a longer fact in each round, so that the words of its facts, and not their number, fill the memory; a chain whose facts hold more words than its bound of facts times this many ends at a bound, in a time of the order of a well-read story");

const MOST_READS: usize = 2_000_000;
because!(MOST_READS, ChainOfRules, "the most facts that the conditions of the rules may read for one story: a text whose words a topic was not written for can make rules fit whole phrases, and their facts grow with each round; the bound ends such a chain in a time of the order of a well-read story, with the facts that it reached");

pub const FILLING_SLOTS: usize = OPEN_PLACES + 1;
because!(FILLING_SLOTS, ChainOfRules, "the places that a filling holds: every open place, and one more, which the reading of the state of a thing opens for its last word when that word is fixed");

#[derive(Clone, Debug)]
pub struct Filling {
    slots: [u8; FILLING_SLOTS],
    entries: u8,
    words: Vec<WordId>,
}
because!(Filling, ChainOfRules, "the words that fill each open place, and nothing for a place that no condition filled yet: for each place the number of its entry, and nought for a place still open, and one list that holds first where each entry ends and then the words of the entries, in the order in which their places were filled, so that a new filling and a copy cost one piece of memory and as much as the places that they fill, and not as all the places that the library may name; get gives the words of a place, filled says whether a place holds words, set fills a place and clear opens it again");

impl Filling {
    fn span(&self, at: usize) -> (usize, usize) {
        let first = usize::from(self.entries);
        let start = at.checked_sub(1).map_or(usize::default(), |before| self.words[before] as usize);
        (first + start, first + self.words[at] as usize)
    }

    pub fn get(&self, place: usize) -> Option<&[WordId]> {
        let (start, end) = self.span(usize::from(self.slots[place]).checked_sub(1)?);
        Some(&self.words[start..end])
    }

    pub fn filled(&self, place: usize) -> bool {
        self.slots[place] != 0
    }

    pub fn set(&mut self, place: usize, words: Vec<WordId>) {
        let first = usize::from(self.entries);
        match usize::from(self.slots[place]).checked_sub(1) {
            Some(at) => {
                let (start, end) = self.span(at);
                let now = words.len();
                self.words.splice(start..end, words).for_each(drop);
                for later in at..first {
                    self.words[later] = (self.words[later] as usize + now - (end - start)) as WordId;
                }
            }
            None => {
                let end = first.checked_sub(1).map_or(usize::default(), |last| self.words[last] as usize) + words.len();
                self.words.insert(first, end as WordId);
                self.words.extend(words);
                self.entries += 1;
                self.slots[place] = self.entries;
            }
        }
    }

    pub fn clear(&mut self, place: usize) {
        let Some(at) = usize::from(self.slots[place]).checked_sub(1) else { return };
        let (start, end) = self.span(at);
        self.words.drain(start..end);
        for later in at + 1..usize::from(self.entries) {
            self.words[later] -= (end - start) as WordId;
        }
        self.words.remove(at);
        self.entries -= 1;
        self.slots[place] = 0;
        for slot in self.slots.iter_mut().filter(|slot| usize::from(**slot) > at) {
            *slot -= 1;
        }
    }
}

pub fn empty_filling() -> Filling {
    Filling { slots: [0; FILLING_SLOTS], entries: 0, words: Vec::new() }
}
because!(empty_filling, ChainOfRules, "a filling with every open place still open");

fn same_filling(one: &Filling, other: &Filling) -> bool {
    (0..FILLING_SLOTS).all(|place| one.get(place) == other.get(place))
}
because!(same_filling, ChainOfRules, "whether two fillings are the same: every place holds the same words, whatever the order in which their places were filled; it reads every place, so only an answer that says every filling uses it");

pub type Time = usize;
because!(Time, ChainOfRules, "the place in the story of the latest told sentence that a fact rests on");

const STORY_END: Time = Time::MAX;
because!(STORY_END, ChainOfRules, "the time that a condition 'at that time' reads when no condition comes before it: the end of the story, as 'latest' reads it");

type Clock = (Time, Time, bool);
because!(Clock, ChainOfRules, "the times of a join of conditions: the place in the story that an outcome takes, the place that a condition 'at that time' reads, and whether such a condition was read, after which the outcome keeps the place of the clause that it pointed to, as an event keeps the time at which it happened");

type TimedFilling = (Filling, Time);
because!(TimedFilling, ChainOfRules, "a filling under which conditions hold, with the latest place in the story of the facts that it read");

pub struct NumberMixing;
source!(NumberMixing, "the hash of the Firefox engine (FxHash): a number is mixed into the hash by a turn of its bits and a product with a large odd number, which spreads the small numbers of words and the numbers that are hashes already over the whole width at the cost of one product, where the hash of the standard map spends many rounds on each");

#[derive(Default)]
struct MixHasher(u64);
because!(MixHasher, NumberMixing, "a hash of numbers for the maps of facts, whose keys are word numbers and hashes of words: one product for each number; write takes other keys eight bytes at a time");

const MIX_FACTOR: u64 = 0x517c_c1b7_2722_0a95;
because!(MIX_FACTOR, NumberMixing, "the odd number of the product, the one the Firefox hash uses, whose bits spread a small number over the whole width");

const MIX_TURN: u32 = 5;
because!(MIX_TURN, NumberMixing, "the turn of the bits before each new number, as the Firefox hash turns them, so that the order of the numbers matters");

const MIX_BYTES: usize = 8;
because!(MIX_BYTES, NumberMixing, "the bytes of one number of the hash");

impl Hasher for MixHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(MIX_BYTES) {
            let mut wide = [0u8; MIX_BYTES];
            wide[..chunk.len()].copy_from_slice(chunk);
            self.write_u64(u64::from_le_bytes(wide));
        }
    }

    fn write_u64(&mut self, number: u64) {
        self.0 = (self.0.rotate_left(MIX_TURN) ^ number).wrapping_mul(MIX_FACTOR);
    }

    fn write_u32(&mut self, number: u32) {
        self.write_u64(u64::from(number));
    }

    fn write_usize(&mut self, number: usize) {
        self.write_u64(number as u64);
    }
}

type MixMap<K, V> = HashMap<K, V, BuildHasherDefault<MixHasher>>;
because!(MixMap, NumberMixing, "a map of the facts keyed by numbers, with the hash of numbers");

static FACTS_MADE: AtomicU64 = AtomicU64::new(0);
because!(FACTS_MADE, ChainOfRules, "the count of the sets of facts that the run made so far, which gives each set its own number");

pub struct FactsId(u64);
because!(FactsId, ChainOfRules, "the number of one set of facts, given when the set is made and never again in the run: what a run keeps for a set is found by this number and not by the place of the set in memory, which a later set takes once the set is let go");

impl Default for FactsId {
    fn default() -> FactsId {
        FactsId(FACTS_MADE.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Default)]
pub struct Facts {
    id: FactsId,
    under: Option<Arc<Facts>>,
    first: usize,
    words: Vec<WordId>,
    ends: Vec<FactPlace>,
    times: Vec<Time>,
    past: Vec<Vec<Time>>,
    moved: HashMap<usize, Vec<Time>>,
    told: Vec<FactPlace>,
    history: bool,
    known: FactIndex,
    by_start: FactIndex,
    by_word: FactIndex,
    work: AtomicUsize,
    stopped: AtomicBool,
}
because!(Facts, ChainOfRules, "the facts of one story in the order in which they came, as a layer over the facts known before it (the facts of a topic before any story), which it reads and never changes, so that a story does not copy them: a set that says whether a sentence is a fact, and for each start of a fact of its first few words (by a hash of its words) the facts with that start, so that a condition whose first words are known looks only at the facts that begin with them, and for each word the facts that hold it, so that any other condition looks only at the facts that hold its rarest word; each in this layer and in the layers under it, whose places come first; and the count of facts that the conditions read, which bounds the work of a story; the places of the sentences that the story told, so that a told sentence is told apart from a fact that the rules made (is_told); and for each fact the place in the story that it rests on, and, when the facts keep their history, every place in the story at which it came, in order, with a later place of a fact of a layer under it kept in this layer. add keeps a new fact, or a later place of a known fact (any new place, with the history), and gives its place and that time, and nothing for a fact that is known with that place or a later one; holds says whether a sentence is a fact; holds_in_order says whether some fact holds some words in their order, from the facts that hold the rarest of them and with no count of the reads, so that a sentence whose fixed words no fact holds in order is passed over before any fit is tried, and a trial costs nothing of the bound of a story; newer says whether a sentence with a place would add to the facts; told_facts gives the told sentences of the story as the facts that they became, a line that writes a sentence again having written them in place, also those of a kept chain under it; shut gives a copy whose indexes are shut, for a layer that is kept and never grows again; layer_into writes the facts of the layer itself as words, with their places in the story, the told places, the later places of facts under it, the count of reads and the mark of a bound, and from_layer reads them back over the same facts under it as the same layer, its indexes made again in the same order and shut; content_into gives the content of all its facts as words, for the key of a chain kept on disk; stopped_at_bound says whether the chain that made them ended at a bound of the facts, of the reads or of the writes of a sentence, so that a fact may be missing or left unwritten; has_history says whether they keep every place in the story at which a fact came");

impl Clone for Facts {
    fn clone(&self) -> Facts {
        Facts { id: FactsId::default(), under: self.under.clone(), first: self.first, words: self.words.clone(), ends: self.ends.clone(), times: self.times.clone(), past: self.past.clone(), moved: self.moved.clone(), told: self.told.clone(), history: self.history, known: self.known.clone(), by_start: self.by_start.clone(), by_word: self.by_word.clone(), work: AtomicUsize::new(self.work.load(Ordering::Relaxed)), stopped: AtomicBool::new(self.stopped_at_bound()) }
    }
}

fn start_key(state: &mut u64, word: WordId) -> u64 {
    *state = (state.rotate_left(MIX_TURN) ^ u64::from(word)).wrapping_mul(MIX_FACTOR);
    *state ^ (*state >> INDEX_FOLD)
}
because!(start_key, NumberMixing, ChainOfRules, "the key of a start of a fact after one more word: the state of the start before it turned, mixed with the word and multiplied by the odd number of the hash, then its high half folded onto its low half, made word by word, so that the index of starts keeps one number for each start in place of a copy of its words at the cost of one product a word; two starts that share a key only add a fact that the fit then turns away");

type FactPlace = u32;
because!(FactPlace, ChainOfRules, "the place of a fact in the facts of a story and the layers under it, in four bytes: a story holds far fewer facts than that, and the tables of the kept stories hold tens of millions of places");

#[derive(Clone)]
enum FactPlaces {
    One(FactPlace),
    More(Vec<FactPlace>),
}
because!(FactPlaces, ChainOfRules, "the places of the facts under one key of an index, in order: one place is held in the key itself, and a list is made only for a second place, since most starts of facts belong to one fact. put adds a place that is not the last one; all gives them as a list");

#[derive(Clone)]
enum FactIndex {
    Open(MixMap<u64, FactPlaces>),
    Shut(ShutIndex),
}
because!(FactIndex, ChainOfRules, "an index of the facts of one layer by a key: open, a map that grows as the facts of a chain are added; shut, one compact table, once the layer is kept and never grows again, so that the many kept chains of a run take a few bytes for each key and each place in place of an entry of a map with a list of its own; places gives the places under a key in the order in which they came, the same in both forms, put adds a place, and shut makes the compact table");

impl Default for FactIndex {
    fn default() -> FactIndex {
        FactIndex::Open(MixMap::default())
    }
}

impl FactIndex {
    fn places(&self, key: u64) -> &[FactPlace] {
        match self {
            FactIndex::Open(map) => map.get(&key).map_or(&[][..], FactPlaces::all),
            FactIndex::Shut(table) => table.places(key),
        }
    }

    fn put(&mut self, key: u64, place: FactPlace) {
        if let FactIndex::Shut(table) = self {
            *self = FactIndex::Open(table.opened());
        }
        if let FactIndex::Open(map) = self {
            map.entry(key).and_modify(|holders| holders.put(place)).or_insert(FactPlaces::One(place));
        }
    }

    fn shut(&self) -> FactIndex {
        match self {
            FactIndex::Open(map) => FactIndex::Shut(ShutIndex::of(map)),
            FactIndex::Shut(table) => FactIndex::Shut(table.clone()),
        }
    }
}

#[derive(Clone, Default)]
struct ShutIndex {
    slots: Vec<u32>,
    keys: Vec<u64>,
    ends: Vec<u32>,
    places: Vec<FactPlace>,
}
because!(ShutIndex, ChainOfRules, "the compact table of a shut index: open addressing over a power of two of slots, at most half of them used, each the number of an entry or nought for a free slot, and for each entry its key and where its places end in one list of all the places, which keeps the places of a key in the order in which they came; of makes it from a map, places reads a key, opened gives the map again");

impl ShutIndex {
    fn of(map: &MixMap<u64, FactPlaces>) -> ShutIndex {
        let room = (map.len() * SHUT_ROOM).next_power_of_two();
        let mut table = ShutIndex { slots: vec![0; room], keys: Vec::with_capacity(map.len()), ends: Vec::with_capacity(map.len()), places: Vec::with_capacity(map.values().map(|holders| holders.all().len()).sum()) };
        for (key, holders) in map {
            table.places.extend_from_slice(holders.all());
            table.keys.push(*key);
            table.ends.push(table.places.len() as u32);
            let mut at = slot_of(*key, room);
            while table.slots[at] != 0 {
                at = (at + 1) & (room - 1);
            }
            table.slots[at] = table.keys.len() as u32;
        }
        table
    }

    fn places(&self, key: u64) -> &[FactPlace] {
        if self.slots.is_empty() {
            return &[];
        }
        let mut at = slot_of(key, self.slots.len());
        loop {
            let Some(entry) = (self.slots[at] as usize).checked_sub(1) else { return &[] };
            if self.keys[entry] == key {
                let start = entry.checked_sub(1).map_or(usize::default(), |before| self.ends[before] as usize);
                return &self.places[start..self.ends[entry] as usize];
            }
            at = (at + 1) & (self.slots.len() - 1);
        }
    }

    fn opened(&self) -> MixMap<u64, FactPlaces> {
        let mut map: MixMap<u64, FactPlaces> = MixMap::default();
        for (entry, key) in self.keys.iter().enumerate() {
            let start = entry.checked_sub(1).map_or(usize::default(), |before| self.ends[before] as usize);
            let held = &self.places[start..self.ends[entry] as usize];
            map.insert(*key, if let [only] = held { FactPlaces::One(*only) } else { FactPlaces::More(held.to_vec()) });
        }
        map
    }
}

const SHUT_ROOM: usize = 2;
because!(SHUT_ROOM, ChainOfRules, "the slots of a shut index for each key: twice as many, so that a free slot comes at least every other step and the search for a key that is not there ends soon");

const INDEX_FOLD: u32 = u64::BITS / 2;
because!(INDEX_FOLD, NumberMixing, "the bits by which the product of a key is folded onto itself: half of the width, so that the high half, which every bit of the key reaches, falls on the slot");

fn slot_of(key: u64, room: usize) -> usize {
    let mixed = key.wrapping_mul(MIX_FACTOR);
    ((mixed ^ (mixed >> INDEX_FOLD)) as usize) & (room - 1)
}
because!(slot_of, NumberMixing, "the first slot of a key in a shut index of the given power of two of slots: the key times the odd number of the hash, its high half folded onto its low half, so that keys that differ only in their high bits, as word numbers that keep their marks in their low bits do, still spread over the slots");

impl FactPlaces {
    fn put(&mut self, place: FactPlace) {
        match self {
            FactPlaces::One(only) if *only == place => {}
            FactPlaces::One(only) => *self = FactPlaces::More(vec![*only, place]),
            FactPlaces::More(list) if list.last() == Some(&place) => {}
            FactPlaces::More(list) => list.push(place),
        }
    }

    fn all(&self) -> &[FactPlace] {
        match self {
            FactPlaces::One(only) => std::slice::from_ref(only),
            FactPlaces::More(list) => list,
        }
    }
}

const START_DEPTH: usize = 3;
because!(START_DEPTH, ChainOfRules, "the most first words of a fact that the index of starts keeps a key for: a condition whose known start is longer takes the facts under the key of its first words and keeps those that begin with its whole start, the same facts in the same order as a key of the whole start would give, and the starts of a few words already hold a few facts each, so the index keeps a few keys for each fact in place of one for each of its words");

const DEEP_START: usize = 5;
because!(DEEP_START, ChainOfRules, "the one longer start of a fact that the index of starts keeps a key for besides the first few words: a condition with a known start of this many words or more takes the facts under the key of that start and keeps those that begin with its whole start, the same facts in the same order as the key of its first words would give, as many facts of a story begin with the same few words (a name and 'is a') and differ in the next ones (sibling, parent)");

fn fact_hash(fact: &[WordId]) -> u64 {
    BuildHasherDefault::<MixHasher>::default().hash_one(fact)
}
because!(fact_hash, NumberMixing, ChainOfRules, "a fact as one number for the set of the facts of a story: a hash of its words by the hash of numbers, so that the set keeps a place in place of a second copy of the words at the cost of one product for each few words; two facts that share a hash are told apart by their words in the list");

fn joined<'a>(under: Cow<'a, [FactPlace]>, own: &'a [FactPlace]) -> Cow<'a, [FactPlace]> {
    match own {
        [] => under,
        own if under.is_empty() => Cow::Borrowed(own),
        own => Cow::Owned(under.iter().chain(own.iter()).copied().collect()),
    }
}
because!(joined, ChainOfRules, "the places of the layers under a layer and of the layer itself, in order, borrowed when one of the two has none");

impl Facts {
    pub fn id(&self) -> u64 {
        self.id.0
    }

    pub fn on(base: &Arc<Facts>) -> Facts {
        Facts { under: Some(Arc::clone(base)), first: base.len(), history: base.history, ..Facts::default() }
    }

    pub(super) fn changed_facts<'a>(&'a self, other: &'a Facts) -> Option<Vec<(u64, &'a [WordId])>> {
        let same_under = match (&self.under, &other.under) {
            (Some(one), Some(two)) => Arc::ptr_eq(one, two),
            _ => false,
        };
        if !same_under || self.first != other.first || self.stopped_at_bound() != other.stopped_at_bound() {
            return None;
        }
        let mut out = Vec::new();
        let mut last_shared: Option<usize> = None;
        for place in self.first..self.len() {
            let fact = self.fact(place);
            match other.place_of(fact) {
                Some(at) if other.times_of(at) == self.times_of(place) => {
                    if last_shared.is_some_and(|last| at < last) {
                        return None;
                    }
                    last_shared = Some(at);
                }
                _ => out.push((words_mask(fact), fact)),
            }
        }
        for place in other.first..other.len() {
            let fact = other.fact(place);
            if self.place_of(fact).is_none() {
                out.push((words_mask(fact), fact));
            }
        }
        for place in self.moved.keys().chain(other.moved.keys()) {
            if self.times_of(*place) != other.times_of(*place) {
                out.push((words_mask(self.fact(*place)), self.fact(*place)));
            }
        }
        Some(out)
    }

    pub(super) fn reads_left(&self) -> usize {
        MOST_READS.saturating_sub(self.work.load(Ordering::Relaxed))
    }

    pub fn given_again(kept: &Arc<Facts>) -> Facts {
        Facts { work: AtomicUsize::new(kept.work.load(Ordering::Relaxed)), stopped: AtomicBool::new(kept.stopped_at_bound()), ..Facts::on(kept) }
    }

    pub fn stopped_at_bound(&self) -> bool {
        self.stopped.load(Ordering::Relaxed) || self.spent()
    }

    pub fn has_history(&self) -> bool {
        self.history
    }

    fn len(&self) -> usize {
        self.first + self.ends.len()
    }

    pub(super) fn place_of(&self, fact: &[WordId]) -> Option<usize> {
        let own = self.known.places(fact_hash(fact)).iter().map(|place| *place as usize).find(|place| self.fact(*place) == fact);
        own.or_else(|| self.under.as_ref()?.place_of(fact))
    }

    fn latest(&self, place: usize) -> Time {
        if place >= self.first {
            return self.times[place - self.first];
        }
        match (self.moved.get(&place).and_then(|times| times.last()), &self.under) {
            (Some(time), _) => *time,
            (None, Some(under)) => under.latest(place),
            (None, None) => Time::default(),
        }
    }

    pub fn add(&mut self, fact: Vec<WordId>, time: Time) -> Option<(usize, Time)> {
        if let Some(place) = self.place_of(&fact) {
            if self.history {
                let at = self.times_of(place).binary_search(&time).err()?;
                if place >= self.first {
                    self.past[place - self.first].insert(at, time);
                    self.times[place - self.first] = self.times[place - self.first].max(time);
                } else {
                    let mut times = self.times_of(place).to_vec();
                    times.insert(at, time);
                    self.moved.insert(place, times);
                }
                return Some((place, time));
            }
            if time > self.latest(place) {
                if place >= self.first {
                    self.times[place - self.first] = time;
                } else {
                    self.moved.insert(place, vec![time]);
                }
                return Some((place, time));
            }
            return None;
        }
        let place = self.len();
        self.push_fact(&fact, time, vec![time]);
        Some((place, time))
    }

    fn push_fact(&mut self, fact: &[WordId], time: Time, past: Vec<Time>) {
        let kept = self.len() as FactPlace;
        let mut state = u64::default();
        for (depth, word) in fact.iter().take(DEEP_START).enumerate() {
            let key = start_key(&mut state, *word);
            if depth < START_DEPTH || depth + 1 == DEEP_START {
                self.by_start.put(key, kept);
            }
        }
        for word in fact {
            self.by_word.put(u64::from(*word), kept);
        }
        self.known.put(fact_hash(fact), kept);
        self.words.extend_from_slice(fact);
        self.ends.push(self.words.len() as FactPlace);
        self.times.push(time);
        if self.history {
            self.past.push(past);
        }
    }

    pub(super) fn content_into(&self, hasher: &mut DefaultHasher) {
        self.history.hash(hasher);
        self.len().hash(hasher);
        for place in 0..self.len() {
            let fact = self.fact(place);
            fact.len().hash(hasher);
            for word in fact {
                word_text(*word).hash(hasher);
            }
            self.times_of(place).hash(hasher);
            self.told_at(place).hash(hasher);
        }
    }

    pub fn layer_into(&self, out: &mut Vec<u8>) {
        put_byte(out, u8::from(self.history));
        put_number(out, self.work.load(Ordering::Relaxed) as u64);
        put_byte(out, u8::from(self.stopped.load(Ordering::Relaxed)));
        put_number(out, self.first as u64);
        put_number(out, self.ends.len() as u64);
        for own in 0..self.ends.len() {
            let fact = self.fact(self.first + own);
            put_number(out, fact.len() as u64);
            for word in fact {
                put_text(out, word_text(*word));
            }
            put_number(out, self.times[own] as u64);
            if self.history {
                put_number(out, self.past[own].len() as u64);
                for time in &self.past[own] {
                    put_number(out, *time as u64);
                }
            }
        }
        put_number(out, self.told.len() as u64);
        for place in &self.told {
            put_number(out, u64::from(*place));
        }
        let mut moved: Vec<(&usize, &Vec<Time>)> = self.moved.iter().collect();
        moved.sort();
        put_number(out, moved.len() as u64);
        for (place, times) in moved {
            put_number(out, *place as u64);
            put_number(out, times.len() as u64);
            for time in times {
                put_number(out, *time as u64);
            }
        }
    }

    pub fn from_layer(bytes: &mut Bytes, base: &Arc<Facts>) -> Option<Facts> {
        let history = bytes.byte()? != 0;
        let work = usize::try_from(bytes.number()?).ok()?;
        let stopped = bytes.byte()? != 0;
        let first = usize::try_from(bytes.number()?).ok()?;
        if first != base.len() || history != base.history {
            return None;
        }
        let mut facts = Facts::on(base);
        let read_number = |bytes: &mut Bytes| bytes.number().and_then(|number| usize::try_from(number).ok());
        for _ in 0..read_number(bytes)? {
            let length = read_number(bytes)?;
            let mut fact: Vec<WordId> = Vec::with_capacity(length);
            for _ in 0..length {
                fact.push(word_number(bytes.text()?));
            }
            let time = read_number(bytes)?;
            let past = if history { (0..read_number(bytes)?).map(|_| read_number(bytes)).collect::<Option<Vec<Time>>>()? } else { Vec::new() };
            facts.push_fact(&fact, time, past);
        }
        for _ in 0..read_number(bytes)? {
            facts.told.push(FactPlace::try_from(read_number(bytes)?).ok()?);
        }
        for _ in 0..read_number(bytes)? {
            let place = read_number(bytes)?;
            let times = (0..read_number(bytes)?).map(|_| read_number(bytes)).collect::<Option<Vec<Time>>>()?;
            facts.moved.insert(place, times);
        }
        bytes.end()?;
        facts.work.store(work, Ordering::Relaxed);
        facts.stopped.store(stopped, Ordering::Relaxed);
        Some(facts.shut())
    }

    pub fn added_to(&self, base: &Facts) -> usize {
        self.len().saturating_sub(base.len())
    }

    pub fn told_facts(&self) -> Vec<&[WordId]> {
        let mut out: Vec<&[WordId]> = self.under.as_ref().map(|under| under.told_facts()).unwrap_or_default();
        out.extend(self.told.iter().map(|place| self.fact(*place as usize)));
        out
    }

    pub fn holds(&self, fact: &[WordId]) -> bool {
        self.place_of(fact).is_some()
    }

    pub fn holds_in_order(&self, words: &[WordId]) -> bool {
        let Some(rarest) = words.iter().min_by_key(|word| self.word_count(**word)) else { return true };
        self.indexed(u64::from(*rarest), |facts| &facts.by_word).iter().any(|place| {
            let mut rest = self.fact(*place as usize).iter();
            words.iter().all(|word| rest.any(|told| told == word))
        })
    }

    pub fn is_told(&self, fact: &[WordId]) -> bool {
        self.place_of(fact).is_some_and(|place| self.told_at(place))
    }

    fn told_at(&self, place: usize) -> bool {
        self.told.contains(&(place as FactPlace)) || self.under.as_ref().is_some_and(|under| under.told_at(place))
    }

    fn newer(&self, fact: &[WordId], time: Time) -> bool {
        match self.place_of(fact) {
            None => true,
            Some(place) if self.history => self.times_of(place).binary_search(&time).is_err(),
            Some(place) => self.latest(place) < time,
        }
    }

    pub(super) fn times_of(&self, place: usize) -> &[Time] {
        if place >= self.first {
            let own = place - self.first;
            return if self.history { &self.past[own] } else { std::slice::from_ref(&self.times[own]) };
        }
        match (self.moved.get(&place), &self.under) {
            (Some(times), _) => times,
            (None, Some(under)) => under.times_of(place),
            (None, None) => &[],
        }
    }

    pub fn fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        for place in 0..self.len() {
            self.fact(place).hash(&mut hasher);
            self.times_of(place).hash(&mut hasher);
        }
        hasher.finish()
    }

    pub fn holds_none(&self, sentences: &[Pattern]) -> bool {
        (0..self.len()).all(|place| sentences.iter().all(|sentence| !holds_words(sentence, self.fact(place))))
    }

    pub fn fact(&self, place: usize) -> &[WordId] {
        match &self.under {
            Some(under) if place < self.first => under.fact(place),
            _ => {
                let own = place - self.first;
                let start = if own == 0 { 0 } else { self.ends[own - 1] as usize };
                &self.words[start..self.ends[own] as usize]
            }
        }
    }

    pub fn exhausted(&self) -> bool {
        self.spent()
    }

    fn spent(&self) -> bool {
        self.work.load(Ordering::Relaxed) > MOST_READS
    }

    fn indexed(&self, key: u64, index: fn(&Facts) -> &FactIndex) -> Cow<'_, [FactPlace]> {
        let under = self.under.as_ref().map_or(Cow::Borrowed(&[][..]), |under| under.indexed(key, index));
        joined(under, index(self).places(key))
    }

    fn word_count(&self, word: WordId) -> usize {
        self.by_word.places(u64::from(word)).len() + self.under.as_ref().map_or(0, |under| under.word_count(word))
    }

    fn shut(&self) -> Facts {
        Facts { id: FactsId::default(), under: self.under.clone(), first: self.first, words: self.words.clone(), ends: self.ends.clone(), times: self.times.clone(), past: self.past.clone(), moved: self.moved.clone(), told: self.told.clone(), history: self.history, known: self.known.shut(), by_start: self.by_start.shut(), by_word: self.by_word.shut(), work: AtomicUsize::new(self.work.load(Ordering::Relaxed)), stopped: AtomicBool::new(self.stopped_at_bound()) }
    }

    pub(super) fn holds_word(&self, word: WordId) -> bool {
        self.word_count(word) > 0
    }

    fn candidates(&self, condition: &Pattern, filling: &Filling) -> Cow<'_, [FactPlace]> {
        let mut length = usize::default();
        for logic_piece in condition {
            match logic_piece {
                LogicPiece::Word(_) => length += 1,
                LogicPiece::Open(place) => match filling.get(*place) {
                    Some(words) => length += words.len(),
                    None => break,
                },
                LogicPiece::OpenWithEnd(place, ending) => match filling.get(*place) {
                    Some(words) if ending.is_empty() => length += words.len(),
                    _ => break,
                },
            }
        }
        let mut start: Vec<WordId> = Vec::with_capacity(length);
        for logic_piece in condition {
            match logic_piece {
                LogicPiece::Word(word) => start.push(*word),
                LogicPiece::Open(place) => match filling.get(*place) {
                    Some(words) => start.extend_from_slice(words),
                    None => break,
                },
                LogicPiece::OpenWithEnd(place, ending) => match filling.get(*place) {
                    Some(words) if ending.is_empty() => start.extend_from_slice(words),
                    _ => break,
                },
            }
        }
        let mut state = u64::default();
        let depth = if start.len() >= DEEP_START { DEEP_START } else { start.len().min(START_DEPTH) };
        let key = start.iter().take(depth).map(|word| start_key(&mut state, *word)).last();
        let found: Cow<'_, [FactPlace]> = match key {
            Some(key) if start.len() <= START_DEPTH => self.indexed(key, |facts| &facts.by_start),
            Some(key) => {
                let under_key = self.indexed(key, |facts| &facts.by_start);
                let mut kept = Vec::with_capacity(under_key.len());
                kept.extend(under_key.iter().copied().filter(|place| self.fact(*place as usize).starts_with(&start)));
                Cow::Owned(kept)
            }
            None => match condition.iter().filter_map(|logic_piece| fixed_word(logic_piece, filling)).min_by_key(|word| self.word_count(*word)) {
                Some(word) => self.indexed(u64::from(word), |facts| &facts.by_word),
                None => Cow::Owned((0..self.len()).map(|place| place as FactPlace).collect()),
            },
        };
        self.work.fetch_add(found.len(), Ordering::Relaxed);
        found
    }
}

fn fixed_word(logic_piece: &LogicPiece, filling: &Filling) -> Option<WordId> {
    match logic_piece {
        LogicPiece::Word(word) => Some(*word),
        LogicPiece::OpenWithEnd(_, ending) if !ending.is_empty() => None,
        LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) => filling.get(*place).and_then(|words| words.first()).copied(),
    }
}
because!(fixed_word, ChainOfRules, "a word that a fact must hold for a piece of a condition: the word itself, or the first word of an open place that the conditions before filled; nothing for a place that is still open or that adds an ending");

pub struct EnglishEndings;
source!(EnglishEndings, "the spelling of an English ending after a stem, as grammars of English state it (Quirk and others): before an ending that starts with a vowel the last letter of a short stem may be doubled (big, bigger), a y after a consonant turns to i (happy, happier), and a final e goes (nice, nicer); a word is cut at its ending as it stands, and a stem that another sentence gave first is compared with the cut under these three changes");

const VOWELS: &str = "aeiou";
because!(VOWELS, EnglishEndings, "the letters that start an ending before which the spelling of a stem may change");

const Y_AS_I: (char, char) = ('i', 'y');
because!(Y_AS_I, EnglishEndings, "the letter that a y of a stem turns into before an ending, and the y");

const DROPPED_E: char = 'e';
because!(DROPPED_E, EnglishEndings, "the letter at the end of a stem that goes before an ending that starts with a vowel");

fn stems_of(cut: &str, ending: &str) -> Vec<WordId> {
    let mut out = vec![word_number(cut)];
    if !ending.starts_with(|letter: char| VOWELS.contains(letter)) {
        return out;
    }
    let letters: Vec<char> = cut.chars().collect();
    if let [.., one, other] = letters.as_slice()
        && one == other
    {
        out.push(word_number(&letters[..letters.len() - 1].iter().collect::<String>()));
    }
    if let Some(before) = cut.strip_suffix(Y_AS_I.0) {
        out.push(word_number(&format!("{before}{}", Y_AS_I.1)));
    }
    out.push(word_number(&format!("{cut}{DROPPED_E}")));
    out
}
because!(stems_of, EnglishEndings, "the stems that a word cut at its ending may come from: the cut itself, and before an ending that starts with a vowel the cut with a doubled last letter made single, with a final i turned back to y, and with a final e put back");

struct Held<'a> {
    slots: [u8; FILLING_SLOTS],
    given: &'a Filling,
    words: Vec<Cow<'a, [WordId]>>,
}
because!(Held, ChainOfRules, "the filling of a search of fillings as it goes: for each place the number of its entry, the filling that the search starts from, read where it is, and the words of the places that the search fills, borrowed from the given words where they can be, in the order in which it filled them, since the search fills places one inside the other and the place that it filled last is the first that it opens again; get gives the words of a place, bind fills a place, release opens the place filled last, and filling is an owned copy for a fit, made as one list");

impl<'a> Held<'a> {
    fn get(&self, place: usize) -> Option<&[WordId]> {
        let at = usize::from(self.slots[place]).checked_sub(1)?;
        match at.checked_sub(usize::from(self.given.entries)) {
            Some(own) => Some(&*self.words[own]),
            None => {
                let (start, end) = self.given.span(at);
                Some(&self.given.words[start..end])
            }
        }
    }

    fn bind(&mut self, place: usize, words: Cow<'a, [WordId]>) {
        self.words.push(words);
        self.slots[place] = (usize::from(self.given.entries) + self.words.len()) as u8;
    }

    fn release(&mut self, place: usize) {
        self.words.pop();
        self.slots[place] = 0;
    }

    fn filling(&self) -> Filling {
        let first = usize::from(self.given.entries);
        let given = &self.given.words;
        let mut words = Vec::with_capacity(given.len() + self.words.len() + self.words.iter().map(|bound| bound.len()).sum::<usize>());
        words.extend_from_slice(&given[..first]);
        let mut end = given.len() - first;
        for bound in &self.words {
            end += bound.len();
            words.push(end as WordId);
        }
        words.extend_from_slice(&given[first..]);
        for bound in &self.words {
            words.extend_from_slice(bound);
        }
        Filling { slots: self.slots, entries: (first + self.words.len()) as u8, words }
    }
}

fn fit_from<'a>(pieces: &[LogicPiece], words: &'a [WordId], held: &mut Held<'a>, out: &mut Vec<Filling>) {
    let Some((first, rest)) = pieces.split_first() else {
        if words.is_empty() {
            out.push(held.filling());
        }
        return;
    };
    match first {
        LogicPiece::Word(word) => {
            if words.first() == Some(word) {
                fit_from(rest, &words[1..], held, out);
            }
        }
        LogicPiece::OpenWithEnd(place, ending) => {
            let Some(word) = words.first().copied().filter(|word| place_takes(*place) == PlaceTakes::Signs || word_has(*word, WORD_IS_TEXT)) else { return };
            if ending.is_empty() {
                match held.get(*place).map(|known| known == [word]) {
                    Some(true) => fit_from(rest, &words[1..], held, out),
                    Some(false) => {}
                    None => {
                        held.bind(*place, Cow::Borrowed(&words[..1]));
                        fit_from(rest, &words[1..], held, out);
                        held.release(*place);
                    }
                }
                return;
            }
            let Some(cut) = word_text(word).strip_suffix(ending.as_str()) else { return };
            if cut.is_empty() {
                return;
            }
            match held.get(*place).map(|known| matches!(known, [stem] if stems_of(cut, ending).contains(stem))) {
                Some(true) => fit_from(rest, &words[1..], held, out),
                Some(false) => {}
                None => {
                    held.bind(*place, Cow::Owned(vec![word_number(cut)]));
                    fit_from(rest, &words[1..], held, out);
                    held.release(*place);
                }
            }
        }
        LogicPiece::Open(place) => match held.get(*place).map(|known| words.starts_with(known).then_some(known.len())) {
            Some(Some(after)) => fit_from(rest, &words[after..], held, out),
            Some(None) => {}
            None if place_takes(*place) == PlaceTakes::Groups => {
                let fixed_after = rest.iter().filter(|logic_piece| matches_one_word(logic_piece)).count();
                let mut depth = usize::default();
                for end in 1..=words.len().saturating_sub(fixed_after) {
                    match group_turn(word_text(words[end - 1])) {
                        Some(true) => depth += 1,
                        Some(false) => {
                            let Some(less) = depth.checked_sub(1) else { break };
                            depth = less;
                        }
                        None => {}
                    }
                    if depth == usize::default() {
                        held.bind(*place, Cow::Borrowed(&words[..end]));
                        fit_from(rest, &words[end..], held, out);
                        held.release(*place);
                    }
                }
            }
            None => {
                let fixed_after = rest.iter().filter(|logic_piece| matches_one_word(logic_piece)).count();
                let longest = match place_takes(*place) {
                    PlaceTakes::Signs => words.len(),
                    PlaceTakes::Marks => words.iter().take_while(|word| !word_has(**word, WORD_IS_BRACKET)).count(),
                    PlaceTakes::Words | PlaceTakes::Groups => words.iter().take_while(|word| word_has(**word, WORD_IS_TEXT)).count(),
                };
                for end in 1..=longest.min(words.len().saturating_sub(fixed_after)) {
                    held.bind(*place, Cow::Borrowed(&words[..end]));
                    fit_from(rest, &words[end..], held, out);
                    held.release(*place);
                }
            }
        },
    }
}
because!(fit_from, ChainOfRules, "every filling under which a sentence with open places is the given words: a word must stand at its place, an open place that a condition filled before must hold the same words (a stem before an ending, the same stem under the changes of English spelling), and an open place that is still open takes one or more words, or one word (cut at its ending when it has one), leaving at least one word for each word or place of one word after it; an open place takes words only, never a mark, but a place that may hold marks takes any marks but the brackets of a name, a place that may hold groups takes words, marks and whole groups in brackets, every bracket that opens in it closed in it, and a place of one sign or of any signs takes any signs; one filling is changed and set back as the search goes, holding the words of the places that it fills as parts of the given words, a place filled last opened first, and copied only for a fit");

pub fn holds_words(pattern: &Pattern, words: &[WordId]) -> bool {
    let mut rest = words.iter();
    pattern.iter().all(|logic_piece| match logic_piece {
        LogicPiece::Word(word) => rest.any(|told| told == word),
        _ => true,
    })
}
because!(holds_words, ChainOfRules, "whether a fact holds the fixed words of a pattern in their order: a fact that misses one cannot fit, and this test is far cheaper than trying every split of the words among the open places, which most facts of a story would fail for most rules");

fn held_filling(filling: &Filling) -> Held<'_> {
    Held { slots: filling.slots, given: filling, words: Vec::new() }
}
because!(held_filling, ChainOfRules, "a filling as the search of fillings holds it, read where it is with no copy: made once for many facts, since the search sets back every place that it fills");

pub(super) fn fit_into(pieces: &[LogicPiece], words: &[WordId], filling: &Filling, out: &mut Vec<Filling>) {
    let mut held = held_filling(filling);
    fit_from(pieces, words, &mut held, out);
}
because!(fit_into, ChainOfRules, "every filling under which a sentence with open places is the given words, from a filling that stays as it is");

fn matches_one_word(logic_piece: &LogicPiece) -> bool {
    !matches_open(logic_piece)
}
because!(matches_one_word, ChainOfRules, "whether a piece takes exactly one word: a word and an open place with an ending do, an open place may take more");

fn matches_open(logic_piece: &LogicPiece) -> bool {
    if let LogicPiece::Open(_) = logic_piece {
        return true;
    }
    false
}
because!(matches_open, ChainOfRules, "whether a piece is an open place that may take more than one word");

pub fn fill(pattern: &Pattern, filling: &Filling) -> Option<Vec<WordId>> {
    let length = pattern.iter().map(|logic_piece| match logic_piece {
        LogicPiece::Open(place) => filling.get(*place).map_or(usize::default(), <[WordId]>::len),
        LogicPiece::Word(_) | LogicPiece::OpenWithEnd(..) => 1,
    }).sum();
    let mut out = Vec::with_capacity(length);
    for logic_piece in pattern {
        match logic_piece {
            LogicPiece::Word(word) => out.push(*word),
            LogicPiece::Open(place) => out.extend(filling.get(*place)?.iter().copied()),
            LogicPiece::OpenWithEnd(place, ending) => {
                let words = filling.get(*place)?;
                let [stem] = words else { return None };
                out.push(if ending.is_empty() { *stem } else { word_number(&format!("{}{ending}", word_text(*stem))) });
            }
        }
    }
    Some(out)
}
because!(fill, ChainOfRules, "a sentence with its open places filled, and nothing while a place is still open; its words are counted before they are written, so that the sentence takes one piece of memory");


fn newest_where(times: &[Time], keep: impl Fn(Time) -> bool) -> &[Time] {
    times.iter().rposition(|at| keep(*at)).map_or(&times[..0], |at| &times[at..=at])
}
because!(newest_where, ChainOfRules, "the latest of the places in the story of a fact, in order, that pass a test, and none when no place passes it");

fn fit_times(times: &[Time], when: Moment, anchor: Time) -> &[Time] {
    match when {
        Moment::Any => times,
        Moment::Latest => newest_where(times, |_| true),
        Moment::AtThatTime => newest_where(times, |at| at <= anchor),
        Moment::BeforeThatTime => newest_where(times, |at| at < anchor),
    }
}
because!(fit_times, ChainOfRules, "the places in the story at which a fact may hold a condition: every place for any fact, else its latest place, its latest up to the time of the clause before, or its latest before it");

fn fact_fits(pattern: &Pattern, when: Moment, facts: &Facts, filling: &Filling, anchor: Time) -> Vec<TimedFilling> {
    let open = pattern.iter().any(|logic_piece| match logic_piece {
        LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) => !filling.filled(*place),
        LogicPiece::Word(_) => false,
    });
    if when != Moment::Any && !open {
        return state_fits(pattern, when, facts, filling, anchor);
    }
    let mut out: Vec<TimedFilling> = Vec::new();
    let mut held = held_filling(filling);
    let mut found = Vec::new();
    for place in facts.candidates(pattern, filling).iter().map(|place| *place as usize) {
        let Some((first, later)) = fit_times(facts.times_of(place), when, anchor).split_first() else { continue };
        if !holds_words(pattern, facts.fact(place)) {
            continue;
        }
        fit_from(pattern, facts.fact(place), &mut held, &mut found);
        for filled in found.drain(..) {
            out.extend(later.iter().rev().map(|at| (filled.clone(), *at)));
            out.push((filled, *first));
        }
    }
    if when == Moment::Any {
        return out;
    }
    let Some(newest) = out.iter().map(|(_, time)| *time).max() else { return out };
    out.into_iter().filter(|(_, time)| *time == newest).collect()
}
because!(fact_fits, ChainOfRules, "every filling under which a sentence is a fact, with the place in the story of that fact, once for each place of its history, the latest first, so that an answer about a story rests on the latest telling of a fact told more than once; for a sentence that only one fact may hold, the fits of the latest place that the condition allows alone, and for such a sentence with no place left open, the fits of the state that it names");

fn state_fits(pattern: &Pattern, when: Moment, facts: &Facts, filling: &Filling, anchor: Time) -> Vec<TimedFilling> {
    let Some((value, frame)) = pattern.split_last() else { return Vec::new() };
    let mut probe = filling.clone();
    let mut asked = frame.to_vec();
    match value {
        LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) => {
            probe.clear(*place);
            asked.push(value.clone());
        }
        LogicPiece::Word(_) => {
            probe.clear(OPEN_PLACES);
            asked.push(LogicPiece::Open(OPEN_PLACES));
        }
    }
    let same = |filled: &Filling| match value {
        LogicPiece::Word(word) => filled.get(OPEN_PLACES) == Some(std::slice::from_ref(word)),
        LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) => filled.get(*place) == filling.get(*place),
    };
    fact_fits(&asked, when, facts, &probe, anchor)
        .into_iter()
        .filter(|(filled, _)| same(filled))
        .map(|(mut filled, time)| {
            filled.clear(OPEN_PLACES);
            (filled, time)
        })
        .collect()
}
because!(state_fits, ChainOfRules, "the fits of a sentence with no open place left that only one fact may hold: its last piece, a word or a place, is the state of the thing that the pieces before it name, as English puts the new part of a sentence last (end focus); the last piece is opened, and the latest fact of the same thing that the condition allows must hold the same words there");

const MOST_PLACES_AFTER_POINT: usize = 12;
because!(MOST_PLACES_AFTER_POINT, ChainOfRules, "the most digits after the point of a result that the engine writes: sums, differences and products of whole numbers end before them");

fn piece_words(logic_piece: &LogicPiece, filling: &Filling) -> Option<Vec<WordId>> {
    match logic_piece {
        LogicPiece::Word(word) => Some(vec![*word]),
        LogicPiece::OpenWithEnd(_, ending) if !ending.is_empty() => None,
        LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) => filling.get(*place).map(<[WordId]>::to_vec),
    }
}
because!(piece_words, ChainOfRules, "the words that a piece of a worked condition stands for: its word, or the words of its open place, and nothing while the place is open or adds an ending");

pub(super) fn reads_as_number(word: WordId) -> bool {
    number_of(&[word]).is_some()
}
because!(reads_as_number, ChainOfRules, "whether a word is a number that a step of arithmetic reads: digits, with a point or a sign below nothing, or a whole number over a whole number");

fn number_of(words: &[WordId]) -> Option<Amount> {
    let [word] = words else { return None };
    if word_has(*word, WORD_IS_DIGITS) {
        return Amount::from_digits(word_text(*word), "");
    }
    let text = word_text(*word);
    if let Some((top, bottom)) = text.split_once(NUMBER_OVER) {
        if bottom.contains(NUMBER_OVER) {
            return None;
        }
        return number_of(&[word_number(top)])?.over(number_of(&[word_number(bottom)])?);
    }
    let (below_nothing, text) = match text.strip_prefix(NUMBER_BELOW_NOTHING) {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let digits = |part: &str| !part.is_empty() && part.bytes().all(|sign| sign.is_ascii_digit());
    let amount = match text.split_once(NUMBER_POINT) {
        Some((whole, below)) => (digits(whole) && digits(below)).then(|| Amount::from_digits(whole, below)).flatten()?,
        None => digits(text).then(|| Amount::from_digits(text, "")).flatten()?,
    };
    if below_nothing { amount.negated() } else { Some(amount) }
}
because!(number_of, ChainOfRules, "the number that the words of a place write: one word of digits, or of digits, a point and digits, with a minus sign before them for a number below nothing, as an exact amount, and nothing for any other words; or one word of a whole number, the mark over, and a whole number, as a step writes an amount whose decimal does not end; a word with the mark twice is no number");

fn settled(result: &LogicPiece, value: Vec<WordId>, filling: &Filling, time: Time) -> Vec<TimedFilling> {
    let open = match result {
        LogicPiece::Open(place) => Some(*place),
        LogicPiece::OpenWithEnd(place, ending) if ending.is_empty() && value.len() == 1 => Some(*place),
        _ => None,
    };
    match open.filter(|place| !filling.filled(*place)) {
        Some(place) => {
            let mut next = filling.clone();
            next.set(place, value);
            vec![(next, time)]
        }
        None if piece_words(result, filling) == Some(value) => vec![(filling.clone(), time)],
        None => Vec::new(),
    }
}
because!(settled, ChainOfRules, "the filling under which a worked out value is the result: an open result takes the value (a place of one word only a value of one word), and a filled one must hold the same words");

fn holds_comparison(comparison: Comparison, first: &LogicPiece, second: &LogicPiece, filling: &Filling) -> bool {
    let (Some(one), Some(other)) = (piece_words(first, filling), piece_words(second, filling)) else { return false };
    if comparison == Comparison::Same {
        return one == other;
    }
    if comparison == Comparison::Earlier {
        return word_texts(&one) < word_texts(&other);
    }
    let (Some(one), Some(other)) = (number_of(&one), number_of(&other)) else { return false };
    match comparison {
        Comparison::Less => one < other,
        Comparison::More => one > other,
        Comparison::AtLeast => one >= other,
        Comparison::AtMost => one <= other,
        Comparison::Same | Comparison::Earlier => one == other,
    }
}
because!(holds_comparison, ChainOfRules, "whether a comparison holds: the same words, words in the order of the alphabet (by the order of their letters, word by word, as a dictionary sorts them), or two numbers in the order that it names; nothing holds while a place is open or does not write a number");

fn reckoned(reckoning: Reckoning, first: &LogicPiece, second: &LogicPiece, filling: &Filling) -> (Option<Vec<WordId>>, bool) {
    let (Some(one), Some(other)) = (piece_words(first, filling), piece_words(second, filling)) else { return (None, false) };
    worked_words(reckoning, &one, &other)
}
because!(reckoned, ChainOfRules, "the words of the result of a step of arithmetic on the words of two pieces, and nothing while a piece stands for no words, as worked_words gives it");

pub(super) fn worked_words(reckoning: Reckoning, one: &[WordId], other: &[WordId]) -> (Option<Vec<WordId>>, bool) {
    let (Some(one), Some(other)) = (number_of(one), number_of(other)) else { return (None, false) };
    if reckoning == Reckoning::Over && other == Amount::whole(0) {
        return (None, false);
    }
    let value = match reckoning {
        Reckoning::Plus => one.plus(other),
        Reckoning::Minus => one.minus(other),
        Reckoning::Times => one.times(other),
        Reckoning::Over => one.over(other),
        Reckoning::Whole => Some(one.whole_part()),
    };
    let Some(value) = value else { return (None, true) };
    match value.written(MOST_PLACES_AFTER_POINT) {
        Some(written) => (Some(text_words(&written).iter().map(|word| word_number(word)).collect()), false),
        None => (Some(vec![word_number(&value.written_over(NUMBER_OVER))]), false),
    }
}
because!(worked_words, ChainOfRules, "the words of the result of a step of arithmetic on the words of two numbers, and nothing while a number is missing or a division is by nought; a result whose whole numbers leave the width of an exact amount gives nothing and says so, since the chain then stops at a bound (an exact fraction of a long chain of divisions can grow past the width); an amount whose decimal does not end within the places is one word of its whole number over its whole number, so that a later step reads it exactly");

fn counted(pattern: &Pattern, place: usize, facts: &Facts, filling: &Filling) -> (usize, Time) {
    let mut probe = filling.clone();
    probe.clear(place);
    let mut seen: Vec<Vec<WordId>> = Vec::new();
    let mut time = 0;
    for (filled, at) in fact_fits(pattern, Moment::Any, facts, &probe, STORY_END) {
        if let Some(words) = filled.get(place).map(<[WordId]>::to_vec)
            && !seen.contains(&words)
        {
            seen.push(words);
            time = time.max(at);
        }
    }
    (seen.len(), time)
}
because!(counted, ChainOfRules, "how many different words a place takes in the facts that fit a sentence under a filling, and the latest place in the story of those facts");

fn totalled(pattern: &Pattern, place: usize, facts: &Facts, filling: &Filling) -> (Option<Vec<WordId>>, Time) {
    let mut probe = filling.clone();
    probe.clear(place);
    let mut seen: Vec<Vec<WordId>> = Vec::new();
    let mut total = Some(Amount::whole(i128::default()));
    let mut time = 0;
    for (filled, at) in fact_fits(pattern, Moment::Any, facts, &probe, STORY_END) {
        let Some(fact) = fill(pattern, &filled) else { continue };
        if seen.contains(&fact) {
            continue;
        }
        seen.push(fact);
        time = time.max(at);
        total = match (total, filled.get(place).and_then(number_of)) {
            (Some(sum), Some(value)) => sum.plus(value),
            _ => None,
        };
    }
    (total.and_then(|sum| sum.written(MOST_PLACES_AFTER_POINT)).map(|written| text_words(&written).iter().map(|word| word_number(word)).collect()), time)
}
because!(totalled, ChainOfRules, "the sum of the numbers that a place holds in the different facts that fit a sentence under a filling, each fact once, written as words, and the latest place in the story of those facts; nothing when a fact holds no number there or the sum leaves the width of an exact amount; the sum of no fact is nought");

pub struct LargestFlow;
source!(LargestFlow, "a building block of the engine, a condition worked out over all the facts of a form, as a count and a sum are: the largest flow through a network of edges with capacities, a quantity of any network of amounts (water in pipes, cars on roads, money between accounts, goods between places), so that any text that tells such a network can ask it, not one set; found by augmenting paths breadth first (Ford and Fulkerson; Edmonds and Karp) on exact amounts: while a path from the start to the end has room left on every edge, the flow grows by the least room on it, and a flow on an edge may be sent back");

fn flowed(pattern: &Pattern, (start, end, capacity): (usize, usize, usize), (source, sink): (&LogicPiece, &LogicPiece), facts: &Facts, filling: &Filling) -> (Option<Vec<WordId>>, Time) {
    let (Some(from), Some(to)) = (piece_words(source, filling), piece_words(sink, filling)) else { return (None, 0) };
    let mut probe = filling.clone();
    for place in [start, end, capacity] {
        probe.clear(place);
    }
    let mut nodes: Vec<Vec<WordId>> = vec![from, to];
    let mut edges: Vec<(usize, usize, Amount)> = Vec::new();
    let mut seen: Vec<Vec<WordId>> = Vec::new();
    let mut time = 0;
    for (filled, at) in fact_fits(pattern, Moment::Any, facts, &probe, STORY_END) {
        let Some(fact) = fill(pattern, &filled) else { continue };
        if seen.contains(&fact) {
            continue;
        }
        seen.push(fact);
        let (Some(head), Some(tail), Some(room)) = (filled.get(start).map(<[WordId]>::to_vec), filled.get(end).map(<[WordId]>::to_vec), filled.get(capacity).and_then(number_of)) else { return (None, 0) };
        time = time.max(at);
        let mut node = |words: Vec<WordId>| nodes.iter().position(|known| *known == words).unwrap_or_else(|| {
            nodes.push(words);
            nodes.len() - 1
        });
        let (head, tail) = (node(head), node(tail));
        edges.push((head, tail, room));
    }
    if nodes[0] == nodes[1] {
        return (None, 0);
    }
    let zero = Amount::whole(i128::default());
    let mut room = vec![vec![zero; nodes.len()]; nodes.len()];
    for (head, tail, more) in edges {
        let Some(sum) = room[head][tail].plus(more) else { return (None, 0) };
        room[head][tail] = sum;
    }
    let mut flow = zero;
    loop {
        let mut came: Vec<Option<usize>> = vec![None; nodes.len()];
        let mut queue = std::collections::VecDeque::from([0]);
        came[0] = Some(0);
        while let Some(at) = queue.pop_front() {
            for next in 0..nodes.len() {
                if came[next].is_none() && room[at][next] > zero {
                    came[next] = Some(at);
                    queue.push_back(next);
                }
            }
        }
        let sink_place = 1;
        if came[sink_place].is_none() {
            break;
        }
        let mut least: Option<Amount> = None;
        let mut at = sink_place;
        while at != 0 {
            let Some(before) = came[at] else { break };
            least = Some(least.map_or(room[before][at], |known| known.min(room[before][at])));
            at = before;
        }
        let Some(least) = least else { break };
        let mut at = sink_place;
        while at != 0 {
            let Some(before) = came[at] else { break };
            let (Some(less), Some(back)) = (room[before][at].minus(least), room[at][before].plus(least)) else { return (None, 0) };
            room[before][at] = less;
            room[at][before] = back;
            at = before;
        }
        let Some(more) = flow.plus(least) else { return (None, 0) };
        flow = more;
    }
    (flow.written(MOST_PLACES_AFTER_POINT).map(|written| text_words(&written).iter().map(|word| word_number(word)).collect()), time)
}
because!(flowed, LargestFlow, "the largest flow from the words of the start to the words of the end over the edges that the facts fitting a sentence tell (each fact once, its places the start, the end and the capacity of an edge; parallel edges add their capacities), written as words, and the latest place in the story of those facts; nought when no path joins them, nothing when the start is the end, a fact holds no number as its capacity, or an amount leaves the width");

pub(super) fn every_fit(pattern: &Pattern, facts: &Facts) -> Vec<(Vec<Option<Vec<String>>>, Time)> {
    fact_fits(pattern, Moment::Any, facts, &empty_filling(), STORY_END).into_iter().map(|(filling, time)| ((0..OPEN_PLACES).map(|place| filling.get(place).map(word_texts)).collect(), time)).collect()
}
because!(every_fit, ChainOfRules, "every filling under which a sentence is a fact, with the place in the story of that fact, each place as the words that fill it in letters, for a building block that reads all the facts of a form");

pub(super) fn condition_fits(condition: &Condition, facts: &Facts, filling: &Filling, anchor: Time) -> Vec<TimedFilling> {
    match condition {
        Condition::Holds { sentence, word } => {
            let Some(told) = piece_words(sentence, filling) else { return Vec::new() };
            let asked = piece_words(word, filling).and_then(|words| if let [one] = words.as_slice() { Some(*one) } else { None });
            held_words(&told, asked).into_iter().flat_map(|held| settled(word, vec![held], filling, 0)).collect()
        }
        Condition::After { sentence, earlier, later } => {
            let Some(told) = piece_words(sentence, filling) else { return Vec::new() };
            let one = |logic_piece: &LogicPiece| piece_words(logic_piece, filling).and_then(|words| if let [single] = words.as_slice() { Some(*single) } else { None });
            later_pairs(&told, one(earlier), one(later))
                .into_iter()
                .flat_map(|(first, second)| settled(earlier, vec![first], filling, 0).into_iter().flat_map(move |(next, time)| settled(later, vec![second], &next, time)))
                .collect()
        }
        Condition::Told(sentence) => match piece_words(sentence, filling) {
            Some(words) if facts.is_told(&words) => vec![(filling.clone(), 0)],
            _ => Vec::new(),
        },
        Condition::HoldsKind { sentence, kind } => match piece_words(sentence, filling) {
            Some(words) if words.iter().any(|word| facts.holds(&[std::slice::from_ref(word), kind.as_slice()].concat())) => vec![(filling.clone(), 0)],
            _ => Vec::new(),
        },
        Condition::Backward { result, run } => match piece_words(run, filling) {
            Some(words) => settled(result, words.into_iter().rev().collect(), filling, 0),
            None => Vec::new(),
        },
        Condition::Letter { word, letter, place: None } => match piece_words(word, filling).as_deref() {
            Some([one]) => first_letter(*one).map_or_else(Vec::new, |found| settled(letter, vec![found], filling, 0)),
            _ => Vec::new(),
        },
        Condition::Letter { word, letter, place: Some(place) } => match piece_words(word, filling).as_deref() {
            Some([one]) => placed_letters(*one)
                .into_iter()
                .flat_map(|(at, found)| settled(place, vec![at], filling, 0).into_iter().flat_map(move |(next, time)| settled(letter, vec![found], &next, time)))
                .collect(),
            _ => Vec::new(),
        },
        Condition::Proves { statement, holds } => match fill(statement, filling) {
            Some(words) if refuted(&words, facts, !*holds) => vec![(filling.clone(), 0)],
            _ => Vec::new(),
        },
        Condition::Follows { statement, said } => {
            let told = fact_fits(said, Moment::Any, facts, filling, anchor);
            match fill(statement, filling) {
                Some(words) if told.is_empty() && follows(&word_texts(&words), facts) => vec![(filling.clone(), 0)],
                _ => told,
            }
        }
        Condition::Fact { pattern, when } => fact_fits(pattern, *when, facts, filling, anchor),
        Condition::Compare(comparison, first, second) => {
            if holds_comparison(*comparison, first, second, filling) { vec![(filling.clone(), 0)] } else { Vec::new() }
        }
        Condition::Alike { first, second } => match (fill(first, filling), fill(second, filling)) {
            (Some(one), Some(other)) if one == other => vec![(filling.clone(), 0)],
            _ => Vec::new(),
        },
        Condition::Reckon(reckoning, first, second, result) => match reckoned(*reckoning, first, second, filling) {
            (Some(value), _) => settled(result, value, filling, 0),
            (None, beyond) => {
                if beyond {
                    facts.stopped.store(true, Ordering::Relaxed);
                }
                Vec::new()
            }
        },
        Condition::Count { result, counted: place, pattern, tally: Tally::Number } => {
            let (count, time) = counted(pattern, *place, facts, filling);
            settled(result, vec![word_number(&count.to_string())], filling, time)
        }
        Condition::Count { result, counted: place, pattern, tally: Tally::Sum } => match totalled(pattern, *place, facts, filling) {
            (Some(total), time) => settled(result, total, filling, time),
            (None, _) => Vec::new(),
        },
        Condition::Count { result, counted: start, pattern, tally: Tally::Flow { source, sink, end, capacity } } => match flowed(pattern, (*start, *end, *capacity), (source, sink), facts, filling) {
            (Some(flow), time) => settled(result, flow, filling, time),
            (None, _) => Vec::new(),
        },
    }
}
because!(condition_fits, ChainOfRules, "every filling under which a condition holds, with the place in the story of the facts that it read: the fits of a sentence that must be a fact, read at the time of the clause before it when it says so, or the filling that a comparison, a step of arithmetic or a count holds under; for 'it follows that', the facts that say it, else the filling when the worlds of the story give it; a letter at a place gives each place of the word and its letter in turn, a filled place or letter keeping only the fits that hold it");

fn anchored(condition: &Condition) -> bool {
    matches!(condition, Condition::Fact { when: Moment::AtThatTime | Moment::BeforeThatTime, .. })
}
because!(anchored, ChainOfRules, "whether a condition reads the facts at the time of the clause before it");

fn anchor_after(condition: &Condition, anchor: Time, at: Time) -> Time {
    match condition {
        Condition::Fact { .. } | Condition::Count { .. } => at,
        Condition::Compare(..) | Condition::Alike { .. } | Condition::Reckon(..) | Condition::Follows { .. } | Condition::Holds { .. } | Condition::HoldsKind { .. } | Condition::Backward { .. } | Condition::After { .. } | Condition::Proves { .. } | Condition::Letter { .. } | Condition::Told(..) => anchor,
    }
}
because!(anchor_after, ChainOfRules, "the time that a later condition 'at that time' reads after a condition: the place in the story of the fact that it read, also when that condition read the facts at an earlier time, so that a chain of them reaches back to the earliest (what one person knows of what another saw); a comparison or a step of arithmetic reads no fact and leaves the time as it was");

pub fn keeps_history(rules: &[ToldRule], questions: &[AskedForm]) -> bool {
    rules.iter().flat_map(|rule| &rule.conditions).chain(questions.iter().flat_map(|form| &form.replies).flat_map(|answer| &answer.conditions)).any(anchored)
}
because!(keeps_history, ChainOfRules, "whether the facts of a topic keep every place in the story at which they came: only when a rule or an answer reads facts at the time of a clause before it, since a fact told twice holds at both times, and the other topics keep their speed");

fn piece_filled(logic_piece: &LogicPiece, filling: &Filling) -> bool {
    match logic_piece {
        LogicPiece::Word(_) => true,
        LogicPiece::OpenWithEnd(_, ending) if !ending.is_empty() => false,
        LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) => filling.filled(*place),
    }
}
because!(piece_filled, ChainOfRules, "whether a piece of a worked condition stands for words now, as piece_words would give them, with no copy of the words");

fn ready(condition: &Condition, filling: &Filling) -> bool {
    let filled = |logic_piece: &LogicPiece| piece_filled(logic_piece, filling);
    match condition {
        Condition::Fact { .. } | Condition::Count { .. } => true,
        Condition::Compare(_, first, second) | Condition::Reckon(_, first, second, _) => filled(first) && filled(second),
        Condition::Alike { first, second } => fill(first, filling).is_some() && fill(second, filling).is_some(),
        Condition::Follows { .. } => true,
        Condition::Proves { statement, .. } => fill(statement, filling).is_some(),
        Condition::Holds { sentence, .. } | Condition::HoldsKind { sentence, .. } | Condition::After { sentence, .. } => filled(sentence),
        Condition::Backward { run, .. } => filled(run),
        Condition::Letter { word, .. } | Condition::Told(word) => filled(word),
    }
}
because!(ready, ChainOfRules, "whether a condition can be read under a filling: a sentence and a count always, a comparison or a step of arithmetic once the places that it reads are filled");

fn join(conditions: &mut [&Condition], facts: &Facts, filling: &Filling, (time, anchor, pinned): Clock, out: &mut Vec<TimedFilling>) {
    if conditions.is_empty() {
        out.push((filling.clone(), time));
        return;
    }
    if facts.spent() {
        return;
    }
    let plain = |condition: &&Condition| match condition {
        Condition::Fact { pattern, .. } => pattern.iter().all(|logic_piece| !matches!(logic_piece, LogicPiece::OpenWithEnd(place, ending) if !ending.is_empty() && !filling.filled(*place))),
        _ => true,
    };
    let fact_like = |condition: &&Condition| !matches!(condition, Condition::Count { .. } | Condition::Follows { .. } | Condition::Proves { .. });
    let pick = if conditions.iter().any(|condition| anchored(condition)) {
        ready(conditions[0], filling).then_some(0)
    } else {
        let bound = |condition: &&Condition| match condition {
            Condition::Fact { pattern, .. } => pattern.iter().filter(|logic_piece| matches!(logic_piece, LogicPiece::Open(place) | LogicPiece::OpenWithEnd(place, _) if filling.filled(*place))).count(),
            _ => usize::MAX,
        };
        conditions
            .iter()
            .enumerate()
            .filter(|(_, condition)| ready(condition, filling) && plain(condition) && fact_like(condition))
            .min_by_key(|(_, condition)| std::cmp::Reverse(bound(condition)))
            .map(|(at, _)| at)
            .or_else(|| conditions.iter().position(|condition| ready(condition, filling) && fact_like(condition)))
            .or_else(|| conditions.iter().position(|condition| ready(condition, filling)))
    };
    let Some(pick) = pick else { return };
    let picked = conditions[pick];
    let pins = anchored(picked) && !pinned && anchor != STORY_END;
    let fits = condition_fits(picked, facts, filling, anchor);
    conditions[..=pick].rotate_right(1);
    for (next, at) in fits {
        let after = if pins { anchor } else { time.max(at) };
        join(&mut conditions[1..], facts, &next, (after, anchor_after(picked, anchor, at), pinned || anchored(picked)), out);
    }
    conditions[..=pick].rotate_left(1);
}
because!(join, ChainOfRules, "every filling under which all conditions hold, each condition read with the places that the conditions before it filled, of the conditions that can be read, the one with the most places that the conditions before it filled taken first (a comparison or a step of arithmetic at once), the first in the order written among equals, so that a rule whose new fact fills the places of a later condition reads that condition before one with no place filled, and one that fills a place through an ending only after those that may fill it with a plain word (a place of one word is filled with a plain word), so that a stem is compared under the changes of spelling, and a count only after every other condition that can be read, so that the places that it shares with them are filled when it counts; while a condition at the time of the clause before it waits, the conditions are read in the order written, so that it reads the time of the condition written before it; with the latest place in the story of the facts read, but from the first condition 'at that time' or 'before that time' on, the place of the clause that it points to in place of the places of the conditions before that clause; no filling when no condition left can be read; the conditions left are read from the same list, with the one read moved before them and set back after them in their order, so that a join of many conditions makes no new list at each step");

pub(super) fn holding(conditions: &[&Condition], facts: &Facts, filling: &Filling) -> Vec<Filling> {
    let mut out = Vec::new();
    join(&mut conditions.to_vec(), facts, filling, (0, STORY_END, false), &mut out);
    out.into_iter().map(|(filling, _)| filling).collect()
}
because!(holding, ChainOfRules, "every filling under which the given conditions hold together with the facts of a story from a given filling, as an answer of the library reads them: for a diagnosis that asks how far the conditions of an answer hold");

pub(super) fn none_fits(absent: &[Condition], facts: &Facts, filling: &Filling) -> bool {
    absent.iter().all(|condition| match condition {
        Condition::Fact { pattern, .. } => {
            let mut held = held_filling(filling);
            let mut found = Vec::new();
            facts.candidates(pattern, filling).iter().map(|place| *place as usize).all(|place| {
                fit_from(pattern, facts.fact(place), &mut held, &mut found);
                found.is_empty()
            })
        }
        _ => condition_fits(condition, facts, filling, STORY_END).is_empty(),
    })
}
because!(none_fits, ChainOfRules, "whether no condition that must not hold holds under a filling: no fact fits a sentence that must be absent, and a comparison, a step of arithmetic or a count after 'not' does not hold; a place that the filling leaves open may take any words, so a sentence with an open place is absent only when no fact fits it at all");

fn steady(rule: &ToldRule) -> bool {
    rule.absent.is_empty() && rule.conditions.iter().all(|condition| matches!(condition, Condition::Fact { when: Moment::Any, .. } | Condition::Compare(..) | Condition::Alike { .. } | Condition::Reckon(..) | Condition::Backward { .. } | Condition::Holds { .. } | Condition::HoldsKind { .. } | Condition::After { .. } | Condition::Letter { .. } | Condition::Told(..)))
}
because!(steady, ChainOfRules, "whether a rule only adds as facts grow: it has no condition that must be absent, none that only one fact chosen by its place in the story may hold and no count, so a new fact never takes back its outcome");

fn last(rule: &ToldRule) -> bool {
    !rule.absent.is_empty() || rule.conditions.iter().any(|condition| matches!(condition, Condition::Count { .. } | Condition::Follows { .. } | Condition::Proves { .. }))
}
because!(last, ChainOfRules, "whether a rule of the later layer reads what is absent or how many facts there are: such a rule must read the facts only when every other rule adds nothing more, as a stratum after the rules that read facts by their place in the story, since an outcome that it makes from facts that are not all there yet would stay");

pub(super) fn adds(rule: &ToldRule) -> bool {
    rule.kind == LineKind::Adds && rule.name.is_empty()
}
because!(adds, ChainOfRules, "whether a line of rules adds its outcome as a fact, as a rule does, and is not a line that writes a fact again or that reads told rules, nor a rule that the story names, whose outcome may be defeated by a preferred rule");

pub struct DefeasibleRules;
source!(DefeasibleRules, "rules with priorities, as in defeasible logic (Nute) and prioritized logic programs: the rules that a story names hold unless a rule that the story prefers to them gives the opposite outcome; they are read after every other rule, the most preferred first, so that an outcome is never added once a preferred rule has given its opposite, and a defeated outcome never feeds a later rule");

const PREFERRED: &str = "A is preferred over B";
because!(PREFERRED, DefeasibleRules, "the sentence by which a story prefers one of its named rules to another");

fn preferences(facts: &Facts) -> Vec<(Vec<WordId>, Vec<WordId>)> {
    fact_fits(&crate::tools::explain::text::pattern(PREFERRED), Moment::Any, facts, &empty_filling(), STORY_END).into_iter().filter_map(|(filling, _)| Some((filling.get(0)?.to_vec(), filling.get(1)?.to_vec()))).collect()
}
because!(preferences, DefeasibleRules, "each told preference as the name of the preferred rule and the name of the other");

fn preference_ranks(rules: &[ToldRule], preferred: &[(Vec<WordId>, Vec<WordId>)]) -> Vec<usize> {
    let mut rank = vec![usize::default(); rules.len()];
    for _ in 0..=rules.len() {
        let mut moved = false;
        for (at, rule) in rules.iter().enumerate().filter(|(_, rule)| !rule.name.is_empty()) {
            for (better, _) in preferred.iter().filter(|(_, worse)| *worse == rule.name) {
                if let Some(over) = rules.iter().position(|other| other.name == *better)
                    && rank[at] <= rank[over]
                {
                    rank[at] = rank[over] + 1;
                    moved = true;
                }
            }
        }
        if !moved {
            break;
        }
    }
    rank
}
because!(preference_ranks, DefeasibleRules, "the rank of each named rule: one more than that of each rule preferred to it, so that the most preferred are read first; a cycle of preferences stops growing after as many rounds as there are rules");

fn defeasible_outcomes(rules: &[ToldRule], facts: &Facts, made_by: &mut HashMap<Vec<WordId>, Vec<Vec<WordId>>>) -> Vec<(Vec<WordId>, Time)> {
    let preferred = preferences(facts);
    let ranks = preference_ranks(rules, &preferred);
    for rank in 0..=ranks.iter().copied().max().unwrap_or_default() {
        let mut outcomes = Vec::new();
        for (rule, _) in rules.iter().zip(&ranks).filter(|(rule, at)| **at == rank && !rule.name.is_empty() && rule.kind == LineKind::Adds) {
            let mut found = Vec::new();
            whole_outcomes(rule, facts, &mut found);
            for (outcome, time) in found {
                let defeated = made_by.get(&opposite(&outcome)).is_some_and(|makers| makers.iter().any(|maker| preferred.iter().any(|(better, worse)| better == maker && *worse == rule.name)));
                if !defeated {
                    made_by.entry(outcome.clone()).or_default().push(rule.name.clone());
                    outcomes.push((outcome, time));
                }
            }
        }
        if !outcomes.is_empty() {
            return outcomes;
        }
    }
    Vec::new()
}
because!(defeasible_outcomes, DefeasibleRules, "the outcomes of the named rules of a story, rank by rank, the most preferred first: an outcome whose opposite a rule preferred to its own has made is defeated and left out; the first rank that adds an outcome stops the step, so that the chain goes on from it before less preferred rules are read"); 

pub struct DefaultRules;
source!(DefaultRules, "default reasoning as Reiter named it: a rule that holds usually adds its outcome only where nothing that the story knows rules it out, in one stratum after every exact rule, so that a told fact beats a default (John is male by default, and female when the story says that John is a mother), and two defaults that give rival values to one thing give neither; a fact that a default made does not make another default fire, as the stratum is read once (a known limit)");

fn default_outcomes(rules: &[ToldRule], facts: &Facts) -> Vec<(Vec<WordId>, Time)> {
    let mut found: Vec<(Vec<WordId>, Time, &Pattern)> = Vec::new();
    for rule in rules.iter().filter(|rule| rule.kind == LineKind::AddsByDefault) {
        let mut outcomes = Vec::new();
        whole_outcomes(rule, facts, &mut outcomes);
        for (outcome, time) in outcomes {
            if !found.iter().any(|(known, _, _)| *known == outcome) {
                found.push((outcome, time, &rule.outcome));
            }
        }
    }
    if found.is_empty() {
        return Vec::new();
    }
    let rivals = known_rivals(facts);
    let fixed = |pattern: &Pattern| -> HashSet<WordId> { pattern.iter().filter_map(|piece| if let LogicPiece::Word(word) = piece { Some(*word) } else { None }).collect() };
    let rival = |one: &[WordId], other: &[WordId], values: &HashSet<WordId>| {
        let differ: Vec<usize> = (0..one.len().min(other.len())).filter(|at| one[*at] != other[*at]).collect();
        one.len() == other.len() && matches!(differ.as_slice(), [at] if values.contains(&one[*at]) && rivals.get(&one[*at]).is_some_and(|words| words.contains(&other[*at])))
    };
    found
        .iter()
        .filter(|(outcome, _, pattern)| {
            let values = fixed(pattern);
            !overruled(outcome, facts, (&values, &rivals)) && !found.iter().any(|(other, _, _)| rival(outcome, other, &values))
        })
        .map(|(outcome, time, _)| (outcome.clone(), *time))
        .collect()
}
because!(default_outcomes, DefaultRules, "the outcomes of the rules that hold usually, read once after every exact rule: each outcome of a default, left out when a fact of the story or of what the topic knows holds the same sentence with a rival in place of one of the default's own words (john is female beside john is male), or when another default gives such a rival");

fn overruled(outcome: &[WordId], facts: &Facts, (values, rivals): (&HashSet<WordId>, &HashMap<WordId, HashSet<WordId>>)) -> bool {
    overruling_fact(outcome, facts, (values, rivals)).is_some()
}
because!(overruled, DefaultRules, "whether the facts hold the outcome of a default with a rival in place of one of the default's own words, the fact that rules the default out");

pub(super) fn overruling_fact(outcome: &[WordId], facts: &Facts, (values, rivals): (&HashSet<WordId>, &HashMap<WordId, HashSet<WordId>>)) -> Option<Vec<WordId>> {
    (0..outcome.len()).filter(|at| values.contains(&outcome[*at])).find_map(|at| {
        let others = rivals.get(&outcome[at])?;
        let pattern: Pattern = outcome.iter().enumerate().map(|(place, word)| if place == at { LogicPiece::OpenWithEnd(usize::default(), String::new()) } else { LogicPiece::Word(*word) }).collect();
        fact_fits(&pattern, Moment::Any, facts, &empty_filling(), STORY_END).into_iter().find_map(|(filling, _)| match filling.get(usize::default()) {
            Some([word]) if others.contains(word) => fill(&pattern, &filling),
            _ => None,
        })
    })
}
because!(overruling_fact, DefaultRules, "the fact that rules out the outcome of a default: the same sentence with a rival in place of one of the default's own words, as the facts hold it, or nothing");

pub(super) fn known_rivals(facts: &Facts) -> HashMap<WordId, HashSet<WordId>> {
    let mut by_context: HashMap<(Option<WordId>, Option<WordId>), HashSet<WordId>> = HashMap::new();
    for place in 0..facts.first {
        let fact = facts.fact(place);
        for at in 0..fact.len() {
            let context = (at.checked_sub(1).map(|before| fact[before]), fact.get(at + 1).copied());
            by_context.entry(context).or_default().insert(fact[at]);
        }
    }
    let mut out: HashMap<WordId, HashSet<WordId>> = HashMap::new();
    for words in by_context.values().filter(|words| words.len() > 1) {
        for word in words {
            out.entry(*word).or_default().extend(words.iter().filter(|other| *other != word));
        }
    }
    out
}
because!(known_rivals, DefaultRules, "for each word, the words that the facts known before the story give the same spot between the same neighbours (male and female in a male child and a female child): the rivals of a value, of which a thing takes one");

pub(super) fn apart(rule: &ToldRule, filling: &Filling) -> bool {
    if !rule.learned || rule.kind != LineKind::Adds || rule.conditions.len() + rule.absent.len() <= 1 || !rule.conditions.iter().chain(&rule.absent).all(|condition| matches!(condition, Condition::Fact { .. })) {
        return true;
    }
    let mut seen: [&[WordId]; OPEN_PLACES] = [&[]; OPEN_PLACES];
    let mut count = 0;
    for place in 0..OPEN_PLACES {
        if let Some(words) = filling.get(place) {
            if seen[..count].contains(&words) {
                return false;
            }
            seen[count] = words;
            count += 1;
        }
    }
    true
}
because!(apart, ChainOfRules, "whether the places of a rule hold different words where the rule asks it: a rule that a learner made, that adds its outcome from facts alone (its conditions and the conditions that must be absent), stands for different things in its different places (a parent of two children makes them siblings only when they are two, and no child its own sibling), the usual bias of rule learning; only the open places count, not the slot that the reading of a state opens for its last word, so that a filled slot never refuses a right filling; a rule of the library, a line that writes or reads, a rule that holds usually, one with a comparison, a step of arithmetic or a count, and a rule of one condition may fill two places with the same words: a rule of one condition says again what its one sentence says, so a sentence that names one thing twice (either in the kitchen or the kitchen) gives a fact that names it twice, while the bias of different things is for rules that join two facts or more. The known limit: two places of a learned rule that truly hold the same thing (two people in rooms, with one room) are blocked");

fn whole_outcomes(rule: &ToldRule, facts: &Facts, outcomes: &mut Vec<(Vec<WordId>, Time)>) {
    let mut held: Vec<&Condition> = rule.conditions.iter().collect();
    let mut joined = Vec::new();
    join(&mut held, facts, &empty_filling(), (0, STORY_END, false), &mut joined);
    for (whole, time) in joined {
        if none_fits(&rule.absent, facts, &whole)
            && apart(rule, &whole)
            && let Some(outcome) = fill(&rule.outcome, &whole)
            && facts.newer(&outcome, time)
        {
            outcomes.push((outcome, time));
        }
    }
}
because!(whole_outcomes, ChainOfRules, "the outcomes of one rule read on all facts at once: every filling of its conditions under which no condition that must not hold holds, and whose outcome adds to the facts");

pub struct NegationStrata;
source!(NegationStrata, "stratified negation (Apt, Blair and Walker; the perfect model of Przymusinski): a rule that reads what is absent, or how many facts there are, must wait for every rule of its kind that can make such a fact, also through rules that read the outcome of that rule, so that it reads the facts only once they can no longer grow; the rules that wait for each other in a cycle have no such order, and they are read together, as before");

fn can_fit(outcome: &Pattern, sentence: &Pattern) -> bool {
    let (count, width) = (outcome.len(), sentence.len());
    let row = width + 1;
    let mut reached = vec![false; (count + 1) * row];
    reached[0] = true;
    for at in 0..=count {
        for other in 0..=width {
            if !reached[at * row + other] || at == count || other == width {
                continue;
            }
            let (one, two) = (&outcome[at], &sentence[other]);
            if !pieces_meet(one, two) {
                continue;
            }
            reached[(at + 1) * row + other + 1] = true;
            if matches_open(one) {
                reached[at * row + other + 1] = true;
            }
            if matches_open(two) {
                reached[(at + 1) * row + other] = true;
            }
        }
    }
    reached[count * row + width]
}
because!(can_fit, NegationStrata, "whether some fact that a sentence with open places makes could fit another such sentence: the two are laid side by side, a word must meet the same word or an open place, a place with an ending a word with that ending, and an open place of one or more words may take more pieces of the other; the words that two places take are not compared, so a fit may be found where none can come, and never missed");

fn pieces_meet(one: &LogicPiece, two: &LogicPiece) -> bool {
    match (one, two) {
        (LogicPiece::Word(word), LogicPiece::Word(other)) => word == other,
        (LogicPiece::Word(word), LogicPiece::OpenWithEnd(_, ending)) | (LogicPiece::OpenWithEnd(_, ending), LogicPiece::Word(word)) => word_text(*word).len() > ending.len() && word_text(*word).ends_with(ending.as_str()),
        _ => true,
    }
}
because!(pieces_meet, NegationStrata, "whether two pieces of two sentences may stand for the same word: two words that are the same, a word and an open place, a word with the ending of an open place with an ending, or two open places");

fn read_sentences(rule: &ToldRule) -> Vec<&Pattern> {
    rule.conditions
        .iter()
        .filter_map(|condition| match condition {
            Condition::Fact { pattern, .. } | Condition::Count { pattern, .. } | Condition::Follows { said: pattern, .. } => Some(pattern),
            Condition::Compare(..) | Condition::Alike { .. } | Condition::Reckon(..) | Condition::Holds { .. } | Condition::HoldsKind { .. } | Condition::Backward { .. } | Condition::After { .. } | Condition::Proves { .. } | Condition::Letter { .. } | Condition::Told(..) => None,
        })
        .collect()
}
because!(read_sentences, NegationStrata, "the sentences whose facts a rule reads to hold: those of its conditions that must be facts, those of its counts, and the words of a condition 'it follows that', which a fact may say");

fn waited_sentences(rule: &ToldRule) -> Vec<&Pattern> {
    let absent = rule.absent.iter().filter_map(|condition| match condition {
        Condition::Fact { pattern, .. } | Condition::Count { pattern, .. } => Some(pattern),
        Condition::Compare(..) | Condition::Alike { .. } | Condition::Reckon(..) | Condition::Follows { .. } | Condition::Holds { .. } | Condition::HoldsKind { .. } | Condition::Backward { .. } | Condition::After { .. } | Condition::Proves { .. } | Condition::Letter { .. } | Condition::Told(..) => None,
    });
    let counted = rule.conditions.iter().filter_map(|condition| match condition {
        Condition::Count { pattern, .. } => Some(pattern),
        Condition::Follows { .. } => Some(announced_sentence()),
        Condition::Proves { .. } => Some(any_sentence()),
        _ => None,
    });
    absent.chain(counted).collect()
}
because!(waited_sentences, NegationStrata, "the sentences of a rule whose facts must be complete before it is read: those that must be absent and those that it counts");

pub fn not_strata(rules: &[ToldRule]) -> (Vec<usize>, bool) {
    let closing: Vec<usize> = (0..rules.len()).filter(|at| adds(&rules[*at]) && last(&rules[*at]) && !rules[*at].conditions.is_empty()).collect();
    let mut strata = vec![usize::default(); rules.len()];
    if closing.len() <= 1 {
        return (strata, false);
    }
    let reached: Vec<Vec<&Pattern>> = closing.iter().map(|maker| reaches(rules, *maker)).collect();
    let waits: Vec<Vec<usize>> = closing
        .iter()
        .map(|reader| {
            let waited = waited_sentences(&rules[*reader]);
            closing.iter().zip(&reached).filter(|(_, made)| made.iter().any(|made| waited.iter().any(|sentence| can_fit(made, sentence)))).map(|(maker, _)| *maker).collect()
        })
        .collect();
    for _ in 0..=closing.len() {
        let mut moved = false;
        for (reader, makers) in closing.iter().zip(&waits) {
            for maker in makers {
                if strata[*reader] <= strata[*maker] {
                    strata[*reader] = strata[*maker] + 1;
                    moved = true;
                }
            }
        }
        if !moved {
            return (strata, false);
        }
    }
    (vec![usize::default(); rules.len()], true)
}
because!(not_strata, NegationStrata, "the stratum of each rule that reads what is absent or how many: one more than that of each such rule whose outcome, or a fact made from it by the other rules, can fit a sentence that it waits for; when such rules wait for each other in a cycle, every rule has the first stratum and the cycle is said, as before the strata all such rules were read together");

pub struct RuleCosts;
source!(RuleCosts, "a profile of the rules of the library by the time of their readings and the facts that each reads (as a profiler of a program counts the work of each function), so that the rules that cost the most are where a text writer gains most by a better order of conditions");

pub static COSTING: AtomicBool = AtomicBool::new(false);
because!(COSTING, RuleCosts, "whether the engine counts the facts that each rule reads; off unless a command asks for the profile, so that a run pays nothing for it");

static COSTS: OnceLock<Mutex<HashMap<String, RuleCost>>> = OnceLock::new();
because!(COSTS, RuleCosts, "the count of each rule of the run, by the rule as the library writes it: a rule learned in a search is let go, and a later rule may take its place in memory");

pub struct RuleCost {
    pub text: String,
    pub reads: usize,
    pub readings: usize,
    pub seconds: f64,
}
because!(RuleCost, RuleCosts, "what a rule cost in a run: the rule as the library writes it, the facts that its conditions read, how often it was read, and the seconds of its readings over all threads");

fn costed<T>(rule: &ToldRule, facts: &Facts, run: impl FnOnce() -> T) -> T {
    if !COSTING.load(Ordering::Relaxed) {
        return run();
    }
    let before = facts.work.load(Ordering::Relaxed);
    let started = std::time::Instant::now();
    let value = run();
    let seconds = started.elapsed().as_secs_f64();
    let reads = facts.work.load(Ordering::Relaxed).saturating_sub(before);
    if let Ok(mut costs) = COSTS.get_or_init(Mutex::default).lock() {
        let text = rule_text(rule);
        let cost = costs.entry(text.clone()).or_insert_with(|| RuleCost { text, reads: usize::default(), readings: usize::default(), seconds: f64::default() });
        cost.reads += reads;
        cost.readings += 1;
        cost.seconds += seconds;
    }
    value
}
because!(costed, RuleCosts, "a reading of one rule, with the facts that its conditions read added to its count when the profile is on");

pub fn rule_costs() -> Vec<RuleCost> {
    let mut out: Vec<RuleCost> = COSTS.get_or_init(Mutex::default).lock().map(|mut costs| costs.drain().map(|(_, cost)| cost).collect()).unwrap_or_default();
    out.sort_by(|one, other| other.seconds.total_cmp(&one.seconds).then_with(|| one.text.cmp(&other.text)));
    out
}
because!(rule_costs, RuleCosts, "the counts of the run, the rules that took the most time first, rules of the same time in the order of their texts");

const COST_IF: &str = "If";
because!(COST_IF, RuleCosts, "the word before the conditions of a rule, as the library writes it");

const COST_AND: &str = " and ";
because!(COST_AND, RuleCosts, "the words between two conditions of a rule, as the library writes them");

const COST_THEN: &str = " then ";
because!(COST_THEN, RuleCosts, "the words between the conditions of a rule and its outcome");

const COST_NOT: &str = "not ";
because!(COST_NOT, RuleCosts, "the word before a condition that must not hold");

pub(super) fn rule_text(rule: &ToldRule) -> String {
    let held = rule.conditions.iter().map(|condition| condition_text(condition, &piece_text)).chain(rule.absent.iter().map(|condition| format!("{COST_NOT}{}", condition_text(condition, &piece_text))));
    let usually = if rule.kind == LineKind::AddsByDefault { USUALLY } else { "" };
    format!("{COST_IF} {}{COST_THEN}{usually}{}", held.collect::<Vec<String>>().join(COST_AND), pattern_text(&rule.outcome))
}
because!(rule_text, RuleCosts, "a rule as a line of the library, its sentences with their letters, for the profile");

fn piece_text(logic_piece: &LogicPiece) -> String {
    match logic_piece {
        LogicPiece::Word(word) => word_text(*word).to_string(),
        LogicPiece::Open(place) => place_letter(*place),
        LogicPiece::OpenWithEnd(place, ending) => format!("{}{ending}", place_letter(*place)),
    }
}
because!(piece_text, NegationStrata, "a piece of a sentence as the library writes it: its word, or the letter of its place with its ending");

pub(super) fn pattern_text(pattern: &Pattern) -> String {
    pattern.iter().map(piece_text).collect::<Vec<String>>().join(" ")
}
because!(pattern_text, NegationStrata, "a sentence with open places as the library writes it, each place by its letter, to say which sentences make a cycle");

pub fn not_cycle_causes(rules: &[ToldRule]) -> Vec<String> {
    let closing: Vec<usize> = (0..rules.len()).filter(|at| adds(&rules[*at]) && last(&rules[*at]) && !rules[*at].conditions.is_empty()).collect();
    let mut edges: Vec<(usize, usize, String, String)> = Vec::new();
    for (reader, reader_rule) in closing.iter().map(|at| (*at, &rules[*at])) {
        let waited = waited_sentences(reader_rule);
        for maker in closing.iter().copied() {
            let made = reaches(rules, maker);
            if let Some((sentence, read)) = made.iter().find_map(|sentence| waited.iter().find(|read| can_fit(sentence, read)).map(|read| (*sentence, *read))) {
                edges.push((maker, reader, pattern_text(sentence), pattern_text(read)));
            }
        }
    }
    let leads_to = |from: usize, to: usize| {
        let mut seen = vec![from];
        let mut at = 0;
        while let Some(now) = seen.get(at).copied() {
            at += 1;
            for (maker, reader, _, _) in &edges {
                if *maker == now && !seen.contains(reader) {
                    seen.push(*reader);
                }
            }
        }
        edges.iter().any(|(maker, reader, _, _)| *reader == to && seen.contains(maker))
    };
    let mut causes: Vec<(String, Vec<String>)> = Vec::new();
    for (maker, reader, made, read) in edges.iter().filter(|(maker, reader, _, _)| leads_to(*reader, *maker) || maker == reader) {
        let cause = format!("'{}' can make '{made}'", pattern_text(&rules[*maker].outcome));
        let waiting = format!("'{read}' (the rule that makes '{}')", pattern_text(&rules[*reader].outcome));
        match causes.iter_mut().find(|(known, _)| *known == cause) {
            Some((_, readers)) if !readers.contains(&waiting) => readers.push(waiting),
            Some(_) => {}
            None => causes.push((cause, vec![waiting])),
        }
    }
    causes.into_iter().map(|(cause, readers)| format!("{cause}, which can fit {}", readers.join("; "))).collect()
}
because!(not_cycle_causes, NegationStrata, "the waits that make the cycles of the rules that read what is absent or how many: for each outcome of a rule of a cycle and the sentence that it can make, the sentences that the rules of the cycle that wait for it read as absent or count, so that a writer of the library can narrow the outcome that is too wide");

fn reaches(rules: &[ToldRule], maker: usize) -> Vec<&Pattern> {
    let mut made: Vec<&Pattern> = vec![&rules[maker].outcome];
    let mut taken = vec![false; rules.len()];
    let mut at = 0;
    while let Some(sentence) = made.get(at).copied() {
        at += 1;
        for (place, rule) in rules.iter().enumerate() {
            if !taken[place] && adds(rule) && !last(rule) && read_sentences(rule).iter().any(|read| can_fit(sentence, read)) {
                taken[place] = true;
                made.push(&rule.outcome);
            }
        }
    }
    made
}
because!(reaches, NegationStrata, "the sentences whose facts a rule can make: its outcome, and the outcomes of the rules of the other layers that can read a fact made so, and so on");

type StrataKey = u64;
because!(StrataKey, NegationStrata, "a hash of the rules of a topic, by which the strata of its rules are kept");

static STRATA: OnceLock<Mutex<HashMap<StrataKey, Arc<Vec<usize>>>>> = OnceLock::new();
because!(STRATA, NegationStrata, "the strata of the rules of each topic, found once for the run, since a story without told rules reads only the rules of its topic");

fn strata_of(rules: &[ToldRule], told_rules: bool, key: u64) -> Arc<Vec<usize>> {
    if told_rules {
        return Arc::new(not_strata(rules).0);
    }
    let store = STRATA.get_or_init(Mutex::default);
    if let Some(found) = store.lock().ok().and_then(|kept| kept.get(&key).cloned()) {
        return found;
    }
    let found = Arc::new(not_strata(rules).0);
    if let Ok(mut kept) = store.lock() {
        kept.insert(key, Arc::clone(&found));
    }
    found
}
because!(strata_of, NegationStrata, "the strata of the rules of a chain: kept for the rules of a topic by the hash that the caller keeps for them, and found again for a story that adds rules of its own");

fn closing_outcomes(rules: &[ToldRule], strata: &[usize], stratum: usize, facts: &Facts, changes: &[usize], seen: &mut Vec<Option<usize>>) -> Vec<(Vec<WordId>, Time)> {
    let mut outcomes = Vec::new();
    seen.resize(rules.len(), None);
    for (number, rule) in rules.iter().enumerate().filter(|(number, rule)| strata.get(*number) == Some(&stratum) && adds(rule) && last(rule) && !rule.conditions.is_empty()) {
        let from = seen[number];
        seen[number] = Some(changes.len());
        match from {
            Some(from) if seeds_alone(rule) => {
                let mut fresh: Vec<usize> = changes[from..].to_vec();
                fresh.sort_unstable();
                fresh.dedup();
                let seeds: Vec<(usize, Time)> = fresh.into_iter().map(|place| (place, facts.latest(place))).collect();
                costed(rule, facts, || seeded_outcomes(rule, facts, &seeds, &mut outcomes));
            }
            Some(from) => {
                let sentences: Vec<&Pattern> = read_sentences(rule).into_iter().chain(waited_sentences(rule)).collect();
                if changes[from..].iter().any(|place| sentences.iter().any(|sentence| holds_words(sentence, facts.fact(*place)))) {
                    costed(rule, facts, || whole_outcomes(rule, facts, &mut outcomes));
                }
            }
            None => costed(rule, facts, || whole_outcomes(rule, facts, &mut outcomes)),
        }
    }
    outcomes
}
because!(closing_outcomes, NegationStrata, "the outcomes of the rules that read what is absent or how many, of one stratum: the first time read on all facts at once; afterwards, a rule whose conditions only read facts at any time (with sentences that must not hold) is read only on the facts that came or moved since, as a new outcome must rest on one of them, since a fact that came can only make a sentence that must be absent hold (semi-naive); any other rule is read again on all facts, and only when a fact that came could fit one of its sentences");

fn seeds_alone(rule: &ToldRule) -> bool {
    rule.conditions.iter().all(|condition| matches!(condition, Condition::Fact { when: Moment::Any, .. } | Condition::Compare(..) | Condition::Alike { .. } | Condition::Reckon(..) | Condition::Backward { .. } | Condition::Holds { .. } | Condition::HoldsKind { .. } | Condition::After { .. } | Condition::Letter { .. } | Condition::Told(..)))
}
because!(seeds_alone, NegationStrata, "whether every new outcome of a rule must rest on a fact that came since its last reading: true when its conditions read facts at any time and work out values, and no count, statement of worlds or proof reads all facts at once");

fn places_from(facts: &Facts, from: Time) -> Vec<(Time, usize)> {
    let mut timed: Vec<(Time, usize)> = Vec::new();
    let mut put = |place: usize| timed.extend(facts.times_of(place).iter().filter(|time| **time >= from).map(|time| (*time, place)));
    if from == Time::default() {
        (0..facts.len()).for_each(&mut put);
    } else {
        facts.moved.keys().copied().for_each(&mut put);
        (facts.first..facts.len()).for_each(&mut put);
    }
    timed.sort();
    timed.dedup();
    timed
}
because!(places_from, ChainOfRules, "the places of the facts that hold at a place in the story from the given one on, with that place, in the order of the story: the facts under the story only when the reading starts at the knowledge before the story, where they all hold, and else only the facts of the story and those that it moved to a later place");

fn word_bit(word: WordId) -> u64 {
    1u64 << (word % u64::BITS)
}
because!(word_bit, ChainOfRules, "the bit of a word in the mask of the words of a fact, by its number, so that a mask tells at once that a fact lacks a word");

fn words_mask(words: &[WordId]) -> u64 {
    words.iter().fold(0, |mask, word| mask | word_bit(*word))
}
because!(words_mask, ChainOfRules, "the mask of the words of a fact: a word that a fact holds always has its bit set, so a fact whose mask lacks a bit of a pattern's fixed words cannot hold them, and holds_words is not tried");

fn condition_mask(condition: &Condition) -> u64 {
    let Condition::Fact { pattern, .. } = condition else { return 0 };
    pattern_mask(pattern)
}

fn pattern_mask(pattern: &Pattern) -> u64 {
    pattern.iter().fold(0, |mask, logic_piece| match logic_piece {
        LogicPiece::Word(word) => mask | word_bit(*word),
        _ => mask,
    })
}
because!(pattern_mask, ChainOfRules, "the mask of the fixed words of a sentence with open places");

pub(super) fn reply_masks(form: &AskedForm) -> Option<Vec<u64>> {
    if form.replies.iter().any(|reply| reply.absent.len() > 1) {
        return None;
    }
    let mut out = Vec::new();
    for condition in form.replies.iter().flat_map(|reply| reply.conditions.iter().chain(&reply.absent)) {
        let Condition::Fact { pattern, .. } = condition else { return None };
        let mask = pattern_mask(pattern);
        if mask == 0 {
            return None;
        }
        out.push(mask);
    }
    Some(out)
}
pub(super) fn reply_patterns(form: &AskedForm) -> impl Iterator<Item = &Pattern> {
    form.replies.iter().flat_map(|reply| reply.conditions.iter().chain(&reply.absent)).filter_map(|condition| match condition {
        Condition::Fact { pattern, .. } => Some(pattern),
        _ => None,
    })
}
because!(reply_patterns, ChainOfRules, "the sentences that the answers of a question form read by fact conditions, that must hold or must not hold, in the order of reply_masks, so that a fact can be tested against each of them by its words");

pub(super) fn reads_none<'p>(mut patterns: impl Iterator<Item = (u64, &'p Pattern)>, (mask, fact): (u64, &[WordId])) -> bool {
    patterns.all(|(pattern_mask, pattern)| pattern_mask & !mask != 0 || !holds_words(pattern, fact))
}
because!(reads_none, ChainOfRules, "whether a fact holds the fixed words of none of the given sentences, each tried by its mask first and by its words only where the mask leaves it open: the answer is that of the words alone, as a fact that lacks a bit lacks a word, so it does not depend on the numbers that the words got in a run, which differ from run to run when lanes meet new words in another order");

because!(reply_masks, ChainOfRules, "for a question form whose answers read facts by fact conditions alone, the masks of the fixed words of every sentence that they read (that must hold or must not hold), or nothing when a condition of another kind or a sentence with no fixed word is among them, or an answer with more than one condition that must not hold, as each such condition reads its candidates once, so that one fact read could lead to more reads than the bound of the reads of a reply counts; so that a fact whose mask misses a bit of each of them cannot change any of its answers, and each list of candidates that a reply reads holds at most one more fact for each differing fact");

fn reply_starts(form: &AskedForm, words: &[WordId]) -> usize {
    let mut asked = Vec::new();
    fit_into(&form.question, words, &empty_filling(), &mut asked);
    asked.len() * form.replies.len()
}
because!(reply_starts, ChainOfRules, "the joins that a reply to a question starts before any fact is read: one for each answer under each filling of the question, as reply tries them; every other read of candidates follows a fact read before, so the reads of candidates of a reply are at most these starts and twice its reads");

const READ_LEADS: usize = 2;
because!(READ_LEADS, ChainOfRules, "the reads of candidates that one fact read by a reply can lead to: the next condition of its join, or the one condition that must be absent once its filling is whole (the masks of the answers keep no form with an answer of more such conditions)");

pub(super) fn reply_read_bound(form: &AskedForm, words: &[WordId], reads: usize) -> usize {
    reply_starts(form, words) + reads * READ_LEADS
}
because!(reply_read_bound, ChainOfRules, "the most reads of candidates that a reply to a question made, from the facts that it read: its starts, and what each fact read can lead to");
because!(condition_mask, ChainOfRules, "the mask of the fixed words of a condition that reads a fact, which every fact that holds them covers; none for a condition of another kind");

fn anchor_clause(rule: &ToldRule) -> Option<usize> {
    let first = rule.conditions.iter().position(anchored)?;
    let before = first.checked_sub(1)?;
    matches!(rule.conditions[before], Condition::Fact { when: Moment::Any, .. }).then_some(before)
}
because!(anchor_clause, ChainOfRules, "the condition whose time a rule of the later layer points to with its first condition 'at that time' or 'before that time', when it reads any fact: the outcome takes the place of the fact that it reads, so the rule is read at that place in the order of the story");

fn ordered_added(rules: &[ToldRule], writes: &[ToldRule], facts: &mut Facts, (from, longest, fills): (&mut Time, usize, Option<&Cell<usize>>)) -> Vec<(usize, Time)> {
    let timed: Vec<&ToldRule> = rules.iter().filter(|rule| !steady(rule) && adds(rule) && !last(rule) && !rule.conditions.is_empty()).collect();
    let seeded: Vec<(&ToldRule, usize, u64)> = timed.iter().filter_map(|rule| anchor_clause(rule).map(|at| (*rule, at, condition_mask(&rule.conditions[at])))).collect();
    let placed = places_from(facts, *from);
    let mut start = usize::default();
    while start < placed.len() {
        let time = placed[start].0;
        let end = start + placed[start..].iter().take_while(|(other, _)| *other == time).count();
        let mut outcomes = Vec::new();
        let group: Vec<(&[WordId], u64)> = placed[start..end].iter().map(|(_, place)| {
            let fact = facts.fact(*place);
            (fact, words_mask(fact))
        }).collect();
        for (rule, at, needed) in &seeded {
            let Condition::Fact { pattern, .. } = &rule.conditions[*at] else { continue };
            let mut before: Vec<&Condition> = rule.conditions[..*at].iter().collect();
            let mut after: Vec<&Condition> = rule.conditions[*at + 1..].iter().collect();
            costed(rule, facts, || for (fact, held) in &group {
                if needed & !held != 0 || !holds_words(pattern, fact) {
                    continue;
                }
                let mut found = Vec::new();
                fit_into(pattern, fact, &empty_filling(), &mut found);
                if !took_fillings(fills, found.len()) {
                    break;
                }
                for filling in found {
                    let mut opened = Vec::new();
                    join(&mut before, facts, &filling, (0, STORY_END, false), &mut opened);
                    for (filled, earlier) in opened {
                        let mut joined = Vec::new();
                        join(&mut after, facts, &filled, (earlier.max(time), time, false), &mut joined);
                        for (whole, outcome_time) in joined {
                            if none_fits(&rule.absent, facts, &whole)
                                && apart(rule, &whole)
                                && let Some(outcome) = fill(&rule.outcome, &whole)
                                && facts.newer(&outcome, outcome_time)
                            {
                                outcomes.push((outcome, outcome_time));
                            }
                        }
                    }
                }
            });
        }
        let outcomes = in_place(writes, facts, outcomes);
        let added = added_within(facts, longest, outcomes);
        if !added.is_empty() {
            *from = time;
            return added;
        }
        start = end;
    }
    *from = STORY_END;
    let mut outcomes = Vec::new();
    for rule in timed.iter().filter(|rule| anchor_clause(rule).is_none()) {
        whole_outcomes(rule, facts, &mut outcomes);
    }
    let outcomes = in_place(writes, facts, outcomes);
    added_within(facts, longest, outcomes)
}
because!(ordered_added, ChainOfRules, "the facts that the rules of the later layer that read facts by their place in the story add, read in the order of the story: place by place from the first place not yet settled, each rule whose time comes from a condition that reads any fact read with each fact at that place in that condition, at that place alone, the conditions before it and after it as join reads them, so that a rule that reads what held before or at the time of a clause reads every fact of the earlier places, also those that such rules make there; at the first place where they add facts, those facts, for the steady rules to read before the next place; a rule whose time comes from no such condition read on all facts once every place is settled");

struct AddedWords<'a> {
    added: &'a [(usize, Time)],
    holding: MixMap<WordId, Vec<usize>>,
    mask: u64,
}
because!(AddedWords, ChainOfRules, "the facts that a round added, and for each word the places in that list of the added facts that hold it, in their order, so that a condition is tried only on the added facts that hold its rarest fixed word; of reads holds the facts in its order");

impl<'a> AddedWords<'a> {
    fn of(added: &'a [(usize, Time)], facts: &Facts) -> AddedWords<'a> {
        let mut holding: MixMap<WordId, Vec<usize>> = MixMap::default();
        let mut mask = 0;
        for (at, (place, _)) in added.iter().enumerate() {
            for word in facts.fact(*place) {
                mask |= word_bit(*word);
                let list = holding.entry(*word).or_default();
                if list.last() != Some(&at) {
                    list.push(at);
                }
            }
        }
        AddedWords { added, holding, mask }
    }

    fn fitting<'b>(&'b self, pattern: &Pattern) -> Box<dyn Iterator<Item = &'b (usize, Time)> + 'b> {
        let rarest = pattern.iter().filter_map(|logic_piece| match logic_piece {
            LogicPiece::Word(word) => Some(self.holding.get(word).map_or(&[][..], Vec::as_slice)),
            _ => None,
        }).min_by_key(|list| list.len());
        match rarest {
            Some(list) => Box::new(list.iter().map(|at| &self.added[*at])),
            None => Box::new(self.added.iter()),
        }
    }
}

fn seeded_outcomes(rule: &ToldRule, facts: &Facts, added: &[(usize, Time)], outcomes: &mut Vec<(Vec<WordId>, Time)>) {
    seeded_by_words(rule, facts, &AddedWords::of(added, facts), outcomes, None);
}
because!(seeded_outcomes, ChainOfRules, "the outcomes of one rule that rest on the given facts, read with the words of those facts found once for the rule");

fn seeded_by_words(rule: &ToldRule, facts: &Facts, words: &AddedWords, outcomes: &mut Vec<(Vec<WordId>, Time)>, fills: Option<&Cell<usize>>) {
    for (at, condition) in rule.conditions.iter().enumerate() {
        let Condition::Fact { pattern: first, .. } = condition else { continue };
        if condition_mask(condition) & !words.mask != 0 {
            continue;
        }
        let mut seeds = words.fitting(first).filter(|(place, _)| holds_words(first, facts.fact(*place))).peekable();
        if seeds.peek().is_none() {
            continue;
        }
        let mut others: Vec<&Condition> = rule.conditions.iter().enumerate().filter(|(other, _)| *other != at).map(|(_, condition)| condition).collect();
        let empty = empty_filling();
        let mut held = held_filling(&empty);
        let mut found = Vec::new();
        let mut joined = Vec::new();
        for (place, at) in seeds {
            fit_from(first, facts.fact(*place), &mut held, &mut found);
            if !took_fillings(fills, found.len()) {
                return;
            }
            for filling in found.drain(..) {
                join(&mut others, facts, &filling, (*at, *at, false), &mut joined);
                for (whole, time) in joined.drain(..) {
                    if none_fits(&rule.absent, facts, &whole)
                        && apart(rule, &whole)
                        && let Some(outcome) = fill(&rule.outcome, &whole)
                        && facts.newer(&outcome, time)
                    {
                        outcomes.push((outcome, time));
                    }
                }
            }
        }
    }
}
because!(seeded_by_words, ChainOfRules, "the outcomes of one rule that rest on the given facts: each of its conditions read on one of them that holds its rarest fixed word, in their order, at the place in the story at which it came, its other conditions read on all facts, and no condition that must not hold holding");

fn round_outcomes(rules: &[ToldRule], facts: &Facts, added: &[(usize, Time)], fills: Option<&Cell<usize>>) -> Vec<(Vec<WordId>, Time)> {
    let mut outcomes = Vec::new();
    let words = AddedWords::of(added, facts);
    for rule in rules.iter().filter(|rule| steady(rule) && adds(rule)) {
        costed(rule, facts, || seeded_by_words(rule, facts, &words, &mut outcomes, fills));
    }
    outcomes
}

fn took_fillings(fills: Option<&Cell<usize>>, count: usize) -> bool {
    let Some(left) = fills else { return true };
    if count >= left.get() {
        left.set(usize::default());
        return false;
    }
    left.set(left.get() - count);
    true
}
because!(took_fillings, ChainOfRules, "whether a trial chain may still take the given fillings: each takes one of the fillings left, and a chain whose fillings run out takes none and is ended as one that ran away; a chain with no count takes them all");
because!(round_outcomes, ChainOfRules, "the outcomes of one round that add to the facts: each rule of the first layer, with each of its conditions read on a fact that the round before added, at the place in the story at which it came, and its other conditions read on all facts");

const MOST_WRITES: usize = 256;
because!(MOST_WRITES, ChainOfRules, "the most times that one sentence is written in a new form: each writing changes one part of it, as one name of a sentence with several names or one function of a question of functions nested a few dozen deep, and a line whose new form fits its own old form again would write without end; past the bound the facts are marked as stopped at a bound");

fn bracket_spans(fact: &[WordId]) -> Vec<(usize, usize)> {
    let mut open: Vec<usize> = Vec::new();
    let mut spans = Vec::new();
    for (at, word) in fact.iter().enumerate() {
        match group_turn(word_text(*word)) {
            Some(true) => open.push(at),
            Some(false) => {
                if let Some(start) = open.pop() {
                    spans.push((start + 1, at));
                }
            }
            None => {}
        }
    }
    spans.sort_unstable();
    spans
}
because!(bracket_spans, ChainOfRules, "the words inside each pair of brackets of a sentence, as the places where they start and end, in the order of their opening brackets, so that an outer part is tried before the parts inside it");

fn written_once(write: &ToldRule, fact: &[WordId], facts: &Facts) -> Option<Vec<WordId>> {
    let [Condition::Fact { pattern: old, .. }, others @ ..] = write.conditions.as_slice() else { return None };
    if !holds_words(old, fact) {
        return None;
    }
    let mut spans = vec![(0, fact.len())];
    if write.kind == LineKind::ReplacesInside {
        spans.extend(bracket_spans(fact));
    }
    let mut others: Vec<&Condition> = others.iter().collect();
    for (start, end) in spans {
        let mut found = Vec::new();
        fit_into(old, &fact[start..end], &empty_filling(), &mut found);
        for filling in found {
            let mut joined = Vec::new();
            join(&mut others, facts, &filling, (0, STORY_END, false), &mut joined);
            let changed = joined.iter().filter(|(whole, _)| none_fits(&write.absent, facts, whole)).filter_map(|(whole, _)| fill(&write.outcome, whole)).find(|new| new.as_slice() != &fact[start..end]);
            if let Some(new) = changed {
                return Some(fact[..start].iter().copied().chain(new).chain(fact[end..].iter().copied()).collect());
            }
        }
    }
    None
}
because!(written_once, ChainOfRules, "the new form of a sentence under the first line that writes it again: the first filling under which the sentence is the old form and the other conditions of the line hold on the facts so far, and nothing when no line fits; a line that writes inside brackets also fits the words inside each pair of brackets, the whole sentence first and then the parts from the left, and writes the new form in place of those words alone; a new form that is the same as the old one is no fit, so that such a line does not use up the writes of a sentence and the next line still writes it");

fn in_place(writes: &[ToldRule], facts: &Facts, outcomes: Vec<(Vec<WordId>, Time)>) -> Vec<(Vec<WordId>, Time)> {
    if writes.is_empty() {
        return outcomes;
    }
    outcomes
        .into_iter()
        .map(|(mut fact, time)| {
            let mut left = MOST_WRITES;
            while let Some(new) = writes.iter().find_map(|write| written_once(write, &fact, facts)) {
                if left == usize::default() {
                    facts.stopped.store(true, Ordering::Relaxed);
                    break;
                }
                fact = new;
                left -= 1;
            }
            (fact, time)
        })
        .collect()
}
because!(in_place, ChainOfRules, "the sentences that come to the facts, each written in its new form as long as a line that writes a sentence again fits it, in the order of the library, so that a sentence with several names that each line writes one at a time comes as one sentence and not as every mix of old and new names; up to the bound of writes of one sentence, past which the facts are marked as stopped at a bound");

fn added_places(facts: &mut Facts, outcomes: Vec<(Vec<WordId>, Time)>) -> Vec<(usize, Time)> {
    let mut added: Vec<(usize, Time)> = outcomes.into_iter().filter_map(|(outcome, time)| facts.add(outcome, time)).collect();
    if !facts.history {
        for (place, time) in added.iter_mut() {
            *time = facts.latest(*place);
        }
    }
    added.sort_unstable();
    added.dedup();
    added
}
because!(added_places, ChainOfRules, "the places of the facts that outcomes added or moved to a later place in the story, each once with its latest place in the story; with the history, each new place in the story of a fact");

pub fn knowledge_base(known: &[Vec<WordId>], rules: &[ToldRule], history: bool) -> Facts {
    let mut facts = Facts { history, ..Facts::default() };
    let mut added = added_places(&mut facts, known.iter().map(|fact| (fact.clone(), 0)).collect());
    while !added.is_empty() && facts.len() < MOST_KNOWN_FACTS && facts.words.len() <= MOST_KNOWN_FACTS * MOST_FACT_WORDS && !facts.spent() {
        let outcomes = round_outcomes(rules, &facts, &added, None);
        added = added_places(&mut facts, outcomes);
    }
    if !added.is_empty() {
        facts.stopped.store(true, Ordering::Relaxed);
    }
    facts.work.store(0, Ordering::Relaxed);
    facts
}
because!(knowledge_base, ChainOfRules, "the told facts of a topic and every outcome of its rules of the first layer on them: they rest on no told sentence of a story, so they are found once for a topic and lie under each story, before its first told sentence; the rules of the later layer wait for the story, since a told sentence may take back what they would say; up to the bound of what a topic may know, in facts and in their words, with a mark when it stopped there with facts still to read");

pub(super) fn reads_question(rules: &[ToldRule]) -> bool {
    let start: Pattern = fact_numbers(QUESTION_TOLD).into_iter().map(LogicPiece::Word).collect();
    rules.iter().flat_map(|rule| &rule.conditions).any(|condition| matches!(condition, Condition::Fact { pattern, .. } if pattern.starts_with(&start)))
}
because!(reads_question, ChainOfRules, "whether a rule of a topic reads the question as a fact: a condition that starts with the words that tell the question; only such a topic is told the question, so that a rule written for any sentence (\"If A is B\") never reads the question of a kind that it was not written for");

pub fn told_with_question<'a>(told: &'a [String], question: &str, rules: &[ToldRule]) -> Cow<'a, [String]> {
    if !reads_question(rules) {
        return Cow::Borrowed(told);
    }
    Cow::Owned(told.iter().cloned().chain([format!("{QUESTION_TOLD}{question}")]).collect())
}
because!(told_with_question, ChainOfRules, "the told sentences of a case, and after them the question as one more told sentence when a rule of the topic reads it, so that rules can take the question apart into the descriptions that it asks about, as they take apart a sentence of the story");

pub fn chain_facts(told: &[String], base: &Arc<Facts>, rules: &[ToldRule], rules_hash: u64) -> Facts {
    chain_facts_within(told, base, (rules, rules_hash), base.len() + MOST_FACTS, MOST_READS)
}
because!(chain_facts, ChainOfRules, "the facts of a story under the bound of the engine on the facts of one story, counted over the facts that its topic knows before it, with the hash of the rules that the caller keeps for them, so that the rules are not hashed again for each story");

pub struct StoryStore;
source!(StoryStore, "memoization of a pure function (Michie): the facts of a story depend only on the rules, the facts known before it, the told sentences and the bounds, and the board reads one story under one topic many times (other questions of the story, each learning of a kind, the controls), so the facts of a chain are kept and given again; a kept chain is given as a new empty layer over it with its own count of the facts read, starting where the chain left it, so that the facts read by the answers of one reader never count against another and the answers stay the same, and no fact is copied");

const MOST_STORED_WORDS: usize = 40_000_000;
because!(MOST_STORED_WORDS, StoryStore, "the most words of the facts of kept stories, so that the memory of a run stays bounded while the stories of the kinds that run at the same time come again; room for the chains of the kinds that run at the same time and for most of those that come again later, since memory is the scarce part when several runs share the machine and a chain that was let go is only made again; counted in words and not in facts, so that chains whose facts are long take no more memory than chains of short facts");

const STORED_AFTER_LETTING_GO: usize = 20_000_000;
because!(STORED_AFTER_LETTING_GO, StoryStore, "the words that the facts of the kept chains may hold after the oldest are let go: half of the most, so that the chains kept last, those of the kinds that run now, stay and still give their facts, and letting go comes seldom");

type StoryKey = (u64, u64, usize, usize, Vec<String>);
because!(StoryKey, StoryStore, "what a chain reads: a hash of the rules, the number of the facts known before any story (FactsId), the two bounds, and the told sentences");

#[derive(Default)]
struct KeptStories {
    by_key: HashMap<StoryKey, (Arc<Facts>, usize)>,
    order: VecDeque<StoryKey>,
    words: usize,
}
because!(KeptStories, StoryStore, "the kept chains by what they read, each with the number of words of its own facts, the order in which they were kept, and how many words their own facts hold in all");

static KEPT: OnceLock<Mutex<KeptStories>> = OnceLock::new();
because!(KEPT, StoryStore, "the kept chains of the run, shared by the lanes, since a story comes again under the same topic in other kinds and other learnings");

pub fn rules_key(rules: &[ToldRule]) -> u64 {
    let mut hasher = DefaultHasher::new();
    rules.hash(&mut hasher);
    hasher.finish()
}
because!(rules_key, StoryStore, "a hash of the rules of a chain, so that two topics with different rules over the same known facts never share a kept chain");

pub fn chain_facts_once(told: &[String], base: &Arc<Facts>, rules: (&[ToldRule], u64), bounds: (usize, usize, usize)) -> Facts {
    chain_words_once(&told_words(told), base, rules, bounds)
}

fn told_words(told: &[String]) -> Vec<Vec<WordId>> {
    told.iter().map(|sentence| fact_numbers(sentence)).collect()
}
because!(told_words, ChainOfRules, "the told sentences of a story read into the words that the rules see, one fact for each sentence in order");

pub fn chain_words_once(told: &[Vec<WordId>], base: &Arc<Facts>, rules: (&[ToldRule], u64), (most, reads, more_words): (usize, usize, usize)) -> Facts {
    let fills = Cell::new(MOST_TRIAL_FILLINGS);
    let made = chained(told, base, rules, (most, reads, more_words), Some(&fills));
    add_scope_work(chain_work(&made, reads) + (MOST_TRIAL_FILLINGS - fills.get()), false);
    made
}

const MOST_TRIAL_FILLINGS: usize = 200_000;
because!(MOST_TRIAL_FILLINGS, StoryStore, "the most fillings that the first conditions of the rules of one trial chain may take from facts: the trials of the board take a few thousand at most, while a reading of open words alone (a sentence form with no fixed word) fits a long fact in thousands of ways and feeds on its own outcomes, so that its chain runs for seconds before the bound on the length of a fact ends it; past this many the trial is refused at once as one that ran away, which it would be anyway");
because!(chain_words_once, StoryStore, "chain_facts_once for told sentences already read into words, so that a search that chains the same lessons under many readings reads their sentences once");
because!(chain_facts_once, StoryStore, "the facts of a story under the given bounds, not kept: for a caller that reads them once, such as a trial of learned rules whose rules never come again, since a kept chain costs memory beyond its facts; a fact longer than all its told sentences together and the given words more ends the chain as one that reached the bound on the facts read");

pub fn idle_sentences(before: &[ToldRule], after: &[ToldRule]) -> Option<Vec<Pattern>> {
    let key = |rule: &ToldRule| BuildHasherDefault::<DefaultHasher>::default().hash_one(rule);
    let kept: Vec<u64> = before.iter().map(key).collect();
    let mut kept_at: Vec<usize> = Vec::with_capacity(kept.len());
    let mut added: Vec<&ToldRule> = Vec::new();
    for (at, rule) in after.iter().enumerate() {
        if kept_at.len() < kept.len() && key(rule) == kept[kept_at.len()] {
            kept_at.push(at);
        } else {
            added.push(rule);
        }
    }
    let plain = |rule: &ToldRule| rule.kind == LineKind::Adds && rule.name.is_empty() && rule.absent.is_empty() && !rule.conditions.is_empty() && rule.conditions.iter().all(|condition| matches!(condition, Condition::Fact { when: Moment::Any, .. }));
    if kept_at.len() < kept.len() || !added.iter().all(|rule| plain(rule)) || after.iter().any(|rule| rule.kind == LineKind::ReadsTold) || reads_question(before) != reads_question(after) {
        return None;
    }
    let (old, new) = (not_strata(before).0, not_strata(after).0);
    if kept_at.iter().zip(&old).any(|(at, stratum)| new[*at] != *stratum) {
        return None;
    }
    Some(added.iter().flat_map(|rule| &rule.conditions).filter_map(|condition| match condition {
        Condition::Fact { pattern, .. } => Some(pattern.clone()),
        _ => None,
    }).collect())
}
because!(idle_sentences, ChainOfRules, "the sentences of the rules that one list of rules adds to another, when a story that has no fact that holds the fixed words of one of them has the same chain under both lists: the rules of the first list stand in the second in the same order; each added rule only adds its outcome, has no name, nothing that must be absent and only conditions that must be facts at any time, so that a round reads its other conditions only from a fact that fits one of them, and it reads no fact before; no rule reads told rules, so the rules of a story are the rules of the list; the question is told under both lists alike, and the rules that read what is absent or how many have the same strata; nothing when one of these does not hold");

pub fn chain_facts_within(told: &[String], base: &Arc<Facts>, (rules, rules_hash): (&[ToldRule], u64), most: usize, reads: usize) -> Facts {
    if rules.is_empty() {
        let made = chained(&told_words(told), base, (rules, rules_hash), (most, reads, usize::MAX), None);
        add_scope_work(chain_work(&made, reads), false);
        return made;
    }
    let key: StoryKey = (rules_hash, base.id(), most, reads, told.to_vec());
    let store = KEPT.get_or_init(Mutex::default);
    if let Some((kept, _)) = store.lock().ok().and_then(|kept| kept.by_key.get(&key).cloned()) {
        add_scope_work(chain_work(&kept, reads), false);
        return Facts::given_again(&kept);
    }
    let on_disk = disk_key(told, base, (rules, rules_hash), (most, reads));
    let (made, fresh) = match on_disk.as_ref().and_then(|disk| disk_read(disk, base)) {
        Some(read) => (Arc::new(read), false),
        None => (Arc::new(chained(&told_words(told), base, (rules, rules_hash), (most, reads, usize::MAX), None).shut()), true),
    };
    let own = made.words.len();
    let mut gone: Vec<(Arc<Facts>, usize)> = Vec::new();
    let mut kept_now = false;
    if let Ok(mut kept) = store.lock()
        && !kept.by_key.contains_key(&key)
    {
        if kept.words + own > MOST_STORED_WORDS {
            while kept.words + own > STORED_AFTER_LETTING_GO
                && let Some(oldest) = kept.order.pop_front()
            {
                if let Some(chain) = kept.by_key.remove(&oldest) {
                    kept.words -= chain.1;
                    gone.push(chain);
                }
            }
        }
        kept.words += own;
        kept.order.push_back(key.clone());
        kept.by_key.insert(key, (Arc::clone(&made), own));
        kept_now = true;
    }
    drop(gone);
    if fresh
        && kept_now
        && let Some(disk) = &on_disk
    {
        disk_write(disk, &made);
    }
    add_scope_work(chain_work(&made, reads), false);
    Facts::given_again(&made)
}

fn chain_work(made: &Facts, reads: usize) -> usize {
    made.work.load(Ordering::Relaxed).saturating_sub(MOST_READS.saturating_sub(reads))
}
because!(chain_work, WorkBudget, StoryStore, "the facts that a chain read, from the count it started at for its bound of reads: the same for a chain made now, kept from earlier or read back from disk, so that the work that a learning counts for it does not depend on which learning made it first");
because!(chain_facts_within, StoryStore, StoryDisk, "the facts of a story under the given bounds: a copy of the kept chain of the same rules, known facts, bounds and told sentences, or the chain of the same content read back from the store on disk of an earlier run when the run keeps one, or a new chain, kept for later with its indexes shut and written to the store on disk, and given as a layer over the kept chain in place of a second copy of its facts, the same for the first reader as for every later one; when the facts of the kept chains would hold too many words, the oldest are let go until half of the most is left, and their memory is given back after the other lanes may read the kept chains again; a chain with no rules is only its told sentences and is not kept");

fn added_within(facts: &mut Facts, longest: usize, outcomes: Vec<(Vec<WordId>, Time)>) -> Vec<(usize, Time)> {
    let added = added_places(facts, outcomes);
    if added.iter().any(|(place, _)| facts.fact(*place).len() > longest) {
        facts.work.store(MOST_READS + 1, Ordering::Relaxed);
    }
    added
}
because!(added_within, ChainOfRules, "the outcomes added to the facts, and the chain marked as one that read its bound of facts when one of them is longer than the given words: a rule whose outcome holds its own condition with more words, as 'If A B C then the man is A', makes a longer fact in each round, and the ways to read a fact grow as the square of its length, while the facts grow by few in each round");

fn story_rules(facts: &Facts, added: &[(usize, Time)], any: &[WordId], read: &mut HashSet<usize>) -> Vec<ToldRule> {
    added.iter().filter(|(place, _)| read.insert(*place)).filter_map(|(place, _)| told_rule(facts.fact(*place), any)).collect()
}
because!(story_rules, ChainOfRules, "the rules that the facts added last state, each fact read once, also when it comes again at a later place");

fn chained(told: &[Vec<WordId>], base: &Arc<Facts>, (rules, rules_hash): (&[ToldRule], u64), (most, reads, more_words): (usize, usize, usize), fills: Option<&Cell<usize>>) -> Facts {
    let spent = learning_clock();
    let most_words = most.saturating_sub(base.len()).saturating_mul(MOST_FACT_WORDS);
    let full = |facts: &Facts| facts.len() >= most || facts.words.len() > most_words;
    let mut facts = Facts::on(base);
    facts.work.store(MOST_READS.saturating_sub(reads), Ordering::Relaxed);
    let outcomes: Vec<(Vec<WordId>, Time)> = told.iter().enumerate().map(|(at, words)| (words.clone(), at + 1)).collect();
    let longest = outcomes.iter().map(|(fact, _)| fact.len()).sum::<usize>().saturating_add(more_words);
    let writes: Vec<ToldRule> = rules.iter().filter(|rule| matches!(rule.kind, LineKind::Replaces | LineKind::ReplacesInside)).cloned().collect();
    let any: Vec<WordId> = rules.iter().filter(|rule| rule.kind == LineKind::ReadsTold).flat_map(|rule| &rule.outcome).filter_map(|logic_piece| match logic_piece {
        LogicPiece::Word(word) => Some(*word),
        _ => None,
    }).collect();
    let reads_told = rules.iter().any(|rule| rule.kind == LineKind::ReadsTold);
    let mut rules: Cow<'_, [ToldRule]> = Cow::Borrowed(rules);
    let mut read: HashSet<usize> = HashSet::new();
    let outcomes = in_place(&writes, &facts, outcomes);
    let mut added = added_within(&mut facts, longest, outcomes);
    facts.told = added.iter().map(|(place, _)| *place as FactPlace).collect();
    let mut changes: Vec<usize> = added.iter().map(|(place, _)| *place).collect();
    let mut seen: Vec<Option<usize>> = Vec::new();
    let mut kept_strata: Option<(usize, Arc<Vec<usize>>)> = None;
    let mut made_by: HashMap<Vec<WordId>, Vec<Vec<WordId>>> = HashMap::new();
    let mut defaults_read = false;
    let mut later_from = Time::default();
    loop {
        while !full(&facts) && !facts.spent() {
            if let Some(first) = added.iter().map(|(_, time)| *time).min() {
                later_from = later_from.min(first);
            }
            let fresh = if reads_told { story_rules(&facts, &added, &any, &mut read) } else { Vec::new() };
            if !fresh.is_empty() {
                let mut outcomes = Vec::new();
                for rule in fresh.iter().filter(|rule| steady(rule) && adds(rule)) {
                    whole_outcomes(rule, &facts, &mut outcomes);
                }
                rules.to_mut().extend(fresh);
                let outcomes = in_place(&writes, &facts, outcomes);
                let more = added_within(&mut facts, longest, outcomes);
                changes.extend(more.iter().map(|(place, _)| *place));
                added.extend(more);
                added.sort_unstable();
                added.dedup();
            }
            if added.is_empty() {
                break;
            }
            let outcomes = in_place(&writes, &facts, round_outcomes(&rules, &facts, &added, fills));
            if fills.is_some_and(|left| left.get() == usize::default()) {
                facts.work.store(MOST_READS + 1, Ordering::Relaxed);
                break;
            }
            added = added_within(&mut facts, longest, outcomes);
            changes.extend(added.iter().map(|(place, _)| *place));
        }
        let layered = rules.iter().any(|rule| (!steady(rule) && adds(rule)) || !rule.name.is_empty() || rule.kind == LineKind::AddsByDefault);
        if !layered || full(&facts) || facts.spent() {
            break;
        }
        added = ordered_added(&rules, &writes, &mut facts, (&mut later_from, longest, fills));
        if fills.is_some_and(|left| left.get() == usize::default()) {
            facts.work.store(MOST_READS + 1, Ordering::Relaxed);
            break;
        }
        changes.extend(added.iter().map(|(place, _)| *place));
        if !added.is_empty() {
            continue;
        }
        let strata = match &kept_strata {
            Some((count, strata)) if *count == rules.len() => Arc::clone(strata),
            _ => {
                let strata = strata_of(&rules, matches!(rules, Cow::Owned(_)), rules_hash);
                kept_strata = Some((rules.len(), Arc::clone(&strata)));
                strata
            }
        };
        for stratum in 0..=strata.iter().copied().max().unwrap_or_default() {
            let outcomes = in_place(&writes, &facts, closing_outcomes(&rules, &strata, stratum, &facts, &changes, &mut seen));
            added = added_within(&mut facts, longest, outcomes);
            changes.extend(added.iter().map(|(place, _)| *place));
            if !added.is_empty() {
                break;
            }
        }
        if added.is_empty() {
            let outcomes = in_place(&writes, &facts, defeasible_outcomes(&rules, &facts, &mut made_by));
            added = added_within(&mut facts, longest, outcomes);
            changes.extend(added.iter().map(|(place, _)| *place));
        }
        if added.is_empty() && !defaults_read {
            defaults_read = true;
            let outcomes = in_place(&writes, &facts, default_outcomes(&rules, &facts));
            added = added_within(&mut facts, longest, outcomes);
            changes.extend(added.iter().map(|(place, _)| *place));
        }
        if added.is_empty() {
            break;
        }
    }
    if full(&facts) || facts.spent() {
        facts.stopped.store(true, Ordering::Relaxed);
    }
    chain_ended(spent, facts.stopped_at_bound());
    facts
}
because!(chained, ChainOfRules, "the facts of a story: a layer over the facts that the topic knows before any story, which it reads and does not copy, then its told sentences, each with its place in the story and in its new form when a line writes it again, and every outcome of the rules, round after round, until a round adds nothing or the given bounds on the facts (on their number, and on their words: the bound on their number times the words that a fact may hold on average) and on the facts read are reached, a fact that states a rule ('if ... then ...') becoming a rule of the story when the topic reads told rules, read at once on all facts; then the rules of the later layer that read facts by their place in the story add their outcomes, and the chain goes on from them; only when they add nothing do the rules that read what is absent or how many add theirs, stratum by stratum, the next stratum only when the one before adds nothing, each such rule read again only when a fact that could fit it has come since, and the strata found once for each set of rules of the story, and the chain goes on from them, until none adds a fact; with a mark when the bounds ended it");

pub struct Answered<'a> {
    pub words: Vec<WordId>,
    pub reply: &'a Reply,
    pub filling: Filling,
}
because!(Answered, ChainOfRules, "the words of an answer with the answer of the library that gave them, for the note, and the filling of the places of its question form and its sentences under which it held, for a reply line");

pub fn asks(form: &AskedForm, words: &[WordId]) -> bool {
    if !holds_words(&form.question, words) {
        return false;
    }
    let mut found = Vec::new();
    fit_into(&form.question, words, &empty_filling(), &mut found);
    !found.is_empty()
}
because!(asks, ChainOfRules, "whether a question has the form of a question form, from the words of the question as the rules see them, which the caller cuts once for all the forms that it tries; a question that misses a fixed word of the form is passed over before any fit is tried");

pub fn reply<'a>(forms: &'a [AskedForm], words: &[WordId], facts: &Facts) -> Option<Answered<'a>> {
    let started = learning_clock();
    let answered = replied(forms, words, facts);
    reply_ended(started);
    answered
}
because!(reply, ChainOfRules, "the answer of the first question form that asks the question and holds on the facts, as replied finds it, with the reply counted for the costs of a learning");

fn replied<'a>(forms: &'a [AskedForm], words: &[WordId], facts: &Facts) -> Option<Answered<'a>> {
    for form in forms {
        let mut asked = Vec::new();
        fit_into(&form.question, words, &empty_filling(), &mut asked);
        let mut found = Vec::new();
        for answer in &form.replies {
            let mut held: Vec<&Condition> = answer.conditions.iter().collect();
            for filling in &asked {
                found.clear();
                join(&mut held, facts, filling, (0, STORY_END, false), &mut found);
                let mut holding = found.iter().filter(|(whole, _)| none_fits(&answer.absent, facts, whole)).filter_map(|(whole, _)| fill(&answer.words, whole).map(|said| (said, whole)));
                if answer.every {
                    let mut seen: Vec<&Filling> = Vec::new();
                    let mut said: Vec<WordId> = Vec::new();
                    for (words, whole) in holding.by_ref() {
                        if !seen.iter().any(|known| same_filling(known, whole)) {
                            seen.push(whole);
                            said.extend(words);
                        }
                    }
                    if let Some(first) = seen.first() {
                        return Some(Answered { words: said, reply: answer, filling: (*first).clone() });
                    }
                } else if let Some((said, whole)) = holding.next() {
                    return Some(Answered { words: said, reply: answer, filling: whole.clone() });
                }
            }
        }
    }
    None
}
because!(replied, ChainOfRules, "the first answer, in the order of the library, whose sentences hold for a question: a question form that the question fits gives the fillings of its places, an answer holds when its sentences are facts under one of them (only the latest fact for a sentence marked latest) and no fact fits a sentence that must be absent, and an answer that holds otherwise holds at once; an answer that says every filling gives the words of each different filling under which it holds, in the order found, one after another; each answer is tried with every filling of the question before the next answer, so that an answer that holds otherwise comes only after every way to read the question failed the answers before it");

pub fn reply_filling<'a>(lines: &'a [ReplyLine], words: &[WordId], answered: &Answered) -> Option<(&'a ReplyLine, Filling)> {
    for line in lines {
        let mut asked = Vec::new();
        fit_into(&line.question, words, &answered.filling, &mut asked);
        for filling in asked {
            let mut said = Vec::new();
            fit_into(&line.answer, &answered.words, &filling, &mut said);
            if let Some(whole) = said.into_iter().find(|whole| fill(&line.said, whole).is_some()) {
                return Some((line, whole));
            }
        }
    }
    None
}
because!(reply_filling, ChainOfRules, "the first reply line, in the order of the topic, that says an answer: its question fits the question and its answer fits the words of the answer, each place with the words that the answer gave it, and its sentence has no place left open; with the filling of its places");
