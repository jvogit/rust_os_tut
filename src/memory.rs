use x86_64::{
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable},
    VirtAddr,
};

pub unsafe fn init(physical_offset: VirtAddr) -> OffsetPageTable<'static> {
    OffsetPageTable::new(active_level_4_page_table(physical_offset), physical_offset)
}

pub unsafe fn active_level_4_page_table(physical_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_page_frame, _flags) = Cr3::read();
    let phys_addr = level_4_page_frame.start_address();
    let virtual_addr = physical_offset + phys_addr.as_u64();
    let phys_addr: *mut PageTable = virtual_addr.as_mut_ptr();

    &mut *phys_addr // this is unsafe
}
