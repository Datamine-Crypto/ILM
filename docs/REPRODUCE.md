# Reproduce the results

This page gives the commands for each number of [RESULTS.md](RESULTS.md). Run all commands from the root of the repository. Each run prints the counts of right answers. On one computer these counts are the same in every run and for every number of threads. Another operating system can give a logarithm that differs in its last bit, which in a rare close decision could move a count; the CI checks the bAbI counts on Linux, Windows and macOS. The times change with the computer.

## What you need

- Rust 1.97 or later and a C compiler (the memory allocator mimalloc needs one). A release build takes about 4 minutes.
- `curl`, which `ilm fetch` uses to get the public data files.
- Disk space: less than 1 GB for each benchmark.

Build the program:

```
cargo build --release
```

The program is `target/release/ilm` (`target\release\ilm.exe` on Windows). The commands below call it `ilm`. To call it by name, put its folder first on the path: `export PATH="$PWD/target/release:$PATH"` in bash, or `$env:Path = "$PWD\target\release;" + $env:Path` in PowerShell. The loop of the board below is for bash (Git Bash on Windows).

## How the data comes in

`data/sources.txt` names every raw file that ILM reads: its host, repository, revision, path and SHA-256. `ilm fetch` gets the files of the sets that you name into `data/raw`, and it keeps a file only when its SHA-256 is right. Run it again to check the files: a file with the right hash is not fetched again.

`ilm export` reads the raw files of a set, as its description in `data/sets` names them, and writes one case file for each kind. A description in `data/sets/final` reads the official test split where the description of the board does not read it alone: for bAbI the board reads only the train and validation questions, and for CLUTRR in the graph form and ToM-QA it also reads the validation split.

`ilm measure` learns each kind from its lessons and reads its read part. `held=` sets the most read cases of a kind: a number above the size of the test split reads all of it. A list of kinds after the folder measures those kinds only. `small=1` reads a kind however few cases it has. `sealed=1` reads the sealed part of each kind in place of its read part. The last line of the output gives the right answers of each kind.

## bAbI, 20 tasks

```
ilm fetch data/sources.txt sets=src_babi
ilm export src_babi data/sets/final/src_babi.txt cases/final/src_babi
ilm measure cases/final/src_babi
```

The lessons are the 1,000 train and validation questions of each task, and the read part is its test questions. Expected: babi01 998 of 998, babi02 1000 of 1000, babi03 1000 of 1000, babi04 560 of 560, babi05 998 of 1000, babi06 998 of 998, babi07 997 of 999, babi08 994 of 1000, babi09 1000 of 1000, babi10 997 of 997, babi11 1000 of 1000, babi12 1000 of 1000, babi13 998 of 998, babi14 519 of 1000, babi15 1000 of 1000, babi16 995 of 1000, babi17 968 of 968, babi18 1000 of 1000, babi19 1000 of 1000, babi20 881 of 881; 18,903 of 19,399 in all.

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
ilm measure cases/src_proofwriter pw_open_d0,pw_open_d1,pw_open_d2,pw_open_d3,pw_open_d4,pw_open_d5,pw_closed_d0,pw_closed_d1,pw_closed_d2,pw_closed_d3,pw_closed_d4,pw_closed_d5 held=100000
```

The list names the twelve kinds of one depth of proof each; the set has more kinds for the board. Expected: pw_open_d0 6194 of 6194, pw_open_d1 4518 of 4527, pw_open_d2 2866 of 2879, pw_open_d3 2348 of 2364, pw_open_d4 2073 of 2097, pw_open_d5 1888 of 1959, pw_closed_d0 6125 of 6125, pw_closed_d1 4290 of 4315, pw_closed_d2 2884 of 2897, pw_closed_d3 2457 of 2467, pw_closed_d4 2184 of 2192, pw_closed_d5 2043 of 2047; 39,870 of 40,063 in all (open form 19,887 of 20,020, closed form 19,983 of 20,043).

## LoNLI

```
ilm fetch data/sources.txt sets=src_lonli
ilm export src_lonli data/sets/src_lonli.txt cases/src_lonli
ilm measure cases/src_lonli sealed=1
```

`sealed=1` reads the sealed part of each group in place of its read part. Expected: lonli_boolean 1000 of 1000, lonli_causal 1000 of 1000, lonli_comparative 999 of 1000, lonli_conditional 1000 of 1000, lonli_coreference 999 of 1000, lonli_implicature 995 of 995, lonli_lexical 831 of 997, lonli_negation 994 of 994, lonli_numerical 1000 of 1000, lonli_presupposition 994 of 1000, lonli_quantifier 997 of 1000, lonli_relational 905 of 1000, lonli_spatial 1000 of 1000, lonli_syntactic 1000 of 1000, lonli_taxonomic 937 of 1000, lonli_temporal 1000 of 1000; 14,714 of 14,986 over the 15 groups with unseen cases (lonli_taxonomic reads copies of lessons and is not in the total; lonli_world has no sealed part).

## CLUTRR, graph form

```
ilm fetch data/sources.txt sets=src_clutrr_graph
ilm export src_clutrr_graph data/sets/final/src_clutrr_graph.txt cases/final/src_clutrr_graph
ilm measure cases/final/src_clutrr_graph clutrrg_k2,clutrrg_k3,clutrrg_k4,clutrrg_k5,clutrrg_k6,clutrrg_k7,clutrrg_k8,clutrrg_k9,clutrrg_k10 small=1
```

`small=1` reads every kind of the test split, also a kind with fewer than 100 test graphs. Expected: clutrrg_k2 35 of 35, clutrrg_k3 102 of 107, clutrrg_k4 73 of 77, clutrrg_k5 185 of 185, clutrrg_k6 105 of 105, clutrrg_k7 155 of 155, clutrrg_k8 135 of 135, clutrrg_k9 124 of 124, clutrrg_k10 122 of 122; 1,036 of 1,045 in all.

## Sentence transformations

```
ilm fetch data/sources.txt sets=src_transform
ilm export src_transform data/sets/src_transform.txt cases/src_transform
ilm measure cases/src_transform held=10000
```

Each kind whose name ends in `_test` reads the whole test file of its part, and each kind whose name ends in `_gen` the whole generalization file. Expected: each of the 8 kinds 10000 of 10000; 80,000 of 80,000 in all.

## PCFG SET

```
ilm fetch data/sources.txt sets=src_pcfg
ilm export src_pcfg data/sets/src_pcfg.txt cases/src_pcfg
ilm measure cases/src_pcfg held=12000
```

Expected: pcfg_plain 9721 of 9721, pcfg_longer 11333 of 11333, pcfg_new 10175 of 10175.

## ToM-QA

```
ilm fetch data/sources.txt sets=src_tomqa
ilm export src_tomqa data/sets/final/src_tomqa.txt cases/final/src_tomqa
ilm measure cases/final/src_tomqa
```

Expected: tomqa_easy_real 167 of 167, tomqa_easy_begin 157 of 157, tomqa_easy_look 171 of 171, tomqa_easy_think 155 of 155, tomqa_noise_real 156 of 156, tomqa_noise_begin 171 of 171, tomqa_noise_look 166 of 166, tomqa_noise_think 157 of 157, tomqa_hard_real 167 of 167, tomqa_hard_begin 158 of 158, tomqa_hard_look 170 of 170, tomqa_hard_think 155 of 155; 1,950 of 1,950 in all.

## The board

The board reads the read parts of all kinds of 25 sets:

```
for set in src_atis src_babi src_bigbench src_boardgame src_clutrr src_clutrr_graph src_fld src_lonli src_mindgames src_nlgraph src_pararules src_pcfg src_prontoqa src_proofwriter src_prosqa src_regex src_ruletaker src_scan_step src_snips src_spartqa src_stepgame src_tomi src_tomqa src_transform src_wordproblems; do
  ilm fetch data/sources.txt sets=$set
  ilm export $set data/sets/$set.txt cases/board/$set
done
ilm board cases/board
```

Expected: `240 kinds in 112 tasks; 165 kinds learned; 79 tasks learned in 14 sets`. `out=<file>` also writes the table to a file, and `ilm compare <board file> <board file>` prints what changed from one board to another.

## Facts from a large text

The board reads `data/explain/learned/capitals.txt` as it is in the repository. The facts step (`ilm facts`) wrote it from a plain-text copy of the English Wikipedia (from wikimedia/wikipedia on the Hugging Face hub, 20231101.en, in 10 files), with the lessons of lonli_world as the judge. The copy is not in the repository, and `ilm fetch` does not get it. Each fact of the file names the files and lines of the sentences that gave it, but the release does not hold the steps that made the 10 plain-text files from that copy, so these files and lines cannot be checked from the repository alone.
