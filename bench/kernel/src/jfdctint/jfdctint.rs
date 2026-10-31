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

  Name: jfdctint

  Author: Thomas G. Lane, Public domain JPEG source code.
          Modified by Steven Li at Princeton University.

  Function: JPEG slow-but-accurate integer implementation of the
            forward  DCT (Discrete Cosine Transform) on a 8x8
            pixel block [from original file documentations]

   Copyright (C) 1991-1994, Thomas G. Lane.
   This file is part of the Independent JPEG Group's software.
   For conditions of distribution and use, see the accompanying README file.

   This file contains a slow-but-accurate integer implementation of the
   forward DCT (Discrete Cosine Transform).

   A 2-D DCT can be done by 1-D DCT on each row followed by 1-D DCT
   on each column.  Direct algorithms are also available, but they are
   much more complex and seem not to be any faster when reduced to code.

   This implementation is based on an algorithm described in
     C. Loeffler, A. Ligtenberg and G. Moschytz, "Practical Fast 1-D DCT
     Algorithms with 11 Multiplications", Proc. Int'l. Conf. on Acoustics,
     Speech, and Signal Processing 1989 (ICASSP '89), pp. 988-991.
   The primary algorithm described there uses 11 multiplies and 29 adds.
   We use their alternate method with 12 multiplies and 32 adds.
   The advantage of this method is that no data path contains more than one
   multiplication; this allows a very simple and accurate implementation in
   scaled fixed-point arithmetic, with a minimal number of shifts.

  Source: SNU-RT Benchmark Suite for Worst Case Timing Analysis
          Collected and Modified by S.-S. Lim
          Real-Time Research Group
          Seoul National University

  Changes: Moved initialisation code from jfdctint_main() to jfdctint_init(),
           added checksum calculation in jfdctint_return()

  License: see README

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

const DCTSIZE: usize = 8;
const fn descale(x: i32, n: i32) -> i32 {
    (x + (1 << (n - 1))) >> n
}

const CONST_BITS: i32 = 13;
const PASS1_BITS: i32 = 2;

const FIX_0_298631336: i32 = 2446;
const FIX_0_390180644: i32 = 3196;
const FIX_0_541196100: i32 = 4433;
const FIX_0_765366865: i32 = 6270;
const FIX_0_899976223: i32 = 7373;
const FIX_1_175875602: i32 = 9633;
const FIX_1_501321110: i32 = 12299;
const FIX_1_847759065: i32 = 15137;
const FIX_1_961570560: i32 = 16069;
const FIX_2_053119869: i32 = 16819;
const FIX_2_562915447: i32 = 20995;
const FIX_3_072711026: i32 = 25172;

const CHECKSUM: i32 = 1668124;

fn jfdctint_init(data: &mut [i32]) {
    let mut seed = 1;
    // loopbound min 64 max 64
    for d in data {
        seed = ((seed * 133) + 81) % 65535;
        *d = seed;
    }
}

fn jfdctint_return(data: &[i32]) -> bool {
    data.iter().sum::<i32>() == CHECKSUM
}

fn jfdctint_jpeg_fdct_islow(data: &mut [i32]) {
    // loopbound min 8 max 8
    for datap in data.chunks_exact_mut(DCTSIZE) {
        let tmp0 = datap[0] + datap[7];
        let tmp7 = datap[0] - datap[7];
        let tmp1 = datap[1] + datap[6];
        let tmp6 = datap[1] - datap[6];
        let tmp2 = datap[2] + datap[5];
        let tmp5 = datap[2] - datap[5];
        let tmp3 = datap[3] + datap[4];
        let tmp4 = datap[3] - datap[4];

        let tmp10 = tmp0 + tmp3;
        let tmp13 = tmp0 - tmp3;
        let tmp11 = tmp1 + tmp2;
        let tmp12 = tmp1 - tmp2;

        datap[0] = (tmp10 + tmp11) << PASS1_BITS;
        datap[4] = (tmp10 - tmp11) << PASS1_BITS;

        let z1 = (tmp12 + tmp13) * FIX_0_541196100;
        datap[2] = descale(z1 + tmp13 * FIX_0_765366865, CONST_BITS - PASS1_BITS);
        datap[6] = descale(z1 + tmp12 * (-FIX_1_847759065), CONST_BITS - PASS1_BITS);

        let z1 = tmp4 + tmp7;
        let z2 = tmp5 + tmp6;
        let z3 = tmp4 + tmp6;
        let z4 = tmp5 + tmp7;
        let z5 = (z3 + z4) * FIX_1_175875602; /* sqrt(2) * c3 */

        let tmp4 = tmp4 * FIX_0_298631336; /* sqrt(2) * (-c1+c3+c5-c7) */
        let tmp5 = tmp5 * FIX_2_053119869; /* sqrt(2) * ( c1+c3-c5+c7) */
        let tmp6 = tmp6 * FIX_3_072711026; /* sqrt(2) * ( c1+c3+c5-c7) */
        let tmp7 = tmp7 * FIX_1_501321110; /* sqrt(2) * ( c1+c3-c5-c7) */
        let z1 = z1 * (-FIX_0_899976223); /* sqrt(2) * (c7-c3) */
        let z2 = z2 * (-FIX_2_562915447); /* sqrt(2) * (-c1-c3) */
        let mut z3 = z3 * (-FIX_1_961570560); /* sqrt(2) * (-c3-c5) */
        let mut z4 = z4 * (-FIX_0_390180644); /* sqrt(2) * (c5-c3) */

        z3 += z5;
        z4 += z5;

        datap[7] = descale(tmp4 + z1 + z3, CONST_BITS - PASS1_BITS);
        datap[5] = descale(tmp5 + z2 + z4, CONST_BITS - PASS1_BITS);
        datap[3] = descale(tmp6 + z2 + z3, CONST_BITS - PASS1_BITS);
        datap[1] = descale(tmp7 + z1 + z4, CONST_BITS - PASS1_BITS);
    }

    let mut datap = &mut data[..];
    // loopbound min 8 max 8
    for _ in 0..DCTSIZE {
        let tmp0 = datap[0] + datap[DCTSIZE * 7];
        let tmp7 = datap[0] - datap[DCTSIZE * 7];
        let tmp1 = datap[DCTSIZE] + datap[DCTSIZE * 6];
        let tmp6 = datap[DCTSIZE] - datap[DCTSIZE * 6];
        let tmp2 = datap[DCTSIZE * 2] + datap[DCTSIZE * 5];
        let tmp5 = datap[DCTSIZE * 2] - datap[DCTSIZE * 5];
        let tmp3 = datap[DCTSIZE * 3] + datap[DCTSIZE * 4];
        let tmp4 = datap[DCTSIZE * 3] - datap[DCTSIZE * 4];

        let tmp10 = tmp0 + tmp3;
        let tmp13 = tmp0 - tmp3;
        let tmp11 = tmp1 + tmp2;
        let tmp12 = tmp1 - tmp2;

        datap[0] = descale(tmp10 + tmp11, PASS1_BITS);
        datap[DCTSIZE * 4] = descale(tmp10 - tmp11, PASS1_BITS);

        let z1 = (tmp12 + tmp13).wrapping_mul(FIX_0_541196100);
        datap[DCTSIZE * 2] = descale(
            z1 + tmp13.wrapping_mul(FIX_0_765366865),
            CONST_BITS + PASS1_BITS,
        );
        datap[DCTSIZE * 6] = descale(
            z1.wrapping_add(tmp12.wrapping_mul(-FIX_1_847759065)),
            CONST_BITS + PASS1_BITS,
        );

        let z1 = tmp4 + tmp7;
        let z2 = tmp5 + tmp6;
        let z3 = tmp4 + tmp6;
        let z4 = tmp5 + tmp7;
        let z5 = (z3 + z4).wrapping_mul(FIX_1_175875602); /* sqrt(2) * c3 */

        let tmp4 = tmp4.wrapping_mul(FIX_0_298631336); /* sqrt(2) * (-c1+c3+c5-c7) */
        let tmp5 = tmp5.wrapping_mul(FIX_2_053119869); /* sqrt(2) * ( c1+c3-c5+c7) */
        let tmp6 = tmp6.wrapping_mul(FIX_3_072711026); /* sqrt(2) * ( c1+c3+c5-c7) */
        let tmp7 = tmp7.wrapping_mul(FIX_1_501321110); /* sqrt(2) * ( c1+c3-c5-c7) */
        let z1 = z1.wrapping_mul(-FIX_0_899976223); /* sqrt(2) * (c7-c3) */
        let z2 = z2.wrapping_mul(-FIX_2_562915447); /* sqrt(2) * (-c1-c3) */
        let mut z3 = z3.wrapping_mul(-FIX_1_961570560); /* sqrt(2) * (-c3-c5) */
        let mut z4 = z4.wrapping_mul(-FIX_0_390180644); /* sqrt(2) * (c5-c3) */

        z3 = z3.wrapping_add(z5);
        z4 = z4.wrapping_add(z5);

        datap[DCTSIZE * 7] = descale(
            tmp4.wrapping_add(z1).wrapping_add(z3),
            CONST_BITS + PASS1_BITS,
        );
        datap[DCTSIZE * 5] = descale(
            tmp5.wrapping_add(z2).wrapping_add(z4),
            CONST_BITS + PASS1_BITS,
        );
        datap[DCTSIZE * 3] = descale(
            tmp6.wrapping_add(z2).wrapping_add(z3),
            CONST_BITS + PASS1_BITS,
        );
        datap[DCTSIZE] = descale(
            tmp7.wrapping_add(z1).wrapping_add(z4),
            CONST_BITS + PASS1_BITS,
        );

        datap = &mut datap[1..]; /* advance to next column */
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn jfdctint_main(data: &mut [i32]) {
    jfdctint_jpeg_fdct_islow(data);
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn jfdctint_log(data: &mut [i32]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        jfdctint_init(black_box(data));
        let start = black_box(DWT::cycle_count());
        jfdctint_main(black_box(data));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("jfdctint - elapsed: {}", cycles);
    });

    black_box(jfdctint_return(black_box(data)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn jfdctint_std(data: &mut [i32]) -> bool {
    jfdctint_init(black_box(data));
    jfdctint_main(black_box(data));

    black_box(jfdctint_return(black_box(data)))
}

fn jfdctint() -> bool {
    static mut DATA: [i32; 64] = [0; 64];
    // SAFETY: This is the only place we ever reference these and main is not reentrant.
    let data = unsafe { &mut *{ &raw mut DATA } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(jfdctint_log(black_box(data)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(jfdctint_std(black_box(data)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(jfdctint());

    #[cfg(feature = "log")]
    {
        rprintln!("jfdctint: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(jfdctint());

    #[cfg(feature = "log")]
    std::println!("jfdctint: {}", success);

    std::process::exit(!success as i32);
}
