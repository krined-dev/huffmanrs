extern crate core;

use std::cmp::{max, Reverse};
use std::collections::HashMap;
use std::{fmt, fs};
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::time::Instant;
use bitvec::array::BitArray;
use bitvec::bits;
use bitvec::prelude::BitVec;
use itertools::Itertools;
use priority_queue::PriorityQueue;
use clap::Parser;

/// Initial implementation of the huffman encoding. Currently just using one file for the project
/// Refactoring and modularization will be done once I have a working prototype.
/// This is an initial naive solution, will optimize as we go

fn main() {
    let args = Args::parse();
    // Test value
    let medium_input = &fs::read_to_string(&args.file.clone()).expect("Invalid file path")[..];
    // The Rust std lib is missing some useful iterators so we use the itertools crate
    // Let's find the frequency.

    let huffman_code = match process(medium_input) {
        Ok(huffman_code) => huffman_code,
        Err(error) => panic!("Unexpected error: {}: {}", error, error.to_string())
    };

    write_to_file(args.file.clone(), huffman_code.2, huffman_code.1).expect("TODO: panic message");
    let from_file = fs::read(args.file.clone() + ".hz").unwrap();
    let bits_from_file = BitVec::from_vec(from_file);
    let text = decode_huffman(&Box::new(huffman_code.0), bits_from_file);
    println!("Decoded to correct string: {}", text == medium_input);

}

fn process(input: &str) -> Result<(Node, BitVec<u8>, Vec<(char, u32)>)> {
    let frq: Vec<_> = input.chars()
        .sorted()
        .group_by(|&c| c) // From itertools
        .into_iter()
        .map(|(k, v)| (k, v.count() as u32))
        .collect();

    let mut pq: PriorityQueue<Node, Reverse<u32>> = PriorityQueue::from_iter(
        frq.clone().into_iter().map(|it|
            (Node {
                content: Some(it.0),
                value: Some(it.1),
                left_child: None,
                right_child: None,
            }, Reverse(it.1))
        ));

    let tree = create_huffman(&mut pq)?;

    let encoded = encode_huffman(&tree, input.to_string())?;

    Ok((tree, encoded, frq))
}

fn create_huffman(pq: &mut PriorityQueue<Node, Reverse<u32>>) -> Result<Node> {
    if pq.len() == 1 {
        return match pq.pop() {
            Some(priority_queue) => Ok(priority_queue.0),
            None => Err(HuffmanError::InvalidPriorityQueue)
        }
    }

    let node_one = match pq.pop() {
        Some(node) => node,
        None => return Err(HuffmanError::InvalidPriorityQueue)
    };

    let node_two = match pq.pop() {
        Some(node) => node,
        None => return Err(HuffmanError::InvalidPriorityQueue)
    };

    let node_one_val = match node_one.0.value {
        Some(value) => value,
        None => return Err(HuffmanError::UnexpectedNoneValueForNodeValue)
    };

    let node_two_val = match node_two.0.value {
        Some(value) => value,
        None => return Err(HuffmanError::UnexpectedNoneValueForNodeValue)
    };

    let value = node_one_val + node_two_val;

    pq.push(Node {
        content: None,
        value: Some(value),
        left_child: Some(Box::new(node_one.0)),
        right_child: Some(Box::new(node_two.0)),
    }, Reverse(value));

    Ok(create_huffman(pq)?)
}

fn decode_huffman(root: &Box<Node>, code: BitVec<u8>) -> String {

    let mut text = String::new();
    let mut node = root;

    code.iter().for_each(|val| {

        match *val.clone() {
            true =>  if let Some(ref left_child) = node.left_child { node = left_child }
            _ =>if let Some(ref right_child) = node.right_child { node = right_child }
        }

        if let Some(content) = node.content {
            text.push(content);
            node = root
        }
    });

    text
}

fn encode_huffman(root: &Node, text: String) -> Result<BitVec<u8>> {
    let mut codes = BitVec::new();
    let mut huffman: HashMap<char, BitVec> = HashMap::new();

    encode_helper(root.clone(), codes, &mut huffman)?;

    let mut encoded: BitVec<u8> = BitVec::new();
    let mut errors: Vec<char> = Vec::new();

    text.chars().for_each(|c| {
        match huffman.get(&c) {
            Some(code) => encoded.extend(code),
            None => errors.push(c)
        }
    });

    if errors.is_empty() {
        Ok(encoded)
    } else {
        Err(HuffmanError::MissingCodesForKeys)
    }
}

fn encode_helper(root: &Node, mut code: BitVec, huffman: &mut HashMap<char, BitVec>) -> Result<()> {
    if root.left_child.is_none() && root.right_child.is_none() {
        let content = match root.content {
            Some(content) => content,
            None => return Err(HuffmanError::NodeContentNoneError)
        };

        huffman.entry(content).or_insert(code.clone());
    }

    if root.left_child.is_some() {
        if let Some(left_child) = &root.left_child {
            let mut left_code = code.clone();
            left_code.push(true);
            encode_helper(left_child, left_code, huffman)?;
        } else {
            return Err(HuffmanError::ChildNodeNoneError)
        }
    }
    if root.right_child.is_some() {
        let mut right_code = code.clone();
        right_code.push(false);
        if let Some(right_child) = &root.right_child {
            encode_helper(right_child, right_code, huffman)?;
        } else {
            return Err(HuffmanError::ChildNodeNoneError)
        }
    }

    Ok(())
}

// Structs are stack allocated by default.
// Recursive structs makes the compiler cry out in agony as it has to calculate infinite stack size
// A box is however a pointer to heap allocated memory, now the compiler just has to calculate the size of the pointer
// The compiler is now happy

#[derive(Debug, Hash, Eq, PartialEq)]
struct Node {
    content: Option<char>,
    value: Option<u32>,
    left_child: Option<Box<Node>>,
    right_child: Option<Box<Node>>,
}

fn tree_height(node: &Node) -> Result<usize> {
    return if node.left_child.is_none() && node.right_child.is_none() {
        Ok(0)
    } else {
        let left_height = tree_height(
            match node.left_child.as_ref() {
                Some(node) => node,
                None => return Err(HuffmanError::TreeHeightError)
            });

        let right_height = tree_height(
            match node.right_child.as_ref() {
                Some(node) => node,
                None => return Err(HuffmanError::TreeHeightError)
            });
        Ok(max(left_height?, right_height?) + 1)
    }
}

fn write_to_file(path: String, frq: Vec<(char, u32)>, mut code: BitVec<u8>) -> Result<()> {
    let mut buffer = match File::create(path + ".hz") { // A take on .gz?
        Ok(file) => file,
        Err(_) => return Err(HuffmanError::UnableToCreateOutFile)
    };


    match buffer.write(code.as_raw_slice()) {
        Ok(_) => {},
        Err(_) => return Err(HuffmanError::CouldNotWriteEncodedToFile)
    }

    Ok(())
}

// Defining types for error handling using custom error types and typedef for Result

type Result<T> = std::result::Result< T, HuffmanError>;

#[derive(Debug)]
enum HuffmanError {
    TreeHeightError,
    ChildNodeNoneError,
    NodeContentNoneError,
    MissingCodesForKeys,
    InvalidPriorityQueue,
    UnexpectedNoneValueForNodeValue,
    UnableToCreateOutFile,
    CouldNotWriteEncodedToFile
}

impl fmt::Display for HuffmanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HuffmanError::TreeHeightError => {
                write!(f, "Could not evaluate height of huffman tree")
            },
            HuffmanError::ChildNodeNoneError => {
                write!(f, "Could not access child node. Error in huffman tree stucture")
            },
            HuffmanError::NodeContentNoneError=> {
                write!(f, "Could not access node content. Error in huffman tree stucture")
            }
            HuffmanError::MissingCodesForKeys => {
                write!(f, "Could not find corresponding binary code for the given key")
            },
            HuffmanError::InvalidPriorityQueue=> {
                write!(f, "Tried to pop None value from PQ")
            }
            HuffmanError::UnexpectedNoneValueForNodeValue=> {
                write!(f, "Tried to access value of node, but it was None")
            },
            HuffmanError::UnableToCreateOutFile=> {
                write!(f, "The application failed when creating output file.")
            },
            HuffmanError::CouldNotWriteEncodedToFile=> {
                write!(f, "Unable to write encoded buffer to file output, system out of memory or permission error?")
            }
        }
    }
}

impl std::error::Error for HuffmanError {}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// path to file to encode
    #[clap(short, long, value_parser)]
    file: String,
}

// test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let text = "Hello world!";
        let encoded = process(text).unwrap();
        let decoded = decode_huffman(&Box::new(encoded.0), encoded.1);
        assert_eq!(text, decoded);
    }

    // test compressed bitsize is smaller
    #[test]
    fn test_compressed_size() {
        let text = "Hello world!";
        let encoded = process(text).unwrap();
        //let decoded = decode_huffman(&Box::new(encoded.0), encoded.1);
        //assert_eq!(text, decoded);
        assert!(encoded.1.len() < text.len() * 8);
    }

}