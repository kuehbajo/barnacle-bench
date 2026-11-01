/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license as the original main file.

  THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND,
  EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
  OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
  IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
  DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
  OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR
  THE USE OR OTHER DEALINGS IN THE SOFTWARE.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite.

  C version: wccmalloc.c

*/

#[derive(Clone, Copy)]
pub struct HeapSlice {
    pub start: u16,
    pub end: u16,
    pub type_size: u8,
}

impl HeapSlice {
    pub const fn new() -> Self {
        HeapSlice {
            start: 0,
            end: 0,
            type_size: 0,
        }
    }
}

// Lowered size to fit in .bss
pub struct SimulatedHeap {
    pub heap_content: [u8; 20000],
    pub free_pos: u16,
}

impl Default for SimulatedHeap {
    fn default() -> Self {
        SimulatedHeap {
            heap_content: [0; 20000],
            free_pos: 0,
        }
    }
}

impl SimulatedHeap {
    const HEAP_SIZE: usize = 20000;

    pub const fn new() -> Self {
        SimulatedHeap {
            heap_content: [0; Self::HEAP_SIZE],
            free_pos: 0,
            // allocs: [None; Self::HEAP_SIZE],
        }
    }

    pub fn malloc(&mut self, number_of_bytes: u16, type_size: u8) -> HeapSlice {
        // Get a 4-byte adress for alignment purposes
        let offset = self.free_pos % 4;
        if offset > 0 { self.free_pos += 4 - offset; }
        let start = self.free_pos;
        let end = self.free_pos + number_of_bytes;
        self.free_pos += number_of_bytes;
        HeapSlice { start, end, type_size }
    }

    pub fn free_all(&mut self) {
        self.free_pos = 0;
        // self.allocs = [None; 30000];
    }

    pub fn return_slice(&mut self, slice: HeapSlice) -> &mut [u8] {
        &mut self.heap_content[(slice.start as usize)..(slice.end as usize)]
    }

    // SAFETY: i8 and u8 have the same alignment and width -> no data corruption
    pub fn return_slice_i8(&mut self, slice: HeapSlice) -> &mut [i8] {
        let curr_u8 = &mut self.heap_content[(slice.start as usize)..(slice.end as usize)];
        unsafe {
            let ptr = curr_u8.as_mut_ptr() as *mut i8;
            core::slice::from_raw_parts_mut(ptr, (slice.end - slice.start) as usize)
        }
    }

    pub fn memcpy(&mut self, src: HeapSlice, dst: HeapSlice) {
        // loopbound min 76 max 76
        self.heap_content.copy_within((src.start as usize)..(src.end as usize), dst.start as usize);
    }

    pub fn reset(&mut self) {
        self.heap_content = [0; Self::HEAP_SIZE];
        self.free_pos = 0;
    }
}

pub fn memset<T: Clone>(ary: &mut [T], val: T) {
    // loopbound min 7220 max 7220 -> for use with x_size * y_size slices
    ary.fill(val);
}
