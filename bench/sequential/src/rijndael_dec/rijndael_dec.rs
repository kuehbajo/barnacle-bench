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

  Function: rijndael_dec is an implementation of the AES decryption
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

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

mod aes;
use aes::{ AES, rijndael_dec_decrypt, rijndael_dec_set_key, AESKey };

mod aestab;

mod file;
use file::File;

mod input;
use input::RIJNDAEL_DEC_DATA;

fn rijndael_dec_init(
    key: &mut [u8; 32],
    key_len: &mut usize,
    fin: &mut File,
    checksum: &mut i32,
) {
    fin.cur_pos = 0;
    *checksum = 0;
    /* this is a pointer to the hexadecimal key digits  */
    let mut buf = *b"1234567890abcdeffedcba09876543211234567890abcdeffedcba0987654321";
    let mut cp = str::from_utf8_mut(&mut buf).unwrap();
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

fn rijndael_dec_return(checksum: i32) -> bool {
    checksum == 262180
}

fn rijndael_dec_decfile(
    ctx: &mut AES,
    fin: &mut File,
    checksum: &mut i32,
) {
    let mut inbuf1: [u8; 16] = [0; 16];
    let mut inbuf2: [u8; 16] = [0; 16];
    let mut outbuf: [u8; 16] = [0; 16];

    let _ = fin.read(&mut inbuf1, 1, 16);

    let i = fin.read(&mut inbuf2, 1, 16); /* read 1st encrypted file block    */

    if i.is_ok() && i.unwrap() != 16 {
        *checksum = -10;
        return;
    }

    rijndael_dec_decrypt(&inbuf2, &mut outbuf, &ctx); /* decrypt it                       */

    *checksum += outbuf[15] as i32;

    //loopbound min 16 max 16
    for (outb, inb) in outbuf.iter_mut().zip(inbuf1.iter()) {   /* xor with previous input          */
        *outb ^= *inb;
    }

    let mut switch = false;

    /* TODO: this is necessarily an input-dependent loop bound */
    //loopbound min 2046 max 2046
    loop {
        let i = if !switch {    /* read next encrypted block    */
            fin.read(&mut inbuf1, 1, 16)
        } else {
            fin.read(&mut inbuf2, 1, 16)
        };
        /* to first input buffer        */
        if !i.is_ok() || i.unwrap() != 16 {    /* no more bytes in input - the decrypted   */
            break;          /* partial final buffer needs to be output  */
        }

        /* if a block has been read the previous block must have been   */
        /* full lnegth so we can now write it out                       */

        if !switch {
            rijndael_dec_decrypt(&inbuf1, &mut outbuf, ctx); /* decrypt the new input block and  */
        } else {
            rijndael_dec_decrypt(&inbuf2, &mut outbuf, ctx); /* decrypt the new input block and  */
        }

        *checksum += outbuf[15] as i32;

        //loopbound min 16 max 16
        for (i, outb) in outbuf.iter_mut().enumerate() {    /* xor it with previous input block */
            if !switch {
                *outb ^= inbuf2[i];
            } else {
                *outb ^= inbuf1[i];
            }
        }
        switch = !switch;
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn rijndael_dec_main(
    key: &mut [u8; 32],
    key_len: &mut usize,
    fin: &mut File,
    checksum: &mut i32,
) {
    let mut ctx: AES = AES::default();

    /* decryption in Cipher Block Chaining mode */
    rijndael_dec_set_key(key, *key_len as u32, AESKey::Dec, &mut ctx);
    rijndael_dec_decfile(&mut ctx, fin, checksum);
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn rijndael_dec_log(
    key: &mut [u8; 32],
    key_len: &mut usize,
    fin: &mut File,
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
        rijndael_dec_init(
            black_box(key), black_box(key_len),
            black_box(fin), black_box(checksum)
        );
        let start = black_box(DWT::cycle_count());
        rijndael_dec_main(
            black_box(key), black_box(key_len),
            black_box(fin), black_box(checksum),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("rijndael_dec - elapsed: {}", cycles);
    });

    black_box(rijndael_dec_return(black_box(*checksum)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn rijndael_dec_std(
    key: &mut [u8; 32],
    key_len: &mut usize,
    fin: &mut File,
    checksum: &mut i32,
) -> bool {
    rijndael_dec_init(
        black_box(key), black_box(key_len), 
        black_box(fin), black_box(checksum)
    );
    rijndael_dec_main(
        black_box(key), black_box(key_len),
        black_box(fin), black_box(checksum),
    );
    black_box(rijndael_dec_return(black_box(*checksum)))
}

fn rijndael_dec() -> bool {
    static mut KEY: [u8; 32] = [0; 32];
    static mut KEY_LEN: usize = 32;
    static mut FIN: File<'static> = File::new(&RIJNDAEL_DEC_DATA);
    static mut CHECKSUM: i32 = 0;
    // SAFETY: This is the only place we ever referdece these and this function is never called again.
    let key = unsafe { &mut *{ &raw mut KEY } };
    let key_len = unsafe { &mut *{ &raw mut KEY_LEN } };
    let fin = unsafe { &mut *{ &raw mut FIN } };
    let checksum = unsafe { &mut *{ &raw mut CHECKSUM } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(rijndael_dec_log(
            black_box(key), black_box(key_len),
            black_box(fin), black_box(checksum),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(rijndael_dec_std(
            black_box(key), black_box(key_len),
            black_box(fin), black_box(checksum),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(rijndael_dec());
    
    #[cfg(feature = "log")]
    {
        rprintln!("rijndael_dec: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(rijndael_dec());

    #[cfg(feature = "log")]
    std::println!("rijndael_dec: {}", success);

    std::process::exit(!success as i32);
}
