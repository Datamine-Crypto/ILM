use ilm_core::base::lanes::{hardware_lanes, on_lanes, shards};
use ilm_core::know::letters::letters;
use ilm_core::know::reader::{Entry, Side};
use ilm_core::know::sure::Knowledge;
use ilm_core::know::titles::names;
use ilm_core::base::lines::one_line;
use patterns::{because, source};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub struct SavedFacts;
source!(SavedFacts, "the table of what was read, written once as files of plain lines and looked up by name: the facts of each article in the order of the titles, and every name of an article as its letters with the place of the facts of that article in the first file; a request reads the lines of the names that it asks for and never the whole table, so an answer takes milliseconds after a reading of minutes");

const FACTS_FILE: &str = "facts.txt";
because!(FACTS_FILE, SavedFacts, "the file of the facts of the table, one fact on a line, the facts of one article together");

const NAMES_FILE: &str = "names.txt";
because!(NAMES_FILE, SavedFacts, "the file of the names of the articles of the table, in the order of their letters");

const TABLE_FIELD: char = '\t';
because!(TABLE_FIELD, SavedFacts, "the sign between the fields of a line of a file of the table");

const TABLE_LINE: char = '\n';
because!(TABLE_LINE, SavedFacts, "the sign after each line of a file of the table");

const HEX_BASE: u32 = 16;
because!(HEX_BASE, SavedFacts, "the base in which the bits of a chance are written: four bits for each sign, so that the text is short and the number comes back to the last bit");

const BY_WORDS: &str = "words";
because!(BY_WORDS, SavedFacts, "the mark of a fact that sure words of the article gave, where no sentence states it");

const BY_READING: &str = "read";
because!(BY_READING, SavedFacts, "the mark of a fact that a sentence of the article states");

const BY_LIST: &str = "list";
because!(BY_LIST, SavedFacts, "the mark of a fact that a listed object of the lessons gave, whose every word the article writes");

#[derive(Clone, Debug, Default)]
pub struct TableFact {
    pub title: String,
    pub relation: String,
    pub word: String,
    pub chance: f64,
    pub sentence: String,
    pub frame: String,
    pub hit: u32,
    pub seen: u32,
    pub right: u32,
    pub answers: u32,
    pub sure: bool,
    pub listed: bool,
}
because!(TableFact, SavedFacts, "one fact of the table: the article, the relation, the answer and its chance, the sentence that states it (for an answer by sure words: the first sentence with the sure word; for a listed object: the sentence of its first word), the frame that chose the answer with the hits and readings that the lessons gave it, how many held lesson facts were right at the level of its chance, and whether sure words or a listed object gave it");

#[derive(Clone, Debug)]
pub struct NamePlace {
    pub letters: String,
    pub title: String,
    pub start: u64,
    pub length: usize,
}
because!(NamePlace, SavedFacts, "one name of an article as its letters, with the title and the place and length of the facts of that article in the file of the facts");

const WRITE_BATCH: usize = 8192;
because!(WRITE_BATCH, SavedFacts, "the articles whose lines are made on the threads while the lines made before them are written: enough to keep every thread busy, few enough that the lines in memory are a small part of the file");

const WRITE_RUN: usize = 128;
because!(WRITE_RUN, SavedFacts, "the articles that one thread makes the lines of at a time, as one text: the file gets few large writes, and a batch has many runs, so that a run of long articles does not hold up the other threads");

type TableRun = (String, Vec<(usize, usize, Vec<String>)>);
because!(TableRun, SavedFacts, "the lines of a run of articles as one text, with the length of the lines of each article, the number of its facts and the letters of each of its names");

fn fact_line(model: &Knowledge, title: &str, relation: u32, entry: &Entry) -> String {
    let sure = entry.frame.side == Side::Sure;
    let listed = entry.frame.side == Side::Listed;
    let (hit, seen) = if sure {
        let mark = (entry.frame.words[0], entry.frame.words[1]);
        model.sure.get(&mark).and_then(|rows| rows.iter().find(|row| row.relation == relation)).map_or((0, 0), |row| (row.hit, row.seen))
    } else if listed {
        model.listed_of(relation, &entry.answer).map_or((0, 0), |object| (object.hit, object.seen))
    } else {
        model.frames.counts.get(&(entry.frame, relation)).copied().unwrap_or((0, 0))
    };
    let (right, answers) = model.measured(entry);
    let fields = [
        one_line(title),
        model.frames.relations.name(relation).to_string(),
        one_line(&entry.answer),
        format!("{:x}", entry.chance.to_bits()),
        one_line(&entry.sentence),
        model.frames.frame_text(&entry.frame),
        hit.to_string(),
        seen.to_string(),
        right.to_string(),
        answers.to_string(),
        (if sure {
            BY_WORDS
        } else if listed {
            BY_LIST
        } else {
            BY_READING
        })
        .to_string(),
    ];
    fields.join(&TABLE_FIELD.to_string())
}
because!(fact_line, SavedFacts, "one fact as a line of the file; the chance is written as the bits of its number, so that it is read back the same to the last bit");

fn table_block(model: &Knowledge, title: &str, text: &mut String) -> usize {
    let mut entries: Vec<&(u32, Entry)> = model.by_title[title].iter().collect();
    entries.sort_by(|one, other| model.frames.relations.name(one.0).cmp(model.frames.relations.name(other.0)));
    for (relation, entry) in &entries {
        text.push_str(&fact_line(model, title, *relation, entry));
        text.push(TABLE_LINE);
    }
    entries.len()
}
because!(table_block, SavedFacts, "the lines of the facts of one article put after a text, its relations in the order of their names; gives the number of its facts");

pub fn write_table(folder: &Path, model: &Knowledge, lanes: usize) -> std::io::Result<(usize, usize)> {
    std::fs::create_dir_all(folder)?;
    let mut titles: Vec<&String> = model.by_title.keys().collect();
    titles.sort_unstable();
    let mut file = std::fs::File::create(folder.join(FACTS_FILE))?;
    let mut places: Vec<NamePlace> = Vec::new();
    let mut start = 0u64;
    let mut count = 0usize;
    let (send, take) = std::sync::mpsc::sync_channel::<Vec<String>>(1);
    std::thread::scope(|scope| {
        let writer = scope.spawn(move || -> std::io::Result<()> {
            for texts in take {
                for text in texts {
                    file.write_all(text.as_bytes())?;
                }
            }
            file.flush()
        });
        for batch in titles.chunks(WRITE_BATCH) {
            let runs: Vec<&[&String]> = batch.chunks(WRITE_RUN).collect();
            let made: Vec<TableRun> = on_lanes(runs.len(), lanes, &|run| {
                let mut text = String::new();
                let rows = runs[run]
                    .iter()
                    .map(|title| {
                        let before = text.len();
                        let facts = table_block(model, title, &mut text);
                        (text.len() - before, facts, names(title).iter().map(|name| letters(name)).collect())
                    })
                    .collect();
                (text, rows)
            });
            let mut titles_of = batch.iter();
            let mut texts: Vec<String> = Vec::with_capacity(made.len());
            for (text, rows) in made {
                for ((length, facts, letters), title) in rows.into_iter().zip(titles_of.by_ref()) {
                    for letters in letters {
                        places.push(NamePlace { letters, title: title.to_string(), start, length });
                    }
                    start += length as u64;
                    count += facts;
                }
                texts.push(text);
            }
            if send.send(texts).is_err() {
                break;
            }
        }
        drop(send);
        writer.join().unwrap_or_else(|fault| std::panic::resume_unwind(fault))
    })?;
    places.sort_by(|one, other| (&one.letters, &one.title).cmp(&(&other.letters, &other.title)));
    places.dedup_by(|one, other| one.letters == other.letters && one.title == other.title);
    let mut out = BufWriter::new(std::fs::File::create(folder.join(NAMES_FILE))?);
    for place in &places {
        writeln!(out, "{}{TABLE_FIELD}{}{TABLE_FIELD}{}{TABLE_FIELD}{}", place.letters, one_line(&place.title), place.start, place.length)?;
    }
    out.flush()?;
    Ok((titles.len(), count))
}
because!(write_table, SavedFacts, "the table of a model as its two files: the facts of the articles in the order of their titles, each article with its relations in the order of their names, and the names of each article once, in the order of their letters and then of the titles; gives the number of articles and of facts. The lines of a batch of articles are made on all threads in runs while a thread of its own writes the batch before, in the order of the titles");

fn read_row(line: &str) -> Option<TableFact> {
    let fields: Vec<&str> = line.split(TABLE_FIELD).collect();
    let [title, relation, word, chance, sentence, frame, hit, seen, right, answers, way] = fields.as_slice() else { return None };
    let number = |text: &str| text.parse::<u32>().unwrap_or(0);
    Some(TableFact {
        title: title.to_string(),
        relation: relation.to_string(),
        word: word.to_string(),
        chance: f64::from_bits(u64::from_str_radix(chance, HEX_BASE).ok()?),
        sentence: sentence.to_string(),
        frame: frame.to_string(),
        hit: number(hit),
        seen: number(seen),
        right: number(right),
        answers: number(answers),
        sure: *way == BY_WORDS,
        listed: *way == BY_LIST,
    })
}
because!(read_row, SavedFacts, "one fact from its line, and nothing for a line that is not whole");

pub struct FactLookup {
    folder: PathBuf,
    places: Vec<NamePlace>,
}
because!(FactLookup, SavedFacts, "a table that was written, opened for looking up: the names of its articles in memory, their lines taken apart on all threads and kept in their order, the facts read from their file only where a name asks for them. under gives the articles of a name in the order of their titles; rows gives the fact of each of them for a relation");

impl FactLookup {
    pub fn open(folder: &Path) -> std::io::Result<FactLookup> {
        let text = std::fs::read_to_string(folder.join(NAMES_FILE))?;
        let lines: Vec<&str> = text.lines().collect();
        let lanes = hardware_lanes();
        let pieces = shards(lines.len(), lanes);
        let found: Vec<Vec<NamePlace>> = on_lanes(pieces.len(), lanes, &|piece| {
            lines[pieces[piece].clone()]
                .iter()
                .filter_map(|line| {
                    let fields: Vec<&str> = line.split(TABLE_FIELD).collect();
                    match fields.as_slice() {
                        [letters, title, start, length] => Some(NamePlace { letters: letters.to_string(), title: title.to_string(), start: start.parse().unwrap_or(0), length: length.parse().unwrap_or(0) }),
                        _ => None,
                    }
                })
                .collect()
        });
        Ok(FactLookup { folder: folder.to_path_buf(), places: found.into_iter().flatten().collect() })
    }

    pub fn under(&self, letters: &str) -> &[NamePlace] {
        let first = self.places.partition_point(|place| place.letters.as_str() < letters);
        let end = self.places.partition_point(|place| place.letters.as_str() <= letters);
        &self.places[first..end]
    }

    pub fn rows(&self, letters: &str, relation: &str) -> std::io::Result<Vec<TableFact>> {
        let places = self.under(letters);
        if places.is_empty() {
            return Ok(Vec::new());
        }
        let mut file = std::fs::File::open(self.folder.join(FACTS_FILE))?;
        let mut found: Vec<TableFact> = Vec::new();
        for place in places {
            file.seek(SeekFrom::Start(place.start))?;
            let mut bytes = vec![0u8; place.length];
            file.read_exact(&mut bytes)?;
            let text = String::from_utf8_lossy(&bytes);
            found.extend(text.lines().filter_map(read_row).filter(|row| row.relation == relation));
        }
        Ok(found)
    }
}
