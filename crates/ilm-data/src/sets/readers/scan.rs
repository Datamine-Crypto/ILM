use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::readers::common::{even_lessons, table_columns, text_value};
use patterns::{because, source};

pub struct ScanSet;
source!(ScanSet, "SCAN of Lake and Baroni: commands in a small made-up English and the actions that they mean, in four splits, from the copies of the hub with prompt templates, where each case of the set stands once for each template; the rows of the plain template are the cases, the train part first and then the test part, and the place of the first test case is where the cases to read start");

const COMMANDS: &str = "commands";
because!(COMMANDS, ScanSet, "the column of the command of a case");

const ACTIONS: &str = "actions";
because!(ACTIONS, ScanSet, "the column of the actions of a case, the words joined by a blank");

const TEMPLATE: &str = "template_name";
because!(TEMPLATE, ScanSet, "the column that names the prompt template of a row");

const PLAIN_TEMPLATE: &str = "plain";
because!(PLAIN_TEMPLATE, ScanSet, "the template whose rows hold each case of the set once and as it is");

const SCAN_TRAIN: &str = "train";
because!(SCAN_TRAIN, ScanSet, "the role of the train file of a split in the description");

const SCAN_TEST: &str = "test";
because!(SCAN_TEST, ScanSet, "the role of the test file of a split in the description");

fn scan_part(place: &str) -> Result<Vec<Case>, String> {
    let [commands, actions, templates] = table_columns(place, [COMMANDS, ACTIONS, TEMPLATE])?;
    let mut out: Vec<Case> = Vec::new();
    for ((command, action), template) in commands.iter().zip(&actions).zip(&templates) {
        if template.text() == Some(PLAIN_TEMPLATE) {
            out.push(Case { sentences: Vec::new(), question: text_value(command, place)?, answer: text_value(action, place)? });
        }
    }
    Ok(out)
}
because!(scan_part, ScanSet, "the cases of one file of a split, in its order: the rows of the plain template, the command as the question and the actions as the answer, with no told sentence");

pub fn scan_step_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let train = scan_part(&described.file(kind, SCAN_TRAIN)?)?;
    let test = scan_part(&described.file(kind, SCAN_TEST)?)?;
    let read_from = Some(train.len());
    Ok(KindCases { cases: even_lessons(train, test), read_from, ..KindCases::default() })
}
because!(scan_step_cases, ScanSet, "the cases of a kind of SCAN with its lessons spread evenly over the train part, and the place of the first test case");
