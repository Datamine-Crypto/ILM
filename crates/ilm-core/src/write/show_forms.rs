use crate::base::keys::NO_NUMBER;
use crate::write::forms::{FormStore, ListSides, ListWay, SentenceForm, SlotPhrase};
use crate::write::lessons::WriterLessons;
use crate::write::reader::TextReader;
use crate::write::show::{ModelLines, ShownLines, WRITE_SHOW_ITEMS, WRITE_SHOW_NONE, counts_shown, slots_shown, units_shown, value_shown, values_shown};
use crate::base::first_sight::{FirstSightCount, FirstSightTable};
use patterns::because;

fn form_shown(lessons: &WriterLessons, form: &SentenceForm) -> Vec<String> {
    let name = |slot: u32| lessons.slots.name(slot).to_string();
    let joined = |parts: Vec<String>, mark: &str| parts.join(mark);
    let sets = |sets: &[(u32, Vec<u32>)], sort: bool| {
        let mut parts: Vec<String> = sets.iter().map(|(slot, values)| values_shown(lessons, *slot, values)).collect();
        if sort {
            parts.sort();
        }
        parts.join(";")
    };
    let mut tied: Vec<String> = form.tied.iter().map(|(slot, value)| format!("{}:{}", name(*slot), value_shown(lessons, *slot, *value))).collect();
    tied.sort();
    let lesson = if form.parts.is_some() { format!("({}, {})", form.lesson.0, form.lesson.1) } else { form.lesson.0.to_string() };
    vec![
        "form".to_string(),
        units_shown(lessons, &form.key),
        form.count.to_string(),
        form.first.to_string(),
        form.later.to_string(),
        format!("places={}", joined(form.places.iter().map(|(at, slot)| format!("{at}:{}", name(*slot))).collect(), ",")),
        format!("slots={}", slots_shown(lessons, form.slots, ",")),
        format!("fixed={}", sets(&form.fixed, false)),
        format!("said_at={}", joined(form.said_at.iter().map(|(slot, places)| format!("{}:{}", name(*slot), joined(places.iter().map(|(from, to)| format!("{from}-{to}")).collect(), ","))).collect(), ";")),
        format!("tied={}", tied.join(";")),
        format!("which={}", joined(form.which.iter().enumerate().filter(|(_, number)| **number != NO_NUMBER).map(|(at, number)| format!("{at}:{number}")).collect(), ",")),
        format!("needs={}", sets(&form.needs, true)),
        format!("after={}", sets(&form.after, true)),
        format!("usual={}", form.usual),
        format!("rare={}", form.rare),
        format!("lesson={lesson}"),
    ]
}
because!(form_shown, ModelLines, "one form as the parts of a line: its units, its counts, its places, the slots that it says, the values that it is bound to, where it says each slot, and how usual it is");

pub fn show_forms(lessons: &WriterLessons, reader: &TextReader, forms: &FormStore, shown: &mut ShownLines) {
    let name = |slot: u32| lessons.slots.name(slot).to_string();
    let unit = |sym: u32| if sym == NO_NUMBER { WRITE_SHOW_NONE.to_string() } else { lessons.syms.name(sym).to_string() };
    let words = |units: &[u32]| -> Vec<String> { units.iter().map(|sym| lessons.syms.name(*sym).to_string()).collect() };
    let nothing = |units: &[u32]| if units.is_empty() { "<nothing>".to_string() } else { units_shown(lessons, units) };
    shown.put("forms", &["clean".to_string(), forms.clean.to_string()]);
    for form in &forms.forms[..forms.lesson_forms] {
        shown.put("forms", &form_shown(lessons, form));
    }
    shown.put("forms", &["again".to_string(), slots_shown(lessons, forms.again, " ")]);
    for (which, starts) in forms.starts.iter().enumerate() {
        let rows = starts.iter().map(|start| (words(start), vec!["start".to_string(), which.to_string(), units_shown(lessons, start)])).collect();
        shown.sorted("forms", rows);
    }
    for (label, table) in [("left", &forms.left), ("right", &forms.right)] {
        let mut rows: Vec<((String, u32, String), Vec<String>)> = Vec::new();
        for ((slot, position, beside), seen) in table {
            rows.push(((name(*slot), *position, unit(*beside)), vec![label.to_string(), name(*slot), position.to_string(), unit(*beside), counts_shown(lessons, seen)]));
        }
        rows.sort_by(|one, other| one.0.cmp(&other.0));
        for (_, parts) in rows {
            shown.put("tables", &parts);
        }
    }
    let mut rows: Vec<((String, u32), Vec<String>)> = Vec::new();
    for ((slot, position), seen) in &forms.in_place {
        rows.push(((name(*slot), *position), vec!["in_place".to_string(), name(*slot), position.to_string(), counts_shown(lessons, seen)]));
    }
    rows.sort_by(|one, other| one.0.cmp(&other.0));
    for (_, parts) in rows {
        shown.put("tables", &parts);
    }
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for ((slot, earlier), found) in &forms.lead {
        for (before, seen) in found.rows() {
            let key = vec![name(*slot), unit(*earlier), unit(*before)];
            let mut row = vec!["lead".to_string()];
            row.extend(key.iter().cloned());
            row.push(counts_shown(lessons, seen));
            rows.push((key, row));
        }
    }
    shown.sorted("tables", rows);
    shown.put("joins", &["least".to_string(), forms.join_least.to_string(), forms.join_pairs.to_string()]);
    for form in &forms.forms[forms.lesson_forms..] {
        shown.put("joins", &form_shown(lessons, form));
        if let Some((a, i, b, j)) = form.parts {
            shown.put("joins", &["parts".to_string(), units_shown(lessons, &forms.form(a).key), i.to_string(), units_shown(lessons, &forms.form(b).key), j.to_string()]);
        }
    }
    let rows = forms.usual_runs.iter().map(|(run, count)| (words(run), vec!["usual_run".to_string(), units_shown(lessons, run), count.to_string()])).collect();
    shown.sorted("joins", rows);
    let rows = forms.list_gaps.iter().map(|(gap, count)| (words(gap), vec!["list_gap".to_string(), nothing(gap), count.to_string()])).collect();
    shown.sorted("joins", rows);
    for (label, table) in [("list_two", &forms.list_two), ("list_more", &forms.list_more)] {
        let mut order: Vec<(String, &FirstSightTable<ListWay, FirstSightCount<ListSides>>)> = Vec::new();
        for (slot, found) in table.rows() {
            if *slot == NO_NUMBER {
                if !found.rows().is_empty() {
                    order.push(("None".to_string(), found));
                }
            } else {
                order.push((name(*slot), found));
            }
        }
        order.sort_by(|one, other| one.0.cmp(&other.0));
        for (slot, found) in order {
            for ((inner, last), seen) in found.rows() {
                let text = if label == "list_two" { units_shown(lessons, last) } else { format!("{} / {}", units_shown(lessons, inner), units_shown(lessons, last)) };
                let sides: Vec<String> = seen.rows().iter().map(|((left, right), count)| format!("{},{}:{count}", unit(*left), unit(*right))).collect();
                shown.put("joins", &[label.to_string(), slot.clone(), text, sides.join(" ")]);
            }
        }
    }
    let rows = forms.list_pair.iter().map(|(gap, count)| (words(gap), vec!["list_pair".to_string(), nothing(gap), count.to_string()])).collect();
    shown.sorted("joins", rows);
    let rows = forms.list_last.rows().iter().map(|(gap, count)| (words(gap), vec!["list_last".to_string(), nothing(gap), count.to_string()])).collect();
    shown.sorted("joins", rows);
    let mut sides: Vec<(Vec<String>, &FirstSightTable<(u32, Vec<u32>), SlotPhrase>)> = forms.between.rows().iter().map(|((left, right), found)| (vec![unit(*left), unit(*right)], found)).collect();
    sides.sort_by(|one, other| one.0.cmp(&other.0));
    for (pair, found) in sides {
        for ((slot, phrase), entry) in found.rows() {
            let mut values: Vec<String> = entry.values.iter().map(|value| value_shown(lessons, *slot, *value)).collect();
            values.sort();
            shown.put(
                "phrases",
                &[
                    "between".to_string(),
                    pair.join(" "),
                    name(*slot),
                    units_shown(lessons, phrase),
                    entry.count.to_string(),
                    entry.places.iter().map(u32::to_string).collect::<Vec<String>>().join(","),
                    values.join(WRITE_SHOW_ITEMS),
                    format!("{},{}", entry.beside.0, entry.beside.1),
                ],
            );
        }
    }
    shown.put("phrases", &["runs_seen".to_string(), forms.runs_seen.len().to_string()]);
    shown.put("phrases", &["start_end".to_string(), forms.start_end.len().to_string()]);
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for ((part, first), pairs) in &forms.pairs_after {
        let values: Vec<String> = part.iter().zip(&reader.carried_order).map(|(value, slot)| value_shown(lessons, *slot, *value)).collect();
        let mut key = values.clone();
        key.push(unit(*first));
        rows.push((key, vec!["pairs_after".to_string(), values.join(WRITE_SHOW_ITEMS), unit(*first), pairs.len().to_string()]));
    }
    shown.sorted("phrases", rows);
    for (slots, by_fixed) in forms.by_slots.rows() {
        for (fixed, members) in by_fixed.rows() {
            let mut bound: Vec<String> = fixed.iter().map(|(slot, values)| values_shown(lessons, *slot, values)).collect();
            bound.sort();
            shown.put("groups", &["group".to_string(), slots_shown(lessons, *slots, ","), bound.join(";"), members.len().to_string()]);
            for number in members {
                let form = forms.form(*number);
                shown.put("groups", &["in".to_string(), units_shown(lessons, &form.key), form.count.to_string(), u8::from(form.parts.is_some()).to_string()]);
            }
        }
    }
    for (slots, rows) in forms.near.rows() {
        shown.put("groups", &["near".to_string(), slots_shown(lessons, *slots, ","), rows.len().to_string()]);
    }
}
because!(show_forms, ModelLines, "what was learned about forms, as lines: every lesson form and every joined form in its order, the counts of values beside units, the ways to write lists, the phrases with the units beside them, and the groups of forms in the order in which a text is searched");
