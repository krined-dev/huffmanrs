use std::cmp::{max, Reverse};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use itertools::Itertools;

/// Initial implementation of the huffman encoding. Currently just using one file for the project
/// Refactoring and modularization will be done once I have a working prototype.
/// This is an initial naive solution, will optimize as we go
use priority_queue::PriorityQueue;

fn main() {
    // Test value
    let to_be_encoded = "
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum leo arcu, aliquet a nisi non, tincidunt tincidunt ligula. Donec placerat, nunc vel fringilla fringilla, libero mi tincidunt ante, non venenatis urna felis nec quam. Quisque fermentum nec risus id ullamcorper. Sed pretium aliquam ex sit amet fringilla. Maecenas vehicula ante et urna lacinia, sit amet scelerisque mi mattis. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Donec sit amet elit at nibh semper dapibus. In ut dolor at dolor efficitur aliquam vitae eu mi.
Pellentesque porta varius neque at pharetra. Etiam venenatis purus a massa suscipit consectetur eget et lorem. Vivamus aliquam elementum lacus eget vestibulum. Donec accumsan fermentum felis ac aliquam. Donec vel nibh id ipsum maximus semper quis id nunc. Praesent ut venenatis ante. Pellentesque ut augue tincidunt, semper sem vel, vehicula turpis. Maecenas sodales vehicula ex. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Proin dapibus consequat blandit. Nunc luctus vehicula est nec lacinia. Quisque ligula sem, bibendum nec porttitor ut, euismod ac ante.
Donec ac est ut sem aliquet dictum. Fusce egestas faucibus nisi a lacinia. Curabitur egestas nisi cursus mauris elementum, sit amet vulputate justo varius. Donec vel vehicula dui, ac rhoncus felis. Aliquam erat volutpat. Nullam elementum lorem non placerat feugiat. Cras facilisis, nisi vel ultricies condimentum, dolor dolor facilisis velit, vitae fringilla magna eros at lacus. Integer ullamcorper luctus libero vitae fermentum. Fusce nec lacus a lectus convallis cursus. Pellentesque rhoncus ante a velit dignissim, quis placerat tellus placerat. Nunc at elit vitae nisl venenatis molestie. Vivamus tristique scelerisque enim, a ultricies ex auctor non.
Etiam mattis lectus vitae magna varius cursus. Cras maximus malesuada diam eu facilisis. Fusce tempus sollicitudin massa. Morbi rutrum pulvinar lorem, sit amet scelerisque risus volutpat vitae. In hac habitasse platea dictumst. Quisque ullamcorper lectus justo, ut maximus sapien lobortis at. Sed iaculis porta dictum. Nunc at facilisis lectus, eget viverra magna.
Vivamus iaculis non nulla et congue. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia curae; Nulla ac enim orci. Maecenas volutpat ex ac mauris rutrum, nec sodales neque vulputate. Nulla ornare at urna ac tincidunt. Nam dapibus purus id mi condimentum, non eleifend eros egestas. Aenean lectus arcu, aliquam convallis accumsan eu, cursus et tortor. Etiam fringilla felis non odio congue convallis. Maecenas laoreet tempor mauris eget tincidunt. Pellentesque eu orci in tellus interdum fermentum et eget neque. Duis scelerisque, ex eget bibendum sodales, lorem elit lobortis nisl, eu euismod nunc lacus id neque. ";

    // The Rust std lib is missing some useful iterators so we use the itertools crate
    // Let's find the frequency and sort it :)
    let mut frq: Vec<_> = to_be_encoded.chars()
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

    let tree = create_huffman(&mut pq);

    println!("raw string size: {}, ",std::mem::size_of_val(&*to_be_encoded));
    let encoded = encode(&tree, to_be_encoded.to_string());
    println!("encoded size: {}", (std::mem::size_of_val(&*encoded)) / 8);
    let text = decode(&Box::new(tree), encoded);
    println!("{}", text == to_be_encoded)
}

fn create_huffman(pq: &mut PriorityQueue<Node, Reverse<u32>>) -> Node {
    if pq.len() == 1 { return pq.pop().unwrap().0 }

    let mut node_one = pq.pop().unwrap();
    let mut node_two = pq.pop().unwrap();

    let value = node_two.0.value.unwrap() + node_one.0.value.unwrap();

    pq.push(Node {
        content: None,
        value: Some(value),
        left_child: Some(Box::new(node_one.0)),
        right_child: Some(Box::new(node_two.0)),
    }, Reverse(value));

    create_huffman(pq)
}

fn decode(root: &Box<Node>, code: Vec<u8> ) -> String {

    let mut text = String::new();
    let mut node = root;

    code.iter().for_each(|val| {

        match val.clone() {
            0 =>  if let Some(ref left_child) = node.left_child { node = left_child }
            _ =>  if let Some(ref right_child) = node.right_child { node = right_child }
        }

        if let Some(content) = node.content {
            text.push(content);
            node = root
        }
    });
    text
}


fn encode(root: &Node, text: String) -> Vec<u8> {
    let height = tree_height(&root);
    let mut codes: Vec<_> = vec![0; height];
    codes.fill(0);
    let mut huffman: HashMap<char, Vec<u8>> = HashMap::new();

    encode_helper(root.clone(), codes, 0, &mut huffman);
    let mut encoded: Vec<u8> = Vec::new();
    text.chars().for_each(|c| {
        let code = huffman.get(&c).unwrap();
        encoded.extend(code);
    });
    encoded
}
fn encode_helper(root: &Node, mut code: Vec<u32>, idx: usize, huffman: &mut HashMap<char, Vec<u8>>) {
    if root.left_child.is_none() && root.right_child.is_none() {
        huffman.entry(root.content.unwrap()).or_insert(format_arr(code.clone(), idx));
    }

    if root.left_child.is_some() {
        code[idx] = 0;
        encode_helper(&*root.left_child.as_ref().unwrap(), code.clone(), idx + 1, huffman)
   }

    if root.right_child.is_some() {
        code[idx] = 1;
        encode_helper(&*root.right_child.as_ref().unwrap(), code.clone(), idx + 1, huffman)
   }
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

fn tree_height(node: &Node) -> usize {
    return if node.left_child.is_none() && node.right_child.is_none() {
        0
    } else {
        let left_height = tree_height(node.left_child.as_ref().unwrap());
        let right_height = tree_height(node.right_child.as_ref().unwrap());
        max(left_height, right_height) + 1
    }
}

fn format_arr(codes: Vec<u32>, idx: usize) -> Vec<u8> {
    let mut arr: Vec<u8> = Vec::new();
    for i in 0..idx { arr.push(codes[i] as u8) }
    arr
}
