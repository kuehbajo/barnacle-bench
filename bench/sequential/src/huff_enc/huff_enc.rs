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

  Name: huff_enc

  Author: David Bourgin (David.Bourgin@ufrima.imag.fr)

  Function: Example of Huffman encoding

  Source: ftp://turing.imag.fr/pub/compression/ (1994-09-22)

  Original name: codhuff.c

  Changes: I/O to char arrays instead of file i/o.
           Dynamic memory allocation replaced by array.
           Explicit sorting algorithm.

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
    weight: u64,
    left: Option<NodeIndex>,
    right: Option<NodeIndex>,
}

impl Node {
    const fn new() -> Self {
        Node {
            byte: 0,
            weight: 0,
            left: None,
            right: None,
        }
    }
}

#[derive(Default, Clone, Copy)]
struct BinVal {
    bits: [u8; 32],
    bits_nb: u32,
}

impl BinVal {
    const fn new() -> Self {
        BinVal {
            bits: [0; 32],
            bits_nb: 0,
        }
    }
}

// this is adapted from the quicksort kernel version
mod quicksort {
    use crate::Node;

    #[derive(PartialEq)]
    enum Compare { Lower, Equals, Greater }

    fn compare_weights(w1: u64, w2: u64) -> Compare {
    /* Returned parameters: Returns a comparison status
    Action: Returns a negative, zero or positive integer depending on the weight
            of 'tree2' is less than, equal to, or greater than the weight of
            'tree1'
    Errors: None
    */    
        if (w1 ^ w2) > 0 {
            if w2 < w1 {
                Compare::Lower
            } else {
                Compare::Greater
            }
        } else {
            Compare::Equals
        }
    }

    fn pivot(arr: &[usize], weight_arr: &[Node]) -> usize {
        let i = arr.len() / 6;
        let si = weight_arr[arr[i]].weight; // 1/6

        let j = 3 * i;
        let sj = weight_arr[arr[j]].weight; // 1/2

        let k = 5 * i;
        let sk = weight_arr[arr[k]].weight; // 5/6

        if compare_weights(si, sj) == Compare::Lower {
            if compare_weights(si, sk) == Compare::Lower {
                if compare_weights(sj, sk) == Compare::Lower {
                    return j;
                }
                return k;
            }
            return i;
        }

        if compare_weights(sj, sk) == Compare::Lower {
            if compare_weights(si, sk) == Compare::Lower {
                return i;
            }
            return k;
        }
        j
    }

    // in total 648 (recursion)
    fn sorts(mut arr: &mut [usize], weight_arr: &[Node]) {
        // loopbound min 0 max 7
        while arr.len() > 1 {
            let pivot = if arr.len() > 10 {
                pivot(arr, weight_arr)
            } else {
                arr.len() >> 1
            };

            arr.swap(0, pivot);
            let mut pi = 0;
            let mut pj = arr.len();
            let pn = pj;

            // loopbound min 0 max 109
            loop {
                // loopbound min 1 max 19
                loop {
                    pi += 1;

                    if !(pi < pn && compare_weights(weight_arr[arr[pi]].weight, weight_arr[arr[0]].weight) == Compare::Lower) {
                        break;
                    }
                }
                // loopbound min 1 max 25
                loop {
                    pj -= 1;

                    if !(pj > 0 && compare_weights(weight_arr[arr[pj]].weight, weight_arr[arr[0]].weight) == Compare::Greater) {
                        break;
                    }
                }

                if pj < pi {
                    break;
                }

                arr.swap(pi, pj);
            }

            arr.swap(0, pj);

            let n = arr.len() - pj - 1;
            if pj >= n {
                sorts(&mut arr[..pj], weight_arr);
                arr = &mut arr[pj + 1..]
            } else {
                sorts(&mut arr[pj + 1..], weight_arr);
                arr = &mut arr[..pj];
            }
        }
    }

    pub fn sort(arr: &mut [usize], weight_arr: &[Node]) {  
        sorts(arr, weight_arr);
    }
}

use quicksort::sort;

/*
  Initialization- and return-value-related functions
*/

// removed last period as the original had byte length of 601
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

fn huff_enc_init(
    input_pos: &mut usize,
    output_pos: &mut usize,
    encoding_table: &mut [BinVal; 257],
    byte_nb_to_write: &mut u8,
    val_to_write: &mut u8,
    heap: &mut [Node; 514],
) {
    *input_pos = 0;
    *output_pos = 0;
    *encoding_table = [BinVal::default(); 257];
    *val_to_write = 0;
    *byte_nb_to_write = 0;
    *heap = [Node::default(); 514];
}

fn huff_enc_return(output: &[u8; 1024]) -> i32 {
    // loopbound min 419 max 419
    for (i, (e1, e2)) in (ENCODED.iter().zip(output.iter())).enumerate() {
        if e1 != e2 { return (i + 1) as i32; }
    }
    0
}

/*
  Input / output functions
*/

fn huff_enc_beginning_of_data(input_pos: &mut usize) {
    *input_pos = 0;
}

fn huff_enc_end_of_data(input_pos: usize) -> bool {
    input_pos >= PLAINTEXT_LEN
}

fn huff_enc_read_byte(input_pos: &mut usize) -> u8 {
    let res = PLAINTEXT[*input_pos];
    *input_pos += 1;
    res
}

fn huff_enc_write_byte(ch: u8 , output: &mut [u8], output_pos: &mut usize) {
    output[*output_pos] = ch;
    *output_pos += 1;
}

fn huff_enc_write_bin_val(
    bin_val: &mut BinVal,
    output: &mut [u8],
    output_pos: &mut usize,
    val_to_write: &mut u8,
    byte_nb_to_write: &mut u8,
) {
/* Returned parameters: None
   Action: Writes in the output stream the value binary-coded into 'bin_val'
   Errors: An input/output error could disturb the running of the program
*/
    let mut bin_pos: u32 = (bin_val.bits_nb - 1) & 7;
    let mut pos_byte: usize = ((bin_val.bits_nb - 1) >> 3) as usize;

    // loopbound min 1 max 9
    for _ in 1..=bin_val.bits_nb {
        /* Watch for the current bit to write */
        *val_to_write = (*val_to_write << 1 ) | ( ( bin_val.bits[ pos_byte ] >> bin_pos ) & 1 );
        /* Move to the next bit to write */
        if bin_pos == 0 {
            pos_byte -= 1;
            bin_pos = 7;
        } else {
            bin_pos -= 1;
        }
        if *byte_nb_to_write == 7 {
            /* Are already 8 bits written? */
            huff_enc_write_byte(*val_to_write, output, output_pos);
            *byte_nb_to_write = 0;
            *val_to_write = 0;
        } else {
            /* No, then the next writting will be in the next bit */
            *byte_nb_to_write += 1;
        } 
    }
}

fn huff_enc_fill_encoding(
    output: &mut [u8],
    output_pos: &mut usize,
    val_to_write: &mut u8,
    byte_nb_to_write: &mut u8,
) {
/* Returned parameters: None
   Action: Fills the last byte to write in the output stream with zero values
   Errors: An input/output error could disturb the running of the program
*/
    if *byte_nb_to_write > 0 {
        huff_enc_write_byte(*val_to_write << (8 - *byte_nb_to_write), output, output_pos);
    }
}

fn huff_enc_write_header(
    codes_table: &mut [BinVal; 257],
    output_pos: &mut usize,
    output: &mut [u8],
    val_to_write: &mut u8,
    byte_nb_to_write: &mut u8,
) {
/* Returned parameters: None
   Action: Writes the header in the stream of codes
   Errors: An input/output error could disturb the running of the program
*/
    let mut bin_val_to_0 = BinVal::default();
    let mut bin_val_to_1 = BinVal::default();
    let mut bin_val = BinVal::default();

    /* Is used to send in binary mode via huff_enc_write_bin_val */

    bin_val_to_0.bits = [0; 32];
    bin_val_to_0.bits_nb = 1;
    bin_val_to_1.bits[0] = 1;
    bin_val_to_1.bits_nb = 1;
    // loopbound min 256 max 256
    let mut i = 0;
    for j in 0..=255 {
        if codes_table[j].bits_nb > 0 { i += 1 }
    }
    /* From there, i contains the number of bytes of the several
      non 0 occurrences to encode.
      First part of the header: Specifies the bytes that appear
      in the source of encoding */
    if i < 32 {
        /* Encoding of the appeared bytes with a block of bytes */
        huff_enc_write_bin_val(
            &mut bin_val_to_0, output, output_pos, val_to_write, byte_nb_to_write,
        );
        bin_val.bits_nb = 5;
        bin_val.bits[0] = i - 1;
        huff_enc_write_bin_val(
            &mut bin_val, output, output_pos, val_to_write, byte_nb_to_write,
        );
        bin_val.bits_nb = 8;
        // loopbound min 256 max 256
        for j in 0..=255 {
            if codes_table[j].bits_nb > 0 {
                bin_val.bits[0] = j as u8;
                huff_enc_write_bin_val(
                    &mut bin_val, output, output_pos, val_to_write, byte_nb_to_write,
                );
            }
        } 
    } else {
        /* Encoding of the appeared bytes with a block of bits */
        huff_enc_write_bin_val(
            &mut bin_val_to_1, output, output_pos, val_to_write, byte_nb_to_write,
        );
        // loopbound min 256 max 256
        for j in 0..=255 {
            if codes_table[j].bits_nb > 0 {
                huff_enc_write_bin_val(
                    &mut bin_val_to_1, output, output_pos, val_to_write, byte_nb_to_write,
                );
            } else {
                huff_enc_write_bin_val(
                    &mut bin_val_to_0, output, output_pos, val_to_write, byte_nb_to_write,
                );
            }
        }
    };
    /* Second part of the header: Specifies the encoding of the bytes
      (fictive or not) that appear in the source of encoding */
    // loopbound min 257 max 257
    for el in codes_table.iter_mut() {
        let j = el.bits_nb;
        if j != 0 {
            if j < 33 {
                huff_enc_write_bin_val(
                    &mut bin_val_to_0, output, output_pos, val_to_write, byte_nb_to_write,
                );
                bin_val.bits_nb = 5;
            } else {
                huff_enc_write_bin_val(
                    &mut bin_val_to_1, output, output_pos, val_to_write, byte_nb_to_write,
                );
                bin_val.bits_nb = 8;
            }
            bin_val.bits[0] = (j - 1) as u8; 
            huff_enc_write_bin_val(
                &mut bin_val, output, output_pos, val_to_write, byte_nb_to_write,
            );
            huff_enc_write_bin_val(
                el, output, output_pos, val_to_write, byte_nb_to_write,
            );
        }
    }
}

fn huff_enc_build_tree_encoding(
    heap: &mut [Node; 514],
    input_pos: &mut usize,
) -> [NodeIndex; 257] {
/* Returned parameters: Returns a tree of encoding
   Action: Generates an Huffman encoding tree based on the data from
           the stream to compress
   Errors: None
*/
    let mut heap_top = 0;
    let mut occurrences_table: [NodeIndex; 257] = [0; 257];

    /* Sets up the occurrences number of all bytes to 0 */
    // loopbound min 257 max 257
    for (i, occ) in occurrences_table.iter_mut().enumerate() {
        *occ = heap_top;
        heap[heap_top] = Node {
            byte: i as u32,
            weight: 0,
            left: None,
            right: None,
        };
        heap_top += 1;
    }

    /* Valids the occurrences of 'occurrences_table' with regard to the data to
      compress */
    if !huff_enc_end_of_data(*input_pos) {
        // loopbound min 600 max 600
        while !huff_enc_end_of_data(*input_pos) {
            let i = huff_enc_read_byte(input_pos) as usize;
            heap[occurrences_table[i]].weight += 1;
        }
        heap[occurrences_table[256]].weight = 1;

        /* Sorts the occurrences table depending on the weight of each character */
        sort(&mut occurrences_table, heap);

        // loopbound min 218 max 218
        let mut i = 256;
        while i != 0 && heap[occurrences_table[i]].weight == 0 {
            i -= 1;
        }
        i += 1;
        /* From there, 'i' gives the number of different bytes with a 0 occurrence
          in the stream to compress */
        // loopbound min 38 max 38
        while i > 0 {
            /* Looks up (i+1)/2 times the occurrence table to link the nodes in an
              unique tree */
            let fictive_node = heap_top;
            heap_top += 1;
            i -= 1;

            heap[fictive_node] = Node {
                byte: 257,
                weight: heap[occurrences_table[i]].weight,
                left: Some(occurrences_table[i]),
                right: None,
            };
            if i > 0 {
                i -= 1;
                heap[fictive_node].weight += heap[occurrences_table[i]].weight;
                heap[fictive_node].right = Some(occurrences_table[i]);
            }
            occurrences_table[i] = fictive_node;

            sort(&mut occurrences_table[..(i+1)], heap);

            if i > 0 { /* Is there an other node in the occurrence tables? */
                i += 1; /* Yes, then takes care to the fictive node */
            }
        }
    }
    occurrences_table
}

enum NodeSide { Left, Right }

#[inline(always)]
fn huff_enc_encode_helper(
  heap: &mut [Node; 514],
  node_i: NodeIndex,
  codes_table: &mut [BinVal; 257],
  code_val: &mut BinVal,
  side: NodeSide,
) {
    let tmp_code_val = *code_val;
    // loopbound min 31 max 31
    for i in (1..32).rev() {
        code_val.bits[i] = (code_val.bits[i] << 1) | (code_val.bits[i - 1] >> 7)
    }
    match side {
        NodeSide::Left => {
            code_val.bits[0] = (code_val.bits[0] << 1 ) | 1;
            code_val.bits_nb += 1;
            huff_enc_encode_codes_table(heap, heap[node_i].left.unwrap(), codes_table, code_val);
        },
        NodeSide::Right => {
            code_val.bits[0] <<= 1;
            code_val.bits_nb += 1;
            huff_enc_encode_codes_table(heap, heap[node_i].right.unwrap(), codes_table, code_val);
        },
    }
    *code_val = tmp_code_val;
}

fn huff_enc_encode_codes_table(
    heap: &mut [Node; 514],
    node_i: NodeIndex,
    codes_table: &mut [BinVal; 257],
    code_val: &mut BinVal,
) {
/* Returned parameters: The data of 'codes_table' can have been modified
   Action: Stores the encoding tree as a binary encoding table to speed up the
          access. 'val_code' gives the encoding for the current node of the tree
   Errors: None
*/
    if heap[node_i].byte == 257 {


        /* The sub-trees on left begin with an bit set to 1 */
      if heap[node_i].left.is_some() {
          huff_enc_encode_helper(heap, node_i, codes_table, code_val, NodeSide::Left);
      }
      /* The sub-trees on right begin with an bit set to 0 */
      if heap[node_i].right.is_some() {
          huff_enc_encode_helper(heap, node_i, codes_table, code_val, NodeSide::Right);
      }
    } else {
        codes_table[heap[node_i].byte as usize] = *code_val;
    }
}

fn huff_enc_create_codes_table(
    heap: &mut [Node; 514],
    tree_i: usize,
    codes_table: &mut [BinVal; 257],
) {
/* Returned parameters: The data in 'codes_table' will be modified
   Action: Stores the encoding tree as a binary encoding table to speed up
           the access by calling encode_codes_table
   Errors: None
*/
    // loopbound min 32 max 32
    let mut code_val = BinVal::default();
    
    // loopbound min 257 max 257
    for bin_val in codes_table.iter_mut() {
        // loopbound min 32 max 32
        bin_val.bits.fill(0);
    }
    // flowrestriction 1*huff_enc_encode_codes_table <= 77*call_encode
    huff_enc_encode_codes_table(heap, tree_i, codes_table, &mut code_val);
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn huff_enc_main(
    input_pos: &mut usize,
    output_pos: &mut usize,
    output: &mut [u8; 1024],
    val_to_write: &mut u8,
    byte_nb_to_write: &mut u8,
    encoding_table: &mut [BinVal; 257],
    heap: &mut [Node; 514],
) {
/* Returned parameters: None
   Action: Compresses with Huffman method all bytes read by the function
           'huff_enc_read_byte'
   Errors: None
*/
    if !huff_enc_end_of_data(*input_pos) {
        /* Generates only whether there are data */
        let tree_indices = huff_enc_build_tree_encoding(heap, input_pos);

        /* Creation of the best adapted tree */
        huff_enc_create_codes_table(heap, tree_indices[0], encoding_table);

        /* Obtains the binary encoding in an array to speed up the accesses */
        huff_enc_write_header(
            encoding_table, output_pos, output, val_to_write, byte_nb_to_write
        );
        /* Writes the defintion of the encoding */
        huff_enc_beginning_of_data(input_pos);  /* Real compression of the data */
        // loopbound min 600 max 600
        while !huff_enc_end_of_data(*input_pos) {
            let byte_read = huff_enc_read_byte(input_pos);
            huff_enc_write_bin_val(
                &mut encoding_table[byte_read as usize], output, output_pos, val_to_write, byte_nb_to_write,
            );
        }
        huff_enc_write_bin_val(
                &mut encoding_table[256], output, output_pos, val_to_write, byte_nb_to_write,
            );
        /* Code of the end of encoding */
        huff_enc_fill_encoding(
            output, output_pos, val_to_write, byte_nb_to_write,
        );
        /* Fills the last byte before closing file, if any */
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn huff_enc_log(
    input_pos: &mut usize,
    output_pos: &mut usize,
    output: &mut [u8; 1024],
    byte_nb_to_write: &mut u8,
    val_to_write: &mut u8,
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
        huff_enc_init(
            black_box(input_pos), black_box(output_pos),
            black_box(encoding_table), black_box(byte_nb_to_write),
            black_box(val_to_write), black_box(heap),
        );
        let start = black_box(DWT::cycle_count());
        huff_enc_main(
            black_box(input_pos),
            black_box(output_pos), black_box(output),
            black_box(byte_nb_to_write), black_box(val_to_write),
            black_box(encoding_table), black_box(heap),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("huff_enc - elapsed: {}", cycles);
    });

    black_box(huff_enc_return(black_box(output))) == 0
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn huff_enc_std(
    input_pos: &mut usize,
    output_pos: &mut usize,
    output: &mut [u8; 1024],
    byte_nb_to_write: &mut u8,
    val_to_write: &mut u8,
    encoding_table: &mut [BinVal; 257],
    heap: &mut [Node; 514],
) -> bool {
    huff_enc_init(
        black_box(input_pos), black_box(output_pos),
        black_box(encoding_table), black_box(byte_nb_to_write),
        black_box(val_to_write), black_box(heap),
    );
    huff_enc_main(
        black_box(input_pos),
        black_box(output_pos), black_box(output),
        black_box(byte_nb_to_write), black_box(val_to_write),
        black_box(encoding_table), black_box(heap),
    );
    black_box(huff_enc_return(black_box(output))) == 0
}

fn huff_enc() -> bool {
    static mut INPUT_POS: usize = 0;
    static mut OUTPUT_POS: usize = 0;
    static mut OUTPUT: [u8; 1024] = [0; 1024];
    static mut BYTE_NB_TO_WRITE: u8 = 0;
    static mut VAL_TO_WRITE: u8 = 0;
    static mut ENCODING_TABLE: [BinVal; 257] = [BinVal::new(); 257];
    static mut HEAP: [Node; 514] = [Node::new(); 514];

    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let input_pos = unsafe { &mut *{ &raw mut INPUT_POS } };
    let output_pos = unsafe { &mut *{ &raw mut OUTPUT_POS } };
    let output = unsafe { &mut *{ &raw mut OUTPUT } };
    let byte_nb_to_write = unsafe { &mut *{ &raw mut BYTE_NB_TO_WRITE } };
    let val_to_write = unsafe { &mut *{ &raw mut VAL_TO_WRITE } };
    let encoding_table = unsafe { &mut *{ &raw mut ENCODING_TABLE } };
    let heap = unsafe { &mut *{ &raw mut HEAP } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(huff_enc_log(
            black_box(input_pos),
            black_box(output_pos), black_box(output),
            black_box(byte_nb_to_write), black_box(val_to_write),
            black_box(encoding_table), black_box(heap),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(huff_enc_std(
            black_box(input_pos),
            black_box(output_pos), black_box(output),
            black_box(byte_nb_to_write), black_box(val_to_write),
            black_box(encoding_table), black_box(heap),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(huff_enc());
    
    #[cfg(feature = "log")]
    {
        rprintln!("huff_enc: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(huff_enc());

    #[cfg(feature = "log")]
    std::println!("huff_enc: {}", success);

    std::process::exit(!success as i32);
}

// FIXME: has a stack usage problem -> check C version
// 42928
