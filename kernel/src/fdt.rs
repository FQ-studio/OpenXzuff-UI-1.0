use crate::memory::MemoryMap;

const MAGIC: u32 = 0xd00d_feed;
const BEGIN_NODE: u32 = 1;
const END_NODE: u32 = 2;
const PROP: u32 = 3;
const NOP: u32 = 4;
const END: u32 = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FdtError { Header, Bounds, Structure, Cells, Memory, Reservation }

fn be32(b: &[u8], p: usize) -> Result<u32, FdtError> {
    let s = b.get(p..p.checked_add(4).ok_or(FdtError::Bounds)?).ok_or(FdtError::Bounds)?;
    Ok(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}
fn read_cells(b: &[u8], p: usize, cells: u32) -> Result<u64, FdtError> {
    let mut value = 0u64;
    for i in 0..cells {
        value = value.checked_shl(32).ok_or(FdtError::Bounds)? | be32(b, p + i as usize * 4)? as u64;
    }
    Ok(value)
}
fn align4(v: usize) -> Result<usize, FdtError> {
    v.checked_add(3).map(|n| n & !3).ok_or(FdtError::Bounds)
}
fn name_eq(bytes: &[u8], expected: &[u8]) -> bool {
    bytes.strip_suffix(&[0]).unwrap_or(bytes) == expected
}
fn node_is_memory(name: &[u8]) -> bool {
    name.strip_suffix(&[0]).unwrap_or(name).split(|b| *b == b'@').next() == Some(&b"memory"[..])
}
fn prop_name(strings: &[u8], offset: u32) -> Result<&[u8], FdtError> {
    let tail = strings.get(offset as usize..).ok_or(FdtError::Bounds)?;
    let n = tail.iter().position(|b| *b == 0).ok_or(FdtError::Structure)?;
    Ok(&tail[..n])
}

/// Parse firmware FDT into usable RAM ranges and apply its memreserve entries.
/// Kernel image and the complete FDT blob must be reserved by the caller.
pub fn parse_memory(blob: &[u8], map: &mut MemoryMap) -> Result<usize, FdtError> {
    if blob.len() < 40 || be32(blob, 0)? != MAGIC { return Err(FdtError::Header); }
    let total = be32(blob, 4)? as usize;
    let off_struct = be32(blob, 8)? as usize;
    let off_strings = be32(blob, 12)? as usize;
    let off_reserve = be32(blob, 16)? as usize;
    let size_strings = be32(blob, 32)? as usize;
    let size_struct = be32(blob, 36)? as usize;
    if total < 40 || total > blob.len()
        || off_struct.checked_add(size_struct).filter(|v| *v <= total).is_none()
        || off_strings.checked_add(size_strings).filter(|v| *v <= total).is_none()
        || off_reserve < 40 || off_reserve >= total { return Err(FdtError::Bounds); }
    let strings = &blob[off_strings..off_strings + size_strings];
    let struct_end = off_struct + size_struct;

    // Pass one: root cell widths define each memory reg tuple.
    let (mut address_cells, mut size_cells) = (2u32, 1u32);
    let (mut cursor, mut depth, mut root_seen) = (off_struct, 0usize, false);
    while cursor + 4 <= struct_end {
        let token = be32(blob, cursor)?; cursor += 4;
        match token {
            BEGIN_NODE => {
                let end = blob.get(cursor..struct_end).ok_or(FdtError::Bounds)?
                    .iter().position(|b| *b == 0).map(|n| cursor + n).ok_or(FdtError::Structure)?;
                if depth == 0 { root_seen = true; }
                cursor = align4(end.checked_add(1).ok_or(FdtError::Bounds)?)?;
                depth += 1;
            }
            END_NODE => { if depth == 0 { return Err(FdtError::Structure); } depth -= 1; }
            PROP => {
                if cursor + 8 > struct_end || depth == 0 { return Err(FdtError::Structure); }
                let len = be32(blob, cursor)? as usize;
                let nameoff = be32(blob, cursor + 4)?;
                cursor += 8;
                let value_end = cursor.checked_add(len).filter(|v| *v <= struct_end).ok_or(FdtError::Bounds)?;
                let name = prop_name(strings, nameoff)?;
                if depth == 1 && name == b"#address-cells" {
                    if len != 4 { return Err(FdtError::Cells); }
                    address_cells = be32(blob, cursor)?;
                } else if depth == 1 && name == b"#size-cells" {
                    if len != 4 { return Err(FdtError::Cells); }
                    size_cells = be32(blob, cursor)?;
                }
                cursor = align4(value_end)?;
            }
            NOP => {}
            END => break,
            _ => return Err(FdtError::Structure),
        }
        if cursor > struct_end { return Err(FdtError::Bounds); }
    }
    if !root_seen || !(1..=2).contains(&address_cells) || !(1..=2).contains(&size_cells) {
        return Err(FdtError::Cells);
    }

    // Pass two: hold node properties until END_NODE so reg tuples are complete.
    let mut node_memory = [false; 32];
    let mut node_device_memory = [false; 32];
    let mut node_reg: [Option<(usize, usize)>; 32] = [None; 32];
    cursor = off_struct; depth = 0;
    let mut found_memory = false;
    while cursor + 4 <= struct_end {
        let token = be32(blob, cursor)?; cursor += 4;
        match token {
            BEGIN_NODE => {
                if depth >= 32 { return Err(FdtError::Structure); }
                let end = blob.get(cursor..struct_end).ok_or(FdtError::Bounds)?
                    .iter().position(|b| *b == 0).map(|n| cursor + n).ok_or(FdtError::Structure)?;
                node_memory[depth] = node_is_memory(&blob[cursor..end]);
                node_device_memory[depth] = false;
                node_reg[depth] = None;
                cursor = align4(end.checked_add(1).ok_or(FdtError::Bounds)?)?;
                depth += 1;
            }
            END_NODE => {
                if depth == 0 { return Err(FdtError::Structure); }
                depth -= 1;
                if node_memory[depth] || node_device_memory[depth] {
                    let (p, len) = node_reg[depth].ok_or(FdtError::Memory)?;
                    let tuple_cells = address_cells + size_cells;
                    let tuple_bytes = tuple_cells as usize * 4;
                    if tuple_bytes == 0 || len == 0 || len % tuple_bytes != 0 { return Err(FdtError::Cells); }
                    let mut off = 0;
                    while off < len {
                        let base = read_cells(blob, p + off, address_cells)?;
                        let size = read_cells(blob, p + off + address_cells as usize * 4, size_cells)?;
                        if size == 0 { return Err(FdtError::Memory); }
                        let end = base.checked_add(size).ok_or(FdtError::Bounds)?;
                        let start = usize::try_from(base).map_err(|_| FdtError::Bounds)?;
                        let end = usize::try_from(end).map_err(|_| FdtError::Bounds)?;
                        map.add(start, end).map_err(|_| FdtError::Memory)?;
                        found_memory = true;
                        off += tuple_bytes;
                    }
                }
            }
            PROP => {
                if cursor + 8 > struct_end || depth == 0 { return Err(FdtError::Structure); }
                let len = be32(blob, cursor)? as usize;
                let nameoff = be32(blob, cursor + 4)?;
                cursor += 8;
                let end = cursor.checked_add(len).filter(|v| *v <= struct_end).ok_or(FdtError::Bounds)?;
                match prop_name(strings, nameoff)? {
                    b"reg" => node_reg[depth - 1] = Some((cursor, len)),
                    b"device_type" if name_eq(&blob[cursor..end], b"memory") => node_device_memory[depth - 1] = true,
                    _ => {}
                }
                cursor = align4(end)?;
            }
            NOP => {}
            END => break,
            _ => return Err(FdtError::Structure),
        }
        if cursor > struct_end { return Err(FdtError::Bounds); }
    }
    if !found_memory { return Err(FdtError::Memory); }

    cursor = off_reserve;
    loop {
        if cursor.checked_add(16).filter(|v| *v <= total).is_none() { return Err(FdtError::Reservation); }
        let address = ((be32(blob, cursor)? as u64) << 32) | be32(blob, cursor + 4)? as u64;
        let size = ((be32(blob, cursor + 8)? as u64) << 32) | be32(blob, cursor + 12)? as u64;
        cursor += 16;
        if address == 0 && size == 0 { break; }
        let start = usize::try_from(address).map_err(|_| FdtError::Bounds)?;
        let end = usize::try_from(address.checked_add(size).ok_or(FdtError::Bounds)?).map_err(|_| FdtError::Bounds)?;
        map.reserve(start, end).map_err(|_| FdtError::Reservation)?;
    }
    Ok(total)
}
