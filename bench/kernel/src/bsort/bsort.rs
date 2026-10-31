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

  Version 2.0

  Name: bsort

  Author: unknown

  Function: A program for testing the basic loop constructs,
            integer comparisons, and simple array handling by
            sorting 100 integers

  Source: MRTC
          http://www.mrtc.mdh.se/projects/wcet/wcet_bench/bsort100/bsort100.c

  Original name: bsort100

  Changes: See ChangeLog.txt

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

fn bsort_init(arr: &mut [i32; 100]) {
    // loopbound min 100 max 100
    for (i, x) in arr.iter_mut().enumerate() {
        *x = -(i as i32 + 1)
    }
}

fn bsort_bubblesort(arr: &mut [i32; 100]) {
    let mut sorted;

    // loopbound min 99 max 99
    for i in 0..arr.len() - 1 {
        sorted = true;

        // loopbound min 3 max 99
        for index in 0..arr.len() - 1 {
            if index > arr.len() - i {
                break;
            }

            if arr[index] > arr[index + 1] {
                arr.swap(index, index + 1);
                sorted = false;
            }
        }

        if sorted {
            break;
        }
    }
}

fn bsort_return(arr: &mut [i32; 100]) -> bool {
    // loopbound min 99 max 99
    arr.windows(2).all(|w| w[0] < w[1])
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn bsort_main(arr: &mut [i32; 100]) {
    bsort_bubblesort(arr);
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn bsort_log(arr: &mut [i32; 100]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        bsort_init(black_box(arr));
        let start = black_box(DWT::cycle_count());
        bsort_main(black_box(arr));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("bsort - elapsed: {}", cycles);
    });

    black_box(bsort_return(black_box(arr)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn bsort_std(arr: &mut [i32; 100]) -> bool {
    bsort_init(black_box(arr));
    bsort_main(black_box(arr));

    black_box(bsort_return(black_box(arr)))
}

fn bsort() -> bool {
    static mut ARRAY: [i32; 100] = [0; 100];
    // SAFETY: This is the only place we ever reference ARRAY and this function is not reentrant.
    let arr: &'static mut [i32; 100] = unsafe { &mut *{ &raw mut ARRAY } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(bsort_log(black_box(arr)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(bsort_std(black_box(arr)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(bsort());

    #[cfg(feature = "log")]
    {
        rprintln!("bsort: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(bsort());

    #[cfg(feature = "log")]
    std::println!("bsort: {}", success);

    std::process::exit(!success as i32);
}
