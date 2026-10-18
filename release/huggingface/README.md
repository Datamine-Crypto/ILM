---
license: apache-2.0
language:
- en
- de
pipeline_tag: question-answering
datasets:
- Muennighoff/babi
- marcov/scan_simple_promptsource
- marcov/scan_length_promptsource
- marcov/scan_addprim_jump_promptsource
- marcov/scan_addprim_turn_left_promptsource
- tasksource/proofwriter
- rlhf-and-friends/proofwriter-source
- tasksource/lonli
- CLUTRR/v1
- amueller/syntactic_transformations
- Lots-of-LoRAs/task151_tomqa_find_location_easy_clean
- Lots-of-LoRAs/task152_tomqa_find_location_easy_noise
- Lots-of-LoRAs/task153_tomqa_find_location_hard_clean
- wikimedia/wikipedia
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
      name: bAbI, 20 tasks, 1,000 examples each (18,903 of 19,399 read questions; 601 test questions with the story of a lesson are not read)
      type: Muennighoff/babi
      split: test
    metrics:
    - type: accuracy
      value: 97.44
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
      value: 99.34
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
      value: 99.70
      name: Accuracy
  - task:
      type: text-classification
      name: Natural language inference over 17 groups of logic
    dataset:
      name: LoNLI, the sealed part of 15 groups
      type: tasksource/lonli
    metrics:
    - type: accuracy
      value: 98.18
      name: Accuracy
  - task:
      type: question-answering
      name: Kinship over told relations
    dataset:
      name: CLUTRR, graph form, 2 to 10 relations
      type: CLUTRR/v1
      config: gen_train234_test2to10
      split: test
    metrics:
    - type: accuracy
      value: 99.14
      name: Accuracy
  - task:
      type: text2text-generation
      name: A sentence into a question or into the passive
    dataset:
      name: Syntactic transformations, test and generalization sets of 4 parts
      type: amueller/syntactic_transformations
    metrics:
    - type: exact_match
      value: 100
      name: Whole sentence right
  - task:
      type: question-answering
      name: Belief over a story
    dataset:
      name: ToM-QA, easy, 4 kinds (650 test questions)
      type: Lots-of-LoRAs/task151_tomqa_find_location_easy_clean
      split: test
    metrics:
    - type: accuracy
      value: 100
      name: Accuracy
  - task:
      type: question-answering
      name: Belief over a story
    dataset:
      name: ToM-QA, easy with noise, 4 kinds (650 test questions)
      type: Lots-of-LoRAs/task152_tomqa_find_location_easy_noise
      split: test
    metrics:
    - type: accuracy
      value: 100
      name: Accuracy
  - task:
      type: question-answering
      name: Belief over a story
    dataset:
      name: ToM-QA, hard, 4 kinds (650 test questions)
      type: Lots-of-LoRAs/task153_tomqa_find_location_hard_clean
      split: test
    metrics:
    - type: accuracy
      value: 100
      name: Accuracy
---

# 🧮 ILM: Integer Language Model

⚡ **CPU only.** ILM is a language model that needs no GPU, to learn or to answer. On one desktop CPU it learns each benchmark below from its lessons, then answers its test questions.

🤔 **Why no GPU?** A neural network, such as a transformer, learns by changing millions of decimal weights. It repeats billions of multiplications over many passes through the data, and a GPU is built for that work. ILM has no weights. What it learns is lines of English: rules, question forms and facts. It keeps a line when the line answers the lessons right, and it finds this by counting. Counting is the addition of whole numbers, and a normal CPU does it fast.

🧠 ILM learns from lessons: a short text, a question and the exact answer. What it learned is a set of lines that you can read, and each answer names the lines that gave it, with a proof tree when steps prove it. ILM gives the same answers on every run and for every number of threads.

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

ILM has no weights to download: it learns each task from the lessons of that task when you run it. The program reads its library of texts, the descriptions of the sets and the list of sources from the GitHub repository, so run it from a clone of that repository.

## 📊 Results

Each benchmark is read with frozen code on its test data: the official test split, or for LoNLI the part that [docs/RESULTS.md](https://github.com/Datamine-Crypto/ILM/blob/main/docs/RESULTS.md) names. The runs use one desktop CPU (Intel Core i9-12900K, 24 threads, no GPU). The time is for the whole run, learning from all lessons and answering the test questions, as the best of three or more runs.

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

The settings, the counts, the published numbers with their sources, how ILM stands against transformers with the evidence for each claim, and notes on each benchmark are in [docs/RESULTS.md](https://github.com/Datamine-Crypto/ILM/blob/main/docs/RESULTS.md) of the GitHub repository.

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

On macOS, take `ilm-macos-arm64` in the same way, and also run `xattr -d com.apple.quarantine ./ilm`, because the program is not signed with an Apple developer certificate. On an Intel Mac, build the program from source. On Windows, copy `ilm-windows-x86_64.exe` into the clone as `ilm.exe` and run `.\ilm.exe fetch data/sources.txt sets=src_babi`, and so on. To build the program yourself in place of a download: `cargo build --release` (Rust 1.97 or later and a C compiler), then call `./target/release/ilm`. `fetch` gets the raw files of a set from the Hugging Face hub or GitHub at a fixed revision and checks their SHA-256. `export` writes a case file for each kind of the set. `measure` learns each kind from its lessons and reads its test cases. `prove` prints the answer to a story and a question with its proof tree. [docs/REPRODUCE.md](https://github.com/Datamine-Crypto/ILM/blob/main/docs/REPRODUCE.md) gives the commands for every benchmark.

## ⚙️ How it works

ILM has one tool, the explain tool, and a library of texts in plain English: rules, told facts, question forms with their answers, and lines that write a sentence again. The engine chains the rules over the told sentences of a case and answers with the first answer whose conditions hold. For each kind of task, the tool learns lines of the library from the lessons: the wording of the questions, rules discovered from the lessons alone, the ways of joining of a language of commands, and the rewrites of a sentence into another build. The facts step reads facts of the library from plain text that people wrote (the capitals of the library come from a plain-text copy of the English Wikipedia). The tool tries its ways on two thirds of the lessons and judges them on the last third; no person chooses a way for a kind, and no setting changes between tasks. Each answer is marked proof, default, guess or none, and `ilm prove` prints its proof tree.

## 🚧 Limits

ILM is not a general chat model. It learns one kind of task at a time from lessons of that kind, and each lesson needs the exact answer. On a board of 25 public sets (240 kinds in 112 tasks), it learns 79 tasks: every kind of the task at 95% right or more on held cases (90% where people wrote the text). It learns no task whose text people wrote: the stories of CLUTRR, the word problems (MAWPS, ASDiv, SVAMP), the descriptions of regular expressions of NL-RX, the rules in the words of people of ParaRules, and the intents and slots of ATIS and SNIPS. On these, pre-trained transformers lead. It gets no result on StepGame, SpartQA, FLD and BoardgameQA. On bAbI, task 14 (time) fails on its test split, 519 of 1,000 right (51.9%), although ILM reads its development data at 100%: learned from 943 or more of its lessons, it reads "before" in the order of the story, and the stories of that task tell the times out of order. On ProofWriter, rules of the library read the told rules: that number measures the engine, not rules that ILM learned, and it is level with ProofWriter (T5-11B), not ahead. The ToM-QA number is not a read of unseen cases: the board read that test split during development (shares only, never a wrong case). During development the board read 527 of the 1,045 CLUTRR graph test graphs (shares only, never a wrong case).

## 📜 License

Apache License 2.0. The sets that ILM reads keep their own licenses, and this repository holds none of their data.
