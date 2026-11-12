use killer_assets::client::{AssetError, Client, DatTables};
use killer_assets::language::ClientLanguage;
use std::sync::Arc;

#[derive(Default)]
pub struct SessionTables {
    entry: Option<(ClientLanguage, Arc<DatTables>)>,
}

impl SessionTables {
    pub fn clear(&mut self) {
        self.entry = None;
    }

    pub fn load(&mut self, client: &mut Client) -> Result<Arc<DatTables>, AssetError> {
        let language = client.client_language();
        self.load_with(language, |language| client.tables_for_language(language))
    }

    fn load_with<E>(&mut self, language: ClientLanguage, load: impl FnOnce(&ClientLanguage) -> Result<DatTables, E>) -> Result<Arc<DatTables>, E> {
        if let Some((cached_language, tables)) = &self.entry
            && cached_language == &language {
                return Ok(tables.clone());
            }
        let tables = Arc::new(load(&language)?);
        self.entry = Some((language, tables.clone()));
        Ok(tables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn original_tables_are_shared_without_redecoding_any_records() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let mut cache = SessionTables::default();
        let start = std::time::Instant::now();
        let original = cache.load(&mut client).unwrap();
        let cold = start.elapsed();
        let start = std::time::Instant::now();
        let shared = cache.load(&mut client).unwrap();
        let warm = start.elapsed();
        assert!(Arc::ptr_eq(&original, &shared));
        assert!(!original.itemname.is_empty());
        assert!(!original.weapongrp.is_empty());
        assert!(!original.armorgrp.is_empty());
        assert!(!original.skillgrp.is_empty());
        eprintln!("tabelas originais: {:?} leitura / {:?} reuso; {} itens, {} armas, {} armaduras; mesma alocação integral", cold, warm, original.itemname.len(), original.weapongrp.len(), original.armorgrp.len());
    }

    #[test]
    fn reuse_requires_same_language_and_mounted_files_and_never_publishes_failures() {
        let mut cache = SessionTables::default();
        let language = ClientLanguage::parse("", "");
        let first = cache.load_with(language.clone(), |_| Ok::<_, ()>(DatTables::default())).unwrap();
        let reused = cache.load_with(language.clone(), |_| Err(())).unwrap();
        assert!(Arc::ptr_eq(&first, &reused));
        for changed in [
            ClientLanguage { language_type: 1, ..language.clone() },
            ClientLanguage { country: "Custom".into(), ..language.clone() },
            ClientLanguage { code: "e".into(), ..language.clone() },
            ClientLanguage { is_native: false, ..language.clone() },
        ] {
            assert!(cache.load_with(changed.clone(), |_| Err(())).is_err());
            assert!(Arc::ptr_eq(&first, &cache.load_with(language.clone(), |_| Err(())).unwrap()));
            let replacement = cache.load_with(changed, |_| Ok::<_, ()>(DatTables::default())).unwrap();
            assert!(!Arc::ptr_eq(&first, &replacement));
            cache.entry = Some((language.clone(), first.clone()));
        }
        cache.clear();
        assert!(cache.load_with(language.clone(), |_| Err(())).is_err());
        let replacement = cache.load_with(language, |_| Ok::<_, ()>(DatTables::default())).unwrap();
        assert!(!Arc::ptr_eq(&first, &replacement));
    }
}
