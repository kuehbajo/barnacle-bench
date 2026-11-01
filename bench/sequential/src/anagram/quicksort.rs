/*

  Rust Implementation
  The alterations are the reimplementation in the Rust programming language.

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
  
  Version 2.0

  Name: anagram_stdlib.c

  Author: Raymond Chen

  Function: This file contains the C standard library functions used by anagram.

  Source: anagram

  Original name: anagram

  Changes: See ChangeLog.txt

  License: See anagram.c

*/

#[derive(PartialEq)]
enum Compare { Lower = -1, Equals = 0, Greater = 1 }

fn compare_frequencies<T, U>(frequencies: &[U], p1: T, p2: T) -> Compare
where
    T: Ord + Copy + Into<usize>,
    U: Ord + Copy,
{
    if frequencies[p1.into()] < frequencies[p2.into()] {
        Compare::Lower
    } else if frequencies[p1.into()] == frequencies[p2.into()] {

        Compare::Equals
    } else {
        Compare::Greater
    }
}

fn pivot<T, U>(arr: &[T], frequencies: &[U]) -> usize
where
    T: Ord + Copy + Into<usize>,
    U: Ord + Copy,
{
    let i = arr.len() / 6;
    let si = arr[i]; // 1/6

    let j = 3 * i;
    let sj = arr[j]; // 1/2

    let k = 5 * i;
    let sk = arr[k]; // 5/6

    if compare_frequencies(frequencies, si, sj) == Compare::Lower {
        if compare_frequencies(frequencies, si, sk) == Compare::Lower {
            if compare_frequencies(frequencies, sj, sk) == Compare::Lower {
                return j;
            }
            return k;
        }
        return i;
    }

    if compare_frequencies(frequencies, sj, sk) == Compare::Lower {
        if compare_frequencies(frequencies, si, sk) == Compare::Lower {
            return i;
        }
        return k;
    }
    j
}

fn sorts<T, U>(mut arr: &mut [T], frequencies: &[U])
where
    T: Ord + Copy + Into<usize>,
    U: Ord + Copy,
{
    // loopbound min 0 max 3
    while arr.len() > 1 {
        let pivot = if arr.len() > 10 {
            pivot(arr, frequencies)
        } else {
            arr.len() >> 1
        };

        arr.swap(0, pivot);
        let mut pi = 0;
        let mut pj = arr.len();
        let pn = pj;

        // loopbound min 0 max 10
        loop {
            // loopbound min 1 max 6
            loop {
                pi += 1;

                if !(pi < pn && compare_frequencies(frequencies, arr[pi], arr[0]) == Compare::Lower) {
                    break;
                }
            }
            // loopbound min 1 max 7
            loop {
                pj -= 1;

                if !(pj > 0 && compare_frequencies(frequencies, arr[pj], arr[0]) == Compare::Greater) {
                    break;
                }
            }

            if pj < pi {
                break;
            }

            arr.swap(pi, pj);
        }

        arr.swap(0, pj);

        let n = arr.len() - pj - 1;
        if pj >= n {
            sorts(&mut arr[..pj], frequencies);
            arr = &mut arr[pj + 1..]
        } else {
            sorts(&mut arr[pj + 1..], frequencies);
            arr = &mut arr[..pj];
        }
    }
}

pub fn sort<T, U>(arr: &mut [T], frequencies: &[U])
where
    T: Ord + Copy + Into<usize>,
    U: Ord + Copy,
{
    // marker recursivecall
    // flowrestriction 1*quicksort_sorts <= 17*recursivecall  
    sorts(arr, frequencies);
}
