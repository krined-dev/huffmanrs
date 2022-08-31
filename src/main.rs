use std::cmp::{max, Reverse};
use std::collections::HashMap;
use std::{fmt, fs};
use std::fmt::{Debug, Formatter};
use std::time::Instant;
use itertools::Itertools;
use priority_queue::PriorityQueue;
use clap::Parser;

/// Initial implementation of the huffman encoding. Currently just using one file for the project
/// Refactoring and modularization will be done once I have a working prototype.
/// This is an initial naive solution, will optimize as we go

fn main() {
    let args = Args::parse();
    // Test value
    let medium_input = &fs::read_to_string(args.file).expect("Invalid file path")[..];
    // The Rust std lib is missing some useful iterators so we use the itertools crate
    // Let's find the frequency.

    let now = Instant::now();
    let huffman_code = process(medium_input).unwrap();
    let elapsed = now.elapsed().as_millis();
    println!("Text encoded in {} millis", elapsed);
    let text = decode(&Box::new(huffman_code.0), huffman_code.1);
    println!("Decoded to correct string: {}", text == medium_input)
}

fn process(input: &str) -> Result<(Node, Vec<u8>)> {
    let frq: Vec<_> = input.chars()
        .sorted()
        .group_by(|&c| c) // From itertools
        .into_iter()
        .map(|(k, v)| (k, v.count() as u32))
        .collect();

    // Priority queue - Can provide my own implementation of a priority queue in rust if needed for the assignment
    let mut pq: PriorityQueue<Node, Reverse<u32>> = PriorityQueue::from_iter(
        frq.into_iter().map(|it|
            (Node {
                content: Some(it.0),
                value: Some(it.1),
                left_child: None,
                right_child: None,
            }, Reverse(it.1))
        ));

    let tree = create_huffman(&mut pq)?;

    let encoded = encode(&tree, input.to_string())?;

    Ok((tree, encoded))
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

fn decode(root: &Box<Node>, code: Vec<u8> ) -> String {

    let mut text = String::new();
    let mut node = root;

    code.iter().for_each(|val| {

        match val.clone() {
            0 =>  if let Some(ref left_child) = node.left_child { node = left_child }
            _ =>if let Some(ref right_child) = node.right_child { node = right_child }
        }

        if let Some(content) = node.content {
            text.push(content);
            node = root
        }
    });

    text
}

fn encode(root: &Node, text: String) -> Result<Vec<u8>> {
    let height = tree_height(&root);
    let mut codes: Vec<_> = vec![0; height?];
    codes.fill(0);
    let mut huffman: HashMap<char, Vec<u8>> = HashMap::new();

    encode_helper(root.clone(), codes, 0, &mut huffman)?;


    let mut encoded: Vec<u8> = Vec::new();
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

fn encode_helper(root: &Node, mut code: Vec<u32>, idx: usize, huffman: &mut HashMap<char, Vec<u8>>) -> Result<()> {
    if root.left_child.is_none() && root.right_child.is_none() {
        let content = match root.content {
            Some(content) => content,
            None => return Err(HuffmanError::NodeContentNoneError)
        };

        huffman.entry(content).or_insert(format_arr(code.clone(), idx));
    }

    if root.left_child.is_some() {
        code[idx] = 0;
        if let Some(left_child) = &root.left_child {
            encode_helper(left_child, code.clone(), idx + 1, huffman)?;
        } else {
            return Err(HuffmanError::ChildNodeNoneError)
        }
    }
    if root.right_child.is_some() {
        code[idx] = 1;
        if let Some(right_child) = &root.right_child {
            encode_helper(right_child, code.clone(), idx + 1, huffman)?;
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

fn format_arr(codes: Vec<u32>, idx: usize) -> Vec<u8> {
    let mut arr: Vec<u8> = Vec::new();
    for i in 0..idx { arr.push(codes[i] as u8) }
    arr
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
    UnexpectedNoneValueForNodeValue
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
