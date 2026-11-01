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

  Name: rijndael_enc

  Author: Dr Brian Gladman

  Function: rijndael_enc is an implementation of the AES encryption
            algorithm (Rijndael).

  Source: security section of MiBench

  Changes: Add computation of a checksum, refactoring

  License: see below

*/

/*
  -----------------------------------------------------------------------
  Copyright (c) 2001 Dr Brian Gladman <brg@gladman.uk.net>, Worcester, UK

  TERMS

  Redistribution and use in source and binary forms, with or without
  modification, are permitted provided that the following conditions
  are met:
  1. Redistributions of source code must retain the above copyright
     notice, this list of conditions and the following disclaimer.
  2. Redistributions in binary form must reproduce the above copyright
     notice, this list of conditions and the following disclaimer in the
     documentation and/or other materials provided with the distribution.

  This software is provided 'as is' with no guarantees of correctness or
  fitness for purpose.
  -----------------------------------------------------------------------
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#![allow(non_snake_case, non_upper_case_globals)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

mod aes;
use aes::{ AES, rijndael_enc_encrypt, rijndael_enc_set_key, AESKey };

mod aestab;

mod file;
use file::File;

mod input;
use input::rijndael_enc_data;

fn rijndael_enc_init(
    key: &mut [u8; 32],
    key_len: &mut usize,
    fin: &mut File,
    a: &mut [u32; 2],
    mt: &mut bool,
    count: &mut usize,
    r: &mut [u8; 4],
    checksum: &mut i32,
) {
    // additional resets
    *mt = true;
    *fin = File::new(&rijndael_enc_data);
    *a = [0; 2];
    *count = 4;
    *r = [0; 4];
    *checksum = 0;

    /* this is a pointer to the hexadecimal key digits  */
    let mut buf = *b"1234567890abcdeffedcba09876543211234567890abcdeffedcba0987654321";
    let mut cp: &mut str = str::from_utf8_mut(&mut buf).unwrap();
    let mut ch: u8;
    let mut by = 0;

    let mut i = 0;  /* this is a count for the input digits processed */
    // loopbound min 64 max 64
    while i < 64 && !cp.is_empty() {   /* the maximum key length is 32 bytes and       */
        /* hence at most 64 hexadecimal digits            */
        ch = cp.bytes().next().unwrap().to_ascii_uppercase();  /* process a hexadecimal digit  */
        cp = &mut cp[1..];

        by = match ch {
            b'0'..=b'9' => (by << 4) + (ch - b'0'),
            b'A'..=b'F' => (by << 4) + (ch - b'A' + 10),
            _ => {  /* error if not hexadecimal     */
                *checksum = -2;
                return;
            }
        };
        /* store a key byte for each pair of hexadecimal digits         */
        if i & 1 != 0 {
            key[i / 2] = by & 0xff;
        }
        i += 1;
    }

    if !cp.is_empty() {
        *checksum = -3;
        return;
    } else if i < 32 || (i & 15) != 0 {
        *checksum = -4;
        return;
    }

    *key_len = i / 2;
}

fn rijndael_enc_return(checksum: i32) -> bool {
    checksum == 249509
}

/* A Pseudo Random Number Generator (PRNG) used for the     */
/* Initialisation Vector. The PRNG is George Marsaglia's    */
/* Multiply-With-Carry (MWC) PRNG that concatenates two     */
/* 16-bit MWC generators:                                   */
/*     x(n)=36969 * x(n-1) + carry mod 2^16                 */
/*     y(n)=18000 * y(n-1) + carry mod 2^16                 */
/* to produce a combined PRNG with a period of about 2^60.  */

#[inline]
fn RAND(a: &mut [u32; 2]) -> u32 {
    a[0] = 36969u32.wrapping_mul(a[0] & 65535).wrapping_add(a[0] >> 16);
    a[1] = 18000u32.wrapping_mul(a[1] & 65535).wrapping_add(a[1] >> 16);
    a[0].wrapping_shl(16) + a[1]
}

fn rijndael_enc_fillrand(
    buf: &mut [u8],
    amount: usize,
    a: &mut [u32; 2],
    mt: &mut bool,
    count: &mut usize,
    r: &mut [u8; 4],
) {
    if *mt {
        *mt = false;
        a[0] = 0xeaf3;
        a[1] = 0x35fe;
    }

    //loopbound min 1 max 16
    for i in 0..amount {
        if *count == 4 {
            let val = RAND(a);
            let bytes = val.to_ne_bytes();
            r.copy_from_slice(&bytes);
            *count = 0;
        }

        buf[i] = r[*count];
        *count += 1;
    }
}

// in the original, inbuf is not properly initialized
fn rijndael_enc_encfile(
    ctx: &mut AES,
    fin: &mut File,
    a: &mut [u32; 2],
    mt: &mut bool,
    count: &mut usize,
    r: &mut [u8; 4],
    checksum: &mut i32,
) {
    let mut inbuf: [u8; 16] = [0; 16];
    let mut outbuf: [u8; 16] = [0; 16];
    let mut i = 0;

    rijndael_enc_fillrand(&mut outbuf,  16, a, mt, count, r);    /* set an IV for CBC mode */
    let flen = fin.data.len();

    rijndael_enc_fillrand(&mut inbuf, 1, a, mt, count, r);        /* make top 4 bits of a byte random */

    let mut l = 15;                                 /* and store the length of the last */
    /* block in the lower 4 bits        */
    inbuf[0] = (flen as u8 & 15) | (inbuf[ 0 ] & !15);

    /* TODO: this is necessarily an input-dependent loop bound */
    // loopbound min 1960 max 1960
    while !fin.eof() {  /* loop to encrypt the input file   */
        /* input 1st 16 bytes to buf[ 1..16 ] */
        i = fin.read(&mut inbuf[(16 - l as usize)..], 1, l).unwrap(); /* on 1st round byte[ 0 ] */
        /* is the length code    */
        if i < l { break; }     /* if end of the input file reached */

        // loopbound min 16 max 16
        for (inb, outb) in inbuf.iter_mut().zip(outbuf.iter()) {    /* xor in previous cipher text      */
            *inb ^= *outb;
        }

        rijndael_enc_encrypt(&mut inbuf, &mut outbuf, ctx);  /* and do the encryption            */

        *checksum += outbuf[15] as i32;

        /* in all but first round read 16   */
        l = 16;                       /* bytes into the buffer            */
    }

    /* except for files of length less than two blocks we now have one  */
    /* byte from the previous block and 'i' bytes from the current one  */
    /* to encrypt and 15 - i empty buffer positions. For files of less  */
    /* than two blocks (0 or 1) we have i + 1 bytes and 14 - i empty    */
    /* buffer position to set to zero since the 'count' byte is extra   */

    if l == 15 {    /* adjust for extra byte in the */
        i += 1;                             /* in the first block           */
    }
    
    if i > 0 {                          /* if bytes remain to be output */
        // loopbound min 6 max 6
        inbuf[i..].fill(0);     /* clear empty buffer positions */

        // loopbound min 16 max 16
        for (inb, outb) in inbuf.iter_mut().zip(outbuf.iter()) {    /* xor in previous cipher text      */
            *inb ^= *outb;
        }

        rijndael_enc_encrypt(&mut  inbuf, &mut outbuf, ctx); /* encrypt and output it */

        *checksum += outbuf[15] as i32;
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn rijndael_enc_main(
    key: &mut [u8; 32],
    key_len: &mut usize,
    fin: &mut File,
    a: &mut [u32; 2],
    mt: &mut bool,
    count: &mut usize,
    r: &mut [u8; 4],
    checksum: &mut i32,
) {
    let mut ctx: AES = AES::default();

    /* encryption in Cipher Block Chaining mode */
    rijndael_enc_set_key(key, *key_len as u32, AESKey::Enc, &mut ctx);
    rijndael_enc_encfile(
        &mut ctx, fin, a, mt, count, r, checksum,
    );
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn rijndael_enc_log(
    key: &mut [u8; 32],
    key_len: &mut usize,
    fin: &mut File,
    a: &mut [u32; 2],
    mt: &mut bool,
    count: &mut usize,
    r: &mut [u8; 4],
    checksum: &mut i32,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        rijndael_enc_init(
            black_box(key), black_box(key_len),
            black_box(fin), black_box(a),
            black_box(mt), black_box(count),
            black_box(r), black_box(checksum),
        );
        let start = black_box(DWT::cycle_count());
        rijndael_enc_main(
            black_box(key), black_box(key_len),
            black_box(fin), black_box(a),
            black_box(mt), black_box(count),
            black_box(r), black_box(checksum),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("rijndael_enc - elapsed: {}", cycles);
    });

    black_box(rijndael_enc_return(black_box(*checksum)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn rijndael_enc_std(
    key: &mut [u8; 32],
    key_len: &mut usize,
    fin: &mut File,
    a: &mut [u32; 2],
    mt: &mut bool,
    count: &mut usize,
    r: &mut [u8; 4],
    checksum: &mut i32,
) -> bool {
    rijndael_enc_init(
        black_box(key), black_box(key_len),
        black_box(fin), black_box(a),
        black_box(mt), black_box(count),
        black_box(r), black_box(checksum),
    );
    rijndael_enc_main(
        black_box(key), black_box(key_len),
        black_box(fin), black_box(a),
        black_box(mt), black_box(count),
        black_box(r), black_box(checksum),
    );
    black_box(rijndael_enc_return(black_box(*checksum)))
}

fn rijndael_enc() -> bool {
    static mut KEY: [u8; 32] = [0; 32];
    static mut KEY_LEN: usize = 32;
    static mut FIN: File<'static> = File::new(&rijndael_enc_data);
    static mut A: [u32; 2] = [0; 2];
    static mut MT: bool = true;
    static mut COUNT: usize = 4;
    static mut R: [u8; 4] = [0; 4];
    static mut CHECKSUM: i32 = 0;
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let key = unsafe { &mut *{ &raw mut KEY } };
    let key_len = unsafe { &mut *{ &raw mut KEY_LEN } };
    let fin = unsafe { &mut *{ &raw mut FIN } };
    let a = unsafe { &mut *{ &raw mut A } };
    let mt = unsafe { &mut *{ &raw mut MT } };
    let count = unsafe { &mut *{ &raw mut COUNT } };
    let r = unsafe { &mut *{ &raw mut R } };
    let checksum = unsafe { &mut *{ &raw mut CHECKSUM } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(rijndael_enc_log(
            black_box(key), black_box(key_len),
            black_box(fin), black_box(a),
            black_box(mt), black_box(count),
            black_box(r), black_box(checksum),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(rijndael_enc_std(
            black_box(key), black_box(key_len),
            black_box(fin), black_box(a),
            black_box(mt), black_box(count),
            black_box(r), black_box(checksum),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(rijndael_enc());
    
    #[cfg(feature = "log")]
    {
        rprintln!("rijndael_enc: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(rijndael_enc());

    #[cfg(feature = "log")]
    std::println!("rijndael_enc: {}", success);

    std::process::exit(!success as i32);
}
