# Results

ILM reads each benchmark below with frozen code on its test data; each section names the part that is read. The published numbers beside it come from the papers named in each table (by arXiv number).

All runs use one desktop computer: an Intel Core i9-12900K with 24 threads, and no GPU. A time is the wall-clock time of the whole run with the release build: the choice among the tools, the controls, the learning from all lessons, and the read of the test data. Each time is the best of three or more runs in turn of the commands of [REPRODUCE.md](REPRODUCE.md), in a fresh copy of this repository with the data fetched from the public sources.

## Summary

| Benchmark | Lessons | ILM | Time |
|---|---|---|---|
| bAbI, 20 tasks | 1,000 for each task, answers only | 20 of 20 tasks pass; 19,356 of 19,399 right (99.78%) | 5.9 s |
| SCAN, 4 splits | the first 1,000 of each train split | 17,016 of 17,016 right (100%) | 0.8 s |
| ProofWriter, depth 5, open and closed form | about 600 for each depth of proof, answers only (closed form, depth 5: also the formal forms of the set) | 40,063 of 40,063 right (100%) | 4.0 s |
| LoNLI, 17 groups of logic in NLI | 1,000 for each group, answers only | 14,222 of 14,986 right (94.9%) on its sealed part | 0.7 s |
| CLUTRR, 2 to 10 steps | all 12,064 training stories, with their step labels | 758 of 1,048 right (72.3%) | 15.9 s |
| LAMA T-REx | 6,791 lesson facts and the text of Wikipedia | 17,439 of 27,248 right (64.0%) | 11.6 s |
| SVAMP, standard setting | 3,061 problems of MAWPS and ASDiv-A, answers only | 115 of 300 right (38.3%) | 27.6 s |
| E2E NLG | all 42,061 train rows | BLEU 65.37, NIST 8.29 | 9.8 s |

Where ILM stands:

- bAbI, SCAN and ProofWriter: ILM reads every test split at or near 100%, from 600 to 1,000 lessons for each kind. We found no published learner that does this with the same lessons.
- LoNLI: far above models tuned on other NLI sets that never see LoNLI, and below RoBERTa-large tuned on a part of LoNLI.
- CLUTRR: above GPT-4o with a graph and a symbolic solver (67.7), and at the level of the best row of the table, Faithful CoT with code-davinci-002 and votes (71.9). ILM learns from the step labels that the set gives.
- LAMA: above every model of BERT size, at the level of an 11-billion-parameter model with trained prompts.
- SVAMP: above the models that learn from scratch, below the pre-trained models and the large language models.
- E2E: at the level of the best systems of the E2E challenge, below fine-tuned GPT-2.

## bAbI, 20 tasks, 1,000 examples

Data: Muennighoff/babi on the Hugging Face hub. Lessons: the 1,000 train and validation questions of each task (900 and 100), with answers only and no supporting facts. Read: the 1,000 test questions of each task. A test question whose story is also the story of a lesson is not read, because the cut keeps a story on one side: 601 questions (tasks 4, 17 and 20 lose 440, 32 and 119; tasks 1, 6, 7, 10 and 13 lose 2, 2, 1, 3 and 2).

ILM: 19,356 of 19,399 right (99.78%). All 20 tasks pass the bar of the bAbI paper (an error of 5% or less). The mean error is 0.22%. Wrong answers: task 3: 11, task 5: 2, task 17: 10, task 18: 20; all other tasks: 0.

| System | Training | Tasks failed (of 20) | Mean error % | Source |
|---|---|---|---|---|
| ILM | counting, 1,000 examples for each task | 0 | 0.22 | this repository |
| N-gram classifier | 1,000 for each task | 20 | (mean accuracy 34) | arXiv 1502.05698 |
| LSTM | gradients | 20 | 51.3 | arXiv 1503.08895 |
| MemN2N, best of 10 runs | gradients | 11 | 13.9 | arXiv 1503.08895 |
| Entity network | gradients | 15 | 29.6 | arXiv 1612.03969 |
| QRN 2r, best of 10 runs | gradients | 7 | 9.9 | arXiv 1606.04582 |
| GA reader with entity links made by a rule | gradients | 5 | 8.7 | arXiv 1703.02620 |
| Transformer, best of 10 runs | gradients | 5 | 21.8 | arXiv 1807.03819 |
| Universal Transformer, dynamic halting, best seed for each task | gradients | 3 | 4.55 | arXiv 1807.03819 |

All rows learn from the answers only. With stronger lessons, fewer tasks fail: DMN with labels of the supporting sentences, 2 (arXiv 1506.07285); GGT-NN with the state graph of the generator after each sentence, 1 (Johnson, ICLR 2017); rules learned by inductive logic programming with a given parser and mode declarations written for each task, 1 (arXiv 1802.07966).

The bAbI paper says that for a learner from the answers only (the weakly supervised case) with 1,000 examples or fewer, "there is no known general (i.e. non-hand engineered) method that solves the tasks" (arXiv 1502.05698).

## SCAN

Data: the four splits of marcov/scan_*_promptsource on the hub. Lessons: the first 1,000 commands of each train split. Read: the whole test split. A case is right only when the whole action sequence is right.

| System | Simple | Length | Add jump | Add turn left | Source |
|---|---|---|---|---|---|
| ILM, 1,000 lessons | 100.0 (4,182) | 100.0 (3,920) | 100.0 (7,706) | 100.0 (1,208) | this repository |
| LSTM, the best of the first paper | 99.7 | 13.8 | 0.08 | 90.3 | arXiv 1711.00350 |
| Transformer | - | 0.0 | 1.0 | 99.6 | arXiv 2007.08970 |
| Universal Transformer | - | 0.0 | 0.3 | 99.4 | arXiv 2007.08970 |
| T5-11B, pre-trained, tuned on the split | - | 2.0 | 98.3 | 87.9 | arXiv 2007.08970 |
| LANE (made for SCAN) | - | 100.0 | 100.0 | - | arXiv 2007.08970 |

The published models learn from the whole train split (14,670 to 21,890 commands). ILM reads SCAN with the parts tool, which is not made for SCAN: the same tool reads other sets of the board.

## ProofWriter, depth 5

Data: the open form (tasksource/proofwriter, depth-5) and the closed form (rlhf-and-friends/proofwriter-source, CWA depth-5) on the hub. One kind for each depth of proof, 0 to 5. Lessons: the first 600 train statements of each depth (600 to 604, because the cut keeps a story whole), with answers only; for the closed form at depth 5, the lessons also have the formal forms that the set gives for their sentences. Read: every test statement of each depth.

ILM: open form 20,020 of 20,020, closed form 20,043 of 20,043 (100.0 at every depth). At depth 5 of the closed form, the question alone gives 96.2%, so `ilm measure` does not mark that kind LEARNED. The board passes it at the bar alone, because the set marks it as a kind where no model can be 5 points above the question alone.

| System | Training | Answer accuracy | Source |
|---|---|---|---|
| ILM | about 600 statements for each depth, answers only (closed form, depth 5: also formal forms) | 100.0 (both forms) | this repository |
| ProofWriter (T5-11B), all at once, closed form | about 70,000 questions, with proofs | 99.6 | arXiv 2012.13048 |
| ProofWriter (T5-11B), all at once, open form | about 70,000 questions, with proofs | 99.7 | arXiv 2012.13048 |
| GPT-4, open form, 600 cases: prompt / chain of thought / Logic-LM | prompt | 52.7 / 68.1 / 79.7 | arXiv 2305.12295 |

## LoNLI

Data: tasksource/lonli on the hub: 363 templates of sentence pairs in 17 groups of logic (boolean, comparative, negation and others), each pair with its label of inference (two or three answers, as the group gives). Lessons: 1,000 pairs of each group, with answers only. Read: up to 1,000 pairs of the sealed part of each group, read with frozen code. The read parts of the board served during development; the sealed part did not.

ILM: 14,222 of 14,986 right (94.9%) over the 15 groups with unseen cases. Ten of the 16 groups pass the learned bar.

| Group | Right | Share | Learned |
|---|---|---|---|
| boolean | 991 of 1,000 | 99.1 | yes |
| causal | 1,000 of 1,000 | 100.0 | yes |
| comparative | 581 of 1,000 | 58.1 | no |
| conditional | 1,000 of 1,000 | 100.0 | yes |
| coreference | 992 of 1,000 | 99.2 | yes |
| implicature | 992 of 995 | 99.7 | yes |
| lexical | 880 of 997 | 88.3 | no |
| negation | 994 of 994 | 100.0 | yes |
| numerical | 898 of 1,000 | 89.8 | no |
| presupposition | 1,000 of 1,000 | 100.0 | no: the question alone gives 98.5 |
| quantifier | 966 of 1,000 | 96.6 | yes |
| relational | 1,000 of 1,000 | 100.0 | no: the question alone gives 100.0 |
| spatial | 973 of 1,000 | 97.3 | yes |
| syntactic | 1,000 of 1,000 | 100.0 | yes |
| temporal | 955 of 1,000 | 95.5 | yes |

Not in the total: taxonomic, because every case of its sealed part is a copy of a lesson, and world, which has no sealed part. On the board, temporal passes its share but not its control of changed cases (93.1%; the line is 95%), so the board does not count it.

| System | Training | Mean accuracy | Source |
|---|---|---|---|
| ILM | 1,000 pairs of each group, counting | 94.9 (15 groups) | this repository |
| BERT-base / RoBERTa-large / DeBERTa-large | pre-trained, tuned on MNLI, no LoNLI | 59.4 / 68.2 / 69.9 | arXiv 2112.02333, table 4 |
| RoBERTa-large tuned on MNLI, SNLI, FEVER and ANLI | pre-trained, no LoNLI | 71.1 | arXiv 2112.02333, table 4 |
| RoBERTa-large tuned on 2% of LoNLI | pre-trained, about 7,000 pairs of LoNLI | near 100 (figure only) | arXiv 2112.02333, section 4.4 |

The published means are over the whole set and the 17 groups; ILM's is over the sealed cases of 15 groups.

## CLUTRR

Data: CLUTRR/v1 on the hub, the official split gen_train234_test2to10. Lessons: all 12,064 training stories of 2, 3 and 4 steps. The set gives the label of each step of a training story, and ILM learns from these labels (the frames tool). Read: the 1,048 test stories of 2 to 10 steps. The stories are written by people.

ILM: 758 of 1,048 right (72.3%).

| Steps | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|
| Stories | 38 | 107 | 77 | 185 | 105 | 155 | 135 | 124 | 122 |
| ILM, % right | 68.4 | 80.4 | 76.6 | 76.8 | 74.3 | 71.0 | 63.7 | 70.2 | 68.9 |
| The question alone, % right | 97.4 | 95.3 | 76.6 | 12.4 | 12.4 | 5.8 | 5.9 | 9.7 | 8.2 |

The control "the question alone" sees only the question, which names two people. At 2 to 4 steps, it gives most answers right: there, the names of a test story predict the relation from the training stories. At 5 to 10 steps the names do not help, and the control falls to 6 to 12%. ILM reads the story and not the names.

| System | Training | Accuracy | Source |
|---|---|---|---|
| ILM | 12,064 stories with step labels, counting | 72.3 | this repository |
| BERT / BERT-LSTM, 2 and 3 steps | gradients, 10,000 stories | 37 / 67 | arXiv 1908.06177 |
| DSR-LM (RoBERTa and a rule engine) | gradients | 61.0 | arXiv 2305.03742 |
| GPT-4, 5 examples | prompt | 48.7 | arXiv 2310.07064 |
| GPT-4o, graph and a symbolic solver | prompt | 67.7 | arXiv 2412.17963 |
| code-davinci-002, Faithful CoT with votes | prompt | 71.9 | arXiv 2301.13379 |

The test cases of these papers are not the same in all rows. The models of the first two published rows learn from the story and the answer only.

## LAMA T-REx

Data: facebook/lama on the hub, the T-REx part: 34,039 facts of 41 relations, each with an object of one word. The first fifth of the facts of each relation is the lessons (6,791 facts). The other 27,248 facts are asked. The text is the English Wikipedia of March 2022 (legacy-datasets/wikipedia, 20220301.en). The text store holds 175,367 of its articles (1,399 million signs). The reader also uses a table of word classes, which `ilm classes` counts from the same Wikipedia copy (see [HOW_IT_WORKS.md](HOW_IT_WORKS.md)).

ILM: 17,439 of 27,248 right (64.0%, precision at one). From the text alone: 16,817 (61.7%); the guess from the subject's name gives 622 more. Each answer from the text names where in the text it was read.

Reference points on the same facts and text, with no model: one answer for each relation 20.0%; the first word of the article that is an object of the relation 50.8%; the object stands somewhere in the article 82.6%.

| System | Learns from | Precision at one | Source |
|---|---|---|---|
| ILM | 6,791 lesson facts and the text | 64.0 | this repository |
| BERT-large, written prompt | pre-training | 32.3 | arXiv 1909.01066 |
| BERT-large with retrieved Wikipedia text | pre-training | 43.1 | arXiv 2005.04611 |
| BERT-large with the sentences that state the fact | pre-training | 62.6 | arXiv 2005.04611 |
| OptiPrompt, BERT-base | trained prompts | 48.6 | arXiv 2104.05240 |
| P-tuning, BERT-large | trained prompts | 50.6 | arXiv 2103.10385 |
| P-tuning, MegatronLM (11B), on a subset of 29,000 facts | trained prompts | 64.2 | arXiv 2103.10385 |

The published numbers read all 34,039 facts. ILM reads the 27,248 facts that are not lessons. ILM reads the encyclopedia when it answers, as the retrieval rows do. The text store and the table of word classes are made once from the Wikipedia copy, in 16 s and 41 s.

## SVAMP, standard setting

Data: the four parts of MAWPS (AddSub, SingleOp, SingleEq, MultiArith, from allenai/lila) and ASDiv-A (MU-NLPC/Calc-asdiv_a) are the lessons: 3,061 problems, with answers only. Read: the 300 problems of the test file of ChilleD/SVAMP. The 700 problems of its train file are the development part (44.9%).

ILM: 115 of 300 right (38.3%).

| System | Training | SVAMP | Source |
|---|---|---|---|
| ILM | 3,061 problems, answers only | 38.3 (300 problems) | this repository |
| Seq2Seq / GTS / Graph2Tree, from scratch | gradients, equations as labels | 24.2 / 30.8 / 36.5 | arXiv 2103.07191 |
| The same with RoBERTa | gradients, equations as labels | 40.3 / 41.0 / 43.8 | arXiv 2103.07191 |
| GPT-3 (text-davinci-002), chain of thought | prompt | 68.9 | arXiv 2201.11903 |
| PaLM 540B, chain of thought | prompt | 79.0 | arXiv 2201.11903 |

The published rows read all 1,000 SVAMP problems.

## E2E NLG

Data: tuetschek/e2e_nlg on the hub. Lessons: all 42,061 train rows (a list of facts and a text by a person). Read: the 630 test inputs, with 4,693 references. Scores: BLEU and NIST of the official E2E metrics (tuetschek/e2e-metrics on GitHub, commit 1b9c4d87, its Python implementation).

| System | BLEU | NIST | Source |
|---|---|---|---|
| ILM | 65.37 | 8.29 | this repository |
| TGen (the baseline of the challenge) | 65.93 | 8.61 | arXiv 1810.01170 |
| Slug (the best BLEU of the challenge) | 66.19 | 8.61 | arXiv 1810.01170 |
| The 21 systems of the challenge | 42.0 to 66.2 | | arXiv 1810.01170 |
| GPT-2 medium, fine-tuned / LoRA | 68.2 / 70.4 | 8.62 / 8.85 | arXiv 2106.09685 |
| GPT-2 large, fine-tuned / LoRA | 68.5 / 70.4 | 8.78 / 8.89 | arXiv 2106.09685 |

## The board

The board measures all kinds of 25 public sets on their read parts, with the same program: 240 kinds in 112 tasks. A set gives only its files, its sizes, who wrote its text and its marks. ILM learns 155 kinds and 61 tasks in 17 sets, by the bar of [HOW_IT_WORKS.md](HOW_IT_WORKS.md). 42 of the 61 tasks are learned from 200 lessons. The whole board takes 19 s.

## Notes on the numbers

- During development, ILM used the training and validation data of each set. The board also reads a fixed part of the test split of some sets (the last cases of each kind). Of that part, only the share of right answers was looked at, never a wrong case.
- Reads of test data that gave or changed a number: the test splits of bAbI, SCAN, ProofWriter and SVAMP and the sealed part of LoNLI once, the E2E test set two times, the CLUTRR test split three times and the LAMA probe four times. Between two such reads, the reader, the writer, the joins of the link tool and the table of word classes changed on development data only. The program of each release reads all test data again, for the times and as a check, and gets the same counts. The tables give the last read.
- On one computer, every count is the same in every run and for every number of threads. Another operating system can give a logarithm that differs in its last bit, and in a rare close decision this could move a count; the CI checks the bAbI counts on Linux, Windows and macOS.
- The board does not count bAbI tasks 3 and 18 as learned: in the control of changed cases, a sure rule keeps the old answer when a supporting sentence is taken out.
