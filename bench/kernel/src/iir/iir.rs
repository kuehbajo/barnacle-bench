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

  Name: iir

  Author: Juan Martinez Velarde

  Function:
    The equations of each biquad section filter are:
      w(n) =    x(n) - ai1*w(n-1) - ai2*w(n-2)
      y(n) = b0*w(n) + bi1*w(n-1) + bi2*w(n-2)

    Biquads are sequentally positioned. Input sample for biquad i is
    xi-1(n). Output sample for biquad i is xi(n).
    System input sample is x0(n). System output sample is xN(n) = y(n)
    for N biquads.

    Each section performs following filtering (biquad i) :

                              wi(n)
      xi-1(n) ---(-)---------->-|->---bi0---(+)-------> xi(n)
                  A             |            A
                  |           |1/z|          |
                  |             | wi(n-1)    |
                  |             v            |
                  |-<--ai1----<-|->---bi1-->-|
                  |             |            |
                  |           |1/z|          |
                  |             | wi(n-2)    |
                  |             v            |
                  |-<--ai2----<--->---bi2-->-|

    The values wi(n-1) and wi(n-2) are stored in wi1 and wi2

  Source: DSPstone
          http://www.ice.rwth-aachen.de/research/tools-projects/entry/detail/dspstone

  Original name: iir_N_sections_float

  Changes:
           24-03-94 creation fixed-point (Martinez Velarde)
           16-03-95 adaption floating-point (Harald L. Schraut)

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

fn iir_init(x: &mut f32, coefficients: &mut [f32; 5 * 4], wi: &mut [f32; 2 * 4]) {
    coefficients.fill(7.);
    wi.fill(0.);
    *x = 1.;

    let bitmask = 0u8;

    let range = coefficients.as_mut_ptr_range();
    let (mut p, end) = (range.start.cast::<u8>(), range.end.cast::<u8>());
    // loopbound min 80 max 80
    while p != end {
        // SAFETY: This actually does nothing at all...
        unsafe {
            *p ^= black_box(bitmask);
            p = p.add(1);
        }
    }

    let range = wi.as_mut_ptr_range();
    let (mut p, end) = (range.start as *mut u8, range.end as *mut u8);
    // loopbound min 32 max 32
    while p != end {
        // SAFETY: This actually does nothing at all...
        unsafe {
            *p ^= black_box(bitmask);
            p = p.add(1);
        }
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn iir_main(x: f32, coefficients: &[f32; 5 * 4], wi: &mut [f32; 2 * 4]) {
    let coeff_chunks = coefficients.chunks_exact(5);
    let wi_chunks = wi.chunks_exact_mut(2);

    let mut w: f32;
    let mut y = x;
    // loopbound min 4 max 4
    for (coeffs, wi) in coeff_chunks.zip(wi_chunks) {
        w = y - coeffs[0] * wi[0];
        w -= coeffs[1] * wi[1];

        y = coeffs[2] * w;
        y += coeffs[3] * wi[0];
        y += coeffs[4] * wi[1];

        wi[1] = wi[0];
        wi[0] = w;
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn iir_log(coefficients: &mut [f32; 5 * 4], wi: &mut [f32; 2 * 4]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut x = 0.;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        iir_init(black_box(&mut x), black_box(coefficients), black_box(wi));
        let start = black_box(DWT::cycle_count());
        iir_main(black_box(x), black_box(coefficients), black_box(wi));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("iir - elapsed: {}", cycles);
    });

    let checksum: f32 = wi.iter().sum();
    checksum as i32 == 400
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn iir_std(coefficients: &mut [f32; 5 * 4], wi: &mut [f32; 2 * 4]) -> bool {
    let mut x = 0.;

    iir_init(black_box(&mut x), black_box(coefficients), black_box(wi));
    iir_main(black_box(x), black_box(coefficients), black_box(wi));

    let checksum: f32 = wi.iter().sum();
    checksum as i32 == 400
}

fn iir() -> bool {
    static mut COEFFICIENTS: [f32; 5 * 4] = [0.; 5 * 4];
    static mut WI: [f32; 2 * 4] = [0.; 2 * 4];

    // SAFETY: This is the only place we ever reference these and main is not reentrant.
    let coefficients = unsafe { &mut *{ &raw mut COEFFICIENTS } };
    let wi = unsafe { &mut *{ &raw mut WI } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(iir_log(black_box(coefficients), black_box(wi)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(iir_std(black_box(coefficients), black_box(wi)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(iir());

    #[cfg(feature = "log")]
    {
        rprintln!("iir: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(iir());

    #[cfg(feature = "log")]
    std::println!("iir: {}", success);

    std::process::exit(!success as i32);
}
