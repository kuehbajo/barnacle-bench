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

  Name: complex_updates

  Author: Juan Martinez Velarde

  Function: complex_updates is a program for filter benchmarking.
    This program performs n complex updates of the form
              D(i) = C(i) + A(i)*B(i),
    where A(i), B(i), C(i) and D(i) are complex numbers,
    and i = 1,...,N
             A(i) = Ar(i) + j Ai(i)
             B(i) = Br(i) + j Bi(i)
             C(i) = Cr(i) + j Ci(i)
             D(i) = C(i) + A(i)*B(i) =   Dr(i)  +  j Di(i)
                         =>  Dr(i) = Cr(i) + Ar(i)*Br(i) - Ai(i)*Bi(i)
                         =>  Di(i) = Ci(i) + Ar(i)*Bi(i) + Ai(i)*Br(i)

  Source: DSP-Stone
    http://www.ice.rwth-aachen.de/research/tools-projects/entry/detail/dspstone/

  Original name: n_complex_updates_float

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

const N: usize = 16;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
struct Complex {
    real: f32,
    imag: f32,
}

fn complex_updates_init(
    a: &mut [Complex; N],
    b: &mut [Complex; N],
    c: &mut [Complex; N],
    d: &mut [Complex; N],
) {
    // loopbound min 16 max 16
    for i in 0..N {
        a[i].real = 2.;
        a[i].imag = 1.;
        b[i].real = 2.;
        b[i].imag = 5.;
        c[i].real = 3.;
        c[i].imag = 4.;
        d[i].real = 0.;
        d[i].imag = 0.;
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn complex_updates_main(
    a: &[Complex; N],
    b: &[Complex; N],
    c: &[Complex; N],
    d: &mut [Complex; N],
) {
    // loopbound min 16 max 16
    for i in 0..N {
        d[i].real = c[i].real + a[i].real * b[i].real;
        d[i].real -= a[i].imag * b[i].imag;

        d[i].imag = c[i].imag + a[i].imag * b[i].real;
        d[i].imag += a[i].real * b[i].imag;
    }
}

fn complex_updates_return(d: &[Complex; N]) -> bool {
    let checksum: f32 = d.iter().take(N / 2).map(|x| x.real + x.imag).sum();
    checksum == 144.
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn complex_updates_log(
    a: &mut [Complex; N],
    b: &mut [Complex; N],
    c: &mut [Complex; N],
    d: &mut [Complex; N],
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        complex_updates_init(black_box(a), black_box(b), black_box(c), black_box(d));
        let start = black_box(DWT::cycle_count());
        complex_updates_main(black_box(a), black_box(b), black_box(c), black_box(d));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("complex_updates - elapsed: {}", cycles);
    });

    black_box(complex_updates_return(black_box(d)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn complex_updates_std(
    a: &mut [Complex; N],
    b: &mut [Complex; N],
    c: &mut [Complex; N],
    d: &mut [Complex; N],
) -> bool {
    complex_updates_init(black_box(a), black_box(b), black_box(c), black_box(d));
    complex_updates_main(black_box(a), black_box(b), black_box(c), black_box(d));

    black_box(complex_updates_return(black_box(d)))
}

fn complex_updates() -> bool {
    static mut A: [Complex; N] = [Complex { real: 0., imag: 0. }; N];
    static mut B: [Complex; N] = [Complex { real: 0., imag: 0. }; N];
    static mut C: [Complex; N] = [Complex { real: 0., imag: 0. }; N];
    static mut D: [Complex; N] = [Complex { real: 0., imag: 0. }; N];

    // SAFETY: This is the only place we ever reference A, B, C, D and this function is not reentrant
    let a = unsafe { &mut *{ &raw mut A } };
    let b = unsafe { &mut *{ &raw mut B } };
    let c = unsafe { &mut *{ &raw mut C } };
    let d = unsafe { &mut *{ &raw mut D } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(complex_updates_log(black_box(a), black_box(b), black_box(c), black_box(d)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(complex_updates_std(black_box(a), black_box(b), black_box(c), black_box(d)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(complex_updates());

    #[cfg(feature = "log")]
    {
        rprintln!("complex_updates: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(complex_updates());

    #[cfg(feature = "log")]
    std::println!("complex_updates: {}", success);

    std::process::exit(!success as i32);
}
