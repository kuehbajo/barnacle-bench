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

  Name: fft

  Author: Juan Martinez Velarde

  Function: benchmarking of an integer stage scaling FFT
    To avoid errors caused by overflow and bit growth,
    the input data is scaled. Bit growth occurs potentially
    at butterfly operations, which involve a complex
    multiplication, a complex addition and a complex
    subtraction. Maximal bit growth from butterfly input
    to butterfly output is two bits.

    The input data includes enough extra sign bits, called
    guard bits, to ensure that bit growth never results in
    overflow (Rabiner and Gold, 1975). Data can grow by a
    maximum factor of 2.4 from butterfly input to output
    (two bits of grow). However, a data value cannot grow by
    maximum amount in two consecutive stages.
    The number of guard bits necessary to compensate the
    maximum bit growth in an N-point FFT is (log_2 (N))+1).

    In a 16-point FFT (requires 4 stages), each of the
    input samples should contain 5 guard bits. The input
    data is then restricted to 10 bits, one sign bit and
    nine magnitude bits, in order to prevent an
    overflow from the integer multiplication with the
    precalculed twiddle coefficients.

    Another method to compensate bit growth is to scale the
    outputs down by a factor of two unconditionally after
    each stage. This approach is called unconditional scaling

    Initially, 2 guard bits are included in the input data to
    accomodate the maximum overflow in the first stage.
    In each butterfly of a stage calculation, the data can
    grow into the guard bits. To prevent overflow in the next
    stage, the guard bits are replaced before the next stage is
    executed by shifting the entire block of data one bit
    to the right.

    Input data should not be restricted to a 1.9 format.
    Input data can be represented in a 1.13 format,that is
    14 significant bits, one sign and 13 magnitude bits. In
    the FFT calculation, the data loses a total of (log2 N) -1
    bits because of shifting. Unconditional scaling results
    in the same number of bits lost as in the input data scaling.
    However, it produces more precise results because the
    FFT starts with more precise input data. The tradeoff is
    a slower FFT calculation because of the extra cycles needed
    to shift the output of each stage.

  Source: DSP-Stone
    http://www.ice.rwth-aachen.de/research/tools-projects/entry/detail/dspstone/

  Original name: fft_1024_13
    (merged main1024_bit_reduct and fft_bit_reduct from DSP-Stone)

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

mod input;

use core::hint::black_box;
use input::{FFT_INPUT, FFT_TWIDTABLE};

const N_FFT: usize = 1024;
const NUMBER_OF_BITS: usize = 13; /* fract format 1.NUMBER_OF_BITS = 1.13 */

const BITS_PER_TWID: usize = 13; /* bits per twiddle coefficient */
const SHIFT: usize = BITS_PER_TWID; /* fractional shift after each multiplication */

fn fft_exp2f(x: f32) -> f32 {
    let mut res = 2.;
    // loopbound min 13 max 13
    for _ in 1..(x as i32) {
        res *= 2.;
    }
    res
}

fn fft_modff(x: f32, intpart: &mut f32) -> f32 {
    *intpart = x as i32 as f32;
    x - *intpart
}

fn fft_convert(value: f32) -> i32 {
    let mut man = NUMBER_OF_BITS as f32;
    let exponent = NUMBER_OF_BITS as f32;

    let m = fft_exp2f(exponent + 1.) - 1.;

    let t_val = value * m;
    let frac = fft_modff(t_val, &mut man);
    let rnd_val = if frac < 0.0 {
        if frac > -0.5 { 0 } else { -1 }
    } else if frac < 0.5 {
        0
    } else {
        1
    };
    let int_val = (man as i64 + rnd_val) as u64;
    let pm_val = int_val;
    pm_val as i32
}

fn fft_float2fract(fft_inputfract: &mut [i32; N_FFT]) {
    // loopbound min 1024 max 1024
    for (f, &i) in fft_inputfract.iter_mut().zip(&FFT_INPUT) {
        *f = fft_convert(i);
    }
}

fn fft_pin_down(fft_input_data: &mut [i32; 2 * N_FFT], fft_inputfract: &mut [i32; N_FFT]) {
    /* conversion from input to a 1.13 format */
    fft_float2fract(fft_inputfract);

    // loopbound min 1024 max 1024
    for (d, &i) in fft_input_data.chunks_exact_mut(2).zip(&*fft_inputfract) {
        d[0] = i; /* fill in with real data */
        d[1] = 0; /* imaginary data is equal zero */
    }
}

fn fft_init(fft_input_data: &mut [i32; 2 * N_FFT], fft_inputfract: &mut [i32; N_FFT]) {
    fft_pin_down(fft_input_data, fft_inputfract);
}

fn fft_bit_reduct(fft_input_data: &mut [i32; 2 * N_FFT]) {
    let mut max = 2;
    let n = N_FFT << 1;

    let mut j = 0;

    // loopbound min 1024 max 1024
    for i in (0..n - 1).step_by(2) {
        if j > i {
            fft_input_data.swap(j, i);
            fft_input_data.swap(j + 1, i + 1);
        }

        let mut m = N_FFT;

        // loopbound min 0 max 10
        while m >= 2 && j >= m {
            j -= m;
            m >>= 1;
        }
        j += m;
    }

    let mut fr = 0;

    // loopbound min 10 max 10
    while n > max {
        let level = max << 1;

        // loopbound min 1 max 512
        for m in (1..max).step_by(2) {
            let l = FFT_TWIDTABLE[fr];
            let k = FFT_TWIDTABLE[fr + 1];
            fr += 2;

            // loopbound min 1 max 512
            for i in (m..n).step_by(level) {
                let (q, p) = fft_input_data.split_at_mut(i + 1);
                let p = &mut p[max - 1 - 1..=max - 1];
                let q = &mut q[i - 1..=i];

                let mut tmpr = l * p[0];
                tmpr -= k * p[1];

                let mut tmpi = l * p[1];
                tmpi += k * p[0];

                tmpr >>= SHIFT;
                tmpi >>= SHIFT;

                p[0] = q[0] - tmpr;
                p[1] = q[1] - tmpi;

                q[0] += tmpr;
                q[1] += tmpi;
            }
        }

        /* implement unconditional bit reduction */

        // loopbound min 2048 max 2048
        for d in fft_input_data.iter_mut() {
            *d >>= 1;
        }

        max = level;
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn fft_main(fft_input_data: &mut [i32; 2 * N_FFT]) {
    fft_bit_reduct(fft_input_data);
}

fn fft_return(fft_input_data: &[i32; 2 * N_FFT]) -> bool {
    // loopbound min 2048 max 2048
    let sum = fft_input_data.iter().sum::<i32>();
    sum == 3968
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn fft_log(fft_input_data: &mut [i32; 2 * N_FFT], fft_inputfract: &mut [i32; N_FFT]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {        
        fft_init(black_box(fft_input_data), black_box(fft_inputfract));
        let start = black_box(DWT::cycle_count());
        fft_main(black_box(fft_input_data));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("fft - elapsed: {}", cycles);
    });

    black_box(fft_return(black_box(fft_input_data)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn fft_std(fft_input_data: &mut [i32; 2 * N_FFT], fft_inputfract: &mut [i32; N_FFT]) -> bool {
    fft_init(black_box(fft_input_data), black_box(fft_inputfract));
    fft_main(black_box(fft_input_data));

    black_box(fft_return(black_box(fft_input_data)))
}

fn fft() -> bool {
    static mut FFT_INPUT_DATA: [i32; 2 * N_FFT] = [0; 2 * N_FFT];
    /* will hold the transformed data */
    static mut FFT_INPUTFRACT: [i32; N_FFT] = [0; N_FFT];

    // SAFETY: This is the only place we ever reference these and this function is not reentrant.
    let fft_input_data = unsafe { &mut *{ &raw mut FFT_INPUT_DATA } };
    let fft_inputfract = unsafe { &mut *{ &raw mut FFT_INPUTFRACT } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(fft_log(black_box(fft_input_data), black_box(fft_inputfract)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(fft_std(black_box(fft_input_data), black_box(fft_inputfract)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(fft());

    #[cfg(feature = "log")]
    {
        rprintln!("fft: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(fft());

    #[cfg(feature = "log")]
    std::println!("fft: {}", success);

    std::process::exit(!success as i32);
}
