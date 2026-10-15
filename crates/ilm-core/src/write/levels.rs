use crate::write::desk::{ResizedShare, WRITE_ANY_SORT, WRITE_END_SIGN_ONLY, WRITE_USUAL_ONLY, WritingDesk, WrittenText};
use crate::write::forms::FormStore;
use crate::write::lessons::{GivenFacts, WRITE_EMPTY_VALUE, WRITE_SLOTS_MOST, WriterLessons, slot_bit};
use crate::write::reader::TextReader;
use crate::write::rows::FactRow;
use patterns::{because, source};

pub struct SearchLevels;
source!(SearchLevels, "a text is searched at levels of falling strictness, and a text of an earlier level is always taken before a text of a later one: usual forms first, then forms with rare runs, then changed forms; so the common case is written from what the lessons show most often");

pub const WRITE_MORE_SENTENCES: usize = 2;
because!(WRITE_MORE_SENTENCES, SearchLevels, "the sentences beyond the usual most, for a list of facts that no shorter text says");

pub const WRITE_LEVELS: &[(u32, bool, u32, usize, bool)] = &[
    (WRITE_USUAL_ONLY, false, 0, 0, false),
    (WRITE_END_SIGN_ONLY, false, 0, 0, false),
    (0, false, 0, 0, false),
    (0, false, 1, 0, false),
    (0, false, 1, WRITE_MORE_SENTENCES, false),
    (0, false, 0, 0, true),
    (0, false, 1, 0, true),
    (0, false, 1, WRITE_MORE_SENTENCES, true),
];
because!(WRITE_LEVELS, SearchLevels, "the levels at which a text is searched, in their order, each as its strictness, whether a text that says a part is taken, its looseness, the sentences beyond the usual most, and whether changed lesson forms serve: usual forms; every form with an end sign; every form; a later sentence of the plain sort; more sentences; then the same steps with changed lesson forms. A text of an earlier level is always taken before a text of a later one");

impl<'a> WritingDesk<'a> {
    fn lost_slots(&mut self, given: &GivenFacts, names: &[(u32, Vec<u32>)]) -> u64 {
        let reader = self.filler.reader;
        let Some(best) = self.candidates(given, names, 0, true, 1, WRITE_MORE_SENTENCES, None) else {
            return given.facts.iter().filter(|fact| fact.value != WRITE_EMPTY_VALUE && !reader.carried.get(fact.slot as usize).copied().unwrap_or(false)).fold(0u64, |mask, fact| mask | slot_bit(fact.slot));
        };
        given.mask() & !self.said_of(given, names, &best, WRITE_ANY_SORT)
    }

    pub(crate) fn write_forms(&mut self, given: &GivenFacts, names: &[(u32, Vec<u32>)]) -> Option<WrittenText> {
        let reader = self.filler.reader;
        let any_carried = reader.carried.iter().any(|carried| *carried);
        let mut lost: Option<u64> = None;
        let mut steps: Vec<(bool, u32, bool, u32, usize, bool)> = Vec::new();
        for with_name_test in [true, false] {
            steps.extend(WRITE_LEVELS.iter().map(|(strict, part, loose, more, change)| (with_name_test, *strict, *part, *loose, *more, *change)));
        }
        steps.push((true, 0, true, 1, WRITE_MORE_SENTENCES, false));
        let mut written: Option<WrittenText> = None;
        for (with_name_test, strict, part, loose, more, change) in steps {
            self.filler.name_forms = with_name_test;
            if change && lost.is_none() {
                lost = Some(self.lost_slots(given, names));
            }
            if !(any_carried || loose == 0 || more > 0) {
                continue;
            }
            if let Some(parts) = self.candidates(given, names, strict, part, loose, more, if change { lost } else { None }) {
                written = Some(WrittenText { parts, strict, without_name_test: !with_name_test });
                break;
            }
        }
        self.filler.name_forms = true;
        written
    }

    pub fn write(&mut self, facts: &[(String, String)]) -> String {
        let lessons = self.filler.lessons;
        let reader = self.filler.reader;
        let numbered = lessons.facts_of(facts, &mut self.filler.fresh);
        let values = |all: bool| -> String {
            let kept: Vec<&str> = facts.iter().zip(&numbered).filter(|(_, (slot, _))| all || !reader.carried.get(*slot as usize).copied().unwrap_or(false)).map(|((_, value), (_, number))| if *number == WRITE_EMPTY_VALUE { "" } else { value.as_str() }).collect();
            kept.join(" ")
        };
        let fallback = |values: &dyn Fn(bool) -> String| -> String {
            let said = values(false);
            if said.is_empty() { values(true) } else { said }
        };
        if numbered.iter().any(|(slot, _)| *slot as usize >= WRITE_SLOTS_MOST) {
            return fallback(&values);
        }
        let (given, names) = self.prepared(&numbered);
        let Some(written) = self.write_forms(&given, &names) else {
            return fallback(&values);
        };
        let mut texts: Vec<String> = Vec::new();
        for (number, key, items_at) in &written.parts {
            let forms = self.filler.forms;
            let form = WritingDesk::form_of(forms, &self.local, *number);
            texts.push(self.filler.text(form, key, items_at, &given));
        }
        texts.join(" ")
    }
}

pub struct Writer {
    pub lessons: WriterLessons,
    pub reader: TextReader,
    pub forms: FormStore,
    pub resized: ResizedShare,
}
because!(Writer, SearchLevels, "a model of the writer: what was counted in the lessons, the reader, the forms, and the forms written again for other numbers of items as the texts ask for them. learn makes it from rows of facts and text; desk gives the place of work for one list of facts, where the text and its trace are made; write gives the text for a list of facts; neither changes what the model learned: the forms for other numbers of items that a text adds depend on the model and the numbers alone, so many threads write with one model and the text of a list does not depend on the lists that were written before it");

impl Writer {
    pub fn learn(rows: &[FactRow]) -> Result<Writer, String> {
        let lessons = WriterLessons::learn(rows)?;
        let reader = TextReader::learn(&lessons);
        let forms = FormStore::learn(&lessons, &reader);
        Ok(Writer { lessons, reader, forms, resized: ResizedShare::default() })
    }

    pub fn desk(&self) -> WritingDesk<'_> {
        WritingDesk::shared(&self.lessons, &self.reader, &self.forms, &self.resized)
    }

    pub fn write(&self, facts: &[(String, String)]) -> String {
        self.desk().write(facts)
    }
}
