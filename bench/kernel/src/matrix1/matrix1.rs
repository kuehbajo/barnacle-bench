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

  Name: matrix1

  Author: Juan Martinez Velarde

  Function: Generic matrix - multiply benchmarking

    This program performs a matrix multiplication of the form C=AB,
    where A and B are two dimensional matrices of arbitrary dimension.
    The only restriction os that the inner dimension of the arrays must
    be greater than 1.

      A[ X x Y ] * B[ Y x Z ] = C[ X x Z ]

                       |a11     a12     ..      a1y|
                       |a21     a22     ..      a2y|
      matrix A[ X x Y ]= |..      ..      ..     ..  |
                       |a(x-1)1 a(x-1)2 ..  a(x-1)y|
                       |ax1     ax2     ..      axy|


                       |b11     b12     ..     b1z|
                       |b21     b22     ..     b2z|
      matrix B[ Y x Z ]= |..      ..      ..     .. |
                       |b(y-1)1 b(y-1)2 .. b(y-1)z|
                       |by1     by2     ..     byz|

                       |c11     c12     ..     c1z|
                       |c21     c22     ..     c2z|
      matrix C[ X x Z ]= |..      ..      ..     .. |
                       |c(x-1)1 c(x-1)2 .. c(x-1)z|
                       |cx1     cx2     ..     cxz|

      matrix elements are stored as

      A[ X x Y ] = { a11, a12, .. , a1y,
                   a21, a22, .. , a2y,
                   ...,
                   ax1, ax2, .. , axy}

      B[ Y x Z ] = { b11, b21, .., b(y-1)1, by1, b12, b22, .. , b(y-1)z, byz }

      C[ X x Z ] = { c11, c21, .. , c(x-1)1, cx1, c12, c22, .. ,c(x-1)z, cxz }

  Source: DSP-Stone
          http://www.ice.rwth-aachen.de/research/tools-projects/entry/detail/dspstone

  Changes: no major functional changes

  License: may be used, modified, and re-distributed freely

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

const X: usize = 10;
const Y: usize = 10;
const Z: usize = 10;

fn matrix1_init(a: &mut [i32; X * Y], b: &mut [i32; Y * Z], c: &mut [i32; X * Z]) {
    let x = 1;
    // loopbound min 100 max 100
    for aa in a {
        *aa = black_box(x);
    }
    // loopbound min 100 max 100
    for bb in b {
        *bb = black_box(x);
    }
    // loopbound min 100 max 100
    for cc in c {
        *cc = 0;
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn matrix1_main(a: &[i32; X * Y], b: &[i32; Y * Z], c: &mut [i32; X * Z]) {
    // loopbound min 10 max 10
    for k in 0..Z {
        // loopbound min 10 max 10
        for i in 0..X {
            let mut v = 0;
            // loopbound min 10 max 10
            for f in 0..Y {
                v += a[i * Y + f] * b[f * Z + k];
            }
            c[i * Z + k] = v;
        }
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn matrix1_log(a: &mut [i32; X * Y], b: &mut [i32; Y * Z], c: &mut [i32; X * Z]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        matrix1_init(black_box(a), black_box(b), black_box(c));
        let start = black_box(DWT::cycle_count());
        matrix1_main(black_box(a), black_box(b), black_box(c));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("matrix1 - elapsed: {}", cycles);
    });

    c.iter().sum::<i32>() == 1000
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn matrix1_std(a: &mut [i32; X * Y], b: &mut [i32; Y * Z], c: &mut [i32; X * Z]) -> bool {
    matrix1_init(black_box(a), black_box(b), black_box(c));
    matrix1_main(black_box(a), black_box(b), black_box(c));

    c.iter().sum::<i32>() == 1000
}

fn matrix1() -> bool {
    static mut A: [i32; X * Y] = [0; X * Y];
    static mut B: [i32; Y * Z] = [0; Y * Z];
    static mut C: [i32; X * Z] = [0; X * Z];
    // SAFETY: This is the only place we ever reference these and this function is only ever called once.
    let a = unsafe { &mut *{ &raw mut A } };
    let b = unsafe { &mut *{ &raw mut B } };
    let c = unsafe { &mut *{ &raw mut C } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(matrix1_log(black_box(a), black_box(b), black_box(c)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(matrix1_std(black_box(a), black_box(b), black_box(c)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(matrix1());

    #[cfg(feature = "log")]
    {
        rprintln!("matrix1: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(matrix1());

    #[cfg(feature = "log")]
    std::println!("matrix1: {}", success);

    std::process::exit(!success as i32);
}
