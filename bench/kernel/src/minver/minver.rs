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

  Name: minver

  Author: Sung-Soo Lim

  Function: Matrix inversion for 3x3 floating point matrix.

  Source: SNU-RT Benchmark Suite, via MRTC
    http://www.mrtc.mdh.se/projects/wcet/wcet_bench/minver/minver.c

  Changes: a brief summary of major functional changes (not formatting)

  License: may be used, modified, and re-distributed freely, but
    the SNU-RT Benchmark Suite must be acknowledged

*/

/*
    This program is derived from the SNU-RT Benchmark Suite for Worst
    Case Timing Analysis by Sung-Soo Lim

    Original source: Turbo C Programming for Engineering by Hyun Soo Ahn
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::{f64, hint::black_box};

fn minver_init(initial: &mut [[f64; 3]; 3], a: &mut [[f64; 3]; 3]) {
    let x = 0.;
    // loopbound min 3 max 3
    for (row_a, row_i) in a.iter_mut().zip(initial.iter()) {
        // loopbound min 3 max 3
        for (v_a, v_i) in row_a.iter_mut().zip(row_i.iter()) {
            *v_a = black_box(v_i) + black_box(x);
        }
    }
}

fn minver_minver(a: &mut [[f64; 3]; 3], side: usize, eps: f64) -> Option<f64> {
    if !(2..=500).contains(&side) || eps <= 0.0 {
        return None;
    }

    let mut work = [0; 500];

    let mut w = 0.0;
    let mut wmax;
    let mut r = 0;

    let mut w1 = 1.0;
    // loopbound min 3 max 3
    #[expect(clippy::needless_range_loop)]
    for i in 0..side {
        work[i] = i;
    }

    // loopbound min 3 max 3
    for k in 0..side {
        wmax = 0.0;
        // loopbound min 1 max 3
        #[expect(clippy::needless_range_loop)]
        for i in k..side {
            w = a[i][k].abs();
            if w > wmax {
                wmax = w;
                r = i;
            }
        }
        let pivot = a[r][k];
        let api = pivot.abs();
        if api <= eps {
            return Some(w1);
        }

        w1 *= pivot;
        if r != k {
            w1 = -w;
            work.swap(k, r);
            a.swap(k, r);
        }

        // loopbound min 3 max 3
        for i in 0..side {
            a[k][i] /= pivot;
        }
        // loopbound min 3 max 3
        for i in 0..side {
            if i != k {
                w = a[i][k];
                if w != 0.0 {
                    // loopbound min 3 max 3
                    for j in 0..side {
                        if j != k {
                            a[i][j] -= w * a[k][j];
                        }
                    }
                    a[i][k] = -w / pivot;
                }
            }
        }
        a[k][k] = pivot.recip();
    }
    // loopbound min 3 max 3
    for i in 0..side {
        // loopbound min 1 max 3
        loop {
            let k = work[i];
            if k == i {
                break;
            }

            work.swap(k, i);
            a[k].swap(i, k);
        }
    }

    Some(w1)
}

fn minver_mmul(
    a: &[[f64; 3]; 3],
    b: &[[f64; 3]; 3],
    c: &mut [[f64; 3]; 3],
    row_a: usize,
    col_a: usize,
    row_b: usize,
    col_b: usize,
) -> Result<(), ()> {
    let row_c = row_a;
    let col_c = col_b;

    if row_c < 1 || row_b < 1 || col_c < 1 || col_a != row_b {
        return Err(());
    }

    // loopbound min 3 max 3
    for i in 0..row_c {
        // loopbound min 3 max 3
        for j in 0..col_c {
            let mut w = 0.0;
            // loopbound min 3 max 3
            #[expect(clippy::needless_range_loop)]
            for k in 0..row_b {
                w += a[i][k] * b[k][j];
            }
            c[i][j] = w;
        }
    }

    Ok(())
}

fn minver_return(a_i: &[[f64; 3]; 3]) -> bool {
    // loopbound min 9 max 9
    let checksum: f64 = a_i.iter().flatten().sum();

    (checksum * 100.) as i32 == 48
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn minver_main(
    a: &mut [[f64; 3]; 3],
    b: &[[f64; 3]; 3],
    c: &mut [[f64; 3]; 3],
    aa: &mut [[f64; 3]; 3],
    a_i: &mut [[f64; 3]; 3],
) {
    let eps = 1.0e-6;

    aa.copy_from_slice(a);

    minver_minver(a, 3, eps);
    a_i.copy_from_slice(a);

    let _ = minver_mmul(a, b, c, 3, 3, 3, 3);
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn minver_log(
    initial: &mut [[f64; 3]; 3],
    a: &mut [[f64; 3]; 3],
    b: &[[f64; 3]; 3],
    c: &mut [[f64; 3]; 3],
    aa: &mut [[f64; 3]; 3],
    a_i: &mut [[f64; 3]; 3],
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        minver_init(black_box(initial), black_box(a));
        let start = black_box(DWT::cycle_count());
        minver_main(
            black_box(a),
            black_box(b),
            black_box(c),
            black_box(aa),
            black_box(a_i),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("minver - elapsed: {}", cycles);
    });

    black_box(minver_return(black_box(a_i)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn minver_std(
    initial: &mut [[f64; 3]; 3],
    a: &mut [[f64; 3]; 3],
    b: &[[f64; 3]; 3],
    c: &mut [[f64; 3]; 3],
    aa: &mut [[f64; 3]; 3],
    a_i: &mut [[f64; 3]; 3],
) -> bool {
    minver_init(black_box(initial), black_box(a));
    minver_main(
        black_box(a),
        black_box(b),
        black_box(c),
        black_box(aa),
        black_box(a_i),
    );

    black_box(minver_return(black_box(a_i)))
}

fn minver() -> bool {
    static mut INITIAL: [[f64; 3]; 3] = [[3., -6., 7.], [9., 0., -5.], [5., -8., 6.]];
    static mut A: [[f64; 3]; 3] = [[0.; 3]; 3];
    static mut B: [[f64; 3]; 3] = [[0.; 3]; 3];
    static mut C: [[f64; 3]; 3] = [[0.; 3]; 3];
    static mut AA: [[f64; 3]; 3] = [[0.; 3]; 3];
    static mut A_I: [[f64; 3]; 3] = [[0.; 3]; 3];
    // SAFETY: This is the only place we ever reference these and this function is only ever called once.
    let initial = unsafe { &mut *{ &raw mut INITIAL } };
    let a = unsafe { &mut *{ &raw mut A } };
    let b = unsafe { &mut *{ &raw mut B } };
    let c = unsafe { &mut *{ &raw mut C } };
    let aa = unsafe { &mut *{ &raw mut AA } };
    let a_i = unsafe { &mut *{ &raw mut A_I } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(minver_log(
            black_box(initial),
            black_box(a),
            black_box(b),
            black_box(c),
            black_box(aa),
            black_box(a_i),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(minver_std(
            black_box(initial),
            black_box(a),
            black_box(b),
            black_box(c),
            black_box(aa),
            black_box(a_i),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(minver());

    #[cfg(feature = "log")]
    {
        rprintln!("minver: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(minver());

    #[cfg(feature = "log")]
    std::println!("minver: {}", success);

    std::process::exit(!success as i32);
}
