use crate::base::keys::NO_NUMBER;
use crate::write::lessons::WriterLessons;
use crate::write::reader::{ReadRule, TextReader};
use crate::write::show::{ModelLines, ShownLines, WRITE_SHOW_ITEMS, bits_shown, key_shown, units_shown, value_shown};
use patterns::because;

fn show_rules(lessons: &WriterLessons, reader: &TextReader, key: &str, rules: &[ReadRule], depth: usize, shown: &mut ShownLines) {
    for rule in rules {
        let run = units_shown(lessons, &reader.run_keys[rule.run as usize].iter().copied().filter(|unit| *unit != NO_NUMBER).collect::<Vec<u32>>());
        let mut group: Vec<&str> = rule.group.iter().filter(|value| **value != NO_NUMBER).map(|value| lessons.vals.name(*value)).collect();
        group.sort();
        shown.put("reader", &["rule".to_string(), key.to_string(), depth.to_string(), run, group.join(WRITE_SHOW_ITEMS), rule.had.to_string(), rule.held.to_string()]);
        show_rules(lessons, reader, key, &rule.finer, depth + 1, shown);
    }
}
because!(show_rules, ModelLines, "the rules of one key as lines, each finer rule after the rule that it lies in, with its depth");

pub fn show_reader(lessons: &WriterLessons, reader: &TextReader, shown: &mut ShownLines) {
    let name = |slot: u32| lessons.slots.name(slot).to_string();
    shown.put("reader", &["sentences".to_string(), reader.sent_toks.len().to_string()]);
    for (key, rules) in reader.rules.iter().enumerate() {
        if !reader.read_keys[key] {
            continue;
        }
        let text = key_shown(lessons, &reader.keys[key]);
        shown.put("reader", &["key".to_string(), text.clone(), rules.len().to_string()]);
        show_rules(lessons, reader, &text, rules, 0, shown);
    }
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for (run, key) in reader.owner.iter().enumerate() {
        if *key == NO_NUMBER {
            continue;
        }
        let words: Vec<String> = reader.run_keys[run].iter().filter(|unit| **unit != NO_NUMBER).map(|unit| lessons.syms.name(*unit).to_string()).collect();
        rows.push((words.clone(), vec!["owner".to_string(), words.join(" "), key_shown(lessons, &reader.keys[*key as usize])]));
    }
    shown.sorted("reader", rows);
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for (slot, word) in reader.word_of.iter().enumerate() {
        if *word != NO_NUMBER {
            rows.push((vec![name(slot as u32)], vec!["word_of".to_string(), name(slot as u32), lessons.syms.name(*word).to_string()]));
        }
    }
    shown.sorted("reader", rows);
    let mut carried: Vec<String> = (0..lessons.slots.len() as u32).filter(|slot| reader.carried[*slot as usize]).map(name).collect();
    carried.sort();
    shown.put("reader", &["carried".to_string(), carried.join(" ")]);
    shown.put("reader", &["carried_order".to_string(), reader.carried_order.iter().map(|slot| name(*slot)).collect::<Vec<String>>().join(" ")]);
    for (label, shares) in [("whole_share", &reader.whole_share), ("found_share", &reader.found_share)] {
        let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
        for (slot, share) in shares.iter().enumerate() {
            if let Some(share) = share {
                rows.push((vec![name(slot as u32)], vec![label.to_string(), name(slot as u32), bits_shown(*share)]));
            }
        }
        shown.sorted("reader", rows);
    }
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for slot in &reader.carried_order {
        rows.push((vec![name(*slot)], vec!["plain".to_string(), name(*slot), value_shown(lessons, *slot, reader.plain[*slot as usize])]));
    }
    shown.sorted("reader", rows);
    let mut needs: Vec<String> = (0..lessons.slots.len() as u32).filter(|slot| reader.plain_needs >> slot & 1 == 1).map(name).collect();
    needs.sort();
    shown.put("reader", &["plain_needs".to_string(), needs.join(" ")]);
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for (part, mask) in reader.carried_in.rows() {
        let values: Vec<String> = part.iter().zip(&reader.carried_order).map(|(value, slot)| value_shown(lessons, *slot, *value)).collect();
        let mut slots: Vec<String> = (0..lessons.slots.len() as u32).filter(|slot| mask >> slot & 1 == 1).map(name).collect();
        slots.sort();
        if !slots.is_empty() {
            rows.push((values.clone(), vec!["carried_in".to_string(), values.join(WRITE_SHOW_ITEMS), slots.join(" ")]));
        }
    }
    shown.sorted("reader", rows);
    let mut rows: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for (tokens, slots) in &reader.as_it_is {
        let words: Vec<String> = tokens.iter().map(|unit| lessons.syms.name(*unit).to_string()).collect();
        let mut names: Vec<String> = slots.iter().map(|slot| name(*slot)).collect();
        names.sort();
        rows.push((words.clone(), vec!["as_it_is".to_string(), words.join(" "), names.join(" ")]));
    }
    shown.sorted("reader", rows);
}
because!(show_reader, ModelLines, "what the reader learned, as lines: the rules of each key in their order, the owner of each run, the word of each slot, the carried slots, the shares that decide them, the plain sort of text and the values that stand as they are");
