use crate::{DatReader, Error};

/// Um material da receita: o par lido por `0x10432930` (dois inteiros de 4 bytes, item e
/// quantidade), na ordem `+0x00`, `+0x04`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecipeMaterial {
    pub item_id: i32,
    pub count: i32,
}

/// Uma linha de `Recipe-c.dat`. A ordem vem do serializador do registro (`0x10449440`,
/// chamado por `FL2GameData::RecipeDataLoad @ 0x10462870` depois da contagem): a `FString`
/// de `+0x10` e oito inteiros de `+0x1c` a `+0x38`, seguidos de `+0x38` materiais. Os nomes
/// vêm do caminho de texto da mesma função, que preenche os mesmos campos por nome: `name`,
/// `id`, `recipe_id`, `level`, `product_id`, `product_num`, `mp_consume` e `success_rate`; o
/// último é a contagem de materiais, que o caminho de texto limita a 20 e o binário não.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    pub name: String,
    pub id: i32,
    pub recipe_id: i32,
    pub level: i32,
    pub product_id: i32,
    pub product_num: i32,
    pub mp_consume: i32,
    pub success_rate: i32,
    pub materials: Vec<RecipeMaterial>,
}

pub fn read(body: &[u8]) -> Result<Vec<Recipe>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count)
        .map(|_| {
            let name = reader.ascf()?;
            let id = reader.i32()?;
            let recipe_id = reader.i32()?;
            let level = reader.i32()?;
            let product_id = reader.i32()?;
            let product_num = reader.i32()?;
            let mp_consume = reader.i32()?;
            let success_rate = reader.i32()?;
            let materials = reader.i32()?;
            let materials = (0..materials.max(0))
                .map(|_| Ok(RecipeMaterial { item_id: reader.i32()?, count: reader.i32()? }))
                .collect::<Result<_, Error>>()?;
            Ok(Recipe { name, id, recipe_id, level, product_id, product_num, mp_consume, success_rate, materials })
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

    fn record(id: i32, materials: &[(i32, i32)]) -> Vec<u8> {
        let mut out = ascf("mk_wooden_arrow");
        for value in [id, 1666, 1, 17, 500, 30, 100, materials.len() as i32] {
            out.extend(value.to_le_bytes());
        }
        for (item, count) in materials {
            out.extend(item.to_le_bytes());
            out.extend(count.to_le_bytes());
        }
        out
    }

    #[test]
    fn records_follow_the_10449440_serializer_order() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend(record(1, &[(1864, 4), (1878, 2)]));
        body.extend(record(2, &[]));
        let rows = read(&body).unwrap();
        assert_eq!(rows[0], Recipe {
            name: "mk_wooden_arrow".into(), id: 1, recipe_id: 1666, level: 1, product_id: 17,
            product_num: 500, mp_consume: 30, success_rate: 100,
            materials: vec![RecipeMaterial { item_id: 1864, count: 4 }, RecipeMaterial { item_id: 1878, count: 2 }],
        });
        assert!(rows[1].materials.is_empty());
        assert!(read(&body[..body.len() - 1]).is_err());
    }
}
