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

  Name: cubic

  Author: Ross Cottrell

  Function: cubic solves some cubic functions

  Source: MiBench
          http://wwweb.eecs.umich.edu/mibench

  Original name: basicmath_small

  Changes: no major functional changes

  License: this code is FREE with no restrictions

*/

/*
**  CUBIC.C - Solve a cubic polynomial
**  public domain by Ross Cottrell
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

mod math;

use core::{f32::consts::PI, hint::black_box};
enum CubicSolution {
    One(f32),
    Three(f32, f32, f32),
}

impl CubicSolution {
    fn count(&self) -> i32 {
        match self {
            CubicSolution::One(x) => {
                black_box(x);
                1
            }
            CubicSolution::Three(x, y, z) => {
                black_box((x, y, z));
                3
            }
        }
    }
}

fn cubic_solve(a: f32, b: f32, c: f32, d: f32) -> CubicSolution {
    let a1 = b / a;
    let a2 = c / a;
    let a3 = d / a;
    let q = (a1 * a1 - 3. * a2) / 9.;
    let r = (2. * a1 * a1 * a1 - 9. * a1 * a2 + 27. * a3) / 54.;
    let r2_q3 = r * r - q * q * q;

    if r2_q3 <= 0. {
        let theta = math::acosf(r / math::sqrtf(q * q * q));
        let x = -2. * math::sqrtf(q) * math::cosf(theta / 3.) - a1 / 3.;
        let y = -2. * math::sqrtf(q) * math::cosf((theta + 2. * PI) / 3.) - a1 / 3.;
        let z = -2. * math::sqrtf(q) * math::cosf((theta + 4. * PI) / 3.) - a1 / 3.;

        CubicSolution::Three(x, y, z)
    } else {
        let mut x = math::powf(math::sqrtf(r2_q3) + r.abs(), 1. / 3.);
        x += q / x;
        x *= if r < 0. { 1. } else { -1. };
        x -= a1 / 3.;
        CubicSolution::One(x)
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn cubic_main(
    a: (f32, f32, f32, f32),
    b: (f32, f32, f32, f32),
    c: (f32, f32, f32, f32),
    d: (f32, f32, f32, f32),
) -> i32 {
    let mut checksum = 0;
    checksum += cubic_solve(a.0, a.1, a.2, a.3).count();
    checksum += cubic_solve(b.0, b.1, b.2, b.3).count();
    checksum += cubic_solve(c.0, c.1, c.2, c.3).count();
    checksum += cubic_solve(d.0, d.1, d.2, d.3).count();

    // loopbound min 5 max 5
    for a1 in (1..10).step_by(2) {
        // loopbound min 5 max 5
        for b1 in (1..=10).rev().step_by(2) {
            let mut c1 = 5.;
            // loopbound min 7 max 7
            while c1 < 15. {
                // loopbound min 5 max 5
                for d1 in (-9..=-1).rev().step_by(2) {
                    checksum += cubic_solve(a1 as f32, b1 as f32, c1, d1 as f32).count();
                }
                c1 += 1.5;
            }
        }
    }
    checksum
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn cubic_log() -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let a = (1., -10.5, 32., -30.);
    let b = (1., -4.5, 17., -30.);
    let c = (1., -3.5, 22., -31.);
    let d = (1., -13.7, 1., -35.);
    let mut result: i32 = 0;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        let start = black_box(DWT::cycle_count());
        result = black_box(cubic_main(
            black_box(a),
            black_box(b),
            black_box(c),
            black_box(d),
        ));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("cubic_custom - elapsed: {}", cycles);
    });

    result == 1051
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn cubic_std() -> bool {
    let a = (1., -10.5, 32., -30.);
    let b = (1., -4.5, 17., -30.);
    let c = (1., -3.5, 22., -31.);
    let d = (1., -13.7, 1., -35.);

    let result = black_box(cubic_main(
        black_box(a),
        black_box(b),
        black_box(c),
        black_box(d),
    ));
    result == 1051
}

fn cubic() -> bool {
    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(cubic_log())
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(cubic_std())
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(cubic());

    #[cfg(feature = "log")]
    {
        rprintln!("cubic_custom: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(cubic());

    #[cfg(feature = "log")]
    std::println!("cubic_custom: {}", success);

    std::process::exit(!success as i32);
}
