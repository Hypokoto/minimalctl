use super::schema::SpatialNode;
use memmap2::Mmap;
use rkyv::{
    check_archived_root,
    ser::{serializers::AllocSerializer, Serializer},
};
use std::fs::{File, OpenOptions, DirBuilder};
use std::path::PathBuf;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};

fn get_shm_dir() -> PathBuf {
    if let Some(xdg_runtime) = std::env::var_os("XDG_RUNTIME_DIR") {
        PathBuf::from(xdg_runtime).join("minimal_tmux")
    } else {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/minimal_tmux_{}", uid))
    }
}

pub fn write_layout(layout: &SpatialNode) -> std::io::Result<()> {
    let mut serializer = AllocSerializer::<4096>::default();
    serializer.serialize_value(layout).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
    })?;
    let bytes = serializer.into_serializer().into_inner();

    let dir = get_shm_dir();

    if !dir.exists() {
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&dir)?;
    }

    let tmp_path = dir.join("layout.tmp");
    let bin_path = dir.join("layout.bin");

    let _ = std::fs::remove_file(&tmp_path);

    let mut tmp_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp_path)?;

    std::io::Write::write_all(&mut tmp_file, &bytes)?;
    tmp_file.sync_all()?;

    std::fs::rename(tmp_path, bin_path)?;
    Ok(())
}

pub struct LayoutMap {
    mmap: Mmap,
}

impl LayoutMap {
    pub fn new() -> std::io::Result<Self> {
        let bin_path = get_shm_dir().join("layout.bin");
        let file = File::open(&bin_path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { mmap })
    }

    pub fn get(&self) -> Result<&super::schema::ArchivedSpatialNode, String> {
        check_archived_root::<SpatialNode>(&self.mmap).map_err(|e| e.to_string())
    }
}
