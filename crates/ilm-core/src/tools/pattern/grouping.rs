use crate::tools::pattern::classes::ABSENT;
use crate::tools::pattern::hash::CheapMap;
use crate::tools::pattern::striped::hash_of;
use crate::tools::pattern::table::{LOCAL_PLACE, PLACE_BITS, Places};
use patterns::{because, source};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct RadixClusters;
source!(RadixClusters, "the radix clustering of Manegold, Boncz and Kersten: items are first cut into clusters by some bits of their hash and written one after the other, and each cluster is then grouped in a table small enough for the cache of a core, so that no step looks up a key in one large table; the clusters are grouped side by side on all threads");

pub struct DisjointWrites;
source!(DisjointWrites, "writes of many threads into one array where each place is written by one thread only, as in the scatter step of a parallel radix sort: each pattern has its own range of readings, and only the thread of the cluster of that pattern writes into that range");

pub const CLUSTER_COUNT: usize = 256;
because!(CLUSTER_COUNT, RadixClusters, "the clusters of the readings: enough that the table of one cluster stays within the cache of a core and that each thread gets many clusters, few enough that the open end of every cluster stays within the first cache while a run writes its readings");

pub const CLUSTER_SHIFT: u32 = 40;
because!(CLUSTER_SHIFT, RadixClusters, "the bits of a hash below the bits that choose the cluster: the table of a cluster takes its places from the low bits of the hash and its marks from the highest bits, so the cluster comes from bits between them");

#[derive(Clone, Copy, Debug, Default)]
pub struct Logged {
    pub key: u32,
    pub step: u32,
    pub place: u32,
    pub order: u32,
}
because!(Logged, RadixClusters, "one reading as a run writes it: the pattern as a shorter pattern and a step, the place of the answer (with its high bit set, a place that the run saw first), and the number of the reading in its run, which keeps the order of first sight; when its cluster is grouped, the first field takes the number of the pattern in the cluster");

pub struct RunLog {
    pub clusters: Vec<Vec<Logged>>,
    pub starts: Vec<(u32, u32)>,
    pub places: Places,
    pub count: u32,
}
because!(RunLog, RadixClusters, "the readings of one run of lessons, each in the cluster of its pattern and in the order of the run; the number of the first reading of each lesson, with the lesson; the places that the run saw first; and the number of readings. lesson marks the start of a lesson, place gives the number of a place, add writes one reading");

impl Default for RunLog {
    fn default() -> RunLog {
        RunLog { clusters: vec![Vec::new(); CLUSTER_COUNT], starts: Vec::new(), places: Places::default(), count: 0 }
    }
}

pub fn cluster_of(key: u32, step: u32) -> usize {
    (hash_of(&(key, step)) >> CLUSTER_SHIFT) as usize % CLUSTER_COUNT
}
because!(cluster_of, RadixClusters, "the cluster of a pattern as a shorter pattern and a step, from bits of its hash between the bits that a table takes its places from and the bits that it takes its marks from");

impl RunLog {
    pub fn lesson(&mut self, case: u32) {
        self.starts.push((self.count, case));
    }

    pub fn place(&mut self, all: &Places, row: &[u32]) -> u32 {
        all.find(row).unwrap_or_else(|| LOCAL_PLACE | self.places.number_wide(row))
    }

    pub fn add(&mut self, key: u32, step: u32, place: u32) {
        self.clusters[cluster_of(key, step)].push(Logged { key, step, place, order: self.count });
        self.count += 1;
    }
}

pub struct FirstMarks {
    words: Vec<AtomicU64>,
}
because!(FirstMarks, RadixClusters, "a bit for each reading of a count, set where a pattern that is kept was seen first; the clusters set their bits side by side. ranks ends the marking");

pub struct Ranks {
    words: Vec<u64>,
    before: Vec<u64>,
}
because!(Ranks, RadixClusters, "the marks of first sight with the number of set bits before each word: rank gives the number of marks before a reading, which is the number of its pattern among the kept patterns in the order of first sight; total gives the number of kept patterns");

impl FirstMarks {
    pub fn over(readings: u64) -> FirstMarks {
        let bits = u64::from(u64::BITS);
        FirstMarks { words: (0..readings.div_ceil(bits)).map(|_| AtomicU64::new(0)).collect() }
    }

    pub fn mark(&self, at: u64) {
        let bits = u64::from(u64::BITS);
        let one: u64 = 1;
        self.words[(at / bits) as usize].fetch_or(one << (at % bits), Ordering::Relaxed);
    }

    pub fn ranks(self) -> Ranks {
        let words: Vec<u64> = self.words.into_iter().map(AtomicU64::into_inner).collect();
        let mut before: Vec<u64> = Vec::with_capacity(words.len());
        let mut sum: u64 = 0;
        for word in &words {
            before.push(sum);
            sum += u64::from(word.count_ones());
        }
        Ranks { words, before }
    }
}

impl Ranks {
    pub fn rank(&self, at: u64) -> u64 {
        let bits = u64::from(u64::BITS);
        let one: u64 = 1;
        let word = (at / bits) as usize;
        self.before[word] + u64::from((self.words[word] & ((one << (at % bits)) - one)).count_ones())
    }

    pub fn total(&self) -> u64 {
        self.before.last().zip(self.words.last()).map_or(0, |(before, word)| before + u64::from(word.count_ones()))
    }
}

pub struct ClusterKeys {
    pub keys: Vec<(u32, u32)>,
    pub totals: Vec<u32>,
    pub firsts: Vec<u64>,
    pub rows: Vec<Vec<Logged>>,
}
because!(ClusterKeys, RadixClusters, "one cluster after its grouping: its patterns in the order in which the cluster saw them, the readings of each, where each was seen first among all readings of the count, and the readings of the cluster in each run with the number of their pattern in the cluster");

pub fn cluster_keys(mut rows: Vec<Vec<Logged>>, bases: &[u64], least: u32, marks: &FirstMarks) -> ClusterKeys {
    let mut slot_of: CheapMap<(u32, u32), u32> = CheapMap::default();
    let mut keys: Vec<(u32, u32)> = Vec::new();
    let mut totals: Vec<u32> = Vec::new();
    let mut firsts: Vec<u64> = Vec::new();
    for (row, base) in rows.iter_mut().zip(bases) {
        for logged in row.iter_mut() {
            let fresh = keys.len() as u32;
            let slot = *slot_of.entry((logged.key, logged.step)).or_insert(fresh);
            if slot == fresh {
                keys.push((logged.key, logged.step));
                totals.push(0);
                firsts.push(base + u64::from(logged.order));
            }
            totals[slot as usize] += 1;
            logged.key = slot;
        }
    }
    for (total, first) in totals.iter().zip(&firsts) {
        if *total >= least {
            marks.mark(*first);
        }
    }
    ClusterKeys { keys, totals, firsts, rows }
}
because!(cluster_keys, RadixClusters, "the grouping of one cluster: its readings are read run by run in the order of the lessons, so a pattern is seen first where the whole count saw it first; the first reading of each pattern that is kept is marked");

pub struct SharedReadings<'a> {
    at: *mut u32,
    len: usize,
    held: PhantomData<&'a mut [u32]>,
}
because!(SharedReadings, DisjointWrites, "the array of readings of the table while the threads of all clusters write into it: over takes the array for as long as the writing lasts; put writes one reading and stops the program at a place outside the array, and its caller must make sure that no other thread writes or reads that place while the writing lasts");

unsafe impl Sync for SharedReadings<'_> {}

impl<'a> SharedReadings<'a> {
    pub fn over(row: &'a mut [u32]) -> SharedReadings<'a> {
        SharedReadings { at: row.as_mut_ptr(), len: row.len(), held: PhantomData }
    }

    pub unsafe fn put(&self, place: usize, value: u32) {
        assert!(place < self.len);
        unsafe { self.at.add(place).write(value) }
    }
}

pub unsafe fn write_readings(cluster: &ClusterKeys, numbers: &[u32], starts: &[usize], logs: &[RunLog], place_maps: &[Vec<u32>], out: &SharedReadings) {
    let mut cursor: Vec<usize> = numbers.iter().map(|number| if *number == ABSENT { 0 } else { starts[*number as usize] }).collect();
    for ((row, log), place_map) in cluster.rows.iter().zip(logs).zip(place_maps) {
        let mut lesson = 0;
        for logged in row {
            let slot = logged.key as usize;
            if numbers[slot] == ABSENT {
                continue;
            }
            while lesson + 1 < log.starts.len() && log.starts[lesson + 1].0 <= logged.order {
                lesson += 1;
            }
            let place = if logged.place & LOCAL_PLACE != 0 { place_map[(logged.place & !LOCAL_PLACE) as usize] } else { logged.place };
            unsafe { out.put(cursor[slot], (log.starts[lesson].1 << PLACE_BITS) | place) };
            cursor[slot] += 1;
        }
    }
}
because!(write_readings, DisjointWrites, RadixClusters, "the readings of the kept patterns of one cluster written into their ranges, run by run, so the readings of a pattern stand in the order of the lessons; the lesson of a reading is found from the starts of the lessons of its run, and a place that the run saw first takes its number in the table of all places. Its caller must make sure that the calls that run at once write clusters with no pattern in common, so that no two threads write one range");
