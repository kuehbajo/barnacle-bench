/*

  Rust Implementation

  Author: David Heim
  License: The Rust implementation shares the same license (MIT) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  Version 2.0

  Name: filterbank

  Author: unknown

  Function: Creates a filter bank to perform multirate signal processing.
            The coefficients for the sets of filters are created in the
            top-level init function, and passed down through the init
            functions to FIR filter objects.
            On each branch, a delay, filter, and downsample is performed,
            followed by an upsample, delay, and filter.

  Source: StreamIt
          (http://groups.csail.mit.edu/cag/streamit/shtml/benchmarks.shtml)

  Original name: filterbank

  Changes: See ChangeLog.txt

  License: MIT License

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::{array, hint::black_box};

fn filterbank_core(r: &[f32; 256], y: &mut [f32; 256], h: &[[f32; 32]; 8], f: &[[f32; 32]; 8]) {
    // loopbound min 256 max 256
    y.fill(0.);

    // loopbound min 8 max 8
    for i in 0..8 {
        let mut vect_h = [0.; 256]; /* (output of the H) */
        let mut vect_dn = [0.; 256 / 8]; /* output of the down sampler; */
        let mut vect_up = [0.; 256]; /* output of the up sampler; */
        let mut vect_f = [0.; 256]; /* this is the output of the */

        /* convolving H */
        // loopbound min 256 max 256
        for j in 0..256 {
            let mut k = 0;
            // loopbound min 1 max 32
            while k < 32 && j >= k {
                vect_h[j] += h[i][k] * r[j - k];
                k += 1;
            }
        }

        /* Down Sampling */
        // loopbound min 32 max 32
        for (j, &hv) in vect_h.iter().step_by(8).enumerate() {
            vect_dn[j] = hv;
        }

        /* Up Sampling */
        // loopbound min 32 max 32
        for (j, &dn) in vect_dn.iter().enumerate() {
            vect_up[j * 8] = dn;
        }

        /* convolving F */
        // loopbound min 256 max 256
        for j in 0..256 {
            let mut k = 0;
            // loopbound min 1 max 32
            while k < 32 && j >= k {
                vect_f[j] += f[i][k] * vect_up[j - k];
                k += 1;
            }
        }

        /* adding the results to the y matrix */
        // loopbound min 256 max 256
        for (j, yy) in y.iter_mut().enumerate() {
            *yy += vect_f[j];
        }
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn filterbank_main(numiters: i32) -> i32 {
    // loopbound min 256 max 256
    let r: [f32; 256] = array::from_fn(|i| (i + 1) as f32);
    let mut y = [0.; 256];

    let mut h = [[0.; 32]; 8];
    let mut f = [[0.; 32]; 8];

    // loopbound min 32 max 32
    for i in 0..32 {
        // loopbound min 8 max 8
        for j in 0..8 {
            h[j][i] = (i * 32 + j * 8 + j + i + j + 1) as f32;
            f[j][i] = (i * j + j * j + j + i) as f32;
        }
    }

    // loopbound min 2 max 2
    for _ in 0..numiters {
        filterbank_core(&r, &mut y, &h, &f);
    }

    y[0] as i32
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn filterbank_log() -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let numiters = black_box(2);
    let mut result: i32 = 0;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        let start = black_box(DWT::cycle_count());
        result = black_box(filterbank_main(black_box(numiters)));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("filterbank - elapsed: {}", cycles);
    });

    result == 9408
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn filterbank_std() -> bool {
    let numiters = black_box(2);

    let result = black_box(filterbank_main(black_box(numiters)));

    result == 9408
}

fn filterbank() -> bool {
    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(filterbank_log())
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(filterbank_std())
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(filterbank());

    #[cfg(feature = "log")]
    {
        rprintln!("filterbank: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(filterbank());

    #[cfg(feature = "log")]
    std::println!("filterbank: {}", success);

    std::process::exit(!success as i32);
}
