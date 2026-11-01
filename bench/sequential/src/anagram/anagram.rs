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

  Name: anagram

  Author: Raymond Chen

  Function: A program that computes anagrams.

  Source: See below.

  Original name: anagram

  Changes: See ChangeLog.txt

  License: See below.

*/

/*
   Anagram program by Raymond Chen,
   inspired by a similar program by Brian Scearce

   This program is Copyright 1991 by Raymond Chen.
               (rjc@math.princeton.edu)

   This program may be freely distributed provided all alterations
   to the original are clearly indicated as such.
*/

/* There are two tricks.  First is the Basic Idea:

   When the user types in a phrase, the phrase is first preprocessed to
   determine how many of each letter appears.  A bit field is then constructed
   dynamically, such that each field is large enough to hold the next power
   of two larger than the number of times the character appears.  For example,
   if the phrase is hello, world, the bit field would be

     00 00 00 000 000 00 00
      d e   h   l   o  r  w

   The phrase hello, world, itself would be encoded as

     01 01 01 011 010 01 01
      d e   h   l   o  r  w

   and the word hollow would be encoded as

     00 00 01 010 010 00 01
      d  e  h   l   o  r  w

   The top bit of each field is set in a special value called the sign.
   Here, the sign would be

     10 10 10 100 100 10 10
      d  e  h   l   o  r  w


   The reason for packing the values into a bit field is that the operation
   of subtracting out the letters of a word from the current phrase can be
   carried out in parallel.  for example, subtracting the word hello from
   the phrase hello, world, is merely

      d e   h   l   o  r  w
     01 01 01 011 010 01 01 (dehllloorw)
   - 00 00 01 010 010 00 01 (hlloow)
   ========================
     01 01 00 001 000 01 00 (delr)

   Since none of the sign bits is set, the word fits, and we can continue.
   Suppose the next word we tried was hood.

      d e   h   l   o  r  w
     01 01 00 001 000 01 00 (delr)
   - 01 00 01 000 010 00 00 (hood)
   ========================
     00 00 11 000 110 01 00
           ^      ^
   A sign bit is set.  (Two, actually.)  This means that hood does not
   fit in delr, so we skip it and try another word.  (Observe that
   when a sign bit becomes set, it screws up the values for the letters to
   the left of that bit, but that's not important.)

   The inner loop of an anagram program is testing to see if a
   word fits in the collection of untried letters.  Traditional methods
   keep an array of 26 integers, which are then compared in turn.  This
   means that there are 26 comparisons per word.

   This method reduces the number of comparisons to MAX_QUAD, typically 2.
   Instead of looping through an array, we merely perform the indicated
   subtraction and test if any of the sign bits is set.
*/

/* The nuts and bolts:

   The dictionary is loaded and preprocessed.  The preprocessed dictionary
   is a concatenation of copies of the structure:

   struct dictword {
       char bStructureSize;             -- size of this structure
       char cLetters;                   -- number of letters in the word
       char achWord[];                  -- the word itself (0-terminated)
   }

   Since this is a variable-sized structure, we keep its size in the structure
   itself for rapid stepping through the table.

   When a phrase is typed in, it is first preprocessed as described in the
   Basic Idea.  We then go through the dictionary, testing each word.  If
   the word fits in our phrase, we build the bit field for its frequency
   table and add it to the list of candidates.
*/

/*
   The Second Trick:

   Before diving into our anagram search, we then tabulate how many times
   each letter appears in our list of candidates, and sort the table, with
   the rarest letter first.

   We then do our anagram search.

   Like most anagram programs, this program does a depth-first search.
   Although most anagram programs do some sort of heuristics to decide what
   order to place words in the list_of_candidates, the search itself proceeds
   according to a greedy algorithm.  That is, once you find a word that fits,
   subtract it and recurse.

   This anagram program exercises some restraint and does not march down
   every branch that shows itself.  Instead, it only goes down branches
   that use the rarest unused letter.  This helps to find dead ends faster.

   FindAnagram(unused_letters, list_of_candidates) {
    l = the rarest letter as yet unused
    For word in list_of_candidates {
       if word does not fit in unused_letters, go on to the next word.
       if word does not contain l, defer.
        FindAnagram(unused_letters - word, list_of_candidates[word,...])
    }
   }


   The heuristic of the Second Trick can probably be improved.  I invite
   anyone willing to improve it to do so.
*/

/* Before compiling, make sure Quad and MASK_BITS are set properly.  For best
   results, make Quad the largest integer size supported on your machine.
   So if your machine has long longs, make Quad an unsigned long long.
   (I called it Quad because on most machines, the largest integer size
   supported is a four-byte unsigned long.)

   If you need to be able to anagram larger phrases, increase MAX_QUADS.
   If you increase it beyond 4, you'll have to add a few more loop unrolling
   steps to FindAnagram.
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;
use core::mem::size_of;

mod quicksort;
use quicksort::sort;

mod input;
use input::{ ANAGRAM_ACHPHRASE, ANAGRAM_DICTIONARY };

mod memory;
use memory::{ SimulatedHeap, HeapSlice };

/*
##################
    anagram_ctype.h
##################
*/

#[inline]
fn anagram_islower(c: u8) -> bool {
    b'a' <= c && c <= b'z'
}

#[inline]
fn anagram_isupper(c: u8 ) -> bool {
    b'A' <= c && c <= b'Z'
}

#[inline]
fn anagram_isalpha(c: u8) -> bool {
    anagram_isupper(c) || anagram_islower(c)
}

#[inline]
fn anagram_tolower(c: u8) -> u8 {
    if anagram_isupper(c) { c + (b'a' - b'A') } else { c }
}

/*
  Defines
*/

const MASK_BITS: usize = 32;      /* number of bits in a Quad */
const MAX_QUADS: usize = 2;       /* controls largest phrase */
const MAXCAND: usize = 100;       /* candidates */
const MAXSOL: usize = 51;         /* words in the solution */
const ALPHABET: usize = 26;       /* letters in the alphabet */

#[inline]
fn anagram_one_step(
    i: usize,
    aq_next: &mut [Quad; MAX_QUADS],
    pq_mask: &mut [Quad],
    pw: HeapSlice,
    ppw_start: &mut usize,
    data: &AnagramData,
    heap: &mut SimulatedHeap,
) -> bool {
    let pw_word = Word::build_from_slice(pw, heap);

    aq_next[i] = pq_mask[i] - pw_word.aq_mask[ i ];
    if (aq_next[ i ] & data.aq_main_sign[i]) != 0 {
      *ppw_start += 1;
      return true;
    }
    false
}

/*
  Type definitions
*/

type Quad = u32; /* for building our bit mask */

/* A Word remembers the information about a candidate word. */
struct Word {
    ch_word: HeapSlice,              /* the word itself */
    aq_mask: [Quad; MAX_QUADS],      /* the word's mask */
    cch_length: usize,
}

impl Word {
    fn build_from_slice(hs: HeapSlice, heap: &mut SimulatedHeap) -> Self {
        // build word from Slice
        let ch_word_start_start = hs.start;
        let ch_word_start_end = ch_word_start_start + size_of::<usize>();
        let ch_word_end_start = ch_word_start_end;
        let ch_word_end_end = ch_word_end_start + size_of::<usize>();
        let ch_word_tsize_start = ch_word_end_end;
        let ch_word_tsize_end = ch_word_tsize_start + size_of::<usize>();
        let aq_mask_start = ch_word_tsize_end;
        let aq_mask_end = aq_mask_start + size_of::<Quad>() * MAX_QUADS;
        let cch_length_start = aq_mask_end;
        let cch_length_end = cch_length_start + size_of::<usize>();
        let padding_start = cch_length_end;
        let padding_end = padding_start +  4;

        let mut bytes: [u8; size_of::<usize>()] = [0u8; size_of::<usize>()];
        bytes.copy_from_slice(&heap.heap_content[ch_word_start_start..ch_word_start_end]);
        let word_start = usize::from_ne_bytes(bytes);
        bytes.copy_from_slice(&heap.heap_content[ch_word_end_start..ch_word_end_end]);
        let word_end = usize::from_ne_bytes(bytes);
        bytes.copy_from_slice(&heap.heap_content[ch_word_tsize_start..ch_word_tsize_end]);
        let tsize = usize::from_ne_bytes(bytes);
        let word = HeapSlice {
            start: word_start,
            end: word_end,
            type_size: tsize,
        };
        let mut aq_mask: [Quad; MAX_QUADS] = [0; MAX_QUADS];
        for i in 0..MAX_QUADS  {
            let start = aq_mask_start + i * size_of::<Quad>();
            let end = start + size_of::<Quad>();
            let mut mask: [u8; size_of::<Quad>()] = [0; size_of::<Quad>()];
            mask.copy_from_slice(&heap.heap_content[start..end]);
            let val = Quad::from_ne_bytes(mask);
            aq_mask[i] = val;
        }
        bytes.copy_from_slice(&heap.heap_content[cch_length_start..cch_length_end]);
        let cch_length = usize::from_ne_bytes(bytes);
        let mut pad: [u8; 4] = [0; 4];
        pad.copy_from_slice(&heap.heap_content[padding_start..padding_end]);
        Word {
            ch_word: word,
            aq_mask,
            cch_length,
        }
    }
}

/* A Letter remembers information about each letter in the phrase to
   be anagrammed. */
#[derive(Default, Clone, Copy)]
struct AnagramLetter {
    u_frequency: u32,    /* how many times it appears */
    u_shift: u32,        /* how to mask */
    u_bits: u32,         /* the bit mask itself */
    iq: u32,            /* which Quad to inspect? */
}

impl AnagramLetter {
    const fn new() -> Self {
        AnagramLetter {
            u_frequency: 0,
            u_shift: 0,
            u_bits: 0,
            iq: 0,
        }
    }
}

struct AnagramData {
    /* statistics on the current phrase */
    al_phrase: [AnagramLetter; ALPHABET],
    /* where the sign bits are */
    aq_main_sign: [Quad; MAX_QUADS ],
    /* auGlobalFrequency counts the number of times each letter appears,
      summed over all candidate words. This is used to decide which letter
      to attack first. */
    au_global_frequency: [u32; ALPHABET],
    ach_by_frequency: [usize; ALPHABET ], /* for sorting */
    /* number of letters in phrase */
    cch_phrase_length: usize,
}

impl AnagramData {
    const fn new() -> Self {
        AnagramData {
            al_phrase: [AnagramLetter::new(); ALPHABET],
            aq_main_sign: [0; MAX_QUADS],
            au_global_frequency: [0; ALPHABET],
            ach_by_frequency: [0; ALPHABET],
            cch_phrase_length: 0,
        }
    }
}

const CCH_MIN_LENGTH: usize = 3;

struct AnagramCandidates {
    /* candidates we've found so far */
    apw_cand: [Option<HeapSlice>; MAXCAND],
    /* how many of them? */
    cpw_cand: usize,
    /* the answers */
    apw_sol: [HeapSlice; MAXSOL],
    cpw_last: usize
}

impl AnagramCandidates {
    const fn new() -> Self {
        AnagramCandidates {
            apw_cand: [None; MAXCAND],
            cpw_cand: 0,
            apw_sol: [HeapSlice::new(); MAXSOL],
            cpw_last: 0,
        }
    }
}

/*
  Initialization- and return-value-related functions
*/

/* ReadDict -- read the dictionary file into memory and preprocess it

   A word of length cch in the dictionary is encoded as follows:

      byte 0    = cch + 3
      byte 1    = number of letters in the word
      byte 2... = the word itself, null-terminated

   Observe that cch+3 is the length of the total encoding.  These
   byte streams are concatenated, and terminated with a 0.
*/
fn anagram_read_dict(
    ch_dictionary: &mut HeapSlice,
    heap: &mut SimulatedHeap,
) {
    let bitmask = 0;

    let mut len = 0;
    // loopbound min 2279 max 2279
    for &word in ANAGRAM_DICTIONARY.iter() {
        let strlen = word.len();
        len += strlen + 3;
    }

    len += 1;
    *ch_dictionary = heap.malloc(len, size_of::<u8>());
    let mut pch_base = ch_dictionary.start;

    // loopbound min 2279 max 2279
    for &word in ANAGRAM_DICTIONARY.iter() {
        let mut pch = pch_base + 2; /* reserve for length */
        let mut c_letters = 0;

        // loopbound min 1 max 5
        for ch in word.iter() {
            if anagram_isalpha(*ch) {
                c_letters += 1;
            }
            heap.heap_content[pch] = *ch;
            heap.heap_content[pch] ^= bitmask;
            pch += 1;
        }
        heap.heap_content[pch] = b'\0';
        pch += 1;
        heap.heap_content[pch_base] = (pch - pch_base) as u8;
        heap.heap_content[pch_base + 1] = c_letters as u8;
        pch_base = pch;
    }
    heap.heap_content[pch_base] = 0;
}


fn anagram_init(
    ch_dictionary: &mut HeapSlice,
    heap: &mut SimulatedHeap,
) {
    heap.reset();
    anagram_read_dict(ch_dictionary, heap);
}

fn anagram_return(buffer: &[u8; 30]) -> bool {
    const ANSWER: [u8; 12] = *b"duke yip arm";
    // loopbound min 12 max 12
    for (&res, &letter) in ANSWER.iter().zip(buffer.iter()) {
        if res != letter {
            return false;
        }
    }
    true
}

/*
  Core benchmark functions
*/

/* convert letter to index */
fn anagram_ch2i(ch: u8) -> usize {
    (ch - b'a') as usize
}

fn anagram_reset(
    aq_main_mask: &mut [Quad; MAX_QUADS],
    data: &mut AnagramData,
    candidates: &mut AnagramCandidates,
) {
    data.al_phrase = [AnagramLetter::default(); ALPHABET];
    *aq_main_mask = [0; MAX_QUADS];
    data.aq_main_sign = [0; MAX_QUADS];
    data.au_global_frequency = [0; ALPHABET];
    data.ach_by_frequency = [0; ALPHABET];
    candidates.apw_cand = [None; MAXCAND];
    data.cch_phrase_length = 0;
    candidates.cpw_cand = 0;
}

fn anagram_build_mask(
    aq_main_mask: &mut [Quad; MAX_QUADS],
    pch_phrase: &[u8],
    data: &mut AnagramData,
) {
    /* Tabulate letter frequencies in the phrase */
    data.cch_phrase_length = 0;
    // loopbound min 11 max 11
    for ch in pch_phrase.iter() {
        if anagram_isalpha(*ch) {
            let ch = anagram_tolower(*ch);
            data.al_phrase[anagram_ch2i(ch)].u_frequency += 1;
            data.cch_phrase_length += 1;
        }
    }

    /* Build  masks */
    let iq = 0;                     /* which Quad?; which quad being used */
    let mut cbt_used = 0;            /* bits used in the current Quad; bits used so far */

    // loopbound min 26 max 26
    for i in 0..ALPHABET {
        if data.al_phrase[ i ].u_frequency == 0 {
            data.au_global_frequency[ i ] = !0;  /* to make it sort last */
        } else {
            data.au_global_frequency[i] = 0;
            // loopbound min 1 max 2
            let mut cbt_need = 1;          /* bits needed for current letter */
            let mut q_need = 1;              /* used to build the mask */
            while data.al_phrase[ i ].u_frequency >= q_need {
                cbt_need += 1;
                q_need <<= 1;
            }
            if cbt_used + cbt_need > MASK_BITS {
                cbt_used = 0;
            }
            data.al_phrase[i].u_bits = q_need - 1;
            if cbt_used > 0 {
                q_need <<= cbt_used;
            }
            data.aq_main_sign[iq] |= q_need;
            aq_main_mask[iq] |= data.al_phrase[ i ].u_frequency << cbt_used; // ( anagram_Quad )
            data.al_phrase[i].u_shift = cbt_used as u32;
            data.al_phrase[i].iq = iq as u32;
            cbt_used += cbt_need;
        }
    }
}

fn anagram_new_word(heap: &mut SimulatedHeap) -> HeapSlice {
    let pw = heap.malloc(size_of::<Word>(), size_of::<Word>());
    pw
}

/* NextWord -- get another candidate entry, creating if necessary */
fn anagram_next_word(
    candidates: &mut AnagramCandidates,
    heap: &mut SimulatedHeap
) -> HeapSlice {
    // anagram_PWord pw;
    let pw = candidates.apw_cand[candidates.cpw_cand];
    candidates.cpw_cand += 1;
    if pw.is_some() {
        return pw.unwrap();
    }
    candidates.apw_cand[candidates.cpw_cand - 1] = Some(anagram_new_word(heap));
    candidates.apw_cand[candidates.cpw_cand - 1].unwrap()
}


/* BuildWord -- build a Word structure from an ASCII word
   If the word does not fit, then do nothing. */
fn anagram_build_word(
    pch_word: usize,
    data: &mut AnagramData,
    candidates: &mut AnagramCandidates,
    heap: &mut SimulatedHeap,
) {
    let mut word_end = pch_word;
    let mut cch_frequency: [u8; ALPHABET] = [0; ALPHABET];
    let mut cch_length: usize = 0;

    /* Build frequency table */
    // loopbound min 3 max 5
    while heap.heap_content[word_end] != b'\0' {
        word_end += 1;
        if !anagram_isalpha(heap.heap_content[word_end - 1]) {
            continue;
        }
        let i = anagram_ch2i(anagram_tolower(heap.heap_content[word_end - 1]));
        cch_frequency[i] += 1;
        if (cch_frequency[i] as u32) > data.al_phrase[i].u_frequency {
            return ;
        }
        cch_length += 1;  
    }

    /* Update global count */
    // loopbound min 26 max 26
    for i in 0..ALPHABET {
        data.au_global_frequency[i] += cch_frequency[i] as u32;
    }

    /* Create a Word structure and fill it in, including building the
      bitfield of frequencies. */
    let pw_hs = anagram_next_word(candidates, heap);
    let pw_word_hs: HeapSlice = HeapSlice {
        start: pch_word,
        end: word_end,
        type_size: size_of::<u8>(),
    };
    let bytes = pw_word_hs.to_ne_bytes();
    let start = pw_hs.start;
    let end =  pw_hs.start + size_of::<HeapSlice>();
    heap.heap_content[start..end].copy_from_slice(&bytes);
    
    let bytes = cch_length.to_ne_bytes();
    let start = end + MAX_QUADS * size_of::<Quad>();
    let end = start + size_of::<usize>();
    heap.heap_content[start..end].copy_from_slice(&bytes);

    // loopbound min 26 max 26
    let start = pw_hs.start + size_of::<HeapSlice>();
    for i in 0..ALPHABET {
        let or_val = (cch_frequency[ i ] as Quad) << data.al_phrase[ i ].u_shift;
        let index = data.al_phrase[i].iq as usize;
        let mut bytes: [u8; size_of::<Quad>()] = [0; size_of::<Quad>()];
        bytes.copy_from_slice(&heap.heap_content[(start + index)..(start + index + size_of::<Quad>())]);
        let mut val: Quad = Quad::from_ne_bytes(bytes);
        val |= or_val;
        let bytes = val.to_ne_bytes();
        heap.heap_content[(start + index)..(start + index + size_of::<Quad>())].copy_from_slice(&bytes);
    }
}


/* AddWords -- build the list of candidates */
fn anagram_add_words(
    data: &mut AnagramData,
    candidates: &mut AnagramCandidates,
    ch_dictionary: &mut HeapSlice,
    heap: &mut SimulatedHeap,
  ) {
    let mut pch = ch_dictionary.start; /* walk through the dictionary */

    candidates.cpw_cand = 0;

    // loopbound min 2280 max 2280
    // -> original has loopbound min 1967 max 1967
    // -> this does not match adding up the count
    while heap.heap_content[pch] != 0 {
        let val = heap.heap_content[pch + 1];

        let cond1 = val as usize >= CCH_MIN_LENGTH && val as usize + CCH_MIN_LENGTH <= data.cch_phrase_length;
        let cond2 = val as usize == data.cch_phrase_length;
        if cond1 || cond2 {
            anagram_build_word(pch + 2, data, candidates, heap);
        }
        pch += heap.heap_content[pch] as usize;
    }
}

fn anagram_dump_words(
    candidates: &mut AnagramCandidates,
    buffer: &mut [u8; 30],
    heap: &mut SimulatedHeap,
) {
    let mut offset = 0;
    // loopbound min 3 max 3
    for i in 0..candidates.cpw_last {
        // loopbound min 3 max 5
        let word = Word::build_from_slice(candidates.apw_sol[i], heap);
        let mut j = 0;
        while heap.heap_content[word.ch_word.start + j * word.ch_word.type_size] != b'\0' {
            buffer[offset + j] = heap.heap_content[word.ch_word.start + j * word.ch_word.type_size];
            j += 1;
        }
        offset += j;
        buffer[offset] = b' ';
        offset += 1;
    } 
    buffer[offset] = b'\0';
}

fn anagram_find_anagram(
    pq_mask: &mut [Quad],
    mut ppw_start: usize,
    mut i_letter: usize,
    data: &mut AnagramData,
    candidates: &mut AnagramCandidates,
    buffer: &mut [u8; 30],
    heap: &mut SimulatedHeap,
) {
    let mut aq_next: [Quad; MAX_QUADS] = [0; MAX_QUADS];
    let mut pw: HeapSlice;
    let mut q_mask: Quad;
    let mut iq: usize;
    let mut ppw_end = candidates.cpw_cand;

    // loopbound min 0 max 6
    loop {
        iq = data.al_phrase[data.ach_by_frequency[i_letter]].iq as usize;
        q_mask = data.al_phrase[data.ach_by_frequency[i_letter]].u_bits <<
              data.al_phrase[data.ach_by_frequency[i_letter]].u_shift;
        if ( pq_mask[iq] & q_mask ) != 0 {
            break;
        }
        i_letter += 1;
    }

    // loopbound min 0 max 11
    while ppw_start < ppw_end {
        pw = candidates.apw_cand[ppw_start].unwrap();

        if MAX_QUADS > 0 && anagram_one_step(0, &mut aq_next, pq_mask, pw, &mut ppw_start, data, heap) {
            continue;
        }

        if MAX_QUADS > 1 && anagram_one_step(1, &mut aq_next, pq_mask, pw, &mut ppw_start, data, heap) {
            continue;
        }

        if MAX_QUADS > 2 && anagram_one_step(2, &mut aq_next, pq_mask, pw, &mut ppw_start, data, heap) {
            continue;
        }
        
        if MAX_QUADS > 3 && anagram_one_step(3, &mut aq_next, pq_mask, pw, &mut ppw_start, data, heap) {
            continue;
        }

        let word = Word::build_from_slice(pw, heap);

        /* If the pivot letter isn't present, defer this word until later */
        if (word.aq_mask[ iq ] & q_mask) == 0 {
            ppw_end -= 1;
            candidates.apw_cand[ppw_start] = candidates.apw_cand[ppw_end];
            candidates.apw_cand[ppw_end] = Some(pw);
            continue;
        }

        /* If we get here, this means the word fits. */
        candidates.apw_sol[candidates.cpw_last] = pw;
        candidates.cpw_last += 1;
        data.cch_phrase_length -= word.cch_length;
        if data.cch_phrase_length != 0 { /* recurse */
            /* The recursive call scrambles the tail, so we have to be
            pessimistic. */
            ppw_end = candidates.cpw_cand;
            anagram_find_anagram(
                &mut aq_next, ppw_start, i_letter,
                data, candidates, buffer, heap,
            );
        } else {
            /* found one */
            anagram_dump_words(candidates, buffer, heap);
        }
        data.cch_phrase_length += word.cch_length;
        candidates.cpw_last -= 1;
        ppw_start += 1;
        continue;
    }
}

fn anagram_sort_candidates(data: &mut AnagramData) {
    /* Sort the letters by frequency */
    // loopbound min 26 max 26
    for (i, el) in data.ach_by_frequency.iter_mut().enumerate() {
        *el = i;
    }
    sort(&mut data.ach_by_frequency, &data.au_global_frequency);
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn anagram_main(
    aq_main_mask: &mut [Quad; MAX_QUADS],
    data: &mut AnagramData,
    candidates: &mut AnagramCandidates,
    buffer: &mut [u8; 30],
    ch_dictionary: &mut HeapSlice,
    heap: &mut SimulatedHeap,
) {
    // loopbound min 3 max 3
    for el in ANAGRAM_ACHPHRASE {
        anagram_reset(aq_main_mask, data, candidates);
        anagram_build_mask(aq_main_mask, el, data);
        anagram_add_words(data, candidates, ch_dictionary, heap);
        if candidates.cpw_cand == 0 || data.cch_phrase_length == 0 {
            continue;
        }
        candidates.cpw_last = 0;
        anagram_sort_candidates(data);
        // marker recursivecall
        // flowrestriction 1*anagram_FindAnagram <= 51*recursivecall
        let ppw_start = 0;
        anagram_find_anagram(
            aq_main_mask, ppw_start, 0, data, candidates, buffer, heap,
        );
    }
}

/*
  Main function
*/

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn anagram_log(
    aq_main_mask: &mut [Quad; MAX_QUADS],
    data: &mut AnagramData,
    candidates: &mut AnagramCandidates,
    buffer: &mut [u8; 30],
    ch_dictionary: &mut HeapSlice,
    heap: &mut SimulatedHeap,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        anagram_init(black_box(ch_dictionary), black_box(heap));
        let start = black_box(DWT::cycle_count());
        anagram_main(
            black_box(aq_main_mask),
            black_box(data), black_box(candidates),
            black_box(buffer), black_box(ch_dictionary),
            black_box(heap),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("anagram - elapsed: {}", cycles);
    });

    black_box(anagram_return(black_box(buffer)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn anagram_std(
    aq_main_mask: &mut [Quad; MAX_QUADS],
    data: &mut AnagramData,
    candidates: &mut AnagramCandidates,
    buffer: &mut [u8; 30],
    ch_dictionary: &mut HeapSlice,
    heap: &mut SimulatedHeap,
) -> bool {
    anagram_init(black_box(ch_dictionary), black_box(heap));
    anagram_main(
        black_box(aq_main_mask),
        black_box(data), black_box(candidates),
        black_box(buffer), black_box(ch_dictionary),
        black_box(heap),
    );
    black_box(anagram_return(black_box(buffer)))
}

fn anagram() -> bool {
    static mut AQ_MAIN_MASK: [Quad; MAX_QUADS] = [0; MAX_QUADS];
    static mut DATA: AnagramData = AnagramData::new();
    static mut CANDIDATES: AnagramCandidates = AnagramCandidates::new();
    static mut BUFFER: [u8; 30] = [0; 30];
    static mut CH_DICTIONARY: HeapSlice = HeapSlice::new();
    static mut HEAP: SimulatedHeap = SimulatedHeap::new();
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let aq_main_mask = unsafe { &mut *{ &raw mut AQ_MAIN_MASK } };  /* the bit field for the full phrase */
    let data = unsafe { &mut *{ &raw mut DATA } };
    let candidates = unsafe { &mut *{ &raw mut CANDIDATES } };
    let buffer = unsafe { &mut *{ &raw mut BUFFER } }; /* buffer to write an answer */
    let ch_dictionary = unsafe { &mut *{ &raw mut CH_DICTIONARY } }; /* the dictionary is read here */
    let heap = unsafe { &mut *{ &raw mut HEAP } };    

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(anagram_log(
            black_box(aq_main_mask),
            black_box(data), black_box(candidates),
            black_box(buffer), black_box(ch_dictionary),
            black_box(heap),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(anagram_std(
            black_box(aq_main_mask),
            black_box(data), black_box(candidates),
            black_box(buffer), black_box(ch_dictionary),
            black_box(heap),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(anagram());
    
    #[cfg(feature = "log")]
    {
        rprintln!("anagram: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(anagram());

    #[cfg(feature = "log")]
    std::println!("anagram: {}", success);

    std::process::exit(!success as i32);
}
