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

  Name: recursion

  Author: unknown

  Function: recursion is a recursion program.
    This program computes the Fibonacci number recursively.

  Source: MRTC
          http://www.mrtc.mdh.se/projects/wcet/wcet_bench/recursion/recursion.c

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

use core::hint::black_box;

fn recursion_init(input: &mut i32) {
    let temp_input = black_box(10);
    *input = black_box(temp_input);
}

fn recursion_fib(i: i32) -> i32 {
    match i {
        0 | 1 => 1,
        _ => recursion_fib(i - 1) + recursion_fib(i - 2),
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn recursion_main(input: &mut i32, result: &mut i32) {
    // marker recursivecall
    // flowrestriction 1*fib <= 177*recursivecall
    *result = recursion_fib(*input);
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn recursion_log(input: &mut i32, result: &mut i32) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        recursion_init(black_box(input));
        let start = black_box(DWT::cycle_count());
        recursion_main(black_box(input), black_box(result));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("recursion - elapsed: {}", cycles);
    });

    black_box(*result) == 89
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn recursion_std(input: &mut i32, result: &mut i32) -> bool {
    recursion_init(black_box(input));
    recursion_main(black_box(input), black_box(result));

    black_box(*result) == 89
}

fn recursion() -> bool {
    static mut RECURSION_INPUT: i32 = 0;
    static mut RECURSION_RESULT: i32 = 0;
    // SAFETY: This is the only place we reference these and this function is only ever called once.
    let recursion_input = unsafe { &mut *{ &raw mut RECURSION_INPUT } };
    let recursion_result = unsafe { &mut *{ &raw mut RECURSION_RESULT } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(recursion_log(black_box(recursion_input), black_box(recursion_result)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(recursion_std(black_box(recursion_input), black_box(recursion_result)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(recursion());

    #[cfg(feature = "log")]
    {
        rprintln!("recursion: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(recursion());

    #[cfg(feature = "log")]
    std::println!("recursion: {}", success);

    std::process::exit(!success as i32);
}
