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

  Name: fir2dim

  Author: Juan Martinez Velarde

  Function: prime calculates whether numbers are prime.

  Source: DSP-Stone
          http://www.ice.rwth-aachen.de/research/tools-projects/entry/detail/dspstone/

  Original name: fir2dim_float

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

fn fir2dim_init(
    image: &mut [f32; 4 * 4],
    array: &mut [f32; 6 * 6],
    coefficients: &mut [f32; 3 * 3],
    output: &mut [f32; 4 * 4],
) {
    let bitmask = 0.;

    for x in image {
        *x += black_box(bitmask);
    }
    for x in coefficients {
        *x += black_box(bitmask);
    }
    for x in array {
        *x += black_box(bitmask);
    }
    for x in output {
        *x += black_box(bitmask);
    }
}

fn fir2dim_pin_down(
    image: &mut [f32; 4 * 4],
    array: &mut [f32; 6 * 6],
    coefficients: &mut [f32; 3 * 3],
    output: &mut [f32; 4 * 4],
) {
    // loopbound min 16 max 16
    for i in image.iter_mut() {
        *i = 1.;
    }

    // loopbound min 9 max 9
    for c in coefficients {
        *c = 1.;
    }

    // loopbound min 6 max 6
    for a in &mut array[0..6] {
        *a = 0.
    }
    let mut array_index = 6;
    // loopbound min 4 max 4
    for i in 0..4 {
        array[array_index] = 0.;
        array_index += 1;
        // loopbound min 4 max 4
        for j in 0..4 {
            array[array_index] = image[i * 4 + j];
            array_index += 1;
        }
        array[array_index] = 0.;
        array_index += 1;
    }
    // loopbound min 6 max 6
    for a in &mut array[array_index..] {
        *a = 0.
    }

    // loopbound min 16 max 16
    for o in output {
        *o = 0.;
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn fir2dim_main(
    image: &mut [f32; 4 * 4],
    array: &mut [f32; 6 * 6],
    coefficients: &mut [f32; 3 * 3],
    output: &mut [f32; 4 * 4],
) -> f32 {
    fir2dim_pin_down(image, array, coefficients, output);

    // loopbound min 4 max 4
    for k in 0..4 {
        // loopbound min 4 max 4
        for f in 0..4 {
            let array_offset = k * 6 + f;

            // loopbound min 3 max 3
            let result = array[array_offset..]
                .chunks_exact(3)
                .step_by(2)
                .flatten()
                .zip(&*coefficients)
                .map(|(&a, &c)| a * c)
                .sum::<f32>();

            output[k * 4 + f] = result;
        }
    }

    let result = output[0] + output[5] + array[9];

    fir2dim_pin_down(image, array, coefficients, output);

    result
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn fir2dim_log(
    image: &mut [f32; 4 * 4],
    array: &mut [f32; 6 * 6],
    coefficients: &mut [f32; 3 * 3],
    output: &mut [f32; 4 * 4],
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result : f32 = 0.;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        fir2dim_init(
            black_box(image),
            black_box(array),
            black_box(coefficients),
            black_box(output),
        );
        let start = black_box(DWT::cycle_count());
        result = black_box(fir2dim_main(
            black_box(image),
            black_box(array),
            black_box(coefficients),
            black_box(output),
        ));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("fir2dim - elapsed: {}", cycles);
    });

    result == 14.
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn fir2dim_std(
    image: &mut [f32; 4 * 4],
    array: &mut [f32; 6 * 6],
    coefficients: &mut [f32; 3 * 3],
    output: &mut [f32; 4 * 4],
) -> bool {
    fir2dim_init(
        black_box(image),
        black_box(array),
        black_box(coefficients),
        black_box(output),
    );
    let result = black_box(fir2dim_main(
        black_box(image),
        black_box(array),
        black_box(coefficients),
        black_box(output),
    ));

    result == 14.
}

fn fir2dim() -> bool {
    static mut COEFFICIENTS: [f32; 3 * 3] = [1.; 3 * 3];
    static mut IMAGE: [f32; 4 * 4] = [0.; 4 * 4];
    static mut ARRAY: [f32; 6 * 6] = [0.; 6 * 6];
    static mut OUTPUT: [f32; 4 * 4] = [0.; 4 * 4];
    // SAFETY: This is the only place we ever reference these and main is not reentrant.
    let coefficients = unsafe { &mut *{ &raw mut COEFFICIENTS } };
    let image = unsafe { &mut *{ &raw mut IMAGE } };
    let array = unsafe { &mut *{ &raw mut ARRAY } };
    let output = unsafe { &mut *{ &raw mut OUTPUT } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(fir2dim_log(
            black_box(image),
            black_box(array),
            black_box(coefficients),
            black_box(output),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(fir2dim_std(
            black_box(image),
            black_box(array),
            black_box(coefficients),
            black_box(output),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(fir2dim());

    #[cfg(feature = "log")]
    {
        rprintln!("fir2dim: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(fir2dim());

    #[cfg(feature = "log")]
    std::println!("fir2dim: {}", success);

    std::process::exit(!success as i32);
}
