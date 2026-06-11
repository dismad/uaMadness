# uaMadness
Dance with me

# Intro
A small Rust CLI tool that generates colorful vertical frequency bar charts for Zcash Unified Addresses (UAs).


![ua](https://github.com/user-attachments/assets/1acccfb2-558a-45d1-810a-dd23efd318be)



## Motivation

Count every character occurrence in the UA, then produce a clean, sorted “list” with visual proportional bars.
This completely hides the original sequence and positions while exactly preserving the multiset of characters and their frequencies.
It’s a compact statistical portrait / fingerprint of the UA instead of the long raw string.

## Features

- Rainbow gradient bars (hue shifts with frequency, brightness varies with height)
- Clean terminal output using `crossterm` truecolor
- Process a single UA or a whole file of UAs (`--file`)
- Export an animated GIF of multiple UAs (`--gif`)

## Build

```bash
cargo build --release
```









