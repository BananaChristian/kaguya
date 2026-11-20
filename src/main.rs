#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;

use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker, request::{HhdmRequest, MemmapRequest}};

use crate::interrupts::initialize_interrupts;

mod gdt;
mod interrupts;
mod serial;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION : BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST :  HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMMAP_REQUEST : MemmapRequest = MemmapRequest::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER : RequestsEndMarker = RequestsEndMarker::new();


#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    assert!(BASE_REVISION.is_supported());

    let hhdm = HHDM_REQUEST.response().expect("no HHDM response").offset;
    let memmap = MEMMAP_REQUEST.response().expect("no memory map response");


    println!("[OK] Kaguya core initialized.");
    println!("[OK] Serial console up (COM1)");
    println!("[OK] HHDM: {:#x}", hhdm);
    println!("[OK] Memory map: {} entries", memmap.entries().len());

    gdt::init();
    println!("[OK] GDT & TSS initialized (Double Fault IST active).");

    // Initialize IDT and hardware interrupts
    initialize_interrupts();
    println!("[OK] IDT & PIC initialized.");

    // Sleep state
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
