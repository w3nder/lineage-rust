use crate::{DatReader, Error};
use std::collections::BTreeMap;

pub const SYMBOL_SLOTS: u32 = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub id: u32,
    pub alias: String,
    pub texture: String,
    pub active: i32,
}

pub fn read(body: &[u8]) -> Result<Vec<Symbol>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| Ok(Symbol {
        id: reader.u32()?, alias: reader.ascf()?, texture: reader.ascf()?, active: reader.i32()?,
    })).collect()
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Symbols {
    pub entries: BTreeMap<u32, Symbol>,
}

impl FromIterator<Symbol> for Symbols {
    fn from_iter<T: IntoIterator<Item = Symbol>>(iter: T) -> Self {
        Self { entries: iter.into_iter().map(|symbol| (symbol.id, symbol)).collect() }
    }
}

impl Symbols {
    pub fn by_id(&self, id: u32) -> Option<&Symbol> {
        if id >= SYMBOL_SLOTS { return None; }
        self.entries.get(&id).filter(|symbol| symbol.active != 0)
    }

    pub fn by_alias(&self, alias: &str) -> Option<&Symbol> {
        self.entries.range(..SYMBOL_SLOTS).map(|(_, symbol)| symbol)
            .find(|symbol| symbol.active != 0 && symbol.alias == alias)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_10432bc0_reads_alias_path_and_signed_activation() {
        let mut bytes = 1u32.to_le_bytes().to_vec();
        bytes.extend(105u32.to_le_bytes());
        bytes.extend(b"\x07graded\0\x0fsymbol.grade_d\0");
        bytes.extend((-1i32).to_le_bytes());
        assert_eq!(read(&bytes).unwrap(), [Symbol { id: 105, alias: "graded".into(), texture: "symbol.grade_d".into(), active: -1 }]);
        for end in 0..bytes.len() { assert!(read(&bytes[..end]).is_err()); }
        assert!(read(&[0; 4]).unwrap().is_empty());
    }

    #[test]
    fn get_symbol_texture_10442a40_uses_active_slots_in_id_order() {
        let mut symbols: Symbols = [
            Symbol { id: 3, alias: "same".into(), texture: "second".into(), active: 1 },
            Symbol { id: 1, alias: "same".into(), texture: "inactive".into(), active: 0 },
            Symbol { id: 2, alias: "same".into(), texture: "first".into(), active: -1 },
            Symbol { id: 200, alias: "outside".into(), texture: "outside".into(), active: 1 },
        ].into_iter().collect();
        assert_eq!(symbols.by_alias("same").unwrap().texture, "first");
        assert!(symbols.by_alias("Same").is_none());
        assert!(symbols.by_alias("outside").is_none());
        assert!(symbols.by_id(1).is_none());
        assert!(symbols.by_id(200).is_none());
        symbols.entries.get_mut(&2).unwrap().active = 0;
        assert_eq!(symbols.by_alias("same").unwrap().id, 3);
    }
}
