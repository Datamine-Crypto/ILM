use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases, LabelLines};
use crate::sets::readers::formal::{LABEL_FACT, LABEL_FIELD, LABEL_UNIT, LABEL_YES};
use crate::sets::parquet::records::Value;
use crate::sets::readers::common::{LESSON_FILES, NEW_WORD_START, rows_where, text_value, told_sentences};
use crate::sets::textmatch::TextPattern;
use patterns::{because, source};
use std::collections::{BTreeSet, HashMap};

pub struct KinshipStories;
source!(KinshipStories, "CLUTRR of Sinha and others in the parquet files that the hub made from its loading script: stories of a family where crowd workers wrote a sentence for each told relation, with the pair of people asked about, the relation to find, the task with its number of steps after the last point, the told relations as a list of pairs of places of people and a list of their kinds, and the people with their genders; the train file holds stories of the steps of the lessons, the validation and test files the stories to read; a case tells the sentences of the story or, in the graph form, each told relation as a person, the relation and a person; the new words are the names in square brackets of all stories that stand nowhere in small letters, each with zq before it");

const STORY_PREFIX: &str = "clutrr_k";
because!(STORY_PREFIX, KinshipStories, "the start of the name of a kind of the story form, before its number of steps");

const STORY_LONGER: &str = "clutrr_longer";
because!(STORY_LONGER, KinshipStories, "the kind of the story form that learns from all steps of the train file and reads the stories of the test files with more steps");

const GRAPH_STEPS: &str = "_k";
because!(GRAPH_STEPS, KinshipStories, "the mark before the number of steps at the end of the name of a kind of the graph form");

const GRAPH_LESSONS: &str = "clutrrg_lessons";
because!(GRAPH_LESSONS, KinshipStories, "the kind of the graph form with the lessons alone, the small kind of its pair");

const GRAPH_READ: &str = "clutrrg_read";
because!(GRAPH_READ, KinshipStories, "the start of the name of a kind of the graph form with the stories to read alone, the large kind of its pair");

const VALID_FILES: &str = "valid";
because!(VALID_FILES, KinshipStories, "the role in the description of the validation file, read for the kinds of the steps of the lessons");

const TEST_FILES: &str = "test";
because!(TEST_FILES, KinshipStories, "the role in the description of the test files, read for every kind");

const KIN_COLUMNS: [&str; KIN_WIDTH] = ["story", "query", "target_text", "task_name", "story_edges", "edge_types", "genders"];
because!(KIN_COLUMNS, KinshipStories, "the columns of a story: its text, the pair asked about, the relation to find, the task, the told relations as pairs of places, their kinds, and the people with their genders");

const KIN_WIDTH: usize = 7;
because!(KIN_WIDTH, KinshipStories, "the number of columns of a story that a case is made from");

const TASK_STEPS: char = '.';
because!(TASK_STEPS, KinshipStories, "the mark before the number of steps at the end of the name of a task");

const PEOPLE_GAP: char = ',';
because!(PEOPLE_GAP, KinshipStories, "the mark between the people of the gender column");

const GENDER_GAP: char = ':';
because!(GENDER_GAP, KinshipStories, "the mark between a name and its gender in the gender column");

const EDGE: &str = r"\((\d+),\s*(\d+)\)";
because!(EDGE, KinshipStories, "a told relation in the column of edges: the places of its two people in the gender column");

const EDGE_KIND: &str = r"'([^']*)'";
because!(EDGE_KIND, KinshipStories, "the kind of a told relation in its column, a word in quotes");

const NAME: &str = r"\[([^\]]+)\]";
because!(NAME, KinshipStories, "a name in a story, between square brackets");

const SMALL_WORD: &str = r"[a-z]+";
because!(SMALL_WORD, KinshipStories, "a run of small letters of a story outside the names, which a new word must not be");

const NAME_GAP: &str = " ";
because!(NAME_GAP, KinshipStories, "what stands for a name when the words of a story outside the names are counted");

const TOLD_GAP: &str = " ";
because!(TOLD_GAP, KinshipStories, "the mark between a person, a relation and a person in a told relation of the graph form");

fn kin_number(text: &str, after: &str) -> Result<usize, String> {
    text.rsplit_once(after).and_then(|(_, number)| number.parse().ok()).ok_or_else(|| format!("no number of steps in {text}"))
}
because!(kin_number, KinshipStories, "the number of steps after the last mark in the name of a task or of a kind");

struct KinStory {
    case: Case,
    steps: usize,
}
because!(KinStory, KinshipStories, "one case of a story with its number of steps, by which the kinds take it");

fn told_steps(edges: &str, kinds: &str, genders: &str) -> Result<Vec<[String; KIN_STEP_PARTS]>, String> {
    let people: Vec<&str> = genders.split(PEOPLE_GAP).map(|person| person.split(GENDER_GAP).next().unwrap_or_default()).collect();
    let pairs = TextPattern::new(EDGE)?.find_all(edges);
    let named = TextPattern::new(EDGE_KIND)?.find_all(kinds);
    let mut out: Vec<[String; KIN_STEP_PARTS]> = Vec::new();
    for (pair, kind) in pairs.iter().zip(&named) {
        let ([first, second], [kind]) = (pair.as_slice(), kind.as_slice()) else { return Err(format!("an edge {edges} of no known form")) };
        let person = |place: &str| place.parse::<usize>().ok().and_then(|place| people.get(place).map(|name| name.to_string())).ok_or_else(|| format!("no person {place} in {genders}"));
        out.push([person(first)?, kind.clone(), person(second)?]);
    }
    Ok(out)
}
because!(told_steps, KinshipStories, "the told relations of a story: for each pair of places and its kind, the first person, the kind and the second person, which says that the second person is the kind of the first");

const KIN_STEP_PARTS: usize = 3;
because!(KIN_STEP_PARTS, KinshipStories, "the parts of a told relation: the first person, the kind and the second person");

fn graph_told(edges: &str, kinds: &str, genders: &str) -> Result<Vec<String>, String> {
    Ok(told_steps(edges, kinds, genders)?.into_iter().map(|step| step.join(TOLD_GAP)).collect())
}
because!(graph_told, KinshipStories, "the told relations of a story in the graph form, each as the first person, the kind and the second person");

fn stories(places: &[String], graph: bool) -> Result<Vec<KinStory>, String> {
    let mut out: Vec<KinStory> = Vec::new();
    for place in places {
        for [story, query, target, task, edges, kinds, genders] in rows_where(place, KIN_COLUMNS, None)? {
            let text = |value: &Value| text_value(value, place);
            let sentences = if graph { graph_told(&text(&edges)?, &text(&kinds)?, &text(&genders)?)? } else { told_sentences(&text(&story)?) };
            let case = Case { sentences, question: text(&query)?, answer: text(&target)? };
            out.push(KinStory { case, steps: kin_number(&text(&task)?, &TASK_STEPS.to_string())? });
        }
    }
    Ok(out)
}
because!(stories, KinshipStories, "the stories of some files in the order of the files, each as a case of the story form or of the graph form with its number of steps");

struct KinFiles {
    train: Vec<KinStory>,
    valid: Vec<KinStory>,
    test: Vec<KinStory>,
}
because!(KinFiles, KinshipStories, "the stories of the train file, of the validation file and of the test files");

impl KinFiles {
    fn read(described: &Described, kind: &str, graph: bool) -> Result<KinFiles, String> {
        Ok(KinFiles {
            train: stories(&described.files(kind, LESSON_FILES), graph)?,
            valid: stories(&described.files(kind, VALID_FILES), graph)?,
            test: stories(&described.files(kind, TEST_FILES), graph)?,
        })
    }

    fn lesson_steps(&self) -> BTreeSet<usize> {
        self.train.iter().map(|story| story.steps).collect()
    }

    fn mixed_lessons(&self) -> Vec<Case> {
        let by_steps: Vec<Vec<&Case>> = self.lesson_steps().into_iter().map(|steps| self.train.iter().filter(|story| story.steps == steps).map(|story| &story.case).collect()).collect();
        let mut lessons: Vec<Case> = Vec::new();
        let longest = by_steps.iter().map(Vec::len).max().unwrap_or_default();
        for place in 0..longest {
            lessons.extend(by_steps.iter().filter_map(|cases| cases.get(place).map(|case| (*case).clone())));
        }
        lessons
    }

    fn of_steps(&self, steps: usize, lessons: bool) -> Vec<Case> {
        let parts: Vec<&[KinStory]> = if self.lesson_steps().contains(&steps) {
            if lessons { vec![&self.train, &self.valid, &self.test] } else { vec![&self.valid, &self.test] }
        } else {
            vec![&self.test]
        };
        parts.into_iter().flatten().filter(|story| story.steps == steps).map(|story| story.case.clone()).collect()
    }
}

pub fn clutrr_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let files = KinFiles::read(described, kind, false)?;
    let lesson_steps = files.lesson_steps();
    let new_words = Some(kin_new_words(described, kind)?);
    let mut cases = files.mixed_lessons();
    let read_from = Some(cases.len());
    if kind == STORY_LONGER {
        cases.extend(files.test.iter().filter(|story| story.steps > lesson_steps.last().copied().unwrap_or_default()).map(|story| story.case.clone()));
    } else {
        cases.extend(files.of_steps(kin_number(kind, STORY_PREFIX)?, false));
    }
    Ok(KindCases { cases, read_from, new_words, ..KindCases::default() })
}
because!(clutrr_cases, KinshipStories, "the cases of a kind of the story form: the lessons are the train stories of each number of steps in turn, so that the joins of more steps are learned from the shorter stories; a kind reads the stories of its steps in the validation and test files, or in the test files alone when no lesson has its steps; the longer kind reads the test stories with more steps than any lesson");

pub fn clutrr_graph_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let files = KinFiles::read(described, kind, true)?;
    let mut lessons = files.mixed_lessons();
    let new_words = Some(kin_new_words(described, kind)?);
    if kind == GRAPH_LESSONS {
        return Ok(KindCases { cases: lessons, new_words, ..KindCases::default() });
    }
    let read = files.of_steps(kin_number(kind, GRAPH_STEPS)?, false);
    if kind.starts_with(GRAPH_READ) {
        return Ok(KindCases { cases: read, new_words, ..KindCases::default() });
    }
    let read_from = Some(lessons.len());
    lessons.extend(read);
    Ok(KindCases { cases: lessons, read_from, new_words, ..KindCases::default() })
}
because!(clutrr_graph_cases, KinshipStories, "the cases of a kind of the graph form: the lessons are the train stories of each number of steps in turn; a kind reads the stories of its steps in the validation and test files, or in the test files alone when no lesson has its steps; the lessons kind and the read kind of the pair hold one part each, with no place to read from");

pub fn clutrr_labels(described: &Described, kind: &str, lessons: &[&Case]) -> Result<Vec<LabelLines>, String> {
    let mut steps_of: HashMap<Vec<String>, Vec<[String; KIN_STEP_PARTS]>> = HashMap::new();
    for role in [LESSON_FILES, VALID_FILES, TEST_FILES] {
        for place in described.files(kind, role) {
            for [story, _, _, _, edges, kinds, genders] in rows_where(&place, KIN_COLUMNS, None)? {
                let sentences = told_sentences(&text_value(&story, &place)?);
                if !steps_of.contains_key(&sentences) {
                    let steps = told_steps(&text_value(&edges, &place)?, &text_value(&kinds, &place)?, &text_value(&genders, &place)?)?;
                    steps_of.insert(sentences, steps);
                }
            }
        }
    }
    let mut out: Vec<LabelLines> = Vec::with_capacity(lessons.len());
    for lesson in lessons {
        let Some(steps) = steps_of.get(&lesson.sentences).filter(|steps| !steps.is_empty()) else {
            out.push((Vec::new(), None));
            continue;
        };
        let mut lines: Vec<String> = vec![[LABEL_UNIT.to_string(), 0.to_string(), lesson.sentences.len().to_string()].join(LABEL_FIELD)];
        lines.extend(steps.iter().map(|[first, relation, second]| [LABEL_FACT, first, relation, second, LABEL_YES].join(LABEL_FIELD)));
        out.push((lines, None));
    }
    Ok(out)
}
because!(clutrr_labels, KinshipStories, "the label lines of the lesson stories of a kind: the whole story as one written unit, and each told relation of the set as a fact of it, in the order of the set; a story that no file holds gets none");

fn kin_new_words(described: &Described, kind: &str) -> Result<Vec<(String, String)>, String> {
    let (name, small) = (TextPattern::new(NAME)?, TextPattern::new(SMALL_WORD)?);
    let mut names: BTreeSet<String> = BTreeSet::new();
    let mut words: BTreeSet<String> = BTreeSet::new();
    for role in [LESSON_FILES, VALID_FILES, TEST_FILES] {
        for place in described.files(kind, role) {
            for [story] in rows_where(&place, [KIN_COLUMNS[0]], None)? {
                let story = text_value(&story, &place)?;
                names.extend(name.find_all(&story).into_iter().flatten().map(|held| held.to_lowercase()));
                words.extend(small.find_all(&name.replace_all(&story, NAME_GAP)).into_iter().flatten());
            }
        }
    }
    Ok(names
        .into_iter()
        .filter(|held| !held.is_empty() && held.chars().all(char::is_alphabetic) && !words.contains(held))
        .map(|held| (held.clone(), format!("{NEW_WORD_START}{held}")))
        .collect())
}
because!(kin_new_words, KinshipStories, "the table of new words of the set: each name of all stories in small letters, in the order of the letters, when it is made of letters and stands nowhere in a story as a word in small letters");
