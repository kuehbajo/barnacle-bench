/*

  Rust Implementation

  Author: David Heim
  License: The Rust implementation shares the same license (MIT) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.
  
  Version V 2.0

  Name: bitonic

  Author: Chris Leger

  Function: bitonic implements a recursive sorting network algorithm.

  Source: MiBench
          http://wwweb.eecs.umich.edu/mibench

  Changes: no major functional changes

  License: MIT

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::{cmp::Ordering, hint::black_box};

const BITONIC_CHECKSUM: i32 = 55;

#[derive(Debug, Copy, Clone)]
enum BitonicDir {
    Ascending,
    Descending,
}

fn bitonic_init(data: &mut [i32]) {
    for (i, x) in data.iter_mut().enumerate() {
        *x = 32 - (i as i32);
    }
}

fn bitonic_compare(arr: &mut [i32; 32], i: usize, j: usize, dir: BitonicDir) {
    match (dir, arr[i].cmp(&arr[j])) {
        (BitonicDir::Ascending, Ordering::Greater) | (BitonicDir::Descending, Ordering::Less) => {
            arr.swap(i, j);
        }
        _ => {}
    }
}

fn bitonic_merge(arr: &mut [i32; 32], lo: usize, cnt: usize, dir: BitonicDir) {
    let k = cnt / 2;
    // loopbound min 0 max 16
    for i in lo..(lo + k) {
        bitonic_compare(arr, i, i + k, dir);
    }

    if k > 1 {
        bitonic_merge(arr, lo, k, dir);
        bitonic_merge(arr, lo + k, k, dir);
    }
}

fn bitonic_sort(arr: &mut [i32; 32], lo: usize, cnt: usize, dir: BitonicDir) {
    let mut k = cnt;
    k /= 2;

    // marker recMerge
    if cnt > 1 {
        bitonic_sort(arr, lo, k, BitonicDir::Ascending);
        bitonic_sort(arr, lo, k, BitonicDir::Descending);
    }

    bitonic_merge(arr, lo, cnt, dir);
    // flowrestriction 1*bitonicMerge <= 31*recMerge
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn bitonic_main(arr: &mut [i32; 32]) {
    // marker recSort
    bitonic_sort(arr, 0, 32, BitonicDir::Ascending);
    // flowrestriction 1*bitonicSort <= 63*recSort
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn bitonic_log(data: &mut [i32; 32]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        bitonic_init(black_box(data));
        let start = black_box(DWT::cycle_count());
        bitonic_main(black_box(data));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("bitonic - elapsed: {}", cycles);
    });

    let checksum = black_box(data[0]) + black_box(data[21]) + black_box(data[31]);
    checksum == BITONIC_CHECKSUM
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn bitonic_std(data: &mut [i32; 32]) -> bool {
    bitonic_init(black_box(data));
    bitonic_main(black_box(data));

    let checksum = black_box(data[0]) + black_box(data[21]) + black_box(data[31]);
    checksum == BITONIC_CHECKSUM
}

fn bitonic() -> bool {
    static mut DATA: [i32; 32] = [0; 32];
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let data = unsafe { &mut *{ &raw mut DATA } };
    
    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(bitonic_log(black_box(data)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(bitonic_std(black_box(data)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(bitonic());
    
    #[cfg(feature = "log")]
    {
        rprintln!("bitonic: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(bitonic());

    #[cfg(feature = "log")]
    std::println!("bitonic: {}", success);

    std::process::exit(!success as i32);
}
