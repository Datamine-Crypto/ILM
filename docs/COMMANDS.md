# Commands and files

The program `ilm` has one command for each step. `ilm --help` prints the forms of every command, and `ilm --version` prints the version.

## Data

```
ilm fetch <list of sources> [sets=<set>,<set>] [store=<folder of the encyclopedia>] [folder=<folder of a set>] [check=yes]
ilm export <set> <description file> <folder of the case files>
ilm store <folder of the copy of the encyclopedia> <folder of the store> <description of the probe> <description of the questions> [others=<signs>]
ilm classes <folder of the copy of the encyclopedia> <folder of the table> [files=<n>] [bytes=<n>] [classes=<n>]
ilm made <folder> [families=<count>] [held=1]
ilm parquet <file> [column,column,...]
ilm shared-text <folder of data>[,<folder of data>] [run=<tokens>] [words=<tokens of letters>] [only=<part of a path>]
```

- `fetch` gets the raw files that `data/sources.txt` names (all files, or the files of the named sets) and keeps each file only with its SHA-256. With `check=yes` it only checks the files that are there.
- `export` reads the raw files of a set, as its description names them, and writes its case files, the tables of new words, `split.txt` and `source.txt`.
- `store` makes the text store of the fact reader from a copy of an encyclopedia in parquet files.
- `classes` makes the table of word classes from the same copy: `classes_1000.txt` (a word, its class and its count on each line) and `classes_1000_read.txt` (the first words of each class). `bytes=` sets the text read from each file (100 MB when not named), `files=` the number of files (all when not named), `classes=` the number of classes (1,000 when not named).
- `made` writes the case files of the made families: families of texts that a program draws from a seed. `held=1` opens the held families, for their one test.
- `parquet` prints the rows of a parquet file.
- `shared-text` finds runs of tokens that a text file of the repository shares with a row of the data files under the given folders. It prints the files and the line numbers, never the text.

## Measure

```
ilm board <folder of the sets> [set,set,...] [tools=<names>] [classes=<file>] [lanes=<n>] [out=<file>] [sealed=1]
ilm measure <folder of a set, or a case file> [kind,kind,...] [tool=<name>] [tools=<names>] [classes=<file>] [fit=<n>] [held=<n>] [curve=1] [newwords=1] [show=<n>] [lanes=<n>] [small=1] [sealed=1]
ilm choice <case file> [tools=<names>] [classes=<file>] [lanes=<n>]
ilm control <case file of the kind> <file of changed cases> [tool=<name>] [whole=1] [most=<n>] [classes=<file>] [lanes=<n>]
ilm changes <control> <case file of a kind> [whole=1] [root=<folder>] [part=read] [support=<folder>] [forms=<description>] [about=<description>]
ilm check <case file> <tool> <file of answers for the last lessons> <file of answers for the read part> [classes=<file>] [lanes=<n>]
ilm check-all <folder of answers> <name of the files of answers> <tool> cases=<folder>,<folder> [sources=<set>,<set>] [write=1] [classes=<file>] [lanes=<n>]
```

- `board` measures every kind of every set of a folder through the choice among the tools, and prints the table of the tasks. `sealed=1` reads the sealed part in place of the read part: for one final run with frozen code only.
- `measure` prints one line for each kind of a set: the sizes of the parts, the controls and the share of right answers. Its last line gives the right answers of the read part for each kind. `fit=` and `held=` set the most lessons and the most read cases. `small=1` reads a kind whatever the size of its read part. `tool=` takes one tool in place of the choice. `sealed=1` reads the sealed part of each kind in place of its read part: for one final run with frozen code only.
- `choice` prints what each tool reads right of the lessons that it did not learn from, and the tool that is taken.
- `control` measures whether a model follows changed cases. A model of the first two thirds of the lessons (or of all lessons with `whole=1`) reads the changed forms of the lessons that it answers right. The line says how often it gives the new true answer where the truth turns and where it stays.
- `changes` writes the changed cases of a kind as a file for `control`. Each control has a name, for example `swaps_control` or `parts_control`. `ilm changes` alone lists them. `root=` names the folder for the file; `part=read` writes the changes of the read part; `support=`, `forms=` and `about=` name the files that some controls read beside the case file.
- `check` compares the answers of one tool with two files of answers. `check-all` does it for every kind of a folder of answers, and with `write=1` it writes the files of answers of the present program.

## Facts, questions and requests

```
ilm know <folder of the text store> <file of facts> [part=probe] [text=asked|all] [fit=<n>] [classes=<file>] [lanes=<n>] [dump=<folder>]
ilm questions <file of questions> [names=<file of known names>] [fit=<n>] [dump=<folder>]
ilm ask build <folder of the text store> <file of facts> <file of questions> <file of known names> <folder of the sets> <board file> [classes=<file>] [table=<folder>]
ilm ask measure <folder of the table> <file of questions> [dump=<file>]
ilm ask kinds <folder of the table> <file of questions>
ilm ask <folder of the table> "<request>" [classes=<file>]
ilm ask <folder of the table> [classes=<file>]
```

- `know` learns a reader of facts from the lesson facts, reads the articles of the store into its table, and answers the asked facts from the table. Without `part=probe` it asks the development part of the lesson facts.
- `questions` learns a reader of questions from the lesson questions and reads the held questions: the subject, the relation, and both. With a file of known names it knows which runs of words are names.
- `ask build` makes the table of the one command: the facts of every article of the store, and the router of the kinds of the counted tasks. The board file is the output of `ilm board ... out=<file>`.
- `ask` answers one request with a trace of every step. With no request it is a session: it answers each line of the input, with an empty line after each answer. `ask measure` answers the held questions about facts, and `ask kinds` measures where the requests of known kinds go.

## Writing

```
ilm write out <train rows> <dev, or a file of rows> <lessons> <out file> [all]
ilm write metric-files <file of rows of a part> <file of written texts> <references out> <texts out>
ilm write measure <train rows> <dev, or a file of rows> <lessons> <file of wanted texts>
ilm write texts <train rows> <dev, or a file of rows> <lessons> <file of wanted texts> [lists to show]
ilm write traces <train rows> <dev, or a file of rows> <lessons> <file of wanted traces> [lists to show]
ilm write model <train rows> <lessons> <out file>
```

- `write out` learns a writer and writes a text for each list of facts of a part. With `all` the lessons are every train row.
- `write metric-files` writes the references and the written texts in the form that the official E2E metrics read.
- `write measure`, `texts` and `traces` compare the numbers, the texts and the traces of a writer with a file of wanted values. `write model` prints what a model counted.

## Look inside

```
ilm patterns <words|model|rules|answers> <case file> ...
ilm patterns long-story <far|many> <case file> <file of the words of people and things>
ilm cases <case file>
ilm words <text>
ilm answer-words <text>
```

- `patterns` shows what the pattern learner counted for a kind, stage by stage: its words and kinds, its model, its rules and its answers. `patterns long-story` reads a long story once into an index and measures the answers.
- `cases`, `words` and `answer-words` show how a file and a text are read.

## Options for all commands

- `classes=<file>` gives a table of word classes: a word and its class at the start of each line. The program reads it once and gives it to every tool with its lessons.
- The program uses all threads of the computer. The variable `ILM_LANES` or the option `lanes=<n>` sets another number. The result is the same for every number of threads.

## Files

A set is a folder of plain files:

- `<kind>.txt` holds the cases of one kind. The first line starts with `#` and holds named values (`fit`, `held`, `read_from`, `fit_stories`). A case is a line `S`, a tab and a told sentence for each told sentence, then `Q` with the question, then `A` with the answer, then an empty line. A lesson case can hold label lines of the set (`U`, `F`, `C`, `H`, `T`).
- `<kind>.words.txt` holds the new words of a kind: a word, a tab, its new word.
- `source.txt` holds what the set gives beside its cases: its name, who wrote its text, the most lessons and read cases, its tasks with their kinds, and its pairs of a small and a large kind.

The text store is a folder with files of lines, one article on a line as a JSON object with `title` and `text`. A file of facts holds one fact on a line: `L` or `P` (lesson or probe), the relation, the subject, the object and the sentence form, with tabs between them. A file of questions holds one question on a line: `L` or `H` (lesson or held), the question, the subject, the relation, the label of the relation and the answer. A file of known names holds one name on a line, as its letters, in rising order.

A file of rows for the writer holds one row for each text: a line `F`, a tab, a slot, a tab and a value for each fact, then a line `T` with the text, then an empty line. A file of written texts holds a line `W` and a tab before each text.
