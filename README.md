# Hypothesis-rs

My research project of porting [Hypothesis](https://github.com/HypothesisWorks/hypothesis) to Rust. The main goal is
to deepen my understanding of how Hypothesis works and improve my Rust skills.

This project doesn't aim to be a complete port - it is more about having fun.

In `crates` you may find Rust implementation of various Hypothesis's components. The `bindings` directory contains Python
bindings to them.

You can build and test individual crates in their respective directories with `cargo build` or `cargo test`.
All implemented parts pass relevant Hypothesis tests (ported to Rust) unless mentioned explicitly.

Crates:
 - `charmap`. Implements `hypothesis.internal.charmap`.
 - `database`. Implements `hypothesis.database`, except `ReadOnlyDatabase` and `MultiplexedDatabase`.

## Python bindings

Python bindings for these crates aim to be a drop-in replacement for their respective blocks.
For more information see README files in `bindings/*`.
