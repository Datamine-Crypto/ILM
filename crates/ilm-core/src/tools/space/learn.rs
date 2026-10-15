use crate::base::first_sight::FirstSightTable;
use crate::base::symbols::Symbols;
use crate::base::words::answer_words;
use crate::tools::call::{Learned, Learner, Lessons, Said};
use crate::tools::frequent::frequent_answer;
use crate::cases::Case;
use crate::tools::space::read::{SPACE_BETWEEN, SpaceStory, asked_pair, chain, question_form, space_story};
use crate::tools::space::words::{AXES, AXIS_VALUES, Offset, WordGuess, guess_agrees};
use patterns::{because, source};
use std::collections::HashSet;

pub struct UnitSteps;
source!(UnitSteps, "things stand at points of a plane, and a told link says how its first thing stands to its second as a step of at most one unit along each axis; the answer names how the first asked thing stands to the second, the sign of the sum of the steps along the chain of links between them, so each answer names one step and the answer of the reverse question names the reverse step. A lesson whose two asked things one sentence joins gives the step of that form directly; which step each answer names is found by trying every way to give the answers steps, a pair of answers that name the two ways of one form getting reverse steps, and keeping the way that explains the most lessons");

pub struct StepSearch;
source!(StepSearch, "the steps of the forms that no lesson gives directly are found by local search over the lessons: first by coordinate ascent, each form taking the step that explains the most of its lessons, then by a walk that takes the best change of a form of a lesson that stays unexplained even when it explains no more lessons (the tabu search of Glover, close to the walk of Selman, Kautz and Cohen), keeping the best state that the walk reaches, and then by coordinate ascent again; of steps that explain as many lessons, the one that the words of the form favour is taken");

const SPACE_LEAST_LESSONS: usize = 10;
because!(SPACE_LEAST_LESSONS, UnitSteps, "the tool learns a kind only from this many lessons that it can read or more: with fewer, the step of a form rests on chance");

const SEARCH_ROUNDS: usize = 20;
because!(SEARCH_ROUNDS, StepSearch, "the most rounds of coordinate ascent: a round that changes no form ends it, and this many are enough for the forms of the public sets");

const WALK_PER_FORM: usize = 15;
because!(WALK_PER_FORM, StepSearch, "the steps of the walk for each form that the walk may change: enough for a few visits of each form, and a bound on the time");

const TABU_STEPS: usize = 10;
because!(TABU_STEPS, StepSearch, "a form that the walk changed is not changed again for this many steps, so that the walk does not undo its own last changes");

const SPACE_NO_READ: &str = "the space tool does not read this kind: its questions do not name two things that told links join";
because!(SPACE_NO_READ, UnitSteps, "what the tool says for lessons that it cannot read as things at points");

const SPACE_NO_STEPS: &str = "the space tool does not read this kind: no way to give its answers steps explains the lessons";
because!(SPACE_NO_STEPS, UnitSteps, "what the tool says when the answers cannot name steps of a plane");

const SPACE_NOTE: &str = "the sum of the steps of the told links along the chain between the two things";
because!(SPACE_NOTE, UnitSteps, "the note of an answer that the steps of the links give");

const SPACE_NOTE_NONE: &str = "no chain of told links between the two things";
because!(SPACE_NOTE_NONE, UnitSteps, "the note of an answer for a case whose two things no chain joins");

const SPACE_NOTE_UNNAMED: &str = "the steps along the chain name a way that no answer of the lessons names";
because!(SPACE_NOTE_UNNAMED, UnitSteps, "the note of an answer for a case whose chain names a step that no answer has");

struct SpaceLesson {
    path: Vec<(usize, i32)>,
    answer: usize,
}
because!(SpaceLesson, UnitSteps, "a lesson read for the tool: the forms of the links along its chain, each with the sign of its step along the chain, and the number of its answer");

fn sign_of(sum: i32) -> i32 {
    sum.signum()
}
because!(sign_of, UnitSteps, "the change along an axis that a sum of steps names: back, none or forward");

fn reverse(offset: Offset) -> Offset {
    [-offset[0], -offset[1]]
}
because!(reverse, UnitSteps, "the step back along a link");

fn every_offset() -> Vec<Offset> {
    AXIS_VALUES.iter().flat_map(|across| AXIS_VALUES.iter().map(move |along| [*across, *along])).collect()
}
because!(every_offset, UnitSteps, "every step of at most one unit along each axis, the step of no change among them");

fn path_offset(path: &[(usize, i32)], meanings: &[Option<Offset>]) -> Option<Offset> {
    let mut sum: Offset = [0; AXES];
    for (form, way) in path {
        let step = meanings[*form]?;
        for axis in 0..AXES {
            sum[axis] += way * step[axis];
        }
    }
    Some([sign_of(sum[0]), sign_of(sum[1])])
}
because!(path_offset, UnitSteps, "the step that a chain names, the sign of the sum of its steps along each axis; nothing while a form of the chain has no step");

fn lesson_holds(lesson: &SpaceLesson, meanings: &[Option<Offset>], steps: &[Option<Offset>]) -> bool {
    steps[lesson.answer].is_some() && path_offset(&lesson.path, meanings) == steps[lesson.answer]
}
because!(lesson_holds, UnitSteps, "whether the steps of the forms explain a lesson: its chain names the step of its answer");

struct Observed {
    form: usize,
    answer: usize,
    forward: bool,
}
because!(Observed, UnitSteps, "a lesson whose two asked things one link joins: the form of the link, the answer, and whether the link names the asked things in the order of the question");

fn form_step(seen: &[usize], observed: &[Observed], steps: &[Option<Offset>]) -> Option<Offset> {
    let mut votes: FirstSightTable<Offset, u32> = FirstSightTable::new();
    for at in seen {
        let one = &observed[*at];
        let step = steps[one.answer]?;
        votes.add(if one.forward { step } else { reverse(step) }, 1);
    }
    votes.first_most().map(|(step, _)| *step)
}
because!(form_step, UnitSteps, "the step of a form from the lessons whose asked things it joins, once every answer of those lessons has a step: the step that most of them give, the first of equals");

struct StepWays<'a> {
    order: Vec<usize>,
    observed: &'a [Observed],
    seen: Vec<Vec<usize>>,
    lessons: &'a [SpaceLesson],
    decided_at: Vec<Vec<usize>>,
    later: Vec<usize>,
    steps: Vec<Option<Offset>>,
    most: usize,
    tied: Vec<Vec<Option<Offset>>>,
}
because!(StepWays, UnitSteps, "the search for the steps of the answers: the answers by frequency, the lessons that join the asked things by one link and those of each form, the lessons that are decided once each number of answers has steps and how many are decided later, the steps so far, and the ways that explain the most lessons");

impl StepWays<'_> {
    fn explains(&self, lesson: usize) -> bool {
        let lesson = &self.lessons[lesson];
        let mut sum: Offset = [0; AXES];
        for (form, way) in &lesson.path {
            let Some(step) = form_step(&self.seen[*form], self.observed, &self.steps) else { return false };
            for axis in 0..AXES {
                sum[axis] += way * step[axis];
            }
        }
        self.steps[lesson.answer] == Some([sign_of(sum[0]), sign_of(sum[1])])
    }

    fn search(&mut self, depth: usize, held: usize) {
        if held + self.later[depth] < self.most {
            return;
        }
        if depth == self.order.len() {
            if held > self.most {
                self.most = held;
                self.tied.clear();
            }
            self.tied.push(self.steps.clone());
            return;
        }
        let answer = self.order[depth];
        let choices: Vec<Offset> = if depth == 0 { FIRST_STEPS.to_vec() } else { every_offset() };
        for step in choices {
            if self.steps.contains(&Some(step)) {
                continue;
            }
            self.steps[answer] = Some(step);
            let newly = self.decided_at[depth + 1].iter().filter(|lesson| self.explains(**lesson)).count();
            self.search(depth + 1, held + newly);
            self.steps[answer] = None;
        }
    }
}

const FIRST_STEPS: [Offset; 3] = [[0, 0], [1, 0], [1, 1]];
because!(FIRST_STEPS, UnitSteps, "the steps tried for the most frequent answer: no change, a step along one axis, and a step along both; a turn or a mirror of the plane gives every other step from one of them, and explains the same lessons");

fn answer_steps(observed: &[Observed], forms: &Symbols, lessons: &[SpaceLesson], answers: usize, order: &[usize]) -> Option<Vec<Option<Offset>>> {
    if answers > every_offset().len() {
        return None;
    }
    let mut seen: Vec<Vec<usize>> = vec![Vec::new(); forms.len()];
    observed.iter().enumerate().for_each(|(at, one)| seen[one.form].push(at));
    let rank: Vec<usize> = {
        let mut rank = vec![0usize; answers];
        order.iter().enumerate().for_each(|(at, answer)| rank[*answer] = at);
        rank
    };
    let mut decided_at: Vec<Vec<usize>> = vec![Vec::new(); order.len() + 1];
    for (number, lesson) in lessons.iter().enumerate() {
        let mut last = Some(rank[lesson.answer]);
        for (form, _) in &lesson.path {
            if seen[*form].is_empty() {
                last = None;
                break;
            }
            last = last.max(seen[*form].iter().map(|at| rank[observed[*at].answer]).max());
        }
        if let Some(last) = last {
            decided_at[last + 1].push(number);
        }
    }
    let mut later = vec![0usize; order.len() + 1];
    for depth in (0..order.len()).rev() {
        later[depth] = later[depth + 1] + decided_at[depth + 1].len();
    }
    let mut ways = StepWays { order: order.to_vec(), observed, seen, lessons, decided_at, later, steps: vec![None; answers], most: 0, tied: Vec::new() };
    ways.search(0, 0);
    let mut chosen: Option<(usize, Vec<Option<Offset>>)> = None;
    for steps in ways.tied {
        let meanings: Vec<Option<Offset>> = ways.seen.iter().map(|seen| form_step(seen, observed, &steps)).collect();
        let known = known_steps(forms, &meanings);
        let mut guess = WordGuess::new(&known);
        let right = guess.signs_left_out_right(&known);
        if chosen.as_ref().is_none_or(|(most, _)| right > *most) {
            chosen = Some((right, steps));
        }
    }
    chosen.map(|(_, steps)| steps)
}
because!(answer_steps, UnitSteps, "the step of each answer: every way to give the answers different steps, searched by branch and bound over the answers by frequency, kept by the lessons that the steps of the forms seen directly explain, a lesson counted once every answer that it needs has a step; of ways that explain as many, the one under which the signs of the forms best guess the steps of the other forms (each form left out in turn), the first of equals");

fn known_steps(forms: &Symbols, meanings: &[Option<Offset>]) -> Vec<(String, Offset)> {
    meanings.iter().enumerate().filter_map(|(form, step)| step.map(|step| (forms.name(form as u32).to_string(), step))).collect()
}
because!(known_steps, UnitSteps, "the forms that have a step, with their steps");

fn guessed_steps(forms: &Symbols, guess: &WordGuess) -> Vec<Offset> {
    forms.names().iter().map(|form| guess.guess(form)).collect()
}
because!(guessed_steps, StepSearch, "the step that the words of each form guess");

struct LinkSearch<'a> {
    lessons: &'a [SpaceLesson],
    steps: &'a [Option<Offset>],
    uses: Vec<Vec<usize>>,
    fixed: Vec<bool>,
}
because!(LinkSearch, StepSearch, "the search for the steps of the forms: the lessons, the steps of the answers, the lessons of each form, and the forms whose steps the lessons give directly");

impl LinkSearch<'_> {
    fn held(&self, form: usize, meanings: &[Option<Offset>]) -> usize {
        self.uses[form].iter().filter(|lesson| lesson_holds(&self.lessons[**lesson], meanings, self.steps)).count()
    }

    fn ascent(&self, meanings: &mut [Option<Offset>], guessed: &[Offset]) {
        let mut order: Vec<usize> = (0..meanings.len()).filter(|form| !self.fixed[*form] && !self.uses[*form].is_empty()).collect();
        order.sort_by_key(|form| std::cmp::Reverse(self.uses[*form].len()));
        for _ in 0..SEARCH_ROUNDS {
            let mut changed = false;
            for form in &order {
                let now = meanings[*form];
                let mut best = (self.held(*form, meanings), now.map(|step| guess_agrees(guessed[*form], step)), now);
                for step in every_offset() {
                    if Some(step) == now {
                        continue;
                    }
                    meanings[*form] = Some(step);
                    let tried = (self.held(*form, meanings), Some(guess_agrees(guessed[*form], step)));
                    if tried.0 > best.0 || (tried.0 == best.0 && tried.1 > best.1) {
                        best = (tried.0, tried.1, Some(step));
                    }
                }
                meanings[*form] = best.2;
                changed |= best.2 != now;
            }
            if !changed {
                break;
            }
        }
    }

    fn walk(&self, meanings: &mut Vec<Option<Offset>>, guessed: &[Offset]) {
        let free = (0..meanings.len()).filter(|form| !self.fixed[*form] && !self.uses[*form].is_empty()).count();
        let mut good: Vec<bool> = self.lessons.iter().map(|lesson| lesson_holds(lesson, meanings, self.steps)).collect();
        let mut total = good.iter().filter(|held| **held).count();
        let mut best = (total, meanings.clone());
        let mut changed_at: Vec<Option<usize>> = vec![None; meanings.len()];
        for step in 0..free * WALK_PER_FORM {
            let bad: Vec<usize> = (0..self.lessons.len()).filter(|lesson| !good[*lesson]).collect();
            if bad.is_empty() {
                break;
            }
            let lesson = &self.lessons[bad[step % bad.len()]];
            let mut chosen: Option<(i64, usize, usize, Offset)> = None;
            for (form, _) in &lesson.path {
                if self.fixed[*form] || changed_at[*form].is_some_and(|at| at + TABU_STEPS >= step) {
                    continue;
                }
                let now = meanings[*form];
                let before = self.uses[*form].iter().filter(|other| good[**other]).count() as i64;
                for offset in every_offset() {
                    if Some(offset) == now {
                        continue;
                    }
                    meanings[*form] = Some(offset);
                    let gain = self.held(*form, meanings) as i64 - before;
                    let score = guess_agrees(guessed[*form], offset);
                    if chosen.is_none_or(|(most, top, _, _)| gain > most || (gain == most && score > top)) {
                        chosen = Some((gain, score, *form, offset));
                    }
                }
                meanings[*form] = now;
            }
            let Some((gain, _, form, offset)) = chosen else { continue };
            meanings[form] = Some(offset);
            changed_at[form] = Some(step);
            for other in &self.uses[form] {
                good[*other] = lesson_holds(&self.lessons[*other], meanings, self.steps);
            }
            total = (total as i64 + gain) as usize;
            if total > best.0 {
                best = (total, meanings.clone());
            }
        }
        *meanings = best.1;
    }
}

pub struct SpaceModel {
    form_words: HashSet<String>,
    names: HashSet<String>,
    forms: Symbols,
    meanings: Vec<Option<Offset>>,
    guess: WordGuess,
    answers: Vec<(Offset, Vec<String>)>,
    default: Vec<String>,
}
because!(SpaceModel, UnitSteps, "what the tool learned: the words of the form of the question, the names of the things of the lessons, the forms with their steps, the counts of the words of the forms for a form that no lesson held, the step of each answer, and the most frequent answer of the lessons");

impl Learner for SpaceModel {
    fn answer(&self, sentences: &[String], question: &str) -> Said {
        let none = || Said { words: self.default.clone(), note: SPACE_NOTE_NONE.to_string() };
        let Some(asked) = asked_pair(question, &self.form_words) else { return none() };
        let story = space_story(sentences, &asked, &self.names);
        let known = SpaceStory { links: story.links.iter().filter(|link| self.forms.find(&link.form).is_some_and(|number| self.meanings[number as usize].is_some())).cloned().collect(), asked: story.asked };
        let (story, steps) = match chain(&known) {
            Some(steps) => (known, steps),
            None => match chain(&story) {
                Some(steps) => (story, steps),
                None => return none(),
            },
        };
        let mut sum: Offset = [0; AXES];
        for (link, forward) in steps {
            let form = &story.links[link].form;
            let step = self.forms.find(form).and_then(|number| self.meanings[number as usize]).unwrap_or_else(|| self.guess.guess(form));
            let way = if forward { 1 } else { -1 };
            for axis in 0..AXES {
                sum[axis] += way * step[axis];
            }
        }
        let named = [sign_of(sum[0]), sign_of(sum[1])];
        match self.answers.iter().find(|(step, _)| *step == named) {
            Some((_, words)) => Said { words: words.clone(), note: SPACE_NOTE.to_string() },
            None => Said { words: self.default.clone(), note: SPACE_NOTE_UNNAMED.to_string() },
        }
    }
}

fn story_names(cases: &[Case], form_words: &HashSet<String>) -> HashSet<String> {
    cases.iter().filter_map(|case| asked_pair(&case.question, form_words)).flatten().collect()
}
because!(story_names, UnitSteps, "the names of things: every word that a question of the lessons names as a thing");

pub(super) fn learn_space(lessons: Lessons, _lanes: usize) -> Learned {
    let form_words = question_form(lessons.cases);
    let names = story_names(lessons.cases, &form_words);
    let mut forms = Symbols::new();
    let mut answers = Symbols::new();
    let mut answer_counts: FirstSightTable<usize, u32> = FirstSightTable::new();
    let mut read: Vec<SpaceLesson> = Vec::new();
    let mut observed: Vec<Observed> = Vec::new();
    for case in lessons.cases {
        let Some(asked) = asked_pair(&case.question, &form_words) else { continue };
        let story = space_story(&case.sentences, &asked, &names);
        let answer = answers.number(&answer_words(&case.answer).join(SPACE_BETWEEN)) as usize;
        answer_counts.add(answer, 1);
        for link in &story.links {
            let form = forms.number(&link.form) as usize;
            if [link.first, link.second] == story.asked {
                observed.push(Observed { form, answer, forward: true });
            } else if [link.second, link.first] == story.asked {
                observed.push(Observed { form, answer, forward: false });
            }
        }
        if let Some(steps) = chain(&story) {
            let path = steps.iter().map(|(link, forward)| (forms.number(&story.links[*link].form) as usize, if *forward { 1 } else { -1 })).collect();
            read.push(SpaceLesson { path, answer });
        }
    }
    if read.len() < SPACE_LEAST_LESSONS || observed.is_empty() {
        return Err(SPACE_NO_READ.to_string());
    }
    let order: Vec<usize> = answer_counts.by_count().iter().map(|(answer, _)| *answer).collect();
    let Some(steps) = answer_steps(&observed, &forms, &read, answers.len(), &order) else { return Err(SPACE_NO_STEPS.to_string()) };
    let mut seen: Vec<Vec<usize>> = vec![Vec::new(); forms.len()];
    observed.iter().enumerate().for_each(|(at, one)| seen[one.form].push(at));
    let mut meanings: Vec<Option<Offset>> = seen.iter().map(|seen| form_step(seen, &observed, &steps)).collect();
    let fixed: Vec<bool> = meanings.iter().map(Option::is_some).collect();
    let mut uses: Vec<Vec<usize>> = vec![Vec::new(); forms.len()];
    for (number, lesson) in read.iter().enumerate() {
        for (form, _) in &lesson.path {
            if !uses[*form].contains(&number) {
                uses[*form].push(number);
            }
        }
    }
    let first_guess = guessed_steps(&forms, &WordGuess::new(&known_steps(&forms, &meanings)));
    for (form, step) in meanings.iter_mut().enumerate() {
        if step.is_none() && !uses[form].is_empty() {
            *step = Some(first_guess[form]);
        }
    }
    let search = LinkSearch { lessons: &read, steps: &steps, uses, fixed };
    search.ascent(&mut meanings, &first_guess);
    search.walk(&mut meanings, &first_guess);
    let later_guess = guessed_steps(&forms, &WordGuess::new(&known_steps(&forms, &meanings)));
    search.ascent(&mut meanings, &later_guess);
    let guess = WordGuess::new(&known_steps(&forms, &meanings));
    let answer_words_of: Vec<(Offset, Vec<String>)> = order
        .iter()
        .filter_map(|answer| steps[*answer].map(|step| (step, answers.name(*answer as u32).split(SPACE_BETWEEN).map(str::to_string).collect())))
        .collect();
    Ok(Box::new(SpaceModel { form_words, names, forms, meanings, guess, answers: answer_words_of, default: frequent_answer(lessons.cases).0 }))
}
because!(learn_space, UnitSteps, "the tool on lessons: the things and links of each lesson, the step of each answer, the steps of the forms that join the asked things directly, and the steps of the other forms by the search; the tool says that it does not read a kind where it reads too few lessons or the answers cannot name steps");
