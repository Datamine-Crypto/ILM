use patterns::{because, source};

pub struct PlainLines;
source!(PlainLines, "files of plain lines: one record on a line and its fields parted by a tab, so that a person and any program can read them");

const LINE_SIGNS: &[char] = &['\t', '\r', '\n'];
because!(LINE_SIGNS, PlainLines, "the signs that part the fields and the lines of a file of plain lines; a text that goes into such a file holds none of them");

const ONE_SPACE: &str = " ";
because!(ONE_SPACE, PlainLines, "what stands in a text in place of a sign that parts fields or lines");

pub fn one_line(text: &str) -> String {
    text.replace(LINE_SIGNS, ONE_SPACE)
}
because!(one_line, PlainLines, "a text on one line with no sign that parts fields or lines, so that it fits one field of a file of plain lines");
