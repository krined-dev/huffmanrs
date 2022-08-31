# Simple Rust implementation of a huffman coder

## Build
Only tested on ubuntu 22.04<br>
build with:
```shell
cargo build --release
```
The binary will be placed in target/release/huffmanrs and has a simple CLI interface<br>
Current flags: 
```shell
-f --file : path to file to encode
-h --help : up to date documentation about the cli interface
```

## Notes
It is important to build the application with --release flag. If built with regular debug the performance will suffer.
Example: The medium-input.huff encodes in 600ms in debug and 40ms in release.

## Todos
The application currently only does the encoding in memory and can not export to a file format. Future implementations<br>
will be the ability to compress an input text file to an compressed output file that again can be decoded into the <br>original
text.
