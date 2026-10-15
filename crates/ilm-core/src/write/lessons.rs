use crate::base::keys::NO_NUMBER;
use crate::base::symbols::Symbols;
use crate::write::find::WantedValue;
use crate::write::names::NameSlots;
use crate::write::rows::FactRow;
use crate::write::signs::{holds_token, low_tokens};
use crate::write::surface::WritingHabits;
use crate::base::first_sight::{FirstSightCount, FirstSightTable, NumberBag, NumberTable, all_counts};
use crate::write::units::PlaceUnits;
use patterns::{because, source};

pub struct CountedWriter;
source!(CountedWriter, "a writer that learns by counting: from lessons, each a list of facts and a text that a person wrote from them, it counts which slots are names, which values are lists, how tokens are written, which runs of tokens say which value, and which forms of sentence the lessons have; then it writes a text for a new list of facts from these counts. Nothing in it names a slot, a value or a word of a set; every bound is a count with a test");

pub const WRITE_LONGEST_RUN: usize = 4;
because!(WRITE_LONGEST_RUN, CountedWriter, "the most tokens in the run of a rule of the reader, and in a value that stands as it is");

pub const WRITE_START_MARK: &str = "^";
because!(WRITE_START_MARK, CountedWriter, "the unit that stands before the first unit of a sentence in every count of neighbours");

pub const WRITE_END_MARK: &str = "$";
because!(WRITE_END_MARK, CountedWriter, "the unit that stands after the last unit of a sentence in every count of neighbours");

pub const WRITE_LABEL_OPEN: &str = "[";
because!(WRITE_LABEL_OPEN, CountedWriter, "the sign that opens the label of a place; a label holds the name of its slot between two brackets, and no token of a text can be such a unit");

pub const WRITE_LABEL_CLOSE: &str = "]";
because!(WRITE_LABEL_CLOSE, CountedWriter, "the sign that closes the label of a place");

pub const WRITE_EMPTY_VALUE: u32 = 0;
because!(WRITE_EMPTY_VALUE, CountedWriter, "the number of the empty value, which is the first value that a model numbers; a value with no token is the empty value");

pub const WRITE_EARLIER_UNIT: usize = 2;
because!(WRITE_EARLIER_UNIT, CountedWriter, "how far back from a unit the unit before its left neighbour stands; counts of a value are kept with the left neighbour and with the unit before that one");

pub const WRITE_SLOTS_MOST: usize = 64;
because!(WRITE_SLOTS_MOST, CountedWriter, "the most slots of one model with the slots of one list of facts: a set of slots is kept as the bits of one word, because the search for a text joins and compares such sets for every row");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextUnit {
    pub sym: u32,
    pub start: u32,
    pub end: u32,
    pub slot: u32,
    pub item: u32,
}
because!(TextUnit, CountedWriter, "one unit of a lesson text: its number, where it starts and ends in the text, and what it stands for: nothing for a plain token, a slot for a given name, a slot and an item for a given item of a list that stands word for word");

impl TextUnit {
    pub fn plain(&self) -> bool {
        self.slot == NO_NUMBER
    }

    pub fn name(&self) -> bool {
        self.slot != NO_NUMBER && self.item == NO_NUMBER
    }

    pub fn placed_item(&self) -> bool {
        self.item != NO_NUMBER
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GivenFact {
    pub slot: u32,
    pub value: u32,
    pub items: Vec<u32>,
    pub list: bool,
}
because!(GivenFact, CountedWriter, "one given fact: its slot, its value, and for a slot that holds lists and has a value the items of the value in their order");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GivenFacts {
    pub facts: Vec<GivenFact>,
}
because!(GivenFacts, CountedWriter, "the facts of one list in the order in which they were given, each slot once. fact gives the fact of a slot; value gives its value, and nothing for a slot that the list does not have, which is not the same as an empty value; has says that the slot stands with a value that is not empty; items gives the items of a list; mask gives the slots as the bits of one word");

impl GivenFacts {
    pub fn fact(&self, slot: u32) -> Option<&GivenFact> {
        self.facts.iter().find(|fact| fact.slot == slot)
    }

    pub fn holds(&self, slot: u32) -> bool {
        self.facts.iter().any(|fact| fact.slot == slot)
    }

    pub fn value(&self, slot: u32) -> Option<u32> {
        self.fact(slot).map(|fact| fact.value)
    }

    pub fn has(&self, slot: u32) -> bool {
        self.fact(slot).is_some_and(|fact| fact.value != WRITE_EMPTY_VALUE)
    }

    pub fn items(&self, slot: u32) -> &[u32] {
        self.fact(slot).map_or(&[], |fact| fact.items.as_slice())
    }

    pub fn mask(&self) -> u64 {
        self.facts.iter().fold(0, |mask, fact| mask | slot_bit(fact.slot))
    }
}

pub fn slot_bit(slot: u32) -> u64 {
    1u64 << slot
}
because!(slot_bit, CountedWriter, "the bit of one slot in a set of slots that is kept as one word");

#[derive(Clone, Debug, Default)]
pub struct WriterLesson {
    pub facts: Vec<(u32, u32)>,
    pub text: String,
    pub given: GivenFacts,
    pub units: Vec<TextUnit>,
    pub sents: Vec<(u32, u32)>,
}
because!(WriterLesson, CountedWriter, "one lesson: its facts as numbers, its text, the facts as they are given to the writer, the units of the text, and where each sentence starts and ends among the units");

#[derive(Clone, Debug, Default)]
pub struct WrittenSurface {
    pub written_as: NumberTable<(u32, u32), u32>,
    pub value_as: NumberTable<(u32, u32), u32>,
    pub spaced: NumberTable<(u32, u32), bool>,
    pub spaced_one: NumberTable<u32, bool>,
    pub no_double: bool,
    pub left_sign: NumberTable<u32, FirstSightCount<u32>>,
    pub signs: FirstSightCount<u32>,
    pub lead_sign: NumberTable<u32, FirstSightTable<u32, FirstSightCount<u32>>>,
    pub left_end: Vec<FirstSightTable<u32, FirstSightCount<u32>>>,
    pub lead_end: Vec<FirstSightCount<(u32, u32, u32)>>,
}
because!(WrittenSurface, CountedWriter, "how the lessons write: the most frequent written form of a token, alone and after the token before it; the most frequent written form of a value; whether a space stands between two tokens; whether a token is ever written twice in a row; the first signs of the units after a unit; and for the names of a slot the endings of their first words by the unit to the left");

#[derive(Clone, Debug, Default)]
pub struct WriterLessons {
    pub slots: Symbols,
    pub vals: Symbols,
    pub syms: Symbols,
    pub shapes: Symbols,
    pub rows: Vec<WriterLesson>,
    pub cuts: Vec<Option<String>>,
    pub values: Vec<FirstSightCount<u32>>,
    pub name_slot: Vec<bool>,
    pub ends: NumberBag<u32>,
    pub sure_names: NumberTable<u32, Vec<Vec<u32>>>,
    pub labels: Vec<u32>,
    pub label_slot: NumberTable<u32, u32>,
    pub start: u32,
    pub end: u32,
    pub blank: u32,
    pub surface: WrittenSurface,
    pub rank: Vec<u32>,
    pub value_rank: Vec<u32>,
    pub value_tokens: Vec<Vec<u32>>,
}
because!(WriterLessons, CountedWriter, NameSlots, PlaceUnits, WritingHabits, "the lessons as numbers and what is counted before the reader: the slots, values, units and written forms by first sight; which slots hold lists and with what cut; which slots are names and which names are sure; which tokens end a sentence; the label of the place of each slot; the marks of the start and of the end of a sentence and the unit that is nothing, for a unit that was taken out; how the lessons write; the place of each unit and of each value in the order of their texts, so that a sort over words is a compare of numbers; and the tokens of each value. learn reads the rows once in their order. The methods that cut a text and a value are the same for a lesson and for a list of facts that is written later");

#[derive(Clone, Debug, Default)]
pub struct FreshNumbers {
    pub vals: Vec<String>,
    pub syms: Vec<String>,
    pub slots: Vec<String>,
    pub value_tokens: Vec<Vec<u32>>,
}
because!(FreshNumbers, CountedWriter, "the values, units and slots of one list of facts that the lessons never held, with the tokens of each such value, made when the value gets its number: each gets a number above the numbers of the model for the time of one text, so that the model is only read while it writes and many threads write with one model");

impl FreshNumbers {
    pub(crate) fn number(base: usize, known: Option<u32>, extra: &mut Vec<String>, text: &str) -> u32 {
        if let Some(number) = known {
            return number;
        }
        match extra.iter().position(|seen| seen == text) {
            Some(place) => (base + place) as u32,
            None => {
                extra.push(text.to_string());
                (base + extra.len() - 1) as u32
            }
        }
    }
}

impl WriterLessons {
    pub fn slot_name(&self, slot: u32, fresh: &FreshNumbers) -> String {
        let known = self.slots.len();
        if (slot as usize) < known { self.slots.name(slot).to_string() } else { fresh.slots[slot as usize - known].clone() }
    }

    pub fn fact_value_text<'a>(&'a self, value: u32, fresh: &'a FreshNumbers) -> &'a str {
        let known = self.vals.len();
        if (value as usize) < known { self.vals.name(value) } else { fresh.vals[value as usize - known].as_str() }
    }

    pub fn unit_text<'a>(&'a self, sym: u32, fresh: &'a FreshNumbers) -> &'a str {
        let known = self.syms.len();
        if (sym as usize) < known { self.syms.name(sym) } else { fresh.syms[sym as usize - known].as_str() }
    }

    pub fn value_number(&self, text: &str, fresh: &mut FreshNumbers) -> u32 {
        let number = FreshNumbers::number(self.vals.len(), self.vals.find(text), &mut fresh.vals, text);
        if number as usize >= self.vals.len() + fresh.value_tokens.len() {
            let tokens = self.tokens_of(text, fresh);
            fresh.value_tokens.push(tokens);
        }
        number
    }

    pub fn unit_number(&self, text: &str, fresh: &mut FreshNumbers) -> u32 {
        FreshNumbers::number(self.syms.len(), self.syms.find(text), &mut fresh.syms, text)
    }

    pub fn slot_number(&self, text: &str, fresh: &mut FreshNumbers) -> u32 {
        FreshNumbers::number(self.slots.len(), self.slots.find(text), &mut fresh.slots, text)
    }

    pub fn is_name(&self, slot: u32) -> bool {
        self.name_slot.get(slot as usize).copied().unwrap_or(false)
    }

    pub fn cut(&self, slot: u32) -> Option<&str> {
        self.cuts.get(slot as usize).and_then(|cut| cut.as_deref())
    }

    pub fn is_list(&self, slot: u32) -> bool {
        self.cuts.get(slot as usize).is_some_and(Option::is_some)
    }

    pub fn tokens_of(&self, text: &str, fresh: &mut FreshNumbers) -> Vec<u32> {
        low_tokens(text).iter().map(|token| self.unit_number(token, fresh)).collect()
    }

    pub fn tokens_of_value<'b>(&'b self, value: u32, fresh: &'b FreshNumbers) -> &'b [u32] {
        match self.value_tokens.get(value as usize) {
            Some(tokens) => tokens,
            None => fresh.value_tokens.get(value as usize - self.vals.len()).map_or(&[], |tokens| tokens.as_slice()),
        }
    }

    pub fn given_of(&self, facts: &[(u32, u32)], fresh: &mut FreshNumbers) -> GivenFacts {
        let mut given = GivenFacts::default();
        for (slot, value) in facts {
            let list = self.is_list(*slot) && *value != WRITE_EMPTY_VALUE;
            let items = match (list, self.cut(*slot)) {
                (true, Some(cut)) => {
                    let text = self.fact_value_text(*value, fresh).to_string();
                    text.split(cut).map(|item| self.value_number(item, fresh)).collect()
                }
                _ => Vec::new(),
            };
            let fact = GivenFact { slot: *slot, value: *value, items, list };
            match given.facts.iter_mut().find(|seen| seen.slot == *slot) {
                Some(seen) => *seen = fact,
                None => given.facts.push(fact),
            }
        }
        given
    }

    pub fn facts_of(&self, facts: &[(String, String)], fresh: &mut FreshNumbers) -> Vec<(u32, u32)> {
        facts
            .iter()
            .map(|(slot, value)| {
                let value = if holds_token(value) { self.value_number(value, fresh) } else { WRITE_EMPTY_VALUE };
                (self.slot_number(slot, fresh), value)
            })
            .collect()
    }

    pub fn learn(rows: &[FactRow]) -> Result<WriterLessons, String> {
        let mut lessons = WriterLessons::default();
        lessons.vals.number("");
        let mut fresh = FreshNumbers::default();
        for row in rows {
            let facts: Vec<(u32, u32)> = row
                .facts
                .iter()
                .map(|(slot, value)| {
                    let value = if holds_token(value) { lessons.vals.number(value) } else { WRITE_EMPTY_VALUE };
                    (lessons.slots.number(slot), value)
                })
                .collect();
            lessons.rows.push(WriterLesson { facts, text: row.text.clone(), ..WriterLesson::default() });
        }
        if lessons.slots.len() > WRITE_SLOTS_MOST {
            return Err(format!("{} slots: a set of slots is kept as the bits of one word", lessons.slots.len()));
        }
        lessons.learn_lists();
        for at in 0..lessons.rows.len() {
            let facts = lessons.rows[at].facts.clone();
            for (slot, value) in &facts {
                if let Some(cut) = lessons.cuts[*slot as usize].clone() {
                    if *value != WRITE_EMPTY_VALUE {
                        let text = lessons.vals.name(*value).to_string();
                        for item in text.split(cut.as_str()) {
                            lessons.vals.number(item);
                        }
                    }
                }
            }
            lessons.rows[at].given = lessons.given_of(&facts, &mut fresh);
        }
        lessons.start = lessons.syms.number(WRITE_START_MARK);
        lessons.end = lessons.syms.number(WRITE_END_MARK);
        lessons.blank = lessons.syms.number("");
        for slot in 0..lessons.slots.len() {
            let label = format!("{WRITE_LABEL_OPEN}{}{WRITE_LABEL_CLOSE}", lessons.slots.name(slot as u32));
            let sym = lessons.syms.number(&label);
            lessons.labels.push(sym);
            lessons.label_slot.insert(sym, slot as u32);
        }
        let mut cache: NumberTable<u32, WantedValue> = NumberTable::default();
        lessons.learn_names(&mut cache);
        for at in 0..lessons.rows.len() {
            let units = lessons.units_of(at, &mut cache);
            let sents = lessons.sentences_of(&lessons.rows[at].text, &units);
            lessons.rows[at].units = units;
            lessons.rows[at].sents = sents;
        }
        lessons.learn_surface(&mut cache);
        lessons.rank_all();
        Ok(lessons)
    }

    pub fn compare_units(&self, one: &[u32], other: &[u32]) -> std::cmp::Ordering {
        one.iter().map(|unit| self.rank[*unit as usize]).cmp(other.iter().map(|unit| self.rank[*unit as usize]))
    }

    pub fn lesson_count(&self, slot: u32) -> i64 {
        self.values.get(slot as usize).map_or(0, all_counts)
    }
}
