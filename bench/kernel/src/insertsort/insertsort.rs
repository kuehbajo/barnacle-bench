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

  Name: insertsort

  Author: Sung-Soo Lim

  Function: Insertion sort for 10 integer numbers.
     The integer array insertsort_a[  ] is initialized in main function.
     Input-data dependent nested loop with worst-case of
     (n^2)/2 iterations (triangular loop).

  Source: MRTC
          http://www.mrtc.mdh.se/projects/wcet/wcet_bench/insertsort/insertsort.c

  Changes: a brief summary of major functional changes (not formatting)

  License: may be used, modified, and re-distributed freely, but
           the SNU-RT Benchmark Suite must be acknowledged

*/

/*
  This program is derived from the SNU-RT Benchmark Suite for Worst
  Case Timing Analysis by Sung-Soo Lim
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

fn insertsort_initialize(array: &mut [u32; 11], a: &[u32; 11]) {
    for i in 0..11 {
        array[black_box(i)] = a[black_box(i)];
    }
}

fn insertsort_init(
    array: &mut [u32; 11],
    iters_i: &mut i32,
    min_i: &mut i32,
    max_i: &mut i32,
    iters_a: &mut i32,
    min_a: &mut i32,
    max_a: &mut i32,
) {
    let a = [0u32, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2];

    *iters_i = 0;
    *min_i = 100_000;
    *max_i = 0;
    *iters_a = 0;
    *min_a = 100_000;
    *max_a = 0;

    insertsort_initialize(array, &a);
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn insertsort_main(
    array: &mut [u32; 11],
    iters_i: &mut i32,
    min_i: &mut i32,
    max_i: &mut i32,
    iters_a: &mut i32,
    min_a: &mut i32,
    max_a: &mut i32,
) {
    *iters_i = 0;

    // loopbound min 9 max 9
    for i in 2..=10 {
        *iters_i += 1;

        let mut j = i;
        *iters_a = 0;
        // loopbound min 1 max 9
        while array[j] < array[j - 1] {
            *iters_a += 1;

            array.swap(j, j - 1);

            j -= 1;
        }

        if iters_a < min_a {
            *min_a = *iters_a;
        }
        if iters_a > max_a {
            *max_a = *iters_a;
        }
    }

    if iters_i < min_i {
        *min_i = *iters_i;
    }
    if iters_i > max_i {
        *max_i = *iters_i;
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn insertsort_log(array: &mut [u32; 11]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut iters_i = 0;
    let mut min_i = 0;
    let mut max_i = 0;
    let mut iters_a = 0;
    let mut min_a = 0;
    let mut max_a = 0;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        insertsort_init(
            black_box(array),
            black_box(&mut iters_i),
            black_box(&mut min_i),
            black_box(&mut max_i),
            black_box(&mut iters_a),
            black_box(&mut min_a),
            black_box(&mut max_a),
        );
        let start = black_box(DWT::cycle_count());
        insertsort_main(
            black_box(array),
            black_box(&mut iters_i),
            black_box(&mut min_i),
            black_box(&mut max_i),
            black_box(&mut iters_a),
            black_box(&mut min_a),
            black_box(&mut max_a),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("insertsort - elapsed: {}", cycles);
    });

    let result: u32 = array.iter().sum();
    result == 65
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn insertsort_std(array: &mut [u32; 11]) -> bool {
    let mut iters_i = 0;
    let mut min_i = 0;
    let mut max_i = 0;
    let mut iters_a = 0;
    let mut min_a = 0;
    let mut max_a = 0;

    insertsort_init(
        black_box(array),
        black_box(&mut iters_i),
        black_box(&mut min_i),
        black_box(&mut max_i),
        black_box(&mut iters_a),
        black_box(&mut min_a),
        black_box(&mut max_a),
    );

    insertsort_main(
        black_box(array),
        black_box(&mut iters_i),
        black_box(&mut min_i),
        black_box(&mut max_i),
        black_box(&mut iters_a),
        black_box(&mut min_a),
        black_box(&mut max_a),
    );

    let result: u32 = array.iter().sum();
    result == 65
}

fn insertsort() -> bool {
    static mut ARRAY: [u32; 11] = [0; 11];

    // SAFETY: This is the only place we ever reference these and main is not reentrant.
    let array = unsafe { &mut *{ &raw mut ARRAY } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(insertsort_log(black_box(array)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(insertsort_std(black_box(array)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(insertsort());

    #[cfg(feature = "log")]
    {
        rprintln!("insertsort: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(insertsort());

    #[cfg(feature = "log")]
    std::println!("insertsort: {}", success);

    std::process::exit(!success as i32);
}
