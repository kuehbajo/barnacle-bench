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

  FUNCTION

  The AES algorithm Rijndael implemented for block and key sizes of 128,
  bits (16 bytes) by Brian Gladman.

  This is an implementation of the AES encryption algorithm (Rijndael)
  designed by Joan Daemen and Vincent Rijmen.
*/

use core::ops::BitAnd;

use crate::aestab::{
    RIJNDAEL_DEC_FL_TAB, RIJNDAEL_DEC_IM_TAB, RIJNDAEL_DEC_IT_TAB, RIJNDAEL_DEC_IL_TAB, RIJNDAEL_DEC_RCON_TAB
};

/*
##################
    aes.h
##################
*/

/* The only supported block size for the benchmark is 16 */
const BLOCK_SIZE: usize = 16;

/*
  The number of key schedule words for different block and key lengths
  (allowing for the method of computation which requires the length to
  be a multiple of the key length):

  Key Schedule    key length (bytes)
  Length          16  20  24  28  32
               ---------------------
  block     16 |  44  60  54  56  64
  length    20 |  60  60  66  70  80
  (bytes)   24 |  80  80  78  84  96
            28 | 100 100 102  98 112
            32 | 120 120 120 126 120

  Rcon Table      key length (bytes)
  Length          16  20  24  28  32
               ---------------------
  block     16 |  10   9   8   7   7
  length    20 |  14  11  10   9   9
  (bytes)   24 |  19  15  12  11  11
            28 |  24  19  16  13  13
            32 |  29  23  19  17  14

  The following values assume that the key length will be variable and may
  be of maximum length (32 bytes).

  Nk = number_of_key_bytes / 4
  Nc = number_of_columns_in_state / 4
  Nr = number of encryption/decryption rounds
  Rc = number of elements in rcon table
  Ks = number of 32-bit words in key schedule
*/

#[inline]
fn nr(nk: Word, nc: Word) -> Word {
    if nk > nc { nk + 6 } else { nc + 6 }
}

const KS_LENGTH: usize = 4 * BLOCK_SIZE;

/* End of configuration options, but see also aes.c */

type Byte = u8;                         /* must be an 8-bit storage unit */
pub type Word = u32;                    /* must be a 32-bit storage unit */
type AESRet = i16;                      /* function return value         */

const AES_BAD: AESRet = 0;
const AES_GOOD: AESRet = 1;

/*
  upr(x,n): rotates bytes within words by n positions, moving bytes
  to higher index positions with wrap around into low positions
  ups(x,n): moves bytes by n positions to higher index positions in
  words but without wrap around
  bval(x,n): extracts a byte from a word
*/

#[inline]
fn bval(x: Word, n: u8) -> Byte {
    ((x >> (8 * n)) & 0xFF) as Byte
}

struct AESConst;
impl AESConst {
    const NCOL: usize =  BLOCK_SIZE / 4;
}

#[derive(Clone, Copy)]
#[allow(unused)]
pub enum AESKey {
    Enc = 1,    /* set if encryption is needed */
    Dec = 2,    /* set if decryption is needed */
    Both = 3,   /* set if both are needed      */
}

impl BitAnd<u8> for AESKey {
    type Output = u8;

    fn bitand(self, rhs: u8) -> Self::Output {
        (self as u8) & rhs
    }
}

pub struct AES {
    nkey: Word,                    /* the number of words in the key input block */
    nrnd: Word,                    /* the number of cipher rounds                */
    e_key: [Word; KS_LENGTH ],      /* the encryption key schedule                */
    d_key: [Word; KS_LENGTH],       /* the decryption key schedule                */
    mode: Byte,                     /* encrypt, decrypt or both                   */
}

impl Default for AES {
    fn default() -> Self {
        AES {
            nkey: 0,
            nrnd: 0,
            e_key: [0; KS_LENGTH],
            d_key: [0; KS_LENGTH],
            mode: 0,
        }
    }
}

/*
##################
    aes.c
##################
*/

#[inline]
fn four_tables(
    x: Word,
    tab: &[[Word; 256]; 4],
    vf: fn(Word, Word, Word) -> Word,
    rf: fn(Word, Word) -> Word,
    c: Word,
) -> Word {
    tab[0][bval(vf(x,0,c), rf(0,c) as u8) as usize]
        ^ tab[1][bval(vf(x,1,c), rf(1,c)as u8) as usize]
        ^ tab[2][bval(vf(x,2,c), rf(2,c) as u8) as usize]
        ^ tab[3][bval(vf(x,3,c), rf(3,c) as u8) as usize]
}

#[inline]
fn vf1(x: Word, _r: Word, _c: Word) -> Word { x }

#[inline]
fn rf1(r: Word, _c: Word) -> Word { r }

#[inline]
fn rf2(r: Word, c: Word) -> Word { (r - c) & 3 }

#[inline]
fn ls_box(x: Word, c: Word) -> Word {
    four_tables(x, &RIJNDAEL_DEC_FL_TAB, vf1, rf2, c)
}

#[inline]
fn inv_mcol(x: Word) -> Word {
    four_tables(x, &RIJNDAEL_DEC_IM_TAB, vf1, rf1, 0)
}

/*
  Subroutine to set the block size (if variable) in bytes, legal
  values being 16, 24 and 32.
*/

const NC: usize = AESConst::NCOL;

/*
  Initialise the key schedule from the user supplied key. The key
  length is now specified in bytes - 16, 24 or 32 as appropriate.
  This corresponds to bit lengths of 128, 192 and 256 bits, and
  to Nk values of 4, 6 and 8 respectively.
*/

#[inline]
fn mx(t: &mut Word, f: &mut Word) {
    *t = inv_mcol(*f);
}

#[inline]
fn mix(d: &mut [Word], s: &mut [Word]) {
    mx(&mut d[0],&mut s[0]);
    mx(&mut d[1],&mut s[1]);
    mx(&mut d[2],&mut s[2]);
    mx(&mut d[3],&mut s[3]);
}

pub fn rijndael_dec_set_key(
    in_key: &mut [Byte],
    n_bytes: Word,
    f: AESKey,
    cx: &mut AES,
) -> AESRet {
    let cond1 = n_bytes & 7 != 0;
    let cond2 = n_bytes < 16;
    let cond3 = n_bytes > 32;
    let cond4 = !((f & 1) != 0) && !((f & 2) != 0);
    if cond1 || cond2 || cond3 || cond4 {
        if n_bytes > 0 {
            cx.mode &= !0x03;
            AES_BAD
        } else {
            (cx.nkey << 2) as AESRet
        };
    }

    cx.mode = (cx.mode & !0x03) | (f as Byte & 0x03);
    cx.nkey = n_bytes >> 2;
    cx.nrnd = nr(cx.nkey, NC as Word);

    cx.e_key[0] = u32::from_ne_bytes([in_key[0], in_key[1], in_key[2], in_key[3]]);
    cx.e_key[1] = u32::from_ne_bytes([in_key[4], in_key[5], in_key[6], in_key[7]]);
    cx.e_key[2] = u32::from_ne_bytes([in_key[8], in_key[9], in_key[10], in_key[11]]);
    cx.e_key[3] = u32::from_ne_bytes([in_key[12], in_key[13], in_key[14], in_key[15]]);

    let mut kf_i = 0;
    let kt_i = NC * (cx.nrnd + 1) as usize - cx.nkey as usize;
    let mut rci = 0;

    match cx.nkey {
        4 => {
            // loopbound min 0 max 0
            loop {
                cx.e_key[kf_i + 4] = cx.e_key[kf_i + 0] ^ ls_box(cx.e_key[kf_i + 3], 3) ^ RIJNDAEL_DEC_RCON_TAB[rci];
                cx.e_key[kf_i + 5] = cx.e_key[kf_i + 1] ^ cx.e_key[kf_i + 4];
                cx.e_key[kf_i + 6] = cx.e_key[kf_i + 2] ^ cx.e_key[kf_i + 5];
                cx.e_key[kf_i + 7] = cx.e_key[kf_i + 3] ^ cx.e_key[kf_i + 6];
                rci += 1;
                kf_i += 4;
                if kf_i >= kt_i {
                    break;
                }
            }
        },
        6 => {
            cx.e_key[4] = u32::from_ne_bytes([in_key[16], in_key[17], in_key[18], in_key[19]]);
            cx.e_key[5] = u32::from_ne_bytes([in_key[20], in_key[21], in_key[22], in_key[23]]);
            // loopbound min 0 max 0
            loop {
                cx.e_key[kf_i +  6] = cx.e_key[kf_i + 0] ^ ls_box(cx.e_key[kf_i + 5], 3) ^ RIJNDAEL_DEC_RCON_TAB[rci];
                cx.e_key[kf_i +  7] = cx.e_key[kf_i + 1] ^ cx.e_key[kf_i +  6];
                cx.e_key[kf_i +  8] = cx.e_key[kf_i + 2] ^ cx.e_key[kf_i +  7];
                cx.e_key[kf_i +  9] = cx.e_key[kf_i + 3] ^ cx.e_key[kf_i +  8];
                cx.e_key[kf_i + 10] = cx.e_key[kf_i + 4] ^ cx.e_key[kf_i +  9];
                cx.e_key[kf_i + 11] = cx.e_key[kf_i + 5] ^ cx.e_key[kf_i + 10];
                rci += 1;
                kf_i += 6;
                if kf_i >= kt_i {
                    break;
                }
            }
        },
        8 => {
            cx.e_key[4] = u32::from_ne_bytes([in_key[16], in_key[17], in_key[18], in_key[19]]);
            cx.e_key[5] = u32::from_ne_bytes([in_key[20], in_key[21], in_key[22], in_key[23]]);
            cx.e_key[6] = u32::from_ne_bytes([in_key[24], in_key[25], in_key[26], in_key[27]]);
            cx.e_key[7] = u32::from_ne_bytes([in_key[28], in_key[29], in_key[30], in_key[31]]);
            // loopbound min 7 max 7
            loop {
                cx.e_key[kf_i +  8] = cx.e_key[kf_i + 0] ^ ls_box(cx.e_key[kf_i + 7], 3) ^ RIJNDAEL_DEC_RCON_TAB[rci];
                cx.e_key[kf_i +  9] = cx.e_key[kf_i + 1] ^ cx.e_key[kf_i +  8];
                cx.e_key[kf_i + 10] = cx.e_key[kf_i + 2] ^ cx.e_key[kf_i +  9];
                cx.e_key[kf_i + 11] = cx.e_key[kf_i + 3] ^ cx.e_key[kf_i + 10];
                cx.e_key[kf_i + 12] = cx.e_key[kf_i + 4] ^ ls_box(cx.e_key[kf_i + 11], 0);
                cx.e_key[kf_i + 13] = cx.e_key[kf_i + 5] ^ cx.e_key[kf_i + 12];
                cx.e_key[kf_i + 14] = cx.e_key[kf_i + 6] ^ cx.e_key[kf_i + 13];
                cx.e_key[kf_i + 15] = cx.e_key[kf_i + 7] ^ cx.e_key[kf_i + 14];
                rci += 1;
                kf_i += 8;
                if kf_i >= kt_i {
                    break;
                }
            }
        },
        _ => {}
    }

    if (cx.mode & 3) != AESKey::Enc as u8 {
        let mut kt_i = NC * cx.nrnd as usize;
        let mut kf_i = 0;

        cx.d_key[kt_i..(kt_i + 4)].copy_from_slice(&cx.e_key[kf_i..(kf_i + 4)]);
        kt_i += 4;
        kf_i += 4;

        kt_i -= 2 * NC;

        // loopbound min 13 max 13
        for _ in 1..cx.nrnd {
            mix(&mut cx.d_key[kt_i..], &mut cx.e_key[kf_i..]);
            kt_i += 4;
            kf_i += 4;
            kt_i -= 2 * NC;
        }
        cx.d_key[kt_i..(kt_i + 4)].copy_from_slice(&cx.e_key[kf_i..(kf_i + 4)]);
    }
    return AES_GOOD;
}

#[inline(always)]
fn rijndael_dec_round(
    b0: &[u32; 4],
    b1: &mut [u32; 4],
    kp: &[u32],
    tab: &[[u32; 256]; 4],
) {
    b1[0] = kp[0]
        ^ tab[0][(b0[0]         & 0xff) as usize]
        ^ tab[1][((b0[3] >>  8) & 0xff) as usize]
        ^ tab[2][((b0[2] >> 16) & 0xff) as usize]
        ^ tab[3][((b0[1] >> 24) & 0xff) as usize];

    b1[1] = kp[1]
        ^ tab[0][(b0[1]         & 0xff) as usize]
        ^ tab[1][((b0[0] >>  8) & 0xff) as usize]
        ^ tab[2][((b0[3] >> 16) & 0xff) as usize]
        ^ tab[3][((b0[2] >> 24) & 0xff) as usize];

    b1[2] = kp[2]
        ^ tab[0][(b0[2]         & 0xff) as usize]
        ^ tab[1][((b0[1] >>  8) & 0xff) as usize]
        ^ tab[2][((b0[0] >> 16) & 0xff) as usize]
        ^ tab[3][((b0[3] >> 24) & 0xff) as usize];

    b1[3] = kp[3]
        ^ tab[0][(b0[3]         & 0xff) as usize]
        ^ tab[1][((b0[2] >>  8) & 0xff) as usize]
        ^ tab[2][((b0[1] >> 16) & 0xff) as usize]
        ^ tab[3][((b0[0] >> 24) & 0xff) as usize];
}

#[inline(always)]
fn rijndael_dec_two_rounds(
    b0: &mut [u32; 4],
    b1: &mut [u32; 4],
    kp: &[u32],
    tab: &[[u32; 256]; 4],
) {
    rijndael_dec_round(b0, b1, &kp[0..4], tab);
    rijndael_dec_round(b1, b0, &kp[4..8], tab);
}

pub fn rijndael_dec_decrypt(
    in_blk: &[u8; 16],
    out_blk: &mut [u8; 16],
    cx: &AES,
) -> i16 {
    let mut kp: &[u32] = &cx.d_key;
    if (cx.mode & 2) == 0 { return 0; }

    let mut b0: [u32; 4] = [0; 4];
    
    let bytes = u32::from_ne_bytes([in_blk[0], in_blk[1], in_blk[2], in_blk[3]]);
    b0[0] = bytes ^ kp[0];
    let bytes = u32::from_ne_bytes([in_blk[4], in_blk[5], in_blk[6], in_blk[7]]);
    b0[1] = bytes ^ kp[1];
    let bytes = u32::from_ne_bytes([in_blk[8], in_blk[9], in_blk[10], in_blk[11]]);
    b0[2] = bytes ^ kp[2];
    let bytes = u32::from_ne_bytes([in_blk[12], in_blk[13], in_blk[14], in_blk[15]]);
    b0[3] = bytes ^ kp[3];
    kp = &kp[4..];

    let mut b1: [u32; 4] = [0; 4];

    // 2 additional rounds for AES-256 -> 14 rounds
    if cx.nrnd >= 14 {
        rijndael_dec_two_rounds(&mut b0, &mut b1, kp, &RIJNDAEL_DEC_IT_TAB);
        kp = &kp[8..];
    }

    // 2 additional rounds for AES-192 -> 12 rounds
    if cx.nrnd >= 12 {
        rijndael_dec_two_rounds(&mut b0, &mut b1, kp, &RIJNDAEL_DEC_IT_TAB);
        kp = &kp[8..];
    }

    // 10 rounds for AES-128
    for _ in 0..4 {
        rijndael_dec_two_rounds(&mut b0, &mut b1, kp, &RIJNDAEL_DEC_IT_TAB);
        kp = &kp[8..];
    }
    rijndael_dec_round(&mut b0, &mut b1, kp, &RIJNDAEL_DEC_IT_TAB);
    kp = &kp[4..];
    rijndael_dec_round(&mut b1, &mut b0, kp, &RIJNDAEL_DEC_IL_TAB);
    
    out_blk[0..4].copy_from_slice(&b0[0].to_ne_bytes());
    out_blk[4..8].copy_from_slice(&b0[1].to_ne_bytes());
    out_blk[8..12].copy_from_slice(&b0[2].to_ne_bytes());
    out_blk[12..16].copy_from_slice(&b0[3].to_ne_bytes());
    
    AES_GOOD
}
