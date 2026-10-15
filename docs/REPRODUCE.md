# Reproduce the results

This page gives the commands for each number of [RESULTS.md](RESULTS.md). Run all commands from the root of the repository. Each run prints the counts of right answers. On one computer these counts are the same in every run and for every number of threads. Another operating system can give a logarithm that differs in its last bit, which in a rare close decision could move a count; the CI checks the bAbI counts on Linux, Windows and macOS. The times change with the computer.

## What you need

- Rust 1.90 or later and a C compiler (the memory allocator mimalloc needs one). A release build takes about 4 minutes.
- `curl`, which `ilm fetch` uses to get the public data files.
- Disk space: less than 1 GB for each benchmark, and 13 GB for LAMA (the Wikipedia copy and its text store).
- For E2E only: Python 3 and the official E2E metrics; for all five of its metrics also Java 1.8.

Build the program:

```
cargo build --release
```

The program is `target/release/ilm` (`target\release\ilm.exe` on Windows). The commands below call it `ilm`. To call it by name, put its folder first on the path: `export PATH="$PWD/target/release:$PATH"` in bash, or `$env:Path = "$PWD\target\release;" + $env:Path` in PowerShell. The loop of the board below is for bash (Git Bash on Windows).

## How the data comes in

`data/sources.txt` names every raw file that ILM reads: its host, repository, revision, path and SHA-256. `ilm fetch` gets the files of the sets that you name into `data/raw`, and it keeps a file only when its SHA-256 is right. Run it again to check the files: a file with the right hash is not fetched again.

`ilm export` reads the raw files of a set, as its description in `data/sets` names them, and writes one case file for each kind. The descriptions in `data/sets/final` read the official test splits.

## bAbI, 20 tasks

```
ilm fetch data/sources.txt sets=src_babi
ilm export src_babi data/sets/final/src_babi.txt cases/final/src_babi
ilm measure cases/final/src_babi
```

The last line gives the right answers of each task. The sum is 19,356 of 19,399: task 3 has 989 of 1000, task 5 998 of 1000, task 17 958 of 968, task 18 980 of 1000, and every other task has all its questions right.

## SCAN

```
ilm fetch data/sources.txt sets=src_scan_step
ilm export src_scan_step data/sets/src_scan_step.txt cases/src_scan_step
ilm measure cases/src_scan_step held=8000
```

Expected: scan_simple 4182 of 4182, scan_length 3920 of 3920, scan_addprim_jump 7706 of 7706, scan_addprim_turn_left 1208 of 1208.

## ProofWriter, depth 5

```
ilm fetch data/sources.txt sets=src_proofwriter
ilm export src_proofwriter data/sets/src_proofwriter.txt cases/src_proofwriter
ilm measure cases/src_proofwriter held=100000
```

Expected: every kind pw_open_d0 to pw_open_d5 and pw_closed_d0 to pw_closed_d5 has all its read cases right, 40,063 in all (open form 20,020, closed form 20,043). For pw_closed_d5 the measure marks STRONGER LESSONS: its lessons also have the formal forms that the set gives.

## LoNLI

```
ilm fetch data/sources.txt sets=src_lonli
ilm export src_lonli data/sets/src_lonli.txt cases/src_lonli
ilm measure cases/src_lonli sealed=1
```

`sealed=1` reads the sealed part of each group in place of its read part. Expected: lonli_boolean 991 of 1000, lonli_causal 1000 of 1000, lonli_comparative 581 of 1000, lonli_conditional 1000 of 1000, lonli_coreference 992 of 1000, lonli_implicature 992 of 995, lonli_lexical 880 of 997, lonli_negation 994 of 994, lonli_numerical 898 of 1000, lonli_presupposition 1000 of 1000, lonli_quantifier 966 of 1000, lonli_relational 1000 of 1000, lonli_spatial 973 of 1000, lonli_syntactic 1000 of 1000, lonli_taxonomic 1000 of 1000 (copies of lessons, not in the total), lonli_temporal 955 of 1000; 14,222 of 14,986 over the 15 groups with unseen cases.

## CLUTRR

```
ilm fetch data/sources.txt sets=src_clutrr
ilm export src_clutrr data/sets/final/src_clutrr.txt cases/final/src_clutrr
ilm measure cases/final/src_clutrr clutrr_k2,clutrr_k3,clutrr_k4,clutrr_k5,clutrr_k6,clutrr_k7,clutrr_k8,clutrr_k9,clutrr_k10 small=1
```

The option `small=1` reads every kind of the test split, also a kind with fewer than 100 test stories. Expected: clutrr_k2 26 of 38, clutrr_k3 86 of 107, clutrr_k4 59 of 77, clutrr_k5 142 of 185, clutrr_k6 78 of 105, clutrr_k7 110 of 155, clutrr_k8 86 of 135, clutrr_k9 87 of 124, clutrr_k10 84 of 122; 758 of 1,048 in all.

## LAMA T-REx

The fact reader reads the English Wikipedia of March 2022 (41 parquet files, 11 GB). `store=` names the folder for these files.

```
ilm fetch data/sources.txt sets=src_lama,src_questions,store store=data/raw/wikipedia
ilm export src_lama data/sets/src_lama.txt cases/src_lama
ilm store data/raw/wikipedia cases/store data/sets/src_lama.txt data/sets/src_questions.txt
ilm classes data/raw/wikipedia cases/classes
ilm know cases/store cases/src_lama/facts.txt part=probe text=all classes=cases/classes/classes_1000.txt
```

`ilm store` writes the text store: the articles of the subjects of the facts and of the questions, and other articles up to 1,000 million signs. `ilm classes` counts the table of word classes from 100 MB of text of each file of the copy; it takes about 40 s on 24 threads and 5 GB of memory. The board uses the same table.

Expected: `part probe: 6791 lesson facts, 27248 facts asked, 41 relations`, then 61.7% right from the text alone (16,817), and the line of the guesses: `facts with no answer in the text, guessed from the subject's name: 1927, right 622; right with the guesses, % of all asked facts: 64.0`.

## SVAMP, standard setting

```
ilm fetch data/sources.txt sets=src_wordproblems
ilm export src_wordproblems data/sets/final/src_wordproblems.txt cases/final/src_wordproblems
ilm measure cases/final/src_wordproblems svamp_standard
```

Expected: svamp_standard 115 of 300. The kind svamp_standard_dev reads the 700 problems of the development part.

## E2E NLG

```
ilm fetch data/sources.txt sets=src_e2e
ilm export src_e2e data/sets/final/src_e2e.txt cases/final/src_e2e
ilm write out cases/final/src_e2e/train.txt cases/final/src_e2e/test.txt 42061 runs/e2e/written.txt all
ilm write metric-files cases/final/src_e2e/test.txt runs/e2e/written.txt runs/e2e/references.txt runs/e2e/texts.txt
```

`write out` learns from all 42,061 train rows and writes a text for each of the 630 test inputs. `write metric-files` writes the two files that the official metrics read. Get the official E2E metrics (GitHub tuetschek/e2e-metrics, commit 1b9c4d87fd0bbb171ea966741cdc012588581ed4). With Java 1.8 and its Python requirements installed, its script gives all five metrics:

```
python <folder of e2e-metrics>/measure_scores.py -p runs/e2e/references.txt runs/e2e/texts.txt
```

Without Java, the script stops before it prints BLEU. BLEU and NIST alone come from the same code of the package (metrics/pymteval.py), which needs only the Python package `future` (`pip install future`). Save these lines as `score.py` and run `python score.py <folder of e2e-metrics>`:

```
import sys
sys.path.insert(0, sys.argv[1])
from metrics.pymteval import BLEUScore, NISTScore
references, group = [], []
for line in open("runs/e2e/references.txt", encoding="utf-8"):
    line = line.strip()
    if line:
        group.append(line)
    elif group:
        references.append(group)
        group = []
if group:
    references.append(group)
texts = [line.strip() for line in open("runs/e2e/texts.txt", encoding="utf-8")]
assert len(texts) == len(references), "the texts and the groups of references differ in number"
bleu, nist = BLEUScore(), NISTScore()
for text, group in zip(texts, references):
    bleu.append(text, group)
    nist.append(text, group)
print(f"BLEU {bleu.score():.4f}, NIST {nist.score():.4f}")
```

Expected: BLEU 0.6537 and NIST 8.2896. RESULTS.md gives BLEU and NIST only.

## The board

The board reads the read parts of all kinds of 25 sets, with the table of word classes of the LAMA step:

```
for set in src_atis src_babi src_bigbench src_boardgame src_clutrr src_clutrr_graph src_fld src_lonli src_mindgames src_nlgraph src_pararules src_pcfg src_prontoqa src_proofwriter src_prosqa src_regex src_ruletaker src_scan_step src_snips src_spartqa src_stepgame src_tomi src_tomqa src_transform src_wordproblems; do
  ilm fetch data/sources.txt sets=$set
  ilm export $set data/sets/$set.txt cases/board/$set
done
ilm board cases/board classes=cases/classes/classes_1000.txt
```

Expected: `240 kinds in 112 tasks; 150 kinds learned; 59 tasks learned in 17 sets`.
