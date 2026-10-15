use ilm_core::base::lanes::on_lanes;
use ilm_core::split::parts;
use ilm_core::base::lines::one_line;
use ilm_core::cases::Case;
use ilm_core::cases::START_NAME;
use patterns::{because, source};
use std::collections::BTreeSet;
use std::path::Path;

pub struct SetExport;
source!(SetExport, "a public set as plain files for the program: one case file for each kind (a first line of named values, then for each case its told sentences, the label lines of a lesson, the question, the answer and an empty line), a table of new words for a set that has one, the sizes of the cut of each kind, and the description of the set; the description of a set is a plain file of lines of a key and values, which also names the raw files that the reader of the set reads");

pub const EXPORT_FIELD: char = '\t';
because!(EXPORT_FIELD, SetExport, "the mark between the fields of a line of a description, of a case file, of a table of new words and of the sizes of the cuts");

const LINE: &str = "\n";
because!(LINE, SetExport, "the end of every line that a file of a set holds");

const EXPORT_NAMED: &str = "=";
because!(EXPORT_NAMED, SetExport, "the mark between a name and its value in the first line of a case file");

const HEAD_MARK: &str = "#";
because!(HEAD_MARK, SetExport, "the mark of the first line of a case file");

const SENTENCE_MARK: &str = "S";
because!(SENTENCE_MARK, SetExport, "the mark of a line with one told sentence");

const QUESTION_MARK: &str = "Q";
because!(QUESTION_MARK, SetExport, "the mark of the line that a reader of a case file takes as what is asked, after the told sentences");

const ANSWER_MARK: &str = "A";
because!(ANSWER_MARK, SetExport, "the mark of the line with the answer, which closes a case");

pub const FILE_KEY: &str = "file";
because!(FILE_KEY, SetExport, "the key of a line of a description that names a raw file: the kind or the part it is for, its role, and its path; the program reads it and does not write it to the set");

pub const EVERY_KIND: &str = "*";
because!(EVERY_KIND, SetExport, "the kind of a line of a description that names a raw file of every kind of the set");

const NEW_KEY: &str = "new";
because!(NEW_KEY, SetExport, "the key of a line of a description with a word and its new word, for a set whose table of new words is fixed and the same for every kind; the program reads it and does not write it to the set");

const NEW_FIELD: usize = 2;
because!(NEW_FIELD, SetExport, "the place of the new word on a line of new words, after the key and the word");

const STORIES_KEY: &str = "fit_stories";
because!(STORIES_KEY, SetExport, "the key of a line of a description that says the lessons of the set are counted in stories; the program reads it and does not write it to the set");

const EXPORT_WORDS_END: &str = ".words.txt";
because!(EXPORT_WORDS_END, SetExport, "the end of the name of the table of new words of a kind");

const EXPORT_CASES_END: &str = ".txt";
because!(EXPORT_CASES_END, SetExport, "the end of the name of the case file of a kind");

const SPLIT_NAME: &str = "split.txt";
because!(SPLIT_NAME, SetExport, "the name of the file with the sizes of the cut of each kind");

const SOURCE_NAME: &str = "source.txt";
because!(SOURCE_NAME, SetExport, "the name of the description of a set beside its case files");

#[derive(Clone, Debug, Default)]
pub struct Described {
    pub lines: Vec<Vec<String>>,
}
because!(Described, SetExport, "the description of a set as its lines, each a key and its values. value gives the first value of a key; values gives the values of the first line of a key and a name, setting the first value of a setting that the reader of the set takes; pairs gives the small and the large kind of each pair; file gives the path of the raw file of a kind or part in a role; kinds gives the kinds to write, those of the tasks first, then every kind that a development run measures, then the kinds of the pairs, each once; written gives the lines that the set holds of its description");

impl Described {
    pub fn read(place: &Path) -> Result<Described, String> {
        let text = std::fs::read_to_string(place).map_err(|error| format!("{}: {error}", place.display()))?;
        Ok(Described { lines: text.lines().filter(|line| !line.is_empty()).map(|line| line.split(EXPORT_FIELD).map(str::to_string).collect()).collect() })
    }

    pub fn value(&self, key: &str) -> Option<&str> {
        self.lines.iter().find(|line| line.first().is_some_and(|first| first == key)).and_then(|line| line.get(1)).map(String::as_str)
    }

    pub fn file(&self, kind: &str, role: &str) -> Result<String, String> {
        self.files(kind, role).into_iter().next().ok_or_else(|| format!("the description names no file for {kind} as {role}"))
    }

    pub fn files(&self, kind: &str, role: &str) -> Vec<String> {
        let mine = |wanted: &str| -> Vec<String> {
            self.lines.iter().filter(|line| line.len() > PATH_FIELD && line[0] == FILE_KEY && line[1] == wanted && line[ROLE_FIELD] == role).map(|line| line[PATH_FIELD].clone()).collect()
        };
        let found = mine(kind);
        if found.is_empty() { mine(EVERY_KIND) } else { found }
    }

    pub fn new_words(&self) -> Option<Vec<(String, String)>> {
        let table: Vec<(String, String)> = self.lines.iter().filter(|line| line.len() > NEW_FIELD && line[0] == NEW_KEY).map(|line| (line[1].clone(), line[NEW_FIELD].clone())).collect();
        (!table.is_empty()).then_some(table)
    }

    pub fn kinds(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let mut add = |kind: &str| {
            if !out.iter().any(|held| held == kind) {
                out.push(kind.to_string());
            }
        };
        for line in self.lines.iter().filter(|line| line[0] == TASK_KEY) {
            line.iter().skip(TASK_KINDS_FROM).for_each(|kind| add(kind));
        }
        for line in self.lines.iter().filter(|line| line[0] == ALL_KEY) {
            line.iter().skip(1).for_each(|kind| add(kind));
        }
        for line in self.lines.iter().filter(|line| line[0] == PAIR_KEY) {
            line.iter().skip(TASK_KINDS_FROM).for_each(|kind| add(kind));
        }
        out
    }

    pub fn values(&self, key: &str, name: &str) -> Option<&[String]> {
        self.lines.iter().find(|line| line.len() > SETTING_FIELD && line[0] == key && line[1] == name).map(|line| &line[SETTING_FIELD..])
    }

    pub fn setting(&self, name: &str) -> Option<&str> {
        self.values(SETTING_KEY, name).and_then(|values| values.first()).map(String::as_str)
    }

    pub fn pairs(&self) -> Vec<(String, String)> {
        self.lines.iter().filter(|line| line[0] == PAIR_KEY).filter_map(|line| Some((line.get(TASK_KINDS_FROM)?.clone(), line.get(TASK_KINDS_FROM + 1)?.clone()))).collect()
    }

    pub fn written(&self) -> String {
        let kept: Vec<String> = self.lines.iter().filter(|line| ![FILE_KEY, STORIES_KEY, NEW_KEY, SETTING_KEY, NAMED_KEY].contains(&line[0].as_str())).map(|line| line.join(&EXPORT_FIELD.to_string())).collect();
        kept.join(LINE) + LINE
    }

    pub fn stories(&self) -> bool {
        self.value(STORIES_KEY).and_then(|value| value.parse::<usize>().ok()) == Some(1)
    }
}

pub const SETTING_KEY: &str = "with";
because!(SETTING_KEY, SetExport, "the key of a line of a description with a name and a value that the reader of the set takes, such as the configuration whose rows a kind reads; the program reads it and does not write it to the set");

const SETTING_FIELD: usize = 2;
because!(SETTING_FIELD, SetExport, "the place of the first value on a line of a setting or of named words, after the key and the name");

pub const NAMED_KEY: &str = "named";
because!(NAMED_KEY, SetExport, "the key of a line of a description with a kind and the words that its reader makes new words for; the program reads it and does not write it to the set");

const TASK_KEY: &str = "task";
because!(TASK_KEY, SetExport, "the key of a line of a description with a task and its kinds");

const ALL_KEY: &str = "all";
because!(ALL_KEY, SetExport, "the key of a line of a description with a kind that a development run measures");

const PAIR_KEY: &str = "pair";
because!(PAIR_KEY, SetExport, "the key of a line of a description with a task and the small and the large kind of its pair");

const TASK_KINDS_FROM: usize = 2;
because!(TASK_KINDS_FROM, SetExport, "the place of the first kind on a line of a task or of a pair, after the key and the name of the task");

const ROLE_FIELD: usize = 2;
because!(ROLE_FIELD, SetExport, "the place of the role of a raw file on its line, after the key and the kind");

const PATH_FIELD: usize = 3;
because!(PATH_FIELD, SetExport, "the place of the path of a raw file on its line, after the key, the kind and the role");

const FIT_KEY: &str = "fit";
because!(FIT_KEY, SetExport, "the key of the line of a description with the most lessons of a kind");

const HELD_KEY: &str = "held";
because!(HELD_KEY, SetExport, "the key of the line of a description with the most read cases of a kind");

#[derive(Clone, Debug, Default)]
pub struct KindCases {
    pub cases: Vec<Case>,
    pub read_from: Option<usize>,
    pub told: Vec<Vec<String>>,
    pub asked: Vec<Option<String>>,
    pub new_words: Option<Vec<(String, String)>>,
}
because!(KindCases, SetExport, "what the reader of a set gives for one kind: its cases in their order, the place of the first case to read when the set has one, for a lesson case the label lines after its told sentences and after its question, and the table of new words when the set has one");

pub fn kind_file(source: &str, kind: &str, found: &KindCases, fit: usize, held: usize, stories: bool) -> String {
    let named = |name: &str, value: &dyn std::fmt::Display| format!("{name}{EXPORT_NAMED}{value}");
    let mut head: Vec<String> = vec![named(KIND_NAME, &kind), named(SOURCE_NAME_KEY, &source), named(CASE_COUNT_NAME, &found.cases.len()), named(FIT_KEY, &fit), named(HELD_KEY, &held), named(STORIES_KEY, &usize::from(stories))];
    if let Some(start) = found.read_from {
        head.push(named(START_NAME, &start));
    }
    let mut lines: Vec<String> = vec![[HEAD_MARK.to_string(), head.join(&EXPORT_FIELD.to_string())].join(&EXPORT_FIELD.to_string())];
    for (place, case) in found.cases.iter().enumerate() {
        lines.extend(case.sentences.iter().map(|sentence| format!("{SENTENCE_MARK}{EXPORT_FIELD}{}", one_line(sentence))));
        lines.extend(found.told.get(place).into_iter().flatten().cloned());
        lines.push(format!("{QUESTION_MARK}{EXPORT_FIELD}{}", one_line(&case.question)));
        if let Some(Some(asked)) = found.asked.get(place) {
            lines.push(asked.clone());
        }
        lines.push(format!("{ANSWER_MARK}{EXPORT_FIELD}{}", one_line(&case.answer)));
        lines.push(String::new());
    }
    lines.join(LINE) + LINE
}
because!(kind_file, SetExport, "the text of the case file of a kind: the first line with the kind, the set, the number of cases, the sizes of the cut and the place of the first case to read, then each case with its label lines");

const KIND_NAME: &str = "kind";
because!(KIND_NAME, SetExport, "the name of the kind in the first line of a case file");

const SOURCE_NAME_KEY: &str = "source";
because!(SOURCE_NAME_KEY, SetExport, "the name of the set in the first line of a case file");

const CASE_COUNT_NAME: &str = "cases";
because!(CASE_COUNT_NAME, SetExport, "the name of the number of cases in the first line of a case file");


pub type LabelLines = (Vec<String>, Option<String>);
because!(LabelLines, SetExport, "the label lines of one lesson case: those after its told sentences, and the one after its question when the set gives it");

pub type KindReader<'a> = &'a (dyn Fn(&str) -> Result<KindCases, String> + Sync);
because!(KindReader, SetExport, "what reads the cases of one kind of a set from its raw files; one kind is read on each thread");

pub type LessonLabels<'a> = &'a (dyn Fn(&str, &[&Case]) -> Result<Vec<LabelLines>, String> + Sync);
because!(LessonLabels, SetExport, "what gives the label lines of the lesson cases of a kind, one entry for each lesson in its order, for a set that labels its lessons");

pub struct Sizes {
    pub fit: usize,
    pub held: usize,
    pub stories: bool,
}
because!(Sizes, SetExport, "the most lessons and the most read cases of a kind, and whether the lessons are counted in stories, which the cut of the measure takes");

fn lesson_places(described: &Described, kind: &str, found: &KindCases, sizes: &Sizes, reader: KindReader) -> BTreeSet<usize> {
    let mut places: BTreeSet<usize> = parts(&found.cases, sizes.fit, sizes.held, found.read_from, sizes.stories).learn.into_iter().collect();
    let kinds = described.kinds();
    for (small, large) in described.pairs() {
        if small != kind || !kinds.contains(&large) {
            continue;
        }
        let Ok(more) = reader(&large) else { continue };
        let mut both = found.cases.clone();
        both.extend(more.cases);
        places.extend(parts(&both, sizes.fit, sizes.held, Some(found.cases.len()), false).learn.into_iter().filter(|place| *place < found.cases.len()));
    }
    places
}
because!(lesson_places, SetExport, "the places of the lesson cases of a kind: those of its learned part as the measure cuts it, and for the small kind of a pair those of the learned part of the pair, the small kind before the large one");

fn with_labels(described: &Described, kind: &str, found: &mut KindCases, sizes: &Sizes, reader: KindReader, labels: LessonLabels) -> Result<(), String> {
    let places = lesson_places(described, kind, found, sizes, reader);
    if places.is_empty() {
        return Ok(());
    }
    let lessons: Vec<&Case> = places.iter().map(|place| &found.cases[*place]).collect();
    let lines = labels(kind, &lessons)?;
    found.told.resize(found.cases.len(), Vec::new());
    found.asked.resize(found.cases.len(), None);
    for (place, (told, asked)) in places.iter().zip(lines) {
        found.told[*place] = told;
        found.asked[*place] = asked;
    }
    Ok(())
}
because!(with_labels, SetExport, "the label lines of the lesson cases of a kind put beside those cases, for a set that labels its lessons; no other case gets a label line");

fn write_whole(place: &Path, text: &str) -> Result<(), String> {
    std::fs::write(place, text).map_err(|error| format!("{}: {error}", place.display()))
}
because!(write_whole, SetExport, "a file of a set written whole");

struct KindFiles {
    cases: String,
    words: Option<String>,
    cut: String,
    said: String,
}
because!(KindFiles, SetExport, "what one kind gives to the folder of its set: the text of its case file, the text of its table of new words when it has one, its line of the sizes of the cut, and the line that the run prints");

fn kind_files(source: &str, described: &Described, kind: &str, sizes: &Sizes, reader: KindReader, labels: Option<LessonLabels>) -> Result<KindFiles, String> {
    let mut found = reader(kind)?;
    if let Some(labels) = labels {
        with_labels(described, kind, &mut found, sizes, reader, labels)?;
    }
    let cases = kind_file(source, kind, &found, sizes.fit, sizes.held, sizes.stories);
    let words = found.new_words.as_ref().map(|table| table.iter().map(|(old, new)| format!("{old}{EXPORT_FIELD}{new}{LINE}")).collect::<String>());
    let cut = parts(&found.cases, sizes.fit, sizes.held, found.read_from, sizes.stories);
    Ok(KindFiles {
        cases,
        words,
        cut: [kind.to_string(), found.cases.len().to_string(), cut.learn.len().to_string(), cut.read.len().to_string(), cut.sealed.len().to_string()].join(&EXPORT_FIELD.to_string()),
        said: format!("{source} {kind}: {} cases, learned {}, read {}, sealed {}", found.cases.len(), cut.learn.len(), cut.read.len(), cut.sealed.len()),
    })
}
because!(kind_files, SetExport, "the files of one kind: its cases read, the label lines of its lessons put in when the set labels them, its case file, its table of new words and its line of the cut");

pub fn write_set(source: &str, described: &Described, folder: &Path, reader: KindReader, labels: Option<LessonLabels>, lanes: usize) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    let number = |key: &str| described.value(key).and_then(|value| value.parse::<usize>().ok()).ok_or_else(|| format!("the description has no number for {key}"));
    let sizes = Sizes { fit: number(FIT_KEY)?, held: number(HELD_KEY)?, stories: described.stories() };
    let kinds = described.kinds();
    let made = on_lanes(kinds.len(), lanes, &|place| kind_files(source, described, &kinds[place], &sizes, reader, labels));
    let mut split: Vec<String> = Vec::new();
    let mut said: Vec<String> = Vec::new();
    for (kind, files) in kinds.iter().zip(made) {
        let files = files?;
        write_whole(&folder.join(format!("{kind}{EXPORT_CASES_END}")), &files.cases)?;
        if let Some(words) = &files.words {
            write_whole(&folder.join(format!("{kind}{EXPORT_WORDS_END}")), words)?;
        }
        split.push(files.cut);
        said.push(files.said);
    }
    write_whole(&folder.join(SPLIT_NAME), &(split.join(LINE) + LINE))?;
    write_whole(&folder.join(SOURCE_NAME), &described.written())?;
    Ok(said)
}
because!(write_set, SetExport, "the files of a set: for each kind of the description its case file, its table of new words when the reader gives one, and its line of the sizes of the cut as the measure cuts it, each kind made on a thread of its own and written in the order of the kinds; then the sizes of the cuts and the description of the set");
