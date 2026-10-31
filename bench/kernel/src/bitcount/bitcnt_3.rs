/*

  Rust Implementation

  Author: David Heim
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

  Version V 1.x

  Name: bitcnt_3.c

  Author: Bob Stout & Auke Reitsma

  Function: Bit counting functions using table lookup

  Source: http://www.snippets.org/

  Changes: no major functional changes

  License: May be used, modified, and re-distributed freely.

*/

use core::hint::black_box;

pub fn bitcount_init3(bits: &mut [u8]) {
    let bits_tmp: [u8; 256] = [
        0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4,
        4, 5, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5,
        4, 5, 5, 6, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4,
        4, 5, 4, 5, 5, 6, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5, 4, 5, 5, 6,
        4, 5, 5, 6, 5, 6, 6, 7, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5, 2, 3, 3, 4, 3, 4,
        4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4, 4, 5,
        4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 2, 3, 3, 4, 3, 4, 4, 5, 3, 4, 4, 5, 4, 5, 5, 6, 3, 4,
        4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7, 3, 4, 4, 5, 4, 5, 5, 6, 4, 5, 5, 6, 5, 6, 6, 7,
        4, 5, 5, 6, 5, 6, 6, 7, 5, 6, 6, 7, 6, 7, 7, 8,
    ];
    for (b, &x) in bits.iter_mut().zip(&bits_tmp) {
        *b = black_box(x);
    }
}

pub fn bitcount_ntbl_bitcount(x: u32, bits: &[u8]) -> i32 {
    bits[(x & 0x0000000F) as usize] as i32
        + bits[((x & 0x000000F0) >> 4) as usize] as i32
        + bits[((x & 0x00000F00) >> 8) as usize] as i32
        + bits[((x & 0x0000F000) >> 12) as usize] as i32
        + bits[((x & 0x000F0000) >> 16) as usize] as i32
        + bits[((x & 0x00F00000) >> 20) as usize] as i32
        + bits[((x & 0x0F000000) >> 24) as usize] as i32
        + bits[((x & 0xF0000000) >> 28) as usize] as i32
}

pub fn bitcount_bw_btbl_bitcount(x: u32, bits: &[u8]) -> i32 {
    let bytes = x.to_ne_bytes();
    bits[bytes[0] as usize] as i32
        + bits[bytes[1] as usize] as i32
        + bits[bytes[2] as usize] as i32
        + bits[bytes[3] as usize] as i32
}

pub fn bitcount_ar_btbl_bitcount(x: u32, bits: &[u8]) -> i32 {
    let mut ptr = &x as *const u32 as *const u8;
    let mut acc;

    acc = bits[unsafe { *ptr } as usize] as i32;
    ptr = unsafe { ptr.add(1) };
    acc += bits[unsafe { *ptr } as usize] as i32;
    ptr = unsafe { ptr.add(1) };
    acc += bits[unsafe { *ptr } as usize] as i32;
    ptr = unsafe { ptr.add(1) };
    acc += bits[unsafe { *ptr } as usize] as i32;
    acc
}
