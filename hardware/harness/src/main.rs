#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::arch::asm;

include!(concat!(env!("OUT_DIR"), "/embedded_stub.rs"));

// This can be any import from the `psx` crate
// but there must be at least one.
use psx;
use psx::println;

fn move_to_uncached_memory() {
    // Resume execution from an uncached memory region.
    // We just need to return from this function, but change the RA register
    // to point to the uncached memory region.
    unsafe {
        asm!(
            "lui $t0, 0xa000",
            "or $ra, $ra, $t0",
        )
    }
}

fn copy(test: &'static [u8]) -> extern "C" fn() -> u32 {
    unsafe {
        let dest = 0x8004_0000 as *mut u8;
        for (i, &byte) in test.iter().enumerate() {
            dest.add(i).write_volatile(byte);
        }

        let func: extern "C" fn() -> u32 = core::mem::transmute(dest);
        func
    }
}

/// Ensures interrupts are disabled, returning the previous status.
fn disable_interrupts() -> bool {
    unsafe {
        let mut status: u32;
        asm!("mfc0 {}, $12",
             "nop",
             out(reg) status
        );
        let was_enabled = (status & 0x1) != 0; // Check the IE bit (bit 0)
        status &= !0x1; // Clear the IE bit (bit 0)
        asm!("mtc0 {}, $12", in(reg) status);

        was_enabled
    }
}

fn restore_interrupts(was_enabled: bool) {
    if !was_enabled {
        return; // No need to restore if interrupts were not enabled
    }

    unsafe {
        let mut status: u32;
        asm!("mfc0 {}, $12", 
             "nop",
             out(reg) status
        );
        status |= 0x1; // Set the IE bit (bit 0)
        asm!("mtc0 {}, $12", in(reg) status);
    }
}

#[unsafe(no_mangle)]
fn main() {
    println!("Hello, PSX world!");

    move_to_uncached_memory();

    let func = copy(TEST_STUB);

    let interrupts = disable_interrupts();

    let mut buckets: [u16; 50] = [0; 50];

    // Run once to fill the cache lines
    func();

    for _ in 0..1000 {
        let val = func();
        if val < 50 {
            buckets[val as usize] += 1;
        } else {
            println!("Value out of range: {}", val);
        }
    }

    restore_interrupts(interrupts);

    for (i, &count) in buckets.iter().enumerate() {
        if count > 0 {
            println!("Bucket {}: {}", i, count);
        }
    }

    println!("Test completed. Halting...");
}

