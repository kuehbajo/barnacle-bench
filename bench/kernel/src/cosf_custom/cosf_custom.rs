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

  Name: cosf

  Author: Dustin Green

  Function: cosf performs calculations of the cosinus function

  Source:

  Original name:

  Changes:

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

mod math;

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn cosf_main() -> f32 {
    let mut result = 0.;
    // loopbound min 100 max 100
    for i in 0..100 {
        result += math::cosf(i as f32 * 0.1);
    }
    result
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn cosf_log() -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: f32 = 0.;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        let start = black_box(DWT::cycle_count());
        result = black_box(cosf_main());
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("cosf_custom - elapsed: {}", cycles);
    });

    result as i32 == -4
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn cosf_std() -> bool {
    let result = black_box(cosf_main());

    result as i32 == -4
}

fn cosf() -> bool {
    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(cosf_log())
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(cosf_std())
    }    
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(cosf());

    #[cfg(feature = "log")]
    {
        rprintln!("cosf_custom: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(cosf());

    #[cfg(feature = "log")]
    std::println!("cosf_custom: {}", success);

    std::process::exit(!success as i32);
}
