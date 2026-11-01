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

  Name: audiobeamlibm.c

  Author: Ian Lance Taylor and J.T. Conklin

  Function: IEEE754 software library routines.

  Source: Sun Microsystems and Cygnus

  Original name: Unknown

  Changes: No major functional changes.

  License: See the terms below.

*/

/*
   ====================================================
   Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.

   Developed at SunPro, a Sun Microsystems, Inc. business.
   Permission to use, copy, modify, and distribute this
   software is freely granted, provided that this notice
   is preserved.
   ====================================================
*/

static NPIO2_HW: [i32; 32] = [
    0x3fc90f00, 0x40490f00, 0x4096cb00, 0x40c90f00, 0x40fb5300, 0x4116cb00, 0x412fed00, 0x41490f00,
    0x41623100, 0x417b5300, 0x418a3a00, 0x4196cb00, 0x41a35c00, 0x41afed00, 0x41bc7e00, 0x41c90f00,
    0x41d5a000, 0x41e23100, 0x41eec200, 0x41fb5300, 0x4203f200, 0x420a3a00, 0x42108300, 0x4216cb00,
    0x421d1400, 0x42235c00, 0x4229a500, 0x422fed00, 0x42363600, 0x423c7e00, 0x4242c700, 0x42490f00,
];

const ONE: f32 = 1.;
const HUGE: f32 = 1.0e30;
const TINY: f32 = 1.0e-30;
pub const PI: f32 = f32::from_bits(0x40490fda); // 3.1415925026e+00f

const HALF: f32 = f32::from_bits(0x3f000000); // 5.0000000000e-01

const INVPIO2: f32 = f32::from_bits(0x3f22f984); // 6.3661980629e-01f
const PIO2_1: f32 = f32::from_bits(0x3fc90f80); // 1.5707855225e+00f
const PIO2_1T: f32 = f32::from_bits(0x37354443); // 1.0804334124e-05f
const PIO2_2: f32 = f32::from_bits(0x37354400); // 1.0804273188e-05f
const PIO2_2T: f32 = f32::from_bits(0x2e85a308); // 6.0770999344e-11f
const PIO2_3: f32 = f32::from_bits(0x2e85a300); // 6.0770943833e-11f
const PIO2_3T: f32 = f32::from_bits(0x248d3132); // 6.1232342629e-17f

// kernel_cosf specific
const C1: f32 = f32::from_bits(0x3d2aaaab); //  4.1666667908e-02f
const C2: f32 = f32::from_bits(0xbab60b61); // -1.3888889225e-03f
const C3: f32 = f32::from_bits(0x37d00d01); //  2.4801587642e-05f
const C4: f32 = f32::from_bits(0xb493f27c); // -2.7557314297e-07f
const C5: f32 = f32::from_bits(0x310f74f6); //  2.0875723372e-09f
const C6: f32 = f32::from_bits(0xad47d74e); // -1.1359647598e-11f

// kernel_sinf specific
const S1: f32 = f32::from_bits(0xbe2aaaab); // -1.6666667163e-01f
const S2: f32 = f32::from_bits(0x3c088889); //  8.3333337680e-03f
const S3: f32 = f32::from_bits(0xb9500d01); // -1.9841270114e-04f
const S4: f32 = f32::from_bits(0x3638ef1b); //  2.7557314297e-06f
const S5: f32 = f32::from_bits(0xb2d72f34); // -2.5050759689e-08f
const S6: f32 = f32::from_bits(0x2f2ec9d3); //  1.5896910177e-10f

const _TWO25: f32 = f32::from_bits(0x4c000000); // 3.355443200e+07f
const _TWOM25: f32 = f32::from_bits(0x33000000); // 2.9802322388e-08f/

fn rem_pio2f(x: f32, y: &mut [f32; 2]) -> i32 {
    let hx = x.to_bits() as i32;
    let ix = hx & 0x7fffffff;
    if ix <= 0x3f490fd8 {
        /* |x| ~<= pi/4 , no need for reduction */
        y[0] = x;
        y[1] = 0.;
        return 0;
    }
    if ix < 0x4016cbe4 {
        /* |x| < 3pi/4, special case with n=+-1 */
        if hx > 0 {
            let mut z = x - PIO2_1;
            if (ix & 0xfffffff0u32 as i32) != 0x3fc90fd0 {
                /* 24+24 bit pi OK */
                y[0] = z - PIO2_1T;
                y[1] = (z - y[0]) - PIO2_1T;
            } else {
                /* near pi/2, use 24+24+24 bit pi */
                z -= PIO2_2;
                y[0] = z - PIO2_2T;
                y[1] = (z - y[0]) - PIO2_2T;
            }
            return 1;
        } else {
            /* negative x */
            let mut z = x + PIO2_1;
            if (ix & 0xfffffff0u32 as i32) != 0x3fc90fd0 {
                /* 24+24 bit pi OK */
                y[0] = z + PIO2_1T;
                y[1] = (z - y[0]) + PIO2_1T;
            } else {
                /* near pi/2, use 24+24+24 bit pi */
                z += PIO2_2;
                y[0] = z + PIO2_2T;
                y[1] = (z - y[0]) + PIO2_2T;
            }
            return -1;
        }
    }
    if ix <= 0x43490f80 {
        /* |x| ~<= 2^7*(pi/2), medium size */
        let mut t = x.abs();
        let n = (t * INVPIO2 + HALF) as i32;
        let f_n = n as f32;
        let mut r = t - f_n * PIO2_1;
        let mut w = f_n * PIO2_1T; /* 1st round good to 40 bit */
        if n < 32 && (ix & 0xffffff00u32 as i32) != NPIO2_HW[(n - 1) as usize] {
            y[0] = r - w; /* quick check no cancellation */
        } else {
            let j = ix >> 23;
            y[0] = r - w;
            let high = y[0].to_bits() as i32;
            let mut i = j - ((high >> 23) & 0xff);
            if i > 8 {
                /* 2nd iteration needed, good to 57 */
                t = r;
                w = f_n * PIO2_2;
                r = t - w;
                w = f_n * PIO2_2T - ((t - r) - w);
                y[0] = r - w;
                let high = y[0].to_bits() as i32;
                i = j - ((high >> 23) & 0xff);
                if i > 25 {
                    /* 3rd iteration need, 74 bits acc */
                    t = r; /* will cover all possible cases */
                    w = f_n * PIO2_3;
                    r = t - w;
                    w = f_n * PIO2_3T - ((t - r) - w);
                    y[0] = r - w;
                }
            }
        }
        y[1] = (r - y[0]) - w;
        if hx < 0 {
            y[0] = -y[0];
            y[1] = -y[1];
            return -n;
        } else {
            return n;
        };
    }
    /*
       all other (large) arguments
    */
    if ix >= 0x7f800000 {
        /* x is inf or NaN */
        #[allow(clippy::eq_op)] // let's follow the c source
        {
            y[0] = x - x;
            y[1] = x - x;
        }
        return 0;
    }

    // This will never happen in basicmath_small, because
    // up to this point we have already returned a value
    // for each of the possible inputs
    #[allow(clippy::eq_op)] // let's follow the c source
    {
        y[0] = x - x; /* dummy initialization */
        y[1] = x - x;
    }

    0
}

fn kernel_cosf(x: f32, y: f32) -> f32 {
    let mut ix = x.to_bits();
    ix &= 0x7fffffff;

    if ix < 0x32000000 {
        /* if x < 2**27 */
        if (x as i32) == 0 {
            return ONE; /* generate inexact */
        }
    }
    let z = x * x;
    let r = z * (C1 + z * (C2 + z * (C3 + z * (C4 + z * (C5 + z * C6)))));
    /* if |x| < 0.3 */
    if ix < 0x3e99999a {
        ONE - (0.5 * z - (z * r - x * y))
    } else {
        /* x > 0.78125 */
        let qx = if ix > 0x3f480000 {
            0.28125
        } else {
            f32::from_bits(ix - 0x01000000) /* x/4 */
        };
        let hz = 0.5 * z - qx;
        let a = ONE - qx;
        a - (hz - (z * r - x * y))
    }
}

fn kernel_sinf(x: f32, y: f32, iy: i32) -> f32 {
    let mut ix = x.to_bits();
    ix &= 0x7fffffff; /* high word of x */
    if ix < 0x32000000 {
        /* |x| < 2**-27 */
        if (x as i32) == 0 {
            return x; /* generate inexact */
        }
    }
    let z = x * x;
    let v = z * x;
    let r = S2 + z * (S3 + z * (S4 + z * (S5 + z * S6)));
    if iy == 0 {
        x + v * (S1 + z * r)
    } else {
        x - ((z * (HALF * y - v * r) - y) - v * S1)
    }
}

fn _copysignf(x: f32, y: f32) -> f32 {
    let ix = x.to_bits();
    let iy = y.to_bits();
    let res = (ix & 0x7fffffff) | (iy & 0x80000000);
    f32::from_bits(res)
}

pub(super) fn cosf(x: f32) -> f32 {
    let mut y = [0., 0.];
    let z = 0.0;

    let mut ix = x.to_bits() as i32;

    /* |x| ~< pi/4 */
    ix &= 0x7fffffff;
    if ix <= 0x3f490fd8 {
        kernel_cosf(x, z)
    } else {
        /* cos(Inf or NaN) is NaN */
        if ix >= 0x7f800000 {
            // let's follow the c source
            #[allow(clippy::eq_op)]
            {
                x - x
            }
        } else {
            /* argument reduction needed */
            let n = rem_pio2f(x, &mut y);

            match n & 3 {
                0 => kernel_cosf(y[0], y[1]),
                1 => -kernel_sinf(y[0], y[1], 1),
                2 => -kernel_cosf(y[0], y[1]),
                _ => kernel_sinf(y[0], y[1], 1),
            }
        }
    }
}

fn _fabsf(x: f32) -> f32 {
    let ix = x.to_bits() as i32;
    f32::from_bits((ix & 0x7fffffff) as u32)
}

pub(super) fn floorf(x: f32) -> f32 {
    let mut i0: i64 = x.to_bits() as i64;
    let j0 = ((i0 >> 23) & 0xff) - 0x7f;
    if j0 < 23  {
        if j0 < 0 {
            if HUGE + x > 0.0f32 {
                if i0 >= 0 {
                    i0 = 0;
                } else {
                    if (i0 & 0x7fffffff) != 0 {
                        i0 = 0xbf800000;
                    }
                }
            }
        } else {
            let i: u32 = (0x007fffff >> j0) as u32;
            if (i0 as u32 & i) == 0 { return x; }
            if HUGE + x > 0.0f32 {
                if i0 < 0 {
                    i0 += 0x00800000 >> j0;
                }
                i0 &= !i as i64;
            }
        }
    } else {
        if j0 == 0x80 {
            return x + x;
        } else {
            return x;
        }
    }
    f32::from_bits(i0 as u32)
}

fn _isinff(x: f32) -> bool {
    let ix = x.to_bits() as i32;
    let mut t = ix & 0x7fffffff;
    t ^= 0x7f800000;
    t |= -t;
    !(t >> 31) & (ix >> 30) != 0
}

fn _scalbnf(mut x: f32, n: i32) -> f32 {
    let mut ix = x.to_bits() as i32;
    let mut k = ( ix & 0x7f800000 ) >> 23; /* extract exponent */
    /* 0 or subnormal x */
    if k == 0 {
        if (ix & 0x7fffffff) == 0 {
            return x; /* +-0 */
        }
        x *= _TWO25;
        ix = x.to_bits() as i32;
        k = ((ix & 0x7f800000) >> 23) - 25;
    }
    if k == 0xff {
        return x + x; /* NaN or Inf */
    }
    k += n;
    if (n > 50000) || (k > 0xfe) {
        return HUGE * _copysignf(HUGE, x); /* overflow  */
    }
    if n < -50000 {
        return TINY * _copysignf(TINY, x); /* underflow */
    }
    /* normal result */
    if k > 0 {
        let res = (ix & 0x807fffffu32 as i32) | (k << 23);
        return f32::from_bits(res as u32);
    } 
    if k <= -25 {
        return TINY * _copysignf(TINY, x); /* underflow */
    }
    /* subnormal result */
    k += 25;
    let res = (ix & 0x807fffffu32 as i32) | (k << 23); 
    f32::from_bits(((res as f32) * _TWOM25) as u32)
}

pub(super) fn ceilf(x: f32) -> f32 {
    let mut i0 = x.to_bits() as i64;  
    let j0 = ((i0 >> 23) & 0xff) - 0x7f;
    if j0 < 23 {
        if j0 < 0 {
            if HUGE + x > 0.0f32 {
                if i0 < 0 {
                    i0 = 0x80000000;
                }
                else {
                    if i0 != 0 {
                        i0 = 0x3f800000;
                    }
                }
            }
        } else {
            let i: u32 = 0x007fffff >> j0;
            if (i0 as u32 & i) == 0 { return x; } 
            if HUGE + x > 0.0f32 {
                if i0 > 0 {
                    i0 += 0x00800000 >> j0;
                }
                i0 &= !i as i64;
            }
        }
    } else {
        if j0 == 0x80 {
            return x + x;
        } else {
            return x;
        }
    }
    f32::from_bits(i0 as u32)
}

pub(super) fn sqrtf(x: f32) -> f32 {
    let sign: i32 = 0x80000000u32 as i32;
    let mut ix = x.to_bits() as i32;

    /* take care of Inf and NaN */
    if (ix & 0x7f800000) == 0x7f800000 {
        return x * x + x; /* sqrt(NaN)=NaN, sqrt(+inf)=+inf, sqrt(-inf)=sNaN */
    }
    /* take care of zero */
    if ix <= 0 {
        if (ix & (!sign)) == 0 {
            return x; /* sqrt(+-0) = +-0 */
        } else {
            if ix < 0 {
               #[allow(clippy::eq_op)] // let's follow the c source
                {
                   return (x - x) / (x - x); /* sqrt(-ve) = sNaN */
                } 
            }
        }
        
    }
    /* normalize x */
    let mut m: i32 = ix >> 23;
    /* subnormal x */
    if m == 0 {
        // loopbound min 0 max 0
        let mut i = 0;
        while ix & 0x00800000 == 0 {
            ix <<= 1;
            i += 1;
        }
        m -= i - 1;
    }
    m -= 127; /* unbias exponent */
    ix = ( ix & 0x007fffff ) | 0x00800000;
    /* odd m, double x to make it even */
    if (m & 1) != 0 {
        ix += ix;
    }
    m >>= 1; /* m = [m/2] */

    /* generate sqrt(x) bit by bit */
    ix += ix;
    let mut q: i32 = 0; /* q = sqrt(x) */
    let mut s: i32 = 0;
    let mut r: u32 = 0x01000000; /* r = moving bit from right to left */

    // loopbound min 25 max 25
    while r != 0 {
        let t: i32 = s + r as i32;
        if t <= ix {
            s = t + r as i32;
            ix -= t;
            q += r as i32;
        }
        ix += ix;
        r >>= 1;
    }
    /* use floating add to find out rounding direction */
    if ix != 0 {
        let mut z: f32 = ONE - TINY; /* trigger inexact flag */
        if z >= ONE {
            z = ONE + TINY;
            if z > ONE {
                q += 2;
            } else {
                q += q & 1;
            }
        }
    }
    ix = (q >> 1) + 0x3f000000;
    ix += m << 23;
    f32::from_bits(ix as u32)
}