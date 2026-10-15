use crate::write::desk::{WRITE_ANY_SORT, WRITE_USUAL_ONLY, WritingDesk, WrittenText};
use crate::write::lessons::{GivenFacts, WRITE_EMPTY_VALUE, WRITE_SLOTS_MOST, WriterLessons, slot_bit};
use crate::write::quotes::quoted_text;
use patterns::{because, source};

pub struct WritingTrace;
source!(WritingTrace, "a written text comes with its trace: the forms of the lessons that each sentence came from, with the lesson that first had the form, every step that changed a form, and the facts that no sentence says; so every word of a text can be followed back to the lessons");

const TRACE_BETWEEN: &str = " + ";
because!(TRACE_BETWEEN, WritingTrace, "the mark between the parts of a trace, each part one step or one form");

const TRACE_NO_FORMS: &str = "no sequence of forms says exactly these slots; the values alone are written";
because!(TRACE_NO_FORMS, WritingTrace, "what the trace says when no form serves and the values stand alone");

const TRACE_TAKEN_OUT: &str = "(taken out)";
because!(TRACE_TAKEN_OUT, WritingTrace, "how the trace names a unit before a place that was taken out for the value after it");

fn fact_value_text(lessons: &WriterLessons, slot: u32, value: u32) -> (Vec<String>, String) {
    match lessons.cut(slot) {
        Some(cut) if value != WRITE_EMPTY_VALUE && (value as usize) < lessons.vals.len() => {
            let items: Vec<String> = lessons.vals.name(value).split(cut).map(str::to_string).collect();
            let shown: Vec<String> = items.iter().map(|item| quoted_text(item)).collect();
            let text = if shown.len() == 1 { format!("({},)", shown[0]) } else { format!("({})", shown.join(", ")) };
            (items, text)
        }
        _ => {
            let text = if (value as usize) < lessons.vals.len() { lessons.vals.name(value).to_string() } else { String::new() };
            (vec![text.clone()], quoted_text(&text))
        }
    }
}
because!(fact_value_text, WritingTrace, "a bound value as the trace shows it, in quotes, and a list as its items in quotes inside brackets; with the items as the key by which bound facts are put in order");

impl<'a> WritingDesk<'a> {
    pub(crate) fn said_of(&mut self, given: &GivenFacts, names: &[(u32, Vec<u32>)], parts: &[(u32, Vec<u32>, Vec<(u32, u32)>)], loose: u32) -> u64 {
        let mut said = 0u64;
        for (number, _, items_at) in parts {
            said |= self.form(*number).tied_mask();
            let Some(mut filled) = self.fill(*number, given, loose) else {
                continue;
            };
            filled.items_at = items_at.clone();
            said |= self.said_by(*number, given, names, &filled).unwrap_or(0);
        }
        said
    }

    fn bound_parts(&self, given: &GivenFacts, written: &WrittenText, same: bool) -> Vec<String> {
        let lessons = self.filler.lessons;
        let mut found: Vec<((String, Vec<String>), String)> = Vec::new();
        for (number, _, _) in &written.parts {
            for (slot, value) in &self.form(*number).tied {
                if (given.value(*slot) == Some(*value)) != same {
                    continue;
                }
                let name = lessons.slots.name(*slot).to_string();
                let (items, text) = fact_value_text(lessons, *slot, *value);
                let row = ((name.clone(), items), format!("{name} = {text}"));
                if !found.iter().any(|seen| seen.0 == row.0) {
                    found.push(row);
                }
            }
        }
        found.sort_by(|one, other| one.0.cmp(&other.0));
        found.into_iter().map(|(_, text)| text).collect()
    }

    fn form_steps(&self, written: &WrittenText, out: &mut Vec<String>) {
        let lessons = self.filler.lessons;
        let fresh = &self.filler.fresh;
        let units = |key: &[u32]| key.iter().map(|unit| lessons.unit_text(*unit, fresh)).collect::<Vec<&str>>().join(" ");
        for (number, key, _) in &written.parts {
            let mut form = self.form(*number);
            if *key != form.key {
                let changes: Vec<String> = form.key.iter().zip(key).filter(|(was, now)| was != now).map(|(was, now)| {
                    let now = lessons.unit_text(*now, fresh);
                    format!("{} -> {}", lessons.unit_text(*was, fresh), if now.is_empty() { TRACE_TAKEN_OUT } else { now })
                }).collect();
                out.push(format!("a unit before a place changed for the value: {}", changes.join(", ")));
            }
            if let Some((base, numbers)) = &form.resized {
                let lists: Vec<String> = numbers.iter().map(|(slot, many)| format!("{many} {} of {}", if *many > 1 { "items" } else { "item" }, lessons.slots.name(*slot))).collect();
                out.push(format!("<{}> (a form written again for {}; the form of the lessons is the next one)", units(&form.key), lists.join(", ")));
                form = self.form(*base);
            }
            if let Some((base, taken, put)) = &form.edited {
                let mut steps: Vec<String> = Vec::new();
                if !taken.is_empty() {
                    steps.push(format!("taken out <{}>", units(taken)));
                }
                if !put.is_empty() {
                    steps.push(format!("put in <{}>", units(put)));
                }
                out.push(format!("<{}> (a lesson form changed: {}; the lesson form is the next one)", units(&form.key), steps.join("; ")));
                form = self.form(*base);
            }
            match form.parts {
                None => out.push(format!("<{}> (lessons {}; the first is lesson {})", units(&form.key), form.count, form.lesson.0)),
                Some((a, i, b, j)) => {
                    let (head, tail) = (self.form(a), self.form(b));
                    out.push(format!(
                        "<{} | {}> (head of <{}>, lesson {}; tail of <{}>, lesson {}; lessons with the head times lessons with the tail: {})",
                        units(&head.key[..i as usize]),
                        units(&tail.key[j as usize..]),
                        units(&head.key),
                        head.lesson.0,
                        units(&tail.key),
                        tail.lesson.0,
                        form.count
                    ));
                }
            }
        }
    }

    pub fn trace(&mut self, facts: &[(String, String)]) -> String {
        let lessons = self.filler.lessons;
        let numbered = lessons.facts_of(facts, &mut self.filler.fresh);
        if numbered.iter().any(|(slot, _)| *slot as usize >= WRITE_SLOTS_MOST) {
            return TRACE_NO_FORMS.to_string();
        }
        let (given, names) = self.prepared(&numbered);
        let Some(written) = self.write_forms(&given, &names) else {
            return TRACE_NO_FORMS.to_string();
        };
        let mut out: Vec<String> = Vec::new();
        if written.strict < WRITE_USUAL_ONLY {
            let more = if written.strict > 0 { "" } else { ", also of forms with no end sign" };
            out.push(format!("no sequence of usual forms; the sequence with the fewest rare runs{more}"));
        }
        if written.without_name_test {
            out.push("no sequence says every fact when the form of a name decides its place; written without that test".to_string());
        }
        self.filler.name_forms = !written.without_name_test;
        let said = self.said_of(&given, &names, &written.parts, WRITE_ANY_SORT);
        self.filler.name_forms = true;
        if given.mask() & !said != 0 {
            let lost: Vec<String> = facts
                .iter()
                .zip(&numbered)
                .filter(|(_, (slot, _))| said & slot_bit(*slot) == 0)
                .map(|((slot, value), (_, number))| format!("{slot} = {}", if *number == WRITE_EMPTY_VALUE { "" } else { value.as_str() }))
                .collect();
            out.push(format!("NOT SAID (no sequence of forms says every slot): {}", lost.join(", ")));
        }
        let tied = self.bound_parts(&given, &written, true);
        if !tied.is_empty() {
            out.push(format!("forms of lessons with {}", tied.join(", ")));
        }
        let other = self.bound_parts(&given, &written, false);
        if !other.is_empty() {
            out.push(format!("no sequence of forms of these lessons says every slot; a later sentence is a form of lessons with {}", other.join(", ")));
        }
        self.form_steps(&written, &mut out);
        out.join(TRACE_BETWEEN)
    }

    pub(crate) fn prepared(&mut self, numbered: &[(u32, u32)]) -> (GivenFacts, Vec<(u32, Vec<u32>)>) {
        let lessons = self.filler.lessons;
        let given = lessons.given_of(numbered, &mut self.filler.fresh);
        let mut names: Vec<(u32, Vec<u32>)> = Vec::new();
        for fact in &given.facts {
            if lessons.is_name(fact.slot) && fact.value != WRITE_EMPTY_VALUE {
                names.push((fact.slot, self.filler.value_tokens(fact.value).to_vec()));
            }
        }
        (given, names)
    }
}

