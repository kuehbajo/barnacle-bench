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

  Name: fac

  Author: unknown

  Function: fac is a program to calculate factorials.
    This program computes the sum of the factorials
    from zero to five.

  Source: MRTC
          http://www.mrtc.mdh.se/projects/wcet/wcet_bench/fac/fac.c

  Changes: CS 2006/05/19: Changed loop bound from constant to variable.

  License: public domain

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

fn fac_init(s: &mut i32, n: &mut i32) {
    *s = 0;
    *n = 5;
}

fn fac_fac(n: i32) -> i32 {
    if n == 0 { 1 } else { n * fac_fac(n - 1) }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn fac_main(s: &mut i32, n: i32) {
    // loopbound min 6 max 6
    for i in 0..=n {
        // marker recursivecall
        *s += fac_fac(i);
        // flowrestriction 1*fac_fac <= 6*recursivecall
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn fac_log(s: &mut i32) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut n = 5;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        fac_init(s, &mut n);
        let start = black_box(DWT::cycle_count());
        fac_main(black_box(s), black_box(n));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("fac - elapsed: {}", cycles);
    });

    *s == 154
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn fac_std(s: &mut i32) -> bool {
    let mut n = 5;
    fac_init(s, &mut n);

    fac_main(black_box(s), black_box(n));

    *s == 154
}

fn fac() -> bool {
    static mut S: i32 = 0;
    // SAFETY: This is the only place we ever reference S and this function is not reentrant.
    let s = unsafe { &mut *{ &raw mut S } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(fac_log(black_box(s)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(fac_std(black_box(s)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(fac());

    #[cfg(feature = "log")]
    {
        rprintln!("fac: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(fac());

    #[cfg(feature = "log")]
    std::println!("fac: {}", success);

    std::process::exit(!success as i32);
}
