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

  Name: binarysearch

  Author: Sung-Soo Lim <sslim@archi.snu.ac.kr>

  Function: binarysearch performs binary search in an array of 15 integer
    elements.
    This program is completely structured (no unconditional jumps, no exits
    from loop bodies), and does not contain switch statements, no do-while
    loops.

  Source: MRTC
          http://www.mrtc.mdh.se/projects/wcet/wcet_bench/bs/bs.c

  Original name: bs

  Changes: No major functional changes.

  License: May be used, modified, and re-distributed freely, but
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

use core::{cmp::Ordering, hint::black_box};

#[derive(Debug, Copy, Clone)]
struct Data {
    key: i32,
    value: i32,
}

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

fn binarysearch_init(data: &mut [Data; 15]) {
    let mut rand = RandomInt::init();

    // loopbound min 15 max 15
    for (i, d) in data.iter_mut().enumerate() {
        d.key = i as i32;
        d.value = rand.next();
    }
}

fn binarysearch_search(data: &[Data; 15], x: i32) -> Option<i32> {
    let mut mid;
    let mut low = 0i32;
    let mut up = 14i32;
    let mut fvalue = None;

    // loopbound min 1 max 4
    while low <= up {
        mid = (low + up) >> 1;

        match data[mid as usize].key.cmp(&x) {
            Ordering::Equal => {
                // item found
                up = low - 1;
                fvalue = Some(data[mid as usize].value);
            }
            Ordering::Greater => up = mid - 1,
            Ordering::Less => low = mid + 1,
        }
    }

    fvalue
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn binarysearch_main(data: &[Data; 15]) -> Option<i32> {
    binarysearch_search(data, 8)
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn binarysearch_log(data: &mut [Data; 15]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: Option<i32> = None;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        binarysearch_init(black_box(data));
        let start = black_box(DWT::cycle_count());
        result = black_box(binarysearch_main(black_box(data)));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("binarysearch - elapsed: {}", cycles);
    });

    result == Some(4326)
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn binarysearch_std(data: &mut [Data; 15]) -> bool {
    binarysearch_init(black_box(data));
    let result = black_box(binarysearch_main(black_box(data)));

    result == Some(4326)
}

fn binarysearch() -> bool {
    static mut DATA: [Data; 15] = [Data { key: 0, value: 0 }; 15];
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let data: &'static mut [Data; 15] = unsafe { &mut *{ &raw mut DATA } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(binarysearch_log(black_box(data)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(binarysearch_std(black_box(data)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(binarysearch());
    
    #[cfg(feature = "log")]
    {
        rprintln!("binarysearch: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(binarysearch());

    #[cfg(feature = "log")]
    std::println!("binarysearch: {}", success);

    std::process::exit(!success as i32);
}
