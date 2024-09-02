pub type BlockInfo = (u32, &'static str);

// TODO: Should be persisted (and ideally replicated) somewhere: db, file, etc...
static mut BLOCK_INFO: BlockInfo = (
    859543u32,
    "00000000000000000002a5b47ba711c12593f2054a7ab1e2b6c7d8a19859e317",
);

// TODO: Mocks of DB interactions.
pub fn read_block_info() -> Option<BlockInfo> {
    let mut block_info = None;
    unsafe {
        block_info = Some(BLOCK_INFO.clone());
    }
    block_info
}

pub fn write_block_info(block_info: BlockInfo) {
    unsafe {
        BLOCK_INFO = block_info;
    }
}
