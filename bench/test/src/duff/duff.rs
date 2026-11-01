/*

  Rust Implementation

  Author: kuehbajo
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

  Name: duff

  Author: Jakob Engblom

  Function: Duff's device

  Source: MRTC
          http://www.mrtc.mdh.se/projects/wcet/wcet_bench/duff/duff.c

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

/*
    Initialization- and return-value-related functions
*/

fn duff_init(source: &mut [i8]) {
    let bitmask: i8 = black_box(0);

    duff_initialize(source);

    /*
    Apply volatile XOR-bitmask to entire input array.
    */
    // loopbound min 400 max 400
    for p in source.iter_mut() {
        *p ^= black_box(bitmask);
    }
}

fn duff_return(target: &[i8]) -> bool {
    target[28] - 72 == 0
}


/*
  Algorithm core functions
*/

fn duff_initialize(arr: &mut [i8]) {
    let len = arr.len();
    // loopbound min 100 max 100
    for (i, elem) in arr.iter_mut().enumerate() {
        *elem = (len - i) as i8;
    }
}

fn duff_copy(to: &mut [i8], from: &[i8], count: usize) {
    let n = count / 8;
    let rem = count % 8;

    if rem > 0 {
       to[0..rem].copy_from_slice(&from[0..rem]); 
    }

    let mut i = rem;
    for _ in 0..n {
        to[i..i+8].copy_from_slice(&from[i..i+8]); 
        i += 8;
    }
    // flowrestriction 1*inside <= 6*outside
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn duff_main(to: &mut [i8], from: &mut [i8]) {
    duff_copy(to, from, 43);
}

/*
  Main functions
*/

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 25;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn duff_log(to: &mut [i8], from: &mut [i8]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: u32 = 0;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        duff_init(black_box(black_box(from)));
        let start = black_box(DWT::cycle_count());
        duff_main(black_box(to), black_box(from));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("duff - elapsed: {}", cycles);
    });

    black_box(duff_return(black_box(to)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn duff_std(to: &mut [i8], from: &mut [i8]) -> bool {
    duff_init(black_box(from));
    duff_main(black_box(to), black_box(from));

    black_box(duff_return(black_box(to)))
}

fn duff() -> bool{
    static mut SOURCE: [i8; 100] = [0; 100];
    static mut TARGET: [i8; 100] = [0; 100];

    // SAFETY: This is the only place we ever reference these and main is not reentrant.
    let duff_source = unsafe { &mut *{ &raw mut SOURCE } };
    let duff_target = unsafe { &mut *{ &raw mut TARGET } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(duff_log(black_box(duff_source), black_box(duff_target)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(duff_std(black_box(duff_source), black_box(duff_target)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(duff());

    #[cfg(feature = "log")]
    {
        rprintln!("duff: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(duff());

    #[cfg(feature = "log")]
    std::println!("duff: {}", success);

    std::process::exit(!success as i32);
}
