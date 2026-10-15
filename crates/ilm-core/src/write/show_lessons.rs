use crate::base::keys::NO_NUMBER;
use crate::write::lessons::WriterLessons;
use crate::write::quotes::quoted_text;
use crate::write::show::{ModelLines, ShownLines, pairs_text};
use patterns::because;

pub fn show_lessons(lessons: &WriterLessons, shown: &mut ShownLines) {
    let name = |slot: usize| lessons.slots.name(slot as u32).to_string();
    let unit = |sym: u32| lessons.syms.name(sym).to_string();
    let mut cuts: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for (slot, cut) in lessons.cuts.iter().enumerate() {
        if let Some(cut) = cut {
            cuts.push((vec![name(slot)], vec!["cut".to_string(), name(slot), quoted_text(cut)]));
        }
    }
    shown.sorted("lists", cuts);
    let mut names: Vec<String> = (0..lessons.slots.len()).filter(|slot| lessons.name_slot[*slot]).map(name).collect();
    names.sort();
    shown.put("lists", &["name_slots".to_string(), names.join(" ")]);
    let mut ends: Vec<String> = lessons.ends.iter().map(|sym| unit(*sym)).collect();
    ends.sort();
    shown.put("lists", &["ends".to_string(), ends.join(" ")]);
    let mut sure: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for group in lessons.sure_names.values() {
        for tokens in group {
            let words: Vec<String> = tokens.iter().map(|sym| unit(*sym)).collect();
            sure.push((words.clone(), vec!["sure_name".to_string(), words.join(" ")]));
        }
    }
    shown.sorted("lists", sure);
    for (slot, values) in lessons.values.iter().enumerate() {
        let all: i64 = values.rows().iter().map(|(_, count)| *count).sum();
        shown.put("lists", &["values".to_string(), name(slot), values.rows().len().to_string(), all.to_string()]);
    }
    for (number, lesson) in lessons.rows.iter().enumerate() {
        for (from, to) in &lesson.sents {
            let mut words: Vec<String> = Vec::new();
            for unit in &lesson.units[*from as usize..*to as usize] {
                let mut word = lessons.syms.name(unit.sym).to_string();
                if unit.placed_item() {
                    word.push_str(&format!("{{{}={}}}", name(unit.slot as usize), lessons.vals.name(unit.item)));
                } else if unit.name() {
                    word.push_str(&format!("{{{}}}", name(unit.slot as usize)));
                }
                words.push(word);
            }
            shown.put("units", &[number.to_string(), words.join(" ")]);
        }
    }
    let surface = &lessons.surface;
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for ((before, token), shape) in &surface.written_as {
        let form = lessons.shapes.name(*shape).to_string();
        if *before == NO_NUMBER {
            rows.push((vec![String::new(), unit(*token)], vec!["written".to_string(), unit(*token), form]));
        } else {
            rows.push((vec!["pair".to_string(), unit(*before), unit(*token)], vec!["written".to_string(), format!("{} {}", unit(*before), unit(*token)), form]));
        }
    }
    shown.sorted("surface", rows);
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for ((slot, item), shape) in &surface.value_as {
        let parts = vec![name(*slot as usize), lessons.vals.name(*item).to_string()];
        rows.push((parts.clone(), vec!["value_as".to_string(), parts[0].clone(), parts[1].clone(), lessons.shapes.name(*shape).to_string()]));
    }
    shown.sorted("surface", rows);
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for ((before, token), yes) in &surface.spaced {
        rows.push((vec![unit(*before), unit(*token)], vec!["spaced".to_string(), unit(*before), unit(*token), u8::from(*yes).to_string()]));
    }
    shown.sorted("surface", rows);
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for (token, yes) in &surface.spaced_one {
        rows.push((vec![unit(*token)], vec!["spaced_one".to_string(), unit(*token), u8::from(*yes).to_string()]));
    }
    shown.sorted("surface", rows);
    shown.put("surface", &["no_double".to_string(), u8::from(surface.no_double).to_string()]);
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for (before, signs) in &surface.left_sign {
        rows.push((vec![unit(*before)], vec!["left_sign".to_string(), unit(*before), pairs_text(lessons, signs, true)]));
    }
    shown.sorted("surface", rows);
    shown.put("surface", &["signs".to_string(), pairs_text(lessons, &surface.signs, true)]);
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for (earlier, found) in &surface.lead_sign {
        for (before, signs) in found.rows() {
            rows.push((vec![unit(*earlier), unit(*before)], vec!["lead_sign".to_string(), unit(*earlier), unit(*before), pairs_text(lessons, signs, true)]));
        }
    }
    shown.sorted("surface", rows);
    let mut order: Vec<usize> = (0..lessons.slots.len()).collect();
    order.sort_by_key(|slot| name(*slot));
    for slot in &order {
        for (before, seen) in surface.left_end[*slot].rows() {
            shown.put("surface", &["left_end".to_string(), name(*slot), unit(*before), pairs_text(lessons, seen, false)]);
        }
    }
    for slot in &order {
        let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
        for ((earlier, before, ending), count) in surface.lead_end[*slot].rows() {
            let parts = vec![unit(*earlier), unit(*before), unit(*ending)];
            let mut row = vec!["lead_end".to_string(), name(*slot)];
            row.extend(parts.iter().cloned());
            row.push(count.to_string());
            rows.push((parts, row));
        }
        shown.sorted("surface", rows);
    }
}
because!(show_lessons, ModelLines, "what was counted before the reader, as lines: the cuts of lists, the name slots, the tokens that end a sentence, the sure names, the units of every lesson sentence, and how the lessons write");
