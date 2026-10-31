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

  Name: rad2deg

  Author: unknown

  Function: rad2deg performs conversion of radiant to degree

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

fn rad2deg_init(x: &mut f32, y: &mut f32) {
    *x = 0.;
    *y = 0.;
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn rad2deg_main(x: &mut f32, y: &mut f32) {
    *x = 0.;
    // loopbound min 360 max 360
    while *x <= (2. * PI + 1e-6) {
        *y += *x * 180. / PI;

        *x += PI / 180.;
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn rad2deg_log(x: &mut f32, y: &mut f32) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        rad2deg_init(black_box(x), black_box(y));
        let start = black_box(DWT::cycle_count());
        rad2deg_main(black_box(x), black_box(y));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("rad2deg - elapsed: {}", cycles);
    });

    black_box(*y) == 64620.
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn rad2deg_std(x: &mut f32, y: &mut f32) -> bool {
    rad2deg_init(black_box(x), black_box(y));
    rad2deg_main(black_box(x), black_box(y));

    black_box(*y) == 64620.
}

fn rad2deg() -> bool {
    static mut RAD2DEG_X: f32 = 0.;
    static mut RAD2DEG_Y: f32 = 0.;
    // SAFETY: This is the only place we reference these and this function is only ever called once.
    let rad2deg_x = unsafe { &mut *{ &raw mut RAD2DEG_X } };
    let rad2deg_y = unsafe { &mut *{ &raw mut RAD2DEG_Y } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(rad2deg_log(black_box(rad2deg_x), black_box(rad2deg_y)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(rad2deg_std(black_box(rad2deg_x), black_box(rad2deg_y)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(rad2deg());

    #[cfg(feature = "log")]
    {
        rprintln!("rad2deg: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(rad2deg());

    #[cfg(feature = "log")]
    std::println!("rad2deg: {}", success);

    std::process::exit(!success as i32);
}
