use ilm_core::base::shares::share;
use ilm_core::base::words::answer_words;
use ilm_core::choice::{inner_parts, picked, picked_labels, take};
use ilm_core::split::parts;
use ilm_core::cases::Case;
use ilm_core::classes::WordClasses;
use ilm_data::sets::folder::Loaded;
use ilm_core::tools::call::{Learner, Lessons, Tool};
use patterns::{because, source};

pub struct ChangedCases;
source!(ChangedCases, "a control of changed cases: a lesson that a model answers right is changed in one place, so that its true answer turns or stays, and the model must give the new true answer; a model that answers by a regularity of the set and not by reading the changed place keeps its old answer where the truth turns. The changes and their true answers are made apart from every learner, by a reading of the set that knows its form, and come as a case file");

const ORIGIN_MARK: &str = "O";
because!(ORIGIN_MARK, ChangedCases, "the mark of the line of a changed case that names the lesson it was made from and the sort of the change");

const NO_ORIGIN: &str = "-";
because!(NO_ORIGIN, ChangedCases, "the place of the lesson of a case that is no changed form of a lesson: such a case is read as it is, for a control that asks how a model reads a group of cases");

const CONTROL_FIELD: char = '\t';
because!(CONTROL_FIELD, ChangedCases, "the sign between the fields of the line that names the lesson of a changed case");

const SORTS_SHOWN: usize = 12;
because!(SORTS_SHOWN, ChangedCases, "the sorts of change that a line of the result names: more than a control of one kind uses");

#[derive(Clone, Debug)]
pub struct Changed {
    pub case: Case,
    pub origin: Option<usize>,
    pub sort: String,
}
because!(Changed, ChangedCases, "one changed case: the case with its new true answer, the place of the lesson that it was made from in the learned part of the kind (none for a case that is read as it is), and the sort of the change");

pub fn changed_cases(cases: &[Case], labels: &[Vec<String>]) -> Vec<Changed> {
    let mut found: Vec<Changed> = Vec::new();
    for (place, case) in cases.iter().enumerate() {
        let lines = labels.get(place).map_or(&[][..], Vec::as_slice);
        for line in lines {
            let fields: Vec<&str> = line.split(CONTROL_FIELD).collect();
            if let [ORIGIN_MARK, origin, sort] = fields.as_slice() {
                let origin = if *origin == NO_ORIGIN { None } else { origin.parse::<usize>().ok() };
                if origin.is_some() || fields[1] == NO_ORIGIN {
                    found.push(Changed { case: case.clone(), origin, sort: sort.to_string() });
                }
            }
        }
    }
    found
}
because!(changed_cases, ChangedCases, "the changed cases of a control file, each with its lesson and its sort; a case with no line that names its lesson is passed over");

#[derive(Clone, Debug, Default)]
pub struct SortCount {
    pub sort: String,
    pub turns: usize,
    pub turns_right: usize,
    pub turns_kept: usize,
    pub stays: usize,
    pub stays_right: usize,
    pub read: usize,
    pub read_right: usize,
}
because!(SortCount, ChangedCases, "the count of one sort of change: the changed cases where the true answer turns and those where it stays, each with the cases that the model answers right, the cases where the truth turns and the model keeps its old answer (the sign of a model that does not read the changed place; where the new truth is not known it is written as no answer, and only this count says something), and the cases that are read as they are");

#[derive(Clone, Debug, Default)]
pub struct ControlCount {
    pub lessons: usize,
    pub sorts: Vec<SortCount>,
    pub tool: String,
}
because!(ControlCount, ChangedCases, "the result of a control: how many lessons were answered right and changed, the count of each sort of change in the order of first sight, and the tool whose model was measured");

impl ControlCount {
    pub fn total(&self) -> SortCount {
        self.sorts.iter().fold(SortCount::default(), |sum, one| SortCount {
            sort: String::new(),
            turns: sum.turns + one.turns,
            turns_right: sum.turns_right + one.turns_right,
            turns_kept: sum.turns_kept + one.turns_kept,
            stays: sum.stays + one.stays,
            stays_right: sum.stays_right + one.stays_right,
            read: sum.read + one.read,
            read_right: sum.read_right + one.read_right,
        })
    }

    pub fn line(&self, kind: &str) -> String {
        let all = self.total();
        let sorts: Vec<String> = self
            .sorts
            .iter()
            .take(SORTS_SHOWN)
            .map(|one| {
                if one.read > 0 {
                    return format!("{} {} of {} ({:.places$}%)", one.sort, one.read_right, one.read, share(one.read_right, one.read), places = 1);
                }
                let stays = if one.stays > 0 { format!(", stays {} of {}", one.stays_right, one.stays) } else { String::new() };
                format!("{} {} of {} (old answer kept {}){stays}", one.sort, one.turns_right, one.turns, one.turns_kept)
            })
            .collect();
        format!(
            "{kind}: lessons changed {}; the truth turns: {} of {} ({:.places$}%); stays: {} of {} | {} | the model: {}",
            self.lessons,
            all.turns_right,
            all.turns,
            if all.turns > 0 { share(all.turns_right, all.turns) } else { f64::default() },
            all.stays_right,
            all.stays,
            sorts.join("; "),
            self.tool,
            places = 1
        )
    }
}

fn sort_place(count: &mut ControlCount, sort: &str) -> usize {
    match count.sorts.iter().position(|known| known.sort == sort) {
        Some(place) => place,
        None => {
            count.sorts.push(SortCount { sort: sort.to_string(), ..SortCount::default() });
            count.sorts.len() - 1
        }
    }
}
because!(sort_place, ChangedCases, "the place of the count of a sort of change, a new count at the end for a sort that comes first");

pub fn control(loaded: &Loaded, fit: usize, held: usize, changed: &[Changed], tools: &[Tool], tool: Option<usize>, whole: bool, most: Option<usize>, classes: Option<&WordClasses>, lanes: usize) -> Result<ControlCount, String> {
    let cut = parts(&loaded.kind.cases, fit, held, loaded.start, loaded.stories);
    let learn = picked(&loaded.kind.cases, &cut.learn);
    let labels = picked_labels(&loaded.kind.labels, &cut.learn);
    let (first, last) = inner_parts(&learn);
    let taught: Vec<usize> = if whole { (0..learn.len()).collect() } else { first };
    let lessons = picked(&learn, &taught);
    let lesson_labels = picked_labels(&labels, &taught);
    let given = Lessons { cases: &lessons, labels: &lesson_labels, ask: None, classes, before: None };
    let (model, name): (Box<dyn Learner>, String) = match tool {
        Some(place) => ((tools[place].learn)(given, lanes)?, tools[place].name.to_string()),
        None => {
            let taken = take(tools, given, lanes, false)?;
            let name = taken.named.clone();
            (Box::new(taken), name)
        }
    };
    let mut read = vec![false; learn.len()];
    let open: Vec<usize> = if whole { (0..learn.len()).collect() } else { last };
    for place in open {
        let lesson = &learn[place];
        read[place] = model.answer(&lesson.sentences, &lesson.question).words == answer_words(&lesson.answer);
    }
    let mut count = ControlCount { tool: name, ..ControlCount::default() };
    let mut seen = vec![false; learn.len()];
    for one in changed {
        let truth = answer_words(&one.case.answer);
        let right = |model: &dyn Learner| model.answer(&one.case.sentences, &one.case.question).words == truth;
        let Some(origin) = one.origin else {
            let place = sort_place(&mut count, &one.sort);
            count.sorts[place].read += 1;
            count.sorts[place].read_right += usize::from(right(model.as_ref()));
            continue;
        };
        if origin >= learn.len() || !read[origin] {
            continue;
        }
        if !seen[origin] {
            if most.is_some_and(|most| count.lessons >= most) {
                continue;
            }
            seen[origin] = true;
            count.lessons += 1;
        }
        let old = answer_words(&learn[origin].answer);
        let turns = truth != old;
        let said = model.answer(&one.case.sentences, &one.case.question).words;
        let right = said == truth;
        let kept = said == old;
        let place = sort_place(&mut count, &one.sort);
        let sort = &mut count.sorts[place];
        if turns {
            sort.turns += 1;
            sort.turns_right += usize::from(right);
            sort.turns_kept += usize::from(kept);
        } else {
            sort.stays += 1;
            sort.stays_right += usize::from(right);
        }
    }
    Ok(count)
}
because!(control, ChangedCases, "a control on one kind: a model of the first part of the lessons (or of all lessons) reads the rest of them (or all of them); for each lesson that it answers right, it reads the changed cases made from that lesson, and each is counted by its sort, by whether the true answer turns, and by whether the model gives it; at most a given number of lessons are used, in the order of the file; a case with no lesson is read as it is");
