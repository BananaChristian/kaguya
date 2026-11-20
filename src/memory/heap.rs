use linked_list_allocator::LockedHeap;

#[global_allocator] 
static  ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init(virt_start: usize, size: usize){
    unsafe {
        ALLOCATOR.lock().init(virt_start as *mut u8, size);
    }
}
