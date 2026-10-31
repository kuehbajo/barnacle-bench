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

  Name: quicksortlibm.c

  Author: Ian Lance Taylor

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

#![allow(clippy::excessive_precision)]
#![allow(clippy::collapsible_if, clippy::collapsible_else_if)]

const ZERO: f32 = 0.;
const ONE: f32 = 1.;
const TWO: f32 = 2.;
const HUGE: f32 = 1.0e30;
const TINY: f32 = 1.0e-30;
const TWO24: f32 = f32::from_bits(0x4b800000); // 16777216.0

fn isinff(x: f32) -> bool {
    let ix = x.to_bits() as i32;
    let mut t = ix & 0x7fffffff;
    t ^= 0x7f800000;
    t |= -t;
    !(t >> 31) & (ix >> 30) != 0
}

fn copysignf(x: f32, y: f32) -> f32 {
    let ix = x.to_bits();
    let iy = y.to_bits();
    let res = (ix & 0x7fffffff) | (iy & 0x80000000);
    f32::from_bits(res)
}

const TWO25: f32 = f32::from_bits(0x4c000000); // 3.355443200e+07f
const TWOM25: f32 = f32::from_bits(0x33000000); // 2.9802322388e-08f

fn scalbnf(mut x: f32, n: i32) -> f32 {
    let mut ix = x.to_bits() as i32;
    let mut k = ( ix & 0x7f800000 ) >> 23; /* extract exponent */
    /* 0 or subnormal x */
    if k == 0 {
        if (ix & 0x7fffffff) == 0 {
            return x; /* +-0 */
        }
        x *= TWO25;
        ix = x.to_bits() as i32;
        k = ((ix & 0x7f800000) >> 23) - 25;
    }
    if k == 0xff {
        return x + x; /* NaN or Inf */
    }
    k += n;
    if (n > 50000) || (k > 0xfe) {
        return HUGE * copysignf(HUGE, x); /* overflow  */
    }
    if n < -50000 {
        return TINY * copysignf(TINY, x); /* underflow */
    }
    /* normal result */
    if k > 0 {
        let res = (ix & 0x807fffffu32 as i32) | (k << 23);
        return f32::from_bits(res as u32);
    } 
    if k <= -25 {
        return TINY * copysignf(TINY, x); /* underflow */
    }
    /* subnormal result */
    k += 25;
    let res = (ix & 0x807fffffu32 as i32) | (k << 23); 
    f32::from_bits(((res as f32) * TWOM25) as u32)
}

const BP: [f32; 2] = [1.0f32, 1.5f32];
const DP_H: [f32; 2] = [0.0f32, f32::from_bits(0x3f15c000)]; // 5.84960938e-01f
const DP_L: [f32; 2] = [0.0f32, f32::from_bits(0x35d1cfdc)]; // 1.56322085e-06f
/* poly coefs for (3/2)*(log(x)-2s-2/3*s**3 */
const L1: f32 = f32::from_bits(0x3f19999a); // 6.0000002384e-01f
const L2: f32 = f32::from_bits(0x3edb6db7); // 4.2857143283e-01f
const L3: f32 = f32::from_bits(0x3eaaaaab); // 3.3333334327e-01f
const L4: f32 = f32::from_bits(0x3e8ba305); // 2.7272811532e-01f
const L5: f32 = f32::from_bits(0x3e6c3255); // 2.3066075146e-01f
const L6: f32 = f32::from_bits(0x3e53f142); // 2.0697501302e-01f
const P1: f32 = f32::from_bits(0x3e2aaaab); // 1.6666667163e-01f
const P2: f32 = f32::from_bits(0xbb360b61); // -2.7777778450e-03f
const P3: f32 = f32::from_bits(0x388ab355); // 6.6137559770e-05f
const P4: f32 = f32::from_bits(0xb5ddea0e); // -1.6533901999e-06f
const P5: f32 = f32::from_bits(0x3331bb4c); // 4.1381369442e-08f
const LG2: f32 = f32::from_bits(0x3f317218); // 6.9314718246e-01f
const LG2_H: f32 = f32::from_bits(0x3f317200); // 6.93145752e-01f
const LG2_L: f32 = f32::from_bits(0x35bfbe8c); // 1.42860654e-06f
const OVT: f32 = 4.2995665694e-08f32; // -(128-log2(ovfl+.5ulp))
const CP: f32 = f32::from_bits(0x3f76384f); // 9.6179670095e-01f =  2/(3ln2)
const CP_H: f32 = f32::from_bits(0x3f763800); // 9.6179199219e-01f =  head of cp
const CP_L: f32 = f32::from_bits(0x369dc3a0); // 4.7017383622e-06f =  tail of cp_h
const IVLN2: f32 = f32::from_bits(0x3fb8aa3b); // 1.4426950216e+00f =  1/ln2
const IVLN2_H: f32 = f32::from_bits(0x3fb8aa00); // 1.4426879883e+00f = 16b 1/ln2
const IVLN2_L: f32 = f32::from_bits(0x36eca570); // 7.0526075433e-06f = 0x36eca570 = 1/ln2 tail

pub(super) fn powf(x: f32, y: f32) -> f32 {
    let hx: i32 = x.to_bits() as i32;
    let hy: i32 = y.to_bits() as i32;
    let mut ix: i32 = hx & 0x7fffffff;
    let iy: i32 = hy & 0x7fffffff;
    let mut is: i32;
    let mut j: i32;
    let mut n: i32;
    let mut t1: f32 = 0.;
    let t2: f32;
    let (mut r, mut s, mut t, mut u, mut v, w): (f32, f32, f32, f32, f32, f32);
    let mut p_h: f32;
    let mut p_l: f32;
    let mut z: f32;

    /* y==zero: x**0 = 1 */
    if iy == 0 {
        return ONE;
    }
    /* x==+-1 */
    if x == 1.0f32 {
        return ONE;
    }
    if (x == -1.0f32) && isinff(y) {
        return ONE;
    }
    /* +-NaN return x+y */
    if (ix > 0x7f800000) || (iy > 0x7f800000) {
        return x + y;
    }
    /* determine if y is an odd int when x < 0
        yisint = 0 ... y is not an integer
        yisint = 1 ... y is an odd int
        yisint = 2 ... y is an even int
    */
    let mut yisint = 0;
    if hx < 0 {
        if iy >= 0x4b800000 {
            yisint = 2; /* even integer y */
        } else {
            if iy >= 0x3f800000 {
                let k = (iy >> 23) - 0x7f; /* exponent */
                let j = iy >> (23 - k);
                if (j << (23 - k)) == iy {
                    yisint = 2 - (j & 1);
                }
            }
        }
    }
    /* special value of y */
    if iy == 0x7f800000 { /* y is +-inf */
        if ix == 0x3f800000 {
            #[allow(clippy::eq_op)] // let's follow the c source
            {
                return y - y; /* inf**+-1 is NaN */
            }
        } else {
            /* (|x|>1)**+-inf = inf,0 */
            if ix > 0x3f800000 {
                if hy >= 0 {return y;} else {return ZERO;}
            /* (|x|<1)**-,+inf = inf,0 */
            } else {
                if hy < 0 {return -y;} else {return ZERO;}
            }
        }
    }
    /* y is +-1 */
    if iy == 0x3f800000 {
        if hy < 0 {
            return ONE / x;
        } else {
            return x;
        }
    }
    /* y is 2 */
    if hy == 0x40000000 {
        return x * x;
    }
    /* y is 0.5 */
    if hy == 0x3f000000 {
        if hx >= 0 {
            return sqrtf(x); /* x >= +0 */
        }
    }
    let mut ax = x.abs();
    /* special value of x */
    if (ix == 0x7f800000) || (ix == 0) || (ix == 0x3f800000) {
        let mut z = ax;     /*x is +-0,+-inf,+-1*/
        if hy < 0 {
            z = ONE / z; /* z = (1/|x|) */
        }
        if hx < 0 {
            if ((ix - 0x3f800000) | yisint) == 0 {
                #[allow(clippy::eq_op)] // let's follow the c source
                {
                    z = (z - z) / (z - z); /* (-1)**non-int is NaN */
                }
            } else {
                if yisint == 1 {
                    z = -z; /* (x<0)**odd = -(|x|**odd) */
                }
            }
        }
        return z;
    }
    /* (x<0)**(non-int) is NaN */
    if ((((hx as u32) >> 31) - 1) | (yisint as u32)) == 0 {
        #[allow(clippy::eq_op)] // let's follow the c source
        {
            return (x - x) / (x - x);
        }
    }
     /* |y| is huge */
    if iy > 0x4d000000 { /* if |y| > 2**27 */
        /* over/underflow if x is not close to one */
        if ix < 0x3f7ffff8 {
            if hy < 0 {return HUGE * HUGE;} else {return TINY * TINY;}
        }
        if ix > 0x3f800007 {
            if hy > 0 {return HUGE * HUGE;} else {return TINY * TINY;}
        }
        /* now |1-x| is tiny <= 2**-20, suffice to compute
        log(x) by x-x^2/2+x^3/3-x^4/4 */
        let t = x - 1.;  /* t has 20 trailing zeros */
        let w = (t * t) * (0.5f32 - t * (0.333333333333f32 - t * 0.25f32));
        let u = IVLN2_H * t; /* ivln2_h has 16 sig. bits */
        let v = t * IVLN2_L - w * IVLN2;
        let mut t1 = u + v;
        is = t1.to_bits() as i32;
        t1 = f32::from_bits((is & 0xfffff000u32 as i32) as u32);
        t2 = v - (t1 - u);
    } else {
        let (mut s2, mut s_h, s_l, mut t_h, mut t_l): (f32, f32, f32, f32, f32);
        n = 0;
        /* take care subnormal number */
        if ix < 0x00800000 {
            ax *= TWO24;
            n -= 24;
            ix = ax.to_bits() as i32;
        }
        n += (ix >> 23) - 0x7f;
        j = ix & 0x007fffff;
        /* determine interval */
        ix = j | 0x3f800000;  /* normalize ix */
        let k: usize;
        if j <= 0x1cc471 {
            k = 0; /* |x|<sqrt(3/2) */
        } else {
            if j < 0x5db3d7 {
                k = 1; /* |x|<sqrt(3) */
            }  else {
                k = 0;
                n += 1;
                ix -= 0x00800000;
            }
        }
        ax = f32::from_bits(ix as u32);
        /* compute s = s_h+s_l = (x-1)/(x+1) or (x-1.5)/(x+1.5) */
        u = ax - BP[k]; /* bp[0]=1.0, bp[1]=1.5 */
        v = ONE / (ax + BP[k]);
        s = u * v;
        s_h = s;
        is = s_h.to_bits() as i32;
        s_h = f32::from_bits((is & 0xfffff000u32 as i32) as u32);
        /* t_h=ax+bp[k] High */        
        t_h = f32::from_bits((((ix >> 1) | 0x20000000) + 0x0040000 + (k << 21) as i32) as u32);
        t_l = ax - (t_h - BP[k]);
        s_l = v * ((u - s_h * t_h) - s_h * t_l);
        /* compute log(ax) */
        s2 = s * s;
        r = s2 * s2 * (L1 + s2 * (L2 + s2 * (L3 + s2 * (L4 + s2 * (L5 + s2 * L6)))));
        r += s_l * (s_h + s);
        s2  = s_h * s_h;
        t_h = 3.0f32 + s2 + r;
        is = t_h.to_bits() as i32;
        t_h = f32::from_bits((is & 0xfffff000u32 as i32) as u32);
        t_l = r - ((t_h - 3.0f32) - s2);
        /* u+v = s*(1+...) */
        u = s_h * t_h;
        v = s_l * t_h + t_l * s;
        /* 2/(3log2)*(s+...) */
        p_h = u + v;
        is = p_h.to_bits() as i32;
        p_h = f32::from_bits((is & 0xfffff000u32 as i32) as u32);
        p_l = v - (p_h - u);
        let z_h: f32 = CP_H * p_h; /* cp_h+cp_l = 2/(3*log2) */
        let z_l: f32 = CP_L * p_h + p_l * CP + DP_L[k];
        /* log2(ax) = (s+..)*2/(3*log2) = n + dp_h + z_h + z_l */
        t = n as f32;
        t1 = z_h + z_l + DP_H[k] + t;
        is = t1.to_bits() as i32;
        t1 = f32::from_bits((is & 0xfffff000u32 as i32) as u32);
        t2 = z_l - t1 - t - DP_H[k] - z_h;
    }
    s = ONE; /* s (sign of result -ve**odd) = -1 else = 1 */
    if ((((hx as u32) >> 31) - 1) | (yisint - 1) as u32) == 0 {
        s = -ONE; /* (-ve)**(odd int) */
    }
    /* split up y into y1+y2 and compute (y1+y2)*(t1+t2) */
    is = y.to_bits() as i32;
    let y1 = f32::from_bits((is & 0xfffff000u32 as i32) as u32);
    p_l = (y - y1) * t1 + y * t2;
    p_h = y1 * t1;
    z = p_l + p_h;
    j = z.to_bits() as i32;
    /* if z > 128 */
    if j > 0x43000000 {
        return s * HUGE * HUGE; /* overflow */
    } else {
        /* if z == 128 */
        if j == 0x43000000 {
            if p_l + OVT > z - p_h {
                return s * HUGE * HUGE; /* overflow */
            }
        } else {
            /* z <= -150 */
            if (j & 0x7fffffff) > 0x43160000 {
                return s * TINY * TINY; /* underflow */
            } else {
                /* z == -150 */
                if (j as u32) == 0xc3160000 {
                    if p_l <= z - p_h {
                        return s * TINY * TINY; /* underflow */
                    }
                }
            }     
        }
    }
    /* compute 2**(p_h+p_l) */
    let i = j & 0x7fffffff;
    let mut k = ( i >> 23 ) - 0x7f;
    n = 0;
    /* if |z| > 0.5, set n = [z+0.5] */
    if i > 0x3f000000 {
        n = j + (0x00800000 >> (k + 1));
        k = ((n & 0x7fffffff) >> 23) - 0x7f; /* new k for n */
        t = f32::from_bits((n & !(0x007fffff >> k)) as u32);
        n = ((n & 0x007fffff) | 0x00800000) >> (23 - k);
        if j < 0 {
            n = -n;
        }
        p_h -= t;
    }
    t = p_l + p_h;
    is = t.to_bits() as i32;
    t = f32::from_bits((is & 0xfffff000u32 as i32) as u32);
    u = t * LG2_H;
    v = (p_l - (t - p_h)) * LG2 + t * LG2_L;
    z = u + v;
    w = v - (z - u);
    t  = z * z;
    t1  = z - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
    r  = (z * t1) / (t1 - TWO) - (w + z * w);
    z  = ONE - (r - z);
    j = z.to_bits() as i32;
    j += n << 23;
    if (j >> 23) <= 0 {
        z = scalbnf(z, n); /* subnormal output */
    } else {
        z = f32::from_bits(j as u32);
    }
    s * z
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

