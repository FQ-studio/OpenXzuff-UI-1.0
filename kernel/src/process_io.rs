//! Process-owned capability handles backed by the bounded RAM VFS.
//!
//! This layer joins ProcessId ownership, generation-safe handles and
//! generation-safe VFS node IDs. It is not a persistent filesystem or syscall
//! dispatcher; callers must provide the current process identity.
use crate::handle::{HandleError, HandleId, HandleTable, ObjectKind, Rights};
use crate::process::ProcessId;
use crate::vfs::{NodeId, NodeKind, RamFs, VfsError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessIoError {
    Handle(HandleError),
    Vfs(VfsError),
    InvalidObjectId,
}

pub struct ProcessIo {
    handles: HandleTable,
    fs: RamFs,
}

impl ProcessIo {
    pub const fn new() -> Self {
        Self {
            handles: HandleTable::new(),
            fs: RamFs::new(),
        }
    }

    fn encode_node(id: NodeId) -> u64 {
        ((id.generation as u64) << 16) | id.slot as u64
    }

    fn decode_node(value: u64) -> Result<NodeId, ProcessIoError> {
        let generation = (value >> 16) as u32;
        if value >> 48 != 0 || generation == 0 {
            return Err(ProcessIoError::InvalidObjectId);
        }
        Ok(NodeId { slot: value as u16, generation })
    }

    /// Create a file and return a handle owned by the supplied process.
    /// If handle allocation fails, the new inode is rolled back.
    pub fn create_file(
        &mut self,
        owner: ProcessId,
        path: &[u8],
        rights: Rights,
    ) -> Result<HandleId, ProcessIoError> {
        let node = self.fs.create_file(path).map_err(ProcessIoError::Vfs)?;
        match self.handles.insert(owner, ObjectKind::File, Self::encode_node(node), rights) {
            Ok(handle) => Ok(handle),
            Err(error) => {
                let _ = self.fs.remove_node(node);
                Err(ProcessIoError::Handle(error))
            }
        }
    }

    /// Open an existing regular file with the requested capability rights.
    pub fn open_file(
        &mut self,
        owner: ProcessId,
        path: &[u8],
        rights: Rights,
    ) -> Result<HandleId, ProcessIoError> {
        let (node, kind) = self.fs.lookup(path).map_err(ProcessIoError::Vfs)?;
        if kind != NodeKind::File {
            return Err(ProcessIoError::Vfs(VfsError::IsDirectory));
        }
        self.handles.insert(owner, ObjectKind::File, Self::encode_node(node), rights)
            .map_err(ProcessIoError::Handle)
    }

    /// Explicitly transfer a reduced-rights capability to another process.
    pub fn duplicate(
        &mut self,
        caller: ProcessId,
        handle: HandleId,
        target: ProcessId,
        rights: Rights,
    ) -> Result<HandleId, ProcessIoError> {
        self.handles.duplicate(caller, handle, target, rights)
            .map_err(ProcessIoError::Handle)
    }

    fn checked_node(
        &self,
        owner: ProcessId,
        handle: HandleId,
        rights: Rights,
    ) -> Result<NodeId, ProcessIoError> {
        let info = self.handles.get(owner, handle, Some(ObjectKind::File), rights)
            .map_err(ProcessIoError::Handle)?;
        Self::decode_node(info.object_id)
    }

    pub fn read(
        &self,
        owner: ProcessId,
        handle: HandleId,
        offset: usize,
        output: &mut [u8],
    ) -> Result<usize, ProcessIoError> {
        let node = self.checked_node(owner, handle, Rights::READ)?;
        self.fs.read_node(node, offset, output).map_err(ProcessIoError::Vfs)
    }

    pub fn write(
        &mut self,
        owner: ProcessId,
        handle: HandleId,
        offset: usize,
        bytes: &[u8],
    ) -> Result<usize, ProcessIoError> {
        let node = self.checked_node(owner, handle, Rights::WRITE)?;
        self.fs.write_node(node, offset, bytes).map_err(ProcessIoError::Vfs)
    }

    pub fn close(&mut self, owner: ProcessId, handle: HandleId) -> Result<(), ProcessIoError> {
        self.handles.close(owner, handle).map_err(ProcessIoError::Handle)
    }

    /// Release every file capability owned by a terminating process.
    /// The RAM filesystem nodes remain mounted; only process handles are closed.
    pub fn close_all(&mut self, owner: ProcessId) -> usize {
        self.handles.close_all(owner)
    }

    pub fn open_count(&self, owner: ProcessId) -> usize {
        self.handles.open_count(owner)
    }
}

impl Default for ProcessIo {
    fn default() -> Self { Self::new() }
}
