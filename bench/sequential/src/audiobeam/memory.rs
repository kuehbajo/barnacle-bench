/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license (MIT) as the original main file.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite.

*/

const HEAP_SIZE: usize = 10000;

#[derive(Clone, Copy)]
pub struct HeapSlice {
    pub start: usize,
    pub end: usize,
    pub type_size: usize,
}

impl HeapSlice {
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

    pub fn from_ne_bytes(bytes: [u8; size_of::<HeapSlice>()]) -> HeapSlice {
        let (a, rest) = bytes.split_at(size_of::<usize>());
        let (b, c) = rest.split_at(size_of::<usize>());
        let start = usize::from_ne_bytes(<[u8; size_of::<usize>()]>::try_from(a).unwrap());
        let end = usize::from_ne_bytes(<[u8; size_of::<usize>()]>::try_from(b).unwrap());
        let type_size = usize::from_ne_bytes(<[u8; size_of::<usize>()]>::try_from(c).unwrap());
        HeapSlice { start, end, type_size }
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

    pub fn malloc(&mut self, number_of_bytes: usize, type_size: usize) -> HeapSlice {
        let align = 4;
        // Get a {align}-byte adress for alignment purposes
        let offset = self.free_pos % align;
        if offset > 0 { self.free_pos += align - offset; }
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
