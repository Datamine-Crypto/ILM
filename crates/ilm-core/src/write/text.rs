use crate::base::keys::NO_NUMBER;
use crate::write::fit::PlaceFiller;
use crate::write::forms::SentenceForm;
use crate::write::lessons::{GivenFacts, WRITE_EMPTY_VALUE};
use crate::write::signs::first_capital;
use patterns::{because, source};

pub struct SurfaceText;
source!(SurfaceText, "a filled form is written as people write it: each token in its most frequent written form, a space where the lessons have one, and a capital letter at the start");

pub const WRITE_WORD_SPACE: char = ' ';
because!(WRITE_WORD_SPACE, SurfaceText, "the plain space that people write between tokens, where the lessons show a space between them");

impl<'a> PlaceFiller<'a> {
    pub fn text(&mut self, form: &SentenceForm, key: &[u32], items_at: &[(u32, u32)], given: &GivenFacts) -> String {
        let lessons = self.lessons;
        let surface = &lessons.surface;
        let mut out = String::new();
        let mut last = NO_NUMBER;
        for (at, unit) in key.iter().enumerate() {
            if *unit == lessons.blank {
                continue;
            }
            let shape = surface.written_as.get(&(last, *unit)).or_else(|| surface.written_as.get(&(NO_NUMBER, *unit)));
            let mut piece: String = shape.map_or_else(|| lessons.unit_text(*unit, &self.fresh).to_string(), |shape| lessons.shapes.name(*shape).to_string());
            let mut first = *unit;
            let mut end = *unit;
            if let Some(slot) = form.slot_at(at) {
                let value = items_at.iter().find(|(place, _)| *place as usize == at).map(|(_, item)| *item).or_else(|| given.value(slot)).unwrap_or(WRITE_EMPTY_VALUE);
                piece = self.fact_value_text(value).to_string();
                if !lessons.is_name(slot) {
                    let tokens = self.value_tokens(value);
                    first = tokens.first().copied().unwrap_or(first);
                    end = tokens.last().copied().unwrap_or(end);
                    if let Some(shape) = surface.value_as.get(&(slot, value)) {
                        piece = lessons.shapes.name(*shape).to_string();
                    }
                }
            }
            let spaced = surface.spaced.get(&(last, first)).or_else(|| surface.spaced_one.get(&first)).copied().unwrap_or(true);
            if last != NO_NUMBER && spaced {
                out.push(WRITE_WORD_SPACE);
            }
            out.push_str(&piece);
            last = end;
        }
        first_capital(&out)
    }
}
