use crate::tools::pattern::codes::{ALL_SPLIT, ASKED_SLOT, CodedNames, DOTS, EARLIER, FREE, LABEL_MASK, LITERAL, PAIR_CELLS, Point, SLOT_LETTERS, SPLIT_STEP, Scope, SlotPatterns, Step, asked_name, name_step};
use crate::tools::pattern::kinds::PatternWords;
use crate::tools::pattern::names::WordTexts;
use patterns::because;

pub fn name_text<N: WordTexts>(names: &N, code: u32) -> String {
    if code & EARLIER == 0 {
        let place = (code as usize).saturating_sub(1);
        return format!("<{}>", &SLOT_LETTERS[place..=place]);
    }
    let step = name_step(code);
    let label = code & LABEL_MASK;
    let before = if step == SPLIT_STEP { "S".to_string() } else { step.to_string() };
    let after = if step == SPLIT_STEP { label.to_string() } else if label == LABEL_MASK { "~".to_string() } else { names.name(label).to_string() };
    format!("<{before}.{after}>")
}
because!(name_text, CodedNames, "the text of a name: a slot of the question as its letter, a free slot as the number of its sentence and its label, both between angle marks");

pub fn value_text<N: WordTexts>(names: &N, code: u32) -> String {
    if code == FREE {
        "*".to_string()
    } else if code & LITERAL != 0 {
        names.name(code & !LITERAL).to_string()
    } else {
        name_text(names, code)
    }
}
because!(value_text, CodedNames, "the text of what stands in a place of a form: a star for a free place, a word as it stands, or a name");

pub fn form_text<N: WordTexts>(names: &N, row: &[u32]) -> String {
    let Some((fix, cells)) = row.split_first() else { return String::new() };
    let head = if *fix == DOTS { "[..]".to_string() } else { format!("[{}]", names.name(*fix)) };
    let body: Vec<String> = cells.chunks_exact(PAIR_CELLS).map(|pair| format!("{}={}", names.name(pair[0]), value_text(names, pair[1]))).collect();
    format!("{head} {}", body.join(" "))
}
because!(form_text, SlotPatterns, "the text of the form of a sentence: its fixed words between brackets, or two dots where they are left out, then each place as its label and what stands there");

pub fn step_text<N: WordTexts>(words: &PatternWords, names: &N, step: &Step, form: &[u32]) -> String {
    let scope = match step.scope {
        Scope::Whole => "",
        Scope::All => "ALL ",
        Scope::Before => "BEFORE ",
        Scope::After => "AFTER ",
        Scope::Next => "NEXT ",
        Scope::Prev => "PREV ",
    };
    let named = if step.second == FREE { name_text(names, step.name) } else { format!("{}+{}", name_text(names, step.name), name_text(names, step.second)) };
    let kind = || words.kind_names.get(step.item as usize).map_or("", String::as_str);
    match step.point {
        Point::Any => format!("{scope}ANY {named} : {}", form_text(names, form)),
        Point::Kind => format!("{scope}KIND {named} {} = {}", kind(), value_text(names, step.value)),
        Point::Last => format!("{scope}LASTM : {}", form_text(names, form)),
        Point::End => format!("END : {}", form_text(names, form)),
        Point::FreeKind => format!("KIND {} = *", kind()),
        Point::Near => format!("{scope}: {}", form_text(names, form)),
        Point::NearKind => format!("{scope}KIND {} = {}", kind(), value_text(names, step.value)),
        Point::Split => format!("SPLIT {} : {}", kind(), names.name(step.value)),
    }
}
because!(step_text, SlotPatterns, "the text of one sentence of a pattern: where it is looked for, how it is pointed at and from which names, and its form or its kind; two patterns that are equal in their counts are put in order by these texts");

pub fn place_text<N: WordTexts>(names: &N, place: &[u32]) -> String {
    let parts: Vec<String> = place
        .iter()
        .map(|part| {
            if part & LITERAL != 0 {
                format!("={}", names.name(part & !LITERAL))
            } else if *part == ALL_SPLIT {
                "S.*".to_string()
            } else if part & EARLIER != 0 {
                let text = name_text(names, *part);
                text[1..text.len() - 1].to_string()
            } else {
                format!("Q.{}", name_text(names, *part))
            }
        })
        .collect();
    parts.join("+")
}
because!(place_text, SlotPatterns, "the text of the place of an answer: for each word a slot of the question, a free slot, or the word as it stands, with a plus between them");

pub fn asked_text<N: WordTexts>(names: &N, form: &[u32]) -> String {
    let parts: Vec<String> = form.iter().map(|cell| if cell & ASKED_SLOT != 0 { name_text(names, asked_name((cell & !ASKED_SLOT) as usize)) } else { names.name(*cell).to_string() }).collect();
    parts.join(" ")
}
because!(asked_text, SlotPatterns, "the text of a question form: its words, with the name of a slot in the place of each word that stands in the story");

pub fn step_names(step: &Step, form: &[u32]) -> Vec<u32> {
    let mut out: Vec<u32> = vec![step.name, step.second];
    if matches!(step.point, Point::Kind | Point::NearKind) {
        out.push(step.value);
    }
    out.extend(form.iter().skip(1).skip(1).step_by(PAIR_CELLS).copied().filter(|value| value & LITERAL == 0));
    out.retain(|name| *name != FREE);
    out
}
because!(step_names, SlotPatterns, "the names that one sentence of a pattern uses: the names that it is found from, the name of the word that a pointing by kind finds, and the names in the places of its form");
