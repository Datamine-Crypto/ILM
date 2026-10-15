use ilm_core::cases::Case;
use crate::sets::export::{Described, KindCases};
use crate::sets::json::parse_json;
use crate::sets::parquet::records::Value;
use crate::sets::readers::common::{LESSON_FILES, rows_where, text_value, told_sentences};
use crate::sets::textmatch::TextPattern;
use patterns::{because, source};
use std::collections::BTreeMap;

pub struct ProofTheories;
source!(ProofTheories, "ProofWriter of Tafjord, Dalvi and Clark: theories of facts and rules with statements to judge, each with the number of rule uses of its proof; the open form in the copy of tasksource (rows of one configuration with the answer True, False or Unknown; the part in the words of people, ParaRules, is another configuration of the same copy), the closed form in the source copy of rlhf-and-friends (a theory with its statements as a JSON text, each with its answer, its number of rule uses and how the set made it); the name of a kind gives its form, the numbers of rule uses of its lessons, those that it reads, whether its read part holds as many statements with a proof as with none, and whether it holds its lessons or its read part alone; the lessons come from the train file, the read part from the test file, and for an even kind of the closed form from the dev file and the test file");

const PROOF_KIND: &str = r"pw_(open|closed|people)_d(\d+)(?:to(\d+))?(_even)?(_lessons|_read)?";
because!(PROOF_KIND, ProofTheories, "the name of a kind: the form, the numbers of rule uses of the lessons as one digit each, those of the read part, the mark of an even read part, and the mark of a kind that holds one half alone");

const CLOSED_FORM: &str = "closed";
because!(CLOSED_FORM, ProofTheories, "the form whose statements say how the set made them, read from the source copy");

const LESSONS_HALF: &str = "_lessons";
because!(LESSONS_HALF, ProofTheories, "the end of the name of a kind that holds its lessons alone, the small kind of a pair");

const READ_HALF: &str = "_read";
because!(READ_HALF, ProofTheories, "the end of the name of a kind that holds its read part alone, the large kind of a pair");

const PROOF_TEST: &str = "test";
because!(PROOF_TEST, ProofTheories, "the role in the description of the file whose statements are read");

const PROOF_DEV: &str = "dev";
because!(PROOF_DEV, ProofTheories, "the role in the description of the dev file of the closed form, read before the test file by an even kind");

const UNKNOWN_ANSWER: &str = "Unknown";
because!(UNKNOWN_ANSWER, ProofTheories, "the answer of the open form for a statement with no proof, as the open copy has no field that says how a statement was made");

const PAIRED: &[(&str, &str)] = &[("proof", "rconc"), ("inv-proof", "inv-rconc")];
because!(PAIRED, ProofTheories, "how the closed form made a statement with a proof, and the way of a statement with none that an even kind keeps as many of: the head of a rule that does not fire, and the same with not put in or taken out");

const CONFIG_COLUMN: &str = "config";
because!(CONFIG_COLUMN, ProofTheories, "the column of the open copy whose value, taken from the description, gives the rows of a form");

const PYTHON_TRUE: &str = "True";
because!(PYTHON_TRUE, ProofTheories, "how Python writes a truth value that holds, the answer of a statement of the closed form that holds");

const PYTHON_FALSE: &str = "False";
because!(PYTHON_FALSE, ProofTheories, "how Python writes a truth value that does not hold");

struct ProofRow {
    case: Case,
    depth: i64,
    strategy: String,
}
because!(ProofRow, ProofTheories, "one statement of a theory as a case, with the number of rule uses of its proof and how the set made it (for the open form, its answer)");

fn rule_uses(value: Option<&Value>) -> Result<i64, String> {
    match value {
        Some(Value::Int(number)) => Ok(*number),
        _ => Err("a number of rule uses that is no whole number".to_string()),
    }
}
because!(rule_uses, ProofTheories, "the depth of the proof of a statement, by which a kind takes it, from a value that must be a whole number");

fn python_said(value: Option<&Value>) -> Result<String, String> {
    match value {
        Some(Value::Text(text)) => Ok(text.clone()),
        Some(Value::Bool(yes)) => Ok(if *yes { PYTHON_TRUE } else { PYTHON_FALSE }.to_string()),
        Some(Value::Int(number)) => Ok(number.to_string()),
        _ => Err("a field of a statement with no text".to_string()),
    }
}
because!(python_said, ProofTheories, "a field of a statement as Python writes it as a text: a text as it is, a truth value as True or False, a whole number in digits");

fn proof_rows(described: &Described, form: &str, part: &str) -> Result<Vec<ProofRow>, String> {
    let mut out: Vec<ProofRow> = Vec::new();
    for place in described.files(form, part) {
        if form == CLOSED_FORM {
            for [theory, questions] in rows_where(&place, ["theory", "questions"], None)? {
                let told = told_sentences(&text_value(&theory, &place)?);
                let Value::Struct(statements) = parse_json(&text_value(&questions, &place)?)? else { return Err(format!("{place}: statements that are no group")) };
                for (_, statement) in statements {
                    let case = Case { sentences: told.clone(), question: python_said(statement.get("question"))?, answer: python_said(statement.get("answer"))? };
                    out.push(ProofRow { case, depth: rule_uses(statement.get("QDep"))?, strategy: python_said(statement.get("strategy"))? });
                }
            }
            continue;
        }
        let config = Value::Text(described.setting(form).ok_or_else(|| format!("the description gives no configuration of the form {form}"))?.to_string());
        for [theory, question, answer, depth] in rows_where(&place, ["theory", "question", "answer", "QDep"], Some((CONFIG_COLUMN, &config)))? {
            let answer = text_value(&answer, &place)?;
            let case = Case { sentences: told_sentences(&text_value(&theory, &place)?), question: text_value(&question, &place)?, answer: answer.clone() };
            out.push(ProofRow { case, depth: rule_uses(Some(&depth))?, strategy: answer });
        }
    }
    Ok(out)
}
because!(proof_rows, ProofTheories, "the statements of the files of a form in a role, in the order of the files: of the closed form each statement of each theory, of the open form each row of its configuration");

fn proof_read_part(described: &Described, form: &str, depths: &[i64], even: bool) -> Result<Vec<Case>, String> {
    if !even {
        return Ok(proof_rows(described, form, PROOF_TEST)?.into_iter().filter(|row| depths.contains(&row.depth)).map(|row| row.case).collect());
    }
    let mut out: Vec<Case> = Vec::new();
    if form != CLOSED_FORM {
        let (mut proved, mut unproved) = (0usize, 0usize);
        for row in proof_rows(described, form, PROOF_TEST)?.into_iter().filter(|row| depths.contains(&row.depth)) {
            let (mine, other) = if row.strategy == UNKNOWN_ANSWER { (&mut unproved, proved) } else { (&mut proved, unproved) };
            if *mine <= other {
                *mine += 1;
                out.push(row.case);
            }
        }
        return Ok(out);
    }
    let mut kept: BTreeMap<String, usize> = BTreeMap::new();
    for part in [PROOF_DEV, PROOF_TEST] {
        for row in proof_rows(described, form, part)?.into_iter().filter(|row| depths.contains(&row.depth)) {
            let count = |name: &str| kept.get(name).copied().unwrap_or_default();
            let partner = PAIRED.iter().find(|(made, _)| *made == row.strategy).map(|(_, other)| *other);
            if partner.is_none_or(|other| count(&row.strategy) < count(other)) {
                *kept.entry(row.strategy).or_default() += 1;
                out.push(row.case);
            }
        }
    }
    Ok(out)
}
because!(proof_read_part, ProofTheories, "the read part of a kind: the test statements of its numbers of rule uses; for an even kind of the open form a statement is kept while its class (a proof, or the answer Unknown) has no more kept statements than the other; for an even kind of the closed form, over the dev and then the test statements, a statement with a proof is kept while fewer of it are kept than of its partner with none");

fn depths_of(digits: &str) -> Result<Vec<i64>, String> {
    digits.chars().map(|digit| digit.to_string().parse::<i64>().map_err(|_| format!("no number of rule uses {digit}"))).collect()
}
because!(depths_of, ProofTheories, "the numbers of rule uses that the name of a kind gives, one digit each");

pub fn proof_cases(described: &Described, kind: &str) -> Result<KindCases, String> {
    let groups = TextPattern::new(PROOF_KIND)?.find(kind, true).ok_or_else(|| format!("the kind {kind} is no kind of this set"))?;
    let [_, Some(form), Some(small), large, even, half] = groups.as_slice() else { return Err(format!("the kind {kind} is no kind of this set")) };
    let small = depths_of(small)?;
    let even = even.is_some();
    let new_words = described.new_words();
    if half.as_deref() == Some(READ_HALF) {
        return Ok(KindCases { cases: proof_read_part(described, form, &small, even)?, new_words, ..KindCases::default() });
    }
    let mut cases: Vec<Case> = proof_rows(described, form, LESSON_FILES)?.into_iter().filter(|row| small.contains(&row.depth)).map(|row| row.case).collect();
    if half.as_deref() == Some(LESSONS_HALF) {
        return Ok(KindCases { cases, new_words, ..KindCases::default() });
    }
    let read_from = Some(cases.len());
    let read = match large {
        Some(large) => depths_of(large)?,
        None => small,
    };
    cases.extend(proof_read_part(described, form, &read, even)?);
    Ok(KindCases { cases, read_from, new_words, ..KindCases::default() })
}
because!(proof_cases, ProofTheories, "the cases of a kind: the train statements of the numbers of rule uses of its lessons, then its read part; a kind of one half holds that half alone with no place to read from");
