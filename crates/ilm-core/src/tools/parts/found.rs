use crate::tools::parts::counted::{Counted, FORCED_LESSONS, SURE_LESSONS, counted_table, sure_of};
use crate::tools::parts::language::{WordRow, Word};
use crate::tools::parts::ordered::{Few, Kept};
use crate::tools::parts::places::{Bit, FormKey, PartKind, Lexicon, Out, Place, WHOLE, laid_out};
use crate::tools::parts::programs::{FoundRun, template, whole_share};
use patterns::{because, source};
use std::cell::Cell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::base::first_sight::NumberTable;
use crate::base::lanes::on_lanes;

pub struct FoundForms;
source!(FoundForms, "forms that are found from the lessons with no list of shapes, in the way of the distributional analysis of Harris and of minimal pairs: two lessons whose texts differ in one run of words show what that run says by what changes in their answers; a run that is read gives the answer of the run that stands in its place in another lesson of the same frame; a form is a run with its read runs as parts, and its program is a program of the language that gives the answers of its runs");

pub const FRAME_MOST: usize = 60;
because!(FRAME_MOST, FoundForms, "the lessons with one frame that are compared with each other: the pairs grow with the square of them, and a frame of so many lessons shows its parts already");

pub const READINGS_MOST: usize = 4;
because!(READINGS_MOST, FoundForms, "the answers that are kept for one run of a text and one kind while forms are found, those made with the fewest forms");

pub const TURNS_MOST: usize = 40;
because!(TURNS_MOST, FoundForms, "the times that the lessons are read again for longer forms at most: each turn adds one level of forms inside forms, and texts of the lessons are far less deep");

pub const SEED_MOST: usize = 6;
because!(SEED_MOST, FoundForms, "the words of a run of the first parts at most: a first part is short, and longer runs are parts made of parts");

pub const KINDS_MOST: usize = 4;
because!(KINDS_MOST, FoundForms, "the kinds of parts at most: a language of lessons has few sorts of parts, and each kind costs one more growth of the forms");

const START_SHARE: f64 = 0.1;
because!(START_SHARE, FoundForms, "a first part starts a new kind only when it stands in this share of the frames of the first kind or more: a run of few frames is a part of an old kind seen rarely");

const MINOR_SHARE: f64 = 0.2;
because!(MINOR_SHARE, FoundForms, "an answer of a form that fewer of the runs of its text give is not kept as a second program of the text");

pub const TEXT_MOST: usize = 30;
because!(TEXT_MOST, FoundForms, "the words of the middle text of the lessons at most: with longer texts the found forms are not tried, because the runs of a text grow with the square of its words");

pub const FOUND_LESSONS: usize = 1000;
because!(FOUND_LESSONS, FoundForms, "the lessons that the found forms learn from at most, the first ones: enough for every kind of the measure, and the cost of the frames grows with them");

const FRAME_TEXT_MOST: usize = 60;
because!(FRAME_TEXT_MOST, FoundForms, "the words of a lesson at most that is compared with others for frames: a longer text has too many runs");

pub const CELLS_MOST: u64 = 4000000;
because!(CELLS_MOST, FoundForms, "the runs of texts that are read while no form is found; with forms found the limit is ten times as many: a kind that is not for this way is left after that much work");

pub const CELLS_WITH_FORMS: u64 = 40000000;
because!(CELLS_WITH_FORMS, FoundForms, "the runs of texts that are read at most once forms are found: ten times the limit with no form, since forms that grow are work that pays");

pub const TRIAL_LESSONS: usize = 300;
because!(TRIAL_LESSONS, FoundForms, "the lessons that are read to compare the three ways to the first parts: enough to tell a way that reads from one that does not");

const SAMPLE_LESSONS: usize = 300;
because!(SAMPLE_LESSONS, FoundForms, "the lessons of a first look of the way from pairs, with more than twice as many lessons: when they show no first part, all lessons show none");

const SEVERAL_WORDS: usize = 2;
because!(SEVERAL_WORDS, FoundForms, "an answer of this many words or more is made of parts: an answer of one word has no order of parts to find");

const IN_A_ROW: usize = 2;
because!(IN_A_ROW, FoundForms, "the single parts that stand next to each other when a lesson holds single parts in a row");

pub const CLASS_MOST: usize = 20;
because!(CLASS_MOST, FoundForms, "the different answers at most of the lessons of a class: with more of them the answer is built, not chosen, so the class of what is left is not tried and the found forms are");

const SAMPLE_FROM: usize = 600;
because!(SAMPLE_FROM, FoundForms, "the lessons from which the way from pairs takes a first look at a sample of them: with more than twice the lessons of the sample a look saves work");

const MIDDLE_PARTS: usize = 2;
because!(MIDDLE_PARTS, FoundForms, "the middle of the sorted lengths of the texts is the place half way: the number of lengths divided by this");

const RUN_PAIRS: usize = 2;
because!(RUN_PAIRS, FoundForms, "the runs of a text are counted as its words times its words and one more, divided by this: the number of its runs from one word to all");

pub const SURE_TO_START: usize = 3;
because!(SURE_TO_START, FoundForms, "the frames that a first part must stand in to start a kind at least: one more than the least lessons of a thing that the lessons cannot tell apart");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    Pairs,
    Table,
    Rows,
}
because!(Start, FoundForms, "the three ways to the first parts: from pairs of lessons; from the lessons of one word or the words of the counted table, each word a part of its own; the same with all single parts in a row as one part");

pub type Member = (usize, usize, usize);
because!(Member, FoundForms, "a run of a lesson: the number of the lesson and the first and the end place of the run");

pub type AnswerWay = (Vec<WordRow>, usize);
because!(AnswerWay, FoundForms, "a way to take the answer of a part out of the answer of its lesson: the runs of fixed answer words round the places of the part, and their length");

pub type ChartCell = Few<PartKind, Few<WordRow, u32>>;
because!(ChartCell, FoundForms, "the readings of one run of a text: for each kind, the answers in the order of finding with the forms that each used");

#[derive(Clone, Debug, Default)]
pub struct Cells<T> {
    n: usize,
    cells: Vec<Option<T>>,
}
because!(Cells, FoundForms, "what is found for every run of a text, by the first and the end place of the run, in one flat row; in_order walks the runs as a chart parser finds them, the short runs first and from the left");

pub type Chart = Cells<ChartCell>;
because!(Chart, FoundForms, "the readings of every run of a text that is read as a part");

impl<T: Clone> Cells<T> {
    pub fn new(n: usize) -> Cells<T> {
        Cells { n, cells: vec![None; (n + 1) * (n + 1)] }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.cells.get(x * (self.n + 1) + y).and_then(Option::as_ref)
    }

    pub fn set(&mut self, x: usize, y: usize, cell: T) {
        let place = x * (self.n + 1) + y;
        self.cells[place] = Some(cell);
    }

    pub fn in_order(&self) -> impl Iterator<Item = (usize, usize, &T)> {
        let n = self.n;
        (1..=n).flat_map(move |size| (0..=n - size).map(move |x| (x, x + size))).filter_map(move |(x, y)| self.get(x, y).map(|cell| (x, y, cell)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CutWay {
    pub pattern: Vec<Bit>,
    pub rows: Vec<WordRow>,
    pub words: Vec<Vec<Word>>,
}
because!(CutWay, FoundForms, "a run of a text as the text of a form: its fixed words and kinds of parts, the answers of the parts, and the words of the parts");

pub type RunValue = (Vec<WordRow>, WordRow, Vec<Vec<Word>>);
because!(RunValue, FoundForms, "one run of the text of a form: the answers of its parts, its answer, and the words of its parts");

pub struct Indexed {
    pub form: Arc<FormKey>,
    pub fixed: Vec<Word>,
}
because!(Indexed, FoundForms, "a form of a part with its fixed words, for the question whether a text holds them all");

pub struct WholeForm {
    pub form: Arc<FormKey>,
    pub fixed: Vec<Word>,
    pub count: usize,
}
because!(WholeForm, FoundForms, "a form of a whole text with its fixed words and the runs that gave it");

#[derive(Clone)]
pub struct Grouped {
    pub text: Vec<Bit>,
    pub kind: PartKind,
    pub programs: Arc<Vec<Out>>,
    pub fixed: Vec<Word>,
}
because!(Grouped, FoundForms, "a text of a form with every program that it keeps, as a form of one kind of its parts, while the second and the third way learn");

#[derive(Clone, Debug)]
pub struct Contribution {
    pub kind: PartKind,
    pub row: WordRow,
    pub ways: Vec<CutWay>,
}
because!(Contribution, FoundForms, "what a run that pairs of lessons show gives: its kind, its answer, and the ways to read it as the text of a form");

#[derive(Default)]
pub struct FrameWays {
    pub support: Kept<AnswerWay, HashSet<(usize, PartKind)>>,
    pub applied: HashMap<AnswerWay, HashSet<(usize, PartKind)>>,
    pub answers: HashMap<AnswerWay, Vec<Option<WordRow>>>,
}
because!(FrameWays, FoundForms, "for one frame: each way of taking the answer of a run out of its lesson with the read runs that show it, the read runs whose way was used already, and the answer that each run of the frame gets in each way");

pub struct Work {
    pub lessons: Vec<(Vec<Word>, WordRow)>,
    pub owner: HashMap<Word, HashSet<Word>>,
    pub with_word: HashMap<Word, Vec<usize>>,
    pub frames: Vec<Vec<Member>>,
    pub frame_of: HashMap<Member, usize>,
    pub charts: Vec<Chart>,
    pub dirty: BTreeSet<usize>,
    pub shown: HashMap<Member, Kept<(PartKind, WordRow), usize>>,
    pub done: HashMap<Member, HashSet<(PartKind, WordRow)>>,
    pub contrib: Kept<Member, Option<Contribution>>,
    pub whole_contrib: Kept<usize, Vec<CutWay>>,
    pub keys_of: HashMap<usize, HashSet<Member>>,
    pub cache: HashMap<(Vec<Bit>, PartKind), (HashMap<Vec<Word>, RunValue>, Vec<(Vec<Bit>, Out, usize)>)>,
    pub ways: HashMap<usize, FrameWays>,
}
because!(Work, FoundForms, "what the found forms need while they learn and drop after: the lessons, the text words that say each answer word, the lessons of each word, the frames, the charts of the lessons, the lessons to read again, what pairs of lessons showed for each run, and what was found for each frame and each text of a form");

pub struct FormsFound {
    pub lessons: usize,
    pub mode: Start,
    pub rows_on: bool,
    pub in_a_row: bool,
    pub signs: bool,
    pub open_kind: Option<PartKind>,
    pub single: Kept<Vec<Word>, (PartKind, WordRow)>,
    pub forms: Kept<Arc<FormKey>, usize>,
    pub single_most: usize,
    pub views: bool,
    pub turns: usize,
    pub kinds: usize,
    pub seen: NumberTable<(Top, usize, Top), usize>,
    pub places_seen: NumberTable<(Top, usize), usize>,
    pub whole_seen: NumberTable<Top, usize>,
    pub lex_seen: NumberTable<Vec<Word>, usize>,
    pub lex_all: HashMap<PartKind, usize>,
    pub open_words: HashMap<Vec<Bit>, HashSet<Word>>,
    pub known_words: Option<HashSet<Word>>,
    pub right: usize,
    pub cells: AtomicU64,
    pub fit: bool,
    pub reads_right: usize,
    pub by_first: Kept<Option<Word>, Vec<Indexed>>,
    pub wholes: Vec<WholeForm>,
    pub grouped: Kept<Option<Word>, Vec<Grouped>>,
    pub lanes: usize,
}
because!(FormsFound, FoundForms, "the forms found from the lessons: the single parts (a run of words says a run of answer words), the forms with their programs and the runs that gave them, the kind of a word that the model does not know, the words that an open place took, and the counts of which form stood in which place of which form in the readings of the lessons, which are the costs of a reading; read gives the answer of a whole text by its cheapest reading");

#[derive(Clone, Debug)]
pub enum Top {
    Form(Arc<FormKey>),
    Single(PartKind),
    InRow(PartKind),
    Open,
}
because!(Top, FoundForms, "what made the reading of a run: a form, a single part of a kind, a row of single parts of a kind, or an open place. Two forms are the same when they are the one shared value of the model: a form is put into the model once and stays that value while it is there, so the place in memory says which form it is, and a count by form hashes a number and not a text with its program");

impl PartialEq for Top {
    fn eq(&self, other: &Top) -> bool {
        match (self, other) {
            (Top::Form(one), Top::Form(two)) => Arc::ptr_eq(one, two),
            (Top::Single(one), Top::Single(two)) | (Top::InRow(one), Top::InRow(two)) => one == two,
            (Top::Open, Top::Open) => true,
            _ => false,
        }
    }
}

impl Eq for Top {}

impl Hash for Top {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Top::Form(form) => (Arc::as_ptr(form) as usize).hash(state),
            Top::Single(kind) | Top::InRow(kind) => kind.hash(state),
            Top::Open => {}
        }
    }
}

pub fn trim(one: &[Word], other: &[Word]) -> (usize, usize) {
    let most = one.len().min(other.len());
    let mut start = 0;
    while start < most && one[start] == other[start] {
        start += 1;
    }
    let mut end = 0;
    while end < most - start && one[one.len() - 1 - end] == other[other.len() - 1 - end] {
        end += 1;
    }
    (start, end)
}
because!(trim, FoundForms, "the words that two rows share at the start and the words that they share at the end, the two runs not overlapping");

pub fn find_run(row: &[Word], run: &[Word], start: usize) -> Option<usize> {
    let size = run.len();
    if row.len() < size {
        return None;
    }
    (start..=row.len() - size).find(|place| row[*place..*place + size] == *run)
}
because!(find_run, FoundForms, "the first place of a run in a row from a start, or nothing; an empty run stands at the start");

pub fn holds_run(row: &[Word], run: &[Word]) -> bool {
    find_run(row, run, 0).is_some()
}
because!(holds_run, FoundForms, "whether a run stands in a row");

pub fn cuts(answer: &[Word], row: &[Word]) -> Vec<AnswerWay> {
    let mut places: Vec<usize> = Vec::new();
    let mut found = find_run(answer, row, 0);
    while let Some(place) = found {
        places.push(place);
        found = find_run(answer, row, place + 1);
    }
    let mut ways: Vec<AnswerWay> = places.iter().map(|place| (vec![answer[..*place].to_vec(), answer[place + row.len()..].to_vec()], answer.len() - row.len())).collect();
    let mut apart: Vec<usize> = Vec::new();
    let mut last = 0;
    for place in &places {
        if apart.is_empty() || *place >= last {
            apart.push(*place);
            last = place + row.len();
        }
    }
    if apart.len() > 1 {
        let mut fixed: Vec<WordRow> = Vec::new();
        let mut at = 0;
        for place in &apart {
            fixed.push(answer[at..*place].to_vec());
            at = place + row.len();
        }
        fixed.push(answer[at..].to_vec());
        ways.push((fixed, answer.len() - row.len() * apart.len()));
    }
    ways
}
because!(cuts, FoundForms, "the ways to take the answer of a part out of the answer of its lesson: each single place of the part, and all places together when it stands several times and the places do not overlap");

pub fn solved(answer: &[Word], fixed: &[WordRow], total: usize) -> Option<WordRow> {
    if answer.len() < total {
        return None;
    }
    let gaps = fixed.len() - 1;
    let (size, rest) = ((answer.len() - total) / gaps, (answer.len() - total) % gaps);
    if rest != 0 || size < 1 {
        return None;
    }
    let first = fixed[0].len();
    if answer[..first] != *fixed[0] {
        return None;
    }
    let run: WordRow = answer[first..first + size].to_vec();
    let mut at = first + size;
    for piece in &fixed[1..fixed.len() - 1] {
        if answer.len() < at + piece.len() + size || answer[at..at + piece.len()] != **piece || answer[at + piece.len()..at + piece.len() + size] != *run {
            return None;
        }
        at += piece.len() + size;
    }
    (answer[at..] == *fixed[fixed.len() - 1]).then_some(run)
}
because!(solved, FoundForms, "the run that stands in every place between the fixed runs of an answer, or nothing");

fn intersected(lists: &HashMap<Word, Vec<usize>>, words: impl Iterator<Item = Word>) -> Option<Vec<usize>> {
    let mut found: Option<Vec<usize>> = None;
    for word in words {
        let mine = lists.get(&word).map(Vec::as_slice).unwrap_or_default();
        found = Some(match found {
            None => mine.to_vec(),
            Some(held) => held.into_iter().filter(|lesson| mine.binary_search(lesson).is_ok()).collect(),
        });
    }
    found
}
because!(intersected, FoundForms, "the lessons that hold every one of some words, in their order; nothing when no word is given");

pub fn fits(lessons: &[(Vec<Word>, WordRow)]) -> bool {
    if lessons.is_empty() {
        return false;
    }
    let mut lengths: Vec<usize> = lessons.iter().map(|(text, _)| text.len()).collect();
    lengths.sort_unstable();
    let answers: HashSet<&WordRow> = lessons.iter().map(|(_, answer)| answer).collect();
    let longer = lessons.iter().filter(|(_, answer)| answer.len() >= SEVERAL_WORDS).count();
    answers.len() > CLASS_MOST && whole_share(longer, lessons.len()) && lengths[lengths.len() / MIDDLE_PARTS] <= TEXT_MOST
}
because!(fits, FoundForms, "whether this way is for these lessons at all: more different answers than a class has, an answer of two words or more in more lessons than a set holds by error, and texts that are not long");

impl FormsFound {
    pub fn new(lessons: &[(Vec<Word>, WordRow)], mode: Start, counted: &Counted, lexicon: &Lexicon, searched: &Cell<u64>, lanes: usize) -> FormsFound {
        let mut found = FormsFound {
            lessons: lessons.len(),
            mode,
            rows_on: mode == Start::Rows,
            in_a_row: true,
            signs: false,
            open_kind: None,
            single: Kept::default(),
            forms: Kept::default(),
            single_most: 0,
            views: false,
            turns: 0,
            kinds: 0,
            seen: NumberTable::default(),
            places_seen: NumberTable::default(),
            whole_seen: NumberTable::default(),
            lex_seen: NumberTable::default(),
            lex_all: HashMap::new(),
            open_words: HashMap::new(),
            known_words: None,
            right: 0,
            cells: AtomicU64::new(0),
            fit: false,
            reads_right: 0,
            by_first: Kept::default(),
            wholes: Vec::new(),
            grouped: Kept::default(),
            lanes,
        };
        found.fit = fits(lessons);
        if found.fit {
            found.learn(lessons, counted, lexicon, searched);
            if !found.forms.is_empty() {
                found.nestings(lessons);
            }
        }
        found
    }

    pub fn cells(&self) -> u64 {
        self.cells.load(Ordering::Relaxed)
    }

    pub fn count_cells(&self, words: usize) {
        self.cells.fetch_add((words * (words + 1) / RUN_PAIRS) as u64, Ordering::Relaxed);
    }

    pub fn index(&mut self) {
        self.by_first = Kept::default();
        self.wholes = Vec::new();
        for (form, count) in self.forms.pairs() {
            let mut fixed: Vec<Word> = Vec::new();
            for word in form.pattern.iter().filter_map(Bit::word) {
                if !fixed.contains(&word) {
                    fixed.push(word);
                }
            }
            if form.kind == WHOLE {
                self.wholes.push(WholeForm { form: form.clone(), fixed, count: *count });
            } else {
                let first = form.pattern.first().and_then(Bit::word);
                self.by_first.or_put(first, Vec::new).push(Indexed { form: form.clone(), fixed });
            }
        }
    }

    pub fn usable(&self, words: &HashSet<Word>) -> HashMap<Option<Word>, Vec<&Indexed>> {
        self.by_first
            .pairs()
            .filter(|(first, _)| first.is_none_or(|word| words.contains(&word)))
            .map(|(first, forms)| (*first, forms.iter().filter(|form| form.fixed.iter().all(|word| words.contains(word))).collect()))
            .collect()
    }

    fn banned(&self, work: &Work, pattern: &[Bit]) -> HashSet<Word> {
        work.owner.iter().filter(|(_, words)| !words.iter().any(|word| pattern.contains(&Bit::Word(*word)))).map(|(answer, _)| *answer).collect()
    }

    fn lessons_with(&self, work: &Work, run: &[Word]) -> Vec<usize> {
        let mut words: Vec<Word> = Vec::new();
        for word in run {
            if !words.contains(word) {
                words.push(*word);
            }
        }
        intersected(&work.with_word, words.into_iter()).unwrap_or_default().into_iter().filter(|lesson| holds_run(&work.lessons[*lesson].0, run)).collect()
    }

    fn stands(&self, work: &Work, run: &[Word], row: &[Word], least: usize) -> bool {
        let there = self.lessons_with(work, run);
        let holding = there.iter().filter(|lesson| holds_run(&work.lessons[**lesson].1, row)).count();
        there.len() >= least && sure_of(holding, there.len())
    }

    fn add_single(&mut self, work: &mut Work, run: Vec<Word>, kind: PartKind, row: WordRow) {
        self.single_most = self.single_most.max(run.len());
        let lessons = self.lessons_with(work, &run);
        work.dirty.extend(lessons);
        self.single.put(run, (kind, row));
    }

    fn learn(&mut self, lessons: &[(Vec<Word>, WordRow)], counted: &Counted, lexicon: &Lexicon, searched: &Cell<u64>) {
        let mut owner: HashMap<Word, HashSet<Word>> = HashMap::new();
        for (word, row) in &counted.table {
            for answer in row {
                owner.entry(*answer).or_default().insert(*word);
            }
        }
        let mut with_word: HashMap<Word, Vec<usize>> = HashMap::new();
        for (number, (text, _)) in lessons.iter().enumerate() {
            for word in text {
                let list = with_word.entry(*word).or_default();
                if list.last() != Some(&number) {
                    list.push(number);
                }
            }
        }
        let mut work = Work {
            lessons: lessons.to_vec(),
            owner,
            with_word,
            frames: Vec::new(),
            frame_of: HashMap::new(),
            charts: vec![Chart::default(); lessons.len()],
            dirty: BTreeSet::new(),
            shown: HashMap::new(),
            done: HashMap::new(),
            contrib: Kept::default(),
            whole_contrib: Kept::default(),
            keys_of: HashMap::new(),
            cache: HashMap::new(),
            ways: HashMap::new(),
        };
        let mut firsts: (Vec<Vec<Member>>, Kept<Vec<Word>, WordRow>, HashMap<Vec<Word>, usize>) = (Vec::new(), Kept::default(), HashMap::new());
        if self.mode == Start::Pairs {
            if lessons.len() > SAMPLE_FROM {
                firsts = self.first_parts(&work, 0..SAMPLE_LESSONS);
            }
            if !firsts.1.is_empty() || lessons.len() <= SAMPLE_FROM {
                firsts = self.first_parts(&work, 0..lessons.len());
            }
        }
        let (frames, said, in_frames) = firsts;
        for (number, members) in frames.iter().enumerate() {
            for member in members {
                work.frame_of.insert(*member, number);
            }
        }
        work.frames = frames;
        let mut starts: Vec<&Vec<Word>> = said.keys().collect();
        starts.sort_by(|one, other| in_frames[*other].cmp(&in_frames[*one]).then(one.len().cmp(&other.len())).then_with(|| lexicon.cmp_rows(one, other)));
        for kind in 1..=KINDS_MOST {
            let start = starts.iter().find(|run| !self.single.holds(run) && self.chart(run, None, None).cells.iter().all(Option::is_none)).copied();
            let Some(start) = start else { break };
            let top = starts.first().map_or(0, |first| in_frames[*first]);
            let floor = SURE_TO_START as f64;
            let share = START_SHARE * (top as f64);
            let bar = if floor >= share { floor } else { share };
            if (in_frames[start] as f64) < bar {
                break;
            }
            let row = said.get(start).cloned().unwrap_or_default();
            self.add_single(&mut work, start.clone(), kind as PartKind, row);
            self.kinds = kind;
            self.grow(&mut work, lexicon, searched);
        }
        let mut alone: Kept<Word, WordRow> = Kept::default();
        for (text, answer) in lessons {
            if text.len() == 1 && !answer.is_empty() {
                alone.put(text[0], answer.clone());
            }
        }
        let mut table: Vec<(Word, WordRow)> = counted.table.iter().map(|(word, row)| (*word, row.clone())).collect();
        table.sort_by(|one, other| lexicon.cmp_word(one.0, other.0));
        if self.mode != Start::Pairs && alone.len() >= FORCED_LESSONS {
            table = alone.pairs().map(|(word, row)| (*word, row.clone())).collect();
            work.owner = HashMap::new();
            for (word, row) in &table {
                for answer in row {
                    work.owner.entry(*answer).or_default().insert(*word);
                }
            }
        }
        if self.mode != Start::Pairs && table.iter().any(|(_, row)| !row.is_empty()) {
            self.views = true;
            self.kinds = 1;
            for (word, row) in &table {
                if !row.is_empty() {
                    self.single.put(vec![*word], (1, row.clone()));
                }
            }
            self.single_most = 1;
            self.in_a_row = lessons.iter().any(|(text, _)| text.windows(IN_A_ROW).any(|pair| self.single.find(&pair[..1]).is_some() && self.single.find(&pair[1..]).is_some()));
            self.grow_table(&mut work, lexicon, searched);
        }
        self.open_places();
        self.reduce(lexicon);
        let mut known: HashSet<Word> = self.forms.keys().flat_map(|form| form.pattern.iter().filter_map(Bit::word)).collect();
        known.extend(self.single.keys().flatten().copied());
        self.known_words = Some(known);
        let mut same: HashMap<PartKind, (usize, usize)> = HashMap::new();
        for (run, (kind, row)) in self.single.pairs() {
            if run.len() == 1 {
                let counts = same.entry(*kind).or_default();
                counts.0 += 1;
                counts.1 += usize::from(row == run);
            }
        }
        self.open_kind = same.iter().filter(|(_, (count, itself))| *count >= FORCED_LESSONS && sure_of(*itself, *count)).map(|(kind, (count, _))| (*count, *kind)).max().map(|(_, kind)| kind);
    }

    fn first_parts(&self, work: &Work, some: std::ops::Range<usize>) -> (Vec<Vec<Member>>, Kept<Vec<Word>, WordRow>, HashMap<Vec<Word>, usize>) {
        let lessons = &work.lessons;
        let mut frames: Kept<(usize, usize), Vec<Member>> = Kept::default();
        let mut starts: HashMap<(usize, Word), usize> = HashMap::new();
        let mut ends: HashMap<(usize, Word), usize> = HashMap::new();
        for lesson in some {
            let text = &lessons[lesson].0;
            let n = text.len();
            if n > FRAME_TEXT_MOST {
                continue;
            }
            let mut before = vec![0usize];
            for word in text {
                let next = starts.len() + 1;
                let last = *before.last().unwrap_or(&0);
                before.push(*starts.entry((last, *word)).or_insert(next));
            }
            let mut after = vec![0usize];
            for word in text.iter().rev() {
                let next = ends.len() + 1;
                let last = *after.last().unwrap_or(&0);
                after.push(*ends.entry((last, *word)).or_insert(next));
            }
            for x in 0..n {
                for y in x + 1..=n {
                    if y - x < n {
                        frames.or_put((before[x], after[n - y]), Vec::new).push((lesson, x, y));
                    }
                }
            }
            if n <= SEED_MOST {
                frames.or_put((0, 0), Vec::new).push((lesson, 0, n));
            }
        }
        if let Some(short) = frames.get_mut(&(0, 0)) {
            short.sort_by_key(|member| (member.2, member.0));
        }
        let frames: Vec<Vec<Member>> = frames.into_pairs().filter(|(_, members)| members.len() > 1).map(|(_, mut members)| {
            members.truncate(FRAME_MOST);
            members
        }).collect();
        let mut seen_runs: Kept<Vec<Word>, Kept<WordRow, usize>> = Kept::default();
        let mut in_frames: HashMap<Vec<Word>, usize> = HashMap::new();
        for members in &frames {
            let mut here: HashSet<Vec<Word>> = HashSet::new();
            for (place, (lesson, x, y)) in members.iter().enumerate() {
                if y - x > SEED_MOST {
                    continue;
                }
                let one = &lessons[*lesson].0[*x..*y];
                for (other, x2, y2) in &members[place + 1..] {
                    let two = &lessons[*other].0[*x2..*y2];
                    if y2 - x2 > SEED_MOST || one[0] == two[0] || one[one.len() - 1] == two[two.len() - 1] {
                        continue;
                    }
                    let (first, second) = (&lessons[*lesson].1, &lessons[*other].1);
                    let (start, end) = trim(first, second);
                    seen_runs.or_put(one.to_vec(), Kept::default).count(first[start..first.len() - end].to_vec(), 1);
                    seen_runs.or_put(two.to_vec(), Kept::default).count(second[start..second.len() - end].to_vec(), 1);
                    here.insert(one.to_vec());
                    here.insert(two.to_vec());
                }
            }
            for run in here {
                *in_frames.entry(run).or_default() += 1;
            }
        }
        let mut said: Kept<Vec<Word>, WordRow> = Kept::default();
        for (run, answers) in seen_runs.pairs() {
            let mut order: Vec<(&WordRow, usize)> = answers.pairs().map(|(answer, count)| (answer, *count)).collect();
            order.sort_by_key(|(answer, count)| (std::cmp::Reverse(answer.len()), std::cmp::Reverse(*count)));
            for (answer, count) in order {
                if !answer.is_empty() && count >= FORCED_LESSONS && self.stands(work, run, answer, FORCED_LESSONS) {
                    said.put(run.clone(), answer.clone());
                    break;
                }
            }
        }
        (frames, said, in_frames)
    }

    fn contribution(&self, work: &Work, key: Member) -> Option<Contribution> {
        let (lesson, x, y) = key;
        let ((kind, row), _) = work.shown.get(&key)?.most().map(|(pair, count)| (pair.clone(), count))?;
        let chart = &work.charts[lesson];
        let text = &work.lessons[lesson].0;
        if chart.get(x, y).is_some_and(|cell| cell.iter().any(|(_, rows)| rows.holds(&row))) {
            if self.single.find(&text[x..y]).map(|(_, held)| held) != Some(&row) {
                return None;
            }
            let ways: Vec<CutWay> = self.cut(text, chart, x, y, Some(&row)).into_iter().filter(|way| !way.rows.is_empty()).collect();
            return (!ways.is_empty()).then_some(Contribution { kind, row, ways });
        }
        let ways = self.cut(text, chart, x, y, Some(&row));
        Some(Contribution { kind, row, ways })
    }

    pub fn cut(&self, text: &[Word], chart: &Chart, x: usize, y: usize, within: Option<&WordRow>) -> Vec<CutWay> {
        let mut found = vec![CutWay { pattern: Vec::new(), rows: Vec::new(), words: Vec::new() }];
        let mut at = x;
        while at < y {
            let mut moved = false;
            for end in (at + 1..=y).rev() {
                if (at, end) == (x, y) {
                    continue;
                }
                let Some(cell) = chart.get(at, end) else { continue };
                let options: Vec<(PartKind, &WordRow)> = cell.iter().flat_map(|(kind, rows)| rows.iter().map(move |(row, _)| (*kind, row))).filter(|(_, row)| within.is_none_or(|within| holds_run(within, row))).collect();
                if options.is_empty() {
                    continue;
                }
                let mut next: Vec<CutWay> = Vec::new();
                'all: for way in &found {
                    for (kind, row) in &options {
                        if next.len() >= CUTS_MOST {
                            break 'all;
                        }
                        let mut longer = way.clone();
                        longer.pattern.push(Bit::Part(*kind));
                        longer.rows.push((*row).clone());
                        longer.words.push(text[at..end].to_vec());
                        next.push(longer);
                    }
                }
                found = next;
                at = end;
                moved = true;
                break;
            }
            if !moved {
                for way in &mut found {
                    way.pattern.push(Bit::Word(text[at]));
                }
                at += 1;
            }
        }
        found
    }

    fn grow(&mut self, work: &mut Work, lexicon: &Lexicon, searched: &Cell<u64>) {
        for _ in 0..TURNS_MOST {
            let limit = if self.forms.is_empty() { CELLS_MOST } else { CELLS_WITH_FORMS };
            if self.cells() > limit {
                break;
            }
            let dirty: Vec<usize> = std::mem::take(&mut work.dirty).into_iter().collect();
            let made: Vec<Chart> = {
                let (me, lessons) = (&*self, &work.lessons);
                on_lanes(dirty.len(), self.lanes, &|piece| me.chart(&lessons[dirty[piece]].0, Some(&lessons[dirty[piece]].1), None))
            };
            for (lesson, chart) in dirty.iter().zip(made) {
                work.charts[*lesson] = chart;
            }
            let mut touched: HashSet<Member> = HashSet::new();
            let mut hot: Kept<usize, ()> = Kept::default();
            for lesson in &dirty {
                let row_of = work.lessons[*lesson].1.clone();
                for (x, y, readings) in work.charts[*lesson].in_order() {
                    let Some(frame) = work.frame_of.get(&(*lesson, x, y)).copied() else { continue };
                    let done = work.done.entry((*lesson, x, y)).or_default();
                    for (kind, rows) in readings.iter() {
                        for (row, _) in rows.iter() {
                            if !done.insert((*kind, row.clone())) {
                                continue;
                            }
                            let state = work.ways.entry(frame).or_default();
                            for way in cuts(&row_of, row) {
                                state.support.or_put(way, HashSet::new).insert((*lesson, *kind));
                            }
                            hot.put(frame, ());
                        }
                    }
                }
            }
            for frame in hot.keys() {
                let Some(state) = work.ways.get_mut(frame) else { continue };
                if state.support.is_empty() {
                    continue;
                }
                let members = &work.frames[*frame];
                for way in state.support.keys() {
                    if !state.answers.contains_key(way) {
                        let answers = members.iter().map(|(lesson, _, _)| solved(&work.lessons[*lesson].1, &way.0, way.1)).collect();
                        state.answers.insert(way.clone(), answers);
                    }
                }
                let answers = &state.answers;
                let Some((best, _)) = state.support.first_most(|way, shown| (shown.len(), answers[way].iter().filter(|answer| answer.is_some()).count(), way.0.len())) else { continue };
                let best = best.clone();
                let gave = state.applied.entry(best.clone()).or_default();
                let mut fresh: Vec<(usize, PartKind)> = state.support.get(&best).map(|shown| shown.difference(gave).copied().collect()).unwrap_or_default();
                fresh.sort_unstable();
                for (lesson, kind) in fresh {
                    gave.insert((lesson, kind));
                    for (member, answer) in members.iter().zip(&state.answers[&best]) {
                        if member.0 != lesson {
                            if let Some(answer) = answer.as_ref().filter(|answer| !answer.is_empty()) {
                                work.shown.entry(*member).or_default().count((kind, answer.clone()), 1);
                                touched.insert(*member);
                            }
                        }
                    }
                }
            }
            for member in &touched {
                work.keys_of.entry(member.0).or_default().insert(*member);
            }
            let mut again: HashSet<Member> = touched;
            for lesson in &dirty {
                if let Some(keys) = work.keys_of.get(lesson) {
                    again.extend(keys.iter().copied());
                }
            }
            let mut again: Vec<Member> = again.into_iter().collect();
            again.sort_unstable();
            for key in again {
                let found = self.contribution(work, key);
                work.contrib.put(key, found);
            }
            for lesson in &dirty {
                let (text, answer) = &work.lessons[*lesson];
                let chart = &work.charts[*lesson];
                let mut found: Vec<CutWay> = self.cut(text, chart, 0, text.len(), None).into_iter().filter(|way| !way.rows.is_empty()).collect();
                if let Some(cell) = chart.get(0, text.len()) {
                    for (kind, rows) in cell.iter() {
                        if rows.holds(answer) {
                            found.push(CutWay { pattern: vec![Bit::Part(*kind)], rows: vec![answer.clone()], words: vec![text.clone()] });
                        }
                    }
                }
                work.whole_contrib.put(*lesson, found);
            }
            let mut singles: Kept<Vec<Word>, Kept<(PartKind, WordRow), HashSet<usize>>> = Kept::default();
            let mut runs: Kept<(Vec<Bit>, PartKind), Kept<Vec<Word>, RunValue>> = Kept::default();
            for ((lesson, x, y), contribution) in work.contrib.pairs() {
                let Some(contribution) = contribution else { continue };
                let text = &work.lessons[*lesson].0;
                for way in &contribution.ways {
                    if way.rows.is_empty() {
                        singles.or_put(text[*x..*y].to_vec(), Kept::default).or_put((contribution.kind, contribution.row.clone()), HashSet::new).insert(*lesson);
                    } else if y - x < text.len() {
                        runs.or_put((way.pattern.clone(), contribution.kind), Kept::default).or_put(text[*x..*y].to_vec(), || (way.rows.clone(), contribution.row.clone(), way.words.clone()));
                    }
                }
            }
            for (lesson, found) in work.whole_contrib.pairs() {
                let (text, answer) = &work.lessons[*lesson];
                for way in found {
                    runs.or_put((way.pattern.clone(), WHOLE), Kept::default).or_put(text.clone(), || (way.rows.clone(), answer.clone(), way.words.clone()));
                }
            }
            let mut new = 0;
            let mut ready: Vec<(Vec<Word>, PartKind, WordRow)> = Vec::new();
            for (run, found) in singles.pairs() {
                if self.single.holds(run) {
                    continue;
                }
                let Some(((kind, answer), there)) = found.first_most(|_, there| there.len()) else { continue };
                let alone = there.iter().any(|lesson| work.lessons[*lesson].0 == *run);
                let total: usize = found.values().map(HashSet::len).sum();
                let owned = answer.iter().all(|word| work.owner.get(word).is_none_or(|words| words.iter().any(|owner| run.contains(owner))));
                if (there.len() >= FORCED_LESSONS || alone) && sure_of(there.len(), total) && self.stands(work, run, answer, 1) && owned {
                    ready.push((run.clone(), *kind, answer.clone()));
                }
            }
            let shortest = ready.iter().map(|(run, _, _)| run.len()).min().unwrap_or_default();
            for (run, kind, answer) in ready {
                if run.len() == shortest {
                    self.add_single(work, run, kind, answer);
                    new += 1;
                }
            }
            if new > 0 {
                self.turns += 1;
                continue;
            }
            let mut fresh: Vec<(Vec<Bit>, PartKind, Out, usize)> = Vec::new();
            for ((pattern, kind), found) in runs.pairs() {
                if found.len() < FORCED_LESSONS {
                    continue;
                }
                let content: HashMap<Vec<Word>, RunValue> = found.pairs().map(|(run, value)| (run.clone(), value.clone())).collect();
                let key = (pattern.clone(), *kind);
                if work.cache.get(&key).is_none_or(|(mark, _)| *mark != content) {
                    let items: Vec<FoundRun> = found.values().map(|(rows, target, _)| (rows.clone(), target.clone())).collect();
                    let banned = self.banned(work, pattern);
                    let mut outs: Vec<(Vec<Bit>, Out, usize)> = Vec::new();
                    let mut left: Vec<FoundRun> = items.clone();
                    while left.len() >= FORCED_LESSONS {
                        let (out, explained) = self.program(&left, &banned, lexicon, searched);
                        let Some(out) = out else { break };
                        if explained.len() < FORCED_LESSONS || (explained.len() as f64) < MINOR_SHARE * (items.len() as f64) {
                            break;
                        }
                        outs.push((pattern.clone(), out, explained.len()));
                        left = left.into_iter().enumerate().filter(|(number, _)| explained.binary_search(number).is_err()).map(|(_, item)| item).collect();
                    }
                    work.cache.insert(key.clone(), (content, outs));
                }
                for (text, out, count) in &work.cache[&key].1 {
                    let form = FormKey { pattern: text.clone(), kind: *kind, out: out.clone() };
                    if !self.forms.holds(&Arc::new(form)) {
                        fresh.push((text.clone(), *kind, out.clone(), *count));
                    }
                }
            }
            for (pattern, kind, out, count) in fresh {
                let form = Arc::new(FormKey { pattern, kind, out });
                if self.forms.holds(&form) {
                    continue;
                }
                let there = intersected(&work.with_word, form.pattern.iter().filter_map(Bit::word));
                self.forms.put(form, count);
                new += 1;
                match there {
                    Some(lessons) => work.dirty.extend(lessons),
                    None => work.dirty.extend(0..work.lessons.len()),
                }
            }
            if new > 0 {
                self.index();
            }
            self.turns += 1;
            if new == 0 {
                break;
            }
        }
    }

    pub fn program(&self, items: &[FoundRun], banned: &HashSet<Word>, lexicon: &Lexicon, searched: &Cell<u64>) -> (Option<Out>, Vec<usize>) {
        let (mut out, mut explained) = template(items, banned, crate::tools::parts::language::ONLY_PLAIN, false, searched, lexicon);
        if self.views && (out.is_none() || !sure_of(explained.len(), items.len())) {
            let (other, more) = template(items, banned, crate::tools::parts::language::VIEWS, false, searched, lexicon);
            if other.is_some() && more.len() > explained.len() {
                (out, explained) = (other, more);
            }
            if out.is_none() || !sure_of(explained.len(), items.len()) {
                let (other, more) = template(items, banned, crate::tools::parts::language::VIEWS, true, searched, lexicon);
                if other.is_some() && more.len() > explained.len() {
                    (out, explained) = (other, more);
                }
            }
        }
        (out, explained)
    }

    fn open_places(&mut self) {
        let mut groups: Kept<(PartKind, Vec<Bit>, Out), Kept<Word, usize>> = Kept::default();
        for (form, count) in self.forms.pairs() {
            for (place, bit) in form.pattern.iter().enumerate() {
                let Bit::Word(word) = bit else { continue };
                if !form.out.contains(&Place::Word(*word)) {
                    continue;
                }
                let part = form.pattern[..place].iter().filter(|bit| bit.kind().is_some()).count();
                let shift = |held: usize| held + usize::from(held >= part);
                let answer: Out = form.out.iter().map(|z| match z {
                    Place::Word(other) if other == word => Place::Part(part),
                    Place::Part(held) => Place::Part(shift(*held)),
                    Place::Seen(held, view) => Place::Seen(shift(*held), *view),
                    Place::Looked(held, view) => Place::Looked(shift(*held), *view),
                    Place::Look(held, view, table) => Place::Look(shift(*held), *view, table.clone()),
                    other => other.clone(),
                }).collect();
                let mut pattern = form.pattern.clone();
                pattern[place] = Bit::Part(crate::tools::parts::places::OPEN_KIND);
                groups.or_put((form.kind, pattern, answer), Kept::default).put(*word, *count);
            }
        }
        for (run, (kind, row)) in self.single.pairs() {
            for (place, word) in run.iter().enumerate() {
                if row.contains(word) && run.len() > 1 {
                    let mut pattern: Vec<Bit> = run.iter().map(|word| Bit::Word(*word)).collect();
                    pattern[place] = Bit::Part(crate::tools::parts::places::OPEN_KIND);
                    let answer: Out = row.iter().map(|z| if z == word { Place::Part(0) } else { Place::Word(*z) }).collect();
                    groups.or_put((*kind, pattern, answer), Kept::default).put(*word, 1);
                }
            }
        }
        for ((kind, pattern, out), words) in groups.into_pairs() {
            let form = Arc::new(FormKey { pattern: pattern.clone(), kind, out });
            if words.len() >= FORCED_LESSONS && !self.forms.holds(&form) {
                self.forms.put(form, words.total());
                self.open_words.entry(pattern).or_default().extend(words.keys().copied());
            }
        }
        self.index();
    }

    fn reduce(&mut self, lexicon: &Lexicon) {
        let mut order: Vec<(Arc<FormKey>, usize)> = self.forms.pairs().map(|(form, count)| (form.clone(), *count)).collect();
        order.sort_by_cached_key(|(form, count)| (std::cmp::Reverse(form.pattern.len()), *count, lexicon.repr_form(form)));
        for (form, count) in order {
            self.forms.take(&form);
            self.index();
            let mut text: Vec<Word> = Vec::new();
            let mut given: Vec<(usize, PartKind, WordRow)> = Vec::new();
            let mut marks: Vec<WordRow> = Vec::new();
            for bit in &form.pattern {
                match bit {
                    Bit::Part(kind) => {
                        let first = marks.len() * crate::tools::parts::places::MARK_WORDS;
                        let row: WordRow = (0..crate::tools::parts::places::MARK_WORDS).map(|step| crate::tools::parts::places::MARKS_FROM + (first + step) as Word).collect();
                        marks.push(row.clone());
                        given.push((text.len(), *kind, row));
                        text.push(crate::tools::parts::places::RESERVED_FROM + marks.len() as Word);
                    }
                    Bit::Word(word) => text.push(*word),
                }
            }
            let Some(expected) = laid_out(&form.out, &marks) else {
                self.forms.put(form, count);
                continue;
            };
            let table = self.chart(&text, None, Some(&given));
            let follows = if form.kind == WHOLE {
                let words: HashSet<Word> = text.iter().copied().collect();
                self.wholes.iter().filter(|whole| whole.form.pattern.len() <= text.len() && whole.fixed.iter().all(|word| words.contains(word))).any(|whole| {
                    self.laid(&text, 0, text.len(), &whole.form.pattern, 0, &table).iter().any(|(filled, _)| laid_out(&whole.form.out, filled).as_ref() == Some(&expected))
                })
            } else {
                table.get(0, text.len()).and_then(|cell| cell.get(&form.kind)).is_some_and(|rows| rows.holds(&expected))
            };
            if !follows {
                self.forms.put(form, count);
            }
        }
        self.index();
        let mut runs: Vec<Vec<Word>> = self.single.keys().cloned().collect();
        runs.sort_by(|one, other| other.len().cmp(&one.len()).then_with(|| lexicon.cmp_rows(one, other)));
        for run in runs {
            if run.len() > 1 {
                let Some((kind, row)) = self.single.take(&run) else { continue };
                let chart = self.chart(&run, None, None);
                if !chart.get(0, run.len()).and_then(|cell| cell.get(&kind)).is_some_and(|rows| rows.holds(&row)) {
                    self.single.put(run, (kind, row));
                }
            }
        }
        self.single_most = self.single.keys().map(Vec::len).max().unwrap_or_default();
    }
}

pub const CUTS_MOST: usize = 4;
because!(CUTS_MOST, FoundForms, "the texts of a form that one run gives at most when its parts have several readings: a few, so that a run of many readings does not make many texts");

pub fn found_forms(lessons: &[(Vec<Word>, WordRow)], chars: &HashMap<Word, WordRow>, lexicon: &Lexicon, lanes: usize) -> FormsFound {
    let lessons = &lessons[..lessons.len().min(FOUND_LESSONS)];
    let ones = lessons.iter().filter(|(_, answer)| answer.len() == 1).count();
    let signs = lessons.len() >= SURE_LESSONS && sure_of(ones, lessons.len());
    let lessons: Vec<(Vec<Word>, WordRow)> = if signs {
        lessons.iter().map(|(text, answer)| (text.clone(), answer.iter().flat_map(|word| chars.get(word).cloned().unwrap_or_default()).collect())).collect()
    } else {
        lessons.to_vec()
    };
    let step = (lessons.len() / TRIAL_LESSONS).max(1);
    let counted = counted_table(&lessons, &|word| lexicon.rank.get(word as usize).copied().unwrap_or(word));
    let trial: Vec<&(Vec<Word>, WordRow)> = lessons.iter().step_by(step).collect();
    let searched = Cell::new(0);
    let mut best: Option<FormsFound> = None;
    let mut row_ways = true;
    for mode in [Start::Pairs, Start::Table, Start::Rows] {
        if mode == Start::Rows && !row_ways {
            break;
        }
        let mut model = FormsFound::new(&lessons, mode, &counted, lexicon, &searched, lanes);
        model.reads_right = if model.forms.is_empty() {
            0
        } else {
            let read = &model;
            on_lanes(trial.len(), lanes, &|piece| read.read(&trial[piece].0, lexicon).as_ref() == Some(&trial[piece].1)).into_iter().filter(|right| *right).count()
        };
        row_ways = model.in_a_row;
        let fit = model.fit;
        if best.as_ref().is_none_or(|held| model.reads_right > held.reads_right) {
            best = Some(model);
        }
        let right = best.as_ref().map_or(0, |held| held.reads_right);
        if !fit || sure_of(right, trial.len()) {
            break;
        }
    }
    let mut best = best.unwrap_or_else(|| FormsFound::new(&[], Start::Pairs, &counted, lexicon, &searched, lanes));
    best.signs = signs;
    best
}
because!(found_forms, FoundForms, "the forms found from the first lessons, in the way that reads the most of them right, counted on a sample: the first parts from pairs of lessons; or the lessons of one word, each a part of its own; or these with all single parts in a row as one part; it stops at a way that reads nearly all of the sample. When nearly every answer is one word, an answer is read as the row of its signs");
