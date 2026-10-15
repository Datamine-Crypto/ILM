use crate::write::changed::ChangedForms;
use crate::write::fit::{FilledForm, PlaceFiller};
use crate::write::forms::{BoundKey, FormStore, SentenceForm};
use crate::write::lessons::{GivenFacts, WriterLessons};
use crate::write::levels::SearchLevels;
use crate::write::nearest::{NearestRuns, RunShares};
use crate::write::reader::TextReader;
use crate::write::resize::OtherNumbers;
use crate::write::search::BeamSearch;
use crate::base::first_sight::{FirstSightTable, NumberTable};
use crate::write::trace::WritingTrace;
use crate::write::usable::UsableLines;
use patterns::{because, source};
use std::sync::{Arc, RwLock};

pub struct TextChoice;
source!(TextChoice, "the choice among candidate texts by counts: the share of the runs of tokens of a text that stand in the texts of the lessons with the nearest lists of facts, the overlap that Papineni, Roukos, Ward and Zhu count for a written text against texts by persons, here as the aim of the search and not as a measure; sequences of sentences are searched level by level with a beam");

pub const WRITE_CHOICE_RUN: usize = 4;
because!(WRITE_CHOICE_RUN, TextChoice, "the longest run of tokens that the choice among texts counts, the same length as the common measure of written texts counts");

pub const WRITE_RARE_COST: f64 = 10.0;
because!(WRITE_RARE_COST, TextChoice, "what one rare run of a form costs in the choice: more than any count of runs can give");

pub const WRITE_PLAIN_COST: i64 = 100;
because!(WRITE_PLAIN_COST, TextChoice, "the rare runs that a sentence of the plain sort costs in a text of another sort: more than a form has, so a form of the sort that says the same fact comes first");

pub const WRITE_USUAL_ONLY: u32 = 3;
because!(WRITE_USUAL_ONLY, TextChoice, "the strictness at which only usual forms serve: no rare run, an end sign at the end, a usual start");

pub const WRITE_END_SIGN_ONLY: u32 = 1;
because!(WRITE_END_SIGN_ONLY, TextChoice, "the strictness at which every form with an end sign of the lessons serves");

pub const WRITE_ANY_SORT: u32 = 2;
because!(WRITE_ANY_SORT, TextChoice, "the looseness at which a form of lessons with any value of a slot that shapes the text serves; at the first looseness only a form of the plain sort does");

pub type ChoiceRun = [u32; WRITE_CHOICE_RUN];
because!(ChoiceRun, TextChoice, "a run of tokens of a text as one key for the choice, with the mark of no number in the places that a shorter run leaves");

pub type MatchedRuns = [(f64, i64); WRITE_CHOICE_RUN];
because!(MatchedRuns, TextChoice, "for each length of run: the matched count of a text and the number of its runs of that length");

#[derive(Clone, Debug, Default)]
pub struct SentenceLine {
    pub gain: f64,
    pub form: u32,
    pub matched: MatchedRuns,
    pub toks: Vec<u32>,
    pub said: u64,
    pub key: Vec<u32>,
    pub cost: i64,
    pub items_at: Vec<(u32, u32)>,
    pub other: bool,
}
because!(SentenceLine, TextChoice, "one filled form that the reader reads back, as a sentence that a text can hold: what it adds to the count of a text, its form, its matched counts, its tokens, the slots that it says, its units, the number of its rare runs with the costs of an unusual start and of a made form, the items of its lists, and whether it comes from lessons of another sort of text");

#[derive(Clone, Debug, Default)]
pub struct UsableSentences {
    pub first: FirstSightTable<u64, Vec<u32>>,
    pub later: FirstSightTable<u64, Vec<u32>>,
}
because!(UsableSentences, TextChoice, "the usable sentences for one list of facts, grouped by the slots that they say, in the order in which they were found: those that can open a text and those that can follow");

#[derive(Clone, Debug, Default)]
pub struct WrittenText {
    pub parts: Vec<(u32, Vec<u32>, Vec<(u32, u32)>)>,
    pub strict: u32,
    pub without_name_test: bool,
}
because!(WrittenText, TextChoice, "the sentences chosen for a list of facts, each as its form, its units and the items of its lists; how strict the level was at which they were found; and whether the test of the form of a name had to be left out");

pub struct WritingDesk<'a> {
    pub filler: PlaceFiller<'a>,
    pub local: Vec<SentenceForm>,
    pub lines: Vec<SentenceLine>,
    pub(crate) shares: Option<(RunShares, f64)>,
    pub(crate) resized_kept: NumberTable<(u64, BoundKey, Vec<(u32, u32)>), Vec<u32>>,
    pub(crate) without: NumberTable<(u32, u32), i64>,
    pub(crate) tied_to: NumberTable<(u32, u32), u32>,
    pub(crate) changed_kept: NumberTable<(u64, u32, bool), Vec<u32>>,
    pub(crate) share: Option<&'a ResizedShare>,
}

pub type ResizedKey = (u64, BoundKey, Vec<(u32, u32)>);
because!(ResizedKey, OtherNumbers, "the forms of one set of slots and bound values, written again for the given numbers of items of each list slot among them: the key under which the forms are kept");

pub type ResizedShare = RwLock<NumberTable<ResizedKey, Arc<Vec<SentenceForm>>>>;
because!(ResizedShare, OtherNumbers, "the forms written again for other numbers of items, kept for every text and every thread: they depend on the model and on the numbers alone, so the first thread that makes them makes them for all, and a later text takes them as they are");
because!(WritingDesk, TextChoice, NearestRuns, OtherNumbers, ChangedForms, UsableLines, BeamSearch, SearchLevels, WritingTrace, "the work of writing one list of facts. new gives a desk of its own, shared gives a desk that takes the forms for other numbers of items from the model and keeps them there. All sequences of usable sentences that say exactly the given slots are searched, a first sentence and then later ones; each filled form is read back by the reader and dropped when it does not read back to the given facts. The choice among the sequences is by counts: the share of the runs of tokens of the text that stand in the texts of the lessons with the nearest lists of facts. When no sequence of usual forms says the slots, the levels are tried in a fixed order: the fewest rare runs, forms with no end sign, a later sentence of the plain sort, more sentences, then lesson forms changed by one cut, and all of these again without the test of the form of a name; last comes the sequence that says the most slots. The forms that are changed for the list live for the time of this one text, and so are the forms written again for another number of items when the desk is its own");

impl<'a> WritingDesk<'a> {
    pub fn new(lessons: &'a WriterLessons, reader: &'a TextReader, forms: &'a FormStore) -> WritingDesk<'a> {
        WritingDesk { filler: PlaceFiller::new(lessons, reader, forms), local: Vec::new(), lines: Vec::new(), shares: None, resized_kept: NumberTable::default(), without: NumberTable::default(), tied_to: NumberTable::default(), changed_kept: NumberTable::default(), share: None }
    }

    pub fn shared(lessons: &'a WriterLessons, reader: &'a TextReader, forms: &'a FormStore, share: &'a ResizedShare) -> WritingDesk<'a> {
        WritingDesk { share: Some(share), ..WritingDesk::new(lessons, reader, forms) }
    }

    pub(crate) fn form_of<'b>(forms: &'b FormStore, local: &'b [SentenceForm], number: u32) -> &'b SentenceForm {
        let known = forms.forms.len();
        if (number as usize) < known { &forms.forms[number as usize] } else { &local[number as usize - known] }
    }

    pub fn form(&self, number: u32) -> &SentenceForm {
        WritingDesk::form_of(self.filler.forms, &self.local, number)
    }

    pub(crate) fn keep(&mut self, form: SentenceForm) -> u32 {
        self.local.push(form);
        (self.filler.forms.forms.len() + self.local.len() - 1) as u32
    }

    pub(crate) fn fill(&mut self, number: u32, given: &GivenFacts, loose: u32) -> Option<FilledForm> {
        let forms = self.filler.forms;
        let form = WritingDesk::form_of(forms, &self.local, number);
        self.filler.filled(form, given, loose)
    }

    pub(crate) fn said_by(&mut self, number: u32, given: &GivenFacts, names: &[(u32, Vec<u32>)], filled: &FilledForm) -> Option<u64> {
        let form = self.form(number);
        let placed = PlaceFiller::in_places(form, &filled.items_at);
        self.filler.reader.said(self.filler.lessons, given, names, &filled.toks, &placed)
    }
}
