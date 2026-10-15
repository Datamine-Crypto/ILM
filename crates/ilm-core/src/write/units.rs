use crate::base::keys::NO_NUMBER;
use crate::know::signs::is_space;
use crate::write::find::WantedValue;
use crate::write::lessons::{FreshNumbers, GivenFacts, TextUnit, WRITE_EMPTY_VALUE, WriterLessons};
use crate::write::signs::token_spans;
use crate::base::first_sight::NumberTable;
use patterns::source;

pub struct PlaceUnits;
source!(PlaceUnits, "a given name and a given item of a list that stand word for word in a text are one unit with the label of their slot, the longest value at a place first; every other token is a unit of its own");

impl WriterLessons {
    pub fn spans_of(&self, given: &GivenFacts, text: &str, fresh: &FreshNumbers, cache: Option<&mut NumberTable<u32, WantedValue>>) -> Vec<(usize, usize, u32, u32)> {
        let mut local: NumberTable<u32, WantedValue> = NumberTable::default();
        let cache = cache.unwrap_or(&mut local);
        let mut spans: Vec<(usize, usize, u32, u32)> = Vec::new();
        for fact in &given.facts {
            if self.is_name(fact.slot) && fact.value != WRITE_EMPTY_VALUE {
                let pattern = cache.entry(fact.value).or_insert_with(|| WantedValue::new(self.fact_value_text(fact.value, fresh)));
                spans.extend(pattern.places(text).into_iter().map(|(start, end)| (start, end, fact.slot, NO_NUMBER)));
            } else if self.is_list(fact.slot) {
                for item in &fact.items {
                    let pattern = cache.entry(*item).or_insert_with(|| WantedValue::new(self.fact_value_text(*item, fresh)));
                    spans.extend(pattern.places(text).into_iter().map(|(start, end)| (start, end, fact.slot, *item)));
                }
            }
        }
        spans.sort_by(|one, other| one.0.cmp(&other.0).then(other.1.cmp(&one.1)));
        spans
    }

    pub(crate) fn units_of(&mut self, at: usize, cache: &mut NumberTable<u32, WantedValue>) -> Vec<TextUnit> {
        let fresh = FreshNumbers::default();
        let lesson = &self.rows[at];
        let text = lesson.text.clone();
        let mut spans = self.spans_of(&lesson.given, &text, &fresh, Some(cache));
        spans.push((text.len(), text.len(), NO_NUMBER, NO_NUMBER));
        let mut out: Vec<TextUnit> = Vec::new();
        let mut from = 0usize;
        let mut tokens: Vec<(usize, usize)> = Vec::new();
        for (start, end, slot, item) in spans {
            if start < from {
                continue;
            }
            tokens.clear();
            token_spans(&text, from, start, &mut tokens);
            for (a, b) in &tokens {
                let sym = self.syms.number(&text[*a..*b].to_lowercase());
                out.push(TextUnit { sym, start: *a as u32, end: *b as u32, slot: NO_NUMBER, item: NO_NUMBER });
            }
            if slot != NO_NUMBER {
                out.push(TextUnit { sym: self.labels[slot as usize], start: start as u32, end: end as u32, slot, item });
            }
            from = end;
        }
        out
    }

    pub(crate) fn sentences_of(&self, text: &str, units: &[TextUnit]) -> Vec<(u32, u32)> {
        let mut out: Vec<(u32, u32)> = Vec::new();
        let mut from = 0usize;
        for (at, unit) in units.iter().enumerate() {
            let closes = unit.plain() && self.ends.contains(&unit.sym) && text[unit.end as usize..].chars().next().is_none_or(is_space);
            if closes {
                out.push((from as u32, at as u32 + 1));
                from = at + 1;
            }
        }
        if from < units.len() {
            out.push((from as u32, units.len() as u32));
        }
        out
    }
}
