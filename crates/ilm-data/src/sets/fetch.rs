use ilm_core::base::digest::Sha256;
use patterns::{because, source};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

pub struct FetchSources;
source!(FetchSources, "the raw files of the sets are public files at fixed revisions, each named in the list of sources with its host, repository, revision, path, local path and hash; a run fetches each file that is missing or wrong with the program curl, which the systems carry, from the address that its host gives for that revision, and keeps it only when its hash is the one of the list, so every run reads the same bytes");

const FETCH_OPTION_MARK: char = '=';
because!(FETCH_OPTION_MARK, FetchSources, "the sign between the name of an option and its value");

const FETCH_LIST_MARK: char = ',';
because!(FETCH_LIST_MARK, FetchSources, "the sign between the names of a list of sets");

const FIELD_MARK: char = '\t';
because!(FIELD_MARK, FetchSources, "the sign between the fields of a line of the list of sources");

const NOTE_MARK: char = '#';
because!(NOTE_MARK, FetchSources, "the first sign of a line of the list of sources that names its fields and is no source");

const SOURCE_FIELDS: usize = 7;
because!(SOURCE_FIELDS, FetchSources, "the fields of a line of the list: the sets, the host, the repository, the revision, the path in the repository, the local path and the hash");

const HUB_HOST: &str = "huggingface";
because!(HUB_HOST, FetchSources, "the name of the hub of datasets in the list");

const GITHUB_HOST: &str = "github";
because!(GITHUB_HOST, FetchSources, "the name of the host of repositories of code in the list");

const STORAGE_HOST: &str = "googlestorage";
because!(STORAGE_HOST, FetchSources, "the name of the host of stored objects in the list");

const STORE_HOLDER: &str = "<folder of the encyclopedia>";
because!(STORE_HOLDER, FetchSources, "the place in a local path that the folder of the encyclopedia fills, which a run names because the text is large and kept apart from the sets");

const SET_HOLDER: &str = "<folder of the set>";
because!(SET_HOLDER, FetchSources, "the place in a local path that the folder of a set fills, which a run names");

const CURL_FLAGS: &[&str] = &["--location", "--fail", "--silent", "--show-error", "--proto", "=https", "--proto-redir", "=https", "--globoff"];
because!(CURL_FLAGS, FetchSources, "how curl fetches a file: it follows a move of the address, fails on an answer that is no file, is quiet but for faults, speaks only https also after a move, and reads the address as it stands");

const CURL_RETRIES: u32 = 3;
because!(CURL_RETRIES, FetchSources, "the tries that curl adds after a fault of the network, so that one lost answer does not stop a fetch of many files");

const CURL_WAIT_SECONDS: u32 = 30;
because!(CURL_WAIT_SECONDS, FetchSources, "the seconds that curl waits for a host to answer before it gives up, so that a dead host does not hold the run");

const PART_END: &str = ".part";
because!(PART_END, FetchSources, "the end of the name of a file while it is fetched: a run that stops leaves no file under the true name");

const READ_BLOCK: usize = 1_048_576;
because!(READ_BLOCK, FetchSources, "the bytes that one read of a file gives to the hash: large enough to read fast, small enough to need little room");

const USAGE: &str = "usage: ilm fetch <list of sources> [sets=<set>,<set>] [store=<folder of the encyclopedia>] [folder=<folder of a set>] [check=yes]";
because!(USAGE, FetchSources, "what the command takes: the list, the sets whose files it fetches (all when none is named), the folders that fill the places of a local path, and whether it only checks the files that are there");

struct SourceLine {
    sets: Vec<String>,
    host: String,
    repository: String,
    revision: String,
    path: String,
    local: String,
    hash: String,
}
because!(SourceLine, FetchSources, "one line of the list of sources");

const CHECK_ONLY: &str = "yes";
because!(CHECK_ONLY, FetchSources, "the value of the option check that asks only for a check of the files that are there");

const WEB_SCHEME: &str = "https:";
because!(WEB_SCHEME, FetchSources, "the scheme of every address, before the empty part that the join of the parts of an address turns into two slashes");

const ADDRESS_MARK: &str = "/";
because!(ADDRESS_MARK, FetchSources, "the sign between the parts of an address");

const HUB_SITE: &[&str] = &["huggingface.co", "datasets"];
because!(HUB_SITE, FetchSources, "the site of the hub and the part of its addresses for datasets, before the repository");

const HUB_REVISION: &str = "resolve";
because!(HUB_REVISION, FetchSources, "the part of an address of the hub before the revision of a file");

const GITHUB_SITE: &str = "raw.githubusercontent.com";
because!(GITHUB_SITE, FetchSources, "the site that gives the raw files of a repository of code at a revision");

const STORAGE_SITE: &str = "storage.googleapis.com";
because!(STORAGE_SITE, FetchSources, "the site that gives a file of a bucket of the cloud of Google by its name");

fn address(line: &SourceLine) -> Option<String> {
    let mut parts: Vec<&str> = vec![WEB_SCHEME, ""];
    match line.host.as_str() {
        HUB_HOST => parts.extend(HUB_SITE.iter().copied().chain([line.repository.as_str(), HUB_REVISION, line.revision.as_str(), line.path.as_str()])),
        GITHUB_HOST => parts.extend([GITHUB_SITE, line.repository.as_str(), line.revision.as_str(), line.path.as_str()]),
        STORAGE_HOST => parts.extend([STORAGE_SITE, line.repository.as_str(), line.path.as_str()]),
        _ => return None,
    }
    Some(parts.join(ADDRESS_MARK))
}
because!(address, FetchSources, "the address of a file at its revision, as its host gives it; none for a host that the list does not know");

fn file_hash(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0u8; READ_BLOCK];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hash.add(&buffer[..read]);
    }
    Ok(hash.hex())
}
because!(file_hash, FetchSources, "the hash of a file, read block by block");

fn read_lines(list: &Path) -> Result<Vec<SourceLine>, String> {
    let text = std::fs::read_to_string(list).map_err(|error| format!("{}: {error}", list.display()))?;
    let mut lines: Vec<SourceLine> = Vec::new();
    for (number, line) in text.lines().enumerate() {
        if line.is_empty() || line.starts_with(NOTE_MARK) {
            continue;
        }
        let fields: Vec<&str> = line.split(FIELD_MARK).collect();
        let [sets, host, repository, revision, path, local, hash] = fields.as_slice() else {
            return Err(format!("{} line {}: not {SOURCE_FIELDS} fields", list.display(), number + 1));
        };
        lines.push(SourceLine {
            sets: sets.split(FETCH_LIST_MARK).map(str::to_string).collect(),
            host: host.to_string(),
            repository: repository.to_string(),
            revision: revision.to_string(),
            path: path.to_string(),
            local: local.to_string(),
            hash: hash.to_string(),
        });
    }
    Ok(lines)
}
because!(read_lines, FetchSources, "the lines of the list of sources, or why the list cannot be read");

fn safe_local(local: &str) -> bool {
    Path::new(local).components().all(|part| match part {
        std::path::Component::Normal(_) | std::path::Component::CurDir => true,
        std::path::Component::Prefix(_) | std::path::Component::RootDir | std::path::Component::ParentDir => false,
    })
}
because!(safe_local, FetchSources, "whether the local path of a line of the list stays inside the folders that a run names: only names of folders and files, so no root, no drive and no step up to a parent folder, and a changed list cannot write a file elsewhere");

fn local_path(line: &SourceLine, named: &HashMap<&str, &str>) -> Option<PathBuf> {
    let mut local = line.local.clone();
    for (holder, option) in [(STORE_HOLDER, "store"), (SET_HOLDER, "folder")] {
        if local.contains(holder) {
            local = local.replace(holder, named.get(option)?);
        }
    }
    Some(PathBuf::from(local))
}
because!(local_path, FetchSources, "the local path of a line with its places filled from the options of the run; none when the run does not name a folder that the path needs");

fn fetched(url: &str, place: &Path) -> Result<(), String> {
    if let Some(folder) = place.parent() {
        std::fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    }
    let status = Command::new("curl")
        .args(CURL_FLAGS)
        .arg("--retry")
        .arg(CURL_RETRIES.to_string())
        .arg("--connect-timeout")
        .arg(CURL_WAIT_SECONDS.to_string())
        .arg("--output")
        .arg(place)
        .arg(url)
        .status()
        .map_err(|error| format!("curl: {error}"))?;
    if status.success() { Ok(()) } else { Err(format!("curl stopped with {status} for {url}")) }
}
because!(fetched, FetchSources, "a file fetched from its address into a place, with the folders that the place needs");

pub fn fetch_command(rest: &[String]) -> ExitCode {
    let named: HashMap<&str, &str> = rest.iter().filter_map(|word| word.split_once(FETCH_OPTION_MARK)).collect();
    let plain: Vec<&String> = rest.iter().filter(|word| !word.contains(FETCH_OPTION_MARK)).collect();
    let [list] = plain.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    let lines = match read_lines(Path::new(list.as_str())) {
        Ok(lines) => lines,
        Err(why) => {
            eprintln!("{why}");
            return ExitCode::FAILURE;
        }
    };
    let wanted: Vec<&str> = named.get("sets").map(|sets| sets.split(FETCH_LIST_MARK).collect()).unwrap_or_default();
    let only_check = named.get("check").is_some_and(|value| *value == CHECK_ONLY);
    let (mut have, mut got, mut skipped, mut faults) = (0usize, 0usize, 0usize, 0usize);
    for line in lines.iter().filter(|line| wanted.is_empty() || line.sets.iter().any(|set| wanted.contains(&set.as_str()))) {
        if !safe_local(&line.local) {
            println!("a local path that leaves the folders of the run: {}", line.local);
            faults += 1;
            continue;
        }
        let Some(place) = local_path(line, &named) else {
            skipped += 1;
            continue;
        };
        if place.exists() && file_hash(&place).is_ok_and(|hash| hash == line.hash) {
            have += 1;
            continue;
        }
        if only_check {
            println!("missing or different: {}", place.display());
            faults += 1;
            continue;
        }
        let Some(url) = address(line) else {
            println!("no address for the host {}: {}", line.host, place.display());
            faults += 1;
            continue;
        };
        let part = PathBuf::from(format!("{}{PART_END}", place.display()));
        let result = fetched(&url, &part).and_then(|()| match file_hash(&part) {
            Ok(hash) if hash == line.hash => std::fs::rename(&part, &place).map_err(|error| format!("{}: {error}", place.display())),
            Ok(hash) => Err(format!("{}: the hash is {hash}, the list says {}", place.display(), line.hash)),
            Err(error) => Err(format!("{}: {error}", part.display())),
        });
        match result {
            Ok(()) => {
                got += 1;
                println!("fetched {}", place.display());
            }
            Err(why) => {
                let _ = std::fs::remove_file(&part);
                faults += 1;
                println!("{why}");
            }
        }
    }
    println!("files there with the right hash: {have}; fetched: {got}; not asked (a folder of the path not named): {skipped}; faults: {faults}");
    if faults == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
because!(fetch_command, FetchSources, "the fetch of the raw files of the list: for each line of the asked sets, a file with the right hash is kept, another is fetched into a file of its own and kept under its true name only with the right hash; at the end the counts, and a failure when any file is missing or wrong");
