//! Bounded, allocation-free RAM VFS used as the first concrete filesystem backend.
//! Paths are absolute UTF-8 byte strings. Symlinks and mount namespaces are not
//! implemented here; traversal outside the root is rejected.
pub const MAX_NODES: usize = 64;
pub const MAX_NAME: usize = 31;
pub const MAX_FILE_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeId { pub slot: u16, pub generation: u32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind { Directory, File }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VfsError { InvalidPath, NameTooLong, NotFound, AlreadyExists, NotDirectory, IsDirectory, NotEmpty, NoSpace, FileTooLarge, InvalidNode }

#[derive(Clone, Copy)]
struct Node {
    generation: u32,
    used: bool,
    parent: u16,
    kind: NodeKind,
    name: [u8; MAX_NAME],
    name_len: u8,
    data: [u8; MAX_FILE_BYTES],
    data_len: u16,
}
impl Node {
    const EMPTY: Self = Self { generation: 1, used: false, parent: 0, kind: NodeKind::File,
        name: [0; MAX_NAME], name_len: 0, data: [0; MAX_FILE_BYTES], data_len: 0 };
}

pub struct RamFs { nodes: [Node; MAX_NODES] }
impl RamFs {
    pub const fn new() -> Self {
        let mut nodes = [Node::EMPTY; MAX_NODES];
        nodes[0] = Node { generation: 1, used: true, parent: 0, kind: NodeKind::Directory,
            name: [0; MAX_NAME], name_len: 0, data: [0; MAX_FILE_BYTES], data_len: 0 };
        Self { nodes }
    }
    fn id(&self, index: usize) -> NodeId { NodeId { slot: index as u16, generation: self.nodes[index].generation } }
    fn checked_index(&self, id: NodeId) -> Result<usize, VfsError> {
        let n = self.nodes.get(id.slot as usize).ok_or(VfsError::InvalidNode)?;
        if !n.used || n.generation != id.generation { return Err(VfsError::InvalidNode); }
        Ok(id.slot as usize)
    }
    /// Resolve a generation-tagged inode handle without trusting a stale slot index.
    pub fn kind_by_id(&self, id: NodeId) -> Result<NodeKind, VfsError> {
        let index = self.checked_index(id)?;
        Ok(self.nodes[index].kind)
    }

    fn find_child(&self, parent: usize, name: &[u8]) -> Option<usize> {
        self.nodes.iter().enumerate().position(|(i, n)| i != 0 && n.used
            && n.parent as usize == parent && n.name_len as usize == name.len()
            && &n.name[..name.len()] == name)
    }
    fn resolve(&self, path: &[u8]) -> Result<usize, VfsError> {
        if path.first() != Some(&b'/') || path.contains(&0) { return Err(VfsError::InvalidPath); }
        let mut current = 0usize;
        for component in path.split(|b| *b == b'/').filter(|part| !part.is_empty()) {
            if component == b"." { continue; }
            if component == b".." { return Err(VfsError::InvalidPath); }
            if component.len() > MAX_NAME { return Err(VfsError::NameTooLong); }
            if self.nodes[current].kind != NodeKind::Directory { return Err(VfsError::NotDirectory); }
            current = self.find_child(current, component).ok_or(VfsError::NotFound)?;
        }
        Ok(current)
    }
    // Return an owned component so create_node can mutate the node table without
    // retaining an immutable borrow into the caller's path or this filesystem.
    fn parent_and_name(&self, path: &[u8]) -> Result<(usize, [u8; MAX_NAME], usize), VfsError> {
        if path.first() != Some(&b'/') || path.contains(&0) || path.len() < 2
            || path.last() == Some(&b'/') { return Err(VfsError::InvalidPath); }
        let end = path.len();
        let start = path[..end].iter().rposition(|b| *b == b'/').ok_or(VfsError::InvalidPath)? + 1;
        let component = &path[start..end];
        if component.is_empty() || component == b"." || component == b".." { return Err(VfsError::InvalidPath); }
        if component.len() > MAX_NAME { return Err(VfsError::NameTooLong); }
        let parent_end = if start == 1 { 1 } else { start - 1 };
        let parent = self.resolve(&path[..parent_end])?;
        if self.nodes[parent].kind != NodeKind::Directory { return Err(VfsError::NotDirectory); }
        let mut name = [0; MAX_NAME];
        name[..component.len()].copy_from_slice(component);
        Ok((parent, name, component.len()))
    }
    fn create_node(&mut self, path: &[u8], kind: NodeKind) -> Result<NodeId, VfsError> {
        let (parent, name, name_len) = self.parent_and_name(path)?;
        if self.find_child(parent, &name[..name_len]).is_some() { return Err(VfsError::AlreadyExists); }
        let index = self.nodes.iter().position(|n| !n.used).ok_or(VfsError::NoSpace)?;
        let node = &mut self.nodes[index];
        node.used = true; node.parent = parent as u16; node.kind = kind;
        node.name[..name_len].copy_from_slice(&name[..name_len]); node.name_len = name_len as u8;
        node.data_len = 0;
        Ok(self.id(index))
    }
    pub fn create_dir(&mut self, path: &[u8]) -> Result<NodeId, VfsError> { self.create_node(path, NodeKind::Directory) }
    pub fn create_file(&mut self, path: &[u8]) -> Result<NodeId, VfsError> { self.create_node(path, NodeKind::File) }
    pub fn lookup(&self, path: &[u8]) -> Result<(NodeId, NodeKind), VfsError> {
        let index = self.resolve(path)?;
        Ok((self.id(index), self.nodes[index].kind))
    }
    pub fn read(&self, path: &[u8], offset: usize, out: &mut [u8]) -> Result<usize, VfsError> {
        let index = self.resolve(path)?;
        let node = &self.nodes[index];
        if node.kind == NodeKind::Directory { return Err(VfsError::IsDirectory); }
        let start = offset.min(node.data_len as usize);
        let count = out.len().min(node.data_len as usize - start);
        out[..count].copy_from_slice(&node.data[start..start + count]);
        Ok(count)
    }
    pub fn write(&mut self, path: &[u8], offset: usize, bytes: &[u8]) -> Result<usize, VfsError> {
        let index = self.resolve(path)?;
        let node = &mut self.nodes[index];
        if node.kind == NodeKind::Directory { return Err(VfsError::IsDirectory); }
        let end = offset.checked_add(bytes.len()).ok_or(VfsError::FileTooLarge)?;
        if end > MAX_FILE_BYTES { return Err(VfsError::FileTooLarge); }
        node.data[offset..end].copy_from_slice(bytes);
        node.data_len = node.data_len.max(end as u16);
        Ok(bytes.len())
    }
    pub fn remove(&mut self, path: &[u8]) -> Result<(), VfsError> {
        let index = self.resolve(path)?;
        if index == 0 { return Err(VfsError::InvalidPath); }
        if self.nodes[index].kind == NodeKind::Directory
            && self.nodes.iter().any(|n| n.used && n.parent as usize == index) { return Err(VfsError::NotEmpty); }
        let node = &mut self.nodes[index];
        node.used = false; node.name = [0; MAX_NAME]; node.name_len = 0; node.data = [0; MAX_FILE_BYTES]; node.data_len = 0;
        node.generation = node.generation.wrapping_add(1).max(1);
        Ok(())
    }
    /// Read through a generation-tagged inode handle (suitable for capability-backed VFS handles).
    pub fn read_node(&self, id: NodeId, offset: usize, out: &mut [u8]) -> Result<usize, VfsError> {
        let index = self.checked_index(id)?;
        let node = &self.nodes[index];
        if node.kind == NodeKind::Directory { return Err(VfsError::IsDirectory); }
        let start = offset.min(node.data_len as usize);
        let count = out.len().min(node.data_len as usize - start);
        out[..count].copy_from_slice(&node.data[start..start + count]);
        Ok(count)
    }

    /// Write through a generation-tagged inode handle.
    pub fn write_node(&mut self, id: NodeId, offset: usize, bytes: &[u8]) -> Result<usize, VfsError> {
        let index = self.checked_index(id)?;
        let node = &mut self.nodes[index];
        if node.kind == NodeKind::Directory { return Err(VfsError::IsDirectory); }
        let end = offset.checked_add(bytes.len()).ok_or(VfsError::FileTooLarge)?;
        if end > MAX_FILE_BYTES { return Err(VfsError::FileTooLarge); }
        node.data[offset..end].copy_from_slice(bytes);
        node.data_len = node.data_len.max(end as u16);
        Ok(bytes.len())
    }

    /// Remove through a generation-tagged inode handle, rejecting stale handles.
    pub fn remove_node(&mut self, id: NodeId) -> Result<(), VfsError> {
        let index = self.checked_index(id)?;
        if index == 0 { return Err(VfsError::InvalidPath); }
        if self.nodes[index].kind == NodeKind::Directory
            && self.nodes.iter().any(|n| n.used && n.parent as usize == index) {
            return Err(VfsError::NotEmpty);
        }
        let node = &mut self.nodes[index];
        node.used = false;
        node.name = [0; MAX_NAME];
        node.name_len = 0;
        node.data = [0; MAX_FILE_BYTES];
        node.data_len = 0;
        node.generation = node.generation.wrapping_add(1).max(1);
        Ok(())
    }

    pub fn node_count(&self) -> usize { self.nodes.iter().filter(|n| n.used).count() }
}
impl Default for RamFs { fn default() -> Self { Self::new() } }
