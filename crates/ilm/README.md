# 🧮 datamine-ilm: the program ilm

⚡ ILM, the Integer Language Model, is a language model that needs no GPU, to learn or to answer. It learns each task from its lessons by counting, and what it learns is a set of rules that you can read.

## 📦 Install

```
cargo install datamine-ilm
```

This builds the program `ilm`. The build needs Rust 1.90 or later and a C compiler.

## 🚀 Run

The program reads the descriptions of the public sets from a clone of the GitHub repository, so run it from the root of a clone:

```
git clone https://github.com/Datamine-Crypto/ILM
cd ILM
ilm fetch data/sources.txt sets=src_babi
ilm export src_babi data/sets/final/src_babi.txt cases/final/src_babi
ilm measure cases/final/src_babi
```

The last line gives the right answers of each of the 20 bAbI tasks. Their sum is 19,356 of 19,399.

## 🔗 Links

- 💻 Results, commands and how it works: https://github.com/Datamine-Crypto/ILM
- 🤗 Model card and programs: https://huggingface.co/DatamineNetwork/ILM
- 🌐 Website: https://datamine.network

📜 License: Apache License 2.0.
