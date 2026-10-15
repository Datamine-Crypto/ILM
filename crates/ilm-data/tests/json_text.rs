use ilm_data::sets::json::{json_text, parse_json};
use ilm_data::sets::parquet::records::Value;

#[test]
fn a_text_is_written_as_the_json_module_of_python_writes_it() {
    assert_eq!(json_text("plain text"), "\"plain text\"");
    assert_eq!(json_text("a\"b\\c"), "\"a\\\"b\\\\c\"");
    assert_eq!(json_text("line\nnext\ttab\rret"), "\"line\\nnext\\ttab\\rret\"");
    assert_eq!(json_text("\u{8}\u{c}\u{1}\u{1f}\u{7f}"), "\"\\b\\f\\u0001\\u001f\u{7f}\"");
    assert_eq!(json_text("caf\u{e9} / \u{2028} \u{1f600}"), "\"caf\u{e9} / \u{2028} \u{1f600}\"");
}

#[test]
fn a_written_text_reads_back_as_the_same_text() {
    for text in ["", "plain", "a\"b\\c", "\u{0}\u{1}\u{8}\u{9}\u{a}\u{c}\u{d}\u{1f} end", "caf\u{e9} \u{1f600}"] {
        assert_eq!(parse_json(&json_text(text)), Ok(Value::Text(text.to_string())));
    }
}
