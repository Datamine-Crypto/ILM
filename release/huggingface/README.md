---
license: apache-2.0
language:
- en
pipeline_tag: question-answering
tags:
- cpu
- counting
- no-gradient
- interpretable
- reasoning
- rust
model-index:
- name: ILM
  results:
  - task:
      type: question-answering
      name: Question answering over a story
    dataset:
      name: bAbI, 20 tasks, 1,000 examples each
      type: Muennighoff/babi
      split: test
    metrics:
    - type: accuracy
      value: 99.78
      name: Accuracy
  - task:
      type: text2text-generation
      name: Commands to actions
    dataset:
      name: SCAN, simple split
      type: marcov/scan_simple_promptsource
      split: test
    metrics:
    - type: exact_match
      value: 100.0
      name: Exact match
  - task:
      type: text2text-generation
      name: Commands to actions
    dataset:
      name: SCAN, length split
      type: marcov/scan_length_promptsource
      split: test
    metrics:
    - type: exact_match
      value: 100.0
      name: Exact match
  - task:
      type: text2text-generation
      name: Commands to actions
    dataset:
      name: SCAN, add jump split
      type: marcov/scan_addprim_jump_promptsource
      split: test
    metrics:
    - type: exact_match
      value: 100.0
      name: Exact match
  - task:
      type: text2text-generation
      name: Commands to actions
    dataset:
      name: SCAN, add turn left split
      type: marcov/scan_addprim_turn_left_promptsource
      split: test
    metrics:
    - type: exact_match
      value: 100.0
      name: Exact match
  - task:
      type: question-answering
      name: Deduction over told rules
    dataset:
      name: ProofWriter, depth 5, open form
      type: tasksource/proofwriter
      config: depth-5
      split: test
    metrics:
    - type: accuracy
      value: 100.0
      name: Accuracy
  - task:
      type: question-answering
      name: Deduction over told rules
    dataset:
      name: ProofWriter, depth 5, closed form
      type: rlhf-and-friends/proofwriter-source
      config: CWA-depth-5
      split: test
    metrics:
    - type: accuracy
      value: 100.0
      name: Accuracy
  - task:
      type: text-classification
      name: Natural language inference over 17 groups of logic
    dataset:
      name: LoNLI, the sealed part of 15 groups
      type: tasksource/lonli
    metrics:
    - type: accuracy
      value: 94.9
      name: Accuracy
  - task:
      type: question-answering
      name: Kinship over a story written by people
    dataset:
      name: CLUTRR, 2 to 10 steps
      type: CLUTRR/v1
      config: gen_train234_test2to10
      split: test
    metrics:
    - type: accuracy
      value: 72.3
      name: Accuracy
  - task:
      type: question-answering
      name: Facts from encyclopedia text
    dataset:
      name: LAMA, T-REx part
      type: facebook/lama
      config: trex
    metrics:
    - type: precision_at_1
      value: 64.0
      name: Precision at 1
  - task:
      type: question-answering
      name: Word problems
    dataset:
      name: SVAMP, standard setting (300 test problems)
      type: ChilleD/SVAMP
      split: test
    metrics:
    - type: accuracy
      value: 38.3
      name: Accuracy
  - task:
      type: text-generation
      name: Text from a list of facts
    dataset:
      name: E2E NLG
      type: tuetschek/e2e_nlg
      split: test
    metrics:
    - type: bleu
      value: 65.37
      name: BLEU
    - type: nist
      value: 8.29
      name: NIST
---

# 🧮 ILM: Integer Language Model

⚡ **CPU only.** ILM is a language model that needs no GPU, to learn or to answer. On one desktop CPU it learns each benchmark below from its lessons in seconds, then answers its test questions.

🤔 **Why no GPU?** A neural network, such as a transformer, learns by changing millions of decimal weights. It repeats billions of multiplications over many passes through the data, and a GPU is built for that work. ILM has no weights. It stores whole-number counts: how often each pattern in a text came with each answer. Counting is the addition of whole numbers, and a normal CPU does it fast.

🧠 ILM learns from lessons: a short text, a question and the exact answer (for two tasks below, also the labels that the set gives). It keeps a pattern only when its counts show that the pattern is reliable. So what it learned is a set of rules that you can read, and it gives the same answers on every run and for every number of threads.

- 💻 Source code, documentation and the commands that give each number below: https://github.com/Datamine-Crypto/ILM
- 📦 Crate: https://crates.io/crates/datamine-ilm
- 🌐 Website: https://datamine.network

## 📁 Files

| File | What it is |
|---|---|
| `ilm-windows-x86_64.exe` | the program for Windows on x86-64 |
| `ilm-linux-x86_64` | the program for Linux on x86-64 (glibc 2.35 or later, as on Ubuntu 22.04) |
| `ilm-macos-arm64` | the program for macOS on Apple silicon |
| `SHA256SUMS.txt` | the SHA-256 of each program |
| `LICENSE` | the Apache License 2.0 |
| `THIRD_PARTY_NOTICES.md` | the licenses of the code of others that the programs hold |

ILM has no weights to download: it learns each task from the lessons of that task when you run it. The program reads the descriptions of the sets and the list of sources from the GitHub repository, so run it from a clone of that repository.

## 📊 Results

Each benchmark is read with frozen code on its test data: the official test split, or for LoNLI, LAMA and SVAMP the part that [docs/RESULTS.md](https://github.com/Datamine-Crypto/ILM/blob/main/docs/RESULTS.md) names. The runs use one desktop CPU (Intel Core i9-12900K, 24 threads, no GPU). The time is for the whole run, learning from all lessons and answering the test questions, as the best of three or more runs.

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

The settings, the published numbers with their sources, and notes on each benchmark are in [docs/RESULTS.md](https://github.com/Datamine-Crypto/ILM/blob/main/docs/RESULTS.md) of the GitHub repository.

## 🚀 How to use

The program reads the descriptions of the sets from a clone of the GitHub repository. With the program of this repository, on Linux:

```
git clone https://github.com/Datamine-Crypto/ILM ilm
cd ilm
cp <download folder>/ilm-linux-x86_64 ./ilm
chmod +x ./ilm
./ilm fetch data/sources.txt sets=src_babi
./ilm export src_babi data/sets/final/src_babi.txt cases/final/src_babi
./ilm measure cases/final/src_babi
```

On macOS, take `ilm-macos-arm64` in the same way, and also run `xattr -d com.apple.quarantine ./ilm`, because the program is not signed with an Apple developer certificate. On an Intel Mac, build the program from source. On Windows, copy `ilm-windows-x86_64.exe` into the clone as `ilm.exe` and run `.\ilm.exe fetch data/sources.txt sets=src_babi`, and so on. To build the program yourself in place of a download: `cargo build --release` (Rust 1.90 or later and a C compiler), then call `./target/release/ilm`. `fetch` gets the raw files of a set from the Hugging Face hub or GitHub at a fixed revision and checks their SHA-256. `export` writes a case file for each kind of the set. `measure` learns each kind from its lessons and reads its test cases. [docs/REPRODUCE.md](https://github.com/Datamine-Crypto/ILM/blob/main/docs/REPRODUCE.md) gives the commands for every benchmark.

## ⚙️ How it works

ILM has fifteen tools, each named for its way of reading: patterns, state, amount, order, link, rule, parts, together, list, requests, frames, table, arrange, graph and ranks. For each kind of task, every tool learns from two thirds of the lessons and reads the last third. The tool with the most right answers learns again from all lessons. No person chooses a tool, and no setting changes between tasks. Beside the tools, ILM has a fact reader for encyclopedia text, a question reader, a writer that turns given facts into a text, and one command (`ilm ask`) that routes each request and prints a trace of its steps.

## 🚧 Limits

ILM is not a general chat model. It learns one kind of task at a time from lessons of that kind, and each lesson needs the exact answer. On a board of 25 public sets (240 kinds in 112 tasks), it learns 59 tasks: every kind of the task at 95% right or more on held cases (90% where people wrote the text). On SVAMP and E2E, pre-trained transformers are ahead.

## 📜 License

Apache License 2.0. The sets that ILM reads keep their own licenses, and this repository holds none of their data.
