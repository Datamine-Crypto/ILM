use crate::tools::pattern::classes::KIND_MARK;
use crate::tools::state::read::{FOUR_ARGS, Name, ONE_ARG, StateReading, THREE_ARGS, TWO_ARGS};
use crate::tools::state::table::{OpChoice, Holder, Op, OpStep};
use patterns::{because, source};

pub struct ProgramChoices;
source!(ProgramChoices, "the choices of a program are all operations that the form of a unit allows on its slots, its spans of answers, each word of a kind, and the last holders, and all readings that a form of question allows on its slots and runs of slots; the search tries them and keeps what makes more lessons right");

pub const MOST_ARGS: usize = 8;
because!(MOST_ARGS, ProgramChoices, "a question with more arguments than this gets no reading: it would have thousands of readings, and no question of the public sets needs as many");

const RUN_SIZES: [usize; 2] = [2, 3];
because!(RUN_SIZES, ProgramChoices, "the sizes of a run of slots next to each other that is one argument or one value: a name of a thing with a colour or a size");

fn is_slot(entry: u32) -> bool {
    entry & KIND_MARK != 0
}
because!(is_slot, ProgramChoices, "a place of a form that is a kind, which any word of the kind fills");

pub fn slots(shape: &[u32]) -> Vec<(u8, u8)> {
    (0..shape.len()).filter(|at| is_slot(shape[*at])).map(|at| (at as u8, at as u8 + 1)).collect()
}
because!(slots, ProgramChoices, "the slots of a form, each as a span of one token");

pub fn holders(shape: &[u32], spans: &[(u8, u8)]) -> Vec<Holder> {
    let found = slots(shape);
    let mut out: Vec<Holder> = found.iter().map(|(from, to)| Holder::Span(*from, *to)).collect();
    out.extend(spans.iter().filter(|span| !found.contains(span)).map(|(from, to)| Holder::Span(*from, *to)));
    out.extend(eachs(shape));
    out.push(Holder::Last);
    out
}
because!(holders, ProgramChoices, "what the first argument of an operation can be: a slot, a span where answers of lessons stood, each word of a kind that the form holds more than once, or the last holders");

pub fn values(shape: &[u32], spans: &[(u8, u8)]) -> Vec<(u8, u8)> {
    let found = slots(shape);
    let mut out = found.clone();
    for size in RUN_SIZES {
        for (from, _) in &found {
            if (0..size as u8).all(|step| found.iter().any(|(held, _)| *held == from + step)) {
                out.push((*from, from + size as u8));
            }
        }
    }
    let more: Vec<(u8, u8)> = spans.iter().filter(|span| !out.contains(span)).copied().collect();
    out.extend(more);
    out
}
because!(values, ProgramChoices, "what the value of a set can be: a slot, a run of slots next to each other, or a span where answers of lessons stood");

pub fn eachs(shape: &[u32]) -> Vec<Holder> {
    let mut kinds: Vec<(u32, usize)> = Vec::new();
    for entry in shape.iter().filter(|entry| is_slot(**entry)) {
        match kinds.iter_mut().find(|(kind, _)| *kind == entry & !KIND_MARK) {
            Some(found) => found.1 += 1,
            None => kinds.push((entry & !KIND_MARK, 1)),
        }
    }
    kinds.sort_unstable();
    kinds.into_iter().filter(|(_, count)| *count > 1).map(|(kind, _)| Holder::Each(kind)).collect()
}
because!(eachs, ProgramChoices, "each word of a kind that a form holds more than once, as one holder: an operation done for each of them, as all things of a list are put down");

pub fn sets(shape: &[u32], spans: &[(u8, u8)]) -> Vec<OpChoice> {
    let mut out: Vec<OpChoice> = Vec::new();
    let found = values(shape, spans);
    for holder in holders(shape, spans) {
        for value in &found {
            let fits = match holder {
                Holder::Last => true,
                Holder::Each(kind) => !(value.0..value.1).any(|at| shape[at as usize] == KIND_MARK | kind),
                Holder::Span(from, to) => to <= value.0 || value.1 <= from,
            };
            if fits {
                out.push(OpChoice { op: Op::Set, first: holder, second: Some(*value) });
            }
        }
    }
    out
}
because!(sets, ProgramChoices, "all choices of a set for a form: a holder and a value that do not overlap");

pub fn choices(shape: &[u32], spans: &[(u8, u8)], copies: bool) -> Vec<OpChoice> {
    let found = slots(shape);
    let mut out = sets(shape, spans);
    let single: &[Op] = if copies { &[Op::Drop, Op::Release, Op::Keep, Op::Forget] } else { &[Op::Drop, Op::Release] };
    for op in single {
        for holder in holders(shape, spans) {
            let allowed = matches!(op, Op::Drop | Op::Release)
                || match holder {
                    Holder::Span(from, to) => found.contains(&(from, to)),
                    Holder::Last | Holder::Each(_) => true,
                };
            if allowed {
                out.push(OpChoice { op: *op, first: holder, second: None });
            }
        }
    }
    for op in [Op::Swap, Op::SwapHolders] {
        for (place, first) in found.iter().enumerate() {
            for second in &found[place + 1..] {
                out.push(OpChoice { op, first: Holder::Span(first.0, first.1), second: Some(*second) });
            }
        }
    }
    out
}
because!(choices, ProgramChoices, "all choices of a form: the sets, the operations on one holder (keep and forget only where a reading of copies is searched, since they change no other reading), and the swaps of two slots");

pub fn compile(shape: &[u32], choice: Option<OpChoice>) -> Vec<OpStep> {
    let Some(choice) = choice else { return Vec::new() };
    let firsts: Vec<Option<(u8, u8)>> = match choice.first {
        Holder::Last => vec![None],
        Holder::Each(kind) => (0..shape.len()).filter(|at| shape[*at] == KIND_MARK | kind).map(|at| Some((at as u8, at as u8 + 1))).collect(),
        Holder::Span(from, to) => vec![Some((from, to))],
    };
    firsts.into_iter().map(|first| OpStep { op: choice.op, first, second: choice.second }).collect()
}
because!(compile, ProgramChoices, "a choice as simple operations: one for its holder, or one for each word of the kind of an each");

pub fn readings_of(shape: &[u32]) -> Vec<StateReading> {
    let found: Vec<usize> = (0..shape.len()).filter(|at| is_slot(shape[*at])).collect();
    let mut args: Vec<(u8, u8)> = found.iter().map(|at| (*at as u8, *at as u8 + 1)).collect();
    for size in RUN_SIZES {
        for at in &found {
            if (0..size).all(|step| found.contains(&(at + step))) {
                args.push((*at as u8, (at + size) as u8));
            }
        }
    }
    if args.len() > MOST_ARGS {
        return Vec::new();
    }
    let mut out: Vec<StateReading> = Vec::new();
    for arg in &args {
        out.extend(ONE_ARG.iter().map(|name| StateReading { name: *name, args: vec![*arg] }));
    }
    for a in &args {
        for b in &args {
            if a.1 <= b.0 || b.1 <= a.0 {
                out.extend(TWO_ARGS.iter().map(|name| StateReading { name: *name, args: vec![*a, *b] }));
            }
        }
    }
    let mut ordered = args.clone();
    ordered.sort_unstable();
    for a in &ordered {
        for b in &ordered {
            if a.1 > b.0 {
                continue;
            }
            for c in &ordered {
                if b.1 > c.0 {
                    continue;
                }
                out.extend(THREE_ARGS.iter().map(|name| StateReading { name: *name, args: vec![*a, *b, *c] }));
                for name in FOUR_ARGS {
                    out.extend(ordered.iter().filter(|d| c.1 <= d.0).map(|d| StateReading { name, args: vec![*a, *b, *c, *d] }));
                }
            }
        }
    }
    out
}
because!(readings_of, ProgramChoices, "all readings of a form of question: each argument a slot or a run of slots; two arguments in both orders, three and four in the order of the question; nothing for a question with too many arguments");

pub fn empty_result(name: Name) -> crate::tools::state::read::StateOut {
    use crate::tools::state::read::StateOut;
    match name {
        Name::Members | Name::MembersRev | Name::MembersFirst | Name::Count => StateOut::Count(0),
        Name::Is | Name::IsTop | Name::IsFirst | Name::IsKept | Name::IsKept2 | Name::IsOlder => StateOut::Truth(false),
        _ => StateOut::Unknown,
    }
}
because!(empty_result, ProgramChoices, "the result of a reading when the table holds none of its arguments: no member, false, or nothing known");
