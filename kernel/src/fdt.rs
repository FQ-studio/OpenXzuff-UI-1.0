use crate::memory::MemoryMap;

const MAGIC: u32 = 0xd00d_feed;
const BEGIN_NODE: u32 = 1;
const END_NODE: u32 = 2;
const PROP: u32 = 3;
const NOP: u32 = 4;
const END: u32 = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FdtError { Header, Bounds, Structure, Cells, Memory, Reservation, Timebase, Translation }

fn be32(b: &[u8], p: usize) -> Result<u32, FdtError> {
    let s = b.get(p..p.checked_add(4).ok_or(FdtError::Bounds)?).ok_or(FdtError::Bounds)?;
    Ok(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

// FDT permits one or two 32-bit cells for a 64-bit address/size.
// Do not shift an already-populated u64 by another 32 bits: that discards
// the first cell instead of concatenating the pair.
fn read_cells(b: &[u8], p: usize, cells: u32) -> Result<u64, FdtError> {
    match cells {
        1 => Ok(be32(b, p)? as u64),
        2 => {
            let low_offset = p.checked_add(4).ok_or(FdtError::Bounds)?;
            Ok(((be32(b, p)? as u64) << 32) | be32(b, low_offset)? as u64)
        }
        _ => Err(FdtError::Cells),
    }
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


/// Read the architectural timebase frequency from the root /cpus node.
/// SBI TIME deadlines are expressed in these ticks, not in CPU cycles.
pub fn timebase_frequency(blob: &[u8]) -> Result<usize, FdtError> {
    if blob.len() < 40 || be32(blob, 0)? != MAGIC { return Err(FdtError::Header); }
    let total = be32(blob, 4)? as usize;
    let off_struct = be32(blob, 8)? as usize;
    let off_strings = be32(blob, 12)? as usize;
    let size_strings = be32(blob, 32)? as usize;
    let size_struct = be32(blob, 36)? as usize;
    if total < 40 || total > blob.len()
        || off_struct.checked_add(size_struct).filter(|v| *v <= total).is_none()
        || off_strings.checked_add(size_strings).filter(|v| *v <= total).is_none() {
        return Err(FdtError::Bounds);
    }
    let strings = &blob[off_strings..off_strings + size_strings];
    let struct_end = off_struct + size_struct;
    let mut cursor = off_struct;
    let mut depth = 0usize;
    let mut cpus_node = [false; 32];
    let mut root_seen = false;
    let mut ended = false;
    let mut frequency = None;

    while cursor.checked_add(4).filter(|v| *v <= struct_end).is_some() {
        let token = be32(blob, cursor)?;
        cursor += 4;
        match token {
            BEGIN_NODE => {
                if depth >= cpus_node.len() { return Err(FdtError::Structure); }
                if depth == 0 {
                    if root_seen { return Err(FdtError::Structure); }
                    root_seen = true;
                }
                let end = blob.get(cursor..struct_end).ok_or(FdtError::Bounds)?
                    .iter().position(|b| *b == 0).map(|n| cursor + n).ok_or(FdtError::Structure)?;
                let name = blob.get(cursor..end).ok_or(FdtError::Bounds)?;
                cpus_node[depth] = depth == 1
                    && name.strip_suffix(&[0]).unwrap_or(name)
                        .split(|b| *b == b'@').next() == Some(&b"cpus"[..]);
                cursor = align4(end.checked_add(1).ok_or(FdtError::Bounds)?)?;
                depth += 1;
            }
            END_NODE => {
                if depth == 0 { return Err(FdtError::Structure); }
                depth -= 1;
            }
            PROP => {
                if cursor.checked_add(8).filter(|v| *v <= struct_end).is_none() || depth == 0 {
                    return Err(FdtError::Structure);
                }
                let len = be32(blob, cursor)? as usize;
                let nameoff = be32(blob, cursor + 4)?;
                cursor += 8;
                let end = cursor.checked_add(len).filter(|v| *v <= struct_end).ok_or(FdtError::Bounds)?;
                let name = prop_name(strings, nameoff)?;
                if depth == 2 && cpus_node[1] && name == b"timebase-frequency" {
                    if frequency.is_some() { return Err(FdtError::Timebase); }
                    let value = match len {
                        4 => be32(blob, cursor)? as u64,
                        8 => read_cells(blob, cursor, 2)?,
                        _ => return Err(FdtError::Timebase),
                    };
                    if value == 0 { return Err(FdtError::Timebase); }
                    frequency = Some(value);
                }
                cursor = align4(end)?;
            }
            NOP => {}
            END => {
                if !root_seen || depth != 0 { return Err(FdtError::Structure); }
                ended = true;
                break;
            }
            _ => return Err(FdtError::Structure),
        }
        if cursor > struct_end { return Err(FdtError::Bounds); }
    }
    if !ended { return Err(FdtError::Structure); }
    let frequency = frequency.ok_or(FdtError::Timebase)?;
    usize::try_from(frequency).map_err(|_| FdtError::Timebase)
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

    // Pass one: root cell widths define each memory reg tuple. Validate the
    // complete structure block before trusting any RAM description.
    let (mut address_cells, mut size_cells) = (2u32, 1u32);
    let (mut cursor, mut depth, mut root_seen, mut ended) = (off_struct, 0usize, false, false);
    while cursor.checked_add(4).filter(|v| *v <= struct_end).is_some() {
        let token = be32(blob, cursor)?; cursor += 4;
        match token {
            BEGIN_NODE => {
                if depth >= 32 { return Err(FdtError::Structure); }
                if depth == 0 {
                    if root_seen { return Err(FdtError::Structure); }
                    root_seen = true;
                }
                let end = blob.get(cursor..struct_end).ok_or(FdtError::Bounds)?
                    .iter().position(|b| *b == 0).map(|n| cursor + n).ok_or(FdtError::Structure)?;
                cursor = align4(end.checked_add(1).ok_or(FdtError::Bounds)?)?;
                depth += 1;
            }
            END_NODE => {
                if depth == 0 { return Err(FdtError::Structure); }
                depth -= 1;
            }
            PROP => {
                if cursor.checked_add(8).filter(|v| *v <= struct_end).is_none() || depth == 0 {
                    return Err(FdtError::Structure);
                }
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
            END => {
                if !root_seen || depth != 0 { return Err(FdtError::Structure); }
                ended = true;
                break;
            }
            _ => return Err(FdtError::Structure),
        }
        if cursor > struct_end { return Err(FdtError::Bounds); }
    }
    if !ended { return Err(FdtError::Structure); }
    if !(1..=2).contains(&address_cells) || !(1..=2).contains(&size_cells) {
        return Err(FdtError::Cells);
    }

    // Pass two: hold node properties until END_NODE so reg tuples are complete.
    let mut node_memory = [false; 32];
    let mut node_device_memory = [false; 32];
    // A missing status means enabled; only "okay" and "ok" nodes contribute RAM.
    let mut node_enabled = [true; 32];
    let mut node_reg: [Option<(usize, usize)>; 32] = [None; 32];
    cursor = off_struct;
    depth = 0;
    let mut root_seen = false;
    let mut ended = false;
    let mut found_memory = false;
    while cursor.checked_add(4).filter(|v| *v <= struct_end).is_some() {
        let token = be32(blob, cursor)?; cursor += 4;
        match token {
            BEGIN_NODE => {
                if depth >= 32 { return Err(FdtError::Structure); }
                if depth == 0 {
                    if root_seen { return Err(FdtError::Structure); }
                    root_seen = true;
                }
                let end = blob.get(cursor..struct_end).ok_or(FdtError::Bounds)?
                    .iter().position(|b| *b == 0).map(|n| cursor + n).ok_or(FdtError::Structure)?;
                node_memory[depth] = node_is_memory(&blob[cursor..end]);
                node_device_memory[depth] = false;
                node_enabled[depth] = true;
                node_reg[depth] = None;
                cursor = align4(end.checked_add(1).ok_or(FdtError::Bounds)?)?;
                depth += 1;
            }
            END_NODE => {
                if depth == 0 { return Err(FdtError::Structure); }
                depth -= 1;
                if node_enabled[depth] && (node_memory[depth] || node_device_memory[depth]) {
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
                if cursor.checked_add(8).filter(|v| *v <= struct_end).is_none() || depth == 0 {
                    return Err(FdtError::Structure);
                }
                let len = be32(blob, cursor)? as usize;
                let nameoff = be32(blob, cursor + 4)?;
                cursor += 8;
                let end = cursor.checked_add(len).filter(|v| *v <= struct_end).ok_or(FdtError::Bounds)?;
                match prop_name(strings, nameoff)? {
                    b"reg" => node_reg[depth - 1] = Some((cursor, len)),
                    b"device_type" if name_eq(&blob[cursor..end], b"memory") => node_device_memory[depth - 1] = true,
                    b"status" => node_enabled[depth - 1] = status_is_enabled(&blob[cursor..end])?,
                    _ => {}
                }
                cursor = align4(end)?;
            }
            NOP => {}
            END => {
                if !root_seen || depth != 0 { return Err(FdtError::Structure); }
                ended = true;
                break;
            }
            _ => return Err(FdtError::Structure),
        }
        if cursor > struct_end { return Err(FdtError::Bounds); }
    }
    if !ended { return Err(FdtError::Structure); }
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

/// Return whether an FDT status property marks a node as available.
/// Device-tree string properties must be NUL terminated and contain one value.
fn status_is_enabled(value: &[u8]) -> Result<bool, FdtError> {
    if value.is_empty() || value.last() != Some(&0) || value[..value.len() - 1].contains(&0) {
        return Err(FdtError::Structure);
    }
    let status = &value[..value.len() - 1];
    Ok(status == b"okay" || status == b"ok")
}

/// Match one exact entry in a NUL-delimited FDT compatible string-list.
fn compatible_list_contains(value: &[u8], wanted: &[u8]) -> Result<bool, FdtError> {
    if value.is_empty() || value.last() != Some(&0) {
        return Err(FdtError::Structure);
    }
    let mut start = 0usize;
    let mut found = false;
    while start < value.len() {
        let finish = value[start..].iter().position(|b| *b == 0)
            .map(|n| start + n).ok_or(FdtError::Structure)?;
        if finish == start { return Err(FdtError::Structure); }
        found |= &value[start..finish] == wanted;
        start = finish + 1;
    }
    Ok(found)
}

/// Match an exact compatible entry on an available node.
/// A disabled ancestor also makes its child devices unavailable.
/// This discovers a driver candidate; it does not authorize MMIO access.
pub fn has_compatible(blob: &[u8], wanted: &[u8]) -> Result<bool, FdtError> {
    if wanted.is_empty() || wanted.contains(&0) { return Err(FdtError::Structure); }
    if blob.len() < 40 || be32(blob, 0)? != MAGIC { return Err(FdtError::Header); }
    let total = be32(blob, 4)? as usize;
    let off_struct = be32(blob, 8)? as usize;
    let off_strings = be32(blob, 12)? as usize;
    let size_strings = be32(blob, 32)? as usize;
    let size_struct = be32(blob, 36)? as usize;
    if total < 40 || total > blob.len()
        || off_struct.checked_add(size_struct).filter(|v| *v <= total).is_none()
        || off_strings.checked_add(size_strings).filter(|v| *v <= total).is_none() {
        return Err(FdtError::Bounds);
    }

    let strings = &blob[off_strings..off_strings + size_strings];
    let struct_end = off_struct + size_struct;
    let mut cursor = off_struct;
    let mut depth = 0usize;
    let mut root_seen = false;
    let mut ended = false;
    let mut found = false;
    let mut node_enabled = [true; 32];
    let mut node_compatible = [false; 32];

    while cursor.checked_add(4).filter(|v| *v <= struct_end).is_some() {
        let token = be32(blob, cursor)?;
        cursor += 4;
        match token {
            BEGIN_NODE => {
                if depth >= node_enabled.len() { return Err(FdtError::Structure); }
                let end = blob.get(cursor..struct_end).ok_or(FdtError::Bounds)?
                    .iter().position(|b| *b == 0).map(|n| cursor + n)
                    .ok_or(FdtError::Structure)?;
                if depth == 0 {
                    if root_seen { return Err(FdtError::Structure); }
                    root_seen = true;
                    node_enabled[depth] = true;
                } else {
                    node_enabled[depth] = node_enabled[depth - 1];
                }
                node_compatible[depth] = false;
                cursor = align4(end.checked_add(1).ok_or(FdtError::Bounds)?)?;
                depth += 1;
            }
            END_NODE => {
                if depth == 0 { return Err(FdtError::Structure); }
                depth -= 1;
                if node_enabled[depth] && node_compatible[depth] {
                    found = true;
                }
            }
            PROP => {
                if depth == 0 || cursor.checked_add(8).filter(|v| *v <= struct_end).is_none() {
                    return Err(FdtError::Structure);
                }
                let len = be32(blob, cursor)? as usize;
                let nameoff = be32(blob, cursor + 4)?;
                cursor += 8;
                let end = cursor.checked_add(len).filter(|v| *v <= struct_end)
                    .ok_or(FdtError::Bounds)?;
                match prop_name(strings, nameoff)? {
                    b"compatible" => {
                        if compatible_list_contains(&blob[cursor..end], wanted)? {
                            node_compatible[depth - 1] = true;
                        }
                    }
                    b"status" => {
                        node_enabled[depth - 1] &= status_is_enabled(&blob[cursor..end])?;
                    }
                    _ => {}
                }
                cursor = align4(end)?;
            }
            NOP => {}
            END => {
                if !root_seen || depth != 0 { return Err(FdtError::Structure); }
                ended = true;
                break;
            }
            _ => return Err(FdtError::Structure),
        }
        if cursor > struct_end { return Err(FdtError::Bounds); }
    }

    if !ended { return Err(FdtError::Structure); }
    Ok(found)
}


/// A raw device-tree register tuple. The address is still in the parent bus
/// address space; it must not be used for MMIO until every ancestor `ranges`
/// mapping and platform resource constraint has been resolved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FdtRegion {
    pub bus_address: u64,
    pub size: Option<u64>,
}


fn address_fits_cells(address: u64, cells: u32) -> bool {
    match cells {
        1 => address <= u32::MAX as u64,
        2 => true,
        _ => false,
    }
}

/// Translate a resource from its immediate parent bus to the root address
/// space. Each bus contributes one child-to-parent mapping via `ranges`.
fn translate_region_to_root(
    blob: &[u8],
    mut region: FdtRegion,
    node_index: usize,
    ranges: &[Option<(usize, usize)>; 32],
    ranges_seen: &[bool; 32],
    child_address_cells: &[u32; 32],
    child_size_cells: &[u32; 32],
    parent_address_cells: &[u32; 32],
) -> Result<FdtRegion, FdtError> {
    let access_size = region.size.unwrap_or(1);
    if access_size == 0 { return Err(FdtError::Translation); }
    let mut bus_index = node_index.checked_sub(1).ok_or(FdtError::Structure)?;

    while bus_index > 0 {
        if !ranges_seen[bus_index] { return Err(FdtError::Translation); }
        let (ranges_start, ranges_len) = ranges[bus_index].ok_or(FdtError::Structure)?;
        let child_cells = child_address_cells[bus_index];
        let parent_cells = parent_address_cells[bus_index];
        if !(1..=2).contains(&child_cells) || !(1..=2).contains(&parent_cells) {
            return Err(FdtError::Cells);
        }

        let access_end = region.bus_address
            .checked_add(access_size)
            .ok_or(FdtError::Translation)?;
        if !address_fits_cells(region.bus_address, child_cells)
            || !address_fits_cells(access_end - 1, child_cells) {
            return Err(FdtError::Translation);
        }

        // An explicitly empty ranges property declares identity translation.
        if ranges_len == 0 {
            if !address_fits_cells(region.bus_address, parent_cells)
                || !address_fits_cells(access_end - 1, parent_cells) {
                return Err(FdtError::Translation);
            }
        } else {
            let size_cells = child_size_cells[bus_index];
            if !(1..=2).contains(&size_cells) { return Err(FdtError::Cells); }
            let tuple_cells = child_cells.checked_add(parent_cells)
                .and_then(|n| n.checked_add(size_cells))
                .ok_or(FdtError::Cells)?;
            let tuple_bytes = (tuple_cells as usize).checked_mul(4)
                .ok_or(FdtError::Bounds)?;
            if ranges_len == 0 || ranges_len % tuple_bytes != 0 {
                return Err(FdtError::Cells);
            }

            let mut offset = 0usize;
            let mut translated = None;
            while offset < ranges_len {
                let child_base = read_cells(blob, ranges_start + offset, child_cells)?;
                let parent_offset = offset + child_cells as usize * 4;
                let parent_base = read_cells(blob, ranges_start + parent_offset, parent_cells)?;
                let size_offset = parent_offset + parent_cells as usize * 4;
                let window_size = read_cells(blob, ranges_start + size_offset, size_cells)?;
                if window_size == 0 { return Err(FdtError::Translation); }
                let child_end = child_base.checked_add(window_size)
                    .ok_or(FdtError::Translation)?;
                let parent_end = parent_base.checked_add(window_size)
                    .ok_or(FdtError::Translation)?;
                if !address_fits_cells(child_base, child_cells)
                    || !address_fits_cells(child_end - 1, child_cells)
                    || !address_fits_cells(parent_base, parent_cells)
                    || !address_fits_cells(parent_end - 1, parent_cells) {
                    return Err(FdtError::Translation);
                }

                if region.bus_address >= child_base && access_end <= child_end {
                    if translated.is_some() { return Err(FdtError::Translation); }
                    let delta = region.bus_address - child_base;
                    let root_address = parent_base.checked_add(delta)
                        .ok_or(FdtError::Translation)?;
                    let root_end = root_address.checked_add(access_size)
                        .ok_or(FdtError::Translation)?;
                    if root_end > parent_end
                        || !address_fits_cells(root_address, parent_cells)
                        || !address_fits_cells(root_end - 1, parent_cells) {
                        return Err(FdtError::Translation);
                    }
                    translated = Some(root_address);
                }
                offset += tuple_bytes;
            }
            region.bus_address = translated.ok_or(FdtError::Translation)?;
        }

        bus_index -= 1;
    }

    Ok(region)
}

/// A compatible node selected from the FDT. `reg` is the first tuple in
/// the node's parent-bus address space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FdtNode<'a> {
    pub name: &'a [u8],
    pub reg: Option<FdtRegion>,
}

/// A compatible node with both its raw parent-bus resource and its address
/// translated through every ancestor bus `ranges` property to the root bus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FdtMappedNode<'a> {
    pub name: &'a [u8],
    pub bus_reg: Option<FdtRegion>,
    pub root_reg: Option<FdtRegion>,
}

/// Find the first available compatible node and return its untranslated reg.
pub fn find_compatible_node<'a>(
    blob: &'a [u8],
    wanted: &[u8],
) -> Result<Option<FdtNode<'a>>, FdtError> {
    let mapped = find_compatible_node_inner(blob, wanted, false)?;
    Ok(mapped.map(|node| FdtNode { name: node.name, reg: node.bus_reg }))
}

/// Find the first available compatible node and translate its first reg tuple
/// through each ancestor bus to the root address space. A missing `ranges`
/// property on an intermediate bus is not treated as identity; only an
/// explicitly empty `ranges` property means identity translation.
pub fn find_compatible_node_translated<'a>(
    blob: &'a [u8],
    wanted: &[u8],
) -> Result<Option<FdtMappedNode<'a>>, FdtError> {
    find_compatible_node_inner(blob, wanted, true)
}

fn find_compatible_node_inner<'a>(
    blob: &'a [u8],
    wanted: &[u8],
    translate: bool,
) -> Result<Option<FdtMappedNode<'a>>, FdtError> {
    if wanted.is_empty() || wanted.contains(&0) { return Err(FdtError::Structure); }
    if blob.len() < 40 || be32(blob, 0)? != MAGIC { return Err(FdtError::Header); }
    let total = be32(blob, 4)? as usize;
    let off_struct = be32(blob, 8)? as usize;
    let off_strings = be32(blob, 12)? as usize;
    let size_strings = be32(blob, 32)? as usize;
    let size_struct = be32(blob, 36)? as usize;
    if total < 40 || total > blob.len()
        || off_struct.checked_add(size_struct).filter(|v| *v <= total).is_none()
        || off_strings.checked_add(size_strings).filter(|v| *v <= total).is_none() {
        return Err(FdtError::Bounds);
    }

    let strings = &blob[off_strings..off_strings + size_strings];
    let struct_end = off_struct + size_struct;
    let mut cursor = off_struct;
    let mut depth = 0usize;
    let mut root_seen = false;
    let mut ended = false;

    let mut enabled = [true; 32];
    let mut compatible = [false; 32];
    let mut saw_child = [false; 32];
    let mut names: [Option<(usize, usize)>; 32] = [None; 32];
    let mut regs: [Option<(usize, usize)>; 32] = [None; 32];
    let mut ranges: [Option<(usize, usize)>; 32] = [None; 32];
    let mut ranges_seen = [false; 32];
    let mut child_address_cells = [2u32; 32];
    let mut child_size_cells = [1u32; 32];
    let mut reg_address_cells = [2u32; 32];
    let mut reg_size_cells = [1u32; 32];
    let mut result: Option<FdtMappedNode<'a>> = None;

    while cursor.checked_add(4).filter(|v| *v <= struct_end).is_some() {
        let token = be32(blob, cursor)?;
        cursor += 4;
        match token {
            BEGIN_NODE => {
                if depth >= enabled.len() { return Err(FdtError::Structure); }
                if depth > 0 {
                    saw_child[depth - 1] = true;
                    enabled[depth] = enabled[depth - 1];
                    reg_address_cells[depth] = child_address_cells[depth - 1];
                    reg_size_cells[depth] = child_size_cells[depth - 1];
                } else {
                    if root_seen { return Err(FdtError::Structure); }
                    root_seen = true;
                    enabled[depth] = true;
                    reg_address_cells[depth] = 2;
                    reg_size_cells[depth] = 1;
                }
                let start = cursor;
                let end = blob.get(cursor..struct_end).ok_or(FdtError::Bounds)?
                    .iter().position(|b| *b == 0).map(|n| cursor + n)
                    .ok_or(FdtError::Structure)?;
                names[depth] = Some((start, end - start));
                compatible[depth] = false;
                saw_child[depth] = false;
                regs[depth] = None;
                ranges[depth] = None;
                ranges_seen[depth] = false;
                child_address_cells[depth] = 2;
                child_size_cells[depth] = 1;
                cursor = align4(end.checked_add(1).ok_or(FdtError::Bounds)?)?;
                depth += 1;
            }
            END_NODE => {
                if depth == 0 { return Err(FdtError::Structure); }
                depth -= 1;
                if enabled[depth] && compatible[depth] {
                    let name_range = names[depth].ok_or(FdtError::Structure)?;
                    let reg = if let Some((reg_start, reg_len)) = regs[depth] {
                        let address_cells = reg_address_cells[depth];
                        let size_cells = reg_size_cells[depth];
                        if !(1..=2).contains(&address_cells) || size_cells > 2 {
                            return Err(FdtError::Cells);
                        }
                        let tuple_cells = address_cells.checked_add(size_cells)
                            .ok_or(FdtError::Cells)?;
                        let tuple_bytes = tuple_cells as usize * 4;
                        if reg_len == 0 || reg_len % tuple_bytes != 0 {
                            return Err(FdtError::Cells);
                        }
                        let mut offset = 0usize;
                        let mut first = None;
                        while offset < reg_len {
                            let address = read_cells(blob, reg_start + offset, address_cells)?;
                            let size = if size_cells == 0 {
                                None
                            } else {
                                let size_offset = reg_start + offset + address_cells as usize * 4;
                                let value = read_cells(blob, size_offset, size_cells)?;
                                if value == 0 { return Err(FdtError::Memory); }
                                address.checked_add(value).ok_or(FdtError::Bounds)?;
                                Some(value)
                            };
                            if first.is_none() {
                                first = Some(FdtRegion { bus_address: address, size });
                            }
                            offset += tuple_bytes;
                        }
                        first
                    } else {
                        None
                    };
                    if result.is_none() {
                        let root_reg = if translate {
                            match reg {
                                Some(region) => Some(translate_region_to_root(
                                    blob,
                                    region,
                                    depth,
                                    &ranges,
                                    &ranges_seen,
                                    &child_address_cells,
                                    &child_size_cells,
                                    &reg_address_cells,
                                )?),
                                None => None,
                            }
                        } else {
                            None
                        };
                        result = Some(FdtMappedNode {
                            name: &blob[name_range.0..name_range.0 + name_range.1],
                            bus_reg: reg,
                            root_reg,
                        });
                    }
                }
            }
            PROP => {
                if depth == 0 || cursor.checked_add(8).filter(|v| *v <= struct_end).is_none() {
                    return Err(FdtError::Structure);
                }
                let index = depth - 1;
                // Device-tree node properties precede subnodes. Enforce this so
                // parent bus cell widths are known before parsing child reg data.
                if saw_child[index] { return Err(FdtError::Structure); }
                let len = be32(blob, cursor)? as usize;
                let nameoff = be32(blob, cursor + 4)?;
                cursor += 8;
                let end = cursor.checked_add(len).filter(|v| *v <= struct_end)
                    .ok_or(FdtError::Bounds)?;
                match prop_name(strings, nameoff)? {
                    b"#address-cells" => {
                        if len != 4 { return Err(FdtError::Cells); }
                        let cells = be32(blob, cursor)?;
                        // Unmatched buses may describe wider address tuples
                        // (for example PCIe's three-cell addresses). Preserve
                        // those widths; only a selected node's reg tuple must
                        // fit the current u64 resource representation.
                        if cells > 4 { return Err(FdtError::Cells); }
                        child_address_cells[index] = cells;
                    }
                    b"#size-cells" => {
                        if len != 4 { return Err(FdtError::Cells); }
                        let cells = be32(blob, cursor)?;
                        if cells > 4 { return Err(FdtError::Cells); }
                        child_size_cells[index] = cells;
                    }
                    b"compatible" => {
                        if compatible_list_contains(&blob[cursor..end], wanted)? {
                            compatible[index] = true;
                        }
                    }
                    b"status" => {
                        enabled[index] &= status_is_enabled(&blob[cursor..end])?;
                    }
                    b"reg" => regs[index] = Some((cursor, len)),
                    b"ranges" => {
                        if ranges_seen[index] { return Err(FdtError::Structure); }
                        ranges_seen[index] = true;
                        ranges[index] = Some((cursor, len));
                    }
                    _ => {}
                }
                cursor = align4(end)?;
            }
            NOP => {}
            END => {
                if !root_seen || depth != 0 { return Err(FdtError::Structure); }
                ended = true;
                break;
            }
            _ => return Err(FdtError::Structure),
        }
        if cursor > struct_end { return Err(FdtError::Bounds); }
    }

    if !ended { return Err(FdtError::Structure); }
    Ok(result)
}
