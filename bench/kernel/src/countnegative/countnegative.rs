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

  Version V 2.0

  Name: countnegative

  Author: unknown

  Function: Counts negative and non-negative numbers in a
    matrix. Features nested loops, well-structured code.

  Source: MRTC
          http://www.mrtc.mdh.se/projects/wcet/wcet_bench/cnt/cnt.c

  Changes: Changed split between initialization and computation

  License: May be used, modified, and re-distributed freely

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

const MAX_SIZE: usize = 20;
type Matrix = [[i32; MAX_SIZE]; MAX_SIZE];

struct RandomInt(i32);

impl RandomInt {
    fn init() -> Self {
        RandomInt(0)
    }

    fn next(&mut self) -> i32 {
        self.0 = ((self.0 * 133) + 81) % 8095;
        self.0
    }
}

fn countnegative_init(matrix: &mut Matrix) {
    let mut rand = RandomInt::init();

    // loopbound min 20 max 20
    for row in matrix {
        // loopbound min 20 max 20
        for v in row {
            *v = rand.next();
        }
    }
}

fn countnegative_sum(matrix: &Matrix) -> ((i32, i32), (i32, i32)) {
    let mut pos_total = 0;
    let mut pos_count = 0;
    let mut neg_total = 0;
    let mut neg_count = 0;

    // loopbound min 20 max 20
    for row in matrix {
        // loopbound min 20 max 20
        for &v in row {
            if v >= 0 {
                pos_total += v;
                pos_count += 1;
            } else {
                neg_total += v;
                neg_count += 1;
            }
        }
    }

    ((pos_total, pos_count), (neg_total, neg_count))
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn countnegative_main(matrix: &Matrix) -> ((i32, i32), (i32, i32)) {
    countnegative_sum(matrix)
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn countnegative_log(matrix: &mut Matrix) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let ((mut pos_total, mut pos_count), (mut neg_total, mut neg_count)) : ((i32, i32), (i32, i32)) =
        ((0,0), (0,0));

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        countnegative_init(black_box(matrix));
        let start = black_box(DWT::cycle_count());
        ((pos_total, pos_count), (neg_total, neg_count)) =
            black_box(countnegative_main(black_box(matrix)));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("countnegative - elapsed: {}", cycles);
    });

    let checksum = pos_total + pos_count + neg_total + neg_count;
    checksum == 0x1778de
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn countnegative_std(matrix: &mut Matrix) -> bool {
    countnegative_init(black_box(matrix));
    let ((pos_total, pos_count), (neg_total, neg_count)) =
        black_box(countnegative_main(black_box(matrix)));

    let checksum = pos_total + pos_count + neg_total + neg_count;
    checksum == 0x1778de
}

fn countnegative() -> bool {
    static mut MATRIX: Matrix = [[0; MAX_SIZE]; MAX_SIZE];
    // SAFETY: This is the only place we ever reference MATRIX and this function is not reentrant.
    let matrix = unsafe { &mut *{ &raw mut MATRIX } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(countnegative_log(black_box(matrix)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(countnegative_std(black_box(matrix)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(countnegative());

    #[cfg(feature = "log")]
    {
        rprintln!("countnegative: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(countnegative());

    #[cfg(feature = "log")]
    std::println!("countnegative: {}", success);

    std::process::exit(!success as i32);
}
