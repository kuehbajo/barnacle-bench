/*

  Rust Implementation
  The alterations are the reimplementation in the Rust programming language.

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

  Version 2.0

  Name: anagram_stdlib.c

  Author: Raymond Chen

  Function: This file contains the C standard library functions used by anagram.

  Source: anagram

  Original name: anagram

  Changes: See ChangeLog.txt

  License: See anagram.c

*/

const HEAP_SIZE: usize = 30000;     // original size: 21000

#[derive(Default, Clone, Copy, PartialEq)]
pub struct HeapSlice {
    pub start: usize,
    pub end: usize,
    pub type_size: usize,
}

impl PartialOrd for HeapSlice {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        if self.end < other.start {
            return Some(core::cmp::Ordering::Less);
        } else if self.start > other.start {
            return Some(core::cmp::Ordering::Greater);
        }
        None
    }
}

impl HeapSlice {
    pub const fn new() -> Self {
        HeapSlice {
            start: 0,
            end: 0,
            type_size: 0,
        }
    }

    pub fn to_ne_bytes(&self) -> [u8; size_of::<HeapSlice>()] {
        let mut bytes = [0u8; size_of::<HeapSlice>()];

        let start_bytes = self.start.to_ne_bytes();
        let end_bytes = self.end.to_ne_bytes();
        let type_size_bytes = self.type_size.to_ne_bytes();

        bytes[..size_of::<usize>()].copy_from_slice(&start_bytes);
        bytes[size_of::<usize>()..(size_of::<usize>() * 2)].copy_from_slice(&end_bytes);
        bytes[(size_of::<usize>() * 2)..].copy_from_slice(&type_size_bytes);

        bytes
    }
}

pub struct SimulatedHeap {
    pub heap_content: [u8; HEAP_SIZE],
    pub free_pos: usize,
}

impl Default for SimulatedHeap {
    fn default() -> Self {
        SimulatedHeap {
            heap_content: [0; HEAP_SIZE],
            free_pos: 0,
        }
    }
}

impl SimulatedHeap {
    pub const fn new() -> Self {
        SimulatedHeap {
            heap_content: [0; HEAP_SIZE],
            free_pos: 0,
        }
    }

    // this works slighty different than the original to be consistent with other implementations
    // here: misalignment gets resolved at the start of a new malloc
    // original: misalignment gets resolved at the end of old malloc
    pub fn malloc(&mut self, number_of_bytes: usize, type_size: usize) -> HeapSlice {
        // Get a 4-byte adress for alignment purposes
        let offset = self.free_pos % 4;
        if offset > 0 { self.free_pos += 4 - offset; }
        let start = self.free_pos;
        let end = self.free_pos + number_of_bytes;
        self.free_pos += number_of_bytes;
        HeapSlice { start, end, type_size }
    }

    pub fn reset(&mut self) {
        self.heap_content = [0; HEAP_SIZE];
        self.free_pos = 0;
    }
}
