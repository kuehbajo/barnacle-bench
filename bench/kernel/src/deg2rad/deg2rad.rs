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

  Name: deg2rad

  Author: unknown

  Function: deg2rad performs conversion of degree to radiant

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
#[expect(clippy::approx_constant)]
const PI: f32 = 3.14;

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn deg2rad_main() -> f32 {
    let mut y = 0.0;
    // loopbound min 361 max 361
    for x in 0..=360 {
        y += x as f32 * PI / 180.;
    }
    y
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn deg2rad_log() -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut y: f32 = 0.;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {        
        let start = black_box(DWT::cycle_count());
        y = black_box(deg2rad_main());
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("deg2rad - elapsed: {}", cycles);
    });

    y as i32 == 1133
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn deg2rad_std() -> bool {
    let y = black_box(deg2rad_main());

    y as i32 == 1133
}

fn deg2rad() -> bool {
    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(deg2rad_log())
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(deg2rad_std())
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(deg2rad());

    #[cfg(feature = "log")]
    {
        rprintln!("deg2rad: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(deg2rad());

    #[cfg(feature = "log")]
    std::println!("deg2rad: {}", success);

    std::process::exit(!success as i32);
}
