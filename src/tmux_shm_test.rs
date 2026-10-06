use memmap2::Mmap;
use rkyv::{
    archived_root,
    ser::{serializers::AllocSerializer, Serializer},
};
use std::fs::File;

// Import our schema
#[path = "tmux/schema.rs"]
pub mod schema;
use schema::SpatialNode;

pub fn write_layout(layout: &SpatialNode) -> std::io::Result<()> {
    let mut serializer = AllocSerializer::<4096>::default();
    serializer.serialize_value(layout).unwrap();
    let bytes = serializer.into_serializer().into_inner();
    std::fs::write("/dev/shm/minimal_tmux_layout.tmp", bytes)?;
    std::fs::rename("/dev/shm/minimal_tmux_layout.tmp", "/dev/shm/minimal_tmux_layout.bin")?;
    Ok(())
}

pub struct LayoutMap {
    mmap: Mmap,
}

impl LayoutMap {
    pub fn new() -> std::io::Result<Self> {
        let file = File::open("/dev/shm/minimal_tmux_layout.bin")?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { mmap })
    }

    pub fn get(&self) -> &schema::ArchivedSpatialNode {
        unsafe { archived_root::<SpatialNode>(&self.mmap) }
    }
}
