use crate::{DatReader, Error};

/// Uma linha de `hennagrp-e.dat`. A ordem vem do serializador do registro
/// (`0x104329f0`, chamado pelo `FL2GameData::HennaDataLoad @ 0x10458ab0` uma vez por registro
/// depois de ler a contagem): dois inteiros de 4 bytes e quatro `FString`, nos campos `+0x00`,
/// `+0x04`, `+0x08`, `+0x14`, `+0x20` e `+0x2c` do registro de `0x38` bytes. Os nomes saem do
/// caminho de texto da mesma função, que preenche os mesmos campos por nome: `symbol_id`,
/// `dye_item_id`, `han_name`, `symbol_icon`, `symbol_description` e `symbol_add_name`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HennaGrp {
    pub symbol_id: u32,
    pub dye_item_id: u32,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub add_name: String,
}

pub fn read(body: &[u8]) -> Result<Vec<HennaGrp>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count)
        .map(|_| {
            Ok(HennaGrp {
                symbol_id: reader.u32()?,
                dye_item_id: reader.u32()?,
                name: reader.ascf()?,
                icon: reader.ascf()?,
                description: reader.ascf()?,
                add_name: reader.ascf()?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ascf(s: &str) -> Vec<u8> {
        let mut out = vec![(s.len() + 1) as u8];
        out.extend(s.bytes());
        out.push(0);
        out
    }

    fn record(symbol_id: u32, dye_item_id: u32, name: &str) -> Vec<u8> {
        let mut out = symbol_id.to_le_bytes().to_vec();
        out.extend(dye_item_id.to_le_bytes());
        out.extend(ascf(name));
        out.extend(ascf("icon.henna"));
        out.extend(ascf("descricao"));
        out.extend(ascf("+STR -CON"));
        out
    }

    /// A ordem dos campos é a do serializador do registro em `0x104329f0`: `symbol_id`,
    /// `dye_item_id`, e então nome, ícone, descrição e nome adicional.
    #[test]
    fn records_are_two_ids_and_four_strings_in_the_serializer_order() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend(record(1, 4445, "Symbol of Power"));
        body.extend(record(2, 4446, "Symbol of Grace"));
        let rows = read(&body).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0],
            HennaGrp {
                symbol_id: 1,
                dye_item_id: 4445,
                name: "Symbol of Power".into(),
                icon: "icon.henna".into(),
                description: "descricao".into(),
                add_name: "+STR -CON".into(),
            }
        );
        assert_eq!(rows[1].symbol_id, 2);
        assert!(read(&body[..body.len() - 1]).is_err(), "registro truncado é erro");
    }
}
