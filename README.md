# 🧮 ILM: Integer Language Model

[![crates.io](https://img.shields.io/crates/v/datamine-ilm.svg)](https://crates.io/crates/datamine-ilm) [![CI](https://github.com/Datamine-Crypto/ILM/actions/workflows/ci.yml/badge.svg)](https://github.com/Datamine-Crypto/ILM/actions/workflows/ci.yml) [![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

⚡ **CPU only.** ILM is a language model that needs no GPU, to learn or to answer. On one desktop CPU it learns each benchmark below from its lessons, then answers its test questions.

🤔 **Why no GPU?** A neural network, such as a transformer, learns by changing millions of decimal weights. It repeats billions of multiplications over many passes through the data, and a GPU is built for that work. ILM has no weights. What it learns is lines of English: rules, question forms and facts. It keeps a line when the line answers the lessons right, and it finds this by counting. Counting is the addition of whole numbers, and a normal CPU does it fast.

🧠 ILM learns from lessons: a short text, a question and the exact answer. What it learned is a set of lines that you can read, and each answer names the lines that gave it, with a proof tree when steps prove it. ILM gives the same answers on every run and for every number of threads.

- 🤗 Model card and programs: [huggingface.co/DatamineNetwork/ILM](https://huggingface.co/DatamineNetwork/ILM)
- 📦 Crate: [crates.io/crates/datamine-ilm](https://crates.io/crates/datamine-ilm)
- 🌐 Website: [datamine.network](https://datamine.network)

## 📊 Results

Each benchmark is read with frozen code on its test data: the official test split, or for LoNLI the part that [docs/RESULTS.md](docs/RESULTS.md) names. The runs use one desktop CPU (Intel Core i9-12900K, 24 threads, no GPU). The time is for the whole run, learning from all lessons and answering the test questions, as the best of three or more runs.

| Benchmark | ILM | Published, for comparison | Time |
|---|---|---|---|
| bAbI, 20 tasks, 1,000 examples each | 19 of 20 tasks pass, 97.44% right (18,903 of 19,399; 601 test questions with the story of a lesson are not read) | the best published learner from answers only that we found: 3 tasks fail (Universal Transformer) | 14.0 s |
| SCAN, 4 splits | 100% (17,016 of 17,016) | Transformer: length 0.0, add jump 1.0 | 1.5 s |
| ProofWriter, depth 5, open and closed form | 99.52% (39,870 of 40,063) | ProofWriter (T5-11B): 99.6 to 99.7 | 0.4 s |
| LoNLI, 17 groups of logic in NLI | 98.18% on its sealed part (15 groups) | tuned on MNLI, not on LoNLI: 59 to 71; RoBERTa-large tuned on 2% of LoNLI: near 100 | 2.5 s |
| CLUTRR, graph form, 2 to 10 steps | 99.14% (1,036 of 1,045) | trained on 2 to 4 steps, read at 10 steps: Transformer 70, NCRL 97, R5 98 | 1.4 s |
| Sentence transformations: question and passive, English and German | 100% whole sentences right (80,000 of 80,000) | Transformer from scratch: whole sentences near 0% on the new builds | 1.6 s |
| PCFG SET | 100% (31,229 of 31,229) | Transformer: productivity 50, systematicity 72 | 8.4 s |
| ToM-QA, 12 kinds | 100% (1,950 of 1,950) | best published for each form: 88.6 to 100 | 2.1 s |

The papers give some of these as fractions; they are shown here as percentages.

Where ILM is strong: meaning built from parts. On SCAN length and add jump, on the longer inputs and the new pairs of PCFG SET, and on the new builds of the sentence transformations, transformers that learn from scratch stay low; ILM learns each from 1,000 lessons. It joins up to 10 relations of CLUTRR from lessons of 2 to 4 relations, and it passes 19 of the 20 bAbI tasks from 1,000 lessons each.

Where ILM is weak: text that people wrote. ILM learns no task of the board whose text people wrote: the stories of CLUTRR, the word problems (MAWPS, ASDiv, SVAMP), the descriptions of regular expressions of NL-RX, the rules in the words of people of ParaRules, and the intents and slots of ATIS and SNIPS. On these, pre-trained transformers lead. It gets no result on StepGame, SpartQA, FLD and BoardgameQA, and it does not learn the second belief of ToMi. On bAbI, task 14 (time) fails on its test split, 519 of 1,000 right (51.9%), although ILM reads its development data at 100%: learned from 943 or more of its lessons, it reads "before" in the order of the story, and the stories of that task tell the times out of order. On ProofWriter, rules of the library read the told rules: that number measures the engine, not rules that ILM learned, and it is level with ProofWriter (T5-11B), not ahead. The ToM-QA number is not a read of unseen cases: the board read that test split during development (shares only, never a wrong case). During development the board read 527 of the 1,045 CLUTRR graph test graphs (shares only, never a wrong case).

What ILM does that a transformer does not: a proved answer comes with its proof tree in English, back to the told sentences; each rule in it is marked as written by a person or discovered from the lessons; each fact read from a large text names the file and the line of its sentence; and in the code only two commands write the files of the library or of the store.

[docs/RESULTS.md](docs/RESULTS.md) gives the settings, the counts, the published numbers with their sources, how ILM stands against transformers with the evidence for each claim, and notes on each benchmark.

## 🚀 Quick start

You need the stable Rust toolchain ([rustup.rs](https://rustup.rs)), a C compiler and `curl`. The C compiler: the C++ build tools of Visual Studio on Windows, `xcode-select --install` on macOS, `build-essential` on Ubuntu. Run the commands from the root of the clone, in bash or PowerShell (in cmd.exe, the program is `target\release\ilm.exe`). The release build takes about 4 minutes.

```
cargo build --release
./target/release/ilm fetch data/sources.txt sets=src_babi
./target/release/ilm export src_babi data/sets/final/src_babi.txt cases/final/src_babi
./target/release/ilm measure cases/final/src_babi
```

`fetch` gets the raw files of bAbI from the Hugging Face hub at a fixed revision and checks their SHA-256. `export` writes a case file for each of the 20 tasks. `measure` learns each task from its 1,000 lessons and reads its test questions. The last line gives the right answers of each task; their sum is 18,903 of 19,399. [docs/REPRODUCE.md](docs/REPRODUCE.md) gives the commands for every benchmark.

To see a proof, give a story and a question to `ilm prove`, with the case file of a kind whose lessons the model learns from:

```
./target/release/ilm prove "Zoe went to the studio. Zoe picked up the lamp. Zoe went to the cellar. Where is the lamp?" cases=cases/final/src_babi/babi02.txt
```

It prints the answer and its proof tree, back to the told sentences:

```
cellar
   proof of the answer location: C if latest B has the A now and latest B went to the C
     read at its time: latest zoe has the lamp now (the latest value of this state at that point of the story)
     zoe has the lamp now by the rule: If A got the B and latest the holder of the B is C then C has the B now [given]
       zoe got the lamp by the rule: If A picked up the B then A got the B [discovered on babi02]
         told: zoe picked up the lamp
       read at its time: latest the holder of the lamp is zoe (the latest value of this state at that point of the story)
       the holder of the lamp is zoe by the rule: If A got the B then the holder of the B is A [given]
         zoe got the lamp (its proof is shown above)
     read at its time: latest zoe went to the cellar (the latest value of this state at that point of the story)
     told: zoe went to the cellar
```

Each rule in the tree is marked as given (a line of the library) or as discovered from the lessons of the kind.

To use a program of the GitHub release in place of a build, copy it into the root of the clone and call it there:

- Linux: `cp <download folder>/ilm-linux-x86_64 ./ilm && chmod +x ./ilm`, then `./ilm fetch data/sources.txt sets=src_babi`, and so on.
- macOS on Apple silicon: `cp <download folder>/ilm-macos-arm64 ./ilm && chmod +x ./ilm && xattr -d com.apple.quarantine ./ilm` (the program is not signed with an Apple developer certificate), then `./ilm ...`. On an Intel Mac, build the program from source.
- Windows: copy `ilm-windows-x86_64.exe` as `ilm.exe`, then `.\ilm.exe fetch data/sources.txt sets=src_babi`, and so on.

`SHA256SUMS.txt` of the release gives the SHA-256 of each program.

📦 With Rust, `cargo install --locked datamine-ilm` builds and installs the program `ilm` from [crates.io](https://crates.io/crates/datamine-ilm). Run it from the root of the clone in the same way.

## ⚙️ How it works

ILM has one tool, the explain tool, and a library of texts in plain English (`data/explain`). A text holds rules ("If A is a parent of B and B is a parent of C then A is a grandparent of C."), told facts, question forms with their answers, and lines that write a sentence again. The engine chains the rules over the told sentences of a case until nothing new comes, and it answers with the first answer whose conditions hold.

For each kind of task, the tool learns lines of the library from the lessons:

- Learned wording: the forms of the questions of a kind, and the answers of each form, from facts that the rules make.
- Discovered rules: rules that read the told sentences and rules that join facts, found from the lessons alone and kept when they answer the lessons right. Rules that one kind discovers are kept for the next kinds of its set, and the store in `data/store` offers the rules that the board discovered to other kinds, which judge them again on their own lessons.
- Ways of joining (SCAN), the meanings of function words, and the rewrites of a sentence into another build (the sentence transformations).
- Facts from a large text: `ilm facts` reads the facts of a text of the library from plain text that people wrote. The file `data/explain/learned/capitals.txt` holds the capitals that it read from a plain-text copy of the English Wikipedia, each with the files and lines of its sentences; the copy is not in the repository.

The tool tries its ways on two thirds of the lessons and judges them on the last third; then it learns again from all lessons. No person chooses a way for a kind, and no setting changes between tasks. A learned line is written as a line of the library, so a person reads what was learned.

Each answer has a mark: proof (steps prove it, and the text holds no pair of pieces that the lessons did not show), default (an "otherwise" line gave it), guess, or none. `ilm prove` prints the proof tree of an answer, and `ilm why` shows, for each lesson that a model answers wrong, the first step that the facts of the story miss.

In 11 of the 79 tasks that ILM learns on the board, every rule that the proofs rest on was discovered from the lessons, among them SCAN, the four sentence transformations, CLUTRR in the graph form and bAbI tasks 9 and 10. [docs/HOW_IT_WORKS.md](docs/HOW_IT_WORKS.md) lists what ILM discovered and its features, and describes each part.

## 🚧 What ILM is not

ILM is not a general chat model. It learns one kind of task at a time from lessons of that kind, and each lesson needs the exact answer. It does not learn tasks whose text people wrote (see Results).

The board measures 25 public sets (240 kinds in 112 tasks) on held cases of each kind. ILM learns 79 tasks: every kind of the task at 95% right or more (90% where people wrote the text), and 5 points or more above the controls. The board also says how each learned task reads: 15 discovered (read by learned lines, with no rule of the library), 29 learned (learned lines with rules of the library), 7 mixed, and 28 written (lines of the library alone).

## 🦀 Code

The program is a Rust workspace of five crates:

| Crate | What it holds |
|---|---|
| `crates/ilm-core` | words and numbers, the explain tool with its engine, its learners and its proofs, the choice of its ways, the cut of the cases |
| `crates/ilm-data` | the readers of the public sets, the fetch of their files, the facts step over a large text, the writers of changed cases |
| `crates/ilm-eval` | the measure of a kind, the board, the controls |
| `crates/ilm-commands` | the commands |
| `crates/ilm` | the program `ilm` |

```
cargo build --release
cargo test --release
```

The build needs Rust 1.97 or later. A release build takes about 4 minutes, because of full link-time optimization; `cargo build --profile fast` makes a program for development in less time, with the same answers. `cargo build --profile quick` has no link-time optimization: after a small change it builds in seconds, and the program runs a little slower; use it to edit and test, and `fast` for timed runs.

The code follows the rules of the crate [premise](https://crates.io/crates/premise): an idea is a type with its published source, every constant, type and function states its reason, and the code holds no comments and no unnamed numbers. The tests use made-up cases and need no downloaded data. [docs/COMMANDS.md](docs/COMMANDS.md) lists the commands and the file formats.

The program uses the memory allocator [mimalloc](https://crates.io/crates/mimalloc), so the build needs a C compiler.

## 📚 Data

ILM reads public sets from the Hugging Face hub and GitHub at fixed revisions. `data/sources.txt` names every raw file with its SHA-256, and `data/sets` describes how each set becomes case files. The repository holds no data of the sets. Each set keeps its own license. The library of texts is in `data/explain`, and the store of discovered rules is in `data/store`.

## 📜 License

Apache License 2.0. See [LICENSE](LICENSE).
