/*

  Rust Implementation

  Author: David Heim
  License: The Rust implementation shares the same license (ISC) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  Version V 2.0

  Name: lms

  Author: Jörg Mische

  Function: LMS adaptive signal enhancement

  Source: Completely rewritten for TACLeBench to avoid license issues.
          It has the same functionality as lms.c from the book
          "C Algorithms for Real-Time DSP" by Paul M. Embree, which
          has been used in WCET benchmarking for many years.

  Original name: lms.c

  Changes: Simplified generation of the input (noisy sinus wave).
           No static variables.

  License: ISC (simplified BSD)

*/

/*
  Copyright (c) 2016 Jörg Mische

  Permission to use, copy, modify, and/or distribute this software for any
  purpose with or without fee is hereby granted, provided that the above
  copyright notice and this permission notice appear in all copies.

  THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
  WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
  MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
  ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
  WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
  ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT
  OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

const N: usize = 201;
const L: usize = 20;
const SAMPLING: usize = 5;

const SINTAB: [f64; SAMPLING + 1] = [
    0.0,
    0.437_016_036_207_159,
    0.831_253_895_559_386,
    1.144_122_827_436_525_6,
    1.344_997_039_209_976_4,
    1.414_213_562_373_093_8,
];

fn lms_sinus(i: usize) -> f64 {
    let s = i % (4 * SAMPLING);
    if s > (3 * SAMPLING) {
        -SINTAB[4 * SAMPLING - s]
    } else if s >= (2 * SAMPLING) {
        -SINTAB[s - 2 * SAMPLING]
    } else if s > SAMPLING {
        SINTAB[2 * SAMPLING - s]
    } else {
        SINTAB[s]
    }
}

fn lms_init(input: &mut [f32; N + 1]) {
    let mut seed: u32 = 1;

    input[0] = 0.0;
    {
        const SCALE_FACTOR: f64 = 0.000_000_000_931_322_574_615_478_5;
        let mut r: f64;
        let mut v1: f64;
        let mut v2: f64;
        loop {
            // generate two random numbers between -1.0 and +1.0
            seed = seed.wrapping_mul(1103515245) + 12345;
            v1 = (seed & 0x00007fffffff) as f64 * SCALE_FACTOR - 1.0;
            seed = seed.wrapping_mul(1103515245) + 12345;
            v2 = (seed & 0x00007fffffff) as f64 * SCALE_FACTOR - 1.0;
            r = v1 * v1 + v2 * v2;
            if r <= 1.0 {
                break;
            }
        }
        // radius < 1

        // remap v1 and v2 to two Gaussian numbers
        let noise = r.recip(); // approximation of sqrt(0.96) * sqrt(-log(r)/r);
        input[1] = (lms_sinus(1) + noise * v2) as f32;
    }

    // loopbound min 100 max 100
    for k in (2..N).step_by(2) {
        const SCALE_FACTOR: f64 = 0.000_000_000_931_322_574_615_478_5;
        let mut r: f64;
        let mut v1: f64;
        let mut v2: f64;
        loop {
            // generate two random numbers between -1.0 and +1.0
            seed = seed.wrapping_mul(1103515245) + 12345;
            v1 = (seed & 0x00007fffffff) as f64 * SCALE_FACTOR - 1.0;
            seed = seed.wrapping_mul(1103515245) + 12345;
            v2 = (seed & 0x00007fffffff) as f64 * SCALE_FACTOR - 1.0;
            r = v1 * v1 + v2 * v2;
            if r <= 1.0 {
                break;
            }
        }
        // radius < 1

        // remap v1 and v2 to two Gaussian numbers
        let noise = r.recip(); // approximation of sqrt(0.96) * sqrt(-log(r)/r);
        input[k] = (lms_sinus(k) + noise * v2) as f32;
        input[k + 1] = (lms_sinus(k + 1) + noise * v1) as f32;
    }
}

fn lms_calc(
    x: f32,
    d: f32,
    b: &mut [f32; L + 1],
    mu: f32,
    alpha: f32,
    history: &mut [f32; L + 1],
    sigma: &mut f32,
) -> f32 {
    // shift history
    // loopbound min 20 max 20
    history.rotate_right(1);
    history[0] = x;

    // calculate filter
    let mut y = 0.0;
    *sigma = alpha * x * x + (1. - alpha) * (*sigma);
    // loopbound min 21 max 21
    for (bi, hi) in b.iter().zip(&*history) {
        y += bi * hi;
    }

    // update coefficients
    let e = mu * (d - y) / (*sigma);
    // loopbound min 21 max 21
    for (bi, hi) in b.iter_mut().zip(&*history) {
        *bi += e * hi;
    }

    y
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn lms_main(input: &[f32], output: &mut [f32]) {
    let mut b = [0.0; L + 1];
    let mut history = [0.0; L + 1];
    let mut sigma = 2.0;

    // loopbound min 201 max 201
    for (i, o) in input.windows(2).zip(output).take(N) {
        *o = lms_calc(
            i[0],
            i[1],
            &mut b,
            0.02 / (L + 1) as f32,
            0.01,
            &mut history,
            &mut sigma,
        )
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn lms_log(input: &mut [f32; N + 1], output: &mut [f32]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        lms_init(black_box(input));
        let start = black_box(DWT::cycle_count());
        lms_main(black_box(input), black_box(output));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("lms - elapsed: {}", cycles);
    });

    let result: f32 = output.iter().sum();
    (1000000.0 * (result + 4.705719)) as i32 == 0
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn lms_std(input: &mut [f32; N + 1], output: &mut [f32]) -> bool {
    lms_init(black_box(input));
    lms_main(black_box(input), black_box(output));

    let result: f32 = output.iter().sum();
    (1000000.0 * (result + 4.705719)) as i32 == 0
}

fn lms() -> bool {
    static mut INPUT: [f32; N + 1] = [0.0; N + 1];
    static mut OUTPUT: [f32; N + 1] = [0.0; N + 1];
    // SAFETY: This is the only place we ever reference these and this function is only ever called once.
    let input = unsafe { &mut *{ &raw mut INPUT } };
    let output = unsafe { &mut *{ &raw mut OUTPUT } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(lms_log(black_box(input), black_box(output)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(lms_std(black_box(input), black_box(output)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(lms());

    #[cfg(feature = "log")]
    {
        rprintln!("lms: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(lms());

    #[cfg(feature = "log")]
    std::println!("lms: {}", success);

    std::process::exit(!success as i32);
}
