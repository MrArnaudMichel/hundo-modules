//! Just enough of Unreal Engine's save format (GVAS) to read one map
//! property of the save: the inventory.
//!
//! A property is a tag — its name, its type, its size — then its value. The
//! tag has two layouts: up to UE 5.3 the type is a name followed by the
//! inner types it needs (a map's key and value types); from UE 5.4 it is a
//! tree of names, each with the count of its parameters, then flags. Rather
//! than walk every property before it, the reader finds the property's name
//! in the file and reads the tag from there: the header and the properties
//! the module does not need may change without it noticing.

use hundo_sdk::DriverError;

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

fn corrupt(what: &str) -> DriverError {
    DriverError::Corrupt(format!("sauvegarde illisible : {what}"))
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], DriverError> {
        let end = self.at.checked_add(n).filter(|&e| e <= self.bytes.len()).ok_or_else(|| corrupt("fin inattendue"))?;
        let slice = &self.bytes[self.at..end];
        self.at = end;
        Ok(slice)
    }

    fn i32(&mut self) -> Result<i32, DriverError> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn u8(&mut self) -> Result<u8, DriverError> {
        Ok(self.take(1)?[0])
    }

    /// An FString: its length with the final NUL, then Latin-1 bytes — or,
    /// when the length is negative, UTF-16 code units.
    fn string(&mut self) -> Result<String, DriverError> {
        let len = self.i32()?;
        if len == 0 {
            return Ok(String::new());
        }
        if len > 0 {
            let raw = self.take(len as usize)?;
            let raw = raw.strip_suffix(&[0]).unwrap_or(raw);
            return Ok(raw.iter().map(|&b| b as char).collect());
        }
        let units = len.unsigned_abs() as usize;
        let raw = self.take(units.checked_mul(2).ok_or_else(|| corrupt("chaîne trop longue"))?)?;
        let mut text: Vec<u16> = raw.as_chunks::<2>().0.iter().map(|c| u16::from_le_bytes(*c)).collect();
        if text.last() == Some(&0) {
            text.pop();
        }
        Ok(String::from_utf16_lossy(&text))
    }

    /// A UE 5.4 type name: a name and its parameters, depth first.
    fn type_name(&mut self, depth: u8) -> Result<(String, Vec<String>), DriverError> {
        if depth > 8 {
            return Err(corrupt("type trop imbriqué"));
        }
        let name = self.string()?;
        let count = self.i32()?;
        let mut params = Vec::new();
        for _ in 0..count.clamp(0, 16) {
            params.push(self.type_name(depth + 1)?.0);
        }
        Ok((name, params))
    }
}

/// The bytes of `name` as an FString, as a property tag starts.
fn needle(name: &str) -> Vec<u8> {
    let mut n = ((name.len() + 1) as i32).to_le_bytes().to_vec();
    n.extend_from_slice(name.as_bytes());
    n.push(0);
    n
}

/// The entries of the map property `name` whose keys are names (or strings)
/// and values integers, in the save's order; `None` if the save has no such
/// property.
pub fn name_int_map(bytes: &[u8], name: &str) -> Result<Option<Vec<(String, i32)>>, DriverError> {
    if bytes.len() < 4 || &bytes[..4] != b"GVAS" {
        return Err(corrupt("ce n'est pas une sauvegarde Unreal (GVAS)"));
    }
    let needle = needle(name);
    let Some(start) = bytes.windows(needle.len()).position(|w| w == needle) else {
        return Ok(None);
    };
    let mut r = Reader { bytes, at: start + needle.len() };
    let ty = r.string()?;
    if ty != "MapProperty" {
        return Err(corrupt(&format!("{name} est un {ty}, pas une table")));
    }

    // Up to UE 5.3: size, array index (0), key type, value type, GUID flag.
    // From UE 5.4: parameter count (2), each parameter as a type name, then
    // size and flags. The array index tells them apart.
    let classic = bytes.get(r.at + 4..r.at + 8) == Some(&[0, 0, 0, 0]);
    let (key, value, size) = if classic {
        let size = r.i32()?;
        r.i32()?;
        let key = r.string()?;
        let value = r.string()?;
        if r.u8()? != 0 {
            r.take(16)?;
        }
        (key, value, size)
    } else {
        let count = r.i32()?;
        if count != 2 {
            return Err(corrupt("type de table inattendu"));
        }
        let key = r.type_name(1)?.0;
        let value = r.type_name(1)?.0;
        let size = r.i32()?;
        let flags = r.u8()?;
        if flags & 1 != 0 {
            r.i32()?;
        }
        if flags & 2 != 0 {
            r.take(16)?;
        }
        if flags & 4 != 0 {
            // Property extensions: a byte of flags, then, for override
            // operations, one more byte.
            if r.u8()? & 2 != 0 {
                r.u8()?;
            }
        }
        (key, value, size)
    };
    if !matches!(key.as_str(), "NameProperty" | "StrProperty") || value != "IntProperty" {
        return Err(corrupt(&format!("{name} associe des {key} à des {value}")));
    }
    let end = r.at.checked_add(size.max(0) as usize).filter(|&e| e <= bytes.len()).ok_or_else(|| corrupt("taille de table"))?;

    let removed = r.i32()?;
    for _ in 0..removed.max(0) {
        r.string()?;
    }
    let count = r.i32()?;
    let mut entries = Vec::with_capacity(count.clamp(0, 4096) as usize);
    for _ in 0..count.max(0) {
        let k = r.string()?;
        let v = r.i32()?;
        entries.push((k, v));
        if r.at > end {
            return Err(corrupt("table plus longue que sa taille"));
        }
    }
    Ok(Some(entries))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn fstring(out: &mut Vec<u8>, s: &str) {
        out.extend_from_slice(&needle(s));
    }

    fn body(entries: &[(&str, i32)]) -> Vec<u8> {
        let mut b = 0i32.to_le_bytes().to_vec();
        b.extend_from_slice(&(entries.len() as i32).to_le_bytes());
        for (k, v) in entries {
            fstring(&mut b, k);
            b.extend_from_slice(&v.to_le_bytes());
        }
        b
    }

    /// A save as UE 5.3 and earlier write it, with a property before the map.
    pub fn classic(entries: &[(&str, i32)]) -> Vec<u8> {
        let mut s = b"GVAS".to_vec();
        s.extend_from_slice(&[3, 0, 0, 0, 0x0a, 0x02, 0, 0]);
        fstring(&mut s, "SaveVersion_0");
        fstring(&mut s, "IntProperty");
        s.extend_from_slice(&4i32.to_le_bytes());
        s.extend_from_slice(&0i32.to_le_bytes());
        s.push(0);
        s.extend_from_slice(&7i32.to_le_bytes());
        let body = body(entries);
        fstring(&mut s, "InventoryItems_0");
        fstring(&mut s, "MapProperty");
        s.extend_from_slice(&(body.len() as i32).to_le_bytes());
        s.extend_from_slice(&0i32.to_le_bytes());
        fstring(&mut s, "NameProperty");
        fstring(&mut s, "IntProperty");
        s.push(0);
        s.extend_from_slice(&body);
        fstring(&mut s, "None");
        s
    }

    /// The same as UE 5.4 and later write it: complete type names, flags.
    pub fn complete(entries: &[(&str, i32)]) -> Vec<u8> {
        let mut s = b"GVAS".to_vec();
        s.extend_from_slice(&[3, 0, 0, 0]);
        let body = body(entries);
        fstring(&mut s, "InventoryItems_0");
        fstring(&mut s, "MapProperty");
        s.extend_from_slice(&2i32.to_le_bytes());
        fstring(&mut s, "NameProperty");
        s.extend_from_slice(&0i32.to_le_bytes());
        fstring(&mut s, "IntProperty");
        s.extend_from_slice(&0i32.to_le_bytes());
        s.extend_from_slice(&(body.len() as i32).to_le_bytes());
        s.push(0);
        s.extend_from_slice(&body);
        fstring(&mut s, "None");
        s
    }

    #[test]
    fn reads_both_tag_layouts() {
        let entries = [("Verleso", 1), ("Picto_Augmented_Attack", 3)];
        for save in [classic(&entries), complete(&entries)] {
            let got = name_int_map(&save, "InventoryItems_0").unwrap().unwrap();
            assert_eq!(got, vec![("Verleso".to_string(), 1), ("Picto_Augmented_Attack".to_string(), 3)]);
        }
    }

    #[test]
    fn utf16_keys() {
        let mut save = complete(&[]);
        let at = save.len() - needle("None").len() - 4;
        // Replace the empty map with one UTF-16 key.
        save.truncate(at);
        save.extend_from_slice(&1i32.to_le_bytes());
        let text: Vec<u16> = "Écharpe\0".encode_utf16().collect();
        save.extend_from_slice(&(-(text.len() as i32)).to_le_bytes());
        for u in text {
            save.extend_from_slice(&u.to_le_bytes());
        }
        save.extend_from_slice(&1i32.to_le_bytes());
        // The size before the flags byte still says 8: widen it.
        let size_at = at - 1 - 4 - 4;
        let size = (save.len() - (size_at + 4 + 1)) as i32;
        save[size_at..size_at + 4].copy_from_slice(&size.to_le_bytes());
        let got = name_int_map(&save, "InventoryItems_0").unwrap().unwrap();
        assert_eq!(got, vec![("Écharpe".to_string(), 1)]);
    }

    #[test]
    fn a_missing_property_is_none_and_garbage_is_corrupt() {
        assert!(name_int_map(&classic(&[]), "Nope_0").unwrap().is_none());
        assert!(matches!(name_int_map(b"PNG....", "InventoryItems_0"), Err(DriverError::Corrupt(_))));
        let mut cut = classic(&[("Verleso", 1)]);
        cut.truncate(cut.len() - 12);
        assert!(name_int_map(&cut, "InventoryItems_0").is_err());
    }
}
