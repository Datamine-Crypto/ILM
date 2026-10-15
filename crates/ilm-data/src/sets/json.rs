use crate::sets::parquet::records::Value;
use patterns::{because, source};

pub struct JsonText;
source!(JsonText, "JSON as the standard of the IETF defines it and as the json module of Python reads it: objects, arrays, texts with their escapes (a pair of escaped surrogates is one sign), whole numbers and numbers with a point or an exponent, true, false and null; in an object a key that comes again keeps its first place and takes the last value. A text is written as that module writes it with every sign kept as it is: a quote, a backslash and the signs of control escaped, the five of them that have a letter by that letter, the others by four hexadecimal digits in small letters");

const OPEN_OBJECT: u8 = b'{';
because!(OPEN_OBJECT, JsonText, "the sign that opens an object: named values, each a key in quotes, a colon and a value");

const CLOSE_OBJECT: u8 = b'}';
because!(CLOSE_OBJECT, JsonText, "the sign after the last member of an object");

const OPEN_ARRAY: u8 = b'[';
because!(OPEN_ARRAY, JsonText, "the sign that opens an array: values in their order");

const CLOSE_ARRAY: u8 = b']';
because!(CLOSE_ARRAY, JsonText, "the sign after the last value of an array");

const JSON_QUOTE: u8 = b'"';
because!(JSON_QUOTE, JsonText, "the sign round a text");

const BACKSLASH: u8 = b'\\';
because!(BACKSLASH, JsonText, "the sign that starts an escape in a text");

const COLON: u8 = b':';
because!(COLON, JsonText, "the sign between a key and its value");

const COMMA: u8 = b',';
because!(COMMA, JsonText, "the sign between two members of an object or two values of an array");

const MINUS: u8 = b'-';
because!(MINUS, JsonText, "the sign of a number below nought");

const TRUE_WORD: &str = "true";
because!(TRUE_WORD, JsonText, "the word that JSON writes for a true value, read as the value true");

const FALSE_WORD: &str = "false";
because!(FALSE_WORD, JsonText, "the word that JSON writes for a false value, read as the value false");

const NULL_WORD: &str = "null";
because!(NULL_WORD, JsonText, "the word that JSON writes for a missing value, read as nothing");

const SPACES: &[u8] = b" \t\n\r";
because!(SPACES, JsonText, "the signs of white space between the tokens of JSON");

const ESCAPES: &[(u8, char)] = &[(b'"', '"'), (b'\\', '\\'), (b'/', '/'), (b'b', '\u{8}'), (b'f', '\u{c}'), (b'n', '\n'), (b'r', '\r'), (b't', '\t')];
because!(ESCAPES, JsonText, "the sign after a backslash and the sign that the escape stands for");

const UNICODE_ESCAPE: u8 = b'u';
because!(UNICODE_ESCAPE, JsonText, "the sign after a backslash that starts an escape of four hexadecimal digits");

const NUMBER_SIGNS: &[u8] = b".eE+-";
because!(NUMBER_SIGNS, JsonText, "the signs of a number beside its digits that make it a number with a point or an exponent");

const ESCAPE_START: usize = 2;
because!(ESCAPE_START, JsonText, "the backslash and the letter that open the escape of the second surrogate of a pair");

const HEX_DIGITS: usize = 4;
because!(HEX_DIGITS, JsonText, "the hexadecimal digits of an escape of one unit of the sixteen-bit form of Unicode");

const JSON_HEX_BASE: u32 = 16;
because!(JSON_HEX_BASE, JsonText, "the base of the four digits of an escape of one unit of a text");

const JSON_HIGH_FIRST: u32 = 0xD800;
because!(JSON_HIGH_FIRST, JsonText, "the first high surrogate, which with a low surrogate after it makes one sign");

const JSON_LOW_FIRST: u32 = 0xDC00;
because!(JSON_LOW_FIRST, JsonText, "the first low surrogate: a high surrogate is followed by one from here up to make one sign");

const LOW_END: u32 = 0xE000;
because!(LOW_END, JsonText, "the first unit after the low surrogates");

const PAIR_BASE: u32 = 0x10000;
because!(PAIR_BASE, JsonText, "the first sign that a pair of surrogates stands for");

const JSON_SURROGATE_BITS: u32 = 10;
because!(JSON_SURROGATE_BITS, JsonText, "the bits that each surrogate of a pair carries");

const JSON_DEEPEST: usize = 512;
because!(JSON_DEEPEST, JsonText, "the depth of nested arrays and objects at most, so that a bad file cannot run the reader without end");

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
    depth: usize,
}
because!(Cursor, JsonText, "a reader of JSON over some bytes, with the place it has come to and the depth of nesting");

impl Cursor<'_> {
    fn space(&mut self) {
        while self.at < self.bytes.len() && SPACES.contains(&self.bytes[self.at]) {
            self.at += 1;
        }
    }

    fn fault<T>(&self, what: &str) -> Result<T, String> {
        Err(format!("JSON: {what} at byte {}", self.at))
    }

    fn word(&mut self, word: &str, value: Value) -> Result<Value, String> {
        if self.bytes[self.at..].starts_with(word.as_bytes()) {
            self.at += word.len();
            return Ok(value);
        }
        self.fault("an unknown word")
    }

    fn hex(&mut self) -> Result<u32, String> {
        let digits = self.bytes.get(self.at..self.at + HEX_DIGITS).ok_or("JSON: a text ends in an escape")?;
        self.at += HEX_DIGITS;
        let text = std::str::from_utf8(digits).map_err(|_| "JSON: an escape that is no number".to_string())?;
        u32::from_str_radix(text, JSON_HEX_BASE).map_err(|_| "JSON: an escape that is no number".to_string())
    }

    fn text(&mut self) -> Result<String, String> {
        self.at += 1;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let Some(&byte) = self.bytes.get(self.at) else { return self.fault("a text with no end") };
            self.at += 1;
            if byte == JSON_QUOTE {
                return String::from_utf8(out).map_err(|_| "JSON: a text that is no Unicode".to_string());
            }
            if byte != BACKSLASH {
                out.push(byte);
                continue;
            }
            let Some(&kind) = self.bytes.get(self.at) else { return self.fault("a text ends in an escape") };
            self.at += 1;
            if let Some((_, sign)) = ESCAPES.iter().find(|(held, _)| *held == kind) {
                let mut buffer = [0u8; HEX_DIGITS];
                out.extend_from_slice(sign.encode_utf8(&mut buffer).as_bytes());
                continue;
            }
            if kind != UNICODE_ESCAPE {
                return self.fault("an unknown escape");
            }
            let mut unit = self.hex()?;
            if (JSON_HIGH_FIRST..JSON_LOW_FIRST).contains(&unit) && self.bytes.get(self.at) == Some(&BACKSLASH) && self.bytes.get(self.at + 1) == Some(&UNICODE_ESCAPE) {
                let keep = self.at;
                self.at += ESCAPE_START;
                let low = self.hex()?;
                if (JSON_LOW_FIRST..LOW_END).contains(&low) {
                    unit = PAIR_BASE + ((unit - JSON_HIGH_FIRST) << JSON_SURROGATE_BITS) + (low - JSON_LOW_FIRST);
                } else {
                    self.at = keep;
                }
            }
            let sign = char::from_u32(unit).ok_or("JSON: a lone surrogate, which no text of the program can hold")?;
            let mut buffer = [0u8; HEX_DIGITS];
            out.extend_from_slice(sign.encode_utf8(&mut buffer).as_bytes());
        }
    }

    fn number(&mut self) -> Result<Value, String> {
        let start = self.at;
        if self.bytes.get(self.at) == Some(&MINUS) {
            self.at += 1;
        }
        let mut whole = true;
        while let Some(&byte) = self.bytes.get(self.at) {
            if byte.is_ascii_digit() {
                self.at += 1;
            } else if NUMBER_SIGNS.contains(&byte) {
                whole = false;
                self.at += 1;
            } else {
                break;
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.at]).map_err(|_| "JSON: a number that is no text".to_string())?;
        if whole {
            return text.parse::<i64>().map(Value::Int).map_err(|_| format!("JSON: the whole number {text} is too large"));
        }
        text.parse::<f64>().map(Value::Double).map_err(|_| format!("JSON: {text} is no number"))
    }

    fn value(&mut self) -> Result<Value, String> {
        self.space();
        let Some(&byte) = self.bytes.get(self.at) else { return self.fault("the text ends where a value should be") };
        match byte {
            OPEN_OBJECT => self.object(),
            OPEN_ARRAY => self.array(),
            JSON_QUOTE => self.text().map(Value::Text),
            _ if TRUE_WORD.as_bytes().first() == Some(&byte) => self.word(TRUE_WORD, Value::Bool(true)),
            _ if FALSE_WORD.as_bytes().first() == Some(&byte) => self.word(FALSE_WORD, Value::Bool(false)),
            _ if NULL_WORD.as_bytes().first() == Some(&byte) => self.word(NULL_WORD, Value::Null),
            _ if byte == MINUS || byte.is_ascii_digit() => self.number(),
            _ => self.fault("no value"),
        }
    }

    fn deeper(&mut self) -> Result<(), String> {
        self.depth += 1;
        if self.depth > JSON_DEEPEST {
            return self.fault("too deep a nesting");
        }
        Ok(())
    }

    fn array(&mut self) -> Result<Value, String> {
        self.deeper()?;
        self.at += 1;
        let mut items: Vec<Value> = Vec::new();
        self.space();
        if self.bytes.get(self.at) == Some(&CLOSE_ARRAY) {
            self.at += 1;
            self.depth -= 1;
            return Ok(Value::List(items));
        }
        loop {
            items.push(self.value()?);
            self.space();
            match self.bytes.get(self.at) {
                Some(&COMMA) => self.at += 1,
                Some(&CLOSE_ARRAY) => {
                    self.at += 1;
                    self.depth -= 1;
                    return Ok(Value::List(items));
                }
                _ => return self.fault("an array with no end"),
            }
        }
    }

    fn object(&mut self) -> Result<Value, String> {
        self.deeper()?;
        self.at += 1;
        let mut fields: Vec<(String, Value)> = Vec::new();
        self.space();
        if self.bytes.get(self.at) == Some(&CLOSE_OBJECT) {
            self.at += 1;
            self.depth -= 1;
            return Ok(Value::Struct(fields));
        }
        loop {
            self.space();
            if self.bytes.get(self.at) != Some(&JSON_QUOTE) {
                return self.fault("a key that is no text");
            }
            let key = self.text()?;
            self.space();
            if self.bytes.get(self.at) != Some(&COLON) {
                return self.fault("a key with no colon");
            }
            self.at += 1;
            let value = self.value()?;
            match fields.iter_mut().find(|(held, _)| *held == key) {
                Some((_, held)) => *held = value,
                None => fields.push((key, value)),
            }
            self.space();
            match self.bytes.get(self.at) {
                Some(&COMMA) => self.at += 1,
                Some(&CLOSE_OBJECT) => {
                    self.at += 1;
                    self.depth -= 1;
                    return Ok(Value::Struct(fields));
                }
                _ => return self.fault("an object with no end"),
            }
        }
    }
}

pub fn parse_json(text: &str) -> Result<Value, String> {
    let mut cursor = Cursor { bytes: text.as_bytes(), at: 0, depth: 0 };
    let value = cursor.value()?;
    cursor.space();
    if cursor.at != cursor.bytes.len() {
        return cursor.fault("more after the value");
    }
    Ok(value)
}
because!(parse_json, JsonText, "the one value of a JSON text, with nothing but white space after it");

const SOLIDUS: u8 = b'/';
because!(SOLIDUS, JsonText, "the sign that a reader takes escaped or plain and that a writer leaves plain");

const FIRST_PLAIN: u8 = b' ';
because!(FIRST_PLAIN, JsonText, "the first sign that is no sign of control, which a text holds as it is");

pub fn json_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    out.push(char::from(JSON_QUOTE));
    let mut kept = 0usize;
    for (at, byte) in text.bytes().enumerate() {
        if byte >= FIRST_PLAIN && byte != JSON_QUOTE && byte != BACKSLASH {
            continue;
        }
        out.push_str(&text[kept..at]);
        kept = at + 1;
        out.push(char::from(BACKSLASH));
        match ESCAPES.iter().find(|(letter, made)| u32::from(*made) == u32::from(byte) && *letter != SOLIDUS) {
            Some((letter, _)) => out.push(char::from(*letter)),
            None => {
                out.push(char::from(UNICODE_ESCAPE));
                let code = u32::from(byte);
                out.extend((0..HEX_DIGITS).rev().filter_map(|place| char::from_digit(code / JSON_HEX_BASE.pow(place as u32) % JSON_HEX_BASE, JSON_HEX_BASE)));
            }
        }
    }
    out.push_str(&text[kept..]);
    out.push(char::from(JSON_QUOTE));
    out
}
because!(json_text, JsonText, "a text as a JSON text in quotes, the runs of plain signs copied whole, each other sign by its escape");
