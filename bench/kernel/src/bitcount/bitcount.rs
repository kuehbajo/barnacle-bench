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

  Name: bitcount.c

  Author: Bob Stout & Auke Reitsma

  Function: test program for bit counting functions

  Source: www.snippest.com

  Changes: no major functional changes

  License: May be used, modified, and re-distributed freely.

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

mod bitcnt_1;
mod bitcnt_2;
mod bitcnt_3;
mod bitcnt_4;

use bitcnt_1::bitcount_bit_count;
use bitcnt_2::bitcount_bitcount;
use bitcnt_3::{
    bitcount_ar_btbl_bitcount, bitcount_bw_btbl_bitcount, bitcount_init3, bitcount_ntbl_bitcount,
};
use bitcnt_4::{bitcount_btbl_bitcnt, bitcount_init4, bitcount_ntbl_bitcnt};
use core::hint::black_box;

const FUNCS: usize = 8;
const ITERATIONS: usize = 10;

struct Random(u32);

impl Random {
    fn init() -> Self {
        Self(1)
    }

    fn next(&mut self) -> u32 {
        /*
            Compute x[n + 1] = (7^5 * x[n]) mod (2^31 - 1).
            From "Random number generators: good ones are hard to find",
            Park and Miller, Communications of the ACM, vol. 31, no. 10,
            October 1988, p. 1195.
        */
        let x = self.0 as i32;
        let hi = x / 127773;
        let lo = x % 127773;
        let mut t = 16807 * lo - 2836 * hi;
        if t <= 0 {
            t += 0x7fffffff;
        }
        self.0 = t as u32;
        self.0
    }
}

impl Iterator for Random {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next())
    }
}

fn bitcount_init(bits3: &mut [u8], bits4: &mut [u8]) {
    bitcount_init3(bits3);
    bitcount_init4(bits4);
}

fn bitcount_bit_shifter(mut x: u32) -> i32 {
    let mut n = 0;
    let mut i = 0;
    while x != 0 && i < size_of_val(&x) * 8 {
        n += (x & 1) as i32;

        i += 1;
        x >>= 1;
    }
    n
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn bitcount_main(bits3: &mut [u8], bits4: &mut [u8]) -> i32 {
    let mut sum = 0;
    // loopbound min 8 max 8
    for (i, mut seed) in Random::init().take(FUNCS).enumerate() {
        // loopbound min 10 max 10
        for _ in 0..ITERATIONS {
            let res = match i {
                0 => bitcount_bit_count(seed),
                1 => bitcount_bitcount(seed),
                2 => {
                    // marker call_ntbl
                    bitcount_ntbl_bitcnt(seed, bits4)
                }
                3 => {
                    // marker call_btbl
                    bitcount_btbl_bitcnt(seed, bits4)
                }
                4 => bitcount_ntbl_bitcount(seed, bits3),
                5 => bitcount_bw_btbl_bitcount(seed, bits3),
                6 => bitcount_ar_btbl_bitcount(seed, bits3),
                7 => bitcount_bit_shifter(seed),
                _ => 0,
            };

            sum += res;

            seed += 13;
        }
    }
    // flowrestriction 1*ntbl_bitcount <= 8*call_ntbl
    // flowrestriction 1*btbl_bitcount <= 4*call_btbl
    sum
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn bitcount_log(bits3: &mut [u8], bits4: &mut [u8]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: i32 = 0;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        bitcount_init(black_box(bits3), black_box(bits4));
        let start = black_box(DWT::cycle_count());
        result = black_box(bitcount_main(black_box(bits3), black_box(bits4)));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("bitcount - elapsed: {}", cycles);
    });

    result == 1095
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn bitcount_std(bits3: &mut [u8], bits4: &mut [u8]) -> bool {
    bitcount_init(black_box(bits3), black_box(bits4));
    let result = black_box(bitcount_main(black_box(bits3), black_box(bits4)));

    result == 1095
}

fn bitcount() -> bool {
    static mut BITS3: [u8; 256] = [0; 256];
    static mut BITS4: [u8; 256] = [0; 256];
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let bits3 = unsafe { &mut *{ &raw mut BITS3 } };
    let bits4 = unsafe { &mut *{ &raw mut BITS4 } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(bitcount_log(black_box(bits3), black_box(bits4)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(bitcount_std(black_box(bits3), black_box(bits4)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(bitcount());
    
    #[cfg(feature = "log")]
    {
        rprintln!("bitcount: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(bitcount());

    #[cfg(feature = "log")]
    std::println!("bitcount: {}", success);

    std::process::exit(!success as i32);
}
