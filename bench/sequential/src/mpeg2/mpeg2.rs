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

  Name: mpeg2

  Author: MPEG Software Simulation Group

  Function: This file contains MPEG 2 motion estimation code.

  Source: MediaBench II
          http://euler.slu.edu/~fritts/mediabench (mirror)

  Original name: MPEG-2

  Changes: For TACLeBench, the code has been changed such that input and
    output arrays are scaled down so that the code plus its data fits to the
    memories of typical resource-constrained embedded devices.

  License: See the following disclaimer from the original source codes.

    Disclaimer of Warranty

    These software programs are available to the user without any license fee or
    royalty on an "as is" basis.  The MPEG Software Simulation Group disclaims
    any and all warranties, whether express, implied, or statuary, including any
    implied warranties or merchantability or of fitness for a particular
    purpose.  In no event shall the copyright-holder be liable for any
    incidental, punitive, or consequential damages of any kind whatsoever
    arising from the use of these programs.

    This disclaimer of warranty extends to the user of these programs and user's
    customers, employees, agents, transferees, successors, and assigns.

    The MPEG Software Simulation Group does not represent or warrant that the
    programs furnished hereunder are free of infringement of any third-party
    patents.

    Commercial implementations of MPEG-1 and MPEG-2 video, including shareware,
    are subject to royalty fees to patent holders.  Many of these patents are
    general enough such that they are unavoidable regardless of implementation
    design.

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

mod input;
use input::MPEG2_OLDORGFRAME;

#[derive(Clone, Copy)]
struct Mbinfo {
    mb_type: i32,
    motion_type: i32,
    mv: [[[i32; 2]; 2]; 2],
    mv_field_sel: [[i32; 2]; 2],
    dmvector: [i32; 2],
    var: i32,
}

impl Mbinfo {
    const fn new() -> Self {
        Mbinfo {
            mb_type: 0,
            motion_type: 0,
            mv: [[[0; 2]; 2]; 2],
            mv_field_sel: [[0; 2]; 2],
            dmvector: [0; 2],
            var: 0,
        }
    }
}

// changed: lowered width, height, width2, height2 to accommodate lower input size
const MPEG2_WIDTH: usize = 176;
const MPEG2_HEIGHT: usize = 128;
const MPEG2_WIDTH2: usize = 176;
const MPEG2_HEIGHT2: usize = 128;
const MPEG2_M: i32 = 3;
const MPEG2_PICT_TYPE: i32 = 3;
const MPEG2_PICT_STRUCT: i32 = 3;
const MPEG2_TOPFIRST: i32 = 1;
const MPEG2_FRAME_PRED_DCT: i32 = 0;

/*
  Initialization- and return-value-related functions
*/

fn mpeg2_init(
  mbis: &mut [Mbinfo; 88],
  orgframe: &mut [u8; 22528]
) {

    *mbis = [Mbinfo::new(); 88];
    *orgframe = MPEG2_OLDORGFRAME;
    let bitmask: u8 = 0;
    /*
        Apply volatile XOR-bitmask to entire input array.
    */
    // loopbound min 22528 max 22528
    for elem in orgframe {
        *elem ^= bitmask;
    }
}


fn mpeg2_return(mpeg2_mbinfo: &[Mbinfo; 88]) -> i32 {
    let mut checksum = 0;
    // loopbound min 88 max 88
    for i in 0..88 {
        // loopbound min 2 max 2
        for j in 0..2 {
            // loopbound min 2 max 2
            for k in 0..2 {
                // loopbound min 2 max 2
                for l in 0..2 {
                    checksum += mpeg2_mbinfo[i].mv[j][k][l];
                }
            }
        }
    }
    checksum
}


/*
  Algorithm core functions
*/

/*
  motion estimation for progressive and interlaced frame pictures

  oldorg: source frame for forward prediction (used for P and B frames)
  neworg: source frame for backward prediction (B frames only)
  oldref: reconstructed frame for forward prediction (P and B frames)
  newref: reconstructed frame for backward prediction (B frames only)
  cur:    current frame (the one for which the prediction is formed)
  sxf,syf: forward search window (frame coordinates)
  sxb,syb: backward search window (frame coordinates)
  mbi:    pointer to macroblock info structure

  results:
  mbi.
   mb_type: 0, MB_INTRA, MB_FORWARD, MB_BACKWARD, MB_FORWARD|MB_BACKWARD
   MV[][][]: motion vectors (frame format)
   mv_field_sel: top/bottom field (for field prediction)
   motion_type: MC_FRAME, MC_FIELD

  uses global vars: mpeg2_pict_type, frame_pred_dct
*/
fn mpeg2_motion_estimation(
    oldorg: &[u8; 22528],
    neworg: &[u8; 22528],
    oldref: &[u8; 22528],
    newref: &[u8; 22528],
    cur: &[u8; 22528],
    curref: &[u8; 22528],
    sxf: i32,
    syf: i32,
    sxb: i32,
    syb: i32,
    mbis: &mut [Mbinfo; 88],
    secondfield: i32,
    ipflag: i32,
) {
    let mut mbi_i = 0;
    /* loop through all macroblocks of the picture */
    // loopbound min 8 max 8
    for j in (0..MPEG2_HEIGHT2).step_by(16) {
        // loopbound min 11 max 11
        for i in (0..MPEG2_WIDTH).step_by(16) {
            if MPEG2_PICT_STRUCT == 3 {
                mpeg2_frame_me(
                    oldorg, neworg, oldref, newref, cur, i, j, sxf, syf, sxb, syb, &mut mbis[mbi_i],
                );
            } else {
                mpeg2_field_me(
                    oldorg, neworg, oldref, newref, cur, curref, i, j, sxf, syf, sxb, syb,
                    &mut mbis[mbi_i], secondfield, ipflag,
                );
            }
            mbi_i += 1;
        }
    }
}

#[inline(always)]
fn mpeg2_field_forward_helper(
    mbi: &mut Mbinfo,
    oldref: &[u8],
    mb: &[u8],
    width: usize,
    forward: ForwardParams,
) -> i32 {
    let mut vmc: i32;
    mbi.motion_type = 1;
    mbi.mb_type = 8;

    let index = (forward.imintf >> 1) as usize
        + if forward.tself != 0 { width } else { 0 }
        + (width << 1) * (forward.jmintf >> 1) as usize;
    vmc = mpeg2_dist2(
        &oldref[index..], mb, width << 1, forward.imintf & 1, forward.jmintf & 1, 8
    );
    let index = (forward.iminbf >> 1) as usize
        + if forward.bself != 0 { width } else { 0 }
        + (width << 1) * (forward.jminbf >> 1) as usize;
    vmc += mpeg2_dist2(
        &oldref[index..], &mb[width..], width << 1, forward.iminbf & 1, forward.jminbf & 1, 8
    );

    vmc
}

#[inline(always)]
fn mpeg2_field_backward_helper(
    mbi: &mut Mbinfo,
    newref: &[u8],
    mb: &[u8],
    width: usize,
    backward: BackwardParams,
) -> i32 {
    let mut vmc: i32;
    mbi.motion_type = 1;
    mbi.mb_type = 4;

    let index = (backward.imintr >> 1) as usize
        + if backward.tselr != 0 { width } else { 0 }
        + (width << 1) * (backward.jmintr >> 1) as usize;
    vmc = mpeg2_dist2(
        &newref[index..], mb, width << 1, backward.imintr & 1, backward.jmintr & 1, 8
    );
    let index = (backward.iminbr >> 1) as usize
        + if backward.bselr != 0 { width } else { 0 }
        + (width << 1) * (backward.jminbr >> 1) as usize;
    vmc += mpeg2_dist2(
        &newref[index..], &mb[width..], width << 1, backward.iminbr & 1, backward.jminbr & 1, 8
    );

    vmc
}

#[inline(always)]
fn mpeg2_field_interpolated_helper(
    mbi: &mut Mbinfo,
    oldref: &[u8],
    newref: &[u8],
    mb: &[u8],
    width: usize,
    forward: ForwardParams,
    backward: BackwardParams,
) -> i32 {
    let mut vmc: i32;
    mbi.motion_type = 1;
    mbi.mb_type = 8 | 4;

    let index1 = (forward.imintf >> 1) as usize
        + if forward.tself != 0 { width } else { 0 }
        + (width << 1) * (forward.jmintf >> 1) as usize;
    let index2 = (backward.imintr >> 1) as usize
        + if backward.tselr != 0 { width } else { 0 }
        + (width << 1) * (backward.jmintr >> 1) as usize;
    vmc = mpeg2_bdist2(
        &oldref[index1..], &newref[index2..],
        mb, width << 1,
        forward.imintf & 1, forward.jmintf & 1,
        backward.imintr & 1, backward.jmintr & 1,
        8
    );

    let index1 = (forward.iminbf >> 1) as usize
        + if forward.bself != 0 { width } else { 0 }
        + (width << 1) * (forward.jminbf >> 1) as usize;
    let index2 = (backward.iminbr >> 1) as usize
        + if backward.bselr != 0 { width } else { 0 }
        + (width << 1) * (backward.jminbr >> 1) as usize;
    vmc += mpeg2_bdist2(
        &oldref[index1..], &newref[index2..],
        &mb[width..], width << 1,
        forward.iminbf & 1, forward.jminbf & 1,
        backward.iminbr & 1, backward.jminbr & 1,
        8
    );

    vmc
}

#[inline(always)]
fn mpeg2_frame_forward_helper(
    mbi: &mut Mbinfo,
    oldref: &[u8],
    mb: &[u8],
    width: usize,
    iminf: i32, jminf: i32,
) -> i32 {
    mbi.motion_type = 2;
    mbi.mb_type =  8;
    let index = (iminf >> 1) as usize + width * (jminf >> 1) as usize;
    mpeg2_dist2(
        &oldref[index..],
        mb, width,
        iminf & 1, jminf & 1,
        16
    )
}

#[inline(always)]
fn mpeg2_frame_backward_helper(
    mbi: &mut Mbinfo,
    newref: &[u8],
    mb: &[u8],
    width: usize,
    iminr: i32, jminr: i32,
) -> i32 {
    mbi.motion_type = 2;
    mbi.mb_type =  4;
    let index = (iminr >> 1) as usize + width * (jminr >> 1) as usize;
    mpeg2_dist2(
        &newref[index..],
        mb, width,
        iminr & 1, jminr & 1,
        16
    )
}

#[inline(always)]
fn mpeg2_frame_interpolated_helper(
    mbi: &mut Mbinfo,
    oldref: &[u8],
    newref: &[u8],
    mb: &[u8],
    width: usize,
    iminf: i32, jminf: i32,
    iminr: i32, jminr: i32,
) -> i32 {
    mbi.motion_type = 2;
    mbi.mb_type =  8 | 4;
    let index1 = (iminf >> 1) as usize + width * (jminf >> 1) as usize;
    let index2 = (iminr >> 1) as usize + width * (jminr >> 1) as usize;
    mpeg2_bdist2(
        &oldref[index1..],
        &newref[index2..],
        mb, width,
        iminf & 1, jminf & 1,
        iminr & 1, jminr & 1,
        16
    )
}

#[derive(Default)]
struct ForwardParams {
    tself: i32, bself: i32,
    imintf: i32, jmintf: i32,
    iminbf: i32, jminbf: i32,
}

#[derive(Default)]
struct BackwardParams {
    tselr: i32, bselr: i32,
    imintr: i32, jmintr: i32,
    iminbr: i32, jminbr: i32,
}

#[derive(Default)]
struct IParams {
    imin: i32, iminf: i32, iminr: i32,
    imint: i32, iminb: i32,
    imintf: i32, iminbf: i32,
    imintr: i32, iminbr: i32,
    imins: [[i32; 2]; 2],
    imindp: i32, imindmv: i32,
}

#[derive(Default)]
struct JParams {
    jmin: i32, jminf: i32, jminr: i32,
    jmint: i32, jminb: i32,
    jmintf: i32, jminbf: i32,
    jmintr: i32, jminbr: i32,
    jmins: [[i32; 2]; 2],
    jmindp: i32,jmindmv: i32,
}

fn mpeg2_frame_me(
    oldorg: &[u8; 22528],
    neworg: &[u8; 22528],
    oldref: &[u8; 22528],
    newref: &[u8; 22528],
    cur: &[u8; 22528],
    i: usize,
    j: usize,
    sxf: i32,
    syf: i32,
    sxb: i32,
    syb: i32,
    mbi: &mut Mbinfo,
) {
    let mut ips = IParams::default();
    let mut jps = JParams::default();

    let mut tsel: i32 = 0;
    let mut tself: i32 = 0;
    let mut tselr: i32 = 0;

    let mut bsel: i32 = 0;
    let mut bself: i32 = 0;
    let mut bselr: i32 = 0;

    let mut dmc: i32 = 0;
    let mut dmc_dp: i32 = 0;
    let mut dmcf: i32 = 0;
    let mut dmcr: i32 = 0;
    let dmci: i32;

    let mut dmcfield: i32 = 0;
    let mut dmcfieldf: i32 = 0;
    let mut dmcfieldr: i32 = 0;
    let mut dmcfieldi: i32;

    let mut vmc: i32;
    let mut vmc_dp: i32 = 0;
    let vmcf: i32;
    let vmcr: i32;
    let vmci: i32;

    let mb = &cur[(i + MPEG2_WIDTH * j)..]; 

    let mut var: i32 = mpeg2_variance(mb, MPEG2_WIDTH);

    let width = MPEG2_WIDTH as i32;
    let height = MPEG2_HEIGHT as i32;

    if MPEG2_PICT_TYPE == 1 {
        mbi.mb_type = 1;
    } else if MPEG2_PICT_TYPE == 2 {
      if MPEG2_FRAME_PRED_DCT != 0 {
          mpeg2_fullsearch(
              oldorg, oldref, mb, width, i, j, sxf, syf, 16, width,
              height, &mut ips.imin, &mut jps.jmin,
          );
          let index = (ips.imin >> 1) as usize + MPEG2_WIDTH * (jps.jmin >> 1) as usize;
          vmc =
            mpeg2_dist2(
              &oldref[index..], mb, MPEG2_WIDTH,
              ips.imin & 1, jps.jmin & 1, 16 );
          mbi.motion_type = 2;
      } else {
          mpeg2_frame_estimate(
              oldorg, oldref, mb, i, j, sxf, syf,
              &mut ips.imin, &mut jps.jmin, &mut ips.imint,
              &mut jps.jmint, &mut ips.iminb, &mut jps.jminb,
              &mut dmc, &mut dmcfield,
              &mut tsel, &mut bsel, &mut ips.imins, &mut jps.jmins,
          );

          if MPEG2_M == 1 {
              mpeg2_dpframe_estimate(
                  oldref, mb, i, j >> 1, ips.imins, jps.jmins,
                  &mut ips.imindp, &mut jps.jmindp, &mut ips.imindmv,
                  &mut jps.jmindmv, &mut dmc_dp, &mut vmc_dp
              );
          }

          /* select between dual prime, frame and field prediction */
          if (MPEG2_M == 1) && (dmc_dp < dmc) && (dmc_dp < dmcfield) {
              mbi.motion_type = 3;
              vmc = vmc_dp;
          } else if dmc <= dmcfield {
                mbi.motion_type = 2;
                let index = (ips.imin >> 1) as usize + MPEG2_WIDTH * (jps.jmin >> 1) as usize;
                vmc =
                  mpeg2_dist2(
                    &oldref[index..], mb,
                    MPEG2_WIDTH, ips.imin & 1, jps.jmin & 1, 16 );
            } else {
                mbi.motion_type = 1;
                let index = if tsel != 0 { MPEG2_WIDTH } else { 0 }
                    + (ips.imint >> 1) as usize
                    + (MPEG2_WIDTH << 1) * (jps.jmint >> 1) as usize;
                vmc = mpeg2_dist2(&oldref[index..], mb, MPEG2_WIDTH << 1, ips.imint & 1, jps.jmint & 1, 8);
                let index = if bsel != 0 { MPEG2_WIDTH } else { 0 }
                    + (ips.iminb >> 1) as usize
                    + (MPEG2_WIDTH << 1) * (jps.jminb >> 1) as usize;
                vmc += mpeg2_dist2(&oldref[index..], &mb[MPEG2_WIDTH..], MPEG2_WIDTH << 1, ips.iminb & 1, jps.jminb & 1, 8);
            }
      }

      /*
        select between intra or non-intra coding:

        selection is based on intra block variance (var) vs.
        prediction error variance (vmc)

        blocks with small prediction error are always coded non-intra
        even if variance is smaller (is this reasonable?)
      */
      if (vmc > var) && (vmc >= 9 * 256) {
          mbi.mb_type = 1;
      } else {
        /*
          select between MC / No-MC

          use No-MC if var(No-MC) <= 1.25*var(MC)
          (i.e slightly biased towards No-MC)

          blocks with small prediction error are always coded as No-MC
          (requires no motion vectors, allows skipping)
        */
        let index = i + MPEG2_WIDTH * j;
        let v0 =
          mpeg2_dist2(&oldref[index..], mb, MPEG2_WIDTH, 0, 0, 16);

        if (4 * v0 > 5 * vmc) && (v0 >= 9 * 256) {
          /* use MC */
          var = vmc;
          mbi.mb_type = 8;

          if mbi.motion_type == 2 {
            mbi.mv[ 0 ][ 0 ][ 0 ] = ips.imin - (i << 1) as i32;
            mbi.mv[ 0 ][ 0 ][ 1 ] = jps.jmin - (j << 1) as i32;
          } else if mbi.motion_type == 3 {
              /* these are FRAME vectors */
              /* same parity vector */
              mbi.mv[ 0 ][ 0 ][ 0 ] = ips.imindp - (i << 1) as i32;
              mbi.mv[ 0 ][ 0 ][ 1 ] = ( jps.jmindp << 1 ) - (j << 1) as i32;

              /* opposite parity vector */
              mbi.dmvector[ 0 ] = ips.imindmv;
              mbi.dmvector[ 1 ] = jps.jmindmv;
            } else {
              /* these are FRAME vectors */
              mbi.mv[ 0 ][ 0 ][ 0 ] = ips.imint - (i << 1) as i32;
              mbi.mv[ 0 ][ 0 ][ 1 ] = ( jps.jmint << 1 ) - (j << 1) as i32;
              mbi.mv[ 1 ][ 0 ][ 0 ] = ips.iminb - (i << 1) as i32;
              mbi.mv[ 1 ][ 0 ][ 1 ] = ( jps.jminb << 1 ) - (j << 1) as i32;
              mbi.mv_field_sel[ 0 ][ 0 ] = tsel;
              mbi.mv_field_sel[ 1 ][ 0 ] = bsel;
            }
        } else {
          /* No-MC */
          var = v0;
          mbi.mb_type = 0;
          mbi.motion_type = 2;
          mbi.mv[ 0 ][ 0 ][ 0 ] = 0;
          mbi.mv[ 0 ][ 0 ][ 1 ] = 0;
        }
      }
    } else { /* if (mpeg2_pict_type==B_TYPE) */
      if MPEG2_FRAME_PRED_DCT != 0 {
        /* forward */
        mpeg2_fullsearch(
            oldorg, oldref, mb, width, i, j, sxf, syf, 16, width,
            height, &mut ips.iminf, &mut jps.jminf
        );
        let index = (ips.iminf >> 1) as usize + MPEG2_WIDTH * (jps.jminf >> 1) as usize;
        vmcf =
          mpeg2_dist2(
            &oldref[index..], mb,
            MPEG2_WIDTH, ips.iminf & 1, jps.jminf & 1, 16 );

        /* backward */
        mpeg2_fullsearch(
            neworg, newref, mb, width, i, j, sxb, syb, 16, width,
            height, &mut ips.iminr, &mut jps.jminr
        );
        let index = (ips.iminr >> 1) as usize + MPEG2_WIDTH * (jps.jminr >> 1) as usize;
        vmcr =
          mpeg2_dist2(
            &newref[index..], mb,
            MPEG2_WIDTH, ips.iminr & 1, jps.jminr & 1, 16 );

        /* interpolated (bidirectional) */
        let index1 = (ips.iminf >> 1) as usize + MPEG2_WIDTH * (jps.jminf >> 1) as usize;
        let index2 = (ips.iminr >> 1) as usize + MPEG2_WIDTH * (jps.jminr >> 1) as usize;
        vmci =
          mpeg2_bdist2(
            &oldref[index1..],
            &newref[index2..],
            mb, MPEG2_WIDTH, ips.iminf & 1, jps.jminf & 1, ips.iminr & 1, jps.jminr & 1, 16 );

        /* decisions */

        /*
          select between forward/backward/interpolated prediction:
          use the one with smallest mean sqaured prediction error
        */
        if (vmcf <= vmcr) && (vmcf <= vmci) {
          vmc = vmcf;
          mbi.mb_type = 8;
        } else if vmcr <= vmci {
            vmc = vmcr;
            mbi.mb_type = 4;
          } else {
            vmc = vmci;
            mbi.mb_type = 8 | 4;
          }

        mbi.motion_type = 2;
      } else {
        /* forward prediction */
        mpeg2_frame_estimate(
            oldorg, oldref, mb, i, j, sxf, syf,
            &mut ips.iminf, &mut jps.jminf,
            &mut ips.imintf, &mut jps.jmintf,
            &mut ips.iminbf, &mut jps.jminbf,
            &mut dmcf, &mut dmcfieldf,
            &mut tself, &mut bself,
            &mut ips.imins, &mut jps.jmins
        );

        /* backward prediction */
        mpeg2_frame_estimate(
            neworg, newref, mb, i, j, sxb, syb,
            &mut ips.iminr, &mut jps.jminr,
            &mut ips.imintr, &mut jps.jmintr,
            &mut ips.iminbr, &mut jps.jminbr,
            &mut dmcr, &mut dmcfieldr,
            &mut tselr, &mut bselr,
            &mut ips.imins, &mut jps.jmins
        );

        /* calculate interpolated distance */
        /* frame */
        let index1 = (ips.iminf >> 1) as usize + MPEG2_WIDTH * (jps.jminf >> 1) as usize;
        let index2 = (ips.iminr >> 1) as usize + MPEG2_WIDTH * (jps.jminr >> 1) as usize;
        dmci =
          mpeg2_bdist1(
            &oldref[index1..],
            &newref[index2..],
            mb, MPEG2_WIDTH, ips.iminf & 1, jps.jminf & 1, ips.iminr & 1, jps.jminr & 1, 16 );

        /* top field */
        let index1 = (ips.imintf >> 1) as usize
            + if tself != 0 { MPEG2_WIDTH } else { 0 }
            + (MPEG2_WIDTH << 1) * (jps.jmintf >> 1) as usize;
        let index2 = (ips.imintr >> 1) as usize
            + if tselr != 0 { MPEG2_WIDTH } else { 0 }
            + (MPEG2_WIDTH << 1) * (jps.jmintr >> 1) as usize;
        dmcfieldi = 
          mpeg2_bdist1(
              &oldref[index1..], &newref[index2..],
              mb, MPEG2_WIDTH << 1, ips.imintf & 1, jps.jmintf & 1, ips.imintr & 1, jps.jmintr & 1,
              8
          );

        /* bottom field */
        let index1 = (ips.iminbf >> 1) as usize
            + if bself != 0 { MPEG2_WIDTH } else { 0 }
            + (MPEG2_WIDTH << 1) * (jps.jminbf >> 1) as usize;
        let index2 = (ips.iminbr >> 1) as usize
            + if bselr != 0 { MPEG2_WIDTH } else { 0 }
            + (MPEG2_WIDTH << 1) * (jps.jminbr >> 1) as usize;
        dmcfieldi +=
          mpeg2_bdist1(
              &oldref[index1..], &newref[index2..],
              &mb[MPEG2_WIDTH..], MPEG2_WIDTH << 1, ips.iminbf & 1, jps.jminbf & 1, ips.iminbr & 1,
              jps.jminbr & 1, 8
          );

        /*
          select prediction type of minimum distance from the
          six candidates (field/frame * forward/backward/interpolated)
        */
        if (dmci < dmcfieldi) && (dmci < dmcf) && (dmci < dmcfieldf) &&
             (dmci < dmcr) && (dmci < dmcfieldr) {
          /* frame, interpolated */
            vmc = mpeg2_frame_interpolated_helper(
              mbi, oldref, newref, &mb, MPEG2_WIDTH, ips.iminf, jps.jminf, ips.iminr, jps.jminr
            );
        } else if (dmcfieldi < dmcf)
            && (dmcfieldi < dmcfieldf)
            && (dmcfieldi < dmcr)
            && (dmcfieldi < dmcfieldr)
        {
            /* field, interpolated */
            let forward = ForwardParams {
                tself, bself, imintf: ips.imintf, jmintf: jps.jmintf, iminbf: ips.iminbf, jminbf: jps.jminbf,
            };
            let backward = BackwardParams {
                tselr, bselr, imintr: ips.imintr, jmintr: jps.jmintr, iminbr: ips.iminbr, jminbr: jps.jminbr,
            };
            vmc = mpeg2_field_interpolated_helper(
              mbi, oldref, newref, &mb, MPEG2_WIDTH, forward, backward
            );
          } else if (dmcf < dmcfieldf) && (dmcf < dmcr) && (dmcf < dmcfieldr) {
              /* frame, forward */
              vmc = mpeg2_frame_forward_helper(
                mbi, oldref, &mb, MPEG2_WIDTH, ips.iminf, jps.jminf,
              );
            } else if (dmcfieldf < dmcr) && (dmcfieldf < dmcfieldr) {
                /* field, forward */
                let forward = ForwardParams {
                    tself, bself, imintf: ips.imintf, jmintf: jps.jmintf, iminbf: ips.iminbf, jminbf: jps.jminbf,
                };
                vmc = mpeg2_field_forward_helper(
                    mbi, oldref, &mb, MPEG2_WIDTH, forward
                );
              } else if dmcr < dmcfieldr {
                  /* frame, backward */
                  vmc = mpeg2_frame_backward_helper(
                    mbi, newref, &mb, MPEG2_WIDTH, ips.iminr, jps.jminr,
                  );
                } else {
                  /* field, backward */
                  let backward = BackwardParams {
                      tselr, bselr, imintr: ips.imintr, jmintr: jps.jmintr, iminbr: ips.iminbr, jminbr: jps.jminbr,
                  };
                  vmc = mpeg2_field_backward_helper(
                      mbi, newref, &mb, MPEG2_WIDTH, backward
                  );
                }
      }

      /*
        select between intra or non-intra coding:

        selection is based on intra block variance (var) vs.
        prediction error variance (vmc)

        blocks with small prediction error are always coded non-intra
        even if variance is smaller (is this reasonable?)
      */
      if (vmc > var) && (vmc >= 9 * 256) {
          mbi.mb_type = 1;
      } else {
        var = vmc;

        if mbi.motion_type == 2 {
          /* forward */
          mbi.mv[ 0 ][ 0 ][ 0 ] = ips.iminf - (i << 1) as i32;
          mbi.mv[ 0 ][ 0 ][ 1 ] = jps.jminf - (j << 1) as i32;
          /* backward */
          mbi.mv[ 0 ][ 1 ][ 0 ] = ips.iminr - (i << 1) as i32;
          mbi.mv[ 0 ][ 1 ][ 1 ] = jps.jminr - (j << 1) as i32;
        } else {
          /* these are FRAME vectors */
          /* forward */
          mbi.mv[ 0 ][ 0 ][ 0 ] = ips.imintf - (i << 1) as i32;
          mbi.mv[ 0 ][ 0 ][ 1 ] = ( jps.jmintf << 1 ) - (j << 1) as i32;
          mbi.mv[ 1 ][ 0 ][ 0 ] = ips.iminbf - (i << 1) as i32;
          mbi.mv[ 1 ][ 0 ][ 1 ] = ( jps.jminbf << 1 ) - (j << 1) as i32;
          mbi.mv_field_sel[ 0 ][ 0 ] = tself;
          mbi.mv_field_sel[ 1 ][ 0 ] = bself;
          /* backward */
          mbi.mv[ 0 ][ 1 ][ 0 ] = ips.imintr - (i << 1) as i32;
          mbi.mv[ 0 ][ 1 ][ 1 ] = ( jps.jmintr << 1 ) - (j << 1) as i32;
          mbi.mv[ 1 ][ 1 ][ 0 ] = ips.iminbr - (i << 1) as i32;
          mbi.mv[ 1 ][ 1 ][ 1 ] = ( jps.jminbr << 1 ) - (j << 1) as i32;
          mbi.mv_field_sel[ 0 ][ 1 ] = tselr;
          mbi.mv_field_sel[ 1 ][ 1 ] = bselr;
        }
      }
    }

  mbi.var = var;
}

/*
  motion estimation for field pictures

  oldorg: original frame for forward prediction (P and B frames)
  neworg: original frame for backward prediction (B frames only)
  oldref: reconstructed frame for forward prediction (P and B frames)
  newref: reconstructed frame for backward prediction (B frames only)
  cur:    current original frame (the one for which the prediction is formed)
  curref: current reconstructed frame (to predict second field from first)
  sxf,syf: forward search window (frame coordinates)
  sxb,syb: backward search window (frame coordinates)
  mbi:    pointer to macroblock info structure
  secondfield: indicates second field of a frame (in P fields this means
               that reference field of opposite parity is in curref instead
               of oldref)
  ipflag: indicates a P type field which is the second field of a frame
          in which the first field is I type (this restricts predictions
          to be based only on the opposite parity (=I) field)

  results:
  mbi.
   mb_type: 0, MB_INTRA, MB_FORWARD, MB_BACKWARD, MB_FORWARD|MB_BACKWARD
   MV[][][]: motion vectors (field format)
   mv_field_sel: top/bottom field
   motion_type: MC_FIELD, MC_16X8

  uses global vars: mpeg2_pict_type, mpeg2_pict_struct
*/
fn mpeg2_field_me(
    oldorg: &[u8; 22528],
    neworg: &[u8; 22528],
    oldref: &[u8; 22528],
    newref: &[u8; 22528],
    cur: &[u8; 22528],
    curref: &[u8; 22528],
    i: usize, j: usize,
    sxf: i32, syf: i32,
    sxb: i32, syb: i32,
    mbi: &mut Mbinfo,
    secondfield: i32,
    ipflag: i32,
) {
    let mut imin: i32 = 0;
    let mut imin8u: i32 = 0;
    let mut imin8l: i32 = 0;
    let mut iminf: i32 = 0;
    let mut imin8uf: i32 = 0;
    let mut imin8lf: i32 = 0;
    let mut iminr: i32 = 0;
    let mut imin8ur: i32 = 0;
    let mut imin8lr: i32 = 0;
    let mut imins: i32 = 0;
    let mut imindmv: i32 = 0;
    
    let mut jmin: i32 = 0;
    let mut jmin8u: i32 = 0;
    let mut jmin8l: i32 = 0;
    let mut jminf: i32 = 0;
    let mut jmin8uf: i32 = 0;
    let mut jmin8lf: i32 = 0;
    let mut jminr: i32 = 0;
    let mut jmin8ur: i32 = 0;
    let mut jmin8lr: i32 = 0;
    let mut jmins: i32 = 0;
    let mut jmindmv: i32 = 0;

    let mut sel: i32 = 0;
    let mut sel8u: i32 = 0;
    let mut sel8l: i32 = 0;

    let mut sel_f: i32 = 0;
    let mut sel8uf: i32 = 0;
    let mut sel8lf: i32 = 0;

    let mut selr: i32 = 0;
    let mut sel8ur: i32 = 0;
    let mut sel8lr: i32 = 0;

    let _dmc: i32;
    let mut dmc_dp: i32 = 0;
    let _dmcf: i32;
    let _dmcr: i32;
    let _dmci: i32;
    let mut dmc8: i32 = 0;
    let mut dmc8f: i32 = 0;
    let mut dmc8r: i32 = 0;
    let mut dmc8i: i32;

    let mut dmcfield: i32 = 0;
    let mut dmcfieldf: i32 = 0;
    let mut dmcfieldr: i32 = 0;
    let dmcfieldi: i32;

    let mut vmc: i32;
    let mut vmc_dp: i32 = 0;

    let mut ds: i32 = 0;
    let mut v0: i32 = 0;

    let w2 = MPEG2_WIDTH << 1;
    let mut mb = &cur[(i + w2 * j)..];

    if MPEG2_PICT_STRUCT == 2 {
        mb = &mb[MPEG2_WIDTH..];
    }

    let mut var = mpeg2_variance(mb, w2);

    if MPEG2_PICT_TYPE == 1 {
        mbi.mb_type = 1;
    } else if MPEG2_PICT_TYPE == 2 {
        let mut toporg = oldorg;
        let mut topref = oldref;
        let mut botorg = &oldorg[MPEG2_WIDTH..];
        let mut botref = &oldref[MPEG2_WIDTH..];

        if secondfield != 0 {
            /* opposite parity field is in same frame */
            if MPEG2_PICT_STRUCT == 1 {
                /* current is top field */
                botorg = &cur[MPEG2_WIDTH..];
                botref = &curref[MPEG2_WIDTH..];
            } else {
                /* current is bottom field */
                toporg = &cur;
                topref = &curref;
            }
        }

        mpeg2_field_estimate(
            toporg, topref, botorg, botref, mb, i, j, sxf, syf, ipflag,
            &mut imin, &mut jmin, &mut imin8u, &mut jmin8u,
            &mut imin8l, &mut jmin8l, &mut dmcfield, &mut dmc8,
            &mut sel, &mut sel8u, &mut sel8l,
            &mut imins, &mut jmins, &mut ds
        );

        if (MPEG2_M == 1) && (ipflag == 0) {
            /* generic condition which permits Dual Prime */
            mpeg2_dpfield_estimate(
              topref, botref, mb, i as i32, j as i32, imins, jmins,
              &mut imindmv, &mut jmindmv, &mut dmc_dp,
              &mut vmc_dp
            );
        }

        /* select between dual prime, field and 16x8 prediction */
        if (MPEG2_M == 1) && (ipflag == 0) && (dmc_dp < dmc8) && (dmc_dp < dmcfield) {
            /* Dual Prime prediction */
            mbi.motion_type = 3;
            vmc = vmc_dp;     /* we already calculated L2 error for Dual */
        } else if dmc8 < dmcfield {
            /* 16x8 prediction */
            mbi.motion_type = 2;
            /* upper half block */
            let index = (imin8u >> 1) as usize + w2 * (jmin8u >> 1) as usize;
            vmc =
              mpeg2_dist2(
                if sel8u != 0 { &botref[index..] } else { &topref[index..] },
                mb, w2, imin8u & 1, jmin8u & 1, 8 );
            /* lower half block */
            let index1 = (imin8l >> 1) as usize + w2 * (jmin8l >> 1) as usize;
            let index2 = 8 * w2 as usize;
            vmc +=
              mpeg2_dist2(
                if sel8l != 0 { &botref[index1..] } else { &topref[index1..] },
                &mb[index2..], w2, imin8l & 1, jmin8l & 1, 8 );
          } else {
              /* field prediction */
              mbi.motion_type = 1;
              let index = (imin >> 1) as usize + w2 * (jmin >> 1) as usize;
              vmc =
                mpeg2_dist2(
                  if sel != 0 { &botref[index..] } else { &topref[index..] },
                  mb, w2, imin & 1, jmin & 1, 16 );
          }

        /* select between intra and non-intra coding */
        if (vmc > var) && (vmc >= 9 * 256) {
          mbi.mb_type = 1;
        }
        else {
          /*
            zero MV field prediction from same parity ref. field
            (not allowed if ipflag is set)
          */
          if ipflag == 0 {
              let index = i + w2 * j;
              v0 =
              mpeg2_dist2(
                if MPEG2_PICT_STRUCT == 2 { &botref[index..] } else { &topref[index..] }, mb,
                w2, 0, 0, 16 );
          }

          if ipflag == 0 || ((4 * v0 > 5 * vmc) && (v0 >= 9 * 256)) {
            var = vmc;
            mbi.mb_type = 8;

            if mbi.motion_type == 1 {
              mbi.mv[0][0][0] = imin - (i << 1) as i32;
              mbi.mv[0][0][1] = jmin - (j << 1) as i32;
              mbi.mv_field_sel[0][0] = sel;
            } else if mbi.motion_type == 3 {
                /* same parity vector */
                mbi.mv[0][0][0] = imins - (i << 1) as i32;
                mbi.mv[0][0][1] = jmins - (j << 1) as i32;

                /* opposite parity vector */
                mbi.dmvector[0] = imindmv;
                mbi.dmvector[1] = jmindmv;
              } else {
                mbi.mv[0][0][0] = imin8u - (i << 1) as i32;
                mbi.mv[0][0][1] = jmin8u - (j << 1) as i32;
                mbi.mv[1][0][0] = imin8l - (i << 1) as i32;
                mbi.mv[1][0][1] = jmin8l - ((j + 8) << 1) as i32;
                mbi.mv_field_sel[0][0] = sel8u;
                mbi.mv_field_sel[1][0] = sel8l;
              }
          } else {
            /* No MC */
            var = v0;
            mbi.mb_type = 0;
            mbi.motion_type = 1;
            mbi.mv[0][0][0] = 0;
            mbi.mv[0][0][1] = 0;
            mbi.mv_field_sel[0][0] = (MPEG2_PICT_STRUCT == 2) as i32;
          }
        }
      } else { /* if (mpeg2_pict_type==B_TYPE) */
        /* forward prediction */
        mpeg2_field_estimate(
          oldorg, oldref, &oldorg[MPEG2_WIDTH..], &oldref[MPEG2_WIDTH..], mb, i, j, sxf,
          syf, 0, &mut iminf, &mut jminf, &mut imin8uf,
          &mut jmin8uf, &mut imin8lf, &mut jmin8lf,
          &mut dmcfieldf, &mut dmc8f, &mut sel_f,
          &mut sel8uf, &mut sel8lf, &mut imins,
          &mut jmins, &mut ds
        );

        /* backward prediction */
        mpeg2_field_estimate(
          neworg, newref, &neworg[MPEG2_WIDTH..], &newref[MPEG2_WIDTH..], mb, i, j, sxb,
          syb, 0, &mut iminr, &mut jminr, &mut imin8ur,
          &mut jmin8ur, &mut imin8lr, &mut jmin8lr,
          &mut dmcfieldr, &mut dmc8r, &mut selr,
          &mut sel8ur, &mut sel8lr, &mut imins,
          &mut jmins, &mut ds
        );

        /* calculate distances for bidirectional prediction */
        /* field */
        let index1 = if sel_f != 0 { MPEG2_WIDTH } else { 0 }
            + (iminf >> 1) as usize
            + w2 * (jminf >> 1) as usize;
        let index2 = if selr != 0 { MPEG2_WIDTH } else { 0 }
            + (iminr >> 1) as usize
            + w2 * (jminr >> 1) as usize;
        dmcfieldi =
          mpeg2_bdist1(
            &oldref[index1..],
            &newref[index2..],
            mb, w2, iminf & 1, jminf & 1, iminr & 1, jminr & 1, 16 );

        /* 16x8 upper half block */
        let index1 = if sel8uf != 0 { MPEG2_WIDTH } else { 0 }
            + ( imin8uf >> 1 ) as usize
            + w2 * (jmin8uf >> 1) as usize;
        let index2 = if sel8ur != 0 { MPEG2_WIDTH } else { 0 }
            + ( imin8ur >> 1 ) as usize
            + w2 * (jmin8ur >> 1) as usize;
        dmc8i =
          mpeg2_bdist1(
            &oldref[index1..],
            &newref[index2..],
            mb, w2, imin8uf & 1, jmin8uf & 1, imin8ur & 1, jmin8ur & 1, 8 );

        /* 16x8 lower half block */
        let index1 = if sel8lf != 0 { MPEG2_WIDTH } else { 0 }
            + (imin8lf >> 1) as usize
            + w2 * (jmin8lf >> 1) as usize;
        let index2 = if sel8lr != 0 { MPEG2_WIDTH } else { 0 }
            + (imin8lr >> 1) as usize
            + w2 * (jmin8lr >> 1) as usize;
        dmc8i +=
          mpeg2_bdist1(
            &oldref[index1..],
            &newref[index2..],
            &mb[(8 * w2)..], w2, imin8lf & 1, jmin8lf & 1, imin8lr & 1, jmin8lr & 1,
            8 );

        /* select prediction type of minimum distance */
        if ( dmcfieldi < dmc8i ) && ( dmcfieldi < dmcfieldf ) &&
            ( dmcfieldi < dmc8f ) && ( dmcfieldi < dmcfieldr ) &&
            ( dmcfieldi < dmc8r ) {
          /* field, interpolated */
          mbi.mb_type = 8 | 5;
          mbi.motion_type = 1;
          let index1 = if sel_f != 0 { MPEG2_WIDTH } else { 0 }
              + (iminf >> 1) as usize
              + w2 * (jminf >> 1) as usize;
          let index2 = if selr != 0 { MPEG2_WIDTH } else { 0 }
              + ( iminr >> 1 ) as usize
              + w2 * ( jminr >> 1 ) as usize;
          vmc =
            mpeg2_bdist2(
              &oldref[index1..],
              &newref[index2..],
              mb, w2, iminf & 1, jminf & 1, iminr & 1, jminr & 1, 16 );
        } else if (dmc8i < dmcfieldf) && (dmc8i < dmc8f) && (dmc8i < dmcfieldr) &&
              (dmc8i < dmc8r) {
            /* 16x8, interpolated */
            mbi.mb_type = 8 | 4;
            mbi.motion_type = 2;

            /* upper half block */
            let index1 = if sel8uf != 0 { MPEG2_WIDTH } else { 0 }
                + ( imin8uf >> 1 ) as usize
                + w2 * ( jmin8uf >> 1 ) as usize;
            let index2 = if sel8ur != 0 { MPEG2_WIDTH } else { 0 }
                + ( imin8ur >> 1 ) as usize
                + w2 * ( jmin8ur >> 1 ) as usize;
            vmc =
              mpeg2_bdist2(
                &oldref[index1..],
                &newref[index2..],
                mb, w2, imin8uf & 1, jmin8uf & 1, imin8ur & 1, jmin8ur & 1, 8 );

            /* lower half block */
            let index1 = if sel8lf != 0 { MPEG2_WIDTH } else { 0 }
                + (imin8lf >> 1) as usize
                + w2 * (jmin8lf >> 1) as usize;
            let index2 = if sel8lr != 0 { MPEG2_WIDTH } else { 0 }
                + (imin8lr >> 1) as usize
                + w2 * (jmin8lr >> 1) as usize;
            vmc +=
              mpeg2_bdist2(
                &oldref[index1..],
                &newref[index2..],
                &mb[(8 * w2)..], w2, imin8lf & 1, jmin8lf & 1, imin8lr & 1, jmin8lr & 1,
                8 );
          } else if (dmcfieldf < dmc8f) && (dmcfieldf < dmcfieldr) &&
                (dmcfieldf < dmc8r) {
              /* field, forward */
              mbi.mb_type = 8;
              mbi.motion_type = 1;
              let index = if sel_f != 0 { MPEG2_WIDTH } else { 0 }
                  + ( iminf >> 1 ) as usize
                  + w2 * ( jminf >> 1 ) as usize;
              vmc =
                mpeg2_dist2(
                  &oldref[index..],
                  mb, w2, iminf & 1, jminf & 1, 16 );
            } else if (dmc8f < dmcfieldr) && (dmc8f < dmc8r) {
                /* 16x8, forward */
                mbi.mb_type = 8;
                mbi.motion_type = 2;
                let index = if sel8uf != 0 { MPEG2_WIDTH } else { 0 }
                    + ( imin8uf >> 1 ) as usize
                    + w2 * ( jmin8uf >> 1 ) as usize;
                /* upper half block */
                vmc =
                  mpeg2_dist2(
                    &oldref[index..], mb, w2, imin8uf & 1, jmin8uf & 1, 8
                  );

                /* lower half block */
                let index = if sel8lf != 0 { MPEG2_WIDTH } else { 0 }
                    + ( imin8lf >> 1 ) as usize
                    + w2 * ( jmin8lf >> 1 ) as usize;
                vmc +=
                  mpeg2_dist2(
                    &oldref[index..],
                    &mb[(8 * w2)..], w2, imin8lf & 1, jmin8lf & 1, 8 );
              } else if dmcfieldr < dmc8r {
                  /* field, backward */
                  mbi.mb_type = 4;
                  mbi.motion_type = 1;
                  let index = if selr != 0 { MPEG2_WIDTH } else { 0 }
                      + ( iminr >> 1 ) as usize
                      + w2 * ( jminr >> 1 ) as usize;
                  vmc =
                    mpeg2_dist2(
                      &newref[index..],
                      mb, w2, iminr & 1, jminr & 1, 16 );
                } else {
                  /* 16x8, backward */
                  mbi.mb_type = 4;
                  mbi.motion_type = 2;
                  let index = if sel8ur != 0 { MPEG2_WIDTH } else { 0 }
                      + ( imin8ur >> 1 ) as usize
                      + w2 * ( jmin8ur >> 1 ) as usize;
                  /* upper half block */
                  vmc =
                    mpeg2_dist2(
                      &newref[index..],
                      mb, w2, imin8ur & 1, jmin8ur & 1, 8 );

                  /* lower half block */
                  let index = if sel8lr != 0 { MPEG2_WIDTH } else { 0 }
                      + ( imin8lr >> 1 ) as usize
                      + w2 * ( jmin8lr >> 1 ) as usize;
                  vmc +=
                    mpeg2_dist2(
                      &newref[index..],
                      &mb[(8 * w2)..], w2, imin8lr & 1, jmin8lr & 1, 8 );
                }

        /* select between intra and non-intra coding */
        if (vmc > var) && (vmc >= 9 * 256) {
            mbi.mb_type = 1;
        }
        else {
          var = vmc;
          if mbi.motion_type == 1 {
            /* forward */
            mbi.mv[ 0 ][ 0 ][ 0 ] = iminf - (i << 1) as i32;
            mbi.mv[ 0 ][ 0 ][ 1 ] = jminf - (j << 1) as i32;
            mbi.mv_field_sel[ 0 ][ 0 ] = sel_f;
            /* backward */
            mbi.mv[ 0 ][ 1 ][ 0 ] = iminr - (i << 1) as i32;
            mbi.mv[ 0 ][ 1 ][ 1 ] = jminr - (j << 1) as i32;
            mbi.mv_field_sel[ 0 ][ 1 ] = selr;
          } else { /* MC_16X8 */
            /* forward */
            mbi.mv[ 0 ][ 0 ][ 0 ] = imin8uf - (i << 1) as i32;
            mbi.mv[ 0 ][ 0 ][ 1 ] = jmin8uf - (j << 1) as i32;
            mbi.mv_field_sel[ 0 ][ 0 ] = sel8uf;
            mbi.mv[ 1 ][ 0 ][ 0 ] = imin8lf - (i << 1) as i32;
            mbi.mv[ 1 ][ 0 ][ 1 ] = jmin8lf - ((j + 8) << 1) as i32;
            mbi.mv_field_sel[ 1 ][ 0 ] = sel8lf;
            /* backward */
            mbi.mv[ 0 ][ 1 ][ 0 ] = imin8ur - (i << 1) as i32;
            mbi.mv[ 0 ][ 1 ][ 1 ] = jmin8ur - (j << 1) as i32;
            mbi.mv_field_sel[ 0 ][ 1 ] = sel8ur;
            mbi.mv[ 1 ][ 1 ][ 0 ] = imin8lr - (i << 1) as i32;
            mbi.mv[ 1 ][ 1 ][ 1 ] = jmin8lr - ((j + 8) << 1) as i32;
            mbi.mv_field_sel[ 1 ][ 1 ] = sel8lr;
          }
        }
    }
    
    mbi.var = var;
}


/*
  frame picture motion estimation

  org: top left pel of source reference frame
  ref: top left pel of reconstructed reference frame
  mb:  macroblock to be matched
  i,j: location of mb relative to ref (=center of search window)
  sx,sy: half widths of search window
  iminp,jminp,dframep: location and value of best frame prediction
  imintp,jmintp,tselp: location of best field pred. for top field of mb
  iminbp,jminbp,bselp: location of best field pred. for bottom field of mb
  dfieldp: value of field prediction
*/

fn mpeg2_frame_estimate(
    org: &[u8],
    rf: &[u8],
    mb: &[u8],
    i: usize, j: usize,
    sx: i32, sy: i32,
    imin: &mut i32, jmin: &mut i32,
    imint: &mut i32, jmint: &mut i32,
    iminb: &mut i32, jminb: &mut i32,
    dframe: &mut i32,
    dfield: &mut i32,
    tsel: &mut i32, bsel: &mut i32,
    imins: &mut [[i32; 2]; 2], jmins: &mut [[i32; 2]; 2],
    // unsigned char *org, unsigned char *ref,
    //                        unsigned char *mb, int i, int j, int sx, int sy,
    //                        int *iminp, int *jminp, int *imintp, int *jmintp,
    //                        int *iminbp, int *jminbp, int *dframep, int *dfieldp,
    //                        int *tselp, int *bselp, int imins[ 2 ][ 2 ],
    //                        int jmins[ 2 ][ 2 ]
) {
  // int dt, db, dmint, dminb;
  // int imint, iminb, jmint, jminb;

    let mut imint_tmp = 0;
    let mut iminb_tmp = 0;
    let mut jmint_tmp = 0;
    let mut jminb_tmp = 0;

    let width = MPEG2_WIDTH as i32;
    let height = MPEG2_HEIGHT as i32;

    /* frame prediction */
    *dframe =
      mpeg2_fullsearch(
          org, rf, mb, width, i, j, sx, sy, 16, width, height, imin, jmin
      );

    /* predict top field from top field */
    let dt =
      mpeg2_fullsearch(
        org, rf, mb, width << 1, i, j>>1, sx, sy >> 1, 8, width,
        height >> 1, &mut imint_tmp, &mut jmint_tmp );

    /* predict top field from bottom field */
    let db =
      mpeg2_fullsearch(
        &org[MPEG2_WIDTH..], &rf[MPEG2_WIDTH..], mb, width << 1, i, j>>1, sx,
        sy >> 1, 8, width, height >> 1, &mut iminb_tmp, &mut jminb_tmp );

    imins[ 0 ][ 0 ] = imint_tmp;
    jmins[ 0 ][ 0 ] = jmint_tmp;
    imins[ 1 ][ 0 ] = iminb_tmp;
    jmins[ 1 ][ 0 ] = jminb_tmp;

    let dmint: i32;
    /* select prediction for top field */
    if dt <= db {
      dmint = dt;
      *imint = imint_tmp;
      *jmint = jmint_tmp;
      *tsel = 0;
    } else {
      dmint = db;
      *imint = iminb_tmp;
      *jmint = jminb_tmp;
      *tsel = 1;
    }

    /* predict bottom field from top field */
    let dt =
      mpeg2_fullsearch(
        org, rf, &mb[MPEG2_WIDTH..], width << 1, i, j>>1, sx, sy >> 1, 8,
        width, height >> 1, &mut imint_tmp, &mut jmint_tmp,
      );

    /* predict bottom field from bottom field */
    let db =
      mpeg2_fullsearch(
        &org[MPEG2_WIDTH..], &rf[MPEG2_WIDTH..], &mb[MPEG2_WIDTH..], width << 1,
        i, j >> 1, sx, sy >> 1, 8, width, height >> 1, &mut iminb_tmp, &mut jminb_tmp
      );

    imins[0][1] = imint_tmp;
    jmins[0][1] = jmint_tmp;
    imins[1][1] = iminb_tmp;
    jmins[1][1] = jminb_tmp;

    let dminb: i32;
    /* select prediction for bottom field */
    if db <= dt {
      dminb = db;
      *iminb = iminb_tmp;
      *jminb = jminb_tmp;
      *bsel = 1;
    } else {
      dminb = dt;
      *iminb = imint_tmp;
      *jminb = jmint_tmp;
      *bsel = 0;
    }

    *dfield = dmint + dminb;
}


/*
  field picture motion estimation subroutine

  toporg: address of original top reference field
  topref: address of reconstructed top reference field
  botorg: address of original bottom reference field
  botref: address of reconstructed bottom reference field
  mb:  macroblock to be matched
  i,j: location of mb (=center of search window)
  sx,sy: half width/height of search window

  iminp,jminp,selp,dfieldp: location and distance of best field prediction
  imin8up,jmin8up,sel8up: location of best 16x8 pred. for upper half of mb
  imin8lp,jmin8lp,sel8lp: location of best 16x8 pred. for lower half of mb
  d8p: distance of best 16x8 prediction
  iminsp,jminsp,dsp: location and distance of best same parity field
                     prediction (needed for dual prime, only valid if
                     ipflag==0)
*/
fn mpeg2_field_estimate(
    toporg: &[u8],
    topref: &[u8],
    botorg: &[u8],
    botref: &[u8],
    mb: &[u8],
    i: usize,
    j: usize,
    sx: i32,
    sy: i32,
    ipflag: i32,
    iminp: &mut i32,
    jminp: &mut i32,
    imin8up: &mut i32,
    jmin8up: &mut i32,
    imin8lp: &mut i32,
    jmin8lp: &mut i32,
    dfieldp: &mut i32,
    d8p: &mut i32,
    selp: &mut i32,
    sel8up: &mut i32,
    sel8lp: &mut i32,
    iminsp: &mut i32,
    jminsp: &mut i32,
    dsp: &mut i32,
) {
  // int dt, db, imint, jmint, iminb, jminb, notop, nobot;

    /* if ipflag is set, predict from field of opposite parity only */
    let notop = ipflag != 0 && ( MPEG2_PICT_STRUCT == 1 );
    let nobot = ipflag != 0 && ( MPEG2_PICT_STRUCT == 2 );

    let mut imint: i32 = 0;
    let mut iminb: i32 = 0;
    let mut jmint: i32 = 0;
    let mut jminb: i32 = 0;

    let mut dt: i32;
    let mut db: i32;

    let width = MPEG2_WIDTH as i32;
    let height = MPEG2_HEIGHT as i32;

    /* field prediction */

    /* predict current field from top field */
    if notop {
        dt = 65536; /* infinity */
    } else {
        dt =
        mpeg2_fullsearch(
          toporg, topref, mb, width << 1, i, j, sx, sy>>1, 16, width,
          height >> 1, &mut imint, &mut jmint,
        );
    }

    /* predict current field from bottom field */
    if nobot {
      db = 65536; /* infinity */
    } else {
      db =
        mpeg2_fullsearch(
          botorg, botref, mb, width << 1, i, j, sx, sy>>1, 16, width,
          height >> 1, &mut iminb, &mut jminb,
        );
    }

    /* same parity prediction (only valid if ipflag==0) */
    if MPEG2_PICT_STRUCT == 1 {
      *iminsp = imint;
      *jminsp = jmint;
      *dsp = dt;
    } else {
      *iminsp = iminb;
      *jminsp = jminb;
      *dsp = db;
    }

    /* select field prediction */
    if dt <= db {
      *dfieldp = dt;
      *iminp = imint;
      *jminp = jmint;
      *selp = 0;
    } else {
      *dfieldp = db;
      *iminp = iminb;
      *jminp = jminb;
      *selp = 1;
    }

    /* 16x8 motion compensation */

    /* predict upper half field from top field */
    if notop {
        dt = 65536;
    } else {
        dt =
        mpeg2_fullsearch(
          toporg, topref, mb, width << 1, i, j, sx, sy>>1, 8, width,
          height >> 1, &mut imint, &mut jmint,
        );
    }

    /* predict upper half field from bottom field */
    if nobot {
        db = 65536;
    } else {
        db =
        mpeg2_fullsearch(
          botorg, botref, mb, width << 1, i, j, sx, sy>>1, 8, width,
          height >> 1, &mut iminb, &mut jminb,
        );
    }

    /* select prediction for upper half field */
    if dt <= db {
        *d8p = dt;
        *imin8up = imint;
        *jmin8up = jmint;
        *sel8up = 0;
    } else {
        *d8p = db;
        *imin8up = iminb;
        *jmin8up = jminb;
        *sel8up = 1;
    }

    /* predict lower half field from top field */
    if notop {
        dt = 65536;
    } else {
        dt =
        mpeg2_fullsearch(
          toporg, topref, &mb[( MPEG2_WIDTH << 4 )..], width << 1, i, j + 8,
          sx, sy >> 1, 8, width, height >> 1, &mut imint, &mut jmint,
        );
    }

    /* predict lower half field from bottom field */
    if nobot {
        db = 65536;
    } else {
        db =
        mpeg2_fullsearch(
          botorg, botref, &mb[(MPEG2_WIDTH << 4)..], width << 1, i, j + 8,
          sx, sy >> 1, 8, width, height >> 1, &mut iminb, &mut jminb,
        );
    }

    /* select prediction for lower half field */
    if dt <= db {
      *d8p += dt;
      *imin8lp = imint;
      *jmin8lp = jmint;
      *sel8lp = 0;
    } else {
      *d8p += db;
      *imin8lp = iminb;
      *jmin8lp = jminb;
      *sel8lp = 1;
    }
}


fn mpeg2_dpframe_estimate(
    rf: &[u8],
    mb: &[u8],
    i: usize, j: usize,
    iminf: [[i32; 2]; 2], jminf: [[i32; 2]; 2],
    iminp: &mut i32, jminp: &mut i32,
    imindmvp: &mut i32, jmindmvp: &mut i32,
    dmcp: &mut i32, vmcp: &mut i32,
) { 
    let mut is: i32;
    let mut it: i32;
    let mut ib: i32;
    let mut it0: i32;
    let mut ib0: i32;
    let mut imins: i32 = 0;
    let mut imint: i32 = 0;
    let mut iminb: i32 = 0;
    let mut imindmv: i32 = 0;

    let mut js: i32;
    let mut jt: i32;
    let mut jb: i32;
    let mut jt0: i32;
    let mut jb0: i32;
    let mut jmins: i32 = 0;
    let mut jmint: i32 = 0;
    let mut jminb: i32 = 0;
    let mut jmindmv: i32 = 0;

    let mut local_dist: i32;

    let width = MPEG2_WIDTH as i32;
    let height = MPEG2_HEIGHT as i32;

    /*
      Calculate Dual Prime distortions for 9 delta candidates
      for each of the four minimum field vectors
      Note: only for P pictures!
    */

    /* initialize minimum dual prime distortion to large value */
    let mut vmc = 1 << 30;

    // loopbound min 2 max 2
    for pref in 0..2 {
      // loopbound min 2 max 2
      for ppred in 0..2 {
        /*
          convert Cartesian absolute to relative motion vector
          values (wrt current macroblock address (i,j)
        */
        is = iminf[ pref ][ ppred ] - (i << 1) as i32;
        js = jminf[ pref ][ ppred ] - (j << 1) as i32;

        if pref != ppred {
          /* vertical field shift adjustment */
          if ppred == 0 {
              js += 1;
          } else {
              js -= 1;
          }

          /* mvxs and mvys scaling*/
          is <<= 1;
          js <<= 1;
          if MPEG2_TOPFIRST == ppred as i32 {
            /* second field: scale by 1/3 */
            is = if is >= 0 { ( is + 1 ) / 3 } else { -( ( -is + 1 ) / 3 ) };
            js = if js >= 0 { ( js + 1 ) / 3 } else { -( ( -js + 1 ) / 3 ) };
          } else {
            continue;
          }
        }

        /* vector for prediction from field of opposite 'parity' */
        if MPEG2_TOPFIRST != 0 {
          /* vector for prediction of top field from bottom field */
          it0 = (is + (is > 0) as i32) >> 1;
          jt0 = ((js + (js > 0) as i32) >> 1) - 1;

          /* vector for prediction of bottom field from top field */
          ib0 = (3 * is + (is > 0) as i32) >> 1;
          jb0 = ((3 * js + (js > 0) as i32) >> 1) + 1;
        } else {
          /* vector for prediction of top field from bottom field */
          it0 = (3 * is + (is > 0) as i32) >> 1;
          jt0 = ((3 * js + (js > 0) as i32) >> 1) - 1;

          /* vector for prediction of bottom field from top field */
          ib0 = (is + (is > 0) as i32) >> 1;
          jb0 = ((js + (js > 0) as i32) >> 1) + 1;
        }

        /* convert back to absolute half-pel field picture coordinates */
        is += (i << 1) as i32;
        js += (j << 1) as i32;
        it0 += (i << 1) as i32;
        jt0 += (j << 1) as i32;
        ib0 += (i << 1) as i32;
        jb0 += (j << 1) as i32;

        if (is >= 0) && (is <= (width - 16) << 1) &&
            (js >= 0) && (js <= (height - 16)) {
          // loopbound min 3 max 3
          for delta_y in -1..=1 {
            // loopbound min 3 max 3
            for delta_x in -1..=1 {
              /* opposite field coordinates */
              it = it0 + delta_x;
              jt = jt0 + delta_y;
              ib = ib0 + delta_x;
              jb = jb0 + delta_y;

              if (it >= 0) && (it <= (width - 16) << 1) &&
                  (jt >= 0) && (jt <= (height - 16)) &&
                  (ib >= 0) && (ib <= (width - 16) << 1) &&
                  (jb >= 0) && (jb <= (height - 16)) {
                /* compute prediction error */
                let index1 = ( is >> 1 ) as usize + ( MPEG2_WIDTH << 1 ) * ( js >> 1 ) as usize;
                let index2 = MPEG2_WIDTH + ( it >> 1 ) as usize + ( MPEG2_WIDTH << 1 ) * ( jt >> 1 ) as usize;
                local_dist =
                  mpeg2_bdist2(
                    &rf[index1..],
                    &rf[index2..],
                    mb,             /* current mb location */
                    MPEG2_WIDTH << 1,     /* adjacent line distance */
                    is & 1, js & 1, it & 1, jt & 1, /* half-pel flags */
                    8 );            /* block height */
                let index1 = MPEG2_WIDTH + ( is >> 1 ) as usize + ( MPEG2_WIDTH << 1 ) * ( js >> 1 ) as usize;
                let index2  = ( ib >> 1 ) as usize + ( MPEG2_WIDTH << 1 ) * ( jb >> 1 ) as usize;
                local_dist +=
                  mpeg2_bdist2(
                    &rf[index1..],
                    &rf[index2..],
                    &mb[MPEG2_WIDTH..],     /* current mb location */
                    MPEG2_WIDTH << 1,     /* adjacent line distance */
                    is & 1, js & 1, ib & 1, jb & 1, /* half-pel flags */
                    8 );            /* block height */

                /* update delta with least distortion vector */
                if local_dist < vmc {
                  imins = is;
                  jmins = js;
                  imint = it;
                  jmint = jt;
                  iminb = ib;
                  jminb = jb;
                  imindmv = delta_x;
                  jmindmv = delta_y;
                  vmc = local_dist;
                }
              }
            }  /* end delta x loop */
          } /* end delta y loop */
        }
      }
    }

    /* Compute L1 error for decision purposes */
    let index1 = ( imins >> 1 ) as usize + ( MPEG2_WIDTH << 1 ) * ( jmins >> 1 ) as usize;
    let index2 = MPEG2_WIDTH + ( imint >> 1 ) as usize + ( MPEG2_WIDTH << 1 ) * ( jmint >> 1 ) as usize;
    local_dist =
      mpeg2_bdist1(
        &rf[index1..],
        &rf[index2..],
        mb,
        MPEG2_WIDTH << 1, imins & 1, jmins & 1, imint & 1, jmint & 1, 8
      );
    let index1 = MPEG2_WIDTH + ( imins >> 1 ) as usize + ( MPEG2_WIDTH << 1 ) * ( jmins >> 1 ) as usize;
    let index2 = ( iminb >> 1 ) as usize + ( MPEG2_WIDTH << 1 ) * ( jminb >> 1 ) as usize;
    local_dist +=
      mpeg2_bdist1(
        &rf[index1..],
        &rf[index2..],
        &mb[MPEG2_WIDTH..], MPEG2_WIDTH << 1, imins & 1, jmins & 1, iminb & 1,
        jminb & 1, 8 );

    *dmcp = local_dist;
    *iminp = imins;
    *jminp = jmins;
    *imindmvp = imindmv;
    *jmindmvp = jmindmv;
    *vmcp = vmc;
}

fn mpeg2_dpfield_estimate(
    topref: &[u8],
    botref: &[u8],
    mb: &[u8],
    i: i32, j: i32,
    imins: i32, jmins: i32,
    imindmvp: &mut i32, jmindmvp: &mut i32,
    dmcp: &mut i32, vmcp: &mut i32,
) {
    let mut sameref: &[u8];
    let mut sameref_i = 0;
    let mut oppref: &[u8];
    let mut oppref_i = 0;

    let io0: i32;
    let mut io: i32;
    let mut imino: i32 = 0;
    let mut imindmv: i32 = 0;

    let jo0: i32;
    let mut jo: i32;
    let mut jmino: i32 = 0;
    let mut jmindmv: i32 = 0;

    let width = MPEG2_WIDTH as i32;
    let height2 = MPEG2_HEIGHT2 as i32;

    /* Calculate Dual Prime distortions for 9 delta candidates */
    /* Note: only for P pictures! */

    /* Assign opposite and same reference pointer */
    if MPEG2_PICT_STRUCT == 1 {
        sameref = topref;
        oppref = botref;
    } else {
        sameref = botref;
        oppref = topref;
    }

    /*
      convert Cartesian absolute to relative motion vector
      values (wrt current macroblock address (i,j)
    */
    let mvxs = imins - (i << 1);
    let mvys = jmins - (j << 1);

    /* vector for prediction from field of opposite 'parity' */
    let mvxo0 = ( mvxs + ( mvxs > 0 ) as i32 ) >> 1; /* mvxs // 2 */
    let mut mvyo0 = ( mvys + ( mvys > 0 ) as i32 ) >> 1; /* mvys // 2 */

    /* vertical field shift correction */
    if MPEG2_PICT_STRUCT == 1 {
      mvyo0 -= 1;
    } else {
      mvyo0 += 1;
    }

    /* convert back to absolute coordinates */
    io0 = mvxo0 + (i << 1);
    jo0 = mvyo0 + (j << 1);

    /* initialize minimum dual prime distortion to large value */
    let mut vmc_dp = 1 << 30;

    // loopbound min 3 max 3
    for delta_y in -1..=1 {
      // loopbound min 3 max 3
      for delta_x in -1..1 {
        /* opposite field coordinates */
        io = io0 + delta_x;
        jo = jo0 + delta_y;

        if (io >= 0) && (io <= (width - 16) << 1) &&
            (jo >= 0) && (jo <= (height2 - 16) << 1) {
          /* compute prediction error */
          let index1 = (imins >> 1) as usize + MPEG2_WIDTH2 * (jmins >> 1) as usize;
          let index2 = (io >> 1) as usize + MPEG2_WIDTH2 * (jo >> 1) as usize;
          let local_dist =
            mpeg2_bdist2(
              &sameref[index1..],
              &oppref[index2..],
              mb,            /* current mb location */
              MPEG2_WIDTH2,         /* adjacent line distance */
              imins & 1, jmins & 1, io & 1, jo & 1, /* half-pel flags */
              16
            );           /* block height */
          sameref = &sameref[sameref_i..];
          sameref_i = 0;
          oppref = &oppref[oppref_i..];
          oppref_i = 0;

          /* update delta with least distortion vector */
          if local_dist < vmc_dp {
            imino = io;
            jmino = jo;
            imindmv = delta_x;
            jmindmv = delta_y;
            vmc_dp = local_dist;
          }
        }
      }  /* end delta x loop */
    } /* end delta y loop */

    /* Compute L1 error for decision purposes */
    let index1 = ( imins >> 1 ) as usize + MPEG2_WIDTH2 * ( jmins >> 1 ) as usize;
    let index2 = ( imino >> 1 ) as usize + MPEG2_WIDTH2 * ( jmino >> 1 ) as usize;
    *dmcp =
      mpeg2_bdist1(
        &sameref[index1..],
        &oppref[index2..],
        mb,            /* current mb location */
        MPEG2_WIDTH2,         /* adjacent line distance */
        imins & 1, jmins & 1, imino & 1, jmino & 1, /* half-pel flags */
        16 );           /* block height */

    *imindmvp = imindmv;
    *jmindmvp = jmindmv;
    *vmcp = vmc_dp;
}


/*
  full search block matching

  blk: top left pel of (16*h) block
  h: height of block
  lx: distance (in bytes) of vertically adjacent pels in ref,blk
  org: top left pel of source reference picture
  ref: top left pel of reconstructed reference picture
  i0,j0: center of search window
  sx,sy: half widths of search window
  xmax,ymax: right/bottom limits of search area
  iminp,jminp: pointers to where the result is stored
               result is given as half pel offset from ref(0,0)
               i.e. NOT relative to (i0,j0)
*/

fn mpeg2_fullsearch(
    org: &[u8],
    rf: &[u8],
    blk: &[u8],
    lx: i32,
    i0: usize, j0: usize,
    sx: i32, sy: i32,
    h: i32,
    xmax: i32, ymax: i32,
    iminp: &mut i32, jminp: &mut i32,  
) -> i32 {
    let mut ilow = i0 as i32 - sx;
    let mut ihigh = i0 as i32 + sx;

    if ilow < 0 {
      ilow = 0;
    }

    if ihigh > xmax - 16 {
      ihigh = xmax - 16;
    }

    let mut jlow = j0 as i32 - sy;
    let mut jhigh = j0 as i32 + sy;

    if jlow < 0 {
      jlow = 0;
    }

    if jhigh > (ymax - h) {
      jhigh = ymax - h;
    }

    /* full pel search, spiraling outwards */

    let mut imin = i0 as i32;
    let mut jmin = j0 as i32;
    let index = (imin + lx * jmin) as usize;
    let mut dmin = mpeg2_dist1( &org[index..], blk, lx, 0, 0, h, 65536 );

    let sxy = core::cmp::max(sx, sy);

    // loopbound min 3 max 7
    for l in 1..=sxy {
      let mut i = i0 as i32 - l;
      let mut j = j0 as i32 - l;
      // loopbound min 8 max 56
      for k in 0..(8 * l) {
        if (i as i32 >= ilow) && (i as i32 <= ihigh) && (j as i32 >= jlow) &&
            (j as i32 <= jhigh) {
          let index = (i + lx * j) as usize;
          let d = mpeg2_dist1( &org[index..], blk, lx, 0, 0, h, dmin );

          if d < dmin {
            dmin = d;
            imin = i;
            jmin = j;
          }
        }

        if k < 2 * l {
          i += 1;
        } else if k < 4 * l {
          j += 1;
        } else if k < 6 * l {
          i -= 1;
        } else {
          j -= 1;
        }
      }
    }

    /* half pel */
    dmin = 65536;
    let mut imin = imin as i32;
    let mut jmin = jmin as i32;
    imin <<= 1;
    jmin <<= 1;
    ilow = imin - if imin > 0 { 1 } else { 0 };
    ihigh = imin + if imin < ((xmax - 16) << 1) { 1 } else { 0 };
    jlow = jmin - if jmin > 0 { 1 } else { 0 };
    jhigh = jmin + if jmin < ((ymax - h) << 1) { 1 } else { 0 };

    // loopbound min 2 max 3
    for j in jlow..=jhigh {
      // loopbound min 2 max 3
      for i in ilow..=ihigh {
        let index = (( i >> 1 ) + lx * ( j >> 1 )) as usize;
        let d =
          mpeg2_dist1(
            &rf[index..], blk, lx, i as i32 & 1, j as i32 & 1, h, dmin );

        if d < dmin {
          dmin = d;
          imin = i;
          jmin = j;
        }
      }
    }

    *iminp = imin as i32;
    *jminp = jmin as i32;

    dmin
}

/*
  total absolute difference between two (16*h) blocks
  including optional half pel interpolation of blk1 (hx,hy)
  blk1,blk2: addresses of top left pels of both blocks
  lx:        distance (in bytes) of vertically adjacent pels
  hx,hy:     flags for horizontal and/or vertical interpolation
  h:         height of block (usually 8 or 16)
  distlim:   bail out if sum exceeds this value
*/
fn mpeg2_dist1(
    blk1: &[u8],
    blk2: &[u8],
    lx: i32,
    hx: i32,
    hy: i32,
    h: i32,
    distlim: i32,
) -> i32
{
    let mut s: i32 = 0;
    let mut blk_i: usize = 0;
    if hx == 0 && hy == 0 {
        // loopbound min 0 max 16
        for _ in 0..h {
            for i in 0..16 {
                let mut v: i32 = blk1[blk_i + i] as i32 - blk2[blk_i + i] as i32;
                if v < 0 {
                    v = -v;
                }
                s += v;
            }
            if s >= distlim {
                break;
            }
            blk_i += lx as usize;
        }
    } else if hx != 0 && hy == 0 {
        // loopbound min 8 max 16
        for _ in 0..h {
            // loopbound min 16 max 16
            for i in 0..16 {
                let v = ((blk1[blk_i + i] as i32 + blk1[blk_i + i + 1] as i32 + 1) >> 1) - blk2[blk_i + i] as i32;
                if v >= 0 {
                    s += v;
                } else {
                    s -= v;
                }
            
              
            }
            blk_i += lx as usize;
        }
    } else if hx == 0 && hy != 0 {
        let mut blk1_i = blk_i;
        let mut blk1a_i = blk1_i + lx as usize;
        let mut blk2_i = blk_i;
        // loopbound min 8 max 16
        for _ in 0..h {
            // loopbound min 16 max 16
            for i in 0..16 {
                let v = ((blk1[blk1_i + i] as i32 + blk1[blk1a_i + i] as i32 + 1) >> 1) - blk2[blk2_i + i] as i32;
                if v >= 0 {
                    s += v;
                } else {
                    s -= v;
                }
            }
            blk1_i = blk1a_i;
            blk1a_i += lx as usize;
            blk2_i += lx as usize;
        }
    } else { /* if (hx && hy) */
        let mut blk1_i = blk_i;
        let mut blk1a_i = blk1_i + lx as usize;
        let mut blk2_i = blk_i;
        // loopbound min 8 max 16
        for _ in 0..h {
            // loopbound min 16 max 16
            for i in 0..16 {
                let v = ((blk1[blk1_i + i] as i32 + blk1[blk1_i + i + 1] as i32
                    + blk1[blk1a_i + i] as i32 + blk1[blk1a_i + i + 1] as i32 + 2) >> 2)
                    - blk2[blk2_i + i] as i32;
                if v >= 0 {
                    s += v;
                } else {
                    s -= v;
                }
            }
            blk1_i = blk1a_i;
            blk1a_i += lx as usize;
            blk2_i += lx as usize;
        }
    }
    s
}

/*
  total squared difference between two (16*h) blocks
  including optional half pel interpolation of blk1 (hx,hy)
  blk1,blk2: addresses of top left pels of both blocks
  lx:        distance (in bytes) of vertically adjacent pels
  hx,hy:     flags for horizontal and/or vertical interpolation
  h:         height of block (usually 8 or 16)
*/
fn mpeg2_dist2(
    blk1: &[u8],
    blk2: &[u8],
    lx: usize,
    hx: i32, hy: i32,
    h: i32,
) -> i32 {
    let mut s: i32 = 0;
    let mut v: i32;

    let mut blk1_i = 0;
    let mut blk1a_i = 0;
    let mut blk2_i = 0;

    if hx == 0 && hy == 0 {
        // loopbound min 8 max 16
        for _ in 0..h {
            // loopbound min 16 max 16
            for i in 0..16 {
                v = blk1[blk1_i + i] as i32 - blk2[blk2_i + i] as i32;
                s += v * v;
            }
            blk1_i += lx;
            blk2_i += lx;
        }
    } else if hx != 0 && hy == 0{
        // loopbound min 8 max 16
        for _ in 0..h {
            // loopbound min 16 max 16
            for i in 0..16 {
                v = ( ( blk1[blk1_i + i] as i32 + blk1[blk1_i + i + 1] as i32 + 1 ) >> 1 ) - blk2[blk2_i + i] as i32;
                s += v * v;
            }
            blk1_i += lx;
            blk2_i += lx;
        }
    } else if hx == 0 && hy != 0 {
        blk1a_i = blk1a_i + lx;
        // loopbound min 8 max 16
        for _ in 0..h {
          // loopbound min 16 max 16
          for i in 0..16 {
            v = ( ( blk1[blk1_i + i] as i32 + blk1[blk1a_i + i] as i32 + 1 ) >> 1 ) - blk2[blk2_i + i] as i32;
            s += v * v;
          }
          blk1_i = blk1a_i;
          blk1a_i += lx;
          blk2_i += lx;
        }
    } else { /* if (hx && hy) */
        blk1a_i = blk1a_i + lx;
        // loopbound min 8 max 16
        for _ in 0..h {
            // loopbound min 16 max 16
            for i in 0..16 {
                v =
                  (( blk1[blk1_i + i] as i32 + blk1[blk1_i + i + 1] as i32
                    + blk1[blk1a_i + i] as i32 + blk1[blk1a_i + i + 1] as i32 + 2 ) >> 2 ) - blk2[blk2_i + i] as i32;
                s += v * v;
            }
            blk1_i = blk1a_i;
            blk1a_i += lx;
            blk2_i += lx;
        }
    }
    s
}


/*
  absolute difference error between a (16*h) block and a bidirectional
  prediction

  p2: address of top left pel of block
  pf,hxf,hyf: address and half pel flags of forward ref. block
  pb,hxb,hyb: address and half pel flags of backward ref. block
  h: height of block
  lx: distance (in bytes) of vertically adjacent pels in p2,pf,pb
*/
fn mpeg2_bdist1(
    pf: &[u8],
    pb: &[u8],
    p2: &[u8],
    lx: usize,
    hxf: i32, hyf: i32,
    hxb: i32, hyb: i32,
    h: i32,
) -> i32 {
    let mut pf_i = 0;
    let mut pb_i = 0;
    let mut p2_i = 0;

    let mut pfa_i = hxf as usize;
    let mut pfb_i = lx * hyf as usize;
    let mut pfc_i = pfb_i + hxf as usize;

    let mut pba_i = hxb as usize;
    let mut pbb_i = lx * hyb as usize;
    let mut pbc_i = pbb_i + hxb as usize;

    let mut s = 0;

    // loopbound min 8 max 16
    for _ in 0..h {
      // loopbound min 16 max 16
      for _ in 0..16 {
        let f = (pf[pf_i] as u32 + pf[pfa_i] as u32 + pf[pfb_i] as u32 + pf[pfc_i] as u32 + 2) >> 2;
        let b = (pb[pb_i] as u32 + pb[pba_i] as u32 + pb[pbb_i] as u32 + pb[pbc_i] as u32 + 2) >> 2;
        let v = ((f + b + 1) >> 1) as i32 - p2[p2_i] as i32;      
        s += v.abs();

        pf_i += 1;
        pb_i += 1;
        p2_i += 1;
        pfa_i += 1;
        pfb_i += 1;
        pfc_i += 1;
        pba_i += 1;
        pbb_i += 1;
        pbc_i += 1;
      }
      p2_i += lx - 16;
      pf_i += lx - 16;
      pb_i += lx - 16;
      pfa_i = lx - 16;
      pfb_i = lx - 16;
      pfc_i = lx - 16;
      pba_i = lx - 16;
      pbb_i = lx - 16;
      pbc_i = lx - 16;
    }
    s
}

/*
  squared error between a (16*h) block and a bidirectional
  prediction

  p2: address of top left pel of block
  pf,hxf,hyf: address and half pel flags of forward ref. block
  pb,hxb,hyb: address and half pel flags of backward ref. block
  h: height of block
  lx: distance (in bytes) of vertically adjacent pels in p2,pf,pb
*/
fn mpeg2_bdist2(
    pf: &[u8],
    pb: &[u8],
    p2: &[u8],
    lx: usize,
    hxf: i32, hyf: i32,
    hxb: i32, hyb: i32,
    h: i32,
) -> i32 {
    let mut pfa = &pf[(hxf as usize)..];
    let mut pfb = &pf[(lx * hyf as usize )..];
    let mut pfc = &pfb[(hxf as usize)..];

    let mut pba = &pb[(hxb as usize)..];
    let mut pbb = &pb[(lx * hyb as usize)..];
    let mut pbc = &pbb[(hxb as usize)..];

    let mut pf_i = 0;
    let mut pb_i = 0;
    let mut p2_i = 0;

    let mut s: i32 = 0;

    // loopbound min 8 max 16
    for _ in 0..h {
      // loopbound min 16 max 16
      for _ in 0..16 {
        let v =
          ((((pf[pf_i] + pfa[0] + pfb[0] + pfc[0] + 2) >> 2) +
              ((pb[pb_i] + pba[0] + pbb[0] + pbc[0] + 2) >> 2) +
              1) >> 1) - p2[p2_i];
        pf_i += 1;
        pb_i += 1;
        p2_i += 1;
        pfa = &pfa[1..];
        pfb = &pfb[1..];
        pfc = &pfc[1..];
        pba = &pba[1..];
        pbb = &pbb[1..];
        pbc = &pbc[1..];
        
        s += v as i32 * v as i32;
      }
      pf_i += lx - 16;
      pb_i += lx - 16;
      p2_i += lx - 16;
      pfa = &pfa[(lx - 16)..];
      pfb = &pfb[(lx - 16)..];
      pfc = &pfc[(lx - 16)..];
      pba = &pba[(lx - 16)..];
      pbb = &pbb[(lx - 16)..];
      pbc = &pbc[(lx - 16)..];
    }
    s as i32
}


/*
  variance of a (16*16) block, multiplied by 256
  p:  address of top left pel of block
  lx: distance (in bytes) of vertically adjacent pels
*/
fn mpeg2_variance(p: &[u8], lx: usize) -> i32 {
    let mut s: u32 = 0;
    let mut s2: u32 = 0;

    let mut p_i: usize = 0;
    // loopbound min 16 max 16
    for _ in 0..16 {
        // loopbound min 16 max 16
        for _ in 0..16 {
            let v = p[p_i];
            p_i += 1;
            s += v as u32;
            s2 += v as u32 * v as u32;
        }
        p_i += lx - 16;
    }
    (s2 - (s * s) / 256) as i32
}

/*
  Main functions
*/

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn mpeg2_main(
    mbis: &mut [Mbinfo; 88],
    orgframe: &mut [u8; 22528],
) {
  mpeg2_motion_estimation(
    orgframe, orgframe, orgframe, orgframe,
    orgframe, orgframe, 7, 7, 3, 3, mbis, 0, 0
  );
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn mpeg2_log(
    mbis: &mut [Mbinfo; 88],
    orgframe: &mut [u8; 22528],
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        mpeg2_init(
          black_box(mbis), black_box(orgframe)
        );
        let start = black_box(DWT::cycle_count());
        mpeg2_main(black_box(mbis), black_box(orgframe));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("mpeg2 - elapsed: {}", cycles);
    });

    black_box(mpeg2_return(black_box(mbis))) - (-18) == 0
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn mpeg2_std(
    mbis: &mut [Mbinfo; 88],
    orgframe: &mut [u8; 22528],
) -> bool {
    mpeg2_init(
      black_box(mbis), black_box(orgframe),
    );
    mpeg2_main(black_box(mbis), black_box(orgframe));
    black_box(mpeg2_return(black_box(mbis))) - (-18) == 0
}

fn mpeg2() -> bool {
    static mut MBIS: [Mbinfo; 88] = [Mbinfo::new(); 88];
    static mut ORGFRAME: [u8; 22528] = [0; 22528];
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let mbis = unsafe { &mut *{ &raw mut MBIS } };
    let orgframe = unsafe { &mut *{ &raw mut ORGFRAME } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(mpeg2_log(
            black_box(mbis), black_box(orgframe)
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(mpeg2_std(
            black_box(mbis), black_box(orgframe)
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(mpeg2());
    
    #[cfg(feature = "log")]
    {
        rprintln!("mpeg2: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(mpeg2());

    #[cfg(feature = "log")]
    std::println!("mpeg2: {}", success);

    std::process::exit(!success as i32);
}
