/*

  Rust Implementation

  Author: David Heim
  License: The Rust implementation shares the same license (GNU Lesser General Public License) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  Version V 1.x

  Name: sha.c

  Author: Peter C. Gutmann's (heavily modified by Uwe Hollerbach)

  NIST Secure Hash Algorithm

  Source: Peter C. Gutmann's implementation as found in Applied Cryptography by Bruce Schneier

  Changes: no major functional changes

  License: GNU Lesser General Public License

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

mod input;

use core::{hint::black_box, ptr};

use input::SHA_DATA;

const BLOCK_SIZE: usize = 8192;
const SHA_BLOCK_SIZE: usize = 64;

const CONST1: u32 = 0x5a827999;
const CONST2: u32 = 0x6ed9eba1;
const CONST3: u32 = 0x8f1bbcdc;
const CONST4: u32 = 0xca62c1d6;

const fn f1(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (!x & z)
}
const fn f2(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}
const fn f3(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | (x & z) | (y & z)
}
const fn f4(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}

struct ShaInfo {
    digest: [u32; 5],
    count_lo: u32,
    count_hi: u32,
    data: [u32; 16],
}

impl ShaInfo {
    fn new() -> Self {
        Self {
            digest: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0],
            count_lo: 0,
            count_hi: 0,
            data: [0; 16],
        }
    }

    #[cfg_attr(any(unix, windows, not(feature = "log")), expect(dead_code))]
    fn reset(&mut self) {
        self.digest = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0];
        self.count_lo = 0;
        self.count_hi = 0;
        self.data = [0; 16];
    }
}

struct ShaFile<'a> {
    data: &'a [u8],
    cur_pos: usize,
}

struct Eof(usize);

impl<'a> ShaFile<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, cur_pos: 0 }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Eof> {
        let mut read = 0;
        // loopbound min 0 max 8192
        for (&data, b) in self.data[self.cur_pos..].iter().zip(&mut *buf) {
            *b = data;
            read += 1;
        }

        self.cur_pos += read;

        if read < buf.len() {
            Err(Eof(read))
        } else {
            Ok(())
        }
    }
}

fn sha_byte_reverse(buf: &mut [u32]) {
    // loopbound min 16 max 16
    for b in buf {
        *b = b.swap_bytes();
    }
}

fn sha_transform(sha_info: &mut ShaInfo) {
    let mut w = [0; 80];
    // loopbound min 16 max 16
    w[..sha_info.data.len()].copy_from_slice(&sha_info.data);
    // loopbound min 64 max 64
    for i in 16..80 {
        w[i] = w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16];
    }

    let &[mut a, mut b, mut c, mut d, mut e] = &sha_info.digest;

    // loopbound min 20 max 20
    for &x in &w[..20] {
        let tmp = a
            .rotate_left(5)
            .wrapping_add(f1(b, c, d))
            .wrapping_add(e)
            .wrapping_add(x)
            .wrapping_add(CONST1);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = tmp;
    }
    // loopbound min 20 max 20
    for &x in &w[20..40] {
        let tmp = a
            .rotate_left(5)
            .wrapping_add(f2(b, c, d))
            .wrapping_add(e)
            .wrapping_add(x)
            .wrapping_add(CONST2);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = tmp;
    }
    // loopbound min 20 max 20
    for &x in &w[40..60] {
        let tmp = a
            .rotate_left(5)
            .wrapping_add(f3(b, c, d))
            .wrapping_add(e)
            .wrapping_add(x)
            .wrapping_add(CONST3);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = tmp;
    }
    // loopbound min 20 max 20
    for &x in &w[60..80] {
        let tmp = a
            .rotate_left(5)
            .wrapping_add(f4(b, c, d))
            .wrapping_add(e)
            .wrapping_add(x)
            .wrapping_add(CONST4);
        e = d;
        d = c;
        c = b.rotate_left(30);
        b = a;
        a = tmp;
    }

    sha_info.digest[0] = sha_info.digest[0].wrapping_add(a);
    sha_info.digest[1] = sha_info.digest[1].wrapping_add(b);
    sha_info.digest[2] = sha_info.digest[2].wrapping_add(c);
    sha_info.digest[3] = sha_info.digest[3].wrapping_add(d);
    sha_info.digest[4] = sha_info.digest[4].wrapping_add(e);
}

fn sha_update(sha_info: &mut ShaInfo, mut buf: &[u8]) {
    let mut count = buf.len();
    if sha_info.count_lo.wrapping_add((count as u32) << 3) < sha_info.count_lo {
        sha_info.count_hi += 1;
    }
    sha_info.count_lo = sha_info.count_lo.wrapping_add((count as u32) << 3);
    sha_info.count_hi = sha_info.count_hi.wrapping_add((count as u32) >> 29);

    // loopbound min 8 max 128
    while count >= SHA_BLOCK_SIZE {
        // SAFETY: SHA_BLOCK_SIZE / size_of::<u8>() == sha_info.data.len()
        unsafe {
            ptr::copy_nonoverlapping(
                buf.as_ptr(),
                sha_info.data.as_mut_ptr().cast(),
                SHA_BLOCK_SIZE,
            )
        };
        sha_byte_reverse(&mut sha_info.data);
        sha_transform(sha_info);
        buf = &buf[SHA_BLOCK_SIZE..];
        count -= SHA_BLOCK_SIZE;
    }
    // SAFETY: Same as above + count < SHA_BLOCK_SIZE
    unsafe { ptr::copy_nonoverlapping(buf.as_ptr(), sha_info.data.as_mut_ptr().cast(), count) };
}

fn sha_final(sha_info: &mut ShaInfo) {
    let mut count = (sha_info.count_lo >> 3) & 0x3f;
    unsafe { *(sha_info.data.as_mut_ptr() as *mut u8).add(count as usize) = 0x80 };
    count += 1;
    if count > 56 {
        unsafe {
            ptr::write_bytes(
                (sha_info.data.as_mut_ptr() as *mut u8).add(count as usize),
                0,
                64 - count as usize,
            )
        };
        sha_byte_reverse(&mut sha_info.data);
        sha_transform(sha_info);
        unsafe { ptr::write_bytes(sha_info.data.as_mut_ptr() as *mut u8, 0, 56) };
    } else {
        unsafe {
            ptr::write_bytes(
                (sha_info.data.as_mut_ptr() as *mut u8).add(count as usize),
                0,
                56 - count as usize,
            )
        };
    }

    sha_byte_reverse(&mut sha_info.data);
    sha_info.data[14] = sha_info.count_hi;
    sha_info.data[15] = sha_info.count_lo;
    sha_transform(sha_info);
}

fn sha_stream(sha_info: &mut ShaInfo, mut stream: ShaFile) {
    let mut data = [0; BLOCK_SIZE];

    // loopbound min 5 max 5
    loop {
        match stream.read_exact(&mut data) {
            Ok(_) => sha_update(sha_info, &data),
            Err(Eof(read)) => {
                sha_update(sha_info, &data[..read]);
                break;
            }
        }
    }

    sha_final(sha_info);
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn sha_main(sha_info: &mut ShaInfo) {
    let file = ShaFile::new(&SHA_DATA);
    sha_stream(sha_info, file);
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn sha_log() -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut sha_info = ShaInfo::new();

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        sha_info.reset();
        let start = black_box(DWT::cycle_count());
        sha_main(black_box(&mut sha_info));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("sha - elapsed: {}", cycles);
    });

    let sum = sha_info.data[14] + sha_info.data[15];
    sum == 261944
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn sha_std() -> bool {
    let mut sha_info = ShaInfo::new();

    sha_main(black_box(&mut sha_info));

    let sum = sha_info.data[14] + sha_info.data[15];
    sum == 261944
}

fn sha() -> bool {
    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(sha_log())
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(sha_std())
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(sha());

    #[cfg(feature = "log")]
    {
        rprintln!("sha: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(sha());

    #[cfg(feature = "log")]
    std::println!("sha: {}", success);

    std::process::exit(!success as i32);
}
