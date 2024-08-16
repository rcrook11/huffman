//! A module that provides functions for compressing and decompressing files using the Huffman coding algorithm.
use std::{
    collections::HashMap,
    error::Error as Err,
    fs::File,
    io::{ BufReader, Read, BufWriter, Write, Seek }
};
use bit_vec::BitVec;

/// Compresses a file using the Huffman coding algorithm.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the file to be compressed.
///
/// # Errors
///
/// Returns an error if the file cannot be opened, read, or written to.
pub fn huff(file_path: &str) -> Result<(), Box<dyn Err>> {
    let byte_frequency = count_byte_frequency(file_path)?;

    let mut root = HuffNode::build_huff_tree(&byte_frequency);
    root.update_codes(&0,&0);

    let mut leaves = HashMap::new();
    root.gather_leaves(&mut leaves);

    huff_encode(leaves, file_path, byte_frequency)?;

    Ok(())
}

/// Decompresses a file that was compressed using the Huffman coding algorithm.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the file to be decompressed.
///
/// # Errors
///
/// Returns an error if the file cannot be opened, read, or written to.
pub fn puff(file_path: &str) -> Result<(), Box<dyn Err>> {
    let key = read_huff_key(file_path.to_owned())?;

    let mut root = HuffNode::build_huff_tree(&key.byte_frequency);
    root.update_codes(&0,&0);

    let root = Box::new(root);

    huff_decode(root, file_path, &key)?;
    Ok(())
}

/// Counts the frequency of each byte in a file.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the file to be analyzed.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or read.
fn count_byte_frequency(file_path: &str) -> Result<HashMap<u8, u64>, Box<dyn Err>> {
    let f = BufReader::new(File::open(file_path)?).bytes();

    let mut byte_frequency = HashMap::new();

    for byte in f {
        let count = byte_frequency.entry(byte?).or_insert(0);
        *count += 1;
    }

    Ok(byte_frequency)
}

/// Encodes a file using a Huffman codes generated from using the Huffman tree method
///
/// # Arguments
///
/// * `byte_codes` - A HashMap that maps a byte to it's Huffman code.
///
/// * `file_path` - A string slice that holds the path to the file to be encoded.
///
/// * `byte_frequency` - A HashMap that maps a byte to it's frequency in the file.
///
/// # Errors
///
/// Returns an error if the input file cannot be read.
///
/// Returns an error if the output file cannot be written to.
fn huff_encode(byte_codes: HashMap<u8, Box<HuffCode>>, file_path: &str, byte_frequency: HashMap<u8, u64>) -> Result<u64, Box<dyn Err>> {
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
    write_huff_key(byte_frequency, &mut f_out, bit_vec.len() as u64)?;
    f_out.write(bit_vec.to_bytes().as_slice())?;

    Ok(bit_vec.len() as u64)
}

/// Decodes a file using the key that was generated when the file was compressed
///
/// # Arguments
///
/// * `root` - A Box that holds the re-constructed Huffman tree.
///
/// * `file_path` - A string slice that holds the path to the file to be decoded.
///
/// * `key` - A reference to the key that was generated when the file was compressed.
///
/// # Errors
///
/// Returns an error if the input file cannot be read.
///
/// Returns an error if the output file cannot be written to.
///
/// Returns an error if the key is invalid.
fn huff_decode(root: Box<HuffNode>, file_path: &str, key: &HuffKey) -> Result<(), Box<dyn Err>> {
    let out_path = "out_".to_owned() + &*file_path.to_owned()
        .split(".huff")
        .collect::<Vec<&str>>()[0];

    let mut f = BufReader::new(File::open(file_path)?);
    f.seek(std::io::SeekFrom::Start(key.key_len as u64))?;
    let mut f_out = BufWriter::new(File::create(out_path)?);

    let mut bit_vec = BitVec::from_bytes(&mut f.bytes().collect::<Result<Vec<u8>, _>>()?);
    let mut bits: usize = 0;

    while let Some(byte) = root.as_ref().search_tree(&mut bit_vec, &mut bits) {
        if bits > key.huff_len as usize {
            break;
        }
        f_out.write(&[byte])?;
    }

    Ok(())
}

/// Writes the Huffman key to beginning of the output file
///
/// # Arguments
///
/// * `byte_frequencies` - A HashMap that maps a byte to it's frequency in the file.
///
/// * `f` - A mutable reference to the output file.
///
/// * `huff_len` - The length of the encoded file in bits.
///
/// # Errors
///
/// Returns an error if the key cannot be written to the output file.
fn write_huff_key(byte_frequencies: HashMap<u8, u64>, f: &mut BufWriter<File>, huff_len: u64) -> Result<(), Box<dyn Err>> {
    let key_len: u16 = byte_frequencies.len() as u16 * 9u16 + 10;
    let key_len_bytes = key_len.to_be_bytes();
    let huff_len_bytes = huff_len.to_be_bytes();

    f.write(&key_len_bytes)?;
    f.write(&huff_len_bytes)?;

    for (byte, freq) in byte_frequencies {
        f.write(&[byte])?;
        f.write(&freq.to_be_bytes())?;
    }

    Ok(())
}

/// Reads the Huffman key from the beginning of the compressed file
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the compressed file.
///
/// # Errors
///
/// Returns an error if the key cannot be read from the compressed file.
fn read_huff_key(file_path: String) -> Result<HuffKey, Box<dyn Err>> {
    let mut f = BufReader::new(File::open(file_path)?);
    let mut byte_frequency = HashMap::new();

    let mut key_len_bytes: [u8; 2] = [0; 2];
    f.read_exact(&mut key_len_bytes)?;
    let key_len = u16::from_be_bytes(key_len_bytes);

    let mut huff_len_bytes: [u8; 8] = [0; 8];
    f.read_exact(&mut huff_len_bytes)?;
    let huff_len = u64::from_be_bytes(huff_len_bytes);

    let mut bytes: [u8; 9] = [0; 9];
    let num_byte_codes = (key_len - 10) / 9;

    while f.read_exact(&mut bytes).is_ok() {
        let byte = bytes[0];
        let freq_bytes = &bytes[1..9];
        let freq = u64::from_be_bytes(freq_bytes.try_into().unwrap());
        byte_frequency.insert(byte, freq);
        if byte_frequency.len() as u16 >= num_byte_codes {
            break;
        }
    }

    Ok(HuffKey {
        key_len,
        huff_len,
        byte_frequency,
    })
}

/// A struct that holds the key that was generated when a file was compressed
///
/// # Fields
///
/// * `key_len` - The length of the key in bytes.
///
/// * `huff_len` - The length of the encoded file in bits.
///
/// * `byte_frequency` - A HashMap that maps a byte to it's frequency in the file.
struct HuffKey {
    key_len: u16,
    huff_len: u64,
    byte_frequency: HashMap<u8, u64>,
}

/// A struct that holds the Huffman code for a byte
///
/// # Fields
///
/// * `byte` - The byte that the code represents.
///
/// * `code` - The Huffman code for the byte.
///
/// * `length` - The length of the Huffman code in bits
struct HuffCode {
    byte: u8,
    code: usize,
    length: usize,
}

/// A struct that holds a node in a Huffman tree
///
/// # Fields
///
/// * `code` - The Huffman code for the byte, only relevant for leaf nodes.
///
/// * `frequency` - The frequency of the byte in the file, only relevant for leaf nodes.
///
/// * `left` - The left child of the node.
///
/// * `right` - The right child of the node.
///
/// # Methods
///
/// * `new` - Creates a new leaf node.
///
/// * `build_huff_tree` - Builds a Huffman tree from a HashMap that maps a byte to it's frequency.
///
/// * `is_leaf` - Returns true if the node is a leaf node.
///
/// * `combine` - Combines two nodes into a new node.
///
/// * `update_codes` - Updates the Huffman codes for all the leaf nodes in the tree.
///
/// * `gather_leaves` - Gathers all the leaf nodes in the tree.
///
/// * `search_tree` - Searches the tree for a byte using a BitVec.
struct HuffNode {
    code: Box<HuffCode>,
    frequency: u64,
    left: Option<Box<HuffNode>>,
    right: Option<Box<HuffNode>>,
}

impl HuffNode {
    /// Creates a new leaf node.
    ///
    /// # Arguments
    ///
    /// * `byte` - The byte that the node represents.
    ///
    /// * `frequency` - The frequency of the byte in the file.
    ///
    /// * `code` - The Huffman code for the byte.
    ///
    /// # Returns
    ///
    /// A new leaf node.
    fn new(byte: u8, frequency: u64, code: usize) -> HuffNode {
        HuffNode {
            code: Box::new(HuffCode {
                byte,
                code,
                length: 0,
            }),
            frequency,
            left: None,
            right: None,
        }
    }

    /// Builds a Huffman tree from a HashMap that maps a byte to its frequency. The tree is built
    /// by creating a leaf node for each byte and frequency pair, sorting the nodes by frequency,
    /// and then combining the two nodes with the smallest frequency into a new node. This process
    /// is repeated until there is only one node left, which is the root node of the tree.
    ///
    /// # Arguments
    ///
    /// * `byte_frequency` - A reference to a HashMap that maps a byte to its frequency.
    ///
    /// # Returns
    ///
    /// The root node of the Huffman tree.
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

    /// Returns true if the node is a leaf node.
    ///
    /// # Returns
    ///
    /// True if the node is a leaf node, false otherwise.
    fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    /// Combines two nodes into a new node.
    ///
    /// # Arguments
    ///
    /// * `left` - The left child of the new node.
    ///
    /// * `right` - The right child of the new node.
    ///
    /// # Returns
    ///
    /// A new node that is the combination of the two input nodes with the 'byte' field set to the
    /// smaller of the two input node bytes.
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

    /// Updates the Huffman codes for all the leaf nodes in the tree.
    ///
    /// # Arguments
    ///
    /// * `c` - The Huffman code for the node, should be 0 for the root node. Shifted by one for each
    /// level of the tree and |'d with 1 for the right child.
    ///
    /// * `l` - The length of the Huffman code for the node, should be 0 for the root node. Incremented
    /// by one for each level of the tree.
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

    /// Gathers all the leaf nodes in the tree.
    ///
    /// # Arguments
    ///
    /// * `leaves` - A mutable reference to a HashMap that maps a byte to it's Huffman code.
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

    /// Searches the tree for a byte using a BitVec. The BitVec should be the encoded file. The
    /// function will return the byte that corresponds to the next Huffman code in the BitVec.
    ///
    /// # Arguments
    ///
    /// * `bit_vec` - A mutable reference to a BitVec that holds the encoded file.
    ///
    /// * `bits` - A mutable reference to the current bit in the BitVec.
    ///
    /// # Returns
    ///
    /// The byte that corresponds to the next Huffman code in the BitVec.
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