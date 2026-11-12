use std::collections::BTreeMap;

const LIMIT: usize=256*1024;

#[derive(Default)]
pub struct Preferences {files: BTreeMap<String,String>,revision:u32}

fn name(value: &str) -> Result<String,&'static str> {
    let value=value.to_ascii_lowercase();
    let order_file=value.strip_prefix("iwo/").is_some_and(|file| !file.is_empty() && file.bytes().all(|b| b.is_ascii_digit()||matches!(b,b'_'|b'.')||b.is_ascii_lowercase()));
    if matches!(value.as_str(),"option.ini"|"chatfilter.ini") || order_file {Ok(value)}
    else {Err("arquivo não permitido nas preferências web")}
}

impl Preferences {
    pub fn revision(&self) -> u32 {self.revision}
    pub fn get(&self,key: &str) -> Option<&str> {self.files.get(&key.to_ascii_lowercase()).map(String::as_str)}
    pub fn snapshot(&self) -> BTreeMap<String,String> {self.files.clone()}
    pub fn set(&mut self,key: &str,text: String) -> Result<(),&'static str> {
        let key=name(key)?;
        if text.len()>LIMIT {return Err("preferência excedeu 256 KiB");}
        if self.files.get(&key)!=Some(&text) {
            self.files.insert(key,text);
            self.revision=self.revision.wrapping_add(1);
        }
        Ok(())
    }
    pub fn restore(&mut self,files: BTreeMap<String,String>) -> Result<(),&'static str> {
        let mut next=Self::default();
        for (key,text) in files {
            let key=name(&key)?;
            if next.files.contains_key(&key) {return Err("preferência duplicada");}
            next.set(&key,text)?;
        }
        next.revision=if self.files==next.files {self.revision} else {self.revision.wrapping_add(1)};
        *self=next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn item_order_files_of_the_iwo_folder_are_allowed_and_nothing_else_outside_the_list() {
        let mut settings=Preferences::default();
        settings.set("iwo/1234_1.dat","01000000".into()).unwrap();
        assert!(settings.set("iwo/../x","".into()).is_err());
        assert!(settings.set("iwo/","".into()).is_err());
        assert!(settings.set("other.ini","".into()).is_err());
    }
    #[test]
    fn allowed_files_roundtrip_case_insensitively_without_rewriting_other_values() {
        let mut settings=Preferences::default();
        settings.set("Option.ini","; original\n[Game]\nMyName=True\n".into()).unwrap();
        settings.set("CHATFILTER.INI","[Chat]\nshow=True\n".into()).unwrap();
        let revision=settings.revision();
        settings.set("chatfilter.ini","[Chat]\nshow=True\n".into()).unwrap();
        assert_eq!(settings.revision(),revision);
        let mut restored=Preferences::default();
        restored.restore(settings.snapshot()).unwrap();
        assert_eq!(restored.get("OPTION.INI"),settings.get("option.ini"));
        assert_eq!(restored.get("ChatFilter.ini"),Some("[Chat]\nshow=True\n"));
    }
    #[test]
    fn rejected_restores_preserve_previous_preferences() {
        let mut settings=Preferences::default();
        settings.set("option.ini","original".into()).unwrap();
        for values in [
            BTreeMap::from([("../option.ini".into(),"outside".into())]),
            BTreeMap::from([("engine.dll".into(),"asset".into())]),
            BTreeMap::from([("OPTION.INI".into(),"a".into()),("option.ini".into(),"b".into())]),
            BTreeMap::from([("option.ini".into(),"a".repeat(LIMIT+1))]),
        ] {
            assert!(settings.restore(values).is_err());
            assert_eq!(settings.get("option.ini"),Some("original"));
        }
    }
}
