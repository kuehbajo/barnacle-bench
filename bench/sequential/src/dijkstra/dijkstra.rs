/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license (GPL) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  Version V 2.0

  Name: dijkstra

  Author: unknown

  Function: dijkstra finds the shortest path between nodes in a graph

  Source: network section of MiBench

  Changes: Made some variables local, compute checksum

  License: GPL

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
use input::{ NUM_NODES, INPUT_ADJ_MATRIX };

/*
  Definitions of symbolic constants
*/

const NONE: i32 = 9999;
const OUT_OF_MEMORY: bool = false;
const QUEUE_SIZE: usize = 1000;

/*
  Type declarations
*/

#[derive(Clone, Copy)]
struct _NODE {
    dist: i32,
    prev: i32,
}

impl _NODE {
    const fn new() -> Self {
        _NODE {
            dist: 0,
            prev: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct _QITEM {
    node: usize,
    dist: i32,
    prev: i32,
    next: Option<usize>,
}

impl _QITEM {
    const fn new() -> Self {
        _QITEM {
            node: 0,
            dist: 0,
            prev: 0,
            next: None,
        }
    }
}

fn dijkstra_init(
    adj_matrix: &mut [[u8; NUM_NODES]; NUM_NODES],
    queue_count: &mut i32,
    queue_next: &mut usize,
    queue_head: &mut Option<usize>,
    checksum: &mut i32,
) {
    let x = 0;
    // loopbound min 100 max 100
    for (i, ary) in adj_matrix.iter_mut().enumerate() {
        // loopbound min 100 max 100
        for (j, el) in ary.iter_mut().enumerate() {
            *el = INPUT_ADJ_MATRIX[i][j] ^ x;
        }
    }
    *queue_count = 0;
    *queue_next = 0;
    *queue_head = None;
    *checksum = 0;
}

fn dijkstra_return(checksum: i32) -> bool {
    checksum == 25
}

fn dijkstra_enqueue(
    node: usize,
    dist: i32,
    prev: i32,
    queue_count: &mut i32,
    queue_next: &mut usize,
    queue_items: &mut [_QITEM; QUEUE_SIZE],
    queue_head: &mut Option<usize>,
) -> bool {
    let new_item = *queue_next;
    let mut last = *queue_head;

    // struct _QITEM *newItem = &dijkstra_queueItems[ dijkstra_queueNext ];
    // struct _QITEM *last = dijkstra_queueHead;

    *queue_next += 1;
    if *queue_next >= QUEUE_SIZE {
        return OUT_OF_MEMORY;
    }
    queue_items[new_item].node = node;
    queue_items[new_item].dist = dist;
    queue_items[new_item].prev = prev;
    queue_items[new_item].next = None;

    if last.is_none() {
        *queue_head = Some(new_item);
    } else {
        // loopbound min 0 max 1000
        while !queue_items[last.unwrap()].next.is_none() {
            last = queue_items[last.unwrap()].next;
        }
        queue_items[last.unwrap()].next = Some(new_item);
    }
    *queue_count += 1;
    true
}

fn dijkstra_dequeue(
    node: &mut usize,
    dist: &mut i32,
    prev: &mut i32,
    queue_count: &mut i32,
    queue_items: &mut [_QITEM; QUEUE_SIZE],
    queue_head: &mut Option<usize>,
) {
    if queue_head.is_some() {
        *node = queue_items[queue_head.unwrap()].node;
        *dist = queue_items[queue_head.unwrap()].dist;
        *prev = queue_items[queue_head.unwrap()].prev;
        *queue_head = queue_items[queue_head.unwrap()].next;
        *queue_count -= 1;
    }
}

fn dijkstra_find(
  ch_start: usize,
  ch_end: usize,
  adj_matrix: &mut [[u8; NUM_NODES]; NUM_NODES],
  rgn_nodes: &mut [_NODE; NUM_NODES],
  queue_count: &mut i32,
  queue_next: &mut usize,
  queue_items: &mut [_QITEM; QUEUE_SIZE],
  queue_head: &mut Option<usize>,
) -> bool {
    // int ch;
    // int prev, node = 0;
    // int cost, dist = 0;
    let mut node = 0;
    let mut dist = 0;
    let mut prev = 0;

    // loopbound min 100 max 100
    for node in rgn_nodes.iter_mut() {
        node.dist = NONE;
        node.prev = NONE;
    }

    if ch_start != ch_end {
        rgn_nodes[ch_start].dist = 0;
        rgn_nodes[ch_start].prev = NONE;

        let cond = dijkstra_enqueue(
            ch_start, 0, NONE,
            queue_count, queue_next,
            queue_items, queue_head,
        );
        if cond == OUT_OF_MEMORY {
            return OUT_OF_MEMORY;
        }
        
        // loopbound min 100 max 1000
        while *queue_count > 0 {
            dijkstra_dequeue(
                &mut node, &mut dist, &mut prev,
                queue_count, queue_items, queue_head
            );
            // loopbound min 100 max 100
            for i in 0..NUM_NODES {
                let cost = adj_matrix[node][i];
                if cost as i32 != NONE {
                    let lhs = NONE == rgn_nodes[ i ].dist;
                    let rhs = rgn_nodes[i].dist > (cost as i32 + dist);
                    if lhs || rhs {
                        rgn_nodes[i].dist = dist + cost as i32;
                        rgn_nodes[i].prev = node as i32;
                        let cond = dijkstra_enqueue(
                            i, dist + cost as i32, node as i32,
                            queue_count, queue_next, queue_items, queue_head,
                        );
                        if cond == OUT_OF_MEMORY {
                            return OUT_OF_MEMORY;
                        }
                    }
                }
            }
        }
    }
    true
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn dijkstra_main(
    adj_matrix: &mut [[u8; NUM_NODES]; NUM_NODES],
    rgn_nodes: &mut [_NODE; NUM_NODES],
    queue_count: &mut i32,
    queue_next: &mut usize,
    queue_items: &mut [_QITEM; QUEUE_SIZE],
    queue_head: &mut Option<usize>,
    checksum: &mut i32,
) {
    /* finds 20 shortest paths between nodes */
    // loopbound min 20 max 20
    let mut j = NUM_NODES / 2;
    for i in 0..20 {
        j %= NUM_NODES;
        let cond = dijkstra_find(
          i, j,
          adj_matrix,
          rgn_nodes,
          queue_count,
          queue_next,
          queue_items,
          queue_head
        );
        if cond == OUT_OF_MEMORY {
            *checksum += OUT_OF_MEMORY as i32;
            return;
        } else {
            *checksum += rgn_nodes[j].dist;
        }
        *queue_next = 0;
        j += 1;
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn dijkstra_log(
    adj_matrix: &mut [[u8; NUM_NODES]; NUM_NODES],
    rgn_nodes: &mut [_NODE; NUM_NODES],
    queue_count: &mut i32,
    queue_next: &mut usize,
    queue_items: &mut [_QITEM; QUEUE_SIZE],
    queue_head: &mut Option<usize>,
    checksum: &mut i32,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        dijkstra_init(
            black_box(adj_matrix),
            black_box(queue_count),
            black_box(queue_next),
            black_box(queue_head),
            black_box(checksum),
        );
        let start = black_box(DWT::cycle_count());
        dijkstra_main(
            black_box(adj_matrix),
            black_box(rgn_nodes),
            black_box(queue_count),
            black_box(queue_next),
            black_box(queue_items),
            black_box(queue_head),
            black_box(checksum),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("dijkstra - elapsed: {}", cycles);
    });

    black_box(dijkstra_return(black_box(*checksum)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn dijkstra_std(
    adj_matrix: &mut [[u8; NUM_NODES]; NUM_NODES],
    rgn_nodes: &mut [_NODE; NUM_NODES],
    queue_count: &mut i32,
    queue_next: &mut usize,
    queue_items: &mut [_QITEM; QUEUE_SIZE],
    queue_head: &mut Option<usize>,
    checksum: &mut i32,
) -> bool {
    dijkstra_init(
        black_box(adj_matrix),
        black_box(queue_count),
        black_box(queue_next),
        black_box(queue_head),
        black_box(checksum),
    );
    dijkstra_main(
        black_box(adj_matrix),
        black_box(rgn_nodes),
        black_box(queue_count),
        black_box(queue_next),
        black_box(queue_items),
        black_box(queue_head),
        black_box(checksum),
    );
    black_box(dijkstra_return(black_box(*checksum)))
}


/*
  Global variable definitions
*/

fn dijkstra() -> bool {
    static mut ADJ_MATRIX: [[u8; NUM_NODES]; NUM_NODES] = [[0; NUM_NODES]; NUM_NODES];
    static mut RGN_NODES: [_NODE; NUM_NODES] = [_NODE::new(); NUM_NODES];
    static mut QUEUE_COUNT: i32 = 0;
    static mut QUEUE_NEXT: usize = 0;
    static mut QUEUE_ITEMS: [_QITEM; QUEUE_SIZE] = [_QITEM::new(); QUEUE_SIZE];
    static mut QUEUE_HEAD: Option<usize> = None;
    static mut CHECKSUM: i32 = 0;

    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let adj_matrix = unsafe { &mut *{ &raw mut ADJ_MATRIX } };
    let rgn_nodes = unsafe { &mut *{ &raw mut RGN_NODES } };
    let queue_count = unsafe { &mut *{ &raw mut QUEUE_COUNT } };
    let queue_next = unsafe { &mut *{ &raw mut QUEUE_NEXT } };
    let queue_items = unsafe { &mut *{ &raw mut QUEUE_ITEMS } };
    let queue_head = unsafe { &mut *{ &raw mut QUEUE_HEAD } };
    let checksum = unsafe { &mut *{ &raw mut CHECKSUM } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(dijkstra_log(
            black_box(adj_matrix),
            black_box(rgn_nodes),
            black_box(queue_count),
            black_box(queue_next),
            black_box(queue_items),
            black_box(queue_head),
            black_box(checksum),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(dijkstra_std(
            black_box(adj_matrix),
            black_box(rgn_nodes),
            black_box(queue_count),
            black_box(queue_next),
            black_box(queue_items),
            black_box(queue_head),
            black_box(checksum),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(dijkstra());
    
    #[cfg(feature = "log")]
    {
        rprintln!("dijkstra: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(dijkstra());

    #[cfg(feature = "log")]
    std::println!("dijkstra: {}", success);

    std::process::exit(!success as i32);
}

