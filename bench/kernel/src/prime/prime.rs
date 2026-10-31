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

  Name: prime

  Author: unknown

  Function: prime calculates whether numbers are prime.

  Source: MRTC
          http://www.mrtc.mdh.se/projects/wcet/wcet_bench/prime/prime.c

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

use core::{hint::black_box, mem};

struct RandomInt(u32);

impl RandomInt {
    fn init() -> Self {
        RandomInt(0)
    }

    fn next(&mut self) -> u32 {
        self.0 = ((self.0 * 133) + 81) % 8095;
        self.0
    }
}

fn prime_init(x: &mut u32, y: &mut u32) {
    let mut rand = RandomInt::init();

    *x = rand.next();
    *y = rand.next();
}

fn prime_divides(n: u32, m: u32) -> bool {
    m % n == 0
}

fn prime_even(n: u32) -> bool {
    prime_divides(2, n)
}

fn prime_prime(n: u32) -> bool {
    if prime_even(n) {
        return n == 2;
    }
    // loopbound min 0 max 16
    for i in (3..).step_by(2).take_while(|i| i * i <= n) {
        if prime_divides(i, n) {
            return false;
        }
    }
    n > 1
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn prime_main(x: &mut u32, y: &mut u32) -> bool {
    mem::swap(x, y);

    !(!prime_prime(*x) && !prime_prime(*y))
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn prime_log(prime_x: &mut u32, prime_y: &mut u32) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: bool = false;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        prime_init(black_box(prime_x), black_box(prime_y));
        let start = black_box(DWT::cycle_count());
        result = black_box(prime_main(black_box(prime_x), black_box(prime_y)));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("prime - elapsed: {}", cycles);
    });

    !result
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn prime_std(prime_x: &mut u32, prime_y: &mut u32) -> bool {
    prime_init(black_box(prime_x), black_box(prime_y));
    let result = black_box(prime_main(black_box(prime_x), black_box(prime_y)));

    !result
}

fn prime() -> bool {
    static mut PRIME_X: u32 = 0;
    static mut PRIME_Y: u32 = 0;
    // SAFETY: This is the only place we reference these and this function is only ever called once.
    let prime_x = unsafe { &mut *{ &raw mut PRIME_X } };
    let prime_y = unsafe { &mut *{ &raw mut PRIME_Y } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(prime_log(black_box(prime_x), black_box(prime_y)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(prime_std(black_box(prime_x), black_box(prime_y)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(prime());

    #[cfg(feature = "log")]
    {
        rprintln!("prime: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(prime());

    #[cfg(feature = "log")]
    std::println!("prime: {}", success);

    std::process::exit(!success as i32);
}
