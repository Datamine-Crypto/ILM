# How ILM works

ILM learns a kind of task from lessons. A lesson is a short text, a question and the exact answer. ILM counts which parts of the text go with which answer, and it keeps only what the counts show to be sure. The result is a set of tables, rules and small programs that a person can read.

ILM computes no gradient and has no network. All counts are whole numbers. The same lessons give the same model on every run and for every number of threads.

## Cases, kinds and sets

A case is a list of told sentences, a question and an answer. A kind is a group of cases of one form, for example "bAbI task 2" or "CLUTRR, 5 steps". A set is a public benchmark. Its description in `data/sets` names its kinds, its tasks and its raw files.

`ilm export` reads the raw files of a set and writes one case file for each kind (the format is in [COMMANDS.md](COMMANDS.md)).

ILM is not told the kind of a case. It gets lessons of one kind, learns from them, and reads new cases of that kind.

## One call for every tool

ILM has fifteen tools. Each tool has the same call: it learns from lessons, and then it answers a told text with a question. Each answer has a note that tells how the tool found it: the rule, the sentence or the program.

## The choice among the tools

For each kind, every tool learns from the first two thirds of the lessons and reads the last third. The tool with the most right answers on that third is taken, and it learns again from all lessons. With equal results, the tool that comes first in the list is taken.

No person chooses a tool for a kind, and no setting changes between kinds. This is the cross-validatory choice of Stone.

## The tools

Each tool is named for its way of reading, not for a task. The list below is in the order of the choice.

**Patterns.** A pattern is the question with open places (slots) and up to a few told sentences with slots. Each later sentence of a pattern shares a word with the question or with an earlier sentence (the relational pathfinding of Richards and Mooney). Each lesson gives one reading to each pattern that it shows, with the place of the answer. A pattern that is read often and is not sure gets one more sentence. The sure patterns become an ordered list of rules (sequential covering, as in CN2): the surest rule first, then the rules for the lessons that it does not cover. Words that stand in the same place of the same text are of one kind (the distributional method of Harris). So a word that no lesson held is read by the place where it stands. Told rules ("if ..., then ...") are used until nothing new comes, as in Datalog.

**State.** The tool reads a story into units: sentences and parts of sentences that tell one step each. The state of the story is a small table, for example "thing: place". For each form of unit, the tool finds a program that changes the table. An answer is read from the table by one of a fixed set of readings: the value of a holder, its members, their count, the value before another. The programs are found by local search, and a change stays only when more lessons come out right. A program becomes a rule only when it is right on nearly all cases of the search, and again on nearly all of the last third of the cases, which the search did not see.

**Amount.** The tool holds a small fixed set of exact calculations on the numbers of a text. It counts which calculation each form of text means. It keeps a form only when the sure test passes. All arithmetic is exact, with fractions of whole numbers.

**Order.** The tool learns an order that no case tells and that is the same in all cases. It counts which thing stood before which in the lessons, and it uses that order in a new case.

**Link.** The tool finds a way from one thing of the question to the other through the told links, and it joins the labels of the links along the way. The join table (for example, "the father of the mother is the grandfather") is counted from lessons that give only the story, the question and the answer. A lesson of two links gives a join directly. A longer lesson gives the join of the two parts of its way, at each cut where both parts are joined already, with the answer of the lesson; a pair becomes an entry when its lessons give it one answer. Two labels that give the same answer with every shared partner are alike (the substitution test of Harris), and one lends the other a join that it lacks; a lent join is used only where no counted join reaches the whole way.

**Rule.** The tool turns each told sentence into a fact or a rule. It uses a reader that is counted from lessons for which the set gives formal forms. A small exact part then draws all that follows: forward chaining to a fixed point, with negation as failure.

**Parts.** The meaning of a text is built from the meanings of its parts (the principle of Frege). The tool has a small language of operations on rows of answer words: join two answers, repeat an answer, turn it, take one end. It finds forms and their programs from the lessons by minimal pairs and by laying runs side by side (the anti-unification of Plotkin). It reads a new text with a chart parser (Cocke, Younger and Kasami) and takes the reading with the lowest cost.

**Together.** Tools work together inside one problem. In a word problem, the amount tool reads the numbers and makes each step of the calculation exactly. This tool gives each number a role: added, taken away, a factor, a divisor, or not used. Trees of marks, grown by counts with no chance in them, vote on the roles. For a question that names two numbers, a program on these numbers is found from the lessons with the nearest questions. In a story that moves a place and turns, the state tool reads the units and this tool walks the place.

**List.** The tool counts over a told list: how many things of a class the list holds. It counts the amount of each word and the class of each thing from the lessons. Each step of the sum is an exact addition.

**Requests.** The tool reads a request that a person said or wrote: which words fill which slot (a city, a date, a name) and the label of the whole request. The tags are counted from the words around each word. The program holds no list of cities, days or names.

**Frames.** The tool reads told facts from sentences that people wrote. People write one relation in many ways, and a new story holds sentences that no lesson held. So the tool reads the label of a link from the frames around the relation words of its sentence, counted on the labels of the lesson stories. Then it joins the labels along the way with the join table of the link tool.

**Table.** The tool answers questions over a told table of facts with numbers. For each form of question, it finds a small program from the lessons: keep rows by named values or by a time, pick a row by an order, give a field, a count, a difference or a sorted list.

**Arrange.** The tool answers a question that lists its choices, each a statement about one of the things of the story, which stand in one order. The things are the parts in which the choices differ. Each told sentence and each choice is a form with open places for the things that it names. The tool finds what each form means for the places of its things (a thing at a place counted from the start or from the end, or one thing before or after another) by a search for meanings under which every lesson can be explained: some arrangement of the things makes its told statements and its answer true and its other choices false. The order of the places is free, so the most frequent form about two things is taken to mean "before". A new case is answered with the choice that holds in the arrangements that its told statements allow.

**Graph.** The tool reads a story as a graph. Of the told sentences of all lessons, those of the most frequent shape in which exactly two places change from sentence to sentence are links between the things in those places. The tool holds a small fixed set of properties of a graph: whether the links close a loop, and whether a chain of links joins the two things that the question names. Both are found with disjoint sets (Galler and Fischer). For each form of question, the lessons give which property it asks and which answer each value of the property has.

**Ranks.** The tool reads each number of a case by its rank among the different numbers of its told text and its question (the ordinal scale of Stevens): the smallest number is the first rank. So "420 miles from A and 780 miles from B" and "260 miles from A and 335 miles from B" are one form, and the pattern learner counts that form with its answer on the texts read in this way. A rank word in an answer is given back as the number of that rank. The tool reads a kind only where a lesson orders two different numbers, and the choice tries it only where the pattern learner reads the most lessons right so far, because the tool changes only how that learner reads the texts.

Two controls have the same call as the tools, but they are not in the choice. Plain counting is naive Bayes over the words of the question and of the last told sentences. The question alone is the same count on lessons with every told sentence taken away.

## Word classes from plain text

Some readers use a table of word classes: words that stand in the same places of a text fall into one class, for example first names, cities or months. `ilm classes` counts the table from plain text with no label. For each of the 40,000 most frequent words, it counts how often each of the 2,000 most frequent words stands right before it and right after it. Words with alike counts are grouped into 1,000 classes by k-means on these counts (the spherical k-means of Dhillon and Modha). The first centres are words at even steps of the list by frequency, so the same text always gives the same table.

## Facts from an encyclopedia

The fact reader answers questions such as "Where was X born?" from the text of an encyclopedia. The text store holds one article on each line.

The reader learns from lesson facts (a relation, a subject and an object) and from the articles of their subjects. A frame is what stands around a candidate in an article: one to three words before it, one or two words after it, or both. For each frame and each relation, the lessons give how often the frame stood around a candidate, and how often that candidate was the object of the fact. The candidates of an article are its names (runs of words with capital letters in one sentence) and the objects of the lesson facts. The shares of the frames of one candidate are joined as chances that do not depend on each other (the noisy-or of Pearl).

Two more routes help where no sentence states the fact. Sure words: an article about a woman says "she". Listed objects: some relations have a short list of objects, such as sorts of music. For each relation, a model of a part of the lesson subjects reads the articles of the rest, and the order of the routes with more right answers is kept.

Each answer that comes from a sentence names that sentence. When the text gives no answer, the reader guesses from the name of the subject: an object that stands in the name (a road named after its state), else naive Bayes over the words of the name. All counts come from the lesson facts.

The question reader goes from a question that a person wrote to its subject and its relation. It counts the frames at the two ends of the subject and the form of each question.

## Text from a list of facts

The writer learns from lessons that are each a list of facts and a text that a person wrote from them. It counts which slots are names, which values are lists, which runs of tokens say which value, and which forms of sentence the lessons have. A form is a lesson sentence that the reader of the writer reads back to exactly its facts, with a place for each value.

To write a text, the writer searches sequences of forms with a beam. It takes the text with the highest share of runs of tokens that stand in the texts of the lessons with the nearest lists of facts. A check by counted marks makes sure that the text says every given fact and no other fact.

## One command

`ilm ask` takes one request and prints a trace of every step. A told story with a question goes to the tools, through the router of the counted kinds. A question about a fact goes to the table of facts. A request to a service goes to the reader of requests.

## How a kind is measured

- The cases of a kind are cut into a learned part, a read part and a sealed part. All cases with the same told sentences stay in one part.
- A case is right only when the words of the answer are exactly the words of the true answer.
- Three controls stand beside the model: one answer for all cases, plain counting, and the question alone.
- A kind is learned at 95% or more of the read cases, and only when the model is 5 points or more above one answer for all and above the question alone. So a kind whose answers need no reading of the text is not learned.
- Read cases that are copies of lessons do not count. When a kind has copies, the share on its other read cases counts, and only with 50 or more of them.
- On the board, the bar is 90% for a set whose text people wrote (ATIS, CLUTRR, ParaRules, SNIPS and the word problems). A set can also mark a kind where the question alone gives so much that no model can be 5 points above it (ProofWriter, closed form, depth 5; ParaRules, depths 3 and 4). Such a kind passes at the bar alone.
- On the board, the read cases are read again with new words in place of the names, places and things.
- A control of changed cases asks whether the answer follows a change of the text. A model reads changed forms of lessons that it answers right, and the control counts how often it gives the new true answer. A task counts only when the learner follows this control.
- The sealed part is for one final run with frozen code. The final reads of the benchmarks read their test data in the same way.

## Exact results

- Counts are whole numbers. The shares that decide a vote are exact fractions.
- Sums of doubles use the compensated sum or one fixed order of terms.
- Tables keep the order of first sight. Of equal candidates, the first one is taken.
- Work is cut into runs on all threads, and the runs are joined in their order. So the result is the same for every number of threads.

## Rules of the code

The code follows the rules of the crate [premise](https://crates.io/crates/premise). Each idea is a type with its published source (`source!`). Each constant, type and function has a reason (`because!`). The code has no comment and no number without a name. The checker must say `clean`. The tests run on made-up cases and need no data.
