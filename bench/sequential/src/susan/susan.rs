/**********************************************************************\

  SUSAN Version 2l by Stephen Smith
  Oxford Centre for Functional Magnetic Resonance Imaging of the Brain,
  Department of Clinical Neurology, Oxford University, Oxford, UK
  (Previously in Computer Vision and Image Processing Group - now
  Computer Vision and Electro Optics Group - DERA Chertsey, UK)
  Email:    steve@fmrib.ox.ac.uk
  WWW:      http://www.fmrib.ox.ac.uk/~steve

  (C) Crown Copyright (1995-1999), Defence Evaluation and Research Agency,
  Farnborough, Hampshire, GU14 6TD, UK
  DERA WWW site:
  http://www.dera.gov.uk/
  DERA Computer Vision and Electro Optics Group WWW site:
  http://www.dera.gov.uk/imageprocessing/dera/group_home.html
  DERA Computer Vision and Electro Optics Group point of contact:
  Dr. John Savage, jtsavage@dera.gov.uk, +44 1344 633203

  A UK patent has been granted: "Method for digitally processing
  images to determine the position of edges and/or corners therein for
  guidance of unmanned vehicle", UK Patent 2272285. Proprietor:
  Secretary of State for Defence, UK. 15 January 1997

  This code is issued for research purposes only and remains the
  property of the UK Secretary of State for Defence. This code must
  not be passed on without this header information being kept
  intact. This code must not be sold.

  \**********************************************************************/

/**********************************************************************\

  SUSAN Version 2l
  SUSAN = Smallest Univalue Segment Assimilating Nucleus

  Email:    steve@fmrib.ox.ac.uk
  WWW:      http://www.fmrib.ox.ac.uk/~steve

  Related paper:
  @article{Smith97,
        author = "Smith, S.M. and Brady, J.M.",
        title = "{SUSAN} - A New Approach to Low Level Image Processing",
        journal = "Int. Journal of Computer Vision",
        pages = "45--78",
        volume = "23",
        number = "1",
        month = "May",
        year = 1997}

  To be registered for automatic (bug) updates of SUSAN, send an email.

  Compile with:
  gcc -O4 -o susan susan2l.c -lm

  See following section for different machine information. Please
  report any bugs (and fixes). There are a few optional changes that
  can be made in the "defines" section which follows shortly.

  Usage: type "susan" to get usage. Only PGM format files can be input
  and output. Utilities such as the netpbm package and XV can be used
  to convert to and from other formats. Any size of image can be
  processed.

  This code is written using an emacs folding mode, making moving
  around the different sections very easy. This is why there are
  various marks within comments and why comments are indented.


  SUSAN QUICK:

  This version of the SUSAN corner finder does not do all the
  false-corner suppression and thus is faster and produced some false
  positives, particularly on strong edges. However, because there are
  less stages involving thresholds etc., the corners that are
  correctly reported are usually more stable than those reported with
  the full algorithm. Thus I recommend at least TRYING this algorithm
  for applications where stability is important, e.g., tracking.

  THRESHOLDS:

  There are two thresholds which can be set at run-time. These are the
  brightness threshold (t) and the distance threshold (d).

  SPATIAL CONTROL: d

  In SUSAN smoothing d controls the size of the Gaussian mask; its
  default is 4.0. Increasing d gives more smoothing. In edge finding,
  a fixed flat mask is used, either 37 pixels arranged in a "circle"
  (default), or a 3 by 3 mask which gives finer detail. In corner
  finding, only the larger 37 pixel mask is used; d is not
  variable. In smoothing, the flat 3 by 3 mask can be used instead of
  a larger Gaussian mask; this gives low smoothing and fast operation.

  BRIGHTNESS CONTROL: t

  In all three algorithms, t can be varied (default=20); this is the
  main threshold to be varied. It determines the maximum difference in
  greylevels between two pixels which allows them to be considered
  part of the same "region" in the image. Thus it can be reduced to
  give more edges or corners, i.e. to be more sensitive, and vice
  versa. In smoothing, reducing t gives less smoothing, and vice
  versa. Set t=10 for the test image available from the SUSAN web
  page.

  ITERATIONS:

  With SUSAN smoothing, more smoothing can also be obtained by
  iterating the algorithm several times. This has a different effect
  from varying d or t.

  FIXED MASKS:

  37 pixel mask:    ooo       3 by 3 mask:  ooo
                   ooooo                    ooo
                  ooooooo                   ooo
                  ooooooo
                  ooooooo
                   ooooo
                    ooo

  CORNER ATTRIBUTES dx, dy and I
  (Only read this if you are interested in the C implementation or in
  using corner attributes, e.g., for corner matching)

  Corners reported in the corner list have attributes associated with
  them as well as positions. This is useful, for example, when
  attempting to match corners from one image to another, as these
  attributes can often be fairly unchanged between images. The
  attributes are dx, dy and I. I is the value of image brightness at
  the position of the corner. In the case of susan_corners_quick, dx
  and dy are the first order derivatives (differentials) of the image
  brightness in the x and y directions respectively, at the position
  of the corner. In the case of normal susan corner finding, dx and dy
  are scaled versions of the position of the centre of gravity of the
  USAN with respect to the centre pixel (nucleus).

  BRIGHTNESS FUNCTION LUT IMPLEMENTATION:
  (Only read this if you are interested in the C implementation)

  The SUSAN brightness function is implemented as a LUT
  (Look-Up-Table) for speed. The resulting pointer-based code is a
  little hard to follow, so here is a brief explanation. In
  setup_brightness_lut() the LUT is setup. This mallocs enough space
  for *bp and then repositions the pointer to the centre of the
  malloced space. The SUSAN function e^-(x^6) or e^-(x^2) is
  calculated and converted to a uchar in the range 0-100, for all
  possible image brightness differences (including negative
  ones). Thus bp[ 23 ] is the output for a brightness difference of 23
  greylevels. In the SUSAN algorithms this LUT is used as follows:

  p=in + (i-3)*x_size + j - 1;
  p points to the first image pixel in the circular mask surrounding
  point (x,y).

  cp=bp + in[ i*x_size+j ];
  cp points to a position in the LUT corresponding to the brightness
  of the centre pixel (x,y).

  now for every pixel within the mask surrounding (x,y),
  n+=*(cp-*p++);
  the brightness difference function is found by moving the cp pointer
  down by an amount equal to the value of the pixel pointed to by p,
  thus subtracting the two brightness values and performing the
  exponential function. This value is added to n, the running USAN
  area.

  in SUSAN smoothing, the variable height mask is implemented by
  multiplying the above by the moving mask pointer, reset for each new
  centre pixel.
  tmp = *dpt++ * *(cp-brightness);

  \**********************************************************************/

/**********************************************************************\

  Success has been reported with the following:

  MACHINE  OS         COMPILER

  Sun      4.1.4      bundled C, gcc

  Next

  SGI      IRIX       SGI cc

  DEC      Unix V3.2+

  IBM RISC AIX        gcc

  PC                  Borland 5.0

  PC       Linux      gcc-2.6.3

  PC       Win32      Visual C++ 4.0 (Console Application)

  PC       Win95      Visual C++ 5.0 (Console Application)
                      Thanks to Niu Yongsheng <niuysbit@163.net>:
                      Use the FOPENB option below

  PC       DOS        djgpp gnu C
                      Thanks to Mark Pettovello <mpettove@umdsun2.umd.umich.edu>:
                      Use the FOPENB option below

  HP       HP-UX      bundled cc
                      Thanks to Brian Dixon <briand@hpcvsgen.cv.hp.com>:
                      in ksh:
                      export CCOPTS="-Aa -D_HPUX_SOURCE | -lM"
                      cc -O3 -o susan susan2l.c

  \**********************************************************************/

/**********************************************************************\

  SUSAN Version 2l, 12/2/99
  Changed GNUDOS option to FOPENB.
  (Thanks to Niu Yongsheng <niuysbit@163.net>.)
  Took out redundant "sq=sq/2;".

  SUSAN Version 2k, 19/8/98:
  In corner finding:
  Changed if(yy<sq) {...} else if(xx<sq) {...} to
          if(yy<xx) {...} else {...}
  (Thanks to adq@cim.mcgill.edu - Alain Domercq.)

  SUSAN Version 2j, 22/10/97:
  Fixed (mask_size>x_size) etc. tests in smoothing.
  Added a couple of free() calls for cgx and cgy.
  (Thanks to geoffb@ucs.ed.ac.uk - Geoff Browitt.)

  SUSAN Version 2i, 21/7/97:
  Added information about corner attributes.

  SUSAN Version 2h, 16/12/96:
  Added principle (initial enhancement) option.

  SUSAN Version 2g, 2/7/96:
  Minor superficial changes to code.

  SUSAN Version 2f, 16/1/96:
  Added GNUDOS option (now called FOPENB; see options below).

  SUSAN Version 2e, 9/1/96:
  Added -b option.
  Fixed 1 pixel horizontal offset error for drawing edges.

  SUSAN Version 2d, 27/11/95:
  Fixed loading of certain PGM files in get_image (again!)

  SUSAN Version 2c, 22/11/95:
  Fixed loading of certain PGM files in get_image.
  (Thanks to qu@San-Jose.ate.slb.com - Gongyuan Qu.)

  SUSAN Version 2b, 9/11/95:
  removed "z==" error in edges routines.

  SUSAN Version 2a, 6/11/95:
  Removed a few unnecessary variable declarations.
  Added different machine information.
  Changed "header" in get_image to char.

  SUSAN Version 2, 1/11/95: first combined version able to take any
  image sizes.

  SUSAN "Versions 1", circa 1992: the various SUSAN algorithms were
  developed during my doctorate within different programs and for
  fixed image sizes. The algorithms themselves are virtually unaltered
  between "versions 1" and the combined program, version 2.

  \**********************************************************************/

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
  containing the source and license information above.

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
use input::SUSAN_INPUT;

mod file;
use file::{ SeekOrigin, File };

mod memory;
use memory::{ SimulatedHeap, memset };

use crate::memory::HeapSlice;

const EXP_A: i32 = 184;
const EXP_C: i32 = 16249;

fn susan_expf(y: f32) -> f32 {
    let i: u16 = (EXP_A * y as i32 + EXP_C) as u16;
    let j: u16 = 0u16;

    let bits = if cfg!(target_endian = "little") {
        ((j as u32) << 16) | (i as u32)
    } else {
        ((i as u32) << 16) | (j as u32)
    };
    f32::from_bits(bits)
}

fn susan_sqrtf(val: f32) -> f32 {
    if val == 0. { return 0.; }

    let mut x = val / 10.0;
    let min_tol: f32 = 0.00001f32;
    let mut flag: bool = true;

    // loopbound min 19 max 19
    for _ in 1..20 {
        if flag {
            let dx: f32 = (val - (x * x)) / (2.0 * x);
            x += dx;
            let diff: f32 = val - (x * x);
            if diff.abs() <= min_tol { flag = false; }
        }
    }
    x
}

/* ********** Optional settings */

// Lowered size to fit in .bss
// -> is not used in benchmark anyways
const MAX_CORNERS: i32 = 1000;  /* max corners per frame */

type TotalType = i32;
type Uchar = u8;
type CornerListT = [CornerRep; MAX_CORNERS as usize];

/* size for non-max corner suppression; SEVEN_SUPP or FIVE_SUPP */
const _TOGGLE_SUPPRESSION: bool = true;    // true: SEVEN_SUPP  false: FIVE_SUPP

#[derive(Default, PartialEq, Clone, Copy)]
enum DrawingMode {
    #[default] Smooth, Edges, Corners,
}

#[derive(Clone, Copy)]
struct CornerRep {
    x: i8,
    y: i8,
    info: i8,
    dx: i8,
    dy: i8,
    i: i8,
}

impl CornerRep {
    const fn new() -> Self {
        CornerRep {
            x: 0,
            y: 0,
            info: 0,
            dx: 0,
            dy: 0,
            i: 0,
        }
    }
}

#[derive(Default)]
struct Config {
    dt: f32,
    bt: i32,
    principle_conf: bool,
    thin_post_proc: bool,
    three_by_three: bool,
    drawing_mode: DrawingMode,
    quick: bool,
    max_no_corners: i32,
    max_no_edges: i32,
    mask_mode: bool,
}

impl Config {
    const fn new() -> Self {
        Config {
            dt: 0.,
            bt: 0,
            principle_conf: false,
            thin_post_proc: false,
            three_by_three: false,
            drawing_mode: DrawingMode::Corners,
            quick: false,
            max_no_corners: 0,
            max_no_edges: 0,
            mask_mode: false,
        }
    }
}

struct Data {
    inp: HeapSlice,
    bp: HeapSlice,
    mid: HeapSlice,
    x_size: u16,
    y_size: u16,
    r: HeapSlice,
}

impl Data {
    const fn new() -> Self {
        Data {
            inp: HeapSlice::new(),
            bp: HeapSlice::new(),
            mid: HeapSlice::new(),
            x_size: 0,
            y_size: 0,
            r: HeapSlice::new(),
        }
    }
}

fn susan_getint(fd: &mut File) -> i32 {
    let mut dummy: [i8; 10000] = [0; 10000];

    let tmp = fd.getc().map_or_else(|_| { -1 }, |v| { v as i32 });
    let mut c: u8 = tmp as u8;
    // loopbound min 0 max 3
    loop { /* find next integer */
        if c == b'#' {   /* if we're at a comment, read to end of line */
            _ = fd.read_line(&mut dummy, 9000);
        } else if c >= b'0' && c <= b'9' {
            break;   /* found what we were looking for */
        }
        let tmp = fd.getc().map_or_else(|_| { -1 }, |v| { v as i32 });
        c = tmp as u8;
    }

    /* we're at the start of a number, continue until we hit a non-number */
    let mut i: i32 = 0;
    // loopbound min 1 max 2
    loop {
        i = (i * 10) + (c as i32 - b'0' as i32);
        let tmp = fd.getc();
        if tmp.is_err() { return i; }
        c = tmp.unwrap() as u8;
        if c < b'0' || c > b'9' { break; }
    }
    i
}

fn susan_get_image(
    fd: &mut File,
    inp: &mut HeapSlice,
    x_size: &mut u16,
    y_size: &mut u16,
    heap: &mut SimulatedHeap,
) {
    let mut header: [u8; 100] = [0; 100];

    _ = fd.seek(0, SeekOrigin::SET);

    /* {{{ read header */

    header[0] = fd.getc().map_or_else(|_| { -1 }, |v| { v as i32 }) as u8;
    header[1] = fd.getc().map_or_else(|_| { -1 }, |v| { v as i32 }) as u8;
    if !(header[0] == b'P' && header[1] == b'5') {
        /* "Image does not have binary PGM header." */
    }

    *x_size = susan_getint(fd) as u16;
    *y_size = susan_getint(fd) as u16;
    // dummy read
    susan_getint(fd);

    /* }}} */
    *inp = heap.malloc(*x_size * *y_size, size_of::<u8>() as u8);

    if fd.read(heap.return_slice_i8(*inp), (*x_size * *y_size) as usize) == Ok(0) {
        /* "Image is wrong size.\n" */
    }
}

fn susan_put_image(x_size: u16, y_size: u16) {
  // loopbound min 7220 max 7220
  for _ in 0..(x_size * y_size) {}
}

// r: &[i8],
// inp: &mut [uchar],
fn susan_int_to_uchar(
    r: HeapSlice,
    inp: HeapSlice,
    heap: &mut SimulatedHeap,
) {
    let r_start = r.start as usize;
    let r_end = r.end as usize;
    let inp_start = inp.start as usize;
    let inp_end = inp.end as usize;
    // loopbound min 0 max 0
    let min_r = heap.heap_content[r_start..r_end].iter().min().copied().unwrap_or(0) as i8;
    let max_r = heap.heap_content[r_start..r_end].iter().max().copied().unwrap_or(0) as i8;

    if min_r == max_r {
        // loopbound min 0 max 0
        heap.heap_content[inp_start..inp_end].fill(0);
        return;
    }
    // loopbound min 0 max 0
    for i in 0..(r.end - r.start) {
        let r_val = heap.heap_content[r_start + i as usize] as i8;
        let val = ((r_val - min_r) as u8 * 255) / (max_r - min_r) as u8;
        heap.heap_content[inp_start + i as usize] = val;
    }
}

fn susan_setup_brightness_lut(
    bp: &mut HeapSlice,
    thresh: i32,
    form: i32,
    heap: &mut SimulatedHeap,
) {
    *bp = heap.malloc(516, size_of::<Uchar>() as u8);
    // loopbound min 513 max 513
    for k in 2..515 {
        let mut temp: f32 = k as f32 / thresh as f32;
        temp *= temp;
        if form == 6 { temp = temp * temp * temp; }
        temp = 100.0 * susan_expf(-temp);
        let bp_i = bp.start + bp.type_size as u16 * k;
        heap.heap_content[bp_i as usize] = temp as Uchar;
    }
}

// inp: &[uchar],
fn susan_median(
    inp: HeapSlice,
    i: u16,
    j: u16,
    x_size: u16,
    heap: &mut SimulatedHeap,
) -> u8 {
    let mut p: [i32; 8] = [0; 8];

    p[0] = heap.heap_content[(inp.start + (i - 1) * x_size + j - 1) as usize] as i32;
    p[1] = heap.heap_content[(inp.start + (i - 1) * x_size + j) as usize] as i32;
    p[2] = heap.heap_content[(inp.start + (i - 1) * x_size + j + 1) as usize] as i32;
    p[3] = heap.heap_content[(inp.start + i * x_size + j - 1) as usize] as i32;
    p[4] = heap.heap_content[(inp.start + i * x_size + j + 1) as usize] as i32;
    p[5] = heap.heap_content[(inp.start + (i + 1) * x_size + j - 1) as usize] as i32;
    p[6] = heap.heap_content[(inp.start + (i + 1) * x_size + j) as usize] as i32;
    p[7] = heap.heap_content[(inp.start + (i + 1) * x_size + j + 1) as usize] as i32;

    // original loopbounds: outer min 6 max 6; inner min 0 max outer
    p.sort_unstable();

    return ((p[3] + p[4]) / 2) as Uchar;
}

/* this enlarges "in" so that borders can be dealt with easily */
fn susan_enlarge(
    inp: &mut HeapSlice,
    tmp_image: &mut HeapSlice,
    x_size: &mut u16,
    y_size: &mut u16,
    border: u16,
    heap: &mut SimulatedHeap,
) {
    // loopbound min 95 max 95
    for i in 0..*y_size { /* copy *in into tmp_image */
        let tmp_offset = (i + border) * (*x_size + 2 * border) + border;
        let tmp_slice = HeapSlice {
            start: tmp_image.start + tmp_offset,
            end: tmp_image.end,
            type_size: tmp_image.type_size,
        };
        let inp_slice = HeapSlice {
            start: inp.start + i * *x_size,
            end: inp.start + (i + 1) * *x_size,
            type_size: inp.type_size,
        };
        heap.memcpy(inp_slice, tmp_slice);
    }

    // loopbound min 7 max 7
    for i in 0..border { /* copy top and bottom rows; invert as many as necessary */
        let tmp_offset = (border - 1 - i) * (*x_size + 2 * border);
        let tmp_slice = HeapSlice {
            start: tmp_image.start + tmp_offset,
            end: tmp_image.end,
            type_size: tmp_image.type_size,
        };
        let inp_slice = HeapSlice {
            start: inp.start + i * *x_size,
            end: inp.start + (i + 1) * *x_size,
            type_size: inp.type_size,
        };
        heap.memcpy(inp_slice, tmp_slice);

        let tmp_offset = (*y_size + border + i) * (*x_size + 2 * border) + border;
        let tmp_slice = HeapSlice {
            start: tmp_image.start + tmp_offset,
            end: tmp_image.end,
            type_size: tmp_image.type_size,
        };
        let inp_slice = HeapSlice {
            start: inp.start + (*y_size - i - 1) * *x_size,
            end: inp.start + (*y_size - i) * *x_size,
            type_size: inp.type_size,
        };
        heap.memcpy(inp_slice, tmp_slice);
    }

    // loopbound min 7 max 7
    for i in 0..border { /* copy left and right columns */
        // loopbound min 109 max 109
        for j in 0..(*y_size + 2 * border) {
            let shared = j * (*x_size + 2 * border) + border;
            let src_i = (tmp_image.start + shared + i) as usize;
            let dst_i = (tmp_image.start + shared - 1 - i) as usize;
            heap.heap_content[dst_i] = heap.heap_content[src_i];
            let src_i = (tmp_image.start + shared + *x_size - 1 - i) as usize;
            let dst_i = (tmp_image.start + shared + *x_size + i) as usize;
            heap.heap_content[dst_i] = heap.heap_content[src_i];
        }
    }

    *x_size += 2 * border; /* alter image size */
    *y_size += 2 * border;
    *inp = *tmp_image;    /* repoint in */
}

// inp: &[uchar],
// bp: &[uchar],
// dp: Option<&[uchar]>,
#[inline]
fn susan_smoothing_main<F>(
    inp: HeapSlice,
    bp: HeapSlice,
    dp: Option<HeapSlice>,
    dp_index: &mut u16,
    f: F,
    x_size: u16,
    y_size: u16,
    mask_size: u16,
    increment: u16,
    heap: &mut SimulatedHeap,
) where
    F: Fn(HeapSlice, HeapSlice, Option<HeapSlice>,
        &mut u16, &mut u16, &mut u16,
        &mut TotalType, &mut i32, isize, u16, &mut SimulatedHeap)
{
    let mut inp_index: u16;
    let cp = bp;
    /* {{{ main section */
    for i in mask_size..(y_size - mask_size) {
        for j in mask_size..(x_size - mask_size) {
            let mut area: i32 = 0;
            let mut total: i32 = 0;
            inp_index = (i - mask_size) * x_size + j - mask_size;
            let centre = heap.heap_content[(inp.start + i * x_size + j) as usize];
            let mut cp_index = centre as u16;
            let mut dp_curr = *dp_index;

            f(
                inp, cp, dp, &mut inp_index, &mut cp_index, &mut dp_curr, &mut total, &mut area, mask_size as isize, increment, heap
            );

            let tmp = area - 10000;
            if tmp == 0 {
                heap.heap_content[(inp.start + inp_index) as usize] = susan_median(inp, i, j, x_size, heap);
            } else {
                heap.heap_content[(inp.start + inp_index) as usize] = ((total - (centre as i32 * 10000)) / tmp) as u8;
            }
        }
    }
}

// Changelog: return statements in error if-cases
fn susan_smoothing(
    three_by_three: bool,
    inp: &mut HeapSlice,
    dt: f32,
    x_size: &mut u16,
    y_size: &mut u16,
    bp: HeapSlice,
    heap: &mut SimulatedHeap,
) {
    /* {{{ setup larger image and border sizes */
    let mask_size = if !three_by_three { (1.5 * dt) as u16 + 1 } else { 1 };

    if dt > 15. {
        /*  "Distance_thresh too big for integer arithmetic." */
        // Either reduce it to <=15 or recompile with variable "total"
        // as a float: see top "defines" section.
        return; 
    }

    if (2 * mask_size + 1 > *x_size) || (2 * mask_size + 1 > *y_size) {
      /*  "Mask size too big for image." */
      return;
    }

    let mut tmp_image = heap.malloc((*x_size + mask_size * 2) * (*y_size + mask_size * 2), size_of::<u8>() as u8);

    susan_enlarge(
        inp, &mut tmp_image, x_size, y_size, mask_size, heap,
    );

    let increment;
    if !three_by_three {
        /* large Gaussian masks */
        /* {{{ setup distance lut */

        let n_max = (mask_size * 2) + 1;
        increment = *x_size - n_max;

        let dp = heap.malloc(n_max * n_max, size_of::<Uchar>() as u8);
        let temp: f32 = -(dt * dt);

        let mut dp_index = 0;
        let range: i32 = mask_size as i32;
        // loopbound min 15 max 15
        for i in -range..=range {
            // loopbound min 15 max 15
            for j in -range..=range {
                let x: i32 = 100 * susan_expf(((i * i) + (j * j)) as f32 / temp) as i32;
                heap.heap_content[(dp.start + dp_index) as usize] = x as u8;
                dp_index += 1;
            }
        }

        let f = |
            inp: HeapSlice, cp: HeapSlice, dp: Option<HeapSlice>,
            inp_index: &mut u16, cp_index: &mut u16, dp_index: &mut u16,
            total: &mut TotalType, area: &mut i32, range: isize, increment: u16, heap: &mut SimulatedHeap,
        | {
            // loopbound min 15 max 15
            for _ in -range..=range {
                // loopbound min 15 max 15
                for _ in -range..=range {
                    let brightness = heap.heap_content[(inp.start + *inp_index) as usize];
                    *inp_index += 1;
                    let tmp = heap.heap_content[(dp.unwrap().start + *dp_index) as usize] * heap.heap_content[(cp.start + *cp_index - brightness as u16) as usize];
                    *dp_index += 1;
                    *area += tmp as i32;
                    *total += (tmp * brightness) as i32;
                }
                *inp_index += increment;
            }
        };

        // loopbound min 95 max 95 -> outer
        // loopbound min 76 max 76 -> inner
        susan_smoothing_main(
            *inp, bp, Some(dp), &mut dp_index, f, *x_size, *y_size, mask_size, increment, heap,
        );


    } else {
        /* 3x3 constant mask */

        let f = |
            inp: HeapSlice, cp: HeapSlice, _dp: Option<HeapSlice>,
            inp_index: &mut u16, cp_index: &mut u16, _dp_index: &mut u16,
            total: &mut TotalType, area: &mut i32, _range: isize, _increment: u16, heap: &mut SimulatedHeap,
        | {
            let pattern = [
                (true, 0), (true, 0), (false, *x_size - 2),
                (true, 0), (true, 0), (false, *x_size - 2),
                (true, 0), (true, 0), (false, 0)
            ];
            for (inc, off) in pattern.iter() {
                let brightness = heap.heap_content[(inp.start + *inp_index) as usize];
                if *inc { *inp_index += 1 };      
                let tmp = heap.heap_content[(cp.start + *cp_index - brightness as u16) as usize];
                *area += tmp as i32;
                *total += (tmp * brightness) as TotalType;
                *inp_index += off;
            }
        };

        // loopbound min 15 max 15 -> outer            //neue Änderung min max 107
        // loopbound min 15 max 15 -> inner           //neue Änderung min max 88
        susan_smoothing_main(
            *inp, bp, None, &mut 0, f, *x_size, *y_size, 1, 0, heap,
        );

    }
}

// inp: &mut [uchar],
// mid: &[uchar],
fn susan_edge_draw(
    inp: HeapSlice,
    mid: HeapSlice,
    mid_start: u16,
    x_size: u16,
    y_size: u16,
    drawing_mode: DrawingMode,
    heap: &mut SimulatedHeap,
) {
    if drawing_mode == DrawingMode::Smooth {
        /* mark 3x3 white block around each edge point */
        let mut mid_index = mid_start;
        // loopbound min 7220 max 7220
        for _ in 0..(x_size * y_size) {
            if heap.heap_content[(mid.start + mid_index) as usize] < 8 {
                let mut inp_index = mid_index - mid_start - x_size - 1;
                heap.heap_content[(inp.start + inp_index) as usize] = 255;
                inp_index += 1;
                heap.heap_content[(inp.start + inp_index) as usize] = 255;
                inp_index += 1;
                heap.heap_content[(inp.start + inp_index) as usize] = 255;
                inp_index += x_size - 2;
                heap.heap_content[(inp.start + inp_index) as usize] = 255;
                inp_index += 1;
                heap.heap_content[(inp.start + inp_index) as usize] = 255;
                inp_index += x_size - 2;
                heap.heap_content[(inp.start + inp_index) as usize] = 255;
                inp_index += 1;
                heap.heap_content[(inp.start + inp_index) as usize] = 255;
                inp_index += 1;
                heap.heap_content[(inp.start + inp_index) as usize] = 255;
            }
            mid_index += 1;
        }
    }

    /* now mark 1 black pixel at each edge point */
    let mut mid_index = 0;
    // loopbound min 7220 max 7220
    for _ in 0..(x_size * y_size) {
        if heap.heap_content[(mid.start + mid_index) as usize] < 8 { heap.heap_content[(inp.start + mid_index - mid.start) as usize] = 0; }
        mid_index += 1;
    }
}

// r: &[i8],
// mid: &mut [uchar],
fn susan_thin(
    r: HeapSlice,
    mid: HeapSlice,
    x_size: u16,
    y_size: u16,
    heap: &mut SimulatedHeap,
) {
    let x_usize = x_size as usize;
    let y_usize = y_size as usize;
    let mid_start = mid.start as usize;
    let r_start = r.start as usize;
    // loopbound min 87 max 87
    let mut i = 4;
    while i < (y_usize - 4) {
        // loopbound min 68 max 68
        let mut j = 4;
        while j < (x_usize - 4) {
            if heap.heap_content[mid_start + i * x_usize + j] < 8 {
            let centre = heap.heap_content[r_start + i * x_usize + j];
            /* {{{ count number of neighbours */

            let mid_index = ( i - 1 ) * x_usize + j - 1;

            let n = (heap.heap_content[mid_start + mid_index] < 8) as i32 +
                (heap.heap_content[mid_start + mid_index + 1] < 8 ) as i32 +
                (heap.heap_content[mid_start + mid_index + 2] < 8 ) as i32 +
                (heap.heap_content[mid_start + mid_index + x_usize] < 8 ) as i32 +
                (heap.heap_content[mid_start + mid_index + x_usize + 2] < 8 ) as i32 +
                (heap.heap_content[mid_start + mid_index + x_usize + x_usize] < 8 ) as i32 +
                (heap.heap_content[mid_start + mid_index + x_usize + x_usize + 1] < 8 ) as i32 +
                (heap.heap_content[mid_start + mid_index + x_usize + x_usize + 2] < 8 ) as i32;

            /* {{{ n==0 no neighbours - remove point */
            if n == 0 { heap.heap_content[mid_start + i * x_usize + j] = 100; }

            /* {{{ n==1 - extend line if I can */

            /* extension is only allowed a few times - the value of mid is used to control this */

            if (n == 1) && (heap.heap_content[mid_start + i * x_usize + j] < 6) {
                /* find maximum neighbour weighted in direction opposite the
                    neighbour already present. e.g.
                    have: O O O  weight r by 0 2 3
                        X X O              0 0 4
                        O O O              0 2 3
                */
                let mut l: [i32; 9] = [0; 9];
                l[0] = heap.heap_content[r_start + (i - 1) * x_usize + j - 1] as i32;
                l[1] = heap.heap_content[r_start + (i - 1) * x_usize + j] as i32;
                l[2] = heap.heap_content[r_start + (i - 1) * x_usize + j + 1] as i32;
                l[3] = heap.heap_content[r_start + i * x_usize + j - 1] as i32;
                l[4] = 0;
                l[5] = heap.heap_content[r_start + i * x_usize + j + 1] as i32;
                l[6] = heap.heap_content[r_start + (i + 1) * x_usize + j - 1] as i32;
                l[7] = heap.heap_content[r_start + (i + 1) * x_usize + j] as i32;
                l[8] = heap.heap_content[r_start + (i + 1) * x_usize + j + 1] as i32;

                let weight_mask = [
                    [0, 0, 2, 0, -1, 3, 2, 3, 4],
                    [0, 0, 0, 2, -1, 2, 3, 4, 3],
                    [2, 0, 0, 3, -1, 0, 4, 3, 2],
                    [0, 2, 3, 0, -1, 4, 0, 2, 3],
                    [-1, -1, -1, -1, -1, -1, -1, -1, -1],  // not used
                    [3, 2, 0, 4, -1, 0, 3, 2, 0],
                    [2, 3, 4, 0, -1, 3, 0, 0, 2],
                    [3, 4, 3, 2, -1, 2, 0, 0, 0],
                    [4, 3, 2, 3, -1, 0, 2, 0, 0],
                ];

                for a in -1i32..=1i32 {
                    for b in -1i32..=1i32 {
                        if a == 0 && b == 0 { continue; }
                        if heap.heap_content[(mid.start + ((i as i32 - a) * x_size as i32 + j as i32 - b) as u16) as usize] < 8 {
                            let wm = weight_mask[((a + 1) * 3 + b + 1) as usize];
                            for (ind, val) in l.iter_mut().enumerate() {
                                if ind == 4 { continue; }
                                *val = wm[ind];
                            }
                            break;       
                        }
                    }
                }

                let mut m = 0;   /* find the highest point */
                let mut a = 0;
                let mut b = 0;
                // loopbound min 3 max 3
                for y in 0..3 {
                    // loopbound min 3 max 3
                    for x in 0..3 {
                        if l[y + y + y + x] > m {
                            m = l[y + y + y + x];
                            a = y;
                            b = x;
                        }
                    }
                }

                if m > 0 {
                    if heap.heap_content[mid_start +  i * x_usize + j] < 4 {
                        heap.heap_content[mid_start + (i + a - 1) * x_usize + j + b - 1] = 4;
                    } else {
                        heap.heap_content[mid_start + (i + a - 1) * x_usize + j + b - 1]
                                = heap.heap_content[mid_start + i * x_usize + j] + 1;
                    }
                    if (a + a + b) < 3 { /* need to jump back in image */
                        i += a - 1;
                        j += b - 2;
                    if i < 4 { i = 4; }
                    if j < 4 { j = 4; }
                }
                }
            }

            /* {{{ n==2 */

            if n == 2 {
                /* put in a bit here to straighten edges */
                let b00 = heap.heap_content[mid_start + (i - 1) * x_usize + j - 1] < 8; /* corners of 3x3 */
                let b02 = heap.heap_content[mid_start + (i - 1) * x_usize + j + 1] < 8;
                let b20 = heap.heap_content[mid_start + (i + 1) * x_usize + j - 1] < 8;
                let b22 = heap.heap_content[mid_start + (i + 1) * x_usize + j + 1] < 8;
                if ((b00 as i32 + b02 as i32 + b20 as i32 + b22 as i32) == 2)
                    && ((b00 | b22) & (b02 | b20)) {
                    /* case: move a point back into line.
                        e.g. X O X  CAN  become X X X
                            O X O              O O O
                            O O O              O O O    */
                    let x: i32;
                    let y: i32;
                    if b00 {
                        if b02 {
                            x = 0;
                            y = -1;
                        } else     {
                            x = -1;
                            y = 0;
                        }
                    } else {
                        if b02 {
                            x = 1;
                            y = 0;
                        } else     {
                            x = 0;
                            y = 1;
                        }
                    }
                    if (heap.heap_content[r_start + ((i as i32 + y) * x_size as i32 + j as i32 + x) as usize] as f32 / centre as f32) > 0.7 {
                        let tmp: i32 = (i as i32 + 2 * y) * x_size as i32 + j as i32;
                        let lhs: bool = (x == 0)
                            && (heap.heap_content[(mid.start as i32 + tmp) as usize] > 7)
                            && (heap.heap_content[(mid.start as i32 + (tmp - 1)) as usize] > 7)
                            && (heap.heap_content[(mid.start as i32 + (tmp + 1)) as usize] > 7);
                        let tmp: i32 = x_size as i32 + j as i32 + 2 * x;
                        let rhs: bool = (y == 0)
                            && (heap.heap_content[(mid.start as i32 + (i as i32 * tmp)) as usize] > 7)
                            && (heap.heap_content[(mid.start as i32 + ((i + 1) as i32 * tmp)) as usize] > 7)
                            && (heap.heap_content[(mid.start as i32 + ((i - 1) as i32 * tmp)) as usize] > 7) ;
                        if lhs || rhs {
                            heap.heap_content[mid_start + i * x_usize + j] = 100;
                            heap.heap_content[(mid.start as i32 + ((i as i32 + y) * x_size as i32 + j as i32 + x)) as usize] = 3; /* no jumping needed */
                        }
                    }
                } else {
                    let b01 = heap.heap_content[mid_start + (i - 1) * x_usize + j] < 8;
                    let b12 = heap.heap_content[mid_start + i* x_usize + j + 1] < 8;
                    let b21 = heap.heap_content[mid_start + (i + 1) * x_usize + j] < 8;
                    let b10 = heap.heap_content[mid_start + i * x_usize + j - 1] < 8;
                    /* {{{ right angle ends - not currently used */

                    if (b00 & b01) | (b00 & b10)
                        | (b02 & b01) | (b02 & b12)
                        | (b20 & b10) | (b20 & b21)
                        | (b22 & b21) | (b22 & b12) {
                        /* case; right angle ends. clean up.
                            e.g.; X X O  CAN  become X X O
                            O X O              O O O
                            O O O              O O O        */
                        let cond1 = b01
                            & (heap.heap_content[mid_start + (i - 2) * x_usize + j - 1] > 7)
                            & (heap.heap_content[mid_start + (i - 2) * x_usize + j] > 7)
                            & (heap.heap_content[mid_start + (i - 2) * x_usize + j + 1] > 7)
                            & ((b00 & ((2 * heap.heap_content[r_start + (i - 1) * x_usize + j + 1]) > centre))
                                | (b02 & ((2 * heap.heap_content[r_start + (i - 1) * x_usize + j - 1]) > centre)));
                        let cond2 = b10
                            & (heap.heap_content[mid_start + (i - 1) * x_usize + j - 2] > 7)
                            & (heap.heap_content[mid_start + i * x_usize + j - 2] > 7)
                            & (heap.heap_content[mid_start + (i + 1) * x_usize + j - 2] > 7)
                            & ((b00 & ((2 * heap.heap_content[r_start + (i + 1) * x_usize + j - 1]) > centre))
                                | (b20 & ((2 * heap.heap_content[r_start + (i - 1) * x_usize + j - 1]) > centre)));
                        let cond3 = b12
                            & (heap.heap_content[mid_start + (i - 1) * x_usize + j + 2] > 7)
                            & (heap.heap_content[mid_start + i * x_usize + j + 2] > 7)
                            & (heap.heap_content[mid_start + (i + 1) * x_usize + j + 2] > 7)
                            & ((b02 & ((2 * heap.heap_content[r_start + (i + 1) * x_usize + j + 1]) > centre))
                                | (b22 & ((2 * heap.heap_content[r_start + (i - 1) * x_usize + j + 1]) > centre)));
                        let cond4 = b21
                            & (heap.heap_content[mid_start + (i + 2) * x_usize + j - 1] > 7)
                            & (heap.heap_content[mid_start + (i + 2) * x_usize + j] > 7)
                            & (heap.heap_content[mid_start + (i + 2) * x_usize + j + 1] > 7 )
                            & ((b20 & ((2 * heap.heap_content[r_start + (i + 1) * x_usize + j + 1]) > centre))
                                | (b22 & ((2 * heap.heap_content[r_start + (i + 1) * x_usize + j - 1]) > centre)));
                        if cond1 | cond2 | cond3 | cond4 {
                            heap.heap_content[mid_start + i * x_usize + j] = 100;
                            if b10 & b20 { j -= 2; }
                            if b00 | b01 | b02 {
                                i -= 1;
                                j -= 2;
                            }
                        }
                    }
                    let cond1 = b01 & ((heap.heap_content[mid_start + (i - 2) * x_usize + j - 1] < 8)
                        | (heap.heap_content[mid_start + (i - 2) * x_usize + j + 1] < 8));
                    let cond2 = b10 & ((heap.heap_content[mid_start + (i - 1) * x_usize + j - 2] < 8)
                        | (heap.heap_content[mid_start + (i + 1) * x_usize + j - 2] < 8));
                    let cond3 = b12 & ((heap.heap_content[mid_start + (i - 1) * x_usize + j + 2] < 8)
                        | (heap.heap_content[mid_start + (i + 1) * x_usize + j + 2] < 8));
                    let cond4 = b21 & ((heap.heap_content[mid_start + (i + 2) * x_usize + j - 1] < 8)
                        | (heap.heap_content[mid_start + (i + 2) * x_usize + j + 1] < 8));
                    let cond = cond1 | cond2 | cond3 | cond4;
                    if ((b01 as i32 + b12 as i32 + b21 as i32 + b10 as i32) == 2)
                        && ((b10 | b12) & (b01 | b21))
                        && cond {
                        /* case; clears odd right angles.
                            e.g.; O O O  becomes O O O
                            X X O          X O O
                            O X O          O X O     */
                        heap.heap_content[mid_start + i * x_usize + j] = 100;
                        i -= 1;               /* jump back */
                        j -= 2;
                        if i < 4 { i = 4 };
                        if j < 4 { j = 4 };
                    }
                }
            }

            /* {{{ n>2 the thinning is done here without breaking connectivity */

                if n > 2 {
                    let b01 = (heap.heap_content[mid_start + (i - 1) * x_usize + j] < 8) as i32;
                    let b12 = (heap.heap_content[mid_start + i * x_usize + j + 1] < 8) as i32;
                    let b21 = (heap.heap_content[mid_start + (i + 1) * x_usize + j] < 8) as i32;
                    let b10 = (heap.heap_content[mid_start + i * x_usize + j - 1] < 8) as i32;
                    if (b01 + b12 + b21 + b10) > 1 {
                        let b00 = (heap.heap_content[mid_start + (i - 1) * x_usize + j - 1] < 8) as i32;
                        let b02 = (heap.heap_content[mid_start + (i - 1) * x_usize + j + 1] < 8) as i32;
                        let b20 = (heap.heap_content[mid_start + (i + 1) * x_usize + j - 1] < 8) as i32;
                        let b22 = (heap.heap_content[mid_start + (i + 1) * x_usize + j + 1] < 8) as i32;
                        let p1 = b00 | b01;
                        let p2 = b02 | b12;
                        let p3 = b22 | b21;
                        let p4 = b20 | b10;

                        if ((p1 + p2 + p3 + p4) - ((b01 & p2) + (b12 & p3) + (b21 & p4) + (b10 & p1))) < 2 {
                            heap.heap_content[mid_start + i * x_usize + j] = 100;
                            i -= 1;
                            j -= 2;
                            if i < 4 { i = 4; }
                            if j < 4 { j = 4; }
                        }
                    }
                }
            }
            j += 1;
        }
        i += 1;
    }
}

// new helper functions to reduce code duplication

// cp: &[uchar],
// p: &[uchar],
fn susan_acc(
    acc: &mut i32,
    cp: HeapSlice,
    cp_index: u16,
    p: HeapSlice,
    p_index: &mut u16,
    count: usize,
    heap: &mut SimulatedHeap
) {
    for i in 0..count {
        *acc += heap.heap_content[(cp.start + cp_index - heap.heap_content[(p.start + *p_index) as usize] as u16) as usize] as i32;
        if i < (count - 1) { *p_index += 1; }
    }
}

// cp: &[uchar],
// p: &[uchar],
fn susan_cornerness_acc(
    cp: &HeapSlice,
    cp_index: u16,
    p: &HeapSlice,
    p_index: &mut u16,
    pattern: &(&[&[i32]], usize),
    accs: &mut [&mut i32],
    heap: &mut SimulatedHeap
) {
    let (weights, cnt) = pattern;
    for i in 0..*cnt {
        let c = heap.heap_content[(cp.start + cp_index - heap.heap_content[(p.start + *p_index) as usize] as u16) as usize];
        if i < (cnt - 1) { *p_index += 1; }
        let var_weights = weights[i];
        for (j, var) in accs.iter_mut().enumerate() {
            **var += var_weights[j] * c as i32;
        }
    }
}

// cp: &[uchar],
// p: &[uchar],
fn susan_cornerness_base(
    cp: &HeapSlice,
    cp_index: u16,
    p: &HeapSlice,
    x_size: u16,
    i: u16,
    j: u16,
    i_offset: u16,
    pattern: &[(&[&[i32]], u16)],
    accs: &mut [&mut i32],
    heap: &mut SimulatedHeap,
) {
    let mut p_index = (i - i_offset) * x_size + j - 1;
    for (weights, inc) in pattern.iter() {
        let cnt = weights.len();
        susan_cornerness_acc(
            &cp, cp_index, &p, &mut p_index, &(*weights, cnt), accs, heap
        );
        p_index += inc;
    }
}

// p: &[uchar],
// cp: &[uchar],
fn susan_main_base(
    p: HeapSlice,
    r: HeapSlice,
    cp: HeapSlice,
    x_size: u16,
    y_size: u16,
    max_no: i32,
    i_offset: u16,
    j_offset: u16,
    pattern: &[(i32, u16)],
    heap: &mut SimulatedHeap
) {
    for i in i_offset..(y_size - i_offset) {
        for j in j_offset..(x_size - j_offset) {
            let mut n: i32 = 100;
            let mut p_index = (i - i_offset) * x_size + j - 1;
            let cp_index = heap.heap_content[(p.start + i * x_size + j) as usize] as u16;
            for (cnt, inc) in pattern.iter() {
                susan_acc(
                    &mut n, cp, cp_index, p, &mut p_index, *cnt as usize, heap,
                );
                p_index += inc;
            }
            if n <= max_no { heap.heap_content[(r.start + i * x_size + j) as usize] = (max_no - n) as u8; }
        }
    }
}

// complete functions

// inp: &[uchar],
// bp: &[uchar],
fn susan_principle(
    inp: HeapSlice,
    r_slice: HeapSlice,
    bp: HeapSlice,
    max_no: i32,
    x_size: u16,
    y_size: u16,
    heap: &mut SimulatedHeap,
) {
    memset(heap.return_slice_i8(r_slice), 0i8);
    let pattern = [
        (3, x_size - 3), (5, x_size - 5),
        (7, x_size - 6), (3, 2), (3, x_size - 6),
        (7, x_size - 5), (5, x_size - 3), (3, 0),
    ];
    // loopbound min 0 max 0 -> outer
    // loopbound min 0 max 0 -> inner
    susan_main_base(
        inp, r_slice, bp, x_size, y_size, max_no,
        3, 3, &pattern, heap
    );
}

// inp: &[uchar],
// bp: &[uchar],
fn susan_edges(
    inp: HeapSlice,
    r_slice: HeapSlice,
    mid_slice: HeapSlice,
    bp: HeapSlice,
    max_no: i32,
    x_size: u16,
    y_size: u16,
    heap: &mut SimulatedHeap,
) {
    memset(heap.return_slice_i8(r_slice), 0);

    let pattern = [
        (3, x_size - 3), (5, x_size - 5),
        (7, x_size - 6), (3, 2), (3, x_size - 6),
        (7, x_size - 5), (5, x_size - 3), (3, 0),
    ];
    // loopbound min 89 max 89 -> outer
    // loopbound min 70 max 70 -> inner
    susan_main_base(
        inp, r_slice, bp, x_size, y_size, max_no,
        3, 3, &pattern, heap,
    );

    let mut z: f32 = 0.;
    // loopbound min 87 max 87
    for i in 4..(y_size - 4) {
        // loopbound min 68 max 68
        for j in 4..(x_size - 4) {
            if heap.return_slice_i8(r_slice)[(i * x_size + j) as usize] > 0 {
                let m = heap.return_slice_i8(r_slice)[(i * x_size + j) as usize];
                let n = max_no - m as i32;
                let cp_index = heap.heap_content[(inp.start + i * x_size + j) as usize] as u16;

                let do_symmetry: bool;
                if n > 600 {
                    let mut x: i32 = 0;
                    let mut y: i32 = 0;

                    let weights: [&[&[i32]]; 8] = [
                        &[&[-1, -3], &[0, -3], &[1, -3]],
                        &[&[-2, -2], &[-1, -2], &[0, -2], &[1, -2], &[2, -2]],
                        &[&[-3, -1], &[-2, -1], &[-1, -1], &[0, -1], &[1, -1], &[2, -1], &[3, -1]],
                        &[&[-3, 0], &[-2, 0], &[-1, 0]],
                        &[&[-1, 0], &[2, 0], &[3, 0]],
                        &[&[-3, 1], &[-2, 1], &[-1, 1], &[0, 1], &[1, 1], &[2, 1], &[3, 1]],
                        &[&[-2, 2], &[-1, 2], &[0, 2], &[1, 2], &[2, 2]],
                        &[&[-1, 3], &[0, 3], &[1, 3]],
                    ];
                    let mut accs = [&mut x, &mut y];
                    let pattern = [
                        (weights[0], x_size - 3),
                        (weights[1], x_size - 5),
                        (weights[2], x_size - 6),
                        (weights[3], 2),
                        (weights[4], x_size - 6),
                        (weights[5], x_size - 5),
                        (weights[6], x_size - 3),
                        (weights[7], 0),
                    ];
                    susan_cornerness_base(
                        &bp, cp_index, &inp, x_size, i, j, 3, &pattern, &mut accs, heap
                    );

                    z = susan_sqrtf((x * x + y * y) as f32);
                    let mut a: i32 = 0;
                    let mut b: i32 = 0;
                    if z > (0.9 * n as f32) {
                        /* 0.5 */
                        do_symmetry = false;
                        susan_edges_non_symmetry(
                            x, y, &mut a, &mut b, &mut z
                        );
                        let cond1 = m > heap.return_slice_i8(r_slice)[((i as i32 + a) * x_size as i32 + j as i32 + b) as usize];
                        let cond2 = m >= heap.return_slice_i8(r_slice)[((i as i32 - a) * x_size as i32 + j as i32 - b) as usize];
                        let cond3 = m > heap.return_slice_i8(r_slice)[((i as i32 + 2 * a) * x_size as i32 + j as i32 + 2 * b) as usize];
                        let cond4 = m >= heap.return_slice_i8(r_slice)[((i as i32 - 2 * a) * x_size as i32 + j as i32 - 2 * b) as usize];
                        if cond1 && cond2 && cond3 && cond4 {
                            heap.return_slice(mid_slice)[(i * x_size + j) as usize] = 1;
                        }
                    } else {
                        do_symmetry = true;
                    }
                } else {
                  do_symmetry = true;
                }

                if do_symmetry {
                    /*   |      \
                        y  -x-  w
                        |        \   */

                    let mut x: i32 = 0;
                    let mut y: i32 = 0;
                    let mut w: i32 = 0;

                    let weights: [&[&[i32]]; 8] = [
                        &[&[1, 9, 3], &[0, 9, 0], &[1, 9, -3]],
                        &[&[4, 4, 4], &[1, 4, 2], &[0, 4, 0], &[1, 4, -2], &[4, 4, -4]],
                        &[&[9, 1, 3], &[4, 1, 2], &[1, 1, 1], &[0, 1, 0], &[1, 1, -1], &[4, 1, -2], &[9, 1, -3]],
                        &[&[9, 0, 0], &[4, 0, 0], &[1, 0, 0]],
                        &[&[1, 0, 0], &[4, 0, 0], &[9, 0, 0]],
                        &[&[9, 1, -3], &[4, 1, -2], &[1, 1, -1], &[0, 1, 0], &[1, 1, 1], &[4, 1, 2], &[9, 1, 3]],
                        &[&[4, 4, -4], &[1, 4, -2], &[0, 4, 0], &[1, 4, 2], &[4, 4, 4]],
                        &[&[1, 9, -3], &[0, 9, 0], &[1, 9, 3]]
                    ];
                    let mut accs = [&mut x, &mut y, &mut w];
                    let pattern = [
                        (weights[0], x_size - 3),
                        (weights[1], x_size - 5),
                        (weights[2], x_size - 6),
                        (weights[3], 2),
                        (weights[4], x_size - 6),
                        (weights[5], x_size - 5),
                        (weights[6], x_size - 3),
                        (weights[7], 0),
                        
                    ];
                    susan_cornerness_base(
                        &bp, cp_index, &inp, x_size, i, j, 3, &pattern, &mut accs, heap,
                    );
                    
                    let mut a: i32 = 0;
                    let mut b: i32 = 0;
                    susan_edges_symmetry(
                        x, y, &mut a, &mut b, &mut z, w,
                    );
                    let cond1 = m > heap.return_slice_i8(r_slice)[((i as i32 + a) * x_size as i32 + j as i32 + b) as usize];
                    let cond2 = m >= heap.return_slice_i8(r_slice)[((i as i32 - a) * x_size as i32 + j as i32 - b) as usize];
                    let cond3 = m > heap.return_slice_i8(r_slice)[((i as i32 + 2 * a) * x_size as i32 + j as i32 + 2 * b) as usize];
                    let cond4 = m >= heap.return_slice_i8(r_slice)[((i as i32 - 2 * a) * x_size as i32 + j as i32 - 2 * b) as usize];
                    if cond1 && cond2 && cond3 && cond4 {
                        heap.return_slice(mid_slice)[(i * x_size + j) as usize] = 2;
                    }
                }
            }
        }
    }
}

// inp: &[uchar],
// r: &mut [i8],
// bp: &[uchar],
fn susan_corners(
  inp: HeapSlice,
  r: HeapSlice,
  bp: HeapSlice,
  max_no: i32,
  corner_list: &mut CornerListT,
  x_size: u16,
  y_size: u16,
  mask_mode: bool,
  heap: &mut SimulatedHeap,
) {
    memset(heap.return_slice_i8(r), 0);

    let cgx = heap.malloc(x_size * y_size, size_of::<i8>() as u8);
    let cgy = heap.malloc(x_size * y_size, size_of::<i8>() as u8);

    // loopbound min 85 max 85
    for i in 5..(y_size - 5) {
        // loopbound min 66 max 66
        for j in 5..(x_size - 5) {
            let mut n: i32 = 100;
            let mut p_index = (i - 3) * x_size + j - 1;
            let cp_index = heap.heap_content[(inp.start + i * x_size + j) as usize] as u16;

            susan_acc(
              &mut n, bp, cp_index, inp, &mut p_index, 3, heap,
            );
            p_index += x_size - 3;
            susan_acc(
              &mut n, bp, cp_index, inp, &mut p_index, 5, heap,
            );
            p_index += x_size - 5;
            susan_acc(
              &mut n, bp, cp_index, inp, &mut p_index, 7, heap,
            );
            p_index += x_size - 6;
            susan_acc(
              &mut n, bp, cp_index, inp, &mut p_index, 3, heap,
            );

            /* do this test early and often ONLY to save wasted computation */
            let table = [
                (2, true), (0, true), (0, false), (x_size - 6, true),
                (0, true), (0, true), (0, true), (0, true), (0, true),
                (0, false), (5, true), (0, true), (0, true), (0, true),
                (0, false), (x_size - 3, true), (0, true), (0, false)
            ];
            for (p_offset, inc) in table.iter() {
                let cond: bool = susan_if_step(
                    &mut n, bp, cp_index, inp, &mut p_index, max_no, Some(*p_offset), *inc, heap,
                );
                if cond { break; }
            }
            if n < max_no {
                let mut x: i32 = 0;
                let mut y: i32 = 0;
                let mut w: i32 = 0;

                let weights: [&[&[i32]]; 8] = [
                    &[&[-1, -3], &[0, -3], &[1, -3]],
                    &[&[-2, -2], &[-1, -2], &[0, -2], &[1, -2], &[2, -2]],
                    &[&[-3, -1], &[-2, -1], &[-1, -1], &[0, -1], &[1, -1], &[2, -1], &[3, -1]],
                    &[&[-3, 0], &[-2, 0], &[-1, 0]],
                    &[&[1, 0], &[2, 0], &[3, 0]],
                    &[&[-3, 1], &[-2, 1], &[-1, 1], &[0, 1], &[1, 1], &[2, 1], &[3, 1]],
                    &[&[-2, 2], &[-1, 2], &[0, 2], &[1, 2], &[2, 2]],
                    &[&[-1, 3], &[0, 3], &[1, 3]],
                ];
                let mut accs = [&mut x, &mut y, &mut w];
                let pattern = [
                  (weights[0], x_size - 3),
                  (weights[1], x_size - 5),
                  (weights[2], x_size - 6),
                  (weights[3], 2),
                  (weights[4], x_size - 6),
                  (weights[5], x_size - 5),
                  (weights[6], x_size - 3),
                  (weights[7], 0),
                ];
                let cp = bp;
                susan_cornerness_base(
                    &cp, cp_index, &inp, x_size, i, j, 3, &pattern, &mut accs, heap,
                );
                let xx: i32 = x * x;
                let yy: i32 = y * y;
                let sq: i32 = xx + yy;
                if sq > ((n * n) / 2) {
                    let mut sq: i32;
                    if yy < xx {
                        let divide = y as f32 / x.abs() as f32;
                        sq = x.signum();
                        let first_i = heap.heap_content[(inp.start as i32 + ((i as i32 + divide as i32) * x_size as i32 + j as i32 + sq)) as usize];
                        let second_i = heap.heap_content[(inp.start as i32 + ((i as i32 + 2 * divide as i32) * x_size as i32 + j as i32 + 2 * sq)) as usize];
                        let third_i = heap.heap_content[(inp.start as i32 + ((i as i32 + 3 * divide as i32) * x_size as i32 + j as i32 + 3 * sq)) as usize];
                        sq = (heap.heap_content[(cp.start + cp_index - first_i as u16) as usize]
                            + heap.heap_content[(cp.start + cp_index - second_i as u16) as usize]
                            + heap.heap_content[(cp.start + cp_index - third_i as u16) as usize]) as i32;
                    } else {
                        let divide = x as f32 / y.abs() as f32;
                        sq = y.signum();
                        let first_i = heap.heap_content[(inp.start as i32 + ((i as i32 + sq) * x_size as i32 + j as i32 + divide as i32)) as usize];
                        let second_i = heap.heap_content[(inp.start as i32 + ((i as i32 + 2 * sq) * x_size as i32 + j as i32 + (2. * divide) as i32)) as usize];
                        let third_i = heap.heap_content[(inp.start as i32 + ((i as i32 + 3 * sq) * x_size as i32 + j as i32 + (3. * divide) as i32)) as usize];
                        sq = (heap.heap_content[(cp.start + cp_index - first_i as u16) as usize]
                            + heap.heap_content[(cp.start + cp_index - second_i as u16) as usize]
                            + heap.heap_content[(cp.start + cp_index - third_i as u16) as usize]) as i32;
                    }
                    if sq > 290 {
                        heap.heap_content[(r.start +  i * x_size + j) as usize] = (max_no - n) as u8;
                        heap.return_slice_i8(cgx)[(i * x_size + j) as usize] = ((51 * x) / n) as i8;
                        heap.return_slice_i8(cgy)[(i * x_size + j) as usize] = ((51 * y) / n) as i8;
                    }
                }
            }
        }
    }
    /* to locate the local maxima */
    let mut n: usize = 0;
    let mask = if mask_mode { FIVE_SUPP } else { SEVEN_SUPP };
    let x_usize = x_size as usize;
    let y_usize = y_size as usize;
    let r_start = r.start as usize;
    let inp_start = inp.start as usize;

    // loopbound min 85 max 85
    for i in 5..(y_usize - 5) {
        // loopbound min 66 max 66
        for j in 5..(x_usize - 5) {
            let x: i8 = heap.heap_content[r_start + i * x_usize + j] as i8;
            if x <= 0 {
                continue;
            }
            if is_local_max(
                heap.return_slice_i8(r), x_size, y_size, i as u16, j as u16, mask
            ) {
                corner_list[n].info = 0;
                corner_list[n].x = j as i8;
                corner_list[n].y = i as i8;
                corner_list[n].dx = heap.return_slice_i8(cgx)[i * x_usize + j] as i8;
                corner_list[n].dy = heap.return_slice_i8(cgx)[i * x_usize + j] as i8;
                corner_list[n].i = heap.heap_content[inp_start + i * x_usize + j] as i8;
                n += 1;
                if n as i32 == MAX_CORNERS { /*  "Too many corners." */ }
            }
        }
    }
    corner_list[n].info = 7;
}

// small / quick functions

fn susan_principle_small( 
    inp: HeapSlice,
    r: HeapSlice,
    bp: HeapSlice,
    max_no: i32,
    x_size: u16,
    y_size: u16,
    heap: &mut SimulatedHeap
) {
    memset(heap.return_slice_i8(r), 0i8);
    let mut pattern = [
        (3, x_size - 2), (1, 2),
        (1, x_size -2), (3, 0),
    ];
    // loopbound min 0 max 0 -> outer
    // loopbound min 0 max 0 -> inner
    susan_main_base(
        inp, r, bp, x_size, y_size, max_no,
        1, 1, &mut pattern, heap
    );
}

#[inline]
fn susan_edges_non_symmetry(
    x: i32,
    y: i32,
    a: &mut i32,
    b: &mut i32,
    z: &mut f32,
) {
    let w: i32;
    if x == 0 {
        *z = 1000000.0;
    } else {
        *z = (y as f32) / (x as f32);
    }
    if *z < 0. {
        *z = -*z;
        w = -1;
    } else {
        w = 1;
    }
    if *z < 0.5 {
        /* vert_edge */
        *a = 0;
        *b = 1;
    } else if *z > 2.0 {
        /* hor_edge */
        *a = 1;
        *b = 0;
    } else if w > 0 {
        /* diag_edge */ 
        *a = 1;
        *b = 1;
    } else {
        *a = -1;
        *b = 1;
    }
}

#[inline]
fn susan_edges_symmetry(
    x: i32,
    y: i32,
    a: &mut i32,
    b: &mut i32,
    z: &mut f32,
    w: i32,
) {
    if y == 0 {
        *z = 1000000.0;
    } else {
        *z = (x as f32) / (y as f32);
    }
    if *z < 0.5 {
        /* vertical */
        *a = 0;
        *b = 1;
    } else if *z > 2.0 {
        /* horizontal */
        *a = 1;
        *b = 0;
    } else if w > 0 {
        /* diagonal */
        *a = -1;
        *b = 1;
    } else {
        *a = 1;
        *b = 1;
    }
}

// inp: &[uchar],
// bp: &[uchar],
fn susan_edges_small(
    inp: HeapSlice,
    r: HeapSlice,
    mid: HeapSlice,
    bp: HeapSlice,
    max_no: i32,
    x_size: u16,
    y_size: u16,
    heap: &mut SimulatedHeap
) {
    let mut pattern = [
        (3, x_size - 2), (1, 2),
        (1, x_size -2), (3, 0),
    ];
    // loopbound min 0 max 0 -> outer
    // loopbound min 0 max 0 -> inner
    susan_main_base(
        inp, r, bp, x_size, y_size,
        max_no, 1, 1, &mut pattern, heap
    );

    let mut z: f32 = 0.;
    let x_usize = x_size as usize;
    let y_usize = y_size as usize;
    let inp_start = inp.start as usize;
    // loopbound min 0 max 0
    for i in 2..(y_usize - 2) {
        // loopbound min 0 max 0
        for j in 2..(x_usize - 2) {
        if heap.return_slice_i8(r)[ i * x_usize + j ] > 0 {
            let m = heap.return_slice_i8(r)[ i * x_usize + j ];
            let n = max_no - m as i32;
            let cp_index = heap.heap_content[inp_start + i * x_usize + j] as u16;

            let do_symmetry: bool;
            if n > 250 {
                let mut x: i32 = 0;
                let mut y: i32 = 0;

                let weights: [&[&[i32]]; 4] = [
                    &[&[-1, -1], &[0, -1], &[1, -1]],
                    &[&[-1, 0]],
                    &[&[1, 0]],
                    &[&[-1, 1], &[0, 1], &[1, 1]]
                ];
                let mut accs = [&mut x, &mut y];
                let pattern = [
                  (weights[0], x_size - 2),
                  (weights[1], 2),
                  (weights[2], x_size - 2),
                  (weights[3], 0),
                ];
                susan_cornerness_base(
                    &bp, cp_index, &inp, x_size, i as u16, j as u16, 1, &pattern, &mut accs, heap,
                );

                z = susan_sqrtf((x * x + y * y) as f32);
                let mut a: i32 = 0;
                let mut b: i32 = 0;
                if z > (0.4 * n as f32) {
                    /* 0.6 */
                    do_symmetry = false;
                    susan_edges_non_symmetry(
                        x, y, &mut a, &mut b, &mut z
                    );
                    let cond1 = m > heap.return_slice_i8(r)[((i as i32 + a) * x_size as i32 + j as i32 + b) as usize];
                    let cond2 = m >= heap.return_slice_i8(r)[((i as i32 - a) * x_size as i32 + j as i32 - b) as usize];
                    if cond1 && cond2 {
                        heap.return_slice(mid)[i * x_usize + j] = 1;
                    }
                } else {
                    do_symmetry = true;
                }
            } else {
                do_symmetry = true;
            }

            if do_symmetry {
                let mut x: i32 = 0;
                let mut y: i32 = 0;
                let mut w: i32 = 0;

                /*   |      \
                    y  -x-  w
                    |        \   */

                let weights: [&[&[i32]]; 4] = [
                    &[&[1, 1, 1], &[0, 1, 0], &[1, 1, -1]],
                    &[&[1, 0, 0]],
                    &[&[1, 0, 0]],
                    &[&[1, 1, -1], &[0, 1, 0], &[1, 1, 1]],
                ];
                let mut accs = [&mut x, &mut y, &mut w];
                let pattern = [
                  (weights[0], x_size - 2),
                  (weights[1], 2),
                  (weights[2], x_size - 2),
                  (weights[3], 0),
                ];
                susan_cornerness_base(
                    &bp, cp_index, &inp, x_size, i as u16, j as u16, 1, &pattern, &mut accs, heap,
                );
                let mut a: i32 = 0;
                let mut b: i32 = 0;
                susan_edges_symmetry(
                    x, y, &mut a, &mut b, &mut z, w
                );
                let cond1 = m > heap.return_slice_i8(r)[((i as i32 + a) * x_size as i32 + j as i32 + b) as usize];
                let cond2 = m >= heap.return_slice_i8(r)[((i as i32 - a) * x_size as i32 + j as i32 - b) as usize];
                if cond1 && cond2 {
                    heap.return_slice(mid)[i * x_usize + j] = 2;
                }
            }
        }
        }
    }
}

// inp: &mut [uchar],
fn susan_corner_draw(
    inp: HeapSlice,
    corner_list: &CornerListT,
    x_size: u16,
    drawing_mode: DrawingMode,
    heap: &mut SimulatedHeap
) {
    let inp_start = inp.start as usize;
    // loopbound min 0 max 0
    while let Some(corner) = corner_list.iter().next() {
        let mut p_index: usize;
        if corner.info == 7 { break; }
        else if drawing_mode == DrawingMode::Smooth {
            p_index = ((corner.y - 1) * x_size as i8 + corner.x - 1) as usize;
            heap.heap_content[inp_start + p_index] = 255;
            p_index += 1;
            heap.heap_content[inp_start + p_index] = 255;
            p_index += 1;
            heap.heap_content[inp_start + p_index] = 255;
            p_index += (x_size - 2) as usize;
            heap.heap_content[inp_start + p_index] = 255;
            p_index += 1;
            heap.heap_content[inp_start + p_index] = 0;
            p_index += 1;
            heap.heap_content[inp_start + p_index] = 255;
            p_index += (x_size - 2) as usize;
            heap.heap_content[inp_start + p_index] = 255;
            p_index += 1;
            heap.heap_content[inp_start + p_index] = 255;
            p_index += 1;
            heap.heap_content[inp_start + p_index] = 255;
        } else {
            p_index = (corner.y * x_size as i8 + corner.x) as usize;
            heap.heap_content[inp_start + p_index] = 0;
        }
    }
}

// cp: &[uchar],
// p: &[uchar],
fn susan_if_step(
    acc: &mut i32,
    cp: HeapSlice,
    cp_index: u16,
    p: HeapSlice,
    p_index: &mut u16,
    max_no: i32,
    p_offset: Option<u16>,
    inc: bool,
    heap: &mut SimulatedHeap,
) -> bool
{
    if *acc >= max_no { return false; }
    if let Some(off) = p_offset { *p_index += off; }
    *acc += heap.heap_content[(cp.start + cp_index - heap.heap_content[(p.start + *p_index) as usize] as u16) as usize] as i32;
    if inc { *p_index += 1; }
    return true;
}

const FIVE_SUPP: &[(isize, isize, bool)] = &[
    (-1,  2, true), ( 0,  1, true), ( 0,  2, true),
    ( 1, -1, true), ( 1,  0, true), ( 1,  1, true),
    ( 1,  2, true), ( 2, -2, true), ( 2, -1, true),
    ( 2,  0, true), ( 2,  1, true), ( 2,  2, true),

    (-2, -2, false), (-2, -1, false), (-2,  0, false), (-2,  1, false),
    (-2,  2, false), (-1, -2, false), (-1, -1, false), (-1,  0, false),
    (-1,  1, false), ( 0, -2, false), ( 0, -1, false), ( 1, -2, false),
];

const SEVEN_SUPP: &[(isize, isize, bool)] = &[
    (-3, -3, true), (-3, -2, true), (-3, -1, true), (-3,  0, true),
    (-3,  1, true), (-3,  2, true), (-3,  3, true),
    (-2, -3, true), (-2, -2, true), (-2, -1, true), (-2,  0, true),
    (-2,  1, true), (-2,  2, true), (-2,  3, true),
    (-1, -3, true), (-1, -2, true), (-1, -1, true), (-1,  0, true),
    (-1,  1, true), (-1,  2, true), (-1,  3, true),

     (0, -3, false), (0, -2, false), (0, -1, false),
     (0,  1, false), (0,  2, false), (0,  3, false),
     (1, -3, false), (1, -2, false), (1, -1, false), (1,  0, false),
     (1,  1, false), (1,  2, false), (1,  3, false),
     (2, -3, false), (2, -2, false), (2, -1, false), (2,  0, false),
     (2,  1, false), (2,  2, false), (2,  3, false),
     (3, -3, false), (3, -2, false), (3, -1, false), (3,  0, false),
     (3,  1, false), (3,  2, false), (3,  3, false),
];


#[inline]
fn is_local_max(
    r: &[i8],
    x_size: u16,
    y_size: u16,
    i: u16,
    j: u16,
    mask: &[(isize, isize, bool)],
) -> bool {
    let center = r[(i * x_size + j) as usize];

    for &(di, dj, strict) in mask {
        let ni = i as isize + di;
        let nj = j as isize + dj;

        if ni < 0 || nj < 0 || ni >= y_size as isize || nj >= x_size as isize {
            continue;
        }

        let neighbor = r[ni as usize * x_size as usize + nj as usize];

        if strict {
            if !(center > neighbor) {
                return false;
            }
        } else {
            if !(center >= neighbor) {
                return false;
            }
        }
    }
    true
}

// inp: &[uchar],
// r: &mut [i8],
// bp: &[uchar],
fn susan_corners_quick(
    inp: HeapSlice,
    r: HeapSlice,
    bp: HeapSlice,
    max_no: i32,
    corner_list: &mut CornerListT,
    x_size: u16,
    y_size: u16,
    mask_mode: bool,
    heap: &mut SimulatedHeap,
) {
    memset(heap.return_slice_i8(r), 0);

    let x_usize = x_size as usize;
    let y_usize = y_size as usize;
    let inp_start = inp.start as usize;
    let r_start = r.start as usize;
     // loopbound min 0 max 0
    for i in 7..(y_usize - 7) {
        // loopbound min 0 max 0
        for j in 7..(x_usize - 7) {
            let mut n: i32 = 100;
            let mut p_index = ((i - 3) * x_usize + j - 1) as u16;
            let cp_index = heap.heap_content[inp_start + i * x_usize + j] as u16;

            susan_acc(
              &mut n, bp, cp_index, inp, &mut p_index, 3, heap
            );
            p_index += x_size - 3;

            susan_acc(
              &mut n, bp, cp_index, inp, &mut p_index, 5, heap
            );
            p_index += x_size - 5;

            susan_acc(
              &mut n, bp, cp_index, inp, &mut p_index, 7, heap
            );
            p_index += x_size - 6;

            susan_acc(
              &mut n, bp, cp_index, inp, &mut p_index, 3, heap
            );
            let table = [
                (2, true), (0, true), (0, false), (x_size - 6, true),
                (0, true), (0, true), (0, true), (0, true), (0, true),
                (0, false), (5, true), (0, true), (0, true), (0, true),
                (0, false), (x_size - 3, true), (0, true), (0, false)
            ];
            for (p_offset, inc) in table.iter() {
                let cond: bool = susan_if_step(
                    &mut n, bp, cp_index, inp, &mut p_index, max_no, Some(*p_offset), *inc, heap
                );
                if cond { break; }
            }
            if n < max_no {
                heap.heap_content[r_start + i * x_usize + j] = (max_no - n) as u8;
            }
        }
    }
    /* to locate the local maxima */
    let mut n: usize = 0;
    let mask = if mask_mode { FIVE_SUPP } else { SEVEN_SUPP };
    // loopbound min 0 max 0
    for i in 7..(y_usize - 7) {
        // loopbound min 0 max 0
        for j in 7..(x_usize - 7) {
            let x = heap.heap_content[r_start + i * x_usize + j] as i8;
            if x <= 0 {
                continue;
            }
            if is_local_max(
                heap.return_slice_i8(r), x_size, y_size, i as u16, j as u16, mask
            ) {
                corner_list[n].info = 0;
                corner_list[n].x = j as i8;
                corner_list[n].y = i as i8;
                let x = (-2isize..=2isize)
                    .flat_map(|di| (-2isize..=2isize).map(move |dj| (di, dj)))
                    .map(|(di, dj)| {
                        let ni = (i as isize + di) as usize;
                        let nj = (j as isize + dj) as usize;
                        heap.heap_content[inp_start + ni * x_usize + nj] as i32
                    })
                    .sum::<i32>();

                corner_list[n].i = (x / 25) as i8;
                /*corner_list[ n ].I=in[ i*x_size+j ];*/
                let sum_col = |dj: isize| -> i32 {
                    (-2..=2)
                        .map(|di| {
                            let ii = i as isize + di;
                            let jj = j as isize + dj;
                            heap.heap_content[inp_start + (ii as usize) * x_usize + (jj as usize)] as i32
                        })
                        .sum()
                };
                let sum_row = |di: isize| -> i32 {
                    (-2..=2)
                        .map(|dj| {
                            let ii = i as isize + di;
                            let jj = j as isize + dj;
                            heap.heap_content[inp_start + (ii as usize) * x_usize + (jj as usize)] as i32
                        })
                        .sum()
                };
                let mut x = sum_col(2) - sum_col(-2);
                x += x + sum_col(1) - sum_col(-1);

                let mut y = sum_row(2) - sum_row(-2);
                y += y + sum_row(1) - sum_row(-1);
                corner_list[n].dx = (x / 15) as i8;
                corner_list[n].dy = (y / 15) as i8;
                n += 1;
                if n as i32 == MAX_CORNERS { /*  "Too many corners." */ }
            }
        }
    }
    corner_list[n].info = 7;
}

fn susan_call_susan(
    fd: &mut File,
    data: &mut Data,
    corner_list: &mut CornerListT,
    config: &mut Config,
    heap: &mut SimulatedHeap,
) {
    susan_get_image(
        fd, &mut data.inp, &mut data.x_size, &mut data.y_size, heap
    );
    if config.dt < 0. { config.three_by_three = true; }
    if config.principle_conf && (config.drawing_mode == DrawingMode::Smooth) {
        config.drawing_mode = DrawingMode::Edges;
    }

    match config.drawing_mode {
        DrawingMode::Smooth => {
            /* {{{ smoothing */

            susan_setup_brightness_lut(
                &mut data.bp, config.bt, 2, heap
            );
            susan_smoothing(
                config.three_by_three, &mut data.inp, config.dt, &mut data.x_size, &mut data.y_size, data.bp, heap
            );
        },
        DrawingMode::Edges => {
            /* {{{ edges */

            data.r = heap.malloc(data.x_size * data.y_size, size_of::<i8>() as u8);
            susan_setup_brightness_lut(
                &mut data.bp, config.bt, 6, heap
            );

            if config.principle_conf {
                if config.three_by_three {
                    susan_principle_small(
                        data.inp, data.r, data.bp, config.max_no_edges,
                        data.x_size, data.y_size, heap
                    );
                } else {
                    susan_principle(
                        data.inp, data.r, data.bp, config.max_no_edges,
                        data.x_size, data.y_size, heap
                    );
                }
                susan_int_to_uchar(
                    data.r, data.inp, heap
                );
            } else {
                data.mid = heap.malloc(data.x_size * data.y_size, size_of::<u8>() as u8);
                {
                    memset(heap.return_slice(data.mid), 100); /* note not set to zero */
                }

                if config.three_by_three {
                    susan_edges_small(
                        data.inp, data.r, data.mid, data.bp,
                        config.max_no_edges, data.x_size, data.y_size, heap
                    );
                }
                else {
                    susan_edges(
                        data.inp, data.r, data.mid, data.bp,
                        config.max_no_edges, data.x_size, data.y_size, heap
                    );
                }
                if config.thin_post_proc {
                    susan_thin(
                        data.r, data.mid, data.x_size, data.y_size, heap,
                    );
                }
                let mid_start = 0;
                susan_edge_draw(
                    data.inp, data.mid, mid_start, data.x_size, data.y_size, config.drawing_mode, heap
                );
            }
        },
        DrawingMode::Corners => {
            /* {{{ corners */

            susan_setup_brightness_lut(
                &mut data.bp, config.bt, 6, heap
            );
            
            data.r = heap.malloc(data.x_size * data.y_size, size_of::<i8>() as u8);
            if config.principle_conf {
                susan_principle(
                    data.inp, data.r, data.bp, config.max_no_corners, data.x_size, data.y_size, heap
                );
                susan_int_to_uchar(
                    data.r, data.inp, heap
                );
            } else {
                if config.quick {
                    susan_corners_quick(
                        data.inp, data.r, data.bp, config.max_no_corners,
                        corner_list, data.x_size, data.y_size, config.mask_mode, heap
                    );
                } else {
                    susan_corners(
                        data.inp, data.r, data.bp, config.max_no_corners,
                        corner_list, data.x_size, data.y_size, config.mask_mode, heap
                    );
                }
                susan_corner_draw(
                    data.inp, corner_list, data.x_size, config.drawing_mode, heap
                );
            }
        },
    }
    susan_put_image(data.x_size, data.y_size);
}

// volatile additions handled through black_box call
fn susan_init(
    fd: &mut File<'static>,
    config: &mut Config,
    heap: &mut SimulatedHeap,
) {
    fd.data = &SUSAN_INPUT;
    fd.cur_pos = 0;

    heap.reset();

    config.dt = 4.0;
    config.bt = 20;
    config.principle_conf = false;
    config.thin_post_proc = true;
    config.three_by_three = false;
    config.drawing_mode = DrawingMode::Smooth;
    config.quick = false;
    config.max_no_corners = 50;
    config.max_no_edges = 50;
    config.mask_mode = false;   // true: FIVE_SUPP | false: SEVEN_SUPP

    // mode=0; /* Smoothing mode */
    // mode=1; /* Edges mode */
    // mode=2; /* Corners mode */

    // principle=1; /* Output initial enhancement image only; corners or edges mode (default is edges mode) */
    // thin_post_proc=0; /* No post-processing on the binary edge map (runs much faster); edges mode */
    // drawing_mode=1; /* Mark corners/edges with single black points instead of black with white border; corners or edges mode */
    // three_by_three=1; /* Use flat 3x3 mask, edges or smoothing mode */
    // susan_quick=1; /* Use faster (and usually stabler) corner mode; edge-like corner suppression not carried out; corners mode */
    // dt=10.0; /* Distance threshold, smoothing mode, (default=4) */
    // bt=50; /* Brightness threshold, all modes, (default=20) */
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn susan_main(
    fd: &mut File,
    data: &mut Data,
    corner_list: &mut CornerListT,
    config: &mut Config,
    heap: &mut SimulatedHeap,
) -> i32
{
    susan_call_susan(
        fd, data, corner_list, config, heap
    );
    heap.free_all();
    susan_call_susan(
        fd, data, corner_list, config, heap
    );
    heap.free_all();
    susan_call_susan(
        fd, data, corner_list, config, heap
    );
    heap.free_all();
    let checksum = 0;
    checksum
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn susan_log(
    fd: &mut File<'static>,
    data: &mut Data,
    corner_list: &mut CornerListT,
    config: &mut Config,
    heap: &mut SimulatedHeap,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: i32 = 0;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        susan_init(
            black_box(fd), black_box(config), black_box(heap),
        );
        let start = black_box(DWT::cycle_count());
        result = black_box(susan_main(
            black_box(fd),
            black_box(data),
            black_box(corner_list),
            black_box(config),
            black_box(heap),
        ));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("susan - elapsed: {}", cycles);
    });

    result == 0
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn susan_std(
    fd: &mut File<'static>,
    data: &mut Data,
    corner_list: &mut CornerListT,
    config: &mut Config,
    heap: &mut SimulatedHeap,
) -> bool {
    susan_init(
        black_box(fd), black_box(config), black_box(heap),
    );
    let result = black_box(susan_main(
        black_box(fd),
        black_box(data),
        black_box(corner_list),
        black_box(config),
        black_box(heap),
    ));
    result == 0
}

fn susan() -> bool {
    static mut FILE: File<'static> = File::new(&SUSAN_INPUT);
    static mut DATA: Data = Data::new();
    static mut CORNER_LIST: CornerListT = [CornerRep::new(); MAX_CORNERS as usize];
    static mut CONFIG: Config = Config::new();
    static mut HEAP: SimulatedHeap = SimulatedHeap::new();

    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let fd = unsafe { &mut *{ &raw mut FILE } };
    let data = unsafe { &mut *{ &raw mut DATA } };
    let corner_list = unsafe { &mut *{ &raw mut CORNER_LIST } };
    let config = unsafe { &mut *{ &raw mut CONFIG } };
    let heap = unsafe { &mut *{ &raw mut HEAP } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(susan_log(
            black_box(fd),
            black_box(data),
            black_box(corner_list),
            black_box(config),
            black_box(heap),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(susan_std(
            black_box(fd),
            black_box(data),
            black_box(corner_list),
            black_box(config),
            black_box(heap),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(susan());
    
    #[cfg(feature = "log")]
    {
        rprintln!("susan: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(susan());

    #[cfg(feature = "log")]
    std::println!("susan: {}", success);

    std::process::exit(!success as i32);
}

// TODO: GANZ WICHTIG -> HeapSlices zu &mut
