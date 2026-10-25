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
    let e = p.checked_add(4).ok_or(FdtError::Bounds)?;
    let s = b.get(p..e).ok_or(FdtError::Bounds)?;
    Ok(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}
fn cells(b: &[u8], p: usize, n: u32) -> Result<u64, FdtError> {
    let mut v = 0u64;
    for i in 0..n { v = (v << 32) | be32(b, p + i as usize * 4)? as u64; }
    Ok(v)
}
fn align4(n: usize) -> Result<usize, FdtError> {
    n.checked_add(3).map(|v| v & !3).ok_or(FdtError::Bounds)
}
fn prop_name(strings: &[u8], offset: u32) -> Result<&[u8], FdtError> {
    let tail = strings.get(offset as usize..).ok_or(FdtError::Bounds)?;
    let len = tail.iter().position(|b| *b == 0).ok_or(FdtError::Structure)?;
    Ok(&tail[..len])
}
fn is_memory(name: &[u8]) -> bool {
    let name = name.split(|b| *b == b'@').next().unwrap_or(name);
    name == b"memory"
}
fn is_named(bytes: &[u8], name: &[u8]) -> bool { bytes.strip_suffix(&[0]).unwrap_or(bytes) == name }

pub fn declared_size(header: &[u8], maximum: usize) -> Result<usize, FdtError> {
    if header.len() < 40 || be32(header, 0)? != MAGIC { return Err(FdtError::Header); }
    let total = be32(header, 4)? as usize;
    if total < 40 || total > maximum { return Err(FdtError::Bounds); }
    Ok(total)
}

pub fn parse_memory(blob: &[u8], map: &mut MemoryMap) -> Result<usize, FdtError> {
    if blob.len() < 40 || be32(blob, 0)? != MAGIC { return Err(FdtError::Header); }
    let total = be32(blob, 4)? as usize;
    let off_struct = be32(blob, 8)? as usize;
    let off_strings = be32(blob, 12)? as usize;
    let off_reserve = be32(blob, 16)? as usize;
    let size_strings = be32(blob, 32)? as usize;
    let size_struct = be32(blob, 36)? as usize;
    if total < 40 || total > blob.len()
        || off_struct.checked_add(size_struct).filter(|x| *x <= total).is_none()
        || off_strings.checked_add(size_strings).filter(|x| *x <= total).is_none()
        || off_reserve < 40 || off_reserve >= total { return Err(FdtError::Bounds); }
    let strings = &blob[off_strings..off_strings + size_strings];
    let struct_end = off_struct + size_struct;

    let (mut address_cells, mut size_cells) = (2u32, 1u32);
    let (mut p, mut depth, mut root) = (off_struct, 0usize, false);
    while p + 4 <= struct_end {
        let token = be32(blob, p)?; p += 4;
        match token {
            BEGIN_NODE => {
                let end = blob.get(p..struct_end).ok_or(FdtError::Bounds)?
                    .iter().position(|b| *b == 0).map(|n| p + n).ok_or(FdtError::Structure)?;
                if depth == 0 { root = true; }
                p = align4(end + 1)?; depth += 1;
            }
            END_NODE => { if depth == 0 { return Err(FdtError::Structure); } depth -= 1; }
            PROP => {
                if p + 8 > struct_end || depth == 0 { return Err(FdtError::Structure); }
                let len = be32(blob, p)? as usize;
                let nameoff = be32(blob, p + 4)?;
                p += 8;
                let end = p.checked_add(len).filter(|x| *x <= struct_end).ok_or(FdtError::Bounds)?;
                let name = prop_name(strings, nameoff)?;
                if depth == 1 && name == b"#address-cells" {
                    if len != 4 { return Err(FdtError::Cells); }
                    address_cells = be32(blob, p)?;
                } else if depth == 1 && name == b"#size-cells" {
                    if len != 4 { return Err(FdtError::Cells); }
                    size_cells = be32(blob, p)?;
                }
                p = align4(end)?;
            }
            NOP => {}
            END => break,
            _ => return Err(FdtError::Structure),
        }
        if p > struct_end { return Err(FdtError::Bounds); }
    }
    if !root || !(1..=2).contains(&address_cells) || !(1..=2).contains(&size_cells) {
        return Err(FdtError::Cells);
    }

    let mut names: [Option<(usize, usize)>; 32] = [None; 32];
    let mut memory = [false; 32];
    let mut device_memory = [false; 32];
    let mut reserved_container = [false; 32];
    let mut reserved_child = [false; 32];
    let mut regs: [Option<(usize, usize)>; 32] = [None; 32];
    p = off_struct; depth = 0;
    let mut found_memory = false;
    let mut reserved_starts = [0usize; 32];
    let mut reserved_ends = [0usize; 32];
    let mut reserved_count = 0usize;
    while p + 4 <= struct_end {
        let token = be32(blob, p)?; p += 4;
        match token {
            BEGIN_NODE => {
                if depth >= 32 { return Err(FdtError::Structure); }
                let end = blob.get(p..struct_end).ok_or(FdtError::Bounds)?
                    .iter().position(|b| *b == 0).map(|n| p + n).ok_or(FdtError::Structure)?;
                names[depth] = Some((p, end));
                let name = &blob[p..end];
                memory[depth] = is_memory(name);
                reserved_container[depth] = is_named(name, b"reserved-memory");
                reserved_child[depth] = depth > 0 && reserved_container[depth - 1];
                device_memory[depth] = false;
                regs[depth] = None;
                p = align4(end + 1)?; depth += 1;
            }
            END_NODE => {
                if depth == 0 { return Err(FdtError::Structure); }
                depth -= 1;
                if reserved_child[depth] {
                    if let Some((rp, len)) = regs[depth] {
                        let tuple = (address_cells + size_cells) as usize * 4;
                        if tuple == 0 || len == 0 || len % tuple != 0 { return Err(FdtError::Cells); }
                        let mut off = 0;
                        while off < len {
                            let base = cells(blob, rp + off, address_cells)?;
                            let size = cells(blob, rp + off + address_cells as usize * 4, size_cells)?;
                            let end = base.checked_add(size).ok_or(FdtError::Bounds)?;
                            if size == 0 { return Err(FdtError::Reservation); }
                            if reserved_count >= reserved_starts.len() { return Err(FdtError::Reservation); }
                            reserved_starts[reserved_count] = usize::try_from(base).map_err(|_| FdtError::Bounds)?;
                            reserved_ends[reserved_count] = usize::try_from(end).map_err(|_| FdtError::Bounds)?;
                            reserved_count += 1;
                            off += tuple;
                        }
                    }
                }
                if memory[depth] || device_memory[depth] {
                    let (rp, len) = regs[depth].ok_or(FdtError::Memory)?;
                    let tuple = (address_cells + size_cells) as usize * 4;
                    if tuple == 0 || len == 0 || len % tuple != 0 { return Err(FdtError::Cells); }
                    let mut off = 0;
                    while off < len {
                        let base = cells(blob, rp + off, address_cells)?;
                        let size = cells(blob, rp + off + address_cells as usize * 4, size_cells)?;
                        let end = base.checked_add(size).ok_or(FdtError::Bounds)?;
                        if size == 0 { return Err(FdtError::Memory); }
                        map.add(usize::try_from(base).map_err(|_| FdtError::Bounds)?,
                                usize::try_from(end).map_err(|_| FdtError::Bounds)?)
                            .map_err(|_| FdtError::Memory)?;
                        found_memory = true; off += tuple;
                    }
                }
            }
            PROP => {
                if p + 8 > struct_end || depth == 0 { return Err(FdtError::Structure); }
                let len = be32(blob, p)? as usize;
                let nameoff = be32(blob, p + 4)?;
                p += 8;
                let end = p.checked_add(len).filter(|x| *x <= struct_end).ok_or(FdtError::Bounds)?;
                match prop_name(strings, nameoff)? {
                    b"reg" => regs[depth - 1] = Some((p, len)),
                    b"device_type" if is_named(&blob[p..end], b"memory") => device_memory[depth - 1] = true,
                    _ => {}
                }
                p = align4(end)?;
            }
            NOP => {}
            END => break,
            _ => return Err(FdtError::Structure),
        }
        if p > struct_end { return Err(FdtError::Bounds); }
    }
    if !found_memory { return Err(FdtError::Memory); }
    // Apply reserved-memory ranges only after all RAM ranges are known; FDT node
    // order must not change whether a reservation is excluded from allocatable RAM.
    for i in 0..reserved_count {
        map.reserve(reserved_starts[i], reserved_ends[i]).map_err(|_| FdtError::Reservation)?;
    }

    p = off_reserve;
    loop {
        if p.checked_add(16).filter(|x| *x <= total).is_none() { return Err(FdtError::Reservation); }
        let address = ((be32(blob, p)? as u64) << 32) | be32(blob, p + 4)? as u64;
        let size = ((be32(blob, p + 8)? as u64) << 32) | be32(blob, p + 12)? as u64;
        p += 16;
        if address == 0 && size == 0 { break; }
        let end = address.checked_add(size).ok_or(FdtError::Bounds)?;
        map.reserve(usize::try_from(address).map_err(|_| FdtError::Bounds)?,
                    usize::try_from(end).map_err(|_| FdtError::Bounds)?)
            .map_err(|_| FdtError::Reservation)?;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_truncated_and_bad_magic_blobs() {
        let mut map = MemoryMap::new();
        assert_eq!(parse_memory(&[0; 20], &mut map), Err(FdtError::Header));
        let mut b = [0u8; 40]; b[..4].copy_from_slice(&MAGIC.to_be_bytes());
        assert_eq!(parse_memory(&b, &mut map), Err(FdtError::Bounds));
    }
}
