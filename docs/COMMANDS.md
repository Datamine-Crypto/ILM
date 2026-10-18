# Commands and files

The program `ilm` has one command for each step. `ilm --help` prints the forms of every command, and `ilm --version` prints the version. The help also lists `ilm store`, `ilm classes`, the option `classes=` and `fetch store=`; no result of these pages uses them.

## Data

```
ilm fetch <list of sources> [sets=<set>,<set>] [folder=<folder of a set>] [check=yes]
ilm export <set> <description file> <folder of the case files>
ilm parquet <file> [column,column,...]
ilm shared-text <folder of data>[,<folder of data>] [run=<tokens>] [words=<tokens of letters>] [only=<part of a path>]
```

- `fetch` gets the raw files that `data/sources.txt` names (all files, or the files of the named sets) and keeps each file only with its SHA-256. With `check=yes` it only checks the files that are there.
- `export` reads the raw files of a set, as its description names them, and writes its case files, the tables of new words, `split.txt` and `source.txt`.
- `parquet` prints the number of rows of a parquet file and a digest of the values of each named column (of every column when it names none), so that a reader of the file can be compared with another reader without one value being shown.
- `shared-text` finds runs of tokens that a text file of the repository shares with a row of the data files under the given folders. It prints the files and the line numbers, never the text.

## Measure

```
ilm board <folder of the sets> [set,set,...] [tools=<names>] [lanes=<n>] [out=<file>] [jobs=<file>] [order=<file>] [sealed=1] [kept=<folder> [engine=<fingerprint>]] [compare=<reference board file>] [fit=<n>]
ilm compare <reference board file> <board file>
ilm measure <folder of a set, or a case file> [kind,kind,...] [tool=<name>] [tools=<names>] [fit=<n>] [held=<n>] [curve=1] [newwords=1] [show=<n>] [lanes=<n>] [small=1] [sealed=1]
ilm choice <case file> [tools=<names>] [lanes=<n>]
ilm control <case file of the kind> <file of changed cases> [tool=<name>] [whole=1] [most=<n>] [lanes=<n>]
ilm changes <control> <case file of a kind> [whole=1] [root=<folder>] [part=read] [support=<folder>] [forms=<description>] [about=<description>]
```

- `board` measures every kind of every set of a folder (or of the named sets) and prints the table of the tasks. For each task the table gives who wrote the text of the set, the lessons, the kinds learned, whether the task is learned, the lowest share, the control of the question alone, the share with new words, whether the task is also learned from 200 lessons, the pair of a small and a large kind, what reads it (read by, rules by, facts by), the answer marks and the seconds. The lines after the table count the learned tasks by these columns. `out=` also writes the table to a file. `jobs=` writes when each job ran (the kind, the pass, the guess of its work, its start, its end, its seconds and the tool that was taken), to see which jobs hold the whole run. `order=` reads such a file of an earlier board and starts the jobs that took longest there first; the order changes no result. `kept=` names a folder that lasts between runs: the board reads back the chains of rules of an earlier run whose engine is the same, and writes the new ones; a board without it reads and writes no such file. `kept=` needs a program built in the clone, because the program finds its engine from the sources of its build: a downloaded or installed program refuses it. `compare=` prints what changed from a reference board after the table. `fit=` sets the most lessons of every set; the controls still learn from the lessons of the set. `sealed=1` reads the sealed part in place of the read part: for one final run with frozen code only.
- `compare` prints what changed from a reference board to a board: each changed field of a task, the tasks that are slower, and the changed lines after the table.
- `tools=` names the tools of the choice, in the order of the list: `explain`, and the controls `plain` and `question`.
- `measure` prints one line for each kind of a set: the sizes of the parts, the controls and the share of right answers, with the answers marked by proof, default, guess or none. Its last line gives the right answers of the read part for each kind. `fit=` and `held=` set the most lessons and the most read cases. `small=1` reads a kind whatever the size of its read part. `tool=` takes one tool in place of the choice. `curve=1` adds the share right by the number of lessons, `newwords=1` reads the read cases again with new words, and `show=` prints that many wrong lessons. `sealed=1` reads the sealed part of each kind in place of its read part: for one final run with frozen code only.
- `choice` prints what each tool reads right of the lessons that it did not learn from, and the tool that is taken.
- `control` measures whether a model follows changed cases. A model of the first two thirds of the lessons (or of all lessons with `whole=1`) reads the changed forms of the lessons that it answers right. The line says how often it gives the new true answer where the truth turns and where it stays.
- `changes` writes the changed cases of a kind as a file for `control`. Each control has a name, for example `swaps_control`. `ilm changes` alone lists them. `root=` names the folder for the file; `part=read` writes the changes of the read part; `support=`, `forms=` and `about=` name the files that some controls read beside the case file.

## Look inside the explain tool

```
ilm prove "<told sentences> <question>" [cases=<case file>] [lanes=<n>]
ilm why <case file> [show=<n>] [all=1] [before=<case file>,...] [fit=<n>] [held=<n>] [lanes=<n>]
ilm concepts [<folder of the sets>] [lanes=<n>]
ilm places <text of the library, or a folder of them>...
ilm wording <case file> topic=<name of a text of the library> [told=1]
ilm joining <case file> [fit=<n>] [held=<n>]
ilm meanings <case file> [fit=<n>] [held=<n>]
ilm explain-cost <case file> [show=<n>] [fit=<n>] [held=<n>] [lanes=<n>]
ilm store-lines <case file> topic=<name of a text of the library>
ilm facts <folder of a large text> topic=<name of a text of the library> kinds=<case file>[,<case file>] [out=<file>] [dump=<file>] [files=<n>] [fit=<n>] [lanes=<n>]
```

- `prove` answers told sentences and a question (the last sentence) and prints the proof tree of the answer, back to the told sentences. Each rule in the tree is marked as given (a line of the library) or as discovered on the kind, and each known fact that was learned is marked as discovered. With no case file, the texts of the library answer alone; with `cases=`, the model that the explain tool learns from the lessons of the case file answers.
- `why` learns a model of the explain tool from the first two thirds of the lessons of a case file (all lessons with `all=1`) and reads the rest of the lessons. For each lesson that it answers wrong (the first ten, or `show=`), it prints the story, the wanted and the given answer, the proof of the given answer, and the first missing step for the wanted answer: of the answers of the model's texts that could give it, the one whose conditions hold furthest, with the conditions that hold and the first one that the facts of the story miss. The last lines count the missing steps, the most common first. `before=` names the case files of the kinds before the kind in its set, whose lessons the model also takes, as the board gives them. It reads lessons only, never the read part.
- `concepts` prints the topics of the library (`data/explain`): each topic with the topics that it builds on and its number of sentences (rules, question forms, reply sentences, told facts). With a folder of sets it also prints, for each topic, every kind where it answers most of its sample of lessons.
- `places` prints each line of the texts of the library that holds open places, with the places that the reader takes in it, for a check of names read as places.
- `wording` learns the wording of a kind from the first two thirds of its lessons, for one text of the library: each question becomes a form (the words that change from lesson to lesson are open places), and each form gets answers whose conditions are facts that the rules of the text make from the story. It prints the learned forms as lines of the library and, on the last third of the lessons, how many the learned forms and the written forms of the text answer right. With `told=1` it first learns rules that read the told sentences, and prints them first.
- `joining` learns the ways of joining of a language of commands (each word of meaning by the token that its lessons always hold, each joining word by the answer form that reads most lessons) and prints them as lines of the library, with how many lessons and read cases the engine reads right with them.
- `meanings` learns the meanings of the function words of a kind whose questions are function words over lists, over the text of the library that names functions on lists, and prints the learned lines and how many lessons and read cases the engine reads right with them.
- `explain-cost` learns from the lessons of a case file and reads its read part with the engine timing each rule, and prints the rules that took the most time, with their share.
- `store-lines` discovers the lines that the lessons of a case file teach a text of the library, and adds them to the file of that text in `data/store`, after a note of their source (the kind, its set and the number of lessons). It is the only command that writes the store.
- `facts` reads the facts of a text of the library from a large plain text that people wrote. It finds first facts by the text's own form, keeps those that the proofs of the lessons of the named kinds confirm, learns the shapes of the sentences that state such facts, finds more facts with those shapes, and judges them again by the proofs of the lessons. `out=` writes the facts as a learned text, each with the files and lines of its sentences; `dump=` writes the shapes; `files=` reads only the first files of the folder.

## Look inside files and words

```
ilm cases <case file>
ilm words <text>
ilm answer-words <text>
```

- `cases` prints the number of cases of a case file and the named values of its first line.
- `words` prints the words of a text as the learners count them; `answer-words` prints the words of an answer as the measure compares them.

## Build

```
cargo build --release
cargo build --profile fast
cargo build --profile quick
```

- `release` has full link-time optimization. Use it for the published times.
- `fast` builds in less time and gives the same answers. Use it for boards and other timed runs.
- `quick` optimizes across the code of each crate, not across crates. After a small change it builds in seconds and gives the same answers. Use it to edit and test (`cargo test --profile quick`); take the times of timed runs from `fast`.

## Options for all commands

- The program uses all threads of the computer. The variable `ILM_LANES` or the option `lanes=<n>` sets another number. The result is the same for every number of threads.
- Every command but `board` passes over an option that it does not know, so a misspelled option (`heldd=8000`) leaves the default in place: check the spelling. The board refuses an option that it does not read.

## Files

A set is a folder of plain files:

- `<kind>.txt` holds the cases of one kind. The first line starts with `#` and holds named values (`fit`, `held`, `read_from`, `fit_stories`). A case is a line `S`, a tab and a told sentence for each told sentence, then `Q` with the question, then `A` with the answer, then an empty line. A lesson case can hold label lines of the set (`U`, `F`, `C`, `H`, `T`).
- `<kind>.words.txt` holds the new words of a kind: a word, a tab, its new word.
- `source.txt` holds what the set gives beside its cases: its name, who wrote its text, the most lessons and read cases, its tasks with their kinds, and its pairs of a small and a large kind.
- `split.txt` holds the sizes of the parts of each kind: the cases, the lessons, the read part and the sealed part.

The library is a folder of plain texts, `data/explain`, with the folders `concepts`, `knowledge`, `wording` and `learned`. [HOW_IT_WORKS.md](HOW_IT_WORKS.md) gives the forms of their lines. A text in `learned` was written by a program, not by a person: its first note holds a print of its lines, and the program refuses the text when a person changed a line of it, or when a written text has the same name.

The store, `data/store`, holds one file for each text of the library that has stored lines: each line is a rule that the board discovered, after a note with the kind, the set and the number of lessons that it came from.
