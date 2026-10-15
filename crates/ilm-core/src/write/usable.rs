use crate::write::clean::WRITE_START_UNITS;
use crate::write::desk::{SentenceLine, UsableSentences, WRITE_PLAIN_COST, WRITE_USUAL_ONLY, WritingDesk};
use crate::write::forms::BoundKey;
use crate::write::lessons::{GivenFacts, slot_bit};
use crate::write::nearest::RunShares;
use patterns::source;

pub struct UsableLines;
source!(UsableLines, "a filled form is usable when the reader reads it back to the given facts; it is kept as a sentence with its cost, to open a text or to follow");

impl<'a> WritingDesk<'a> {
    pub(crate) fn usable(&mut self, given: &GivenFacts, names: &[(u32, Vec<u32>)], shares: &RunShares, strict: u32, loose: u32, changed: &[u32]) -> UsableSentences {
        let lessons = self.filler.lessons;
        let reader = self.filler.reader;
        let forms = self.filler.forms;
        let want = given.mask();
        let mut found = UsableSentences::default();
        let numbers: Vec<(u32, u32)> = given.facts.iter().filter(|fact| fact.list).map(|fact| (fact.slot, fact.items.len() as u32)).collect();
        let needs_met = (0..lessons.slots.len() as u32).all(|slot| reader.plain_needs & slot_bit(slot) == 0 || given.has(slot));
        let loose = if needs_met { loose } else { 0 };
        let mut every: Vec<(u64, BoundKey, u32)> = Vec::new();
        for (slots, by_fixed) in forms.by_slots.rows() {
            if slots & !want != 0 {
                continue;
            }
            for (fixed, members) in by_fixed.rows() {
                every.extend(members.iter().map(|number| (*slots, fixed.clone(), *number)));
                let more = self.for_numbers(*slots, fixed, members, &numbers);
                every.extend(more.into_iter().map(|number| (*slots, fixed.clone(), number)));
            }
        }
        for number in changed {
            let form = self.form(*number);
            if form.slots & !want == 0 {
                every.push((form.slots, form.bound_to(), *number));
            }
        }
        for (slots, fixed, number) in every {
            let in_values = |slot: u32, values: &[u32]| given.value(slot).is_some_and(|value| values.contains(&value));
            let other = !fixed.iter().all(|(slot, values)| in_values(*slot, values));
            if other {
                let plain_serves = |slot: u32, values: &[u32]| match self.filler.plain_of(slot) {
                    Some(plain) => loose != 1 || values.contains(&plain),
                    None => false,
                };
                if !(loose > 0 && fixed.iter().all(|(slot, values)| in_values(*slot, values) || plain_serves(*slot, values))) {
                    continue;
                }
            }
            let (rare, key_end, start, needs_other, made, first_count, later_count) = {
                let form = self.form(number);
                let needs_other = form.needs.iter().any(|(slot, values)| given.has(*slot) && !in_values(*slot, values));
                let start: Vec<u32> = form.key.iter().take(WRITE_START_UNITS).copied().collect();
                (form.rare, form.key[form.key.len() - 1], start, needs_other, form.parts.is_some() || form.resized.is_some() || form.edited.is_some(), form.first, form.later)
            };
            if (strict > 0 && !lessons.ends.contains(&key_end)) || (strict >= WRITE_USUAL_ONLY && rare != 0) {
                continue;
            }
            if needs_other {
                continue;
            }
            let Some(filled) = self.fill(number, given, if other { loose } else { 0 }) else {
                continue;
            };
            let Some(said) = self.said_by(number, given, names, &filled) else {
                continue;
            };
            let form = self.form(number);
            let tied_mask = form.tied_mask();
            if slots & !(said | tied_mask) != 0 {
                continue;
            }
            let said = said | tied_mask;
            let follows = form.after.iter().all(|(slot, values)| !given.has(*slot) || in_values(*slot, values));
            let matched = WritingDesk::matched(&filled.toks, shares);
            for (which, times) in [(0usize, first_count), (1usize, later_count)] {
                let usual_start = forms.starts[which].contains(&start);
                let opens = which == 0;
                if times != 0 && (strict < WRITE_USUAL_ONLY || usual_start) && !(other && opens) && (opens || follows) {
                    let cost = rare + i64::from(!usual_start) + i64::from(strict < WRITE_USUAL_ONLY && made) + WRITE_PLAIN_COST * i64::from(other);
                    self.lines.push(SentenceLine { gain: f64::default(), form: number, matched, toks: filled.toks.clone(), said, key: filled.key.clone(), cost, items_at: filled.items_at.clone(), other });
                    let at = (self.lines.len() - 1) as u32;
                    let table = if opens { &mut found.first } else { &mut found.later };
                    table.at(said).push(at);
                }
            }
        }
        found
    }
}
