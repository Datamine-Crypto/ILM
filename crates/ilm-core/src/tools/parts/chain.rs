use crate::tools::parts::counted::sure_of;
use crate::tools::parts::language::{Word, WordRow};
use crate::tools::parts::ordered::Kept;
use patterns::{because, source};
use std::collections::{HashMap, HashSet};

pub struct StateChains;
source!(StateChains, "a reading of a text as a chain of steps on one state: the one word of the text that the answers hold is the state at the start, each other word is a step that maps a state to the next by a table of its own, the steps are taken in the order of the text or in the other order, and the answer is the row of the states, with the state at the start or without it; a table holds for each state before its step the state after it that the lessons give most often, so the lessons of one step fill the tables that the longer chains use, and the states between the steps of a longer lesson fill them too; words that nearly every lesson holds and no answer holds are marks of the text and no steps");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ChainShape {
    backward: bool,
    with_start: bool,
}
because!(ChainShape, StateChains, "how the steps of a chain are read: in the other order than the text, and whether the answer begins with the state at the start");

const CHAIN_SHAPES: [ChainShape; CHAIN_SHAPE_COUNT] = [
    ChainShape { backward: false, with_start: true },
    ChainShape { backward: false, with_start: false },
    ChainShape { backward: true, with_start: true },
    ChainShape { backward: true, with_start: false },
];
because!(CHAIN_SHAPES, StateChains, "the shapes of a chain in the order in which they are tried, the order of the text first, so that of two shapes that read the same lessons the first one is taken");

const STEP_PAIR: usize = 2;
because!(STEP_PAIR, StateChains, "the state before a step and the state after it, which a lesson gives for the table of the step");

const CHAIN_SHAPE_COUNT: usize = 4;
because!(CHAIN_SHAPE_COUNT, StateChains, "the two orders of the steps by the two forms of the answer");

#[derive(Clone, Debug)]
pub struct Chains {
    states: HashSet<Word>,
    marks: HashSet<Word>,
    tables: HashMap<Word, HashMap<Word, Word>>,
    shape: ChainShape,
}
because!(Chains, StateChains, "a model of chains: the words that are states, the marks of the text, the table of each step, and the shape of the chain. read gives the answer of a text, or nothing when the text is no chain of known steps");

fn chain_parts(words: &[Word], states: &HashSet<Word>, marks: &HashSet<Word>, shape: ChainShape) -> Option<(Word, Vec<Word>)> {
    let starts: Vec<Word> = words.iter().copied().filter(|word| states.contains(word)).collect();
    let [start] = starts.as_slice() else { return None };
    let mut steps: Vec<Word> = words.iter().copied().filter(|word| !states.contains(word) && !marks.contains(word)).collect();
    if shape.backward {
        steps.reverse();
    }
    Some((*start, steps))
}
because!(chain_parts, StateChains, "the state at the start and the steps of a text in the order in which they are taken, when the text holds one state");

fn states_of(answer: &[Word], start: Word, steps: usize, shape: ChainShape) -> Option<Vec<Word>> {
    if shape.with_start {
        (answer.len() == steps + 1 && answer.first() == Some(&start)).then(|| answer.to_vec())
    } else {
        (answer.len() == steps).then(|| [vec![start], answer.to_vec()].concat())
    }
}
because!(states_of, StateChains, "the states of a lesson from the state at the start to the last one, when its answer has the form of the shape");

impl Chains {
    fn learn_shape(lessons: &[(Vec<Word>, WordRow)], states: &HashSet<Word>, marks: &HashSet<Word>, shape: ChainShape) -> Chains {
        let mut counted: HashMap<Word, HashMap<Word, Kept<Word, usize>>> = HashMap::new();
        for (words, answer) in lessons {
            let Some((start, steps)) = chain_parts(words, states, marks, shape) else { continue };
            let Some(row) = states_of(answer, start, steps.len(), shape) else { continue };
            for (step, pair) in steps.iter().zip(row.windows(STEP_PAIR)) {
                counted.entry(*step).or_default().entry(pair[0]).or_default().count(pair[1], 1);
            }
        }
        let tables = counted.into_iter().map(|(step, rows)| (step, rows.into_iter().filter_map(|(state, next)| next.most().map(|(word, _)| (state, *word))).collect())).collect();
        Chains { states: states.clone(), marks: marks.clone(), tables, shape }
    }

    pub fn learn(lessons: &[(Vec<Word>, WordRow)]) -> Option<Chains> {
        let states: HashSet<Word> = lessons.iter().flat_map(|(_, answer)| answer.iter().copied()).collect();
        let mut holding: Kept<Word, usize> = Kept::default();
        for (words, _) in lessons {
            let distinct: HashSet<Word> = words.iter().copied().filter(|word| !states.contains(word)).collect();
            let mut ordered: Vec<Word> = distinct.into_iter().collect();
            ordered.sort_unstable();
            ordered.into_iter().for_each(|word| holding.count(word, 1));
        }
        let marks: HashSet<Word> = holding.pairs().filter(|(_, count)| sure_of(**count, lessons.len())).map(|(word, _)| *word).collect();
        let mut best: Option<(usize, Chains)> = None;
        for shape in CHAIN_SHAPES {
            let model = Chains::learn_shape(lessons, &states, &marks, shape);
            let right = lessons.iter().filter(|(words, answer)| model.read(words).as_ref() == Some(answer)).count();
            if right > 0 && best.as_ref().is_none_or(|(top, _)| right > *top) {
                best = Some((right, model));
            }
        }
        best.map(|(_, model)| model)
    }

    pub fn read(&self, words: &[Word]) -> Option<WordRow> {
        let (start, steps) = chain_parts(words, &self.states, &self.marks, self.shape)?;
        let mut state = start;
        let mut out: WordRow = if self.shape.with_start { vec![start] } else { Vec::new() };
        for step in steps {
            state = *self.tables.get(&step)?.get(&state)?;
            out.push(state);
        }
        Some(out)
    }
}
