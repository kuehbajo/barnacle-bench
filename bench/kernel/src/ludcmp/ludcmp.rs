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

  Name: ludcmp

  Author: Sung-Soo Lim

  Function: Simultaneous linear equations by LU decomposition.

  Source: SNU-RT Benchmark Suite, via MRTC
          http://www.mrtc.mdh.se/projects/wcet/wcet_bench/ludcmp/ludcmp.c

  Changes: Moved initialization into separate function.

  License: May be used, modified, and re-distributed freely, but
           the SNU-RT Benchmark Suite must be acknowledged

*/

/*
  This program is derived from the SNU-RT Benchmark Suite for Worst
  Case Timing Analysis by Sung-Soo Lim

  III-4. ludcmp.c : Simultaneous Linear Equations by LU Decomposition
                    (from the book C Programming for EEs by Hyun Soon Ahn)
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

fn ludcmp_init(a: &mut [[f64; 50]; 50], b: &mut [f64; 50]) {
    let x = 0;
    for i in 0..=5 {
        let mut w: f64 = 0.0;
        for j in 0..=5 {
            a[i][j] = ((i + 1) + (j + 1)) as f64;

            if i == j {
                a[i][j] *= 10.
            }
            w += a[i][j];

            if black_box(x) != 0 {
                a[i][j] += black_box(x) as f64;
            }
        }
        b[i] = w;
        if black_box(x) != 0 {
            b[i] += black_box(x) as f64;
        }
    }
}

fn ludcmp_test(
    a: &mut [[f64; 50]; 50],
    b: &[f64; 50],
    x: &mut [f64; 50],
    n: usize,
    eps: f64,
) -> i32 {
    let mut y = [0f64; 100];

    if n > 99 || eps < 0.0 {
        return 999;
    }
    // loopbound min 5 max 5
    for i in 0..n {
        if a[i][i].abs() <= eps {
            return 1;
        }
        // loopbound min 1 max 5
        for j in i + 1..=n {
            let mut w = a[j][i];
            if i != 0 {
                // loopbound min 1 max 4
                for k in 0..i {
                    w -= a[j][k] * a[k][i];
                }
            }
            a[j][i] = w / a[i][i];
        }

        // loopbound min 1 max 5
        for j in i + 1..=n {
            let mut w = a[i + 1][j];
            // loopbound min 1 max 5
            for k in 0..=i {
                w -= a[i + 1][k] * a[k][j];
            }
            a[i + 1][j] = w;
        }
    }

    y[0] = b[0];

    // loopbound min 5 max 5
    for i in 1..=n {
        let mut w = b[i];
        // loopbound min 1 max 5
        for (j, yy) in y.iter().enumerate().take(i) {
            w -= a[i][j] * yy;
        }
        y[i] = w;
    }
    x[n] = y[n] / a[n][n];

    // loopbound min 5 max 5
    for i in (0..=n - 1).rev() {
        let mut w = y[i];
        // loopbound min 1 max 5
        for (j, xx) in x.iter().enumerate().take(n + 1).skip(i + 1) {
            w -= a[i][j] * xx;
        }
        x[i] = w / a[i][i];
    }

    0
}

fn ludcmp_return(x: &[f64; 50], checksum: i32) -> bool {
    // loopbound min 6 max 6
    let mut checksum = checksum as f64 + x[..=5].iter().sum::<f64>();

    /* allow rounding errors for the checksum */
    checksum -= 6.0;
    checksum < 0.000001 && checksum > -0.000001
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn ludcmp_main(a: &mut [[f64; 50]; 50], b: &[f64; 50], x: &mut [f64; 50]) -> i32 {
    let n = 5;
    let eps = 1.0;
    ludcmp_test(a, b, x, n, eps)
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn ludcmp_log(a: &mut [[f64; 50]; 50], b: &mut [f64; 50], x: &mut [f64; 50]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: i32 = 0;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        ludcmp_init(black_box(a), black_box(b));
        let start = black_box(DWT::cycle_count());
        result = black_box(ludcmp_main(black_box(a), black_box(b), black_box(x)));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("ludcmp - elapsed: {}", cycles);
    });

    black_box(ludcmp_return(black_box(x), black_box(result)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn ludcmp_std(a: &mut [[f64; 50]; 50], b: &mut [f64; 50], x: &mut [f64; 50]) -> bool {
    ludcmp_init(black_box(a), black_box(b));
    let result = black_box(ludcmp_main(black_box(a), black_box(b), black_box(x)));

    black_box(ludcmp_return(black_box(x), black_box(result)))
}

fn ludcmp() -> bool {
    static mut A: [[f64; 50]; 50] = [[0.0; 50]; 50];
    static mut B: [f64; 50] = [0.0; 50];
    static mut X: [f64; 50] = [0.0; 50];
    // SAFETY: This is the only place we ever reference these and this function is only ever called once.
    let a = unsafe { &mut *{ &raw mut A } };
    let b = unsafe { &mut *{ &raw mut B } };
    let x = unsafe { &mut *{ &raw mut X } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(ludcmp_log(black_box(a), black_box(b), black_box(x)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(ludcmp_std(black_box(a), black_box(b), black_box(x)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(ludcmp());

    #[cfg(feature = "log")]
    {
        rprintln!("ludcmp: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(ludcmp());

    #[cfg(feature = "log")]
    std::println!("ludcmp: {}", success);

    std::process::exit(!success as i32);
}
