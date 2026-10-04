use memmap2::{MmapMut, MmapOptions};
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::path::Path;
use rkyv::{Archive, Deserialize, Serialize};
use bytecheck::CheckBytes;

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(check_bytes)]
pub struct ZeroCopyDiffLine {
    pub tag: String, // "+", "-", " "
    pub content: String,
    pub old_lineno: Option<usize>,
    pub new_lineno: Option<usize>,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(check_bytes)]
pub struct ZeroCopyDiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<ZeroCopyDiffLine>,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(check_bytes)]
pub struct DiffFileState {
    pub path: String,
    pub status: String,
    pub attribution: String,
    pub current_hash: u64,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: Vec<ZeroCopyDiffHunk>,
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
#[archive(check_bytes)]
pub struct IdeState {
    pub total_additions: usize,
    pub total_deletions: usize,
    pub changed_files: Vec<DiffFileState>,
}

pub const SHM_SIZE: usize = 1024 * 1024 * 4; // 4MB should be plenty for diffs
pub const SHM_PATH: &str = "/dev/shm/minimal_ide_shm";

/// Layout of the shared memory:
/// offset 0: sequence number (u64, atomic)
/// offset 8: payload size (u32)
/// offset 12: padding
/// offset 16: rkyv serialized payload bytes
pub struct ShmBuffer {
    mmap: MmapMut,
}

impl ShmBuffer {
    pub fn open_or_create() -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(0o666)
            .open(Path::new(SHM_PATH))?;
        file.set_len(SHM_SIZE as u64)?;
        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };
        Ok(Self { mmap })
    }

    pub fn open_read_only() -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true) // Needs write for PROT_WRITE if we wanted, but we map mut anyway
            .open(Path::new(SHM_PATH))?;
        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };
        Ok(Self { mmap })
    }

    #[inline]
    fn seq_ptr(&self) -> &AtomicU64 {
        unsafe { &*(self.mmap.as_ptr() as *const AtomicU64) }
    }

    #[inline]
    fn len_ptr(&self) -> &mut u32 {
        unsafe { &mut *(self.mmap.as_ptr().add(8) as *mut u32) }
    }

    #[inline]
    fn payload_ptr(&self) -> *mut u8 {
        unsafe { self.mmap.as_ptr().add(16) as *mut u8 }
    }

    pub fn write_payload(&mut self, payload: &[u8]) {
        assert!(payload.len() <= SHM_SIZE - 16, "Payload too large");

        // Seq logic: even = stable, odd = writing.
        // Writer sets to odd before writing, then to even (seq + 1) after writing.
        let seq = self.seq_ptr().load(Ordering::Relaxed);
        let next_seq = (seq + 1) | 1; // force odd
        self.seq_ptr().store(next_seq, Ordering::Release);

        unsafe {
            *self.len_ptr() = payload.len() as u32;
            std::ptr::copy_nonoverlapping(payload.as_ptr(), self.payload_ptr(), payload.len());
        }

        self.seq_ptr().store(next_seq + 1, Ordering::Release);
    }

    /// Reads the payload, returning it if the sequence number has changed since `last_seq`.
    /// Also returns the new sequence number.
    /// Uses seqlock logic to retry if the writer is currently writing.
    pub fn read_payload(&self, last_seq: u64) -> Option<(u64, Vec<u8>)> {
        loop {
            let seq1 = self.seq_ptr().load(Ordering::Acquire);

            // If seq is odd, a write is in progress.
            if seq1 & 1 != 0 {
                std::hint::spin_loop();
                continue;
            }

            if seq1 == last_seq {
                return None; // No new data
            }

            let len = unsafe { *self.len_ptr() } as usize;
            if len > SHM_SIZE - 16 {
                // Corrupt or invalid length
                return None;
            }

            let mut buf = vec![0u8; len];
            unsafe {
                std::ptr::copy_nonoverlapping(self.payload_ptr(), buf.as_mut_ptr(), len);
            }

            let seq2 = self.seq_ptr().load(Ordering::Acquire);
            if seq1 == seq2 {
                // Data read successfully and is consistent
                return Some((seq1, buf));
            }
            // Otherwise, writer overwrote while we were reading. Retry.
        }
    }
}
