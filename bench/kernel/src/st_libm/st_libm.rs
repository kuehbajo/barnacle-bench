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

  Name: st

  Author: unknown

  Function: st is a statistics program.
    This program computes for two arrays of numbers the sum, the mean,
    the variance, and standard deviation. It then determines the
    correlation coefficient between the two arrays.

  Source: MRTC
          http://www.mrtc.mdh.se/projects/wcet/wcet_bench/st/st.c

  Changes: No major functional changes.

  License: May be used, modified, and re-distributed freely

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

struct RandomInt(i32);

impl RandomInt {
    fn init() -> Self {
        RandomInt(0)
    }

    fn next(&mut self) -> i32 {
        self.0 = ((black_box(self.0) * 133) + 81) % 8095;
        self.0
    }
}

fn st_initialize(array: &mut [f32], rand: &mut RandomInt) {
    // loopbound min 1000 max 1000
    for (i, a) in array.iter_mut().enumerate() {
        *a = (i as i32 + rand.next()) as f32;
    }
}

fn st_init(a: &mut [f32], b: &mut [f32]) {
    let mut rand = RandomInt::init();
    st_initialize(a, &mut rand);
    st_initialize(b, &mut rand);
}

fn st_sum_mean(array: &[f32], sum: &mut f32, mean: &mut f32) {
    // loopbound min 1000 max 1000
    *sum = array.iter().sum();
    *mean = *sum / array.len() as f32;
}

fn st_var_stddev(array: &[f32], mean: f32, var: &mut f32, stddev: &mut f32) {
    // loopbound min 1000 max 1000
    let diffs: f32 = array.iter().map(|&x| libm::powf(x - mean, 2.)).sum();

    *var = diffs / array.len() as f32;
    *stddev = libm::sqrtf(*var);
}

fn st_lincorrcoeff(a: &[f32], b: &[f32], mean_a: f32, mean_b: f32, coeff: &mut f32) {
    let mut numerator = 0.;
    let mut a_term = 0.;
    let mut b_term = 0.;
    
    // loopbound min 1000 max 1000
    for (a, b) in a.iter().zip(b) {
        numerator += (a - mean_a) * (b - mean_b);
        a_term += libm::powf(a - mean_a, 2.);
        b_term += libm::powf(b - mean_b, 2.);
    }

    *coeff = numerator / (libm::sqrtf(a_term) * libm::sqrtf(b_term));
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
#[expect(clippy::too_many_arguments)]
fn st_main(
    a: &mut [f32],
    b: &mut [f32],
    sum_a: &mut f32,
    mean_a: &mut f32,
    var_a: &mut f32,
    stddev_a: &mut f32,
    sum_b: &mut f32,
    mean_b: &mut f32,
    var_b: &mut f32,
    stddev_b: &mut f32,
    coeff: &mut f32,
) {
    st_sum_mean(a, sum_a, mean_a);
    st_var_stddev(a, *mean_a, var_a, stddev_a);

    st_sum_mean(b, sum_b, mean_b);
    st_var_stddev(b, *mean_b, var_b, stddev_b);

    st_lincorrcoeff(a, b, *mean_a, *mean_b, coeff);
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
#[expect(clippy::too_many_arguments)]
fn st_log(
    a: &mut [f32],
    b: &mut [f32],
    sum_a: &mut f32,
    mean_a: &mut f32,
    var_a: &mut f32,
    stddev_a: &mut f32,
    sum_b: &mut f32,
    mean_b: &mut f32,
    var_b: &mut f32,
    stddev_b: &mut f32,
    coeff: &mut f32,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        st_init(black_box(a), black_box(b));
        let start = black_box(DWT::cycle_count());
        st_main(
            black_box(a),
            black_box(b),
            black_box(sum_a),
            black_box(mean_a),
            black_box(var_a),
            black_box(stddev_a),
            black_box(sum_b),
            black_box(mean_b),
            black_box(var_b),
            black_box(stddev_b),
            black_box(coeff),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("st_libm - elapsed: {}", cycles);
    });

    let mut checksum = *mean_a + *mean_b + *stddev_a + *stddev_b + *coeff;
    /* allow rounding errors for the checksum */
    checksum -= 13_695.986;
    checksum < 0.000001 && checksum > -0.000001
}

#[cfg(any(unix, windows, not(feature = "log")))]
#[expect(clippy::too_many_arguments)]
fn st_std(
    a: &mut [f32],
    b: &mut [f32],
    sum_a: &mut f32,
    mean_a: &mut f32,
    var_a: &mut f32,
    stddev_a: &mut f32,
    sum_b: &mut f32,
    mean_b: &mut f32,
    var_b: &mut f32,
    stddev_b: &mut f32,
    coeff: &mut f32,
) -> bool {
    st_init(black_box(a), black_box(b));
    st_main(
        black_box(a),
        black_box(b),
        black_box(sum_a),
        black_box(mean_a),
        black_box(var_a),
        black_box(stddev_a),
        black_box(sum_b),
        black_box(mean_b),
        black_box(var_b),
        black_box(stddev_b),
        black_box(coeff),
    );

    let mut checksum = *mean_a + *mean_b + *stddev_a + *stddev_b + *coeff;
    /* allow rounding errors for the checksum */
    checksum -= 13_695.986;
    checksum < 0.000001 && checksum > -0.000001
}

fn st() -> bool {
    static mut A: [f32; 1000] = [0.; 1000];
    static mut B: [f32; 1000] = [0.; 1000];
    static mut SUM_A: f32 = 0.0;
    static mut MEAN_A: f32 = 0.0;
    static mut VAR_A: f32 = 0.0;
    static mut STDDEV_A: f32 = 0.0;
    static mut SUM_B: f32 = 0.0;
    static mut MEAN_B: f32 = 0.0;
    static mut VAR_B: f32 = 0.0;
    static mut STDDEV_B: f32 = 0.0;
    static mut COEFF: f32 = 0.0;

    // SAFETY: This is the only place we reference these and this function is only ever called once.
    let a = unsafe { &mut *{ &raw mut A } };
    let b = unsafe { &mut *{ &raw mut B } };

    let sum_a = unsafe { &mut *{ &raw mut SUM_A } };
    let mean_a = unsafe { &mut *{ &raw mut MEAN_A } };
    let var_a = unsafe { &mut *{ &raw mut VAR_A } };
    let stddev_a = unsafe { &mut *{ &raw mut STDDEV_A } };
    let sum_b = unsafe { &mut *{ &raw mut SUM_B } };
    let mean_b = unsafe { &mut *{ &raw mut MEAN_B } };
    let var_b = unsafe { &mut *{ &raw mut VAR_B } };
    let stddev_b = unsafe { &mut *{ &raw mut STDDEV_B } };
    let coeff = unsafe { &mut *{ &raw mut COEFF } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(st_log(
            black_box(a),
            black_box(b),
            black_box(sum_a),
            black_box(mean_a),
            black_box(var_a),
            black_box(stddev_a),
            black_box(sum_b),
            black_box(mean_b),
            black_box(var_b),
            black_box(stddev_b),
            black_box(coeff),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(st_std(
            black_box(a),
            black_box(b),
            black_box(sum_a),
            black_box(mean_a),
            black_box(var_a),
            black_box(stddev_a),
            black_box(sum_b),
            black_box(mean_b),
            black_box(var_b),
            black_box(stddev_b),
            black_box(coeff),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(st());

    #[cfg(feature = "log")]
    {
        rprintln!("st_libm: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(st());

    #[cfg(feature = "log")]
    std::println!("st_libm: {}", success);

    std::process::exit(!success as i32);
}
