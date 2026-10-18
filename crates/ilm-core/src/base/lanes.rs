use patterns::{because, source};
use std::cell::{Cell, RefCell};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};

pub struct SharedQueue;
source!(SharedQueue, "pieces of work that do not depend on each other, done by all hardware threads from one shared counter: each thread takes the next piece that nobody has, so a long piece holds one thread and the others go on; the results are put back in the order of the pieces, so the outcome is the same for every number of threads");

const LANES_NAME: &str = "ILM_LANES";
because!(LANES_NAME, SharedQueue, "the environment variable that sets the number of threads, for a machine that is shared or for a check that the outcome does not depend on it");

pub struct WorkBudget;
source!(WorkBudget, "a bound on the work of a learning, counted in work and never in time, so that a learning stops at the same point at any number of threads, on any machine and cold or warm: the chains of a learning add the facts that they read (and trial chains the fillings that they took) to the scope of work of their thread, a kept chain as much as when it was made, and so do the replies by which the judge of a first reading weighs a reading, which a run of pieces gives to the threads that it starts; a learning that a memo or a store keeps runs in a scope of its own, bounded by the work left of the learning that asks, and is kept only when it did not stop, with its work, which each later learning that takes it adds again; a later learning takes it only when that work is within its own work left, as then no step of it passed that bound either, so that what a learning gives depends on its inputs and its work left alone and never on which learning ran first; a learner looks at the work only between its steps, where every trial before has ended, so that where it stops does not depend on which thread ran first");

pub const MOST_LEARNING_WORK: usize = 100_000_000;
because!(MOST_LEARNING_WORK, WorkBudget, "the most work of the chains of one scope of a learning (the facts that they read and the fillings that its trial chains took), past which its learners stop at their next step: five times the most that a learning of the board needs (nlgraph_cycle's), so that only a search that runs away stops at it, while a search that found nothing for minutes on a copy of the library with a text left out (babi03 with the synonyms of movement out, at eight times the bound) stops within a minute");

pub const BUDGET_STOPPED: &str = "STOPPED AT THE BUDGET of the learning: work of its chains";
because!(BUDGET_STOPPED, WorkBudget, "the mark of a learning whose learners stopped at the bound of its work, before that work, the same on the board and in the reports of a discovery, so that a search that ran away shows where it is read and the time it would have taken does not");

#[derive(Debug)]
pub struct WorkScope {
    done: AtomicUsize,
    most: usize,
    stopped: AtomicBool,
    stored: bool,
    taken: std::sync::Mutex<Vec<String>>,
}
because!(WorkScope, WorkBudget, "the work of one scope of a learning: the work done so far by its chains and the learnings it took, the most it may do, and whether a learner in it stopped at that bound; whether its learners may take lines of the known-good store, and the kinds whose stored lines they took");

thread_local! {
    static WORK_SCOPE: RefCell<Option<Arc<WorkScope>>> = const { RefCell::new(None) };
}
because!(WORK_SCOPE, WorkBudget, "the scope of work of the learning that this thread works for, given to the threads that a run of pieces starts");

struct ScopeBack(Option<Arc<WorkScope>>);
because!(ScopeBack, WorkBudget, "the scope of work that a thread had before another was set for a while, set back when the while ends, also when its work ends with a fault");

impl Drop for ScopeBack {
    fn drop(&mut self) {
        let before = self.0.take();
        WORK_SCOPE.with(|slot| *slot.borrow_mut() = before);
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Worked<T> {
    pub value: T,
    pub work: usize,
    pub stopped: bool,
    pub taken: Vec<String>,
}
because!(Worked, WorkBudget, "what a learning gave, with the work of the chains that it cost, whether a learner in it stopped at the bound, and the kinds whose lines of the known-good store it took, kept together so that a memo or a store that gives it again adds the same work and the same kinds to the learning that takes it. again gives the value and adds the work and the kinds");

impl<T: Clone> Worked<T> {
    pub fn again(&self) -> T {
        add_scope_work(self.work, self.stopped);
        add_scope_taken(&self.taken);
        self.value.clone()
    }
}

pub fn work_left() -> usize {
    work_scope().map_or(MOST_LEARNING_WORK, |scope| scope.most.saturating_sub(scope.done.load(Ordering::Relaxed)))
}
because!(work_left, WorkBudget, "the work that the learning of this thread may still do before its learners stop, or the bound of one learning outside any learning");

pub fn learned_within<T: Clone>(kept: Option<&Worked<T>>, learn: impl FnOnce() -> T) -> (T, Option<Worked<T>>) {
    let left = work_left();
    if let Some(kept) = kept.filter(|kept| !kept.stopped && kept.work <= left) {
        return (kept.again(), None);
    }
    let worked = in_work_scope(left, learn);
    (worked.value.clone(), (!worked.stopped).then_some(worked))
}
because!(learned_within, WorkBudget, "a learning that a memo or a store keeps: the kept one, its work added again, when it did not stop and its work is within the work left, as then no step of it passed the work left either and it is what the learning would give now; else learned now in a scope of its own bounded by the work left, so that its learners stop where the learning that asks would stop, and given with what to keep, nothing when it stopped, as what a stopped learning gives holds only for the work that was left to it");

pub fn work_scope() -> Option<Arc<WorkScope>> {
    WORK_SCOPE.with(|slot| slot.borrow().clone())
}
because!(work_scope, WorkBudget, "the scope of work of this thread, if it works for a learning");

pub fn with_work_scope<R>(scope: Option<Arc<WorkScope>>, run: impl FnOnce() -> R) -> R {
    let _back = ScopeBack(WORK_SCOPE.with(|slot| slot.replace(scope)));
    run()
}
because!(with_work_scope, WorkBudget, "a piece of work done in the given scope of work, the scope of the thread set back after it");

pub fn in_work_scope<R>(most: usize, run: impl FnOnce() -> R) -> Worked<R> {
    in_work_scope_offering(most, store_offered(), run)
}

pub fn store_offered() -> bool {
    work_scope().is_none_or(|scope| scope.stored)
}
because!(store_offered, WorkBudget, "whether the learning that this thread works for may take lines of the known-good store: a learning that must stand on its own lessons alone gets none, and every scope inside it the same");

pub fn in_work_scope_offering<R>(most: usize, stored: bool, run: impl FnOnce() -> R) -> Worked<R> {
    let inner = Arc::new(WorkScope { done: AtomicUsize::new(0), most, stopped: AtomicBool::new(false), stored, taken: std::sync::Mutex::new(Vec::new()) });
    let value = with_work_scope(Some(Arc::clone(&inner)), run);
    let (work, stopped) = (inner.done.load(Ordering::Relaxed), inner.stopped.load(Ordering::Relaxed));
    let taken = inner.taken.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    add_scope_work(work, stopped);
    add_scope_taken(&taken);
    Worked { value, work, stopped, taken }
}
because!(in_work_scope, WorkBudget, "a piece of learning in a scope of work of its own (in_work_scope_offering), with the store offered as in the scope around it");

because!(in_work_scope_offering, WorkBudget, "a piece of learning done in a scope of work of its own with the given bound, its work, its stop and the kinds whose stored lines it took added to the scope around it after it: a learning whose result is kept for others runs this way, so that it gives the same whatever the learning that asked first had done before");

pub fn add_scope_work(work: usize, stopped: bool) {
    if let Some(scope) = work_scope() {
        scope.done.fetch_add(work, Ordering::Relaxed);
        if stopped {
            scope.stopped.store(true, Ordering::Relaxed);
        }
    }
}
because!(add_scope_work, WorkBudget, "work added to the scope of this thread, with a stop at the bound that it carried");

pub fn note_store_taken(kind: &str) {
    add_scope_taken(&[kind.to_string()]);
}
because!(note_store_taken, WorkBudget, "the kind whose stored lines a learning of this thread took, kept by its scope: a learning that takes them stands on more than its own lessons, so the board names that kind beside the lessons of the task");

fn add_scope_taken(kinds: &[String]) {
    let Some(scope) = work_scope() else { return };
    let mut taken = scope.taken.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    for kind in kinds {
        if let Err(place) = taken.binary_search(kind) {
            taken.insert(place, kind.clone());
        }
    }
}
because!(add_scope_taken, WorkBudget, "kinds of stored lines added to the scope of this thread, each once and in the order of their names: threads add them in any order, and a run must read the same at any number of threads");

pub fn scope_spent() -> bool {
    work_scope().is_some_and(|scope| {
        let spent = scope.done.load(Ordering::Relaxed) > scope.most;
        if spent {
            scope.stopped.store(true, Ordering::Relaxed);
        }
        spent
    })
}
because!(scope_spent, WorkBudget, "whether the scope of this thread did more work than its bound, marked as stopped when it did: a learner asks it only between its steps, where every trial before has ended, so the answer does not depend on the order of the threads");

pub fn hardware_lanes() -> usize {
    let asked = std::env::var(LANES_NAME).ok().and_then(|text| text.parse::<usize>().ok()).filter(|lanes| *lanes > 0);
    asked.unwrap_or_else(|| std::thread::available_parallelism().map(usize::from).unwrap_or(1))
}
because!(hardware_lanes, SharedQueue, "the number of threads of a run: what the environment asks for, else every hardware thread of the machine");

thread_local! {
    static IN_QUEUE: Cell<bool> = const { Cell::new(false) };
}
because!(IN_QUEUE, SpareLanes, "whether this thread works for a queue that holds a thread for every hardware thread of the machine and whose waiting threads lend their places (a thread of such a queue, or a helper that one of them started): a run of pieces that this thread asks for runs on it with helpers on the lent places, as a set of threads of its own for each run finds no free hardware thread and waits for its slowest thread on a busy machine; a queue of fewer lanes (the gate's) leaves its runs their own threads, which the hardware threads that it does not use can take");

struct QueueMark(bool);
because!(QueueMark, SpareLanes, "the mark of a thread of a queue (IN_QUEUE) set for the while that the thread works for it, and the mark of before set back when that while ends, also when a piece ends with a fault, so that a thread never keeps the mark of a queue that it left");

impl QueueMark {
    fn set(on: bool) -> QueueMark {
        QueueMark(IN_QUEUE.with(|flag| flag.replace(on)))
    }
}

impl Drop for QueueMark {
    fn drop(&mut self) {
        let before = self.0;
        IN_QUEUE.with(|flag| flag.set(before));
    }
}

fn machine_lanes() -> usize {
    std::thread::available_parallelism().map(usize::from).unwrap_or(1)
}
because!(machine_lanes, SpareLanes, "the hardware threads of the machine, whatever the environment asks a run to use: a queue with as many lanes uses the whole machine");

pub fn on_lanes<T: Send>(pieces: usize, lanes: usize, work: &(dyn Fn(usize) -> T + Sync)) -> Vec<T> {
    let lanes = lanes.clamp(1, pieces.max(1));
    if lanes == 1 {
        return (0..pieces).map(work).collect();
    }
    if IN_QUEUE.with(Cell::get) {
        return on_spare_lanes(pieces, 1, work);
    }
    let next = AtomicUsize::new(0);
    let lane_scope = work_scope();
    let mut done: Vec<(usize, T)> = std::thread::scope(|scope| {
        let hands: Vec<_> = (0..lanes)
            .map(|_| {
                scope.spawn(|| {
                    with_work_scope(lane_scope.clone(), || {
                        let mut mine: Vec<(usize, T)> = Vec::new();
                        loop {
                            let piece = next.fetch_add(1, Ordering::Relaxed);
                            if piece >= pieces {
                                break;
                            }
                            mine.push((piece, work(piece)));
                        }
                        mine
                    })
                })
            })
            .collect();
        hands.into_iter().flat_map(|hand| hand.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault))).collect()
    });
    done.sort_by_key(|(piece, _)| *piece);
    done.into_iter().map(|(_, result)| result).collect()
}
because!(on_lanes, SharedQueue, SpareLanes, "the results of the pieces in the order of the pieces, each piece done once by the thread that took it; a fault in a piece ends the whole call with that fault, so that no result is lost without a sign; asked from a thread of a queue that lends its places (IN_QUEUE), the pieces run as on spare lanes with one lane");

pub struct SpareLanes;
source!(SpareLanes, "a thread of a queue that waits with no piece lends its place: a run of one thread that is still at work can start a helper thread for each place that is lent, so that the last long piece of a queue does not hold the whole run while the other threads wait; the places are counted, so that no more threads work than the queue has");

static SPARE_LANES: AtomicIsize = AtomicIsize::new(0);
because!(SPARE_LANES, SpareLanes, "the places of waiting threads of this program that no helper has taken; a thread that stops waiting takes its place back even when a helper holds it, so the count is below nothing until that helper ends");

fn borrow_spare() -> bool {
    SPARE_LANES.try_update(Ordering::AcqRel, Ordering::Acquire, |spare| (spare > 0).then(|| spare - 1)).is_ok()
}
because!(borrow_spare, SpareLanes, "whether a place of a waiting thread is free, and takes it when it is");

fn pieces_from<T>(next: &AtomicUsize, pieces: usize, work: &(dyn Fn(usize) -> T + Sync)) -> Vec<(usize, T)> {
    let mut mine: Vec<(usize, T)> = Vec::new();
    loop {
        let piece = next.fetch_add(1, Ordering::Relaxed);
        if piece >= pieces {
            return mine;
        }
        mine.push((piece, work(piece)));
    }
}
because!(pieces_from, SpareLanes, "the results of the pieces that one thread takes from the shared counter until none is left, each with the place of its piece");

pub fn on_spare_lanes<T: Send>(pieces: usize, lanes: usize, work: &(dyn Fn(usize) -> T + Sync)) -> Vec<T> {
    if lanes > 1 {
        return on_lanes(pieces, lanes, work);
    }
    let next = AtomicUsize::new(0);
    let lane_scope = work_scope();
    let in_queue = IN_QUEUE.with(Cell::get);
    let mut done: Vec<(usize, T)> = std::thread::scope(|scope| {
        let mut hands = Vec::new();
        let mut mine: Vec<(usize, T)> = Vec::new();
        loop {
            while next.load(Ordering::Relaxed) + 1 < pieces && borrow_spare() {
                hands.push(scope.spawn(|| {
                    let _mark = QueueMark::set(in_queue);
                    let theirs = with_work_scope(lane_scope.clone(), || pieces_from(&next, pieces, work));
                    SPARE_LANES.fetch_add(1, Ordering::AcqRel);
                    theirs
                }));
            }
            let piece = next.fetch_add(1, Ordering::Relaxed);
            if piece >= pieces {
                break;
            }
            mine.push((piece, work(piece)));
        }
        mine.extend(hands.into_iter().flat_map(|hand| hand.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault))));
        mine
    });
    done.sort_by_key(|(piece, _)| *piece);
    done.into_iter().map(|(_, result)| result).collect()
}
because!(on_spare_lanes, SpareLanes, SharedQueue, "the results of the pieces in the order of the pieces: with many threads asked, on all of them; with one thread asked, on this thread and on a helper for each place that a waiting thread of a queue lends while pieces are left; each piece is done once and the results are put back in order, so the outcome does not depend on which thread did a piece");

pub struct FollowUps;
source!(FollowUps, "pieces of work where a finished piece can ask for one more piece that needs its result: a thread takes an asked piece before the next piece of the first list, in the order of the asks, so that a piece and the piece it asks for run as one chain and a chain that starts early ends early, and the pieces of the first list in their order otherwise; the results are put back by the place of the piece, so the outcome is the same for every number of threads");

struct QueueState {
    next: usize,
    asked: std::collections::VecDeque<usize>,
    running: usize,
    fault: bool,
}
because!(QueueState, FollowUps, "what the threads share: the next piece of the first list, the asked pieces that no thread has taken, the pieces that threads work on, and whether a piece ended with a fault, after which no thread takes more work");

enum LaneTask {
    First(usize),
    Asked(usize),
}
because!(LaneTask, FollowUps, "the piece that a thread takes: a piece of the first list, or the piece that a finished piece of the first list asked for");

type LaneResults<T, U> = (Vec<(usize, T)>, Vec<(usize, U)>, Option<Box<dyn std::any::Any + Send>>);
because!(LaneResults, FollowUps, "what one thread gives back: the results of its pieces of the first list and of its asked pieces, each with the place of its piece, and the fault of a piece when one ended with a fault");

fn next_task(state: &std::sync::Mutex<QueueState>, ready: &std::sync::Condvar, pieces: usize) -> Option<LaneTask> {
    let mut held = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut lent = false;
    let task = loop {
        if held.fault {
            break None;
        }
        if let Some(piece) = held.asked.pop_front() {
            held.running += 1;
            break Some(LaneTask::Asked(piece));
        }
        if held.next < pieces {
            let piece = held.next;
            held.next += 1;
            held.running += 1;
            break Some(LaneTask::First(piece));
        }
        if held.running == 0 {
            break None;
        }
        if !lent {
            SPARE_LANES.fetch_add(1, Ordering::AcqRel);
            lent = true;
        }
        held = ready.wait(held).unwrap_or_else(std::sync::PoisonError::into_inner);
    };
    if lent {
        SPARE_LANES.fetch_sub(1, Ordering::AcqRel);
    }
    task
}
because!(next_task, FollowUps, SpareLanes, "the next piece for a thread: an asked piece while there is one, else a piece of the first list; a thread with no piece waits while other threads work, because a piece that they end can ask for one more, and it stops when no piece is left and none runs; while it waits it lends its place to the helpers of the runs that still work");

fn task_done(state: &std::sync::Mutex<QueueState>, ready: &std::sync::Condvar, ask: Option<usize>, fault: bool) {
    let mut held = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(piece) = ask {
        held.asked.push_back(piece);
    }
    held.running -= 1;
    held.fault = held.fault || fault;
    ready.notify_all();
}
because!(task_done, FollowUps, "the end of a piece: its ask is put in the queue, and the waiting threads are woken, because there is work for them or the last piece has ended");

pub fn on_lanes_then<T: Send, U: Send>(pieces: usize, lanes: usize, work: &(dyn Fn(usize) -> T + Sync), wants: &(dyn Fn(usize, &T) -> bool + Sync), then: &(dyn Fn(usize) -> U + Sync)) -> (Vec<T>, Vec<Option<U>>) {
    let lanes = lanes.clamp(1, pieces.max(1));
    if lanes == 1 {
        let firsts: Vec<T> = (0..pieces).map(work).collect();
        let asked: Vec<Option<U>> = (0..pieces).map(|piece| wants(piece, &firsts[piece]).then(|| then(piece))).collect();
        return (firsts, asked);
    }
    let state = std::sync::Mutex::new(QueueState { next: 0, asked: std::collections::VecDeque::new(), running: 0, fault: false });
    let whole = lanes >= machine_lanes();
    let ready = std::sync::Condvar::new();
    let lane_scope = work_scope();
    let found: Vec<LaneResults<T, U>> = std::thread::scope(|scope| {
        let hands: Vec<_> = (0..lanes)
            .map(|_| {
                scope.spawn(|| -> LaneResults<T, U> {
                    let _mark = QueueMark::set(whole);
                    let _back = ScopeBack(WORK_SCOPE.with(|slot| slot.replace(lane_scope.clone())));
                    let mut firsts: Vec<(usize, T)> = Vec::new();
                    let mut asked: Vec<(usize, U)> = Vec::new();
                    while let Some(task) = next_task(&state, &ready, pieces) {
                        let ended = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match task {
                            LaneTask::First(piece) => {
                                let result = work(piece);
                                let ask = wants(piece, &result).then_some(piece);
                                firsts.push((piece, result));
                                ask
                            }
                            LaneTask::Asked(piece) => {
                                asked.push((piece, then(piece)));
                                None
                            }
                        }));
                        match ended {
                            Ok(ask) => task_done(&state, &ready, ask, false),
                            Err(fault) => {
                                task_done(&state, &ready, None, true);
                                return (firsts, asked, Some(fault));
                            }
                        }
                    }
                    (firsts, asked, None)
                })
            })
            .collect();
        hands.into_iter().map(|hand| hand.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault))).collect()
    });
    let mut firsts: Vec<(usize, T)> = Vec::with_capacity(pieces);
    let mut asked: Vec<(usize, U)> = Vec::new();
    for (mine, theirs, fault) in found {
        if let Some(fault) = fault {
            std::panic::resume_unwind(fault);
        }
        firsts.extend(mine);
        asked.extend(theirs);
    }
    firsts.sort_by_key(|(piece, _)| *piece);
    let mut by_piece: Vec<Option<U>> = (0..pieces).map(|_| None).collect();
    for (piece, result) in asked {
        by_piece[piece] = Some(result);
    }
    (firsts.into_iter().map(|(_, result)| result).collect(), by_piece)
}
because!(on_lanes_then, FollowUps, "the results of the pieces of the first list in their order, and for each piece the result of the piece that it asked for, or nothing when it asked for none; each piece is done once, and a fault in a piece ends the whole call with that fault");

pub fn shards(items: usize, lanes: usize) -> Vec<std::ops::Range<usize>> {
    let parts = lanes.clamp(1, items.max(1));
    (0..parts).map(|part| items * part / parts..items * (part + 1) / parts).collect()
}
because!(shards, SharedQueue, "the items cut into as many runs as there are threads, each run of nearly the same length and in the order of the items; counts that add are made for each run and summed in this order");
