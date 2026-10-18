# Results

ILM reads each benchmark below with frozen code on its test data; each section names the part that is read. The published numbers beside it come from the papers named in each table (by arXiv number).

All runs use one desktop computer: an Intel Core i9-12900K with 24 threads, and no GPU. A time is the wall-clock time of the whole run with the release build: the choice of the ways of the tool, the controls, the learning from all lessons, and the read of the test data. Each time is the best of three or more runs in turn of the commands of [REPRODUCE.md](REPRODUCE.md), in a fresh copy of this repository with the data fetched from the public sources. At 24 threads each benchmark needs 1 GB of memory or less, and the board 3.1 GB. On one thread (`ILM_LANES=1`), the whole board of 112 tasks takes 210 s with a peak of 2.6 GB, and bAbI 27.6 s.

## Highlights

| Benchmark | Lessons | ILM | Published, for comparison | Time |
|---|---|---|---|---|
| bAbI, 20 tasks | 1,000 for each task, answers only | 19 of 20 tasks pass; 18,903 of 19,399 right (97.44%) | the best published learner from the answers only that we found fails 3 tasks (Universal Transformer) | 14.0 s |
| SCAN, 4 splits | the first 1,000 of each train split | 17,016 of 17,016 right (100%) | Transformer: length 0.0, add jump 1.0; T5-11B: 2.0 and 98.3 | 1.5 s |
| ProofWriter, depth 5, open and closed form | about 600 for each depth of proof, answers only | 39,870 of 40,063 right (99.52%) | ProofWriter (T5-11B): 99.6 and 99.7 | 0.4 s |
| LoNLI, 17 groups of logic in NLI | 1,000 to 1,013 for each group, answers only | 14,714 of 14,986 right (98.18%) on its sealed part | tuned on MNLI, not on LoNLI: 59.4 to 71.1; RoBERTa-large tuned on 2% of LoNLI: near 100 | 2.5 s |
| CLUTRR, graph form, 2 to 10 relations | 1,002 graphs of 2, 3 and 4 relations, answers only | 1,036 of 1,045 right (99.14%) | at 10 relations: Transformer 70, NCRL 97, R5 98 | 1.4 s |
| Sentence transformations, 4 parts | 1,000 for each part, answers only | 80,000 of 80,000 whole sentences right (100%) | models from scratch: near 0% whole sentences on the generalization sets | 1.6 s |
| PCFG SET, 3 splits | the first 1,000 of each train file | 31,229 of 31,229 right (100%) | Transformer: plain split 92, systematicity 72, productivity 50 | 8.4 s |
| ToM-QA, 12 kinds | 600 stories for each kind, answers only | 1,950 of 1,950 right (100%) | the best model of each form: 88.63 to 100.00 | 2.1 s |

The papers give some of these as fractions; they are shown here as percentages.

## How ILM stands against transformers

Each published number names its paper (by arXiv number or venue). **DIFF** marks a place where the setting of ILM differs from the paper's. The papers give some of these as fractions; they are shown here as percentages.

### Level or ahead

These rows are the final reads of the card. The Transformer column gives the plain Transformer, trained from scratch on the task, where a paper gives one.

| Benchmark | ILM | Transformer, from scratch | Other published learners | DIFF |
|---|---|---|---|---|
| bAbI, 1,000 examples | 1 task fails; mean error 2.48% | 5 tasks fail; mean error 21.8% (best of 10 runs; arXiv 1807.03819) | Universal Transformer, best seed for each task: 3 fail, 4.55% (arXiv 1807.03819) | 601 test questions with the story of a lesson are not read |
| SCAN length and add jump | 100.0 and 100.0 | 0.0 and 1.0 (arXiv 2007.08970) | T5-11B, pre-trained: 2.0 and 98.3; LANE, made for SCAN: 100.0 and 100.0 (arXiv 2007.08970) | ILM learns from 1,000 commands of each train split, the published models from all of it (14,670 to 16,990) |
| PCFG SET, systematicity and productivity | 100.0 and 100.0 | 72 and 50 (arXiv 1908.08351) | LSTMS2S: 53 and 30; ConvS2S: 56 and 31 (arXiv 1908.08351) | 1,000 pairs against 81,000 to 85,000 |
| CLUTRR, graph form, 10 relations | 100 (122 of 122) | 70 (arXiv 2007.06477, 2303.03581) | Edge Transformer 92; CTP 95; NCRL 97; R5 98 (arXiv 2112.00578, 2007.06477, 2303.03581, 2205.06454) | 1,002 lessons against about 15,000 graphs; ILM reads the test split of the hub copy; during development the board read 527 of the 1,045 test graphs (shares only, never a wrong case) |
| Sentence transformations, generalization sets | 100.0 whole sentences | near 0% whole sentences; the right word moved: 4 to 7 (arXiv 2203.09397) | T5 and mT5, pre-trained: near 100% whole sentences in a figure, but for the German passive (arXiv 2203.09397) | 1,000 rows against 100,000 |
| ToM-QA | 100.0 on each kind | - | the best model of each form, from scratch (EntNet, RelNet, Multiple Observer): 88.63 to 100.00 (arXiv 1808.09352) | the test splits of the hub copies, which the board read during development; 600 stories against 10,000 for each pair |
| ProofWriter, depth 5 | 99.52 (level, not ahead) | - | ProofWriter (T5-11B), pre-trained, with proofs: 99.6 and 99.7 (arXiv 2012.13048) | 600 statements against about 70,000; rules of the library read the told rules |
| LoNLI, sealed part | 98.18 (ahead of the models with no LoNLI lesson; not of RoBERTa-large tuned on 2% of LoNLI) | - | BERT-base, RoBERTa-large and DeBERTa-large, pre-trained, tuned on other NLI sets: 59.4 to 71.1 (arXiv 2112.02333, table 4); RoBERTa-large tuned on 2% of LoNLI: near 100 (arXiv 2112.02333) | the published means are over all 17 groups |

### More results from the board

These rows are the board's read parts: the last 400 cases of each kind, or fewer (for ATIS and SNIPS up to 3,000: ATIS reads 890 of its 893 test utterances), and not final reads of whole test splits. For each task, ILM gives its lowest kind and the number of lessons that it learns from. "Not learned" is a kind below the bar; "no result" is a kind that no way of the tool reads.

| Task | ILM, lowest kind (lessons) | Transformer, from scratch | Best published (pre-trained or other) | DIFF |
|---|---|---|---|---|
| ToMi, statement form: where the thing is, where it was, first belief | 100.0, 100.0, 98.2 (600) | - | EntNet 90.6, RelNet 86.0, over all questions (Le and others 2019, ACL anthology D19-1598) | a statement judged true or false (tasksource/tomi-nli), for each question type; ILM does not learn the second belief |
| RuleTaker, closed form, depths 0 to 5 | 97.5 (600) | - | RoBERTa-large, pre-trained: 99 on its own test part; trained to depth 3: 97.6 on depth 5 (arXiv 2002.05867) | 600 against about 70,000; rules of the library read the told rules |
| ParaRules, rules in the words of people | 88.0, not learned (600) | - | ProofWriter (T5-11B): 99.9 to 91.5 by depth (arXiv 2012.13048); RoBERTa trained with ParaRules: 98.8 (arXiv 2002.05867) | 600 against 28,010; rules of the library read the told rules |
| PrOntoQA, 5 hops | 100.0, not counted: a regularity passes it (600) | - | GPT-2, pre-trained and tuned, answers only: 93.8; Coconut: 99.8 (arXiv 2412.06769) | 600 against 9,000 |
| ProsQA | 100.0 (600) | - | GPT-2, pre-trained and tuned, answers only: 76.7; iCoT, with written chains: 98.2 (arXiv 2412.06769) | 600 against 17,886; rules of the library read the told rules |
| NLGraph: connectivity, cycle, flow | 100.0, 100.0, 100.0 (600) | - | text-davinci-003: connectivity 86.82 (chain of thought with self-consistency), cycle 70.33 (4 examples) (arXiv 2305.10037) | a prompt against 600 lessons; rules of the library |
| MindGames: forehead, forehead with mirror, explicit, internal | 100.0, 100.0 (not learned: the question alone gives 98.5), 98.8, 99.0 (600) | - | deberta-small, pre-trained, all four setups together: 83 on the validation part (arXiv 2305.03353) | each setup apart against all setups together; rules of the library |
| StepGame, 1 to 10 steps | no result (600) | Universal Transformer: 45.11 at 1 step, 11.74 at 10 steps (arXiv 2204.08292) | TP-MANN: 85.77 and 21.46 (arXiv 2204.08292); SynSup, pre-trained: 98.6 and 45.7 (arXiv 2307.07696) | about 10.7% wrong labels in a hand count of 1,000 cases |
| SpartQA, yes or no | no result (600) | - | XLNet, pre-trained: 79.91 (arXiv 2104.05832) | ILM's kind has three answers (yes, no, not told) |
| FLD | no result (600) | - | T5, pre-trained, tuned with proofs: 91.6 (arXiv 2308.07336) | |
| BoardgameQA | no result (600) | - | figures only (arXiv 2306.07934) | 1,000 training examples against ILM's 600 |
| CLUTRR, stories by people | no result (1,000) | - | BERT-LSTM: 67 at 2 and 3 relations (arXiv 1908.06177); DSR-LM: 60.98 (arXiv 2305.03742) | |
| NL-RX: made descriptions, descriptions by people | no result (1,000) | - | Deep-RegEx: 88.7 and 58.2 (arXiv 1608.03000) | the paper compares the languages of two expressions, ILM compares strings |
| ATIS: intent, slots | 85.7, not learned; no result (1,000) | - | Slot-Gated, from scratch: 94.1, slots F1 95.2; Joint BERT, pre-trained: 97.5, slots F1 96.1 (arXiv 1902.10909) | the slot kind asks the slot words, not F1 over tags |
| SNIPS: intent, slots | 88.5, not learned; no result (1,000) | - | Slot-Gated: 97.0, slots F1 88.8; Joint BERT: 98.6, slots F1 97.0 (arXiv 1902.10909) | as ATIS |
| Word problems: MAWPS (4 parts), ASDiv-A, ASDiv, SVAMP | 33.3 to 67.2, 7.8, 4.2, no result; not learned (600) | - | Graph2Tree, from scratch: 83.7, 77.4, 36.5; with RoBERTa: 88.7, 82.2, 43.8 (arXiv 2103.07191); PaLM 540B with chain of thought: MAWPS 93.3, SVAMP 79.0 (arXiv 2201.11903) | ILM learns from the answers only, the published models from equations |

BIG-bench tasks of the board. The published rows are large pre-trained language models with 3 solved examples in the prompt, on the test cases of each task, 250 for most and 146 for penguins_in_a_table (BIG-Bench Hard, arXiv 2210.09261, table 3), some tasks as one part or as the mean of three sizes. No paper gives a Transformer trained from scratch on these tasks.

| Task | ILM, lowest kind (600 lessons) | Best published model, 3 examples | Human raters, mean |
|---|---|---|---|
| tracking_shuffled_objects | 100.0 | Codex with chain of thought: 84.5 | 64.7 |
| temporal_sequences | 100.0 | Codex with chain of thought: 96.8 | 90.8 |
| word_sorting | 99.8 | Codex: 50.4 | 62.6 |
| boolean_expressions | 100.0 | Codex with chain of thought: 92.8 | 79.4 |
| multistep_arithmetic | 100.0 | InstructGPT with chain of thought: 53.2 | 9.7 |
| hyperbaton | 100.0 | PaLM 540B with chain of thought: 90.4 | 74.7 |
| logical_deduction | 100.0 | Codex with chain of thought: 60.4; GPT-4 with Logic-LM: 87.63 (arXiv 2305.12295) | 40.3 |
| navigate | 100.0 | Codex with chain of thought: 96.4 | 81.9 |
| object_counting | 100.0 | Codex with chain of thought: 93.2 | 86.1 |
| web_of_lies | 100.0 | PaLM 540B with chain of thought: 100 | 81.3 |
| dyck_languages | 100.0 | Codex with chain of thought: 56.8 | 47.8 |
| reasoning_about_colored_objects | 100.0 | Codex with chain of thought: 91.6 | 75.4 |
| date_understanding | 93.8, not learned | Codex with chain of thought: 87.2 | 76.8 |
| formal_fallacies | 98.0 | InstructGPT with chain of thought: 58.4 | - |
| geometric_shapes | 100.0 | InstructGPT with chain of thought: 56.0 | 54.0 |
| penguins_in_a_table | no result (too few cases) | InstructGPT with chain of thought: 81.5 | 78.0 |
| arithmetic, 1 to 5 digits | 100.0 on 15 of 15 kinds | GPT-4 at 5 digits: 0.0 (multiplication) to 94.1 (addition); Goat-7B, tuned on about 1 million pairs: 96.5 to 98.4 (arXiv 2305.14201) | - |

### Where transformers and other published models lead

On text that people wrote, pre-trained transformers lead: the stories of CLUTRR, the word problems, NL-RX, the rules in the words of people of ParaRules, and the intents and slots of ATIS and SNIPS. Published models also lead on StepGame, SpartQA, FLD and BoardgameQA, where ILM gets no result. The rows are in the table above.

### What ILM does that a transformer does not

- **A proof for each proved answer.** `ilm prove` prints the proof tree of an answer in English, back to the told sentences (the example in the README; tests `an_answer_is_explained_by_a_proof_from_the_told_sentences` and `a_proof_of_the_library_answers_from_told_sentences_alone`).
- **The source of each rule and fact.** Each rule in a proof is marked as given, a line of the library that a person wrote, or as discovered on a kind: the example in the README shows "[discovered on babi02]" from a model of bAbI task 2, and the test `a_proof_marks_each_rule_and_learned_fact_as_discovered_or_given` checks the marks of a proof against the lists of learned rules and facts that it gets. Each fact of `data/explain/learned/capitals.txt` names the files and lines of the sentences that gave it, and each rule of `data/store` names the kind and the number of lessons that it came from (test `the_store_keeps_the_lines_of_each_kind_whole_and_reads_back_what_it_wrote`).
- **A mark for how an answer was found.** Each answer is marked proof, default, guess or none (tests `a_story_with_a_sentence_of_a_frame_that_no_lesson_showed_is_answered_as_a_guess` and `an_answer_that_holds_otherwise_is_known_as_a_default`). On the board, 98.8% of the answers with a proof mark are right, and 74.5% of those with a guess mark. A proof mark is not a promise: in the final read of bAbI task 14, 73.0% of the answers with a proof mark are right.
- **Learning writes no file of the library.** The lines that a kind learns belong to its model (test `a_learned_model_lists_its_learned_lines_and_the_rules_that_a_proof_rests_on`). In the code, only two commands write files of the library or of the store (`data/store`): `ilm facts` with `out=`, and `ilm store-lines`; no test checks the files after a run. A learned text that a person changed is refused (test `a_learned_text_is_refused_when_a_person_edited_it_or_a_hand_text_has_its_name`), and a run with fewer lessons than its set gives is offered no stored rule (test `a_run_of_fewer_lessons_than_its_set_gives_is_offered_no_store`).
- **Few lessons.** 73 of the 79 tasks that the board learns are also learned from 200 lessons of each kind. On SCAN, PCFG SET and the sentence transformations, ILM learns from 1,000 lessons, and the published models from 14,670 to 100,000.
- **The same result on every run.** In the timed runs, on 1 thread and on 24 threads, all 8 benchmarks and the board give the same counts (11 runs of each), on one CPU and with no GPU.

## bAbI, 20 tasks, 1,000 examples

Data: Muennighoff/babi on the Hugging Face hub. Lessons: the 1,000 train and validation questions of each task (900 and 100), with answers only and no supporting facts. Read: the 1,000 test questions of each task. A test question whose story is also the story of a lesson is not read, because the cut keeps a story on one side: 601 questions (tasks 4, 17 and 20 lose 440, 32 and 119; tasks 1, 6, 7, 10 and 13 lose 2, 2, 1, 3 and 2).

ILM: 18,903 of 19,399 right (97.44%). 19 of the 20 tasks pass the bar of the bAbI paper (an error of 5% or less). The mean error is 2.48%. Wrong answers: task 5: 2, task 7: 2, task 8: 6, task 14: 481, task 16: 5; all other tasks: 0.

Task 14 (time) is the task that fails: 519 of 1,000 right (51.9%). When the kind learns from 943 or more of its 1,000 lessons, the learner keeps a reading of the told rules that gets exactly half of the last lessons right, and the search does not try the facts that order the time words. That reading takes "before" in the order of the story, and the stories of task 14 tell the times out of order. On development data (600 lessons, the read part of the board) the kind reads 100.0, and on stories typed in the style of the task the same model reads 35 of 100. In the final read, 71.1% of its answers carry a proof mark and 73.0% of those are right, and 28.9% of its questions get no answer.

| System | Training | Tasks failed (of 20) | Mean error % | Source |
|---|---|---|---|---|
| ILM | 1,000 examples for each task, answers only | 1 | 2.48 | this repository |
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
| ILM, 1,000 lessons | 100.0 (4,182 of 4,182) | 100.0 (3,920 of 3,920) | 100.0 (7,706 of 7,706) | 100.0 (1,208 of 1,208) | this repository |
| LSTM, the best of the first paper | 99.7 | 13.8 | 0.08 | 90.0 | arXiv 1711.00350 |
| Transformer | - | 0.0 | 1.0 | 99.6 | arXiv 2007.08970 |
| Universal Transformer | - | 0.0 | 0.3 | 99.4 | arXiv 2007.08970 |
| T5-11B, pre-trained, tuned on the split | - | 2.0 | 98.3 | 87.9 | arXiv 2007.08970 |
| LANE (made for SCAN) | - | 100.0 | 100.0 | - | arXiv 2007.08970 |

The published models learn from the whole train split (14,670 to 21,890 commands). ILM learns SCAN by its ways of joining: the meaning of each word, and the way in which each joining word puts the parts of a command together, learned from the lessons and written as lines of the library (`ilm joining` prints them). No text of the library is written for SCAN.

## ProofWriter, depth 5

Data: the open form (tasksource/proofwriter, depth-5) and the closed form (rlhf-and-friends/proofwriter-source, CWA depth-5) on the hub. One kind for each depth of proof, 0 to 5. Lessons: the first 600 train statements of each depth (600 to 604, because the cut keeps a story whole), with answers only. Read: every test statement of each depth: open form 20,020, closed form 20,043.

ILM: open form 19,887 of 20,020, closed form 19,983 of 20,043. By depth: open form 100.0, 99.8, 99.5, 99.3, 98.9 and 96.4; closed form 100.0, 99.4, 99.6, 99.6, 99.6 and 99.8 (depths 0 to 5). At depth 5 of the closed form, the question alone gives 96.2%, so `ilm measure` does not mark that kind LEARNED. The board passes it at the bar alone, because the set marks it as a kind where no model can be 5 points above the question alone.

Rules of the library read the told rules and facts of each story. The board marks ProofWriter as written: the answers rest on lines that a person wrote, so this number measures the engine on stories of up to 5 steps, not rules that ILM learned.

| System | Training | Answer accuracy | Source |
|---|---|---|---|
| ILM | about 600 statements for each depth, answers only; rules of the library | 99.52 (open form 99.34, closed form 99.70) | this repository |
| ProofWriter (T5-11B), all at once, closed form | about 70,000 questions, with proofs | 99.6 | arXiv 2012.13048 |
| ProofWriter (T5-11B), all at once, open form | about 70,000 questions, with proofs | 99.7 | arXiv 2012.13048 |
| GPT-4, open form, 600 cases: prompt / chain of thought / Logic-LM | prompt | 52.7 / 68.1 / 79.7 | arXiv 2305.12295 |

## LoNLI

Data: tasksource/lonli on the hub: 363 templates of sentence pairs in 17 groups of logic (boolean, comparative, negation and others), each pair with its label of inference (two or three answers, as the group gives). The set has no test split. Lessons: 1,000 to 1,013 pairs of each group (the cut keeps the pairs of one premise together), with answers only. Read: the sealed part of each group, read with frozen code: up to 1,000 pairs that lie before the last 1,000 pairs outside the lessons (which is as far as a run of development reads), without each pair whose sentences another group learned from or read. The read parts of the board served during development; the sealed part did not.

ILM: 14,714 of 14,986 right (98.18%) over the 15 groups with unseen cases. 12 of the 16 groups pass the learned bar. Lexical (83.4%) and relational (90.5%) are below the bar of 95%; presupposition reads 99.4%, but the question alone gives 100.0%, so it is not learned; taxonomic reads 93.7% (937 of 1,000) on copies of its lessons. The antonyms of the library include direct antonyms of WordNet 3.0, the source from which LoNLI made its lexical group (arXiv 2107.07229).

| Group | Read | Right |
|---|---|---|
| boolean | 1,000 | 1,000 |
| causal | 1,000 | 1,000 |
| comparative | 1,000 | 999 |
| conditional | 1,000 | 1,000 |
| coreference | 1,000 | 999 |
| implicature | 995 | 995 |
| lexical | 997 | 831 |
| negation | 994 | 994 |
| numerical | 1,000 | 1,000 |
| presupposition | 1,000 | 994 |
| quantifier | 1,000 | 997 |
| relational | 1,000 | 905 |
| spatial | 1,000 | 1,000 |
| syntactic | 1,000 | 1,000 |
| temporal | 1,000 | 1,000 |

Not in the total: taxonomic, because every case of its sealed part is a copy of a lesson, and world, which has no sealed part (its 2,000 pairs are all in the lessons and the read part).

| System | Training | Mean accuracy | Source |
|---|---|---|---|
| ILM | 1,000 to 1,013 pairs of each group, counting | 98.18 (15 groups) | this repository |
| BERT-base / RoBERTa-large / DeBERTa-large | pre-trained, tuned on MNLI, no LoNLI | 59.4 / 68.2 / 69.9 | arXiv 2112.02333, table 4 |
| RoBERTa-large tuned on MNLI, SNLI, FEVER and ANLI | pre-trained, no LoNLI | 71.1 | arXiv 2112.02333, table 4 |
| RoBERTa-large tuned on 2% of LoNLI | pre-trained, about 7,000 pairs of LoNLI, if the 2% is of the whole set (363,000 pairs) | near 100 (figure only) | arXiv 2112.02333, section 4.4 |

The published means are over the whole set and the 17 groups; ILM's is over the sealed cases of 15 groups.

## CLUTRR, graph form

Data: CLUTRR/v1 on the hub, the configuration gen_train234_test2to10, in the graph form: each story is a list of told relations, each a name, a word of a relation and a name, and the question names two people whose relation is asked. Lessons: the first 1,002 train graphs, of 2, 3 and 4 relations, with answers only. Read: the test split, graphs of 2 to 10 relations (1,045 of its 1,048 graphs; a test graph whose told relations are those of a lesson is not read).

ILM: 1,036 of 1,045 right (99.14%). By number of relations: 2 relations 35 of 35, 3 relations 102 of 107, 4 relations 73 of 77, and 5 to 10 relations all right (185, 105, 155, 135, 124 and 122). The wrong answers are at 3 and 4 relations, the lengths of the lessons; 4 relations (94.8%) is the one kind of the nine that is below the bar of 95%.

The rules that join the relations are discovered from the lessons; no text of the library holds them. The meanings of the words of relations ("Son means a male child.") are facts that a person wrote in the knowledge of the library, so the board gives the facts of this task as given.

| System | Pre-trained | Trained on | 5 relations | 10 relations | Source |
|---|---|---|---|---|---|
| ILM | no | 2, 3, 4 (1,002 graphs) | 100 (185 of 185) | 100 (122 of 122) | this repository |
| Multi-head attention (Transformer) | no | 2, 3, 4 | 88 | 70 | arXiv 2007.06477; arXiv 2303.03581 |
| LSTM | no | 2, 3, 4 | 98 to 100 | 78 to 92 | arXiv 2007.06477 |
| Edge Transformer | no | 2, 3, 4 | - | 92 | arXiv 2112.00578, table 5 |
| CTP (a neural theorem prover) | no | 2, 3, 4 | 99 | 95 | arXiv 2007.06477 |
| R5 (rule discovery) | no | 2, 3, 4 | 99 | 98 | arXiv 2205.06454, table 3 |
| NCRL | no | 2, 3, 4 | 100 | 97 | arXiv 2303.03581, table 3 |

The published systems learn from about 15,000 graphs of CLUTRR (the Edge Transformer also from 35,000). During development, the board's read part held 527 of these 1,045 test graphs (shares only; see the notes on the numbers).

## Sentence transformations

Data: amueller/syntactic_transformations on the hub (Mueller and others, arXiv 2203.09397): four parts, a sentence into a question and into the passive, in English and in German. Each part has a train file of 100,000 rows, a test file of 10,000 rows with sentences of the builds of the train file, and a generalization file of 10,000 rows with a build that no train sentence of the change has (a relative clause or a phrase on the subject). Lessons: 1,000 rows spread evenly over the train file of each part. Read: the whole test file and the whole generalization file of each part. A case is right only when the whole sentence is right.

| Part | Test | Generalization |
|---|---|---|
| question, English | 10,000 of 10,000 | 10,000 of 10,000 |
| question, German | 10,000 of 10,000 | 10,000 of 10,000 |
| passive, English | 10,000 of 10,000 | 10,000 of 10,000 |
| passive, German | 10,000 of 10,000 | 10,000 of 10,000 |

ILM learns the rewrite of each part from the lessons: the words of the answer are found in the sentence of the question, as runs that move together or as other forms of its words that the lessons show. The rewrites are written as lines of the library; no text of the library is written for these parts.

Published, from the same paper: the models that learn from scratch (LSTM, Transformer) get 93 to 98% of the test sentences right, and on the generalization sets they "consistently stay at or near 0% sequence accuracy". On the looser measures of the paper (the right auxiliary at the front of a question, the right object noun at the front of a passive), they get 4 to 44%. The pre-trained models T5, mT5, BART and mBART get 59 to 100% on the looser measures, and the paper gives their whole-sentence accuracy on the generalization sets in a figure only. All published models learn from the 100,000 train rows.

## PCFG SET

Data: the PCFG SET data of Hupkes and others (arXiv 1908.08351), from the GitHub repository i-machine-think/am-i-compositional: functions on rows of words in prefix notation, in three splits. Lessons: the first 1,000 pairs of each train file. Read: the whole test file of each split. A case is right only when the whole output is right.

| System | Plain split (task) | Systematicity (new pairs of functions) | Productivity (longer inputs) | Source |
|---|---|---|---|---|
| ILM, 1,000 lessons | 100 (9,721 of 9,721) | 100 (10,175 of 10,175) | 100 (11,333 of 11,333) | this repository |
| LSTMS2S | 79 | 53 | 30 | arXiv 1908.08351, table 1 |
| ConvS2S | 85 | 56 | 31 | arXiv 1908.08351, table 1 |
| Transformer | 92 | 72 | 50 | arXiv 1908.08351, table 1 |

The published models learn from 81,000 to 85,000 pairs; their numbers are the means of 3 runs.

## ToM-QA

Data: the ToM-QA stories of Nematzadeh and others (arXiv 1808.09352) as three tasks on the hub: Lots-of-LoRAs/task151_tomqa_find_location_easy_clean, task152 (easy, with sentences that do not matter) and task153 (hard). Each task has its own train, validation and test splits. Four kinds for each task: where the thing is after the last move, where it was before the move, where a person will look for it (first belief), and where one person thinks that another person searches (second belief). Lessons: the first 600 train stories of each kind. Read: the test split of each kind (1,950 in all).

| Kind | Easy | Easy, with noise | Hard |
|---|---|---|---|
| after the last move | 167 of 167 | 156 of 156 | 167 of 167 |
| before the move | 157 of 157 | 171 of 171 | 158 of 158 |
| first belief | 171 of 171 | 166 of 166 | 170 of 170 |
| second belief | 155 of 155 | 157 of 157 | 155 of 155 |

Published, on the test sets of the paper, with 10,000 training examples for each pair of form and question and one model for all: the best model of each form gets 100.00 (easy, all four models), 95.81 (easy with noise, Multiple Observer), 94.31 (the hard form, RelNet) and 88.63 (the hard form with noise, EntNet) (arXiv 1808.09352). ILM reads the test splits of the hub tasks, not the test sets of the paper. During development, the board's read part held all 1,950 of these test cases (shares only; see the notes on the numbers), so this number is not a read of unseen cases.

## The board

The board measures all kinds of 25 public sets on their read parts, with the same program: 240 kinds in 112 tasks. A set gives only its files, its sizes, who wrote its text and its marks. ILM learns 165 kinds and 79 tasks in 14 sets, by the bar of [HOW_IT_WORKS.md](HOW_IT_WORKS.md). 73 of the 79 tasks are also learned from 200 lessons. The whole board takes 29.2 s at 24 threads.

For each task, the board says what reads it:

- Read by: learned (every line that reads the told text or answers was learned from the lessons of the kind), written (lines of the library alone), or mixed.
- Rules by: discovered (every rule that the proofs of the answers rest on was learned from the lessons, and no text of the library holds it), given (rules of the library), or mixed.
- Facts by: the facts known before the story that the proofs rest on, discovered, given or mixed.
- The ladder of the learned tasks: 15 discovered (read by learned lines, with no rule of the library), 29 learned (learned lines with rules of the library), 7 mixed (learned and written lines) and 28 written (written lines alone).
- Answer marks, over all read cases: proof 78.4% (of these, 98.8% right), default 18.1%, guess 2.4% and none 1.1%. A proof mark says that steps prove the answer and that the text holds no pair of pieces in a row that the lessons did not show; a default comes from an "otherwise" line.

## Where ILM does not learn

- Text that people wrote. No task of the board whose text people wrote is learned: the stories of CLUTRR (no result), the word problems of MAWPS and ASDiv (not learned) and of SVAMP (no result), the intents of ATIS and SNIPS (not learned) and their slots (no result), the rules in the words of people of ParaRules (not learned), and the descriptions by people of NL-RX (no result).
- Other sets: StepGame, SpartQA, FLD, BoardgameQA and the made descriptions of NL-RX give no result. ILM does not learn the second belief of ToMi, a kind of the set that the board does not count as a task.
- bAbI task 14 (time): 519 of 1,000 on its test split. Learned from 943 or more of its lessons, the kind reads "before" in the order of the story, and the stories of the task tell the times out of order (see the bAbI section).
- Rules read by the library: ProofWriter, RuleTaker, ProsQA, NLGraph and MindGames are learned tasks of the written rung. Their told rules are read by lines that a person wrote.

## Notes on the numbers

- During development, ILM used the training and validation data of each set. The board also reads a fixed part of the test split of some sets (the last cases of each kind), and of that part only the share of right answers was looked at, never a wrong case: the last 400 cases of each test split of SCAN, ProofWriter, the sentence transformations and PCFG SET; 527 of the 1,045 test graphs of CLUTRR in the graph form; and the whole test split of ToM-QA. So the ToM-QA number is not a read of unseen cases.
- Reads of test data that gave or changed a number: the test splits of bAbI, SCAN and ProofWriter two times, by two builds of the program (the counts of bAbI and ProofWriter differ between the reads, SCAN's do not); the sealed part of LoNLI two times; the test splits of CLUTRR in the graph form, the sentence transformations, PCFG SET and ToM-QA once by a final run, after the board's read parts had held a part of them (see above). The tables give the last read.
- On one computer, every count is the same in every run and for every number of threads. Another operating system can give a logarithm that differs in its last bit, and in a rare close decision this could move a count; the CI checks the bAbI counts on Linux, Windows and macOS.
