/*
  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license (GPL 2) as the original main file.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite.
  
*/

pub fn ammunition_strcmp(
    str1: &[u8],
    str2: &[u8],
) -> i32 {
    let mut index = 0;
    // loopbound min 1 max 11
    while {
        index < str1.len()
            && index < str2.len()
            && str1[index] != b'\0'
            && (str1[index] == str2[index])
    } {
        index += 1;
    }
    if index == str1.len() || index == str2.len() {
        0
    } else {
        str1[index] as i32 - str2[index] as i32
    }
}

pub fn ammunition_atoi(str: &[u8]) -> i32 {
    let mut result: i32 = 0;
    let sign: i32 = if str[ 0 ] == b'-' { -1 } else { 1 };

    let mut reading_pos = 0;
    if str[0] == b'-' || str[0] == b'+' {
        reading_pos += 1;
    }
    // loopbound min 1 max 7
    loop {
        result *= 10;
        result += str[reading_pos] as i32 - 48;
        reading_pos += 1;
        if str[reading_pos] == 0 { break; }
    }

    sign * result
}

pub fn ammunition_sprintf_d(
    s: &mut [u8],
    number: i32,
) -> i32 {
    /* How many decimal digits do we need? */
    let mut digits: i32 = 0;
    let mut write_pos: usize;
    let mut copy_of_number = number as i64;
    // loopbound min 1 max 10
    loop {
        digits += 1;
        copy_of_number /= 10;
        if copy_of_number == 0 { break; }
    }
    write_pos = digits as usize;
    if number < 0 {
        write_pos += 1;
        s[0] = b'-';
    }
    s[write_pos] = 0; // b'\0'

    copy_of_number = number as i64;
    // loopbound min 1 max 10
    loop {
      write_pos -= 1;
      let rhs = (copy_of_number as i64).abs();
      s[write_pos] = (48 + rhs % 10) as u8;
      copy_of_number /= 10;
      if copy_of_number == 0 { break; }
    }
    digits + if number < 0 { 1 } else { 0 }
}

pub fn ammunition_sprintf_u(
    s: &mut [u8],
    number: u32,
) -> i32 {
    /* How many decimal digits do we need? */
    let mut digits: i32 = 0;
    let mut write_pos: usize;
    let mut copy_of_number = number;
    // loopbound min 1 max 10
    loop {
      digits += 1;
      copy_of_number /= 10;
      if copy_of_number == 0 { break; }
    }
    write_pos = digits as usize;
    s[write_pos] = 0;

    copy_of_number = number;
    // loopbound min 1 max 10
    loop {
        write_pos -= 1;
        s[write_pos] = (48 + (copy_of_number % 10)) as u8;
        copy_of_number /= 10;
        if copy_of_number == 0 { break; }
    }

    return digits;
}
