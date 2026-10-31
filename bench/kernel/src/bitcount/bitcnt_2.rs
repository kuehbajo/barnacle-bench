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

  Name: bitcnt_2.c

  Author: Bob Stout & Auke Reitsma

  Function: Test program for bit counting functions

  Source: http://www.snippets.org/

  Changes: no major functional changes

  License: May be used, modified, and re-distributed freely.

*/

pub fn bitcount_bitcount(mut i: u32) -> i32 {
    i = ((i & 0xAAAAAAAA) >> 1) + (i & 0x55555555);
    i = ((i & 0xCCCCCCCC) >> 2) + (i & 0x33333333);
    i = ((i & 0xF0F0F0F0) >> 4) + (i & 0x0F0F0F0F);
    i = ((i & 0xFF00FF00) >> 8) + (i & 0x00FF00FF);
    i = ((i & 0xFFFF0000) >> 16) + (i & 0x0000FFFF);
    i as i32
}
