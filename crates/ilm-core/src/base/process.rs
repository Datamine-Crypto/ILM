use patterns::{because, source};
use std::sync::OnceLock;

pub struct ProcessReading;
source!(ProcessReading, "what the operating system says of a running program: the CPU time of all its threads, in user and in system mode, and the largest private memory that it held; the allocator that the program is linked with, mimalloc, reads it in one call, mi_process_info, which fills eight numbers through pointers that the caller gives (the elapsed, user and system milliseconds, the current and peak resident memory, the current and peak private memory, and the page faults), so that a run can tell its own cost apart from the load of other programs on the machine");

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProcessUse {
    pub cpu_seconds: f64,
    pub peak_bytes: usize,
}
because!(ProcessUse, ProcessReading, "the CPU seconds of a run so far and the largest private memory that it held, in bytes");

pub type ProcessInfoCall = unsafe extern "C" fn(*mut usize, *mut usize, *mut usize, *mut usize, *mut usize, *mut usize, *mut usize, *mut usize);
because!(ProcessInfoCall, ProcessReading, "the call of the allocator that reads the use of the process, as the program that is linked with the allocator gives it");

static READER: OnceLock<ProcessInfoCall> = OnceLock::new();
because!(READER, ProcessReading, "the call that reads the use of the process, which the program gives once at its start; a library alone has none");

const MILLISECONDS: f64 = 1000.0;
because!(MILLISECONDS, ProcessReading, "the milliseconds of a second: the allocator gives its times in milliseconds");

pub fn read_process_use_with(call: ProcessInfoCall) {
    let _ = READER.set(call);
}
because!(read_process_use_with, ProcessReading, "the call that reads the use of the process, given once by the program at its start");

pub fn use_from(call: ProcessInfoCall) -> Option<ProcessUse> {
    let (mut elapsed, mut user, mut system, mut resident, mut peak_resident, mut private, mut peak_private, mut faults) = (usize::default(), usize::default(), usize::default(), usize::default(), usize::default(), usize::default(), usize::default(), usize::default());
    unsafe { call(&mut elapsed, &mut user, &mut system, &mut resident, &mut peak_resident, &mut private, &mut peak_private, &mut faults) };
    let peak_bytes = if peak_private > usize::default() { peak_private } else { peak_resident };
    (user + system + peak_bytes > usize::default()).then(|| ProcessUse { cpu_seconds: (user + system) as f64 / MILLISECONDS, peak_bytes })
}
because!(use_from, ProcessReading, "the CPU seconds of the process (user and system) and its largest private memory, or its largest resident memory where the system gives no private count, from one call of the allocator; the call is unsafe as foreign code writes through raw pointers, and it is sound because each pointer is to a number of this function that lives through the call, the allocator writes each once and keeps none of them, and the call has no other effect; nothing when the system gave no number");

pub fn process_use() -> Option<ProcessUse> {
    READER.get().and_then(|call| use_from(*call))
}
because!(process_use, ProcessReading, "the use of the process so far, or nothing when the program gave no call to read it");
