use ilm_core::tools::explain::chain::{Facts, chain_facts_once, chain_facts_within, knowledge_base, rules_key};
use ilm_core::tools::explain::learn::{LIBRARY_FOLDER, LINES_PRINT_MARK, built_on, lines_print, printed_library, read_library};
use ilm_core::base::lanes::{Worked, in_work_scope};
use ilm_core::cases::Case;
use ilm_core::tools::explain::story_disk::{Bytes, LEARNED_RULES, LEARNED_TOLD, build_sources, engine_print, learned_content, learned_record, learned_records};
use ilm_core::tools::explain::text::{fact_numbers, read_topic};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

const RULES: &str = "Cat is tall.
If A is tall then A is big.
If A is big then A is seen.";

const JOINED_RULES: &str = "If A is tall and A is in the B then A is seen in the B.";

fn story() -> Vec<String> {
    ["Ann is tall.", "Ann is in the hall.", "Cat is tall.", "Bob is tall.", "Ann is in the yard."].iter().map(|line| line.to_string()).collect()
}

#[test]
fn a_chain_read_back_from_its_layer_is_the_same_chain() {
    for history in [false, true] {
        let topic = read_topic("kept", RULES).expect("the rules read");
        let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, history));
        let facts = chain_facts_once(&story(), &base, (&topic.rules, topic.rules_key), (usize::MAX, usize::MAX, usize::MAX));
        let mut bytes = Vec::new();
        facts.layer_into(&mut bytes);
        let read = Facts::from_layer(&mut Bytes::of(&bytes), &base).expect("the layer reads back");
        assert_eq!(read.fingerprint(), facts.fingerprint());
        assert_eq!(read.told_facts(), facts.told_facts());
        assert_eq!(read.stopped_at_bound(), facts.stopped_at_bound());
        assert_eq!(read.exhausted(), facts.exhausted());
        assert!(read.holds(&fact_numbers("Ann is seen.")));
        assert!(read.holds(&fact_numbers("Cat is seen.")));
        assert!(read.is_told(&fact_numbers("Ann is in the yard.")));
    }
}

#[test]
fn a_kept_chain_adds_the_same_work_to_a_learning_as_when_it_was_made() {
    let topic = read_topic("kept", JOINED_RULES).expect("the rules read");
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, false));
    let chain = || chain_facts_within(&story(), &base, (&topic.rules, topic.rules_key), usize::MAX, usize::MAX);
    let made = in_work_scope(usize::MAX, chain);
    let kept = in_work_scope(usize::MAX, chain);
    assert!(made.work > 0);
    assert_eq!(made.work, kept.work);
    assert_eq!(made.value.fingerprint(), kept.value.fingerprint());
    assert!(!made.stopped && !kept.stopped);
}

#[test]
fn a_layer_does_not_read_back_over_other_known_facts() {
    let topic = read_topic("kept", RULES).expect("the rules read");
    let base = Arc::new(knowledge_base(&topic.facts, &topic.rules, false));
    let facts = chain_facts_once(&story(), &base, (&topic.rules, topic.rules_key), (usize::MAX, usize::MAX, usize::MAX));
    let mut bytes = Vec::new();
    facts.layer_into(&mut bytes);
    let with_history = Arc::new(knowledge_base(&topic.facts, &topic.rules, true));
    assert!(Facts::from_layer(&mut Bytes::of(&bytes), &with_history).is_none());
    let empty = Arc::new(knowledge_base(&[], &topic.rules, false));
    assert!(Facts::from_layer(&mut Bytes::of(&bytes), &empty).is_none());
    assert!(Facts::from_layer(&mut Bytes::of(&bytes[..bytes.len() - 1]), &base).is_none());
}

fn crate_paths(text: &str) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    for (at, _) in text.match_indices("crate::") {
        if text[..at].chars().last().is_some_and(|before| before.is_alphanumeric() || before == '_') {
            continue;
        }
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut letters = text[at + "crate::".len()..].chars().peekable();
        loop {
            match letters.next() {
                Some(letter) if letter.is_ascii_lowercase() || letter.is_ascii_digit() || letter == '_' => current.push(letter),
                Some(':') if letters.peek() == Some(&':') => {
                    letters.next();
                    parts.push(std::mem::take(&mut current));
                }
                _ => {
                    if !current.is_empty() {
                        parts.push(current);
                    }
                    break;
                }
            }
        }
        out.push(parts);
    }
    out
}

fn module_file(parts: &[String]) -> Option<String> {
    (1..=parts.len()).rev().find_map(|count| {
        let file = format!("src/{}.rs", parts[..count].join("/"));
        let folder = format!("src/{}/mod.rs", parts[..count].join("/"));
        [file, folder].into_iter().find(|path| std::path::Path::new(path).exists())
    })
}

#[test]
fn the_listed_engine_files_are_all_the_files_that_the_engine_reads() {
    let mut reached: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut waiting = vec!["src/tools/explain/chain.rs".to_string()];
    while let Some(file) = waiting.pop() {
        if !reached.insert(file.clone()) {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("an engine file reads");
        waiting.extend(crate_paths(&text).iter().filter_map(|parts| module_file(parts)));
    }
    let listed: std::collections::BTreeSet<String> = std::fs::read_to_string("tests/engine_files.txt").expect("the list of engine files reads").lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect();
    assert_eq!(reached, listed, "the engine reads another file than the list that its fingerprint hashes: write the list again");
}

#[test]
fn the_sources_of_a_build_are_read_as_whole_paths() {
    let listed = r"C:\a\target\fast\ilm.exe: C:\a\crates\ilm-core\src\lib.rs C:\a\my\ folder\b.rs
";
    let sources = build_sources(listed);
    assert_eq!(sources, vec![PathBuf::from(r"C:\a\crates\ilm-core\src\lib.rs"), PathBuf::from(r"C:\a\my folder\b.rs")]);
    let slashed = build_sources("/a/target/fast/ilm: /a/crates/ilm-core/src/lib.rs
");
    assert!(slashed[0].ends_with("crates/ilm-core/src/lib.rs"));
    assert!(build_sources("no list").is_empty());
}

#[test]
fn the_engine_print_is_refused_for_files_newer_than_the_build() {
    assert!(engine_print(Path::new("."), SystemTime::UNIX_EPOCH).is_err());
    let print = engine_print(Path::new("."), SystemTime::now()).expect("the engine files are older than now");
    assert!(!print.is_empty() && print.chars().all(|sign| sign.is_ascii_hexdigit()));
    assert_eq!(engine_print(Path::new("."), SystemTime::now()), Ok(print));
}

#[test]
fn learned_lines_read_back_from_their_records() {
    let lines = vec!["If A went to the B then the place of A is B.".to_string(), "Write Then A as A.".to_string()];
    let worked = Worked { value: lines.clone(), work: lines.len(), stopped: true, taken: vec!["kin_one".to_string()] };
    let none = Worked { value: Vec::new(), work: usize::default(), stopped: false, taken: Vec::new() };
    let mut bytes = learned_record(u128::MAX, &worked);
    bytes.extend(learned_record(u128::MIN, &none));
    assert_eq!(learned_records(&bytes), vec![(u128::MAX, worked.clone()), (u128::MIN, none)]);
    assert_eq!(learned_records(&bytes[..bytes.len() - 1]), vec![(u128::MAX, worked)]);
}

#[test]
fn the_key_of_what_is_learned_follows_the_text_the_facts_and_the_lessons() {
    let topic = read_topic("kept", RULES).expect("the rules read");
    let other = read_topic("kept", "Cat is tall.
If A is tall then A is big.").expect("the rules read");
    let base = knowledge_base(&topic.facts, &topic.rules, false);
    let case = Case { sentences: story(), question: "Where is Ann?".to_string(), answer: "yard".to_string() };
    let changed = Case { answer: "hall".to_string(), ..case.clone() };
    let key = learned_content(LEARNED_RULES, &topic, &base, &[&case]);
    assert_eq!(key, learned_content(LEARNED_RULES, &topic, &base, &[&case]));
    assert_ne!(key, learned_content(LEARNED_TOLD, &topic, &base, &[&case]));
    assert_ne!(key, learned_content(LEARNED_RULES, &other, &base, &[&case]));
    assert_ne!(key, learned_content(LEARNED_RULES, &topic, &base, &[&changed]));
    assert_ne!(key, learned_content(LEARNED_RULES, &topic, &knowledge_base(&[], &topic.rules, false), &[&case]));
    let mut marked = topic.clone();
    for rule in &mut marked.rules {
        rule.learned = true;
    }
    assert_ne!(key, learned_content(LEARNED_RULES, &marked, &base, &[&case]), "rules that a learner made fill their places with different things, so their text keeps its own key");
}

#[test]
fn every_topic_of_the_library_keeps_the_key_of_its_rules() {
    let texts = read_library(&Path::new("../..").join(LIBRARY_FOLDER)).expect("the library reads");
    let topics = built_on(&texts).expect("every topic it builds on is there");
    assert!(!topics.is_empty());
    for topic in texts.iter().chain(&topics) {
        assert_eq!(rules_key(&topic.rules), topic.rules_key, "the key of the rules of {} is stale: the chains kept by it would be read for other rules", topic.name);
    }
}

#[test]
fn the_fingerprint_of_a_library_changes_with_a_text_and_stays_when_nothing_changed() {
    let folder = std::env::temp_dir().join(format!("ilm_library_print_{}", std::process::id()));
    std::fs::create_dir_all(&folder).expect("the folder is made");
    std::fs::write(folder.join("colours.txt"), "Red is a colour.\nIf A is a colour then A is a color.\n").expect("the text is written");
    let (_, first) = printed_library(&folder).expect("the library reads");
    let (_, again) = printed_library(&folder).expect("the library reads again");
    std::fs::write(folder.join("colours.txt"), "Red is a colour.\nGreen is a colour.\nIf A is a colour then A is a color.\n").expect("the text is written again");
    let (_, changed) = printed_library(&folder).expect("the changed library reads");
    std::fs::remove_dir_all(&folder).ok();
    assert_eq!(first, again);
    assert_ne!(first, changed);
}

#[test]
fn a_learned_text_is_refused_when_a_person_edited_it_or_a_hand_text_has_its_name() {
    let folder = std::env::temp_dir().join(format!("ilm_learned_text_{}", std::process::id()));
    std::fs::create_dir_all(folder.join("learned")).expect("the learned folder is made");
    std::fs::create_dir_all(folder.join("knowledge")).expect("the knowledge folder is made");
    let lines = "Oslo is the capital of Norway.
Note: from two sentences.
Rome is the capital of Italy.
";
    let text = format!("Note: written by a step, {LINES_PRINT_MARK}{}.
{lines}", lines_print(lines));
    let learned = folder.join("learned").join("capitals.txt");
    std::fs::write(&learned, &text).expect("the learned text is written");
    let read = printed_library(&folder);
    std::fs::write(&learned, text.replace("Note: from two sentences.", "Note: from three sentences.")).expect("a note is changed");
    let note_changed = printed_library(&folder);
    std::fs::write(&learned, text.replace("Rome", "Milan")).expect("a fact is changed by hand");
    let edited = printed_library(&folder);
    std::fs::write(&learned, &text).expect("the learned text is written again");
    std::fs::write(folder.join("knowledge").join("capitals.txt"), "Oslo is the capital of Norway.
").expect("a hand text of the same name is written");
    let twice = printed_library(&folder);
    std::fs::remove_dir_all(&folder).ok();
    assert!(read.is_ok(), "{:?}", read.err());
    assert!(note_changed.is_ok(), "a note is no line that the print covers: {:?}", note_changed.err());
    assert!(edited.as_ref().is_err_and(|fault| fault.contains("learned text")), "{:?}", edited.map(|(_, print)| print));
    assert!(twice.as_ref().is_err_and(|fault| fault.contains("two texts")), "{:?}", twice.map(|(_, print)| print));
}
