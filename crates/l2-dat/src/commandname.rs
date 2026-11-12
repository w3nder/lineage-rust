use crate::{DatReader, Error};
use std::collections::HashMap;

/// `commandname-e.dat` — tabela dos comandos de texto de macro (`/target`, `/delay` etc.).
/// `FL2GameData::CommandDataLoad` (engine.dll) carrega essa tabela num mapa em runtime
/// (`FL2GameData+0x3cf8`), usado por `FL2GameData::GetCommandType` (0x1044c310) pra classificar
/// a primeira palavra de um comando de macro (kind=3 no pacote `SendMacroList`). `token` é o
/// valor comparado pelo stepper de macro (`NConsoleWnd`'s `FUN_100ab100` @ 0x100ab100,
/// `nwindow.dll`) — ver `docs/evidence/decompiled/ui/NConsoleWnd_MacroStepper_100ab100.asm` e
/// `FL2GameData_GetCommandType_1044c310.asm`. Formato lido diretamente do arquivo decifrado
/// (110 registros): `u32 id, u32 token, ascf name`; `token` é sempre `id - 1` (0-based) nesta
/// build, mas o campo é lido separado do arquivo, não recalculado, porque não há garantia de
/// que sempre coincida.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandName {
    pub id: u32,
    pub token: u32,
    pub name: String,
}

pub fn read(body: &[u8]) -> Result<Vec<CommandName>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

fn record(r: &mut DatReader<'_>) -> Result<CommandName, Error> {
    let id = r.u32()?;
    let token = r.u32()?;
    let name = r.ascf()?;
    Ok(CommandName { id, token, name })
}

/// Chave em minúsculas — o cliente original casa o verbo sem diferenciar maiúsculas
/// (`ExecuteCommand`/o tokenizer comparam por `appStricmp`, achado em
/// `docs/evidence/decompiled/ui/NConsoleWnd_MacroStepper_100ab100.asm`).
pub fn by_name(rows: Vec<CommandName>) -> HashMap<String, u32> {
    rows.into_iter().map(|c| (c.name.to_ascii_lowercase(), c.token)).collect()
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

    #[test]
    fn target_is_token_twelve_matching_the_stepper_group_that_resolves_a_name() {
        let mut body = 2u32.to_le_bytes().to_vec();
        body.extend(1u32.to_le_bytes());
        body.extend(0u32.to_le_bytes());
        body.extend(ascf("loc"));
        body.extend(13u32.to_le_bytes());
        body.extend(12u32.to_le_bytes());
        body.extend(ascf("target"));
        let rows = read(&body).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1], CommandName { id: 13, token: 12, name: "target".into() });
        let map = by_name(rows);
        assert_eq!(map.get("TARGET"), None, "a chave é sempre minúscula");
        assert_eq!(map.get("target"), Some(&12));
        assert_eq!(map.get("loc"), Some(&0));
        assert!(read(&body[..body.len() - 1]).is_err());
    }
}
