/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license as its origin.

  THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND,
  EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
  OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
  IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
  DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
  OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR
  THE USE OR OTHER DEALINGS IN THE SOFTWARE.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.
  
  Version V 2.0

  Name: huff_dec

  Author: David Bourgin (David.Bourgin@ufrima.imag.fr)

  Function: Example of Huffman decoding

  Source: ftp://turing.imag.fr/pub/compression/ (1994-09-26)

  Original name: dcodhuff.c

  Changes: I/O to char arrays instead of file i/o.
           Dynamic memory allocation replaced by array.

  License:

  The source code files (codrl1.c, dcodrl1.c, codrle2.c, dcodrle2.c, codrle3.c,
  dcodrle3.c, codrle4.c, dcodrle4.c, codhuff.c, dcodhuff.c) are copyrighted.
  They have been uploaded on ftp in turing.imag.fr (129.88.31.7):/pub/compression
  on 22/5/94 and have been modified on 22/9/94.
  (c) David Bourgin - 1994
  The source codes I provide have no buggs (!) but being that I make them
  available for free I have some notes to make. They can change at any time
  without notice. I assume no responsability or liability for any errors or
  inaccurracies, make no warranty of any kind (express, implied or statutory)
  with respect to this publication and expressly disclaim any and all warranties
  of merchantability, fitness for particular purposes. Of course, if you have
  some problems to use the information presented here, I will try to help you if
  I can.

  If you include the source codes in your application, here are the conditions:
  - You have to put my name in the header of your source file (not in the
  excutable program if you don't want) (this item is a must)
  - I would like to see your resulting application, if possible (this item is not
  a must, because some applications must remain secret)
  - Whenever you gain money with your application, I would like to receive a very
  little part in order to be encouraged to update my source codes and to develop
  new schemes (this item is not a must)

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

/*
  Declaration of types
*/

type NodeIndex = usize;

#[derive(Default, Copy, Clone)]
struct Node {
    byte: u32, /* A byte has to be coded as an unsigned integer to allow a node to have a value over 255 */
    left: Option<NodeIndex>,
    right: Option<NodeIndex>,
}

impl Node {
    const fn new() -> Self {
        Node {
            byte: 0,
            left: None,
            right: None,
        }
    }
}

#[derive(Default, Clone, Copy)]
struct BinVal {
    bits: [u8; 32],
    bits_nb: u32,
    presence: u8,
}

impl BinVal {
    const fn new() -> Self {
        BinVal {
            bits: [0; 32],
            bits_nb: 0,
            presence: 0,
        }
    }
}

/*
  Initialization- and return-value-related functions
*/

const PLAINTEXT_LEN: usize = 600;
const PLAINTEXT: [u8; PLAINTEXT_LEN] =
    *b"You are doubtless asking \"How can I reduce the data size without losing \
    some informations?\". It's easy to answer to this question. I'll only take \
    an example. I'm sure you have heard about the morse. This system established \
    in the 19th century use a scheme very close to the huffman one. In the morse \
    you encode the letters to transmit with two kinds of signs. If you encode \
    these two sign possibilities in one bit, the symbol 'e' is transmitted in a \
    single bit and the symbols 'y' and 'z' need four bits. Look at the symbols \
    in the text you are reading, you'll fast understand the compression ratio..";

const ENCODED_LEN: usize = 419;
const ENCODED: [u8; ENCODED_LEN] = [
    128, 0, 0, 0, 80, 133, 32, 32, 128, 100, 4, 32, 63, 239, 255, 240,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    4, 7, 167, 21, 129, 232, 69, 120, 132, 217, 20, 162, 19, 164, 39, 133,
    252, 138, 105, 20, 194, 19, 129, 240, 172, 138, 248, 150, 11, 11, 240, 201,
    68, 64, 114, 53, 17, 42, 37, 195, 128, 212, 116, 194, 41, 98, 52, 51,
    12, 132, 112, 244, 3, 36, 33, 52, 39, 135, 164, 33, 62, 156, 87, 14,
    110, 22, 87, 50, 85, 198, 99, 142, 140, 194, 81, 78, 158, 84, 129, 254,
    129, 248, 110, 179, 159, 192, 145, 133, 184, 184, 28, 210, 96, 146, 73, 10,
    226, 21, 83, 152, 74, 13, 111, 132, 199, 202, 219, 241, 74, 193, 167, 105,
    222, 31, 147, 6, 55, 31, 129, 40, 232, 52, 153, 160, 148, 18, 36, 197,
    45, 216, 202, 86, 30, 31, 177, 90, 133, 138, 248, 23, 81, 195, 160, 100,
    215, 93, 50, 185, 225, 251, 23, 6, 230, 225, 229, 112, 71, 80, 96, 141,
    205, 176, 230, 85, 196, 9, 24, 93, 90, 121, 225, 76, 68, 152, 63, 25,
    107, 140, 101, 204, 214, 77, 26, 194, 96, 18, 48, 77, 210, 137, 1, 253,
    4, 230, 248, 56, 240, 224, 111, 163, 95, 10, 12, 223, 7, 234, 167, 129,
    40, 36, 96, 135, 125, 245, 250, 2, 198, 120, 127, 0, 145, 133, 213, 167,
    135, 149, 195, 67, 235, 108, 9, 24, 87, 17, 102, 152, 37, 4, 222, 131,
    188, 144, 73, 36, 128, 73, 20, 81, 152, 177, 133, 248, 28, 165, 131, 120,
    127, 240, 242, 184, 104, 125, 109, 129, 35, 30, 4, 145, 65, 202, 88, 9,
    138, 103, 44, 205, 100, 167, 24, 152, 11, 24, 51, 37, 66, 9, 24, 31,
    174, 202, 212, 49, 152, 18, 96, 155, 208, 119, 146, 45, 97, 48, 56, 28,
    194, 90, 224, 204, 144, 232, 176, 36, 96, 126, 187, 43, 83, 12, 121, 129,
    209, 96, 197, 35, 2, 54, 176, 249, 92, 208, 204, 145, 188, 41, 170, 180,
    71, 16, 36, 96, 126, 187, 43, 83, 19, 0, 145, 129, 100, 209, 15, 43,
    135, 55, 6, 238, 180, 194, 90, 17, 229, 115, 21, 168, 251, 140, 131, 162,
    217, 166, 93, 22, 4, 140, 31, 91, 166, 55, 25, 202, 192, 111, 20, 171,
    207, 39, 192,
];

fn huff_dec_init(
    input_pos: &mut usize,
    output_pos: &mut usize,
    byte_nb_to_read: &mut u8,
    val_to_read: &mut u32,
    encoding_table: &mut [BinVal; 257],
    heap: &mut [Node; 514],
) {
    *input_pos = 0;
    *output_pos = 0;

    *byte_nb_to_read = 0;
    *val_to_read = 0;
    *encoding_table = [BinVal::default(); 257];
    *heap = [Node::default(); 514];

}

fn huff_dec_return(output: &mut [u8; 1024]) -> i32 {
    // loopbound min 600 max 600
    for (i, (e1, e2)) in (PLAINTEXT.iter().zip(output.iter())).enumerate() {
        if e1 != e2 { return (i + 1) as i32; }
    }
    0
}

/*
  Input / output functions
*/

fn huff_dec_end_of_data(input_pos: usize) -> bool {
    input_pos >= ENCODED_LEN
}

fn huff_dec_read_byte(input_pos: &mut usize) -> u8 {
    let res = ENCODED[*input_pos];
    *input_pos += 1;
    res
}

fn huff_dec_write_byte(ch: u8, output_pos: &mut usize, output: &mut [u8]) {
    output[*output_pos] = ch;
    *output_pos += 1;
}

fn huff_dec_read_code_1_bit(
    input_pos: &mut usize,
    byte_nb_to_read: &mut u8,
    val_to_read: &mut u32,
) -> u8 {
/* Returned parameters: Returns an unsigned integer with the 0-bit (on the
   right of the integer) valid
   Action: Reads the next bit in the stream of data to compress
   Errors: An input/output error could disturb the running of the program
   The source must have enough bits to read
*/
    if *byte_nb_to_read > 0 {
        *byte_nb_to_read -= 1;
        ((*val_to_read >> *byte_nb_to_read) & 1) as u8
    } else {
        *val_to_read = huff_dec_read_byte(input_pos) as u32;
        *byte_nb_to_read = 7;
        ((*val_to_read >> 7) & 1) as u8
    }
}

fn huff_dec_read_code_n_bits(
    n: u8,
    input_pos: &mut usize,
    byte_nb_to_read: &mut u8,
    val_to_read: &mut u32,
) -> u32 {
/* Returned parameters: Returns an unsigned integer with the n-bits (on the
                        right of the integer) valid
   Action: Reads the next n bits in the stream of data to compress
   Errors: An input/output error could disturb the running of the program
   The source must have enough bits to read
*/
    let mut result: u32 = 0;

    // loopbound min 1 max 1
    let mut i = n;
    while i > 0 {
        // loopbound min 0 max 2
        while (*byte_nb_to_read < 9) && !huff_dec_end_of_data(*input_pos) {
            *val_to_read = (*val_to_read << 8) + huff_dec_read_byte(input_pos) as u32;
            *byte_nb_to_read += 8;
        }
        if i >= *byte_nb_to_read {
            result = (result << *byte_nb_to_read) + *val_to_read;
            i -= *byte_nb_to_read;
            *byte_nb_to_read = 0;
        } else {
            result = (result << i) + ((*val_to_read >> (*byte_nb_to_read - i)) & ((1 << i) - 1));
            *byte_nb_to_read -= i;
            i = 0;
        }
    }
    result
}

fn huff_dec_read_header(
    codes_table: &mut [BinVal; 257],
    input_pos: &mut usize,
    byte_nb_to_read: &mut u8,
    val_to_read: &mut u32,
) {
/* Returned parameters: The contain of 'codes_table' is modified
   Action: Rebuilds the binary encoding array by using the header
   Errors: An input/output error could disturb the running of the program
*/
    // loopbound min 257 max 257
    for bin_val in codes_table.iter_mut() {
        bin_val.bits_nb = 0;
        // loopbound min 32 max 32
        for bit in bin_val.bits.iter_mut() {
            *bit = 0;
        }
    }

    /* == Decoding of the first part of the header === */
    if huff_dec_read_code_1_bit(input_pos, byte_nb_to_read, val_to_read) > 0 {
        /* First bit=0 => Present bytes coded on n*8 bits
                    =1 => Present bytes coded on 256 bits */
        // loopbound min 256 max 256
        let end = codes_table.len() - 1;
        for bin_val in codes_table[..end].iter_mut() {
            bin_val.presence = huff_dec_read_code_1_bit(input_pos, byte_nb_to_read, val_to_read);
        }
    } else {
        let mut i = huff_dec_read_code_n_bits(5, input_pos, byte_nb_to_read, val_to_read) + 1;
        // loopbound min 1 max 32
        while i > 0 {
            codes_table[huff_dec_read_code_n_bits(8, input_pos, byte_nb_to_read, val_to_read) as usize].presence = 1;
            i -= 1;
        }
    }
    codes_table[256].presence = 1;
    /* Presence of a fictive 256-byte is enforced! */

    /* == Decoding the second part of the header == */
    // loopbound min 257 max 257
    for bin_val in codes_table.iter_mut() {
        if bin_val.presence > 0 {
            let mut j;
            if huff_dec_read_code_1_bit(input_pos, byte_nb_to_read, val_to_read) > 0 {
                /* First bit=0 => 5 bits of binary length-1 followed by a binary word
                            =1 => 8 bits of binary length-1 followed by a binary word */
                j = huff_dec_read_code_n_bits(8, input_pos, byte_nb_to_read, val_to_read) + 1;
            }
            else {
                j = huff_dec_read_code_n_bits(5, input_pos, byte_nb_to_read, val_to_read) + 1;
            }
            bin_val.bits_nb = j;
            /* Reading of a binary word */
            let mut num_byte: usize = ((j - 1) >> 3) as usize;
            if (j & 7) > 0 {
                /* Reads the bits that takes less than one byte */
                bin_val.bits[num_byte] = huff_dec_read_code_n_bits((j & 7) as u8, input_pos, byte_nb_to_read, val_to_read) as u8;
                j -= j & 7;
                num_byte -= 1;
            }

            // loopbound min 0 max 1
            while j >= 8 {
                /* Reads the bits that takes one byte, at least */
                bin_val.bits[num_byte] = huff_dec_read_code_n_bits(8, input_pos, byte_nb_to_read, val_to_read) as u8;
                j -= 8;
                num_byte -= 1;
            }
        }
    }
}

fn huff_dec_tree_encoding(
    codes_table: &mut [BinVal; 257],
    heap: &mut [Node; 514],
) {
/* Returned parameters: A binary tree is returned
   Action: Returns the decoding binary tree built from 'codes_table'
   Errors: None
*/
    let mut heap_top: NodeIndex = 0;
    let start_tree: NodeIndex = 0;

    heap[heap_top].byte = 257;
    heap[heap_top].left = None;
    heap[heap_top].right = None;
    heap_top += 1;
    // loopbound min 257 max 257
    for (i, bin_val) in codes_table.iter_mut().enumerate() {
        let mut current_node = start_tree;
        // loopbound min 0 max 9
        for j in (1..=bin_val.bits_nb).rev() {
            if bin_val.bits[((j - 1) >> 3) as usize] & (1 << ((j - 1) & 7)) != 0 {
                if heap[current_node].left.is_none() {
                    heap[current_node].left = Some(heap_top);
                    heap_top += 1;
                    current_node = heap[current_node].left.unwrap();
                    heap[current_node].left = None;
                    heap[current_node].right = None;
                } else {
                    current_node = heap[current_node].left.unwrap();
                }
            } else if heap[current_node].right.is_none() {
                heap[current_node].right = Some(heap_top);
                heap_top += 1;
                current_node = heap[current_node].right.unwrap();
                heap[current_node].left = None;
                heap[current_node].right = None;
            } else {
                current_node = heap[current_node].right.unwrap();
            }
            if j == 1 {
                heap[current_node].byte = i as u32;
            } else {
                heap[current_node].byte = 257;
            }
        }
    };
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn huff_dec_main(
    input_pos: &mut usize,
    output_pos: &mut usize,
    output: &mut [u8; 1024],
    byte_nb_to_read: &mut u8,
    val_to_read: &mut u32,
    encoding_table: &mut [BinVal; 257],
    heap: &mut [Node; 514]
) {
/* Returned parameters: None
   Action: Decompresses with Huffman method all bytes read by the function
           'read_code_1_bit' and 'read_code_n_bits'
   Errors: An input/output error could disturb the running of the program
*/
    let tree_root = 0;

    if !huff_dec_end_of_data(*input_pos) { /* Are there data to compress? */
        huff_dec_read_header(encoding_table, input_pos, byte_nb_to_read, val_to_read);
        huff_dec_tree_encoding(encoding_table, heap);
        // loopbound min 601 max 601
        loop {
            let mut current_node = tree_root;
            // loopbound min 3 max 9
            while heap[current_node].byte == 257 {
                if huff_dec_read_code_1_bit(input_pos, byte_nb_to_read, val_to_read) > 0 {
                    /* Bit=1 => Got to left in the node of the tree
                          =0 => Got to right in the node of the tree */
                    current_node = heap[current_node].left.unwrap();
                } else {
                    current_node = heap[current_node].right.unwrap();
                }
            }
            if heap[current_node].byte <= 255 {
                huff_dec_write_byte(heap[current_node].byte as u8, output_pos, output);
            } else if heap[current_node].byte == 256 {
                break;
            }
        }
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn huff_dec_log(
    input_pos: &mut usize,
    output_pos: &mut usize,
    output: &mut [u8; 1024],
    byte_nb_to_read: &mut u8,
    val_to_read: &mut u32,
    encoding_table: &mut [BinVal; 257],
    heap: &mut [Node; 514],
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        huff_dec_init(
            black_box(input_pos), black_box(output_pos),
            black_box(byte_nb_to_read), black_box(val_to_read),
            black_box(encoding_table), black_box(heap),
        );
        let start = black_box(DWT::cycle_count());
        huff_dec_main(
            black_box(input_pos),
            black_box(output_pos), black_box(output),
            black_box(byte_nb_to_read), black_box(val_to_read),
            black_box(encoding_table), black_box(heap),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("huff_dec - elapsed: {}", cycles);
    });

    black_box(huff_dec_return(black_box(output))) == 0
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn huff_dec_std(
    input_pos: &mut usize,
    output_pos: &mut usize,
    output: &mut [u8; 1024],
    byte_nb_to_read: &mut u8,
    val_to_read: &mut u32,
    encoding_table: &mut [BinVal; 257],
    heap: &mut [Node; 514],
) -> bool {
    huff_dec_init(
        black_box(input_pos), black_box(output_pos),
        black_box(byte_nb_to_read), black_box(val_to_read),
        black_box(encoding_table), black_box(heap),
    );
    huff_dec_main(
        black_box(input_pos),
        black_box(output_pos), black_box(output),
        black_box(byte_nb_to_read), black_box(val_to_read),
        black_box(encoding_table), black_box(heap),
    );
    black_box(huff_dec_return(black_box(output))) == 0
}

fn huff_dec() -> bool {
    static mut INPUT_POS: usize = 0;
    static mut OUTPUT_POS: usize = 0;
    static mut OUTPUT: [u8; 1024] = [0; 1024];
    static mut BYTE_NB_TO_READ: u8 = 0;
    static mut VAL_TO_READ: u32 = 0;
    static mut ENCODING_TABLE: [BinVal; 257] = [BinVal::new(); 257];
    static mut HEAP: [Node; 514] = [Node::new(); 514]; /* space for dynamically allocated nodes */

    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let input_pos = unsafe { &mut *{ &raw mut INPUT_POS } };
    let output_pos = unsafe { &mut *{ &raw mut OUTPUT_POS } };
    let output = unsafe { &mut *{ &raw mut OUTPUT } };
    let byte_nb_to_read = unsafe { &mut *{ &raw mut BYTE_NB_TO_READ } };
    let val_to_read = unsafe { &mut *{ &raw mut VAL_TO_READ } };
    let encoding_table = unsafe { &mut *{ &raw mut ENCODING_TABLE } };
    let heap = unsafe { &mut *{ &raw mut HEAP } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(huff_dec_log(
            black_box(input_pos),
            black_box(output_pos), black_box(output),
            black_box(byte_nb_to_read), black_box(val_to_read),
            black_box(encoding_table), black_box(heap),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(huff_dec_std(
            black_box(input_pos),
            black_box(output_pos), black_box(output),
            black_box(byte_nb_to_read), black_box(val_to_read),
            black_box(encoding_table), black_box(heap),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(huff_dec());
    
    #[cfg(feature = "log")]
    {
        rprintln!("huff_dec: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(huff_dec());

    #[cfg(feature = "log")]
    std::println!("huff_dec: {}", success);

    std::process::exit(!success as i32);
}

// FIXME: has a stack usage problem -> check C version
// 33264
