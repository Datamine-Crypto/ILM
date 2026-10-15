use crate::base::keys::NO_NUMBER;
use crate::write::lessons::{FreshNumbers, WRITE_EMPTY_VALUE, WriterLessons};
use crate::write::reader::ReadKey;
use crate::base::first_sight::FirstSightCount;
use patterns::{because, source};

pub struct ModelLines;
source!(ModelLines, "a model that learns by counting is its tables. Printed as lines in a fixed order, each table in the order that carries its meaning, the model can be read by a person, and two builds of it are the same when their lines are the same");

pub const WRITE_SHOW_FIELD: &str = "\t";
because!(WRITE_SHOW_FIELD, ModelLines, "the mark between the parts of a line that shows what a model counted, so that a part may hold spaces");

pub const WRITE_SHOW_NONE: &str = "<none>";
because!(WRITE_SHOW_NONE, ModelLines, "how a line shows a value that a list of facts does not have, apart from an empty value");

pub const WRITE_SHOW_ITEMS: &str = "|";
because!(WRITE_SHOW_ITEMS, ModelLines, "the mark between the values of a group and between the items of a list in a line that shows them");

#[derive(Clone, Debug, Default)]
pub struct ShownLines {
    pub lines: Vec<String>,
}
because!(ShownLines, ModelLines, "the lines that show what a model counted, section by section: a table whose order carries a meaning is shown in that order, every other table in the order of its texts, so that two models are the same when their lines are the same");

impl ShownLines {
    pub fn put(&mut self, section: &str, parts: &[String]) {
        let mut line = String::from(section);
        for part in parts {
            line.push_str(WRITE_SHOW_FIELD);
            line.push_str(part);
        }
        self.lines.push(line);
    }

    pub fn sorted(&mut self, section: &str, mut rows: Vec<(Vec<String>, Vec<String>)>) {
        rows.sort_by(|one, other| one.0.cmp(&other.0));
        for (_, parts) in rows {
            self.put(section, &parts);
        }
    }
}

pub fn pairs_text(lessons: &WriterLessons, seen: &FirstSightCount<u32>, sort: bool) -> String {
    let mut rows: Vec<(&str, i64)> = seen.rows().iter().map(|(sym, count)| (lessons.syms.name(*sym), *count)).collect();
    if sort {
        rows.sort();
    }
    rows.iter().map(|(name, count)| format!("{name}:{count}")).collect::<Vec<String>>().join(" ")
}
because!(pairs_text, ModelLines, "the counts of a small table of units as one text, in the order of the units or in the order of first sight");

pub fn value_shown(lessons: &WriterLessons, slot: u32, value: u32) -> String {
    if value == NO_NUMBER {
        return WRITE_SHOW_NONE.to_string();
    }
    let text = lessons.vals.name(value);
    match lessons.cut(slot) {
        Some(cut) if value != WRITE_EMPTY_VALUE => format!("<{}>", text.split(cut).collect::<Vec<&str>>().join(WRITE_SHOW_ITEMS)),
        _ => text.to_string(),
    }
}
because!(value_shown, ModelLines, "a value as a line shows it: the items of a list between angle marks, the mark for no value, every other value as it is");

pub fn key_shown(lessons: &WriterLessons, key: &ReadKey) -> String {
    let slot = lessons.slots.name(key.slot);
    if key.item == NO_NUMBER { slot.to_string() } else { format!("{slot}={}", lessons.vals.name(key.item)) }
}
because!(key_shown, ModelLines, "a key of the reader as a line shows it: the slot, and for an item of a list the item after it");

pub fn units_shown(lessons: &WriterLessons, units: &[u32]) -> String {
    let fresh = FreshNumbers::default();
    units.iter().map(|unit| if *unit == lessons.blank { "<out>" } else { lessons.unit_text(*unit, &fresh) }).collect::<Vec<&str>>().join(" ")
}
because!(units_shown, ModelLines, "a run of units as one text with a space between them, and a mark for a unit that was taken out");

pub const WRITE_BITS_SIGNS: usize = 16;
because!(WRITE_BITS_SIGNS, ModelLines, "the signs that show the bits of a number of the double width, four bits to a sign");

pub fn bits_shown(number: f64) -> String {
    format!("{:0width$x}", number.to_bits(), width = WRITE_BITS_SIGNS)
}
because!(bits_shown, ModelLines, "a number as the signs of its bits, so that two numbers are shown the same only when they are the same to the last bit");

pub(crate) fn values_shown(lessons: &WriterLessons, slot: u32, values: &[u32]) -> String {
    let mut texts: Vec<String> = values.iter().map(|value| value_shown(lessons, slot, *value)).collect();
    texts.sort();
    format!("{}:{}", lessons.slots.name(slot), texts.join(WRITE_SHOW_ITEMS))
}
because!(values_shown, ModelLines, "a slot with a set of its values as one text, the values in the order of their texts");

pub(crate) fn slots_shown(lessons: &WriterLessons, mask: u64, between: &str) -> String {
    let mut names: Vec<&str> = (0..lessons.slots.len() as u32).filter(|slot| mask >> slot & 1 == 1).map(|slot| lessons.slots.name(slot)).collect();
    names.sort();
    names.join(between)
}
because!(slots_shown, ModelLines, "a set of slots as one text, the names in their order");

pub(crate) fn counts_shown(lessons: &WriterLessons, seen: &FirstSightCount<u32>) -> String {
    let mut rows: Vec<(&str, i64)> = seen.rows().iter().map(|(value, count)| (lessons.vals.name(*value), *count)).collect();
    rows.sort();
    rows.iter().map(|(name, count)| format!("{name}:{count}")).collect::<Vec<String>>().join(" ")
}
because!(counts_shown, ModelLines, "the counts of the values in one place as one text, the values in the order of their texts");
