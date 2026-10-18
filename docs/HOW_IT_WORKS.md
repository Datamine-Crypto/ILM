# How ILM works

ILM learns a kind of task from lessons. A lesson is a short text, a question and the exact answer. What ILM learns is lines of a library in plain English: rules, question forms and facts. It keeps a line when the lessons show that the line answers right. The result is a set of lines that a person can read, and each answer names the lines that gave it, with a proof tree when steps prove it.

ILM computes no gradient and has no network. All counts are whole numbers. The same lessons give the same model on every run and for every number of threads.

## What ILM discovered

On the board, ILM learns 79 of 112 tasks. The board's ladder says how each learned task reads: 15 discovered (read by learned lines, with no rule of the library), 29 learned (learned lines with rules of the library), 7 mixed (learned and written lines) and 28 written (lines of the library alone).

- Rules discovered from lessons. On the discovered rung, 10 learned tasks rest only on rules that were discovered from the lessons, and no text of the library holds them: bAbI tasks 9 (negation) and 10 (indefinite knowledge); CLUTRR in the graph form, where rules learned from graphs of 2 to 4 relations read graphs of up to 10; LoNLI causal and world; SCAN; and the four tasks of the sentence transformations. The 5 other tasks of that rung need no rule: bAbI task 4, LoNLI quantifier, ToM-QA "before the move", and the BIG-bench tasks arithmetic and multistep_arithmetic. ToMi, where the thing is after the last move, also rests only on discovered rules, but written lines read a part of it, so it stands on the mixed rung.
- Rules that a person can read. Two of them, from the store: "If A C B and C means a D child then A is a parent of B." (discovered on the lessons of CLUTRR in the graph form) and "If R got the S there then The got of the S is R." (discovered on bAbI task 2: "the got of" is the name that ILM made for a state that holds from the time that a thing was got). The proof in the README shows "If A picked up the B then A got the B", discovered on bAbI task 2.
- Rewrites with no fact of the library. The four tasks of the sentence transformations are read by rewrites learned from 1,000 lessons for each part, with no rule and no fact of the library (rules by and facts by: discovered). They read all 80,000 sentences of the test and generalization sets right.
- Facts read from Wikipedia. The facts step read the 65 capitals of `data/explain/learned/capitals.txt` from a plain-text copy of the English Wikipedia, with sentence shapes that it learned itself and the lessons of LoNLI's world group as the judge. Each fact names the files and lines of its sentences, and the board counts the facts of that task as discovered.

## Features

- Proofs. `ilm prove` prints the answer to a story and a question with its proof tree in English, back to the told sentences. Each rule is marked as given (a line of the library) or as discovered on a kind, and each learned fact as discovered.
- Answer marks. Each answer is marked proof, default, guess or none. Over all read cases of the board: proof 78.4% (98.8% of them right), default 18.1%, guess 2.4% (74.5% right) and none 1.1%.
- Missing steps. `ilm why` shows, for each lesson that a model answers wrong, the proof of the answer that it gave and the first step that the facts of the story miss for the wanted answer.
- Facts from a large text. `ilm facts` reads the facts of a text of the library from plain text that people wrote, and keeps the source of each fact.
- The store. `data/store` keeps the rules that the board discovered, each with the kind and the number of lessons that it came from, for other kinds to judge on their own lessons. Only `ilm store-lines` writes it.
- The same result on every run. Every count is the same on every run and for every number of threads, and the CI checks the bAbI counts on Linux, Windows and macOS.
- CPU only. Learning and answering run on a CPU, with no GPU and no pre-trained model. On one thread, the whole board of 112 tasks takes 210 s with a peak of 2.6 GB of memory.
- Editing. The library is plain English: a person can add or change a line, `ilm places` lists the open places that the reader takes in each line, and `ilm concepts` lists the texts and what each builds on. A text that a program wrote (`data/explain/learned`) is refused when a person changed a line of it.

## Cases, kinds and sets

A case is a list of told sentences, a question and an answer. A kind is a group of cases of one form, for example "bAbI task 2" or "CLUTRR, 5 steps". A set is a public benchmark. Its description in `data/sets` names its kinds, its tasks and its raw files.

`ilm export` reads the raw files of a set and writes one case file for each kind (the format is in [COMMANDS.md](COMMANDS.md)).

ILM is not told the kind of a case. It gets lessons of one kind, learns from them, and reads new cases of that kind.

## One tool

ILM has one tool, the explain tool. The tool has one call: it learns from lessons, and then it answers a told text with a question. Each answer has a note that tells how the tool found it, and a proof that `ilm prove` prints.

For each kind, the tool tries its ways of reading on the first two thirds of the lessons and judges them on the last third. Then it learns again from all lessons. No person chooses a way for a kind, and no setting changes between kinds. This is the cross-validatory choice of Stone. The ways are in the section "How the tool learns a kind" below.

Two controls stand beside the tool, and the measure uses them only as controls. Plain counting is naive Bayes over the words of the question and of the last told sentences. The question alone is the higher of two shares: the same count on lessons with every told sentence taken away, and, for a kind with told sentences, a decision list over the words of the question and the pairs of its common words.

## The explain engine

The explain tool reads logic that people write in English, and it chains that logic over the told sentences of a case. Its texts are in `data/explain`, in the folders `concepts`, `knowledge`, `wording` and `learned`. A person wrote the texts of the first three folders; a program wrote the texts of `learned`. `ilm concepts` lists the texts, the texts that each builds on, and the kinds where each answers.

### Lines of a text

- A rule: "If A is a parent of B and B is a parent of C then A is a grandparent of C." When every condition holds under one filling of the open places, the outcome holds with the same filling.
- A rule that holds usually: "If A is a male name then usually A is male." It is a default: it adds its outcome only where nothing that the story knows rules it out.
- A told fact: a sentence with a full stop ("Seven is the number 7."). It holds in every story that the text reads.
- A question form with its answers, in the order in which they are tried: "Is A bigger than B? Yes if A is bigger than B. No if B is bigger than A." An answer can hold "otherwise". A question that holds a question mark of its own is written in quotes: To answer "<question>": <answers>.
- A reply line: Reply to "<question>" with <answer>: <sentence>. It says the answer as a sentence in the note of the answer.
- A line that writes a sentence again: "Write <old form> as <new form> if <conditions>." The old form is not kept. A sentence with many names comes as one sentence, written one part at a time.
- "Read told rules with <words> for any thing." A sentence of the story of the form "if ... then ..." becomes a rule of that story, and the given words stand for any thing. A told rule can have a name ("Rule1: if ... then ..."), and a told sentence "Rule1 is preferred over Rule2" makes the first rule win where the two give opposite outcomes.
- "Read the lessons as the answers of their questions." Each lesson that the tool learns from becomes the fact "the answer of <question> is <answer>", so that the rules of the text take what the lessons show.
- "Builds on <texts>." The text uses the rules, reply lines and told facts of the named texts, and of the texts that they build on.
- A line that starts with "Note:" is for the reader, and the engine skips it.

### Open places

- A capital letter from A to P, but not I, is an open place for one or more words.
- A capital letter from R to Z is an open place for exactly one word, such as a name or a verb.
- A letter from A to P before an ending inside one word ("Aes", "Best", "Bing", "A's") is the stem of a word with that ending. A stem that a sentence gave before is compared under the changes of English spelling (big, bigger; happy, happier; nice, nicer).
- A letter from A to P written twice (AA to PP) is an open place that may also hold marks, but not the brackets of a name.
- A letter from A to P written three times (AAA to PPP) is an open place that may also hold whole groups in brackets: every bracket that opens in it closes in it.
- A letter from A to D written four times (AAAA to DDDD) is an open place that may hold any signs, also brackets that it does not close, as a sequence of brackets is read sign by sign.
- A letter from R to U written twice (RR to UU) is an open place for exactly one sign of any kind: a word, a mark or a bracket.
- A hyphen or an apostrophe between two letters joins one word (self-assured, didn't), and a low line parts words as a space does.

### Conditions

- "not <sentence>": no fact fits the sentence. The engine reads a closed world (negation as failure).
- "latest <sentence>": only the latest fact of the story that fits it holds it. "at that time <sentence>" and "before that time <sentence>": the latest fitting fact up to, or before, the place in the story of the fact that the condition before it read.
- Comparisons: "A is less than B", "A is more than B", "A is at least B", "A is at most B", "A is the same as B", and "V comes before W in the alphabet" (letter by letter, as a dictionary sorts words).
- Steps of arithmetic on exact amounts: "A plus B is C", "A minus B is C", "A times B is C". A number with a point (3.5) is one word and an exact amount.
- Counts over the facts that fit a sentence: "C is the number of A where <sentence>", "C is the sum of A where <sentence>", "N is the flow from A to B where <sentence>" (the largest flow over the edges that the facts tell).
- Word positions in a whole sentence of the story: "GG holds the word X", "T comes after S in GG", "the first letter of X is L".
- "GG is told": the sentence was told in the story, and no rule made it.
- "B holds a function word": some word of the run is a word that a fact "W is a function word" names (any kind of word, with a or an). "C is B backward": the words of the run in the reverse order.
- "it follows that <statement>": the statement holds in every world that the announcements of the story leave (the logic of public announcements). The story names persons, what each person sees ("A sees the fact of B", "every person sees the facts of the others"), and what is announced. A statement can say that a person knows that, or knows whether, another statement holds.
- "it is proved that <statement>" and "it is disproved that <statement>": classical logic over told formulas. A told formula ("it holds that ...", with "and", "or", "if ... then", "for some thing" and "for any thing") is read in every direction as clauses of its parts. A sentence told false ("it is false that ...") is a fact of its own. A statement is proved when its opposite, added to what is told, leads to a clash.

### How the engine chains

- The engine adds the told sentences of a case as facts, each with its place in the story. Then it adds the outcomes of the rules, round after round, until a round adds nothing. A round tries only the fillings that use a fact that the round before added.
- Rules that read what is absent, or how many facts there are, wait in a later layer, stratum by stratum, until the rules before them add nothing more. Rules that hold usually come after every exact rule, and the named told rules after the others, the preferred first.
- A fact keeps the place in the story of the latest told sentence that it rests on. When a text reads "at that time" or "before that time", a fact keeps every place at which it came.
- Two bounds stop a chain that grows without end: 50,000 facts for one story, and 2,000,000 facts read by the conditions.
- The tool gives the first answer, in the order of the library, whose conditions hold. It gives no answer when none holds.

### Words as numbers, and the story cache

- Every word of the library and of the stories has a number in one table of the run. The engine compares, hashes and keeps numbers, and it reads the letters of a word only at an ending, in a number, and in the words that it says. The numbers come in the order in which the threads meet the words, so no result reads the order of the numbers.
- The facts of a story depend only on the rules, the facts that the topic knows before any story, the told sentences and the bounds. The engine keeps the facts of each story that it chains and gives them again when the same story comes again under the same topic (memoization). A kept story is given as a new empty layer over it, with its own count of facts read, so the answers stay the same. With `kept=`, the board also keeps these chains in a folder between runs, and reads them back only when the code of the engine is the same.

## How the tool learns a kind

The tool first takes the texts of the library whose question forms fit the questions of the lessons. A text is used for a kind when it answers right most of a sample of the lessons, with at least one of those answers proved and not given otherwise. The text with the largest share answers first. Texts with the same share are judged on more of their lessons, and texts that stay equal keep the order of the library.

Then every way that learns a reading from the lessons is judged on the last lessons of the kind against the same bars: how the written texts that fit read those lessons, and how the question alone reads them. The first way that passes gives the reading. A learned way passes only when it reads the last lessons at least as well as the written texts, and 5 points or more above the question alone. The ways, in their order:

- Labels by words. When every lesson of a kind asks the same question and the answer is a label of the whole told text (the intent of a request), the tool learns a decision list over the words of the told sentences (Rivest): each label of the list holds when a told sentence holds one word, or a pair of words, and the last label holds otherwise.
- Ways to reckon. For a word problem, the answer as a step of arithmetic over the numbers that the story tells, in their order (the first number minus the second, and so on), chosen by a decision list over the words of the story and the question.
- Meanings of function words. When the questions of a kind are words that apply to lists of other words, with no told sentences, the tool learns the meaning of each function word over a text of the library that names functions on lists by what they do (concepts/lists): each function word is tried with each meaning of the text on the lessons where it is the only word whose meaning is not yet known. `ilm meanings` prints the learned lines.
- Ways of joining. When the questions of a kind are commands with no told sentences (SCAN), the tool learns the meaning of each word, as the token that the lessons of the word always hold and the other lessons never hold, and the ways in which a joining word puts the parts of a command together, as the answer form that reads the most lessons. `ilm joining` prints them.
- Rewrites. When each lesson gives a sentence and its new form (a sentence into the passive, into a question), the words of the answer are found in the sentence of the question, as whole runs that move together or as other forms of its words that the lessons show.
- Learned wording, below.

### Learned wording

When no written question form fits a kind, or the written forms read too few of its last lessons, the tool learns the wording of the kind from its lessons, for a text of rules: each question becomes a form whose open places stand for the runs of words that change from lesson to lesson, and each form gets answers whose conditions are facts that the rules of the text make. The answers are chosen one after another by the lessons that they answer right, as a decision list is learned (Rivest). An answer that holds otherwise comes last, only after an answer with a condition, and only when the lessons that no condition covers agree on it, so that no answer comes from the question alone.

- A condition can be one fact, a fact compared with a word of the question, or two facts compared. A question with no open place takes such facts as they stand, so the answer says whether the story holds one.
- A condition can also be a fact of the stories, told or made by the rules, that holds the words of the question, with those words as the places of the form ("Bob is not big" gives "false if A is not B" for the question "Bob is big."). A fact that also holds the answer of its lesson gives a filled answer, with the answer as a free place ("C if A is a B" for "What is A?").
- An answer can rest on two facts joined by a value that the question does not name, the second read at the time of the first or before it ("C if B dropped the A and before that time B went to the C" for "Where is the A?").
- A loose condition that holds on every lesson of a form, and leaves a place of the question unnamed, says nothing of the story. It is never the first answer of a form, and after an answer it becomes the answer otherwise.
- When a kind tells no sentence, its question is the one told sentence ("The question is ..."), so that the rules of a text read the data that the question holds.
- Answer forms by places are candidates too. For a question whose input holds its own data, the answer can be the word at one place of the input, the same word in every lesson, or one step of arithmetic on the words at two places, for each length of input and of answer. The forms are question forms of the library, for example To answer "What is R plus S AAAA": A if R plus S is A.

`ilm wording` prints such a learned wording as lines of the library.

### Discovered rules

The learned wording also finds rules that no text of the library holds. Each is judged on the lessons as any learned line, and the board marks it as discovered.

- Rules of a concept. When no text reads the kind and its lessons tell stories, the tool learns the rules of a concept from the facts of the stories and the facts that answer the lessons: for a text whose reply lines name the asked fact, for a kind of a few labels with the question as a statement of each label in turn, or with the question and its answer as the name of the fact. A rule may read a fact at the time of an earlier sentence of the story.
- The told side. Rules that read the told sentences of a kind into the facts that the rules of a text start from: each sentence form that the stories tell is read into a fact of the text, and the reading that makes the most lessons right is kept.
- States over time. When no fact of a text names what holds between two told events (who holds a thing from the time that it was got until it was dropped), the tool invents that fact, named from the words of the event that starts it, as the event calculus of Kowalski and Sergot lets an event start a state and a later event end it.
- Bridges between names. When rules of a text read a fact that no fact of the stories fits, because the rule that made it was learned under another name, a rule from a sentence that the stories hold lets the rules chain again.
- Known shapes. A place of a learned rule that the lessons show only for some words reads the shape that the known facts give those words (father means a male parent), not every word (the least general generalization of Plotkin).
- Abduction. A question form of a text says which facts give an answer, so the answer of a lesson tells the facts that its story must make, as Peirce named the inference from a result back to its case.

### Kinds in turn, and the store

- The kinds of a set are learned in the order of the set. A kind also gets the lessons of the kinds before it, without any case whose story the kind reads, and the rules that an earlier kind discovered, so that a kind whose asked fact rests on facts of an earlier kind finds them made (curriculum learning, Bengio and others).
- The store in `data/store` keeps the rules that the board discovered on a kind, each with the kind and the number of lessons that it came from. Later kinds take a group of stored rules as candidates and judge it again on their own lessons. A stored rule is never a written line, so a proof that rests on it is discovered. Only `ilm store-lines` writes the store, and a run that learns from another number of lessons than its set gives does not read it.

Each learned way is written as lines of the library, so a person reads what was learned, and the engine reads the learned lines as it reads the written ones.

## Facts from a large text

`ilm facts` reads the facts of a text of the library from a large plain text that people wrote, with no sentence shape that a person wrote:

- The text's own form of its facts ("A is the capital of B") finds the first facts. The facts that the proofs of the lessons of the named kinds confirm become seeds.
- The sentences that hold both names of a seed give the shapes of the relation. A shape is kept by how it does on the seeds, also on the seeds whose true partner is not the name that the text writes most beside the key. The kept shapes find more facts (the dual iterative pattern relation expansion of Brin).
- A value of a relation with one partner (the capital of a country) is taken only when the text names one partner far more often than any other.
- The proofs of the lessons judge the found facts again. Each fact keeps the files and lines of the sentences that gave it, and the shapes stay out of the library, as each holds words of the text.

The file `data/explain/learned/capitals.txt` came from this step over a plain-text copy of the English Wikipedia, with the lessons of LoNLI's world group as the judge. The board counts its facts as discovered.

## Proofs and answer marks

- `ilm prove` prints the answer to told sentences and a question with its proof tree, back to the told sentences: each fact with the rule that made it, or "told". Each rule is marked as given, when a text of the library holds it, or as discovered on the kind, when the model learned it from the lessons. A proof is found backward from the facts that an answer rests on, only when it is asked, so a run that asks none pays nothing.
- Each answer has a mark. Proof: steps prove it, and the told sentences and the question hold no two pieces in a row that the lessons did not show (a piece is a word that a line names, or a run of other words with its length). Guess: they hold such a pair, or plain counting gave the answer. Default: an "otherwise" line gave it. None: no answer. The mark follows the reject option of Chow: a reader marks the answers where its evidence is thin. It is not a promise: on the board, 98.8% of the answers with a proof mark are right.
- `ilm why` takes each lesson that a model answers wrong. Of the answers of the library that could give the wanted answer, it finds the one whose conditions hold furthest, and the first condition that the facts of the story miss (the algorithmic debugging of Shapiro).

## How a kind is measured

- The cases of a kind are cut into a learned part, a read part and a sealed part. All cases with the same told sentences stay in one part.
- A case is right only when the words of the answer are exactly the words of the true answer.
- Three controls stand beside the model: one answer for all cases, plain counting, and the question alone.
- A kind is learned at 95% or more of the read cases, and only when the model is 5 points or more above one answer for all and above the question alone. So a kind whose answers need no reading of the text is not learned.
- Read cases that are copies of lessons do not count. When a kind has copies, the share on its other read cases counts, and only with 50 or more of them.
- On the board, the bar is 90% for a set whose text people wrote (ATIS, CLUTRR, ParaRules, SNIPS and the word problems). A set can also mark a kind where the question alone gives so much that no model can be 5 points above it (ProofWriter, closed form, depth 5; ParaRules, depths 3 and 4). Such a kind passes at the bar alone.
- On the board, the read cases are read again with new words in place of the names, places and things.
- A control of changed cases asks whether the answer follows a change of the text. A model reads changed forms of lessons that it answers right, and the control counts how often it gives the new true answer. `ilm changes` and `ilm control` run this control on request; the audits that ran it set the not_counted marks of the set descriptions, and a task with such a mark does not count.
- The sealed part is for one final run with frozen code. The final reads of the benchmarks read their test data in the same way.

## The board

The board reads every kind of every set through the explain tool and prints one line for each task. Beside the shares and the controls, each line says what reads the task:

- Read by: learned, when every line that reads the told text or answers was learned from the lessons of the kind; written, when lines of the library alone read it; mixed, when both do.
- Rules by: discovered, when every rule that the proofs of the answers rest on was learned from the lessons and no text of the library holds it; given, for rules of the library alone; mixed, for both.
- Facts by: the same for the facts known before the story that the proofs rest on.
- The ladder of a learned task: discovered (read by learned lines, with no rule of the library), learned (learned lines with rules of the library), mixed (learned and written lines) and written (written lines alone).
- Answer marks: the shares of proof, default, guess and none over the read cases, with the share right of the proof and guess marks.
- Learned from 200 lessons: whether the task is also learned when each kind learns from its first 200 lessons alone.

## Exact results

- Counts are whole numbers. The shares that decide a vote are exact fractions.
- Sums of doubles use the compensated sum or one fixed order of terms.
- Tables keep the order of first sight. Of equal candidates, the first one is taken.
- Work is cut into runs on all threads, and the runs are joined in their order. So the result is the same for every number of threads.
- No result reads the order in which words get their numbers.

## Rules of the code

The code follows the rules of the crate [premise](https://crates.io/crates/premise). Each idea is a type with its published source (`source!`). Each constant, type and function has a reason (`because!`). The code has no comment and no number without a name. The checker must say `clean`. The tests run on made-up cases and need no data.
