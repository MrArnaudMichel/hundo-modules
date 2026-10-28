//! `ProfileN.sav`: an `SGB1` header, then the game's Lua state, serialized
//! with luabins and compressed as one LZ4 block.

use hundo_sdk::DriverError;

/// The game's own decompressor allocates this much; no real save comes near it.
const MAX_STATE: usize = 32 << 20;
const MAX_DEPTH: usize = 64;

pub struct Save {
    pub runs: u32,
    pub god_mode: bool,
    pub state: Lua,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lua {
    Nil,
    Bool(bool),
    Num(f64),
    Str(String),
    Table(Vec<(Lua, Lua)>),
}

impl Lua {
    pub fn get(&self, key: &str) -> &Lua {
        match self {
            Lua::Table(pairs) => pairs.iter().find(|(k, _)| matches!(k, Lua::Str(s) if s == key)).map_or(&Lua::Nil, |(_, v)| v),
            _ => &Lua::Nil,
        }
    }

    /// The value at an array index, which luabins stores as a number key.
    pub fn at(&self, index: u32) -> &Lua {
        match self {
            Lua::Table(pairs) => pairs.iter().find(|(k, _)| *k == Lua::Num(index as f64)).map_or(&Lua::Nil, |(_, v)| v),
            _ => &Lua::Nil,
        }
    }

    pub fn num(&self) -> f64 {
        match self {
            Lua::Num(n) => *n,
            _ => 0.0,
        }
    }

    pub fn truthy(&self) -> bool {
        !matches!(self, Lua::Nil | Lua::Bool(false))
    }

    pub fn str(&self) -> Option<&str> {
        match self {
            Lua::Str(s) => Some(s),
            _ => None,
        }
    }

    /// The pairs of a table whose keys are strings.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &Lua)> {
        let pairs = match self {
            Lua::Table(p) => p.as_slice(),
            _ => &[],
        };
        pairs.iter().filter_map(|(k, v)| Some((k.str()?, v)))
    }

    pub fn len(&self) -> usize {
        match self {
            Lua::Table(p) => p.len(),
            _ => 0,
        }
    }
}

pub fn decode(data: &[u8]) -> Result<Save, DriverError> {
    if data.len() < 8 {
        return Err(DriverError::Incomplete("fichier tronqué".into()));
    }
    if &data[..4] != b"SGB1" {
        return Err(DriverError::Corrupt("ce n'est pas une sauvegarde Hades".into()));
    }
    let mut r = Reader { data, pos: 4 };
    let _checksum = r.u32()?;
    let version = r.u32()?;
    if version != 16 {
        return Err(DriverError::UnsupportedVersion(format!("format de sauvegarde v{version}")));
    }
    let _timestamp = r.take(8)?;
    let _location = r.string()?;
    let runs = r.u32()?;
    let _meta_points = r.u32()?;
    let _shrine_points = r.u32()?;
    let god_mode = r.take(1)?[0] != 0;
    let _hell_mode = r.take(1)?[0] != 0;
    for _ in 0..r.u32()? {
        r.string()?;
    }
    let _map = r.string()?;
    let _next_map = r.string()?;
    let len = r.u32()? as usize;
    let block = r.take(len)?;

    let mut raw = vec![0u8; MAX_STATE];
    let n = lz4_flex::block::decompress_into(block, &mut raw).map_err(|e| DriverError::Corrupt(format!("LZ4 : {e}")))?;
    raw.truncate(n);

    let mut lr = Reader { data: &raw, pos: 0 };
    let count = lr.take(1)?[0];
    if count == 0 {
        return Err(DriverError::Corrupt("état Lua vide".into()));
    }
    let state = lr.value(0)?;
    Ok(Save { runs, god_mode, state })
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], DriverError> {
        let end = self.pos.checked_add(n).filter(|&e| e <= self.data.len()).ok_or_else(|| DriverError::Corrupt("fin de fichier inattendue".into()))?;
        let out = &self.data[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    fn u32(&mut self) -> Result<u32, DriverError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn string(&mut self) -> Result<String, DriverError> {
        let n = self.u32()? as usize;
        Ok(String::from_utf8_lossy(self.take(n)?).into_owned())
    }

    /// One luabins value: `-` nil, `0`/`1` booleans, `N` double, `S` string,
    /// `T` table (array size, hash size, then that many key/value pairs).
    fn value(&mut self, depth: usize) -> Result<Lua, DriverError> {
        if depth > MAX_DEPTH {
            return Err(DriverError::Corrupt("tables imbriquées trop profondément".into()));
        }
        Ok(match self.take(1)?[0] {
            b'-' => Lua::Nil,
            b'0' => Lua::Bool(false),
            b'1' => Lua::Bool(true),
            b'N' => Lua::Num(f64::from_le_bytes(self.take(8)?.try_into().unwrap())),
            b'S' => Lua::Str(self.string()?),
            b'T' => {
                let array = self.u32()? as usize;
                let hash = self.u32()? as usize;
                let total = array.saturating_add(hash);
                // Each pair takes at least two bytes: a larger count is a lie.
                if total > (self.data.len() - self.pos) / 2 {
                    return Err(DriverError::Corrupt("table de taille impossible".into()));
                }
                let mut pairs = Vec::with_capacity(total);
                for _ in 0..total {
                    let k = self.value(depth + 1)?;
                    let v = self.value(depth + 1)?;
                    pairs.push((k, v));
                }
                Lua::Table(pairs)
            }
            t => return Err(DriverError::Corrupt(format!("valeur luabins inconnue « {} »", t as char))),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string(s: &str) -> Vec<u8> {
        let mut v = (s.len() as u32).to_le_bytes().to_vec();
        v.extend_from_slice(s.as_bytes());
        v
    }

    #[test]
    fn decodes_header_and_state() {
        // { GameState = { CompletedRunsCache = 3 } }
        let mut lua = vec![1u8, b'T'];
        lua.extend(0u32.to_le_bytes());
        lua.extend(1u32.to_le_bytes());
        lua.push(b'S');
        lua.extend(string("GameState"));
        lua.push(b'T');
        lua.extend(0u32.to_le_bytes());
        lua.extend(1u32.to_le_bytes());
        lua.push(b'S');
        lua.extend(string("CompletedRunsCache"));
        lua.push(b'N');
        lua.extend(3f64.to_le_bytes());
        let block = lz4_flex::block::compress(&lua);

        let mut file = b"SGB1".to_vec();
        file.extend(0u32.to_le_bytes());
        file.extend(16u32.to_le_bytes());
        file.extend([0u8; 8]);
        file.extend(string("Location_Home"));
        for v in [3u32, 0, 0] {
            file.extend(v.to_le_bytes());
        }
        file.extend([0u8, 0]);
        file.extend(0u32.to_le_bytes());
        file.extend(string("RoomOpening"));
        file.extend(string(""));
        file.extend((block.len() as u32).to_le_bytes());
        file.extend(block);

        let save = decode(&file).unwrap();
        assert_eq!(save.runs, 3);
        assert_eq!(save.state.get("GameState").get("CompletedRunsCache").num(), 3.0);
        assert!(matches!(decode(&file[..file.len() - 5]), Err(DriverError::Corrupt(_))));
    }
}
