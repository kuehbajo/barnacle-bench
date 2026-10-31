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

  Version V 1.9

  Name: isqrt

  Author: unknown

  Function: isqrt calculates the integer square root of a number

  Source: MiBench
          http://wwweb.eecs.umich.edu/mibench

  Original name: basicmath_small

  Changes: no major functional changes

  License: this code is FREE with no restrictions

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

#[derive(Debug)]
struct IsqrtQ {
    _sqrt: u16,
    frac: u16,
}

fn isqrt_init(isqrt_l: &mut u32, checksum: &mut u32) {
    *isqrt_l = 0x3fed0169;
    *checksum = 0u32;
}

fn isqrt_usqrt(mut x: u32, q: &mut IsqrtQ) {
    let mut a: u32 = 0;
    let mut r: u32 = 0;
    let mut e: u32;
    // loopbound min 32 max 32
    for _ in 0..u32::BITS {
        r = (r << 2) + (x >> (u32::BITS - 2));
        x <<= 2;
        a <<= 1;
        e = (a << 1) + 1;
        if r >= e {
            r -= e;
            a += 1;
        }
    }

    let bytes = a.to_ne_bytes();
    let (sqrt, frac) = bytes.split_at(size_of::<u16>());
    *q = IsqrtQ {
        _sqrt: u16::from_ne_bytes(sqrt.try_into().unwrap()),
        frac: u16::from_ne_bytes(frac.try_into().unwrap()),
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn isqrt_main(isqrt_q: &mut IsqrtQ, isqrt_l: &u32, checksum: &mut u32) {
    // loopbound min 1000 max 1000
    for i in 1..1001 {
        isqrt_usqrt(i, isqrt_q);
        *checksum += isqrt_q.frac as u32;
    }
    isqrt_usqrt(*isqrt_l, isqrt_q);
    *checksum += isqrt_q.frac as u32;
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn isqrt_log() -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut isqrt_l = 0u32;
    let mut isqrt_q = IsqrtQ { _sqrt: 0, frac: 0 };
    let mut checksum = 0u32;
    
    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        isqrt_init(black_box(&mut isqrt_l), black_box(&mut checksum));
        let start = black_box(DWT::cycle_count());
        isqrt_main(black_box(&mut isqrt_q), black_box(&isqrt_l), black_box(&mut checksum));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("isqrt - elapsed: {}", cycles);
    });

    checksum == 53364
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn isqrt_std() -> bool {
    let mut isqrt_l = 0u32;
    let mut isqrt_q = IsqrtQ { _sqrt: 0, frac: 0 };
    let mut checksum = 0u32;

    isqrt_init(black_box(&mut isqrt_l), &mut checksum);
    isqrt_main(black_box(&mut isqrt_q), black_box(&isqrt_l), black_box(&mut checksum));

    checksum == 53364
}

fn isqrt() -> bool {
    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(isqrt_log())
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(isqrt_std())
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(isqrt());

    #[cfg(feature = "log")]
    {
        rprintln!("isqrt: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(isqrt());

    #[cfg(feature = "log")]
    std::println!("iqsrt: {}", success);

    std::process::exit(!success as i32);
}
