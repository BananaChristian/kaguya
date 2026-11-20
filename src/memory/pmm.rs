use core::ptr::write_bytes;

use limine::memmap::{Entry, MEMMAP_USABLE};
use spin::{Mutex, Once};

const FRAME_SIZE: u64 = 4096;
const HEAP_SIZE: u64 = 4 * 1024 * 1024;

fn align_up(x: u64, a: u64) -> u64 {
    (x + a - 1) & !(a - 1)
}

fn align_down(x: u64, a: u64) -> u64 {
    x & !(a - 1)
}

fn frame_index(phys: u64) -> u64 {
    phys / FRAME_SIZE
}

fn phys_of(index: u64) -> u64 {
    index * FRAME_SIZE
}

struct BitmapAllocator {
    hhdm: u64,
    bitmap: *mut u8,
    total_frames: u64,
    used_frames: u64,
    next_hint: u64,
}

unsafe impl Send for BitmapAllocator {}
unsafe impl Sync for BitmapAllocator {}

impl BitmapAllocator {
    fn bit_ptr(&self, index: u64) -> *mut u8 {
        unsafe { self.bitmap.add((index / 8) as usize) }
    }

    fn is_free(&self, index: u64) -> bool {
        unsafe { *self.bit_ptr(index) & (1u8 << (index % 8)) == 0 }
    }

    fn mark_used(&mut self, index: u64) {
        let ptr = self.bit_ptr(index);
        unsafe { *ptr |= 1u8 << (index % 8) };
    }

    fn mark_free(&mut self, index: u64) {
        let ptr = self.bit_ptr(index);
        unsafe { *ptr &= !(1u8 << (index % 8)) }
    }
}

static PMM: Once<Mutex<BitmapAllocator>> = Once::new();

pub fn init(hhdm: u64, entries: &[&Entry]) -> (u64, u64) {
    let mut max_phys = 0u64;
    for e in entries {
        if e.type_ != MEMMAP_USABLE {
            continue;
        }
        let end = e.base.saturating_add(e.length);
        if end > max_phys {
            max_phys = end;
        }
    }

    let total_frames = max_phys / FRAME_SIZE;
    let bitmap_bytes = align_up(((total_frames + 7) / 8) as u64, 8) as usize;

    let needed = align_up(bitmap_bytes as u64, FRAME_SIZE) + HEAP_SIZE;

    let mut chosen = None;
    for e in entries {
        if e.type_ != MEMMAP_USABLE {
            continue;
        }
        let start = align_up(e.base, FRAME_SIZE);
        let end = align_down(e.base.saturating_add(e.length), FRAME_SIZE);

        if end >= start.saturating_add(needed) {
            chosen = Some((start, end));
            break;
        }
    }

    let (region_start, region_end) = chosen.expect("No usable region big enough for bitmap + heap");
    let bitmap_phys = align_up(region_start, FRAME_SIZE);
    let heap_phys = align_up(bitmap_phys + bitmap_bytes as u64, FRAME_SIZE);

    assert!(heap_phys + HEAP_SIZE <= region_end, "heap does not fit");

    let bitmap_virt = hhdm + bitmap_phys;
    unsafe {
        write_bytes(bitmap_virt as *mut u8, 0xFF, bitmap_bytes);
    }

    let mut allocator = BitmapAllocator {
        hhdm,
        bitmap: bitmap_virt as *mut u8,
        total_frames,
        used_frames: total_frames,
        next_hint: 0,
    };

    let bitmap_start = frame_index(bitmap_phys);
    let bitmap_end = frame_index(align_up(bitmap_phys + bitmap_bytes as u64, FRAME_SIZE));
    let heap_start = frame_index(heap_phys);
    let heap_end = frame_index(heap_phys + HEAP_SIZE);

    for e in entries {
        if e.type_ != MEMMAP_USABLE {
            continue;
        }

        let start = align_up(e.base, FRAME_SIZE);
        let end = align_down(e.base.saturating_add(e.length), FRAME_SIZE);

        let mut f = frame_index(start);
        let f_end = frame_index(end);

        while f < f_end {
            let reserved = f == 0
                || (f >= bitmap_start && f < bitmap_end)
                || (f >= heap_start && f < heap_end);

            if f < total_frames && !reserved {
                allocator.mark_free(f);
                allocator.used_frames -= 1;
            }
            f += 1;
        }
    }

    PMM.call_once(|| Mutex::new(allocator));
    (heap_phys, HEAP_SIZE)
}

pub fn alloc_frame() -> Option<u64> {
    let mut pmm = PMM.get().expect("PMM not initialized").lock();
    let total = pmm.total_frames;
    let mut i = pmm.next_hint;

    for _ in 0..total {
        if i >= total {
            i = 0;
        }

        if pmm.is_free(i) {
            pmm.mark_used(i);
            pmm.used_frames += 1;
            pmm.next_hint = i + 1;
            return Some(phys_of(i));
        }
        i += 1;
    }
    None
}

pub fn free_frame(phys: u64) {
    let mut pmm = PMM.get().expect("PMM not initialized").lock();
    let idx = frame_index(phys);

    assert_eq!(phys_of(idx), phys, "free_frame: unaligned address");
    assert!(idx < pmm.total_frames, "free_frame: out of range");
    assert!(!pmm.is_free(idx), "free_frame: double free");

    pmm.mark_free(idx);
    pmm.used_frames -= 1;
}

pub fn alloc_frame_zeroed() -> Option<u64> {
    let frame = alloc_frame()?;
    let pmm = PMM.get().expect("PMM not initialized").lock();
    unsafe {
        write_bytes((pmm.hhdm + frame) as *mut u8, 0, FRAME_SIZE as usize);
    }
    Some(frame)
}

pub fn used_frames() -> u64 {
    let pmm = PMM.get().expect("PMM not initialized").lock();
    pmm.used_frames
}
