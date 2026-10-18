use crate::sets::parquet::column::{ColumnData, Leaf};
use crate::sets::parquet::meta::{Element, OPTIONAL, REPEATED, is_list, is_map, is_null_type, is_text};
use patterns::{because, source};

pub struct RecordAssembly;
source!(RecordAssembly, "the assembly of nested records from the levels of their leaf columns, as in the Dremel paper of Melnik and others: the level of repetition of a value says which list gets a new element, the level of definition says how deep the value is there; a group annotated as a list holds one repeated group whose child is the element, or by the older rules of the format the repeated field is the element itself; a list is read as a list, a group as named values, a missing value as nothing, in the way that pyarrow gives a column as Python values");

const ARRAY_NAME: &str = "array";
because!(ARRAY_NAME, RecordAssembly, "the name of a repeated group of one field that older writers used for the elements of a list, which is then the element itself");

const TUPLE_END: &str = "_tuple";
because!(TUPLE_END, RecordAssembly, "the end of the name of a repeated group of one field that older writers gave the elements of a list after the name of the list, which is then the element itself");

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Double(f64),
    Text(String),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Struct(Vec<(String, Value)>),
}
because!(Value, RecordAssembly, "one value of a record: nothing, a truth value, a whole number, a number with a point, a text, bytes, a list, or named values in the order of the schema. get gives a named value of a group, text the text of a text value");

impl Value {
    pub fn get(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Struct(fields) => fields.iter().find(|(held, _)| held == name).map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            Value::Text(text) => Some(text),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub element: Element,
    pub children: Vec<Node>,
}
because!(Node, RecordAssembly, "one element of the schema with its children, made from the flat list of the metadata");

pub fn schema_tree(elements: &[Element]) -> Result<Node, String> {
    fn take(elements: &[Element], at: &mut usize) -> Result<Node, String> {
        let element = elements.get(*at).cloned().ok_or("the schema ends inside a group")?;
        *at += 1;
        let mut children: Vec<Node> = Vec::with_capacity(element.children);
        for _ in 0..element.children {
            children.push(take(elements, at)?);
        }
        Ok(Node { element, children })
    }
    let mut at = 0usize;
    take(elements, &mut at)
}
because!(schema_tree, RecordAssembly, "the schema as a tree: the flat list is a walk from the root in which a group is followed by its children");

#[derive(Clone, Debug)]
pub struct PathStep {
    pub element: Element,
    pub definition: u32,
    pub repetition: u32,
    pub transparent: bool,
}
because!(PathStep, RecordAssembly, "one node on the steps from a top field to a leaf: the element, the levels of definition and of repetition that reaching it takes, and whether the node gives the value of its child as its own, which a list group and the repeated group of a list element do");

pub fn leaf_paths(top: &Node) -> Vec<Vec<PathStep>> {
    fn walk(node: &Node, parent: Option<&Node>, definition: u32, repetition: u32, steps: &mut Vec<PathStep>, out: &mut Vec<Vec<PathStep>>) {
        let definition = definition + u32::from(node.element.repetition != crate::sets::parquet::meta::REQUIRED);
        let repetition = repetition + u32::from(node.element.repetition == REPEATED);
        let under_list = parent.is_some_and(|parent| is_list(&parent.element) || is_map(&parent.element));
        let element_itself = node.element.physical.is_some()
            || node.children.len() > 1
            || node.element.name == ARRAY_NAME
            || parent.is_some_and(|parent| node.element.name == format!("{}{TUPLE_END}", parent.element.name));
        let map_group = node.element.logical == Some(crate::sets::parquet::meta::LOGICAL_MAP) || node.element.converted == Some(crate::sets::parquet::meta::CONVERTED_MAP);
        let transparent = node.element.physical.is_none() && (is_list(&node.element) || map_group || (under_list && node.element.repetition == REPEATED && !element_itself));
        steps.push(PathStep { element: node.element.clone(), definition, repetition, transparent });
        if node.children.is_empty() {
            out.push(steps.clone());
        }
        for child in &node.children {
            walk(child, Some(node), definition, repetition, steps, out);
        }
        steps.pop();
    }
    let mut out: Vec<Vec<PathStep>> = Vec::new();
    walk(top, None, 0, 0, &mut Vec::new(), &mut out);
    out
}
because!(leaf_paths, RecordAssembly, "the steps from a top field to each of its leaves, in the order of the leaf columns, with the levels at each node; a repeated group under a list is the element itself when it is a leaf, holds more than one field, or has a name that older writers gave the element");

#[derive(Clone, Copy, Debug)]
struct Triple {
    repetition: u32,
    definition: u32,
    value: Option<usize>,
}
because!(Triple, RecordAssembly, "one entry of a leaf column: its two levels and the place of its value among the values that are there, or none for a value that is missing");

fn leaf_value(leaf: &mut Leaf, element: &Element) -> Result<Value, String> {
    Ok(match leaf {
        Leaf::Bool(truth) => Value::Bool(*truth),
        Leaf::Int(number) => Value::Int(*number),
        Leaf::Double(number) => Value::Double(*number),
        Leaf::Bytes(bytes) if is_text(element) => Value::Text(String::from_utf8(std::mem::take(bytes)).map_err(|_| format!("a value of the text column {} is no text", element.name))?),
        Leaf::Bytes(bytes) => Value::Bytes(std::mem::take(bytes)),
    })
}
because!(leaf_value, RecordAssembly, "the value of one leaf: bytes that the schema calls a text become a text, other bytes stay bytes; the bytes are moved out of the leaf, because each value of a column belongs to one record");

struct Builder<'a> {
    steps: &'a [PathStep],
    values: &'a mut [Leaf],
}
because!(Builder, RecordAssembly, "the steps of one leaf and its values, from which the records of that leaf are built one by one");

impl Builder<'_> {
    fn build(&mut self, at: usize, triples: &[Triple]) -> Result<Value, String> {
        let step = &self.steps[at];
        let first = triples.first().ok_or("a record of a column has no entry")?;
        if step.element.repetition == REPEATED {
            if first.definition < step.definition {
                return Ok(Value::List(Vec::new()));
            }
            let mut items: Vec<Value> = Vec::new();
            let mut start = 0usize;
            for place in 1..=triples.len() {
                if place == triples.len() || triples[place].repetition <= step.repetition {
                    items.push(self.contents(at, &triples[start..place])?);
                    start = place;
                }
            }
            return Ok(Value::List(items));
        }
        if step.element.repetition == OPTIONAL && first.definition < step.definition {
            return Ok(Value::Null);
        }
        self.contents(at, triples)
    }

    fn contents(&mut self, at: usize, triples: &[Triple]) -> Result<Value, String> {
        let step = &self.steps[at];
        if at + 1 == self.steps.len() {
            if is_null_type(&step.element) {
                return Ok(Value::Null);
            }
            let place = triples.first().and_then(|triple| triple.value).ok_or("a defined leaf has no value")?;
            let element = &self.steps[at].element;
            return leaf_value(&mut self.values[place], element);
        }
        let child = self.build(at + 1, triples)?;
        if step.transparent {
            return Ok(child);
        }
        Ok(Value::Struct(vec![(self.steps[at + 1].element.name.clone(), child)]))
    }
}

fn merge(one: Value, other: Value) -> Result<Value, String> {
    match (one, other) {
        (Value::Struct(mut fields), Value::Struct(more)) => {
            for (name, value) in more {
                match fields.last_mut() {
                    Some((last, held)) if *last == name => {
                        let joined = merge(std::mem::replace(held, Value::Null), value)?;
                        *held = joined;
                    }
                    _ => fields.push((name, value)),
                }
            }
            Ok(Value::Struct(fields))
        }
        (Value::List(items), Value::List(more)) => {
            if items.len() != more.len() {
                return Err("two leaves of one list have lists of other lengths".to_string());
            }
            let joined: Result<Vec<Value>, String> = items.into_iter().zip(more).map(|(item, other)| merge(item, other)).collect();
            Ok(Value::List(joined?))
        }
        (Value::Null, Value::Null) => Ok(Value::Null),
        _ => Err("two leaves of one field do not have the same shape".to_string()),
    }
}
because!(merge, RecordAssembly, "the values of one field from two of its leaves as one value: named values joined in the order of the schema, lists element by element, nothing where both are nothing");

pub fn assemble(steps: &[PathStep], data: &mut ColumnData, rows: usize) -> Result<Vec<Value>, String> {
    let most = steps.last().map_or(0, |step| step.definition);
    let mut triples: Vec<Triple> = Vec::with_capacity(data.definitions.len());
    let mut next = 0usize;
    for (repetition, definition) in data.repetitions.iter().zip(&data.definitions) {
        let value = (*definition == most).then(|| {
            next += 1;
            next - 1
        });
        triples.push(Triple { repetition: *repetition, definition: *definition, value });
    }
    let mut builder = Builder { steps, values: &mut data.values };
    let mut out: Vec<Value> = Vec::with_capacity(rows);
    let mut start = 0usize;
    for place in 1..=triples.len() {
        if place == triples.len() || triples[place].repetition == 0 {
            out.push(builder.build(0, &triples[start..place])?);
            start = place;
        }
    }
    Ok(out)
}
because!(assemble, RecordAssembly, "the records of one leaf column: its entries cut where the level of repetition is nought, each record built down the steps of the leaf");

pub fn joined(columns: Vec<Vec<Value>>) -> Result<Vec<Value>, String> {
    let mut columns = columns.into_iter();
    let Some(mut out) = columns.next() else { return Ok(Vec::new()) };
    for column in columns {
        if column.len() != out.len() {
            return Err("two leaves of one field have other numbers of records".to_string());
        }
        out = out.into_iter().zip(column).map(|(one, other)| merge(one, other)).collect::<Result<Vec<Value>, String>>()?;
    }
    Ok(out)
}
because!(joined, RecordAssembly, "the records of a field from the records of each of its leaves");
