use ilm_core::base::signs::is_space;
use patterns::{because, source};

pub struct TitleNames;
source!(TitleNames, "the names under which an article of an encyclopedia can be asked for: an encyclopedia adds a part to a title to tell two things of one name apart, a part in brackets at the end or a part after a comma; so an article answers to its title, to its title without the part in brackets, and to its title without what follows the first comma, each in any case of letters");

const OPEN: char = '(';
because!(OPEN, TitleNames, "the sign that opens the added part of a title");

const CLOSE: char = ')';
because!(CLOSE, TitleNames, "the sign that closes the added part of a title");

const AFTER_NAME: &str = ", ";
because!(AFTER_NAME, TitleNames, "what stands between the name of a title and a larger place or thing that tells it apart");

pub fn without_brackets(title: &str) -> &str {
    let trimmed = title.trim_end_matches(is_space);
    let Some(inside) = trimmed.strip_suffix(CLOSE) else { return title };
    let from = inside.rfind(CLOSE).map_or(0, |at| at + CLOSE.len_utf8());
    let Some(open) = inside[from..].find(OPEN) else { return title };
    title[..from + open].trim_end_matches(is_space)
}
because!(without_brackets, TitleNames, "a title without a part in brackets at its end and without the space before that part: the part starts at the first opening sign after which no closing sign stands but the last one; a title whose end is no such part stays as it is");

pub fn head(title: &str) -> &str {
    let plain = without_brackets(title);
    let name = plain.split(AFTER_NAME).next().unwrap_or(plain);
    if name.is_empty() { title } else { name }
}
because!(head, TitleNames, "the name of a title as it is written: without the part in brackets and without what follows the first comma; the whole title when nothing is left");

pub fn names(title: &str) -> Vec<String> {
    let mut found = vec![title.to_lowercase()];
    let plain = without_brackets(title);
    let name = plain.split(AFTER_NAME).next().unwrap_or(plain);
    for other in [plain, name] {
        let low = other.to_lowercase();
        if !other.is_empty() && !found.contains(&low) {
            found.push(low);
        }
    }
    found
}
because!(names, TitleNames, "the names of an article in small letters, the title itself first, each name once");
