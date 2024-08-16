use std::{
    collections::HashMap,
    error::Error as Err,
    fs::File,
    io::{BufReader, Read, BufWriter, Write}
};
use bit_vec::BitVec;

pub fn run(config: Config) -> Result<(), Box<dyn Err>> {
    match config.cmd.as_str() {
        "huff" => huff(&config.file_path),
        "puff" => puff(&config.file_path),
        _ => Err(Box::from("Invalid command")),
    }
}

pub fn huff(file_path: &str) -> Result<(), Box<dyn Err>> {
    let byte_frequency = count_byte_frequency(file_path)?;

    let mut root = HuffNode::build_huff_tree(&byte_frequency);
    root.update_codes(&0,&0);

    let mut leaves = HashMap::new();
    root.gather_leaves(&mut leaves);

    huff_encode(leaves, file_path)?;
    write_huff_key(byte_frequency, file_path)?;

    Ok(())
}

pub fn puff(file_path: &str) -> Result<(), Box<dyn Err>> {
    let byte_codes = read_huff_key(file_path.to_owned() + "k")?;

    let mut root = HuffNode::build_huff_tree(&byte_codes);
    root.update_codes(&0,&0);

    let root = Box::new(root);

    huff_decode(root, file_path)?;
    Ok(())
}

fn count_byte_frequency(file_path: &str) -> Result<HashMap<u8, u64>, Box<dyn Err>> {
    let f = BufReader::new(File::open(file_path)?).bytes();

    let mut byte_frequency = HashMap::new();

    for byte in f {
        let count = byte_frequency.entry(byte?).or_insert(0);
        *count += 1;
    }

    Ok(byte_frequency)
}


fn huff_encode(byte_codes: HashMap<u8, Box<HuffCode>>, file_path: &str) -> Result<(), Box<dyn Err>> {
    let f = BufReader::new(File::open(file_path)?).bytes();
    let mut bit_vec = BitVec::new();

    for byte in f {
        if let Some(code) = byte_codes.get(&byte?) {
            for i in (0..code.length).rev() {
                bit_vec.push((code.code & (1 << i)) != 0);
            }
        }
    }

    let mut f_out = BufWriter::new(File::create(file_path.to_owned() + ".huff")?);
    f_out.write(bit_vec.to_bytes().as_slice())?;

    Ok(())
}

fn huff_decode(root: Box<HuffNode>, file_path: &str) -> Result<(), Box<dyn Err>> {
    let out_path = "out_".to_owned() + &*file_path.to_owned()
        .split(".huff")
        .collect::<Vec<&str>>()[0];

    let f = BufReader::new(File::open(file_path)?);
    let mut f_out = BufWriter::new(File::create(out_path)?);

    let mut bit_vec = BitVec::from_bytes(&mut f.bytes().collect::<Result<Vec<u8>, _>>()?);
    let mut bits: usize = 0;

    while let Some(byte) = root.as_ref().search_tree(&mut bit_vec, &mut bits) {
        f_out.write(&[byte])?;
    }

    Ok(())
}

fn write_huff_key(byte_codes: HashMap<u8, u64>, file_path: &str) -> Result<(), Box<dyn Err>> {
    let mut f = BufWriter::new(File::create(file_path.to_owned() + ".huffk")?);

    for (byte, freq) in byte_codes {
        f.write(&[byte])?;
        f.write(&freq.to_be_bytes())?;
    }

    Ok(())
}

fn read_huff_key(file_path: String) -> Result<HashMap<u8, u64>, Box<dyn Err>> {
    let mut f = BufReader::new(File::open(file_path)?);
    let mut byte_frequency = HashMap::new();
    let mut bytes: [u8; 9] = [0; 9];

    while f.read_exact(&mut bytes).is_ok() {
        let byte = bytes[0];
        let freq_bytes = &bytes[1..9];
        let freq = u64::from_be_bytes(freq_bytes.try_into().unwrap());
        byte_frequency.insert(byte, freq);
    }

    Ok(byte_frequency)
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
    code: Box<HuffCode>,
    frequency: u64,
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
    fn new(byte: u8, frequency: u64, code: usize) -> HuffNode {
        HuffNode {
            code: Box::new( HuffCode {
                byte,
                code,
                length: 0,
             }),
            frequency,
            left: None,
            right: None,
        }
    }

    fn build_huff_tree(byte_frequency: &HashMap<u8, u64>) -> HuffNode {
        let mut nodes = byte_frequency.iter()
            .map(|(byte, frequency)| HuffNode::new(*byte, *frequency, 0))
            .collect::<Vec<HuffNode>>();

        while nodes.len() > 1 {
            nodes.sort_by(|a, b| a.frequency.cmp(&b.frequency)
                .then_with(|| a.code.byte.cmp(&b.code.byte)));

            let left = nodes.remove(0);
            let right = nodes.remove(0);

            nodes.push(HuffNode::combine(left, right));
        }

        nodes.remove(0)
    }

    fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    fn combine(left: HuffNode, right: HuffNode) -> HuffNode {
        HuffNode {
            code: if &left.code.byte < &right.code.byte {
                Box::new(HuffCode {
                    byte: left.code.byte,
                    code: 0,
                    length: 0,
                })
            } else {
                Box::new(HuffCode {
                    byte: right.code.byte,
                    code: 0,
                    length: 0,
                })
            },
            frequency: left.frequency + right.frequency,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    fn update_codes(&mut self, c: &usize, l: &usize) {
        if self.is_leaf() {
            self.code.code = *c;
            self.code.length = *l;
        }

        if let Some(ref mut left) = self.left {
            left.update_codes(&(c << 1), &(l + 1));
        }

        if let Some(ref mut right) = self.right {
            right.update_codes(&((c << 1) | 1usize), &(l + 1));
        }
    }

    fn gather_leaves(&self, leaves: &mut HashMap<u8, Box<HuffCode>>) {
            if self.is_leaf() {
                leaves.insert(self.code.byte, Box::new(HuffCode {
                    byte: self.code.byte,
                    code: self.code.code,
                    length: self.code.length,
                }));
            }

        if let Some(ref left) = self.left {
            left.gather_leaves(leaves);
        }

        if let Some(ref right) = self.right {
            right.gather_leaves(leaves);
        }
    }

    fn search_tree(&self, bit_vec: &mut BitVec, bits: &mut usize) -> Option<u8> {
        if self.is_leaf() {
            return Some(self.code.byte);
        }

        if let Some(next_bit) = bit_vec.get(*bits) {
            return if next_bit {
                *bits += 1;
                match &self.right {
                    Some(right) => right.search_tree(bit_vec, bits),
                    None => None
                }
            } else {
                *bits += 1;
                match &self.left {
                    Some(left) => left.search_tree(bit_vec, bits),
                    None => None
                }
            }
        } else {
            None
        }
    }
}