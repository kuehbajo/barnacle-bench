/*

  Rust Implementation
  derived from the RSA Data Security, Inc. MD5 Message-Digest Algorithm

  Author: David Heim
  License: The Rust implementation shares the same license as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  Version V 2.0

  Name: md5.c

  Author: unknown

  Function: MD5 cryptographic hash function (see also RFC 1321).

  Source: RSA Data Security, Inc., MD5 message-digest algorithm

  Original name: md5c.c

  Changes: no major functional changes, inlined include files

  License: see below

*/
////////////////////////// md5c.c /////////////////////
/* MD5C.C - RSA Data Security, Inc., MD5 message-digest algorithm
 */

/* Copyright (C) 1991-2, RSA Data Security, Inc. Created 1991. All
   rights reserved.

   License to copy and use this software is granted provided that it
   is identified as the "RSA Data Security, Inc. MD5 Message-Digest
   Algorithm" in all material mentioning or referencing this software
   or this function.

   License is also granted to make and use derivative works provided
   that such works are identified as "derived from the RSA Data
   Security, Inc. MD5 Message-Digest Algorithm" in all material
   mentioning or referencing the derived work.

   RSA Data Security, Inc. makes no representations concerning either
   the merchantability of this software or the suitability of this
   software for any particular purpose. It is provided "as is"
   without express or implied warranty of any kind.

   These notices must be retained in any copies of any part of this
   documentation and/or software.
*/

//////// global.h ///////////////
// #include "global.h"
/* GLOBAL.H - RSAREF types and constants */

/* Copyright (C) RSA Laboratories, a division of RSA Data Security,
     Inc., created 1991. All rights reserved.
*/

///////////// md5.h //////////////
// #include "md5.h"
/* MD5.H - header file for MD5C.C
 */

/* Copyright (C) 1991-2, RSA Data Security, Inc. Created 1991. All
   rights reserved.

   License to copy and use this software is granted provided that it
   is identified as the "RSA Data Security, Inc. MD5 Message-Digest
   Algorithm" in all material mentioning or referencing this software
   or this function.

   License is also granted to make and use derivative works provided
   that such works are identified as "derived from the RSA Data
   Security, Inc. MD5 Message-Digest Algorithm" in all material
   mentioning or referencing the derived work.

   RSA Data Security, Inc. makes no representations concerning either
   the merchantability of this software or the suitability of this
   software for any particular purpose. It is provided "as is"
   without express or implied warranty of any kind.

   These notices must be retained in any copies of any part of this
   documentation and/or software.
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

mod consts {
    #![allow(non_snake_case)]

    pub const RANDOM_BYTES_NEEDED: u32 = 256;
    pub const EXCHANGEKEYS: u32 = 10;

    pub const S11: u32 = 7;
    pub const S12: u32 = 12;
    pub const S13: u32 = 17;
    pub const S14: u32 = 22;
    pub const S21: u32 = 5;
    pub const S22: u32 = 9;
    pub const S23: u32 = 14;
    pub const S24: u32 = 20;
    pub const S31: u32 = 4;
    pub const S32: u32 = 11;
    pub const S33: u32 = 16;
    pub const S34: u32 = 23;
    pub const S41: u32 = 6;
    pub const S42: u32 = 10;
    pub const S43: u32 = 15;
    pub const S44: u32 = 21;

    pub const PADDING: [u8; 64] = [
        0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ];

    const fn F(x: u32, y: u32, z: u32) -> u32 {
        (x & y) | (!x & z)
    }
    const fn G(x: u32, y: u32, z: u32) -> u32 {
        (x & z) | (y & !z)
    }
    const fn H(x: u32, y: u32, z: u32) -> u32 {
        x ^ y ^ z
    }
    const fn I(x: u32, y: u32, z: u32) -> u32 {
        y ^ (x | !z)
    }

    pub const fn FF(mut a: u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) -> u32 {
        a = a.wrapping_add(F(b, c, d)).wrapping_add(x).wrapping_add(ac);
        a = a.rotate_left(s);
        a = a.wrapping_add(b);
        a
    }
    pub const fn GG(mut a: u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) -> u32 {
        a = a.wrapping_add(G(b, c, d)).wrapping_add(x).wrapping_add(ac);
        a = a.rotate_left(s);
        a = a.wrapping_add(b);
        a
    }
    pub const fn HH(mut a: u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) -> u32 {
        a = a.wrapping_add(H(b, c, d)).wrapping_add(x).wrapping_add(ac);
        a = a.rotate_left(s);
        a = a.wrapping_add(b);
        a
    }
    pub const fn II(mut a: u32, b: u32, c: u32, d: u32, x: u32, s: u32, ac: u32) -> u32 {
        a = a.wrapping_add(I(b, c, d)).wrapping_add(x).wrapping_add(ac);
        a = a.rotate_left(s);
        a = a.wrapping_add(b);
        a
    }
}
use consts::*;

#[derive(Debug)]
struct Context {
    state: [u32; 16],
    count: [u32; 2],
    buffer: [u8; 64],
}

impl Context {
    fn new() -> Self {
        Self {
            count: [0, 0],
            state: [
                0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            buffer: [0; 64],
        }
    }

    fn zero(&mut self) {
        self.state = [0; 16];
        self.count = [0; 2];
        self.buffer = [0; 64];
    }
}

#[derive(Debug, Default)]
struct Random {
    bytes_needed: u32,
    state: [u8; 16],
    output_available: u32,
    _output: [u8; 16],
}

impl Random {
    fn init(&mut self) {
        let seed_byte: u8 = 0;
        self.bytes_needed = RANDOM_BYTES_NEEDED;
        self.state.fill(0);
        self.output_available = 0;

        // loopbound min 256 max 256
        loop {
            if self.bytes_needed == 0 {
                break;
            }

            self.update(&[seed_byte]);
        }
    }

    fn update(&mut self, block: &[u8]) {
        let mut ctx = Context::new();
        let mut digest = [0; 64];

        md5_update(&mut ctx, block);
        md5_final(&mut digest, &mut ctx);

        /* add digest to state */
        let mut x: u32 = 0;
        // loopbound min 16 max 16
        for (r, &d) in self.state.iter_mut().zip(digest.iter().take(16)).rev() {
            x += (*r as u32) + (d as u32);
            *r = x as u8;
            x >>= 8;
        }

        if self.bytes_needed < block.len() as u32 {
            self.bytes_needed = 0;
        } else {
            self.bytes_needed -= block.len() as u32;
        }

        // Zeroize sensitive information.
        digest.fill(0);
    }
}

fn md5_encode(output: &mut [u8], input: &[u32]) {
    // loopbound min 2 max 16
    for (o, i) in output.chunks_exact_mut(4).zip(input) {
        o[0] = (i & 0xff) as u8;
        o[1] = ((i >> 8) & 0xff) as u8;
        o[2] = ((i >> 16) & 0xff) as u8;
        o[3] = ((i >> 24) & 0xff) as u8;
    }
}

fn md5_decode(output: &mut [u32], input: &[u8]) {
    // loopbound min 16 max 16
    for (o, i) in output.iter_mut().zip(input.chunks_exact(4)) {
        *o = (i[0] as u32) | ((i[1] as u32) << 8) | ((i[2] as u32) << 16) | ((i[3] as u32) << 24);
    }
}

fn md5_transform(state: &mut [u32; 4], block: &[u8; 64]) {
    let &mut [a, b, c, d] = state;
    let mut x = [0; 16];

    md5_decode(&mut x, block);

    /* Round 1 */
    let a = FF(a, b, c, d, x[0], S11, 0xd76aa478); /* 1 */
    let d = FF(d, a, b, c, x[1], S12, 0xe8c7b756); /* 2 */
    let c = FF(c, d, a, b, x[2], S13, 0x242070db); /* 3 */
    let b = FF(b, c, d, a, x[3], S14, 0xc1bdceee); /* 4 */
    let a = FF(a, b, c, d, x[4], S11, 0xf57c0faf); /* 5 */
    let d = FF(d, a, b, c, x[5], S12, 0x4787c62a); /* 6 */
    let c = FF(c, d, a, b, x[6], S13, 0xa8304613); /* 7 */
    let b = FF(b, c, d, a, x[7], S14, 0xfd469501); /* 8 */
    let a = FF(a, b, c, d, x[8], S11, 0x698098d8); /* 9 */
    let d = FF(d, a, b, c, x[9], S12, 0x8b44f7af); /* 10 */
    let c = FF(c, d, a, b, x[10], S13, 0xffff5bb1); /* 11 */
    let b = FF(b, c, d, a, x[11], S14, 0x895cd7be); /* 12 */
    let a = FF(a, b, c, d, x[12], S11, 0x6b901122); /* 13 */
    let d = FF(d, a, b, c, x[13], S12, 0xfd987193); /* 14 */
    let c = FF(c, d, a, b, x[14], S13, 0xa679438e); /* 15 */
    let b = FF(b, c, d, a, x[15], S14, 0x49b40821); /* 16 */

    /* Round 2 */
    let a = GG(a, b, c, d, x[1], S21, 0xf61e2562); /* 17 */
    let d = GG(d, a, b, c, x[6], S22, 0xc040b340); /* 18 */
    let c = GG(c, d, a, b, x[11], S23, 0x265e5a51); /* 19 */
    let b = GG(b, c, d, a, x[0], S24, 0xe9b6c7aa); /* 20 */
    let a = GG(a, b, c, d, x[5], S21, 0xd62f105d); /* 21 */
    let d = GG(d, a, b, c, x[10], S22, 0x2441453); /* 22 */
    let c = GG(c, d, a, b, x[15], S23, 0xd8a1e681); /* 23 */
    let b = GG(b, c, d, a, x[4], S24, 0xe7d3fbc8); /* 24 */
    let a = GG(a, b, c, d, x[9], S21, 0x21e1cde6); /* 25 */
    let d = GG(d, a, b, c, x[14], S22, 0xc33707d6); /* 26 */
    let c = GG(c, d, a, b, x[3], S23, 0xf4d50d87); /* 27 */
    let b = GG(b, c, d, a, x[8], S24, 0x455a14ed); /* 28 */
    let a = GG(a, b, c, d, x[13], S21, 0xa9e3e905); /* 29 */
    let d = GG(d, a, b, c, x[2], S22, 0xfcefa3f8); /* 30 */
    let c = GG(c, d, a, b, x[7], S23, 0x676f02d9); /* 31 */
    let b = GG(b, c, d, a, x[12], S24, 0x8d2a4c8a); /* 32 */

    /* Round 3 */
    let a = HH(a, b, c, d, x[5], S31, 0xfffa3942); /* 33 */
    let d = HH(d, a, b, c, x[8], S32, 0x8771f681); /* 34 */
    let c = HH(c, d, a, b, x[11], S33, 0x6d9d6122); /* 35 */
    let b = HH(b, c, d, a, x[14], S34, 0xfde5380c); /* 36 */
    let a = HH(a, b, c, d, x[1], S31, 0xa4beea44); /* 37 */
    let d = HH(d, a, b, c, x[4], S32, 0x4bdecfa9); /* 38 */
    let c = HH(c, d, a, b, x[7], S33, 0xf6bb4b60); /* 39 */
    let b = HH(b, c, d, a, x[10], S34, 0xbebfbc70); /* 40 */
    let a = HH(a, b, c, d, x[13], S31, 0x289b7ec6); /* 41 */
    let d = HH(d, a, b, c, x[0], S32, 0xeaa127fa); /* 42 */
    let c = HH(c, d, a, b, x[3], S33, 0xd4ef3085); /* 43 */
    let b = HH(b, c, d, a, x[6], S34, 0x4881d05); /* 44 */
    let a = HH(a, b, c, d, x[9], S31, 0xd9d4d039); /* 45 */
    let d = HH(d, a, b, c, x[12], S32, 0xe6db99e5); /* 46 */
    let c = HH(c, d, a, b, x[15], S33, 0x1fa27cf8); /* 47 */
    let b = HH(b, c, d, a, x[2], S34, 0xc4ac5665); /* 48 */

    /* Round 4 */
    let a = II(a, b, c, d, x[0], S41, 0xf4292244); /* 49 */
    let d = II(d, a, b, c, x[7], S42, 0x432aff97); /* 50 */
    let c = II(c, d, a, b, x[14], S43, 0xab9423a7); /* 51 */
    let b = II(b, c, d, a, x[5], S44, 0xfc93a039); /* 52 */
    let a = II(a, b, c, d, x[12], S41, 0x655b59c3); /* 53 */
    let d = II(d, a, b, c, x[3], S42, 0x8f0ccc92); /* 54 */
    let c = II(c, d, a, b, x[10], S43, 0xffeff47d); /* 55 */
    let b = II(b, c, d, a, x[1], S44, 0x85845dd1); /* 56 */
    let a = II(a, b, c, d, x[8], S41, 0x6fa87e4f); /* 57 */
    let d = II(d, a, b, c, x[15], S42, 0xfe2ce6e0); /* 58 */
    let c = II(c, d, a, b, x[6], S43, 0xa3014314); /* 59 */
    let b = II(b, c, d, a, x[13], S44, 0x4e0811a1); /* 60 */
    let a = II(a, b, c, d, x[4], S41, 0xf7537e82); /* 61 */
    let d = II(d, a, b, c, x[11], S42, 0xbd3af235); /* 62 */
    let c = II(c, d, a, b, x[2], S43, 0x2ad7d2bb); /* 63 */
    let b = II(b, c, d, a, x[9], S44, 0xeb86d391); /* 64 */

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);

    // Zeroize sensitive information
    x.fill(0);
}

fn md5_update(ctx: &mut Context, input: &[u8]) {
    /* Compute number of bytes mod 64 */
    let mut index: usize = (ctx.count[0] as usize >> 3) & 0x3F;

    /* Update number of bits */
    ctx.count[0] += (input.len() as u32) << 3;
    if ctx.count[0] < (input.len() as u32) << 3 {
        ctx.count[1] += 1;
    }
    ctx.count[1] += (input.len() as u32) >> 29;

    let mut i = 0;
    let part_len = 64 - index;

    // Transform as many times as possible.
    if input.len() >= part_len {
        ctx.buffer[index..index + part_len].copy_from_slice(&input[..part_len]);
        md5_transform((&mut ctx.state[..4]).try_into().unwrap(), &ctx.buffer);

        i = part_len;
        // loopbound min 0 max 0
        while i + 63 < input.len() {
            md5_transform(
                (&mut ctx.state[..4]).try_into().unwrap(),
                (&input[i..]).try_into().unwrap(),
            );
            i += 64;
        }

        index = 0;
    }

    /* Buffer remaining input */
    ctx.buffer[index..index + input.len() - i].copy_from_slice(&input[i..]);
}

fn md5_final(digest: &mut [u8; 64], ctx: &mut Context) {
    let mut bits = [0; 8];

    md5_encode(&mut bits, &ctx.count);

    let index = ((ctx.count[0] >> 3) & 0x3f) as usize;
    let pad_len = if index < 56 { 56 - index } else { 120 - index };
    md5_update(ctx, &PADDING[..pad_len]);

    /* Append length (before padding) */
    md5_update(ctx, &bits);

    /* Store state in digest */
    md5_encode(digest, &ctx.state);

    // Zeroize sensitive information.
    ctx.zero();
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn md5_main() -> u32 {
    let mut random = Random::default();
    let mut random2 = Random::default();

    random.init();

    let mut keys_exchanged = 0;
    // loopbound min 10 max 10
    while keys_exchanged != EXCHANGEKEYS {
        keys_exchanged += 1;
        random2.init();
    }

    random.bytes_needed + random2.bytes_needed
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn md5_log() -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: u32 = 0;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        let start = black_box(DWT::cycle_count());
        result = black_box(md5_main());
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("md5 - elapsed: {}", cycles);
    });

    result == 0
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn md5_std() -> bool {
    let result = black_box(md5_main());

    result == 0
}

fn md5() -> bool {
    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(md5_log())
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(md5_std())
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(md5());

    #[cfg(feature = "log")]
    {
        rprintln!("md5: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(md5());

    #[cfg(feature = "log")]
    std::println!("md5: {}", success);

    std::process::exit(!success as i32);
}
