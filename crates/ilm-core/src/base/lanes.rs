use patterns::{because, source};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct SharedQueue;
source!(SharedQueue, "pieces of work that do not depend on each other, done by all hardware threads from one shared counter: each thread takes the next piece that nobody has, so a long piece holds one thread and the others go on; the results are put back in the order of the pieces, so the outcome is the same for every number of threads");

const LANES_NAME: &str = "ILM_LANES";
because!(LANES_NAME, SharedQueue, "the environment variable that sets the number of threads, for a machine that is shared or for a check that the outcome does not depend on it");

pub fn hardware_lanes() -> usize {
    let asked = std::env::var(LANES_NAME).ok().and_then(|text| text.parse::<usize>().ok()).filter(|lanes| *lanes > 0);
    asked.unwrap_or_else(|| std::thread::available_parallelism().map(usize::from).unwrap_or(1))
}
because!(hardware_lanes, SharedQueue, "the number of threads of a run: what the environment asks for, else every hardware thread of the machine");

pub fn on_lanes<T: Send>(pieces: usize, lanes: usize, work: &(dyn Fn(usize) -> T + Sync)) -> Vec<T> {
    let lanes = lanes.clamp(1, pieces.max(1));
    if lanes == 1 {
        return (0..pieces).map(work).collect();
    }
    let next = AtomicUsize::new(0);
    let mut done: Vec<(usize, T)> = std::thread::scope(|scope| {
        let hands: Vec<_> = (0..lanes)
            .map(|_| {
                scope.spawn(|| {
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
            .collect();
        hands.into_iter().flat_map(|hand| hand.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault))).collect()
    });
    done.sort_by_key(|(piece, _)| *piece);
    done.into_iter().map(|(_, result)| result).collect()
}
because!(on_lanes, SharedQueue, "the results of the pieces in the order of the pieces, each piece done once by the thread that took it; a fault in a piece ends the whole call with that fault, so that no result is lost without a sign");

pub struct FollowUps;
source!(FollowUps, "pieces of work where a finished piece can ask for one more piece that needs its result: the threads take the pieces of the first list in their order, and when that list is given out they take the asked pieces in the order of the asks, so that the threads that would wait for the last long pieces of the list do the asked pieces; the results are put back by the place of the piece, so the outcome is the same for every number of threads");

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
    loop {
        if held.fault {
            return None;
        }
        if held.next < pieces {
            let piece = held.next;
            held.next += 1;
            held.running += 1;
            return Some(LaneTask::First(piece));
        }
        if let Some(piece) = held.asked.pop_front() {
            held.running += 1;
            return Some(LaneTask::Asked(piece));
        }
        if held.running == 0 {
            return None;
        }
        held = ready.wait(held).unwrap_or_else(std::sync::PoisonError::into_inner);
    }
}
because!(next_task, FollowUps, "the next piece for a thread: a piece of the first list while there is one, else an asked piece; a thread with no piece waits while other threads work, because a piece that they end can ask for one more, and it stops when no piece is left and none runs");

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
    let ready = std::sync::Condvar::new();
    let found: Vec<LaneResults<T, U>> = std::thread::scope(|scope| {
        let hands: Vec<_> = (0..lanes)
            .map(|_| {
                scope.spawn(|| {
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
