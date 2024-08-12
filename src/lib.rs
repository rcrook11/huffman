use std::collections::HashMap;
use std::error::Error as Err;
use std::fs::File;
use std::io::{BufReader, Read};

pub fn run(config: Config) -> Result<(), Box<dyn Err>> {
    match config.cmd.as_str() {
        "huff" => huff(&config.file_path),
        _ => Err(Box::from("Invalid command")),
    }
}

fn count_byte_frequency(file_path: &str) -> Result<HashMap<u8, usize>, Box<dyn Err>> {
    let f = BufReader::new(File::open(file_path)?);

    let mut byte_frequency = HashMap::new();

    for byte in f.bytes() {
        let count = byte_frequency.entry(byte?).or_insert(0);
        *count += 1;
    }

    Ok(byte_frequency)
}

pub fn huff(file_path: &str) -> Result<(), Box<dyn Err>> {
    let byte_frequency = count_byte_frequency(file_path)?;
    let root = HuffNode::build_huff_tree(&byte_frequency);
    let mut leaves = HashMap::new();
    root.gather_leaves(&mut leaves);
    println!("{:?}", leaves);

    Ok(())
}

pub struct Config {
    pub cmd: String,
    pub file_path: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String> + ExactSizeIterator) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Not enough arguments");
        } else if args.len() > 3 {
            return Err("Too many arguments");
        }

        args.next();

        let cmd = match args.next() {
            Some(arg) => arg,
            None => return Err("No command provided"),
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("No file path provided"),
        };

        Ok(Config{ cmd, file_path })
    }
}

struct HuffNode {
    code: Option<HuffCode>,
    frequency: usize,
    left: Option<Box<HuffNode>>,
    right: Option<Box<HuffNode>>,
}

#[derive(Debug)]
struct HuffCode {
    byte: u8,
    code: usize,
    length: usize,
}

impl HuffNode {
    fn new(byte: u8, frequency: usize, code: usize) -> HuffNode {
        HuffNode {
            code: Some(HuffCode {
                byte,
                code,
                length: 0,
            }),
            frequency,
            left: None,
            right: None,
        }
    }

    pub fn build_huff_tree(byte_frequency: &HashMap<u8, usize>) -> HuffNode {
        let mut nodes = byte_frequency.iter()
            .map(|(byte, frequency)| HuffNode::new(*byte, *frequency, 0))
            .collect::<Vec<HuffNode>>();

        while nodes.len() > 1 {
            nodes.sort_by(|a, b| a.frequency.cmp(&b.frequency));

            let left = nodes.remove(0);
            let right = nodes.remove(0);

            nodes.push(HuffNode::combine(left, right));
        }

        nodes.remove(0)
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    pub fn combine(mut left: HuffNode, mut right: HuffNode) -> HuffNode {
        left.update_codes(false);
        right.update_codes(true);
        HuffNode {
            code: None,
            frequency: left.frequency + right.frequency,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    pub fn update_codes(&mut self, is_right: bool) {
        if let Some(ref mut code) = self.code {
            code.code = code.code << 1;
            code.code |= if is_right { 1 } else { 0 };
            code.length += 1;
        }

        if let Some(ref mut left) = self.left {
            left.update_codes(is_right);
        }

        if let Some(ref mut right) = self.right {
            right.update_codes(is_right);
        }
    }

    pub fn print_leaves(&self) {
        if let Some(ref code) = self.code {
            if self.is_leaf() {
                println!("{} - {:08b} - {:08b} - {}", self.frequency, code.byte, code.code, code.length);
            }
        }

        if let Some(ref left) = self.left {
            left.print_leaves();
        }

        if let Some(ref right) = self.right {
            right.print_leaves();
        }
    }

    pub fn gather_leaves<'a>(&'a self, leaves: &mut HashMap<u8, &'a HuffCode>) {
        if let Some(ref code) = self.code {
            if self.is_leaf() {
                leaves.insert(code.byte, code);
            }
        }

        if let Some(ref left) = self.left {
            left.gather_leaves(leaves);
        }

        if let Some(ref right) = self.right {
            right.gather_leaves(leaves);
        }
    }
}