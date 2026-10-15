use crate::base::keys::NO_NUMBER;
use crate::write::forms::{BoundKey, FormStore, NearLesson, WRITE_HEAD_AND_NEXT, WRITE_USUAL_SENTENCES};
use crate::write::joins::WRITE_JOINED_KEPT;
use crate::write::lessons::{WRITE_EMPTY_VALUE, WriterLessons};
use crate::write::quotes::quoted_text;
use crate::base::first_sight::NumberTable;
use patterns::{because, source};

pub struct FormGroups;
source!(FormGroups, "the forms of the lessons are grouped by the slots that they say and by the values that they are bound to, the most frequent first, so that a text for a list of facts is searched among the forms that can say its slots and nothing else");

pub const WRITE_FORMS_KEPT: usize = 40;
because!(WRITE_FORMS_KEPT, FormGroups, "the lesson forms that are kept for one set of slots and one set of values of their fixed words");

fn tied_text(lessons: &WriterLessons, slot: u32, value: u32) -> String {
    let shown = match lessons.cut(slot) {
        Some(cut) if value != WRITE_EMPTY_VALUE => {
            let items: Vec<String> = lessons.vals.name(value).split(cut).map(quoted_text).collect();
            if items.len() == 1 { format!("({},)", items[0]) } else { format!("({})", items.join(", ")) }
        }
        _ => quoted_text(lessons.vals.name(value)),
    };
    format!("({}, {shown})", quoted_text(lessons.slots.name(slot)))
}
because!(tied_text, FormGroups, "a bound fact as a text in quotes with its slot before it, a list as its items in quotes; forms with the same units and the same counts are put in the order of these texts, so that the order is the same in every run");

impl FormStore {
    pub(crate) fn group_forms(&mut self, lessons: &WriterLessons) {
        let texts: Vec<Vec<String>> = self
            .forms
            .iter()
            .map(|form| {
                let mut tied: Vec<String> = form.tied.iter().map(|(slot, value)| tied_text(lessons, *slot, *value)).collect();
                tied.sort();
                tied
            })
            .collect();
        for (from, to, most) in [(0, self.lesson_forms, WRITE_FORMS_KEPT), (self.lesson_forms, self.forms.len(), WRITE_JOINED_KEPT)] {
            let mut order: Vec<usize> = (from..to).collect();
            order.sort_by(|one, other| {
                let (a, b) = (&self.forms[*one], &self.forms[*other]);
                (a.usual < WRITE_USUAL_SENTENCES).cmp(&(b.usual < WRITE_USUAL_SENTENCES)).then(b.count.cmp(&a.count)).then_with(|| lessons.compare_units(&a.key, &b.key)).then_with(|| texts[*one].cmp(&texts[*other]))
            });
            let mut kept: NumberTable<(u64, BoundKey), usize> = NumberTable::default();
            for number in order {
                let form = &self.forms[number];
                let fixed = form.bound_to();
                let seen = kept.entry((form.slots, fixed.clone())).or_insert(0);
                if *seen < most {
                    *seen += 1;
                    self.by_slots.at(form.slots).at(fixed).push(number as u32);
                }
            }
        }
    }

    pub(crate) fn learn_near(&mut self, lessons: &WriterLessons) {
        for (number, lesson) in lessons.rows.iter().enumerate() {
            let mut toks: Vec<u32> = Vec::with_capacity(lesson.units.len() + WRITE_HEAD_AND_NEXT);
            toks.push(lessons.start);
            toks.extend(lesson.units.iter().map(|unit| unit.sym));
            toks.push(lessons.end);
            let mut values = vec![NO_NUMBER; lessons.slots.len()];
            for fact in &lesson.given.facts {
                values[fact.slot as usize] = fact.value;
            }
            self.near.at(lesson.given.mask()).push(NearLesson { lesson: number as u32, toks, values });
        }
    }
}
