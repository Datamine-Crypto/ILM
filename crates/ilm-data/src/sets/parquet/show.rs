use crate::sets::parquet::{digest, open};
use patterns::{because, source};
use std::path::Path;
use std::process::ExitCode;

pub struct ParquetCommand;
source!(ParquetCommand, "the part of the program that reads a parquet file in the open: the rows, and for each top field of the file the digest of its values in their fixed form, so that the reader can be compared with another reader of the same file without one value being shown");

const BETWEEN_COLUMNS: char = ',';
because!(BETWEEN_COLUMNS, ParquetCommand, "the mark between the names of the columns that a run asks for");

pub fn parquet_command(rest: &[String]) -> ExitCode {
    let Some(place) = rest.first() else {
        eprintln!("usage: ilm parquet <file> [column,column,...]");
        return ExitCode::FAILURE;
    };
    let mut file = match open(Path::new(place)) {
        Ok(file) => file,
        Err(why) => {
            eprintln!("{why}");
            return ExitCode::FAILURE;
        }
    };
    let names: Vec<String> = match rest.get(1) {
        Some(asked) => asked.split(BETWEEN_COLUMNS).map(str::to_string).collect(),
        None => file.names(),
    };
    println!("rows {}", file.meta.rows);
    for name in &names {
        match file.read(&[name.as_str()]) {
            Ok(columns) => println!("column {name} rows {} digest {}", columns[0].len(), digest(&columns[0])),
            Err(why) => println!("column {name} fault {why}"),
        }
    }
    ExitCode::SUCCESS
}
because!(parquet_command, ParquetCommand, "the rows of a parquet file and the digest of each asked top field, or of every top field; a field that the reader cannot read is named with the cause and the others go on");
