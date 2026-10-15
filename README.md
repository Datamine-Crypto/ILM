# 🧮 ILM: Integer Language Model

[![crates.io](https://img.shields.io/crates/v/datamine-ilm.svg)](https://crates.io/crates/datamine-ilm) [![CI](https://github.com/Datamine-Crypto/ILM/actions/workflows/ci.yml/badge.svg)](https://github.com/Datamine-Crypto/ILM/actions/workflows/ci.yml) [![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

⚡ **CPU only.** ILM is a language model that needs no GPU, to learn or to answer. On one desktop CPU it learns each benchmark below from its lessons in seconds, then answers its test questions.

🤔 **Why no GPU?** A neural network, such as a transformer, learns by changing millions of decimal weights. It repeats billions of multiplications over many passes through the data, and a GPU is built for that work. ILM has no weights. It stores whole-number counts: how often each pattern in a text came with each answer. Counting is the addition of whole numbers, and a normal CPU does it fast.

🧠 ILM learns from lessons: a short text, a question and the exact answer (for two tasks below, also the labels that the set gives). It keeps a pattern only when its counts show that the pattern is reliable. So what it learned is a set of rules that you can read, and it gives the same answers on every run and for every number of threads.

- 🤗 Model card and programs: [huggingface.co/DatamineNetwork/ILM](https://huggingface.co/DatamineNetwork/ILM)
- 📦 Crate: [crates.io/crates/datamine-ilm](https://crates.io/crates/datamine-ilm)
- 🌐 Website: [datamine.network](https://datamine.network)

## 📊 Results

Each benchmark is read with frozen code on its test data: the official test split, or for LoNLI, LAMA and SVAMP the part that [docs/RESULTS.md](docs/RESULTS.md) names. The runs use one desktop CPU (Intel Core i9-12900K, 24 threads, no GPU). The time is for the whole run, learning from all lessons and answering the test questions, as the best of three or more runs.

| Benchmark | ILM | Published, for comparison | Time |
|---|---|---|---|
| bAbI, 20 tasks, 1,000 examples each | 20 of 20 tasks pass, 99.78% right | best published learner from answers only: 3 tasks fail (Universal Transformer) | 5.9 s |
| SCAN, 4 splits | 100% on every split | Transformer: length 0.0, add jump 1.0 | 0.8 s |
| ProofWriter, depth 5 | 100% (40,063 of 40,063; closed depth 5 also learns from the formal forms of the set) | ProofWriter, T5-11B: 99.6 to 99.7 | 4.0 s |
| LoNLI, 17 groups of logic in NLI | 94.9% on its sealed part (15 groups; 10 of 16 learned) | tuned on MNLI, not on LoNLI: 59 to 71; RoBERTa-large tuned on 2% of LoNLI: near 100 | 0.7 s |
| CLUTRR, 2 to 10 steps | 72.3% (learns from the step labels of the set) | GPT-4o with a solver: 67.7; Faithful CoT with votes: 71.9 | 15.9 s |
| LAMA T-REx | 64.0% | BERT-large: 32.3; MegatronLM 11B with trained prompts, on a subset: 64.2 | 11.6 s |
| SVAMP | 38.3% | Graph2Tree: 36.5; with RoBERTa: 43.8; GPT-3 with chain of thought: 68.9 | 27.6 s |
| E2E NLG | BLEU 65.37 | TGen: 65.93; GPT-2 large with LoRA: 70.4 | 9.8 s |

ILM is at or near 100% on bAbI, SCAN and ProofWriter. It is in the upper range on CLUTRR, LAMA and LoNLI. On SVAMP and E2E it is in the middle, and pre-trained transformers lead. [docs/RESULTS.md](docs/RESULTS.md) gives the settings, the published numbers with their sources, and notes on each benchmark.

## 🚀 Quick start

You need the stable Rust toolchain ([rustup.rs](https://rustup.rs)), a C compiler and `curl`. The C compiler: the C++ build tools of Visual Studio on Windows, `xcode-select --install` on macOS, `build-essential` on Ubuntu. Run the commands from the root of the clone, in bash or PowerShell (in cmd.exe, the program is `target\release\ilm.exe`). The release build takes about 4 minutes.

```
cargo build --release
./target/release/ilm fetch data/sources.txt sets=src_babi
./target/release/ilm export src_babi data/sets/final/src_babi.txt cases/final/src_babi
./target/release/ilm measure cases/final/src_babi
```

`fetch` gets the raw files of bAbI from the Hugging Face hub at a fixed revision and checks their SHA-256. `export` writes a case file for each of the 20 tasks. `measure` learns each task from its 1,000 lessons and reads its test questions. The last line gives the right answers of each task; their sum is 19,356 of 19,399. [docs/REPRODUCE.md](docs/REPRODUCE.md) gives the commands for every benchmark.

To use a program of the GitHub release in place of a build, copy it into the root of the clone and call it there:

- Linux: `cp <download folder>/ilm-linux-x86_64 ./ilm && chmod +x ./ilm`, then `./ilm fetch data/sources.txt sets=src_babi`, and so on.
- macOS on Apple silicon: `cp <download folder>/ilm-macos-arm64 ./ilm && chmod +x ./ilm && xattr -d com.apple.quarantine ./ilm` (the program is not signed with an Apple developer certificate), then `./ilm ...`. On an Intel Mac, build the program from source.
- Windows: copy `ilm-windows-x86_64.exe` as `ilm.exe`, then `.\ilm.exe fetch data/sources.txt sets=src_babi`, and so on.

`SHA256SUMS.txt` of the release gives the SHA-256 of each program.

📦 With Rust, `cargo install datamine-ilm` builds and installs the program `ilm` from [crates.io](https://crates.io/crates/datamine-ilm). Run it from the root of the clone in the same way.

## ⚙️ How it works

ILM has fifteen tools, each named for its way of reading: patterns, state, amount, order, link, rule, parts, together, list, requests, frames, table, arrange, graph and ranks. For each kind of task, every tool learns from two thirds of the lessons and reads the last third. The tool with the most right answers learns again from all lessons. No person chooses a tool, and no setting changes between tasks.

Beside the tools, ILM has a fact reader for encyclopedia text, a question reader, a writer that turns given facts into a text, and one command (`ilm ask`) that routes each request and prints a trace of its steps.

[docs/HOW_IT_WORKS.md](docs/HOW_IT_WORKS.md) describes each part.

## 🚧 What ILM is not

ILM is not a general chat model. It learns one kind of task at a time from lessons of that kind, and each lesson needs the exact answer. On the board of 25 public sets (240 kinds in 112 tasks), it learns 59 tasks: every kind of the task at 95% right or more on held cases (90% where people wrote the text).

## 🦀 Code

The program is a Rust workspace of five crates:

| Crate | What it holds |
|---|---|
| `crates/ilm-core` | words and numbers, the fifteen tools, the choice among them, the readers of facts and questions, the writer |
| `crates/ilm-data` | the readers of the public sets, the fetch of their files, the text store, the writers of changed cases |
| `crates/ilm-eval` | the measure of a kind, the board, the controls |
| `crates/ilm-commands` | the commands and the one command |
| `crates/ilm` | the program `ilm` |

```
cargo build --release
cargo test --release
```

The build needs Rust 1.90 or later. A release build takes about 4 minutes, because of full link-time optimization; `cargo build --profile fast` makes a program for development in less time, with the same answers.

The code follows the rules of the crate [premise](https://crates.io/crates/premise): an idea is a type with its published source, every constant, type and function states its reason, and the code holds no comments and no unnamed numbers. The tests use made-up cases and need no downloaded data. [docs/COMMANDS.md](docs/COMMANDS.md) lists every command and file format.

The program uses the memory allocator [mimalloc](https://crates.io/crates/mimalloc), so the build needs a C compiler.

## 📚 Data

ILM reads public sets from the Hugging Face hub and GitHub at fixed revisions. `data/sources.txt` names every raw file with its SHA-256, and `data/sets` describes how each set becomes case files. The repository holds no data of the sets. Each set keeps its own license.

## 📜 License

Apache License 2.0. See [LICENSE](LICENSE).
