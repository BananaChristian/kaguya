# 0002  Today I aim to change the bootloader from using the bootloader crate in Rust to the limine bootloader

# Goal
Switching the bootloader to using limine

I removed the old bootloader crate from the cargo.
Added the linker script for the a limine complaint kernel
I got the linker script from the limine-rust-template repo as linked here [[limine-rust-template](https://github.com/jasondyoungberg/limine-rust-template/blob/trunk/kernel/linker-x86_64.ld)]

Removed the vga driver well given my new design direction it should not sit in the kernel it will later be added back in the OS when the kernel gets an abi
It was causing triple faults reason being when I switched the kernel to be limine complaint the vga buffer was at 0xb800 but limine uses higher half paging meaning  that address for vga is not the same virtual address causing a triple fault(I dont have handlers for such issues yet)

Now the kernel prints using COM1 serial ports 
