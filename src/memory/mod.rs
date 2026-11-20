use limine::memmap::Entry;

pub mod pmm;
mod heap;

pub fn init(hhdm: u64, entries: &[&Entry]){
    let (heap_phys,heap_size) = pmm::init(hhdm, entries);
    heap::init((hhdm + heap_phys) as usize, heap_size as usize);
}
