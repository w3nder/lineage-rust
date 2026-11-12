use killer_bot_core::script_host::ScriptStatus;
use serde::Serialize;
use std::path::{Path, PathBuf};

pub const EXTENSION: &str = "killer";

pub const EXAMPLES: [(&str, &str); 7] = [
    ("Buff em mim a cada 10 s (sem alvo)", "while true\n  use_skill \"Wind Walk\"\n  print \"buff enviado\"\n  wait 10000\nend\n"),
    ("Falar no chat geral", "say \"all\" \"teste\"\nprint \"mensagem enviada\"\n"),
    (
        "Farm simples",
        "while true\n  if is_dead\n    wait 2000\n  elif has_target and not target_dead\n    attack\n    wait 1200\n  else\n    target_nearest\n    wait 800\n  end\nend\n",
    ),
    (
        "Descansar com HP baixo",
        "while true\n  if hp < 40 and not sitting\n    sit\n  elif hp > 90 and sitting\n    stand\n  end\n  wait 1000\nend\n",
    ),
    (
        "Responder quando chamarem GM",
        "on chat \"gm\"\n  print event_text\n  say \"all\" \"estou aqui\"\nend\n",
    ),
    (
        "Avisar a morte",
        "on death\n  print \"morri no nivel\" level\n  say \"party\" \"morri, me ajudem\"\nend\non revive\n  say \"party\" \"voltei\"\nend\n",
    ),
    (
        "Skill de ataque a cada 10 s (precisa de alvo)",
        "set vezes = 0\nwhile vezes < 100\n  if has_target and not target_dead and skill_ready 56\n    use_skill 56\n    set vezes = vezes + 1\n  end\n  wait 10000\nend\n",
    ),
];

pub fn clean_name(name: &str) -> String {
    let trimmed = name.trim();
    let trimmed = trimmed.strip_suffix(&format!(".{EXTENSION}")).unwrap_or(trimmed);
    let cleaned: String = trimmed.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' }).collect();
    cleaned.trim().to_owned()
}

pub fn path_of(dir: &Path, name: &str) -> Option<PathBuf> {
    let clean = clean_name(name);
    (!clean.is_empty()).then(|| dir.join(format!("{clean}.{EXTENSION}")))
}

pub fn save_script(dir: &Path, name: &str, text: &str) -> Result<PathBuf, String> {
    let path = path_of(dir, name).ok_or("dê um nome ao script")?;
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    std::fs::write(&path, text).map_err(|error| error.to_string())?;
    Ok(path)
}

pub fn load_script(dir: &Path, name: &str) -> Result<String, String> {
    let path = path_of(dir, name).ok_or("dê um nome ao script")?;
    std::fs::read_to_string(path).map_err(|error| error.to_string())
}

pub fn list_scripts(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let path = entry.path();
                    (path.extension().and_then(|ext| ext.to_str()) == Some(EXTENSION)).then(|| path.file_stem().map(|stem| stem.to_string_lossy().into_owned())).flatten()
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExampleDto {
    pub title: &'static str,
    pub body: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VariableDto {
    pub name: String,
    pub value: String,
    pub kind: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LoadedDto {
    pub seq: u32,
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskDto {
    pub handler: bool,
    pub line: usize,
    pub wait_left_ms: Option<u64>,
}

pub fn tasks(session: &killer_bot_core::session::BotSession) -> Vec<TaskDto> {
    let now = session.script_now();
    session.script_tasks().into_iter().map(|task| TaskDto { handler: task.handler, line: task.line, wait_left_ms: task.wake.map(|wake| wake.saturating_sub(now).as_millis() as u64) }).collect()
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScriptDto {
    pub tasks: Vec<TaskDto>,
    pub kind: &'static str,
    pub status: String,
    pub paused: bool,
    pub message: String,
    pub variables: Vec<VariableDto>,
    pub files: Vec<String>,
    pub loaded: Option<LoadedDto>,
    pub log: Vec<String>,
}

pub fn status_kind(status: &ScriptStatus) -> &'static str {
    match status {
        ScriptStatus::None => "none",
        ScriptStatus::Running => "running",
        ScriptStatus::Waiting => "waiting",
        ScriptStatus::Finished => "finished",
        ScriptStatus::Failed(_) => "failed",
    }
}

pub fn variables(list: &[(String, killer_bot_script::Value)]) -> Vec<VariableDto> {
    list.iter()
        .map(|(name, value)| VariableDto {
            name: name.clone(),
            value: match value {
                killer_bot_script::Value::Text(text) => format!("\"{text}\""),
                other => other.to_string(),
            },
            kind: value.type_name(),
        })
        .collect()
}

pub fn examples() -> Vec<ExampleDto> {
    EXAMPLES.iter().map(|(title, body)| ExampleDto { title, body }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_scripts_all_parse_and_compile() {
        for (title, body) in EXAMPLES {
            killer_bot_script::build(body).unwrap_or_else(|error| panic!("{title}: {error}"));
        }
    }

    #[test]
    fn script_status_text_includes_the_error_line() {
        let error = killer_bot_script::build("if hp < 3\nsit\n").unwrap_err();
        let text = ScriptStatus::Failed(error).text(false);
        assert!(text.starts_with("Erro: linha 1"));
        assert_eq!(ScriptStatus::Waiting.text(true), "Waiting");
    }

    #[test]
    fn script_files_round_trip_in_the_scripts_folder() {
        let dir = std::env::temp_dir().join(format!("killer-scripts-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert!(list_scripts(&dir).is_empty());
        let path = save_script(&dir, "Meu farm/1", "attack\n").unwrap();
        assert!(path.ends_with("Meu farm_1.killer"));
        assert_eq!(load_script(&dir, "Meu farm_1.killer").unwrap(), "attack\n");
        save_script(&dir, "a", "stop\n").unwrap();
        assert_eq!(list_scripts(&dir), vec!["Meu farm_1", "a"]);
        assert!(save_script(&dir, "  ", "x").is_err());
        assert!(load_script(&dir, "nao-existe").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
