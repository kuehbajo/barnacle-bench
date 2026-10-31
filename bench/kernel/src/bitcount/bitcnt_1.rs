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

  Version V 1.x

  Name: bitcnt_1.c

  Author: Ratko Tomic

  Function: Test program for bit counting functions

  Source: http://www.snippets.org/.

  Changes: no major functional changes

  License: May be used, modified, and re-distributed freely.
  
*/

pub fn bitcount_bit_count(mut x: u32) -> i32 {
    let mut n = 0;
    if x != 0 {
        // loopbound min 3 max 8
        loop {
            n += 1;
            x = x & (x - 1);
            if x == 0 {
                break;
            }
        }
    }
    n
}
