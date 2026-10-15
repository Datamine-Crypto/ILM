use crate::base::exact::Amount;
use crate::base::keys::{KeyNumbers, key_field};
use crate::tools::amount::calcs::{AMOUNT_PAIR, AMOUNT_TRIPLE, AmountCalc};
use crate::tools::amount::tokens::{AmountCut, AmountMarks};
use patterns::{because, source};

pub struct OperandForms;
source!(OperandForms, "the form of a place in a text where two or three numbers follow one another: the token before the first number, one token of each gap between them, and the token after the last one, each of which can be left open. One place gives several forms, from these exact neighbours to any numbers that follow one another, and counting says which form stands in many lessons and still means one calculation");

pub struct ShownByExamples;
source!(ShownByExamples, "a calculation that the examples inside a case show: a told sentence with the tokens of the question and its result at the end is an example, and a calculation that fits every example of the case up to one fixed amount is the one that the question asks for");

pub(super) const AMOUNT_GAP_ENDS: usize = 4;
because!(AMOUNT_GAP_ENDS, OperandForms, "of a long gap between two operands, this many tokens at each end can stand in a form: the word that says the calculation stands near one of the two numbers");

pub(super) const AMOUNT_MOST_FORMS: usize = 3000;
because!(AMOUNT_MOST_FORMS, OperandForms, "of a case with many numbers, the forms of its first places up to this count: more forms cost time and memory, and a rule seldom needs a later place");

pub(super) fn amount_gap(between: &[u32], marks: &AmountMarks, out: &mut Vec<u32>) {
    out.clear();
    let ends = AMOUNT_GAP_ENDS.min(between.len());
    for token in between[..ends].iter().chain(&between[between.len() - ends..]) {
        if !out.contains(token) {
            out.push(*token);
        }
    }
    if out.is_empty() {
        out.push(marks.none);
    }
    if !out.contains(&marks.open) {
        out.push(marks.open);
    }
}
because!(amount_gap, OperandForms, "the tokens that can stand for one gap between two operands in a form: each different token of its first and of its last tokens, the mark of no token for an empty gap, and the open mark");

#[derive(Debug, Default)]
pub(super) struct AmountForms {
    pub(super) seen: KeyNumbers,
    pub(super) owner: Vec<u32>,
    pub(super) windows: Vec<(u32, u32)>,
    pub(super) gap: Vec<u32>,
    pub(super) middles: Vec<u128>,
    pub(super) wider: Vec<u128>,
}
because!(AmountForms, OperandForms, "the forms of one case, each with the place that it points at: its first place in the text. fill walks the places of two operands and then of three, from the left, and gives each place its forms: the token before or the open mark, one token of each gap or the open mark, the token after or the open mark; a form that an earlier place has stays with that place; when the case has more forms than the limit, the later places give none");

impl AmountForms {
    pub(super) fn fill(&mut self, cut: &AmountCut, marks: &AmountMarks) {
        self.seen.forget();
        self.owner.clear();
        self.windows.clear();
        let numbers = cut.spots.len();
        for size in [AMOUNT_PAIR, AMOUNT_TRIPLE] {
            if numbers < size {
                continue;
            }
            for from in 0..=numbers - size {
                if self.seen.keys().len() > AMOUNT_MOST_FORMS {
                    return;
                }
                let first = cut.spots[from] as usize;
                let last = cut.spots[from + size - 1] as usize;
                let left = if first > 0 { cut.words[first - 1] } else { marks.first };
                let right = cut.words.get(last + 1).copied().unwrap_or(marks.last);
                self.middles.clear();
                self.middles.push(0);
                for gap in 1..size {
                    amount_gap(&cut.words[cut.spots[from + gap - 1] as usize + 1..cut.spots[from + gap] as usize], marks, &mut self.gap);
                    self.wider.clear();
                    for middle in &self.middles {
                        for token in &self.gap {
                            self.wider.push(middle | key_field(*token, gap));
                        }
                    }
                    std::mem::swap(&mut self.middles, &mut self.wider);
                }
                let window = self.windows.len() as u32;
                self.windows.push((from as u32, size as u32));
                for middle in &self.middles {
                    for (before, behind) in [(left, right), (marks.open, right), (left, marks.open), (marks.open, marks.open)] {
                        let key = middle | key_field(before, 0) | key_field(behind, size);
                        let fresh = self.seen.keys().len();
                        if self.seen.number(key) as usize == fresh {
                            self.owner.push(window);
                        }
                    }
                }
            }
        }
    }
}

pub(super) fn amount_examples(cut: &AmountCut, marks: &AmountMarks, pairs: &[AmountCalc], shown: &mut Vec<(usize, Amount)>) {
    shown.clear();
    let Some(ask) = cut.words.iter().rposition(|word| *word == marks.ask) else { return };
    let asked = cut.spots.partition_point(|spot| (*spot as usize) < ask);
    if cut.spots.len() - asked != AMOUNT_PAIR {
        return;
    }
    let want = &cut.words[ask + 1..];
    let mut examples: Vec<usize> = Vec::new();
    let mut start = 0;
    for place in 0..ask {
        if cut.words[place] != marks.end {
            continue;
        }
        let told = &cut.words[start..place];
        if told.len() == want.len() + 1 && told[..want.len()] == *want && told[want.len()] == marks.number {
            examples.push(cut.spots.partition_point(|spot| (*spot as usize) < start));
        }
        start = place + 1;
    }
    if examples.is_empty() {
        return;
    }
    for (number, calc) in pairs.iter().enumerate() {
        let mut more: Option<Amount> = None;
        let mut one = true;
        for from in &examples {
            let given = &cut.values[*from..*from + AMOUNT_PAIR];
            let apart = calc.run(given).zip(cut.values[*from + AMOUNT_PAIR]).and_then(|(got, result)| result.minus(got));
            one = apart.is_some() && (more.is_none() || more == apart);
            if !one {
                break;
            }
            more = apart;
        }
        if !one {
            continue;
        }
        if let Some(found) = calc.run(&cut.values[asked..]).zip(more).and_then(|(got, apart)| got.plus(apart)) {
            shown.push((number, found));
        }
    }
}
because!(amount_examples, ShownByExamples, "the calculations that the examples inside a case allow, each with its result for the question: a told sentence is an example of the question when its tokens are those of the question with one more number at the end; a calculation of one step is allowed when the last number of each example minus the result on its other numbers is one and the same amount, and its result for the question is the calculation with that amount added");
