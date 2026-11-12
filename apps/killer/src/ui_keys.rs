use killer_assets::client::DatTables;
use killer_ui::abnormal::{self, AbnormalHit, AbnormalView};
use killer_ui::actions::{self, ActionEntry, ActionsView};
use killer_ui::detail_status::{DetailStatusView, Sysstrings};
use killer_ui::draw::DrawList;
use killer_ui::font::BitmapFont;
use killer_ui::interface_sound;
use killer_ui::inventory::{
    self, InventoryHit, InventoryItemView, InventoryView, PaperdollIds, QUEST_TYPE2, item_wnd_wheel, unequip_mask,
};
use killer_ui::keys::{self, Effect, Hooked, KeyState, MainTab, VK_F1};
use killer_ui::mainwnd::{self, MainWndView, PAGE_RECT};
use killer_ui::minimap::{self, MinimapButton, MinimapHit, MinimapView};
use killer_ui::command_channel::{self, Action as ChannelAction, ChannelInfo, CommandChannelView, Labels as ChannelLabels};
use killer_ui::party_room::{self, MakingHit, MakingView, PartyRoomView, RoomAction, RoomData, RoomLabels};
use killer_ui::pledge::{MemberRow, PledgeButton, PledgeHit, PledgeView};
use killer_ui::quest::{QuestHit, QuestItem, QuestNode, QuestStep, QuestView, states_of};

use killer_ui::scroll::{ScrollHit, line_down, line_up};
use killer_ui::modern_shortcut::{ShotKind, Vital};
use killer_ui::shortcut::{BarEvent, DropTarget, ShortcutSlot, ShortcutView, SlotKind};
use killer_ui::skills::{SkillHit, SkillView, SkillsView};
use killer_ui::sysmenu::{self, SysMenuButton, SysMenuView};
use killer_ui::tooltip::{LV_SYSSTRING, Tooltip, TooltipLine, VALUE_COLOR, WEIGHT_SYSSTRING};
use killer_ui::window::{Desktop, Rect, Window, WindowId};
use killer_world::{Dialog, EntityKind, NetCommand, World, WorldEvent};
use l2_dat::etcitemgrp::ETCITEM_TYPE_RECIPE;
use l2_net::game::packets::{
    PAPERDOLL_L_EAR, PAPERDOLL_L_FINGER, PAPERDOLL_R_EAR, PAPERDOLL_R_FINGER, SHORTCUT_SLOTS_PER_PAGE,
};
use l2_net::game::{ShortcutKind, SystemMessage};
use std::collections::HashMap;
use killer_platform::time::Instant;
use winit::keyboard::KeyCode;

pub const FORCE_PETI_SYSMSG: u32 = 1528;
pub const SYSMENU_WINDOW: WindowId = WindowId::SYSTEM_MENU;
const AUTO_POTION_INTERVAL: std::time::Duration = std::time::Duration::from_millis(1500);
pub const DEMO_ADENA: i32 = 152_340;
pub const DEMO_LOAD: (i32, i32) = (2_310, 8_290);
pub const DEMO_INVENTORY_LIMIT: i32 = 80;
pub const DEMO_ITEMS: [(i32, i32, i32, bool, i32); 9] = [
    (57, 100, DEMO_ADENA, false, 0),
    (1, 101, 1, true, inventory::BODY_PART_R_HAND),
    (18, 102, 1, true, inventory::BODY_PART_L_HAND),
    (21, 103, 1, true, inventory::BODY_PART_CHEST),
    (28, 104, 1, true, inventory::BODY_PART_LEGS),
    (48, 105, 1, true, inventory::BODY_PART_GLOVES),
    (35, 106, 1, true, inventory::BODY_PART_FEET),
    (1060, 107, 15, false, 0),
    (17, 108, 500, false, 0),
];
pub const DEMO_SKILLS: [(bool, i32, i32); 4] = [(false, 9, 3), (false, 1, 1177), (true, 1, 194), (false, 1, 56)];
pub const DEMO_SHORTCUTS: [(i32, i32); 4] = [(0, 107), (1, 3), (2, 1177), (3, 2)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemDropTarget {
    World,
    Trash,
    Crystallize,
    Pet,
    PetEquipped,
    FromPet,
}

const QUEST_ITEMS_MAX: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayRecord {
    Start,
    Stop,
    Toggle,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiCommand {
    Net(NetCommand),
    ReplayRecord(ReplayRecord),
    DebugCommand { window: bool },
    ServerTransfer { id: i32 },
    ToggleHelp { macro_help: bool },
    LoadHelp(String),
    LoadPetitionPage(String),
    OpenPetition(Option<String>),
    GmEventFile { file: String, commands: bool },
    CrestFile { kind: killer_assets::crest::CrestKind, path: String },
    Log(String),
    InterfaceSound(&'static str),
    SystemMessage(u32),
    ColoredSystemMessage { id: u32, color: u32 },
    WarningMessage { id: u32, numbers: Vec<i32> },
    /// `FLandMark::DeleteLandMark(0)` que o `NConsoleWnd::ActionOnTarget` (`0x10071cb0`) chama
    /// depois de pedir `Action`/`Attack` (macros `/attack` e `/attackstand`).
    DeleteLandMark,
    /// Macro arrastado pra lixeira da janela de lista: pede a confirmação `0x33c` antes de
    /// mandar o `RequestDeleteMacro`, como `NCMacroArrayWnd::OnLButtonUp @ 0x100a9b47` faz.
    ConfirmDeleteMacro { id: i32, name: String },
    ConfirmDeleteRecipe { recipe_id: i32, name: String },
    OpenPetNameDialog,
    ItemDrop { object_id: i32, item_id: i32, count: i32, name: String, target: ItemDropTarget },
    /// OK da janela de opções: grava no `option.ini` os campos que o cliente modela ali
    /// (transparência, enter-chat, janela/dano/consumíveis das mensagens de sistema).
    ApplyOptions { close: bool },
    InitializeInterface,
    OptionResolutionChanged(i32),
    OptionKeepMinFrameRateChanged,
    ToggleMinFrameRate,
}


/// Atraso de cada comando de macro (`FUN_100a8120` @ `0x100a8120`, `nwindow.dll`): o campo
/// `+0xf0` do comando atual, que o stepper (`FUN_100ab100`) usa pra decidir quando avançar.
/// Para `kind == 2` (ação) o valor sai de `APawn::GetSocialAnimDuration(<anim>)` — resolvido via
/// tabela de import (`0x1019e428` -> entrada 26 do bloco `Engine.dll`) — com o `<anim>` vindo da
/// tabela de bytes em `0x100a87cc` (36 entradas, indexada pelo id de ação de `actionname-e.dat`)
/// somada à tabela de saltos em `0x100a8794`. Só ações que tocam animação social têm atraso;
/// todo o resto mapeia pro salto 13, que deixa o atraso zerado. Cada par abaixo confere com o id
/// social que o próprio dispatcher usa (ex.: ação 12 "socialhello" -> anim 2, e
/// `FUN_10058330` faz `FUN_10068550(2)` pro `/socialhello`).
const MACRO_ACTION_SOCIAL_ANIM: &[(i32, usize)] = &[
    (12, 2),  // socialhello
    (13, 3),  // socialvictory
    (14, 4),  // socialcharge
    (24, 6),  // socialyes
    (25, 5),  // socialno
    (26, 7),  // socialbow
    (29, 8),  // socialunaware
    (30, 9),  // socialwaitinga
    (31, 10), // sociallaugh
    (33, 11), // socialapplause
    (34, 12), // socialdance
    (35, 13), // socialsad
];

/// Ação 0 ("sitstand") é o caso especial do `0x100a8247`: o atraso depende do estado atual —
/// de pé toca a animação de sentar (social 0), sentado toca a de levantar (social 1).
const MACRO_ACTION_SITSTAND: i32 = 0;

/// Execução de macro em andamento. O original roda **um comando por vez**: dispara o primeiro na
/// hora e só avança pro próximo quando o atraso do atual termina (`FUN_100ab100`, campos `+0xa48`
/// índice atual, `+0xf8` tempo acumulado, `+0xf0` limite).
#[derive(Debug, Clone, PartialEq)]
struct MacroRun {
    macro_id: i32,
    index: usize,
    started: Instant,
    delay: f32,
    waiting_skill: Option<i32>,
}

/// Raio das buscas por proximidade dos comandos de macro `/targetnext`, `/pickup` e
/// `/petcollect` — o dispatcher original passa 200 literal nas três (`FUN_10071a70(200, …)`,
/// `FUN_100719e0(200, …)`, `FUN_10069110(200, …)` em `FUN_10058330`).
pub const MACRO_SEARCH_RADIUS: f32 = 200.0;

/// Comandos de texto de macro que no original são `FUN_1007e980(<id>, 0, 0)` — ou seja, usar a
/// ação de `actionname-e.dat` com esse id. Cada par foi conferido contra o campo `cmd` da ação
/// correspondente na tabela real (ex.: token `0x39` "sitstand" -> ação 0, cujo `cmd` é
/// "sitstand"). Token esquerdo: `commandname-e.dat`. Id direito: `actionname-e.dat`.
/// Texto que a janela de macro mostra pra cada comando. O original remonta a linha no
/// `NCMacroArrayWnd::OnLButtonUp @ 0x100a9a40`, um ramo por `kind`, sempre com a `/` de
/// `0x1019f24c` na frente e o verbo tirado da tabela de `commandname-e.dat` (o array de `FString`
/// indexado pelo token: `useskill` = `0x18` em `+0xa14e8`, `useshortcut` = `0x1b` em `+0xa150c`,
/// `delay` = `0x4c` em `+0xa1758`):
///
/// - `kind 1`: `"%s%s %s"` (`0x101bb20c`) — `/useskill <nome da skill>`, com o nome vindo da
///   `skillname-e.dat` pelo par id+nível (`0x100a9d46`);
/// - `kind 2`: `"%s%s"` (`0x101abd74`) — `/<verbo da ação>`;
/// - `kind 3`: `"%s"` (`0x1019f34c`) — o texto cru, do jeito que veio;
/// - `kind 4`: `"%s%s %d %d"` (`0x101bb1f4`) — `/useshortcut <página+1> <slot+1>`;
/// - `kind 6`: `"%s%s %d"` (`0x101bb1e4`) — `/delay <segundos>`.
///
/// O verbo do `kind 2` é o de `FL2GameData::GetCmdStringByActionID` (engine.dll 0x1045ebc0): o `cmd`
/// da ação passa pela tabela estática de 110 nomes do engine (`l2_dat::actionname::COMMAND_TOKENS`)
/// e o token achado indexa o verbo de `commandname-e.dat`. Ação sem token fica com o id cru.
const SHORTCUT_OWNER_PET: i32 = 2;

pub fn is_dwarven_class(class_id: i32) -> bool {
    matches!(class_id, 0x35..=0x39 | 0x75 | 0x76)
}

fn macro_line(
    command: &l2_net::game::packets::MacroCommand,
    skillname: &HashMap<(u32, u32), l2_dat::skillname::SkillName>,
    commandname: &HashMap<String, u32>,
    action_commands: &HashMap<i32, String>,
) -> String {
    let verb = |token: u32| {
        commandname.iter().find(|(_, t)| **t == token).map(|(name, _)| name.clone()).unwrap_or_default()
    };
    match command.kind {
        1 => {
            let key = (command.skill_id.max(0) as u32, u32::from(command.shortcut_id));
            let name = skillname.get(&key).map_or_else(|| command.skill_id.to_string(), |s| s.name.clone());
            format!("/{} {name}", verb(MACRO_VERB_USESKILL))
        }
        2 => match action_verb(action_commands, commandname, command.skill_id) {
            Some(verb) => format!("/{verb}"),
            None => format!("/{}", command.skill_id),
        },
        3 => command.command.clone(),
        4 => format!("/{} {} {}", verb(MACRO_VERB_USESHORTCUT), command.skill_id + 1, i32::from(command.shortcut_id) + 1),
        6 => format!("/{} {}", verb(MACRO_VERB_DELAY), command.skill_id),
        _ => command.command.clone(),
    }
}

/// Transforma uma linha de texto do editor no comando que vai no `RequestMakeMacro`. É o porte
/// de `FUN_100aa7a0` (chamado pelo Save, `NCMacroEditWnd` slot `0x128` = `0x100aba70`), lido
/// ramo a ramo. O verbo é tokenizado com a mesma tabela `commandname-e.dat` do dispatcher, e o
/// `switch` sobre o token é:
///
/// - `0x18` `useskill` (`0x100aa8a7`): junta o resto das palavras com espaço, procura o nome na
///   tabela de skills e grava `kind 1` com o id achado e nível 1; nome desconhecido vira id 0 e
///   nível 0.
/// - `0x19` `useskillforce` e `0x1a` `useskillstand`: caem no ramo de texto livre.
/// - `0x1b` `useshortcut` (`0x100aa964`): dois números, o primeiro de 1 a 10 e o segundo de 1 a
///   12; grava `kind 4` com os dois menos um. Fora da faixa, a linha é descartada.
/// - `0x4c` `delay` (`0x100aa9fa`): um número maior ou igual a 1, gravado no campo de 4 bytes —
///   `kind 6`. Sem número, a linha é descartada.
/// - qualquer outro token abaixo de `0x6e`, e o texto que o tokenizador não reconhece
///   (`0x100aaa53`): `kind 3`, com a linha inteira como texto.
/// - token `0x6e` ou acima: a linha é descartada.
pub fn macro_command_from_line(
    line: &str,
    index: u8,
    skillname: &HashMap<(u32, u32), l2_dat::skillname::SkillName>,
    commandname: &HashMap<String, u32>,
) -> Option<l2_net::game::packets::MacroCommand> {
    let command = |kind: u8, skill_id: i32, shortcut_id: u8, text: &str| {
        Some(l2_net::game::packets::MacroCommand { index, kind, skill_id, shortcut_id, command: text.to_string() })
    };
    let free_text = || command(3, 0, 0, line);
    let mut words = line.split_whitespace();
    let Some(verb) = words.next() else { return free_text() };
    let Some(token) = commandname.get(&verb.trim_start_matches('/').to_ascii_lowercase()).copied() else {
        return free_text();
    };
    match token {
        MACRO_VERB_USESKILL => {
            let name = words.collect::<Vec<_>>().join(" ");
            match skill_by_name(skillname, &name) {
                Some(id) => command(1, id, 1, ""),
                None => command(1, 0, 0, ""),
            }
        }
        MACRO_VERB_USESHORTCUT => {
            let page: i32 = words.next()?.parse().ok()?;
            let slot: i32 = words.next()?.parse().ok()?;
            ((1..=10).contains(&page) && (1..=12).contains(&slot))
                .then(|| command(4, page - 1, u8::try_from(slot - 1).ok()?, ""))?
        }
        MACRO_VERB_DELAY => {
            let seconds: i32 = words.next()?.parse().ok()?;
            (seconds >= 1).then(|| command(6, seconds, 0, ""))?
        }
        token if token >= MACRO_TOKEN_MAX => None,
        _ => free_text(),
    }
}

pub const MACRO_CODE_NONE: i64 = -1;
pub const MACRO_UPDATE_DELETE: u8 = 0;
pub const MACRO_CODE_CHAT: i64 = 0x2000_0000;
pub const MACRO_CODE_BUILDER: i64 = 0x1000_0000;
pub const MACRO_CODE_BUILDER_ALT: i64 = 0x1000_0001;

pub fn macro_line_code(line: &str, commandname: &HashMap<String, u32>) -> i64 {
    if !line.starts_with('/') {
        return MACRO_CODE_CHAT;
    }
    let first = line.split(|c: char| c.is_whitespace() || c == ',').find(|word| !word.is_empty()).unwrap_or("");
    let units = first.chars().count();
    if units > 3 && first.starts_with("///") {
        return MACRO_CODE_BUILDER_ALT;
    }
    if units > 2 && first.starts_with("//") {
        return MACRO_CODE_BUILDER;
    }
    if units > 1 {
        return commandname.get(&first[1..].to_ascii_lowercase()).map_or(MACRO_CODE_NONE, |token| i64::from(*token));
    }
    MACRO_CODE_NONE
}

fn macro_line_tokens(line: &str) -> impl Iterator<Item = &str> {
    line.split(|c: char| c.is_whitespace() || c == ',').filter(|word| !word.is_empty()).skip(1)
}

fn macro_atoi(token: &str) -> i32 {
    let digits = token.trim_start();
    let (sign, rest) = match digits.strip_prefix('-') {
        Some(rest) => (-1i64, rest),
        None => (1, digits.strip_prefix('+').unwrap_or(digits)),
    };
    let value: i64 = rest.chars().take_while(char::is_ascii_digit).fold(0i64, |acc, c| (acc * 10 + i64::from(c as u8 - b'0')).min(i64::from(u32::MAX)));
    (sign * value).clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

pub fn macro_validate_lines(
    lines: &[String],
    commandname: &HashMap<String, u32>,
    shortcut_is_macro: &dyn Fn(i32, i32) -> bool,
    builder: bool,
) -> bool {
    let mut filled = true;
    for (index, line) in lines.iter().enumerate() {
        if line.is_empty() {
            if filled {
                if index == 0 {
                    return false;
                }
                filled = false;
            }
            continue;
        }
        if !filled {
            return false;
        }
        let code = macro_line_code(line, commandname);
        if code == i64::from(MACRO_VERB_DELAY) {
            if macro_line_tokens(line).next().is_none_or(|token| macro_atoi(token) < 1) {
                return false;
            }
        } else if (0x1b..=0x1d).contains(&code) {
            let mut tokens = macro_line_tokens(line);
            let Some(page) = tokens.next().map(macro_atoi).filter(|page| (1..=10).contains(page)) else { return false };
            let Some(slot) = tokens.next().map(macro_atoi).filter(|slot| (1..=12).contains(slot)) else { return false };
            if shortcut_is_macro(page, slot) {
                return false;
            }
        } else if code == MACRO_CODE_BUILDER || code == MACRO_CODE_BUILDER_ALT {
            if !builder {
                return false;
            }
        } else {
            return code < i64::from(MACRO_TOKEN_MAX) || code == MACRO_CODE_CHAT;
        }
    }
    true
}

pub fn macro_save_records(
    lines: &[String],
    skillname: &HashMap<(u32, u32), l2_dat::skillname::SkillName>,
    commandname: &HashMap<String, u32>,
    builder: bool,
) -> (usize, Vec<l2_net::game::packets::MacroCommand>) {
    use l2_net::game::packets::MacroCommand;
    let count = lines.iter().position(String::is_empty).unwrap_or(lines.len());
    let mut records = Vec::new();
    let mut number: u8 = 0;
    for line in lines {
        let code = if line.is_empty() { MACRO_CODE_NONE } else { macro_line_code(line, commandname) };
        let record = |number: u8, kind: u8, skill_id: i32, shortcut_id: u8, text: &str| MacroCommand {
            index: number,
            kind,
            skill_id,
            shortcut_id,
            command: text.to_string(),
        };
        let mut plain = false;
        if code > i64::from(MACRO_VERB_DELAY) {
            if code >= MACRO_CODE_BUILDER {
                if code <= MACRO_CODE_BUILDER_ALT {
                    plain = builder;
                } else if code == MACRO_CODE_CHAT {
                    plain = true;
                }
            } else if code < i64::from(MACRO_TOKEN_MAX) {
                plain = true;
            }
        } else if code == i64::from(MACRO_VERB_DELAY) {
            number = number.wrapping_add(1);
            if let Some(seconds) = macro_line_tokens(line).next().map(macro_atoi).filter(|seconds| *seconds >= 1) {
                records.push(record(number, 6, seconds, 0, ""));
            }
        } else if (i64::from(MACRO_VERB_USESKILL)..=i64::from(MACRO_VERB_USESKILL) + 3).contains(&code) {
            if code == i64::from(MACRO_VERB_USESKILL) {
                let name = macro_line_tokens(line).collect::<Vec<_>>().join(" ");
                number = number.wrapping_add(1);
                match skill_by_name(skillname, &name) {
                    Some(id) => records.push(record(number, 1, id, 1, "")),
                    None => records.push(record(number, 1, 0, 0, "")),
                }
            } else if code == i64::from(MACRO_VERB_USESHORTCUT) {
                number = number.wrapping_add(1);
                let mut tokens = macro_line_tokens(line);
                let page = tokens.next().map(macro_atoi).filter(|page| (1..=10).contains(page));
                let slot = tokens.next().map(macro_atoi).filter(|slot| (1..=12).contains(slot));
                if let (Some(page), Some(slot)) = (page, slot) {
                    records.push(record(number, 4, page - 1, (slot - 1) as u8, ""));
                }
            } else {
                plain = true;
            }
        } else if code < i64::from(MACRO_TOKEN_MAX) {
            plain = true;
        }
        if plain {
            number = number.wrapping_add(1);
            records.push(record(number, 3, 0, 0, line));
        }
    }
    (count, records)
}

/// Procura a skill pelo nome exato, sem diferenciar maiúsculas, como o original faz no Save
/// (`*0x1019e530` sobre a `GL2GameData`).
fn skill_by_name(skillname: &HashMap<(u32, u32), l2_dat::skillname::SkillName>, name: &str) -> Option<i32> {
    skillname
        .values()
        .filter(|skill| skill.name.eq_ignore_ascii_case(name))
        .map(|skill| skill.id as i32)
        .min()
}

/// Token a partir do qual o Save descarta a linha (`cmpl $0x6e` em `0x100aaa53`).
const MACRO_TOKEN_MAX: u32 = 0x6e;

/// Tokens de `commandname-e.dat` que o original usa pra montar as linhas da janela de macro.
const MACRO_VERB_USESKILL: u32 = 0x18;
const MACRO_VERB_USESHORTCUT: u32 = 0x1b;
const MACRO_VERB_DELAY: u32 = 0x4c;

fn action_verb(actions: &HashMap<i32, String>, commandname: &HashMap<String, u32>, id: i32) -> Option<String> {
    let token = l2_dat::actionname::command_token(actions.get(&id)?)?;
    commandname.iter().find(|(_, value)| **value == token).map(|(name, _)| name.clone())
}

const TOGGLE_OPERATE_TYPE: u32 = 3;

const MACRO_FORBIDDEN_ACTIONS: [i32; 5] = [0x14, 0x1b, 0x20, 0x24, 0x27];
const MACRO_FORBIDDEN_ACTION_SYSMSG: u32 = 1257;

const MACRO_ACTION_COMMANDS: &[(u32, i32)] = &[
    (0x39, 0x00), // sitstand               -> "sitstand"
    (0x3a, 0x01), // walkrun                -> "walkrun"
    (0x3c, 0x26), // mountdismount          -> "mountdismount"
    (0x3f, 0x0f), // pethold                -> "pethold"
    (0x40, 0x10), // petattack              -> "petattack"
    (0x41, 0x11), // petstop                -> "petstop"
    (0x43, 0x13), // petrevert              -> "petrevert"
    (0x45, 0x15), // summonhold             -> "summonhold"
    (0x46, 0x16), // summonattack           -> "summonattack"
    (0x47, 0x17), // summonstop             -> "summonstop"
    (0x4b, 0x28), // evaluate               -> "commend" (Recommend)
    (0x69, 0x37), // startend_videorecording-> "recstartstop"
    (0x6a, 0x36), // petmove                -> "petmove"
    (0x6b, 0x35), // servitormove           -> "summonmove"
    (0x6c, 0x34), // unsummon               -> "unsummon"
];

/// Comandos de texto de macro que no original são `FUN_10068550(<id>)` — a ação social com esse
/// id. Os ids saem direto do dispatcher (`FUN_10058330`), não de tabela nenhuma.
const MACRO_SOCIAL_COMMANDS: &[(u32, i32)] = &[
    (0x15, 2),  // socialhello
    (0x16, 3),  // socialvictory
    (0x17, 4),  // socialcharge
    (0x2e, 5),  // socialno
    (0x2f, 6),  // socialyes
    (0x30, 7),  // socialbow
    (0x31, 8),  // socialunaware
    (0x32, 9),  // socialwaiting
    (0x33, 10), // sociallaugh
    (0x36, 11), // socialapplause
    (0x37, 12), // socialdance
    (0x38, 13), // socialsad
];

pub const DEFAULT_LINE_HEIGHT: i32 = 13;
pub const INVITE_NEEDS_TARGET_SYSMSG: u32 = 186;
pub const TARGET_NOT_FOUND_SYSMSG: u32 = 50;
pub const PLEDGE_PENALTY_USER_COMMAND: i32 = 100;
/// Mensagens do `/allyinvite` no original (`FUN_1007d430(0x1fe/0x1ff, …)` em `FUN_10058330`):
/// a primeira confirma o convite enviado, a segunda reclama que não há alvo válido.
pub const ALLY_INVITE_SENT_SYSMSG: u32 = 0x1fe;
pub const ALLY_INVITE_NEEDS_TARGET_SYSMSG: u32 = 0x1ff;
/// Perguntas dos diálogos de `/allyleave` e `/allydissolve` (`FUN_10007b70(0x200)` e `(0x202)`).
pub const ALLY_LEAVE_SYSMSG: u32 = 0x200;
pub const ALLY_DISSOLVE_SYSMSG: u32 = 0x202;
pub const ALLY_DISMISS_SYSMSG: u32 = 0x201;
pub const ALLY_DISMISS_CONFIRM_SYSMSG: u32 = 0x4e3;
/// Confirmações de `clanwarstart` e `clanwarstop` (`0x5fb` e `0x5fc`, casos `0x56` e `0x57`).
pub const PLEDGE_WAR_START_SYSMSG: u32 = 0x5fb;
pub const PLEDGE_WAR_STOP_SYSMSG: u32 = 0x5fc;
/// Mensagem dourada quando `channelinvite`/`channelkick` não têm nome nem alvo (`0x5fa`).
pub const CHANNEL_NEEDS_TARGET_SYSMSG: u32 = 0x5fa;
pub const SELECT_MEMBER_SYSMSG: u32 = 259;
pub const LEADER_CANNOT_WITHDRAW_SYSMSG: u32 = 239;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragSource {
    Grid { object_id: i32, count: i32 },
    Scroll(ScrollHit),
    Paperdoll { object_id: i32, body_part: i32 },
    Skill { id: i32, level: i32 },
    Action { id: i32 },
    SysMenu(SysMenuButton),
    Tab(MainTab),
    Pledge(PledgeButton),
    Minimap(MinimapButton),
    Recipe { id: i32 },
    PetItem { object_id: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Press {
    window: WindowId,
    source: Option<DragSource>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Cooldown {
    level: i32,
    started: Instant,
    reuse: f32,
    remaining: f32,
    flash: Option<Instant>,
    stopped: bool,
}

fn in_item_table(tables: Option<&DatTables>, item_id: i32) -> bool {
    tables.is_none_or(|tables| u32::try_from(item_id).is_ok_and(|id| tables.item_icons.get(id).is_some()))
}

pub struct KeyWindows {
    pub shop: crate::ui_shop::ShopWindow,
    pub private_shop: crate::ui_shop::ShopWindow,
    pub shop_preview: crate::ui_shop::ShopWindow,
    pub warehouse: crate::ui_shop::ShopWindow,
    pub gm_store: crate::ui_shop::ShopWindow,
    pub manor_shop: crate::ui_shop::ShopWindow,
    pub delivery: crate::ui_shop::ShopWindow,
    pub select_delivery: killer_ui::delivery::SelectDeliveryView,
    pub trade: crate::ui_trade::TradeWindow,
    pub multisell: crate::ui_multisell::MultiSellWindow,
    pub manor_info: killer_ui::manor_info::ManorInfoView,
    pub manor_setting: crate::ui_manor_setting::ManorSettingWindow,
    pub hero_tower: crate::ui_hero_tower::HeroTowerWindow,
    pub empower: killer_ui::empower::EmpowerView,
    pub quest_info: killer_ui::quest_info::QuestInfoView,
    pub calc: killer_ui::calc_wnd::CalcView,
    pub observer: killer_ui::observer::ObserverView,
    pub gm: killer_ui::gm_wnd::GmView,
    pub gm_event: killer_ui::gm_event::GmEventView,
    pub gm_fence: killer_ui::gm_fence::GmFenceView,
    pub snoop: killer_ui::gm_snoop::SnoopManager,
    pub debug: killer_ui::debug_wnd::DebugWnd,
    pub gm_inventory: killer_ui::gm_inventory::GmInventoryView,
    pub gm_status: killer_ui::gm_status::GmStatusView,
    pub gm_skills: killer_ui::gm_skills::GmSkillsView,
    pub gm_quest: killer_ui::gm_quest::GmQuestView,
    pub gm_clan: killer_ui::gm_clan::GmClanView,
    pub broadcast: crate::ui_broadcast::BroadcastWindows,
    pub messenger: crate::ui_messenger::MessengerWindows,
    gm_searches: Vec<(killer_ui::gm_wnd::SearchKind, String)>,
    pending_gm_event: Vec<killer_ui::gm_event::GmEventAction>,
    pub olympiad: crate::ui_olympiad::OlympiadWindows,
    calc_tick: Option<killer_platform::time::Instant>,
    broadcast_tick: Option<killer_platform::time::Instant>,
    messenger_tick: Option<killer_platform::time::Instant>,
    pub petition: crate::ui_petition::PetitionWindows,
    pub train: crate::ui_train::TrainWindow,
    pub henna: crate::ui_henna::HennaWindows,
    pub recipes: crate::ui_recipe::RecipeWindows,
    pub pet: crate::ui_pet::PetWindow,
    pub special_maps: crate::ui_special_maps::SpecialMaps,
    pub macros: crate::ui_macros::MacroWindow,
    pub party_room_making: MakingView,
    pub party_room: PartyRoomView,
    pub party_match: killer_ui::party_match::MatchView,
    pub minimized: crate::ui_minimized::MinimizedWindows,
    party_match_title: String,
    force_peti_text: String,
    obscene: Vec<String>,
    pub command_channel: CommandChannelView,
    pub siege: crate::ui_siege::SiegeWindow,
    pub ssq: killer_ui::ssq::SsqView,
    pub command_channel_tick: Option<Instant>,
    pub main: MainWndView,
    pub inventory: InventoryView,
    pub sysmenu: SysMenuView,
    pub skills: SkillsView,
    pub actions: ActionsView,
    pub pledge: PledgeView,
    pledge_show_pending: bool,
    pledge_list_reset: u32,
    pub quest: QuestView,
    quest_painted: std::cell::Cell<bool>,
    quest_dirty: bool,
    pub target_player: Option<String>,
    pub community_request: bool,
    pub minimap: MinimapView,
    pub line_height: i32,
    pub hidden: bool,
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub enter_chatting: bool,
    pub sysstrings: HashMap<u32, String>,
    pub grade_textures: Vec<Option<String>>,
    pub fishing: killer_ui::fishing::FishPaint,
    pub skillname: HashMap<(u32, u32), l2_dat::skillname::SkillName>,
    pub skill_tips: HashMap<(u32, u32), l2_dat::skillgrp::SkillTip>,
    pub commandname: HashMap<String, u32>,
    action_social: HashMap<i32, i32>,
    action_commands: HashMap<i32, String>,
    pub party_looting: i32,
    /// Duração de cada animação social do personagem (índice = id social 0..13), usada como
    /// atraso entre comandos de macro. Preenchida por `App` a partir do `AnimSet` do jogador
    /// (`social_anim_duration`, o mesmo cálculo de `APawn::GetSocialAnimDuration`); enquanto
    /// vazia, as ações sociais avançam sem espera.
    pub social_durations: Vec<f32>,
    macro_run: Option<MacroRun>,
    pub abnormal: AbnormalView,
    pub enchant: killer_ui::enchant_wnd::EnchantView,
    pub option: killer_ui::option::OptionView,
    pub option_pressed: Option<killer_ui::option::OptionHit>,
    pub option_applying: bool,
    pending: Vec<UiCommand>,
    pub dialogs: Vec<Dialog>,
    pub tooltip: Option<Tooltip>,
    enchant_refill: bool,
    recipe_double_clicked: bool,
    drag_moved: bool,
    press: Option<Press>,
    grid_double_clicked: bool,
    minimap_recenter: bool,
    pub zones: killer_ui::zone_title::ZoneResolver,
    party_mark: usize,
    cooldowns: HashMap<i32, Cooldown>,
    bar_ticked: Option<Instant>,
    item_order: Vec<i32>,
    skills_visible: bool,
    bar_visible: bool,
    inventory_shown: bool,
    character_name: String,
    auto_potion_at: [Option<Instant>; 3],
    cooldown_seen: Vec<l2_net::game::SkillReuse>,
    inventory_seen: Option<(Vec<l2_net::game::packets::InventoryItem>, usize)>,
    removed_items: std::collections::HashSet<i32>,
    macro_list_receiving: bool,
    macro_list_received: i32,
    skills_seen: Option<(Vec<l2_net::game::packets::SkillEntry>, usize)>,
}

impl Default for KeyWindows {
    fn default() -> Self {
        KeyWindows::new(&HashMap::new(), false)
    }
}

impl KeyWindows {
    pub fn new(sysstrings: &HashMap<u32, String>, enter_chatting: bool) -> KeyWindows {
        let lookup = |id: u32| sysstrings.get(&id).cloned();
        let mut warehouse = crate::ui_shop::ShopWindow::new(lookup);
        warehouse.id = WindowId::WAREHOUSE;
        let mut manor_shop = crate::ui_shop::ShopWindow::new(lookup);
        manor_shop.id = WindowId::MANOR_SHOP;
        let mut gm_store = crate::ui_shop::ShopWindow::new(lookup);
        gm_store.id = WindowId::GM_STORE;
        gm_store.view.gm = true;
        KeyWindows {
            warehouse,
            gm_store,
            manor_shop,
            delivery: {
                let mut shop = crate::ui_shop::ShopWindow::new(lookup);
                shop.id = WindowId::DELIVERY;
                shop
            },
            select_delivery: killer_ui::delivery::SelectDeliveryView::new(lookup),
            trade: crate::ui_trade::TradeWindow::new(lookup),
            multisell: crate::ui_multisell::MultiSellWindow::new(lookup),
            manor_info: killer_ui::manor_info::ManorInfoView::new(lookup),
            manor_setting: crate::ui_manor_setting::ManorSettingWindow::new(lookup),
            hero_tower: crate::ui_hero_tower::HeroTowerWindow::new(lookup),
            empower: killer_ui::empower::EmpowerView::new(lookup),
            quest_info: killer_ui::quest_info::QuestInfoView::new(lookup),
            calc: killer_ui::calc_wnd::CalcView::new(lookup),
            observer: killer_ui::observer::ObserverView::new(lookup),
            gm: killer_ui::gm_wnd::GmView::new(lookup),
            gm_event: killer_ui::gm_event::GmEventView::new(lookup),
            gm_fence: killer_ui::gm_fence::GmFenceView::new(lookup),
            snoop: killer_ui::gm_snoop::SnoopManager::new(lookup),
            debug: killer_ui::debug_wnd::DebugWnd::new(0, Vec::new(), false),
            gm_inventory: killer_ui::gm_inventory::GmInventoryView::new(lookup),
            gm_status: killer_ui::gm_status::GmStatusView::new(lookup),
            gm_skills: killer_ui::gm_skills::GmSkillsView::new(lookup),
            gm_quest: killer_ui::gm_quest::GmQuestView::new(lookup),
            gm_clan: killer_ui::gm_clan::GmClanView::new(lookup),
            broadcast: crate::ui_broadcast::BroadcastWindows::new((1024, 768)),
            messenger: crate::ui_messenger::MessengerWindows::new(sysstrings),
            gm_searches: Vec::new(),
            pending_gm_event: Vec::new(),
            olympiad: crate::ui_olympiad::OlympiadWindows::new(lookup),
            calc_tick: None,
            broadcast_tick: None,
            messenger_tick: None,
            petition: crate::ui_petition::PetitionWindows::new(lookup, 0, Vec::new()),
            train: crate::ui_train::TrainWindow::new(lookup),
            henna: crate::ui_henna::HennaWindows::new(lookup),
            recipes: crate::ui_recipe::RecipeWindows::new(lookup),
            pet: crate::ui_pet::PetWindow::new(lookup),
            special_maps: crate::ui_special_maps::SpecialMaps::new(lookup),
            macros: crate::ui_macros::MacroWindow::new(lookup),
            party_room_making: MakingView::new([String::new(), String::new(), String::new(), String::new(), String::new()], lookup(140).unwrap_or_default().as_str(), lookup(141).unwrap_or_default().as_str()),
            party_room: PartyRoomView::new(RoomLabels::from_sysstrings(lookup)),
            command_channel: CommandChannelView::new(ChannelLabels::from_sysstrings(lookup)),
            minimized: Default::default(),
            party_match: killer_ui::party_match::MatchView::new(killer_ui::party_match::Labels::from_sysstrings(lookup), DEFAULT_LINE_HEIGHT),
            party_match_title: String::new(),
            force_peti_text: String::new(),
            obscene: Vec::new(),
            siege: crate::ui_siege::SiegeWindow::new(lookup),
            ssq: killer_ui::ssq::SsqView::new(killer_ui::ssq::Labels::from_sysstrings(lookup)),
            command_channel_tick: None,
            shop: crate::ui_shop::ShopWindow::new(lookup),
            private_shop: {
                let mut shop = crate::ui_shop::ShopWindow::new(lookup);
                shop.id = WindowId::PRIVATE_SHOP;
                shop
            },
            shop_preview: {
                let mut shop = crate::ui_shop::ShopWindow::new(lookup);
                shop.id = WindowId::SHOP_PREVIEW;
                shop
            },
            main: MainWndView::with_sysstrings(lookup),
            inventory: InventoryView::with_sysstrings(lookup),
            sysmenu: SysMenuView::with_sysstrings(lookup),
            skills: SkillsView::with_sysstrings(lookup),
            actions: ActionsView::with_sysstrings(lookup),
            pledge: PledgeView::with_sysstrings(lookup),
            pledge_show_pending: false,
            pledge_list_reset: 0,
            quest: QuestView::with_sysstrings(lookup),
            quest_painted: std::cell::Cell::new(false),
            quest_dirty: true,
            target_player: None,
            community_request: false,
            minimap: MinimapView::with_sysstrings(lookup),
            line_height: DEFAULT_LINE_HEIGHT,
            hidden: false,
            alt: false,
            ctrl: false,
            shift: false,
            enter_chatting,
            sysstrings: sysstrings.clone(),
            grade_textures: Vec::new(),
            fishing: killer_ui::fishing::FishPaint::default(),
            skillname: HashMap::new(),
            skill_tips: HashMap::new(),
            commandname: HashMap::new(),
            action_social: HashMap::new(),
            action_commands: HashMap::new(),
            party_looting: 0,
            social_durations: Vec::new(),
            macro_run: None,
            abnormal: AbnormalView::default(),
            enchant: killer_ui::enchant_wnd::EnchantView::new(lookup),
            option: killer_ui::option::OptionView::new(lookup),
            option_pressed: None,
            option_applying: false,
            pending: Vec::new(),
            dialogs: Vec::new(),
            tooltip: None,
            enchant_refill: false,
            recipe_double_clicked: false,
            drag_moved: false,
            press: None,
            grid_double_clicked: false,
            minimap_recenter: false,
            zones: Default::default(),
            party_mark: 0,
            cooldowns: HashMap::new(),
            bar_ticked: None,
            item_order: Vec::new(),
            skills_visible: false,
            bar_visible: false,
            inventory_shown: false,
            character_name: String::new(),
            auto_potion_at: [None; 3],
            cooldown_seen: Vec::new(),
            inventory_seen: None,
            removed_items: std::collections::HashSet::new(),
            macro_list_receiving: true,
            macro_list_received: 0,
            skills_seen: None,
        }
    }

    pub fn set_sysstrings(&mut self, tables: &DatTables) {
        self.obscene = tables.obscene.clone();
        self.siege.set_tables(tables);
        self.party_match = killer_ui::party_match::MatchView::new(killer_ui::party_match::Labels::from_sysstrings(|id| tables.sysstring.get(&id).cloned()), self.line_height);
        self.party_room_making.members.text_height = self.line_height;
        let categories = killer_ui::debug_wnd::load_categories(&killer_ui::ini::parse_ini(&tables.debug_ini));
        self.debug = killer_ui::debug_wnd::DebugWnd::new(self.debug.list.rect.w, categories, tables.login_ini.debug_log);
        self.force_peti_text = tables.systemmsg.get(&FORCE_PETI_SYSMSG).map(|message| message.text.clone()).unwrap_or_default();
        self.party_match_title = tables.systemmsg.get(&1398).map(|message| message.text.clone()).unwrap_or_default();
        self.command_channel.too_long = tables.systemmsg.get(&command_channel::TOO_LONG_SYSMSG).map(|message| message.text.clone()).unwrap_or_default();
        self.ssq = killer_ui::ssq::SsqView::new(crate::ui_ssq::labels(tables));
        self.sysstrings = tables.sysstring.clone();
        self.skillname = tables.skillname.clone();
        self.grade_textures = (0..=5)
            .map(|grade| killer_ui::item_name::grade_symbol_1006b5a0(grade).and_then(|alias| tables.symbolname.by_alias(alias)).map(|symbol| symbol.texture.clone()))
            .collect();
        self.skill_tips = tables.skillgrp.iter().map(|(key, skill)| (*key, skill.tip())).collect();
        self.commandname = tables.commandname.clone();
        let lookup = |id: u32| tables.sysstring.get(&id).cloned();
        self.shop.view.strings = killer_ui::shop::ShopStrings::with_sysstrings(lookup);
        self.private_shop.view.strings = killer_ui::shop::ShopStrings::with_sysstrings(lookup);
        self.shop_preview.view.strings = killer_ui::shop::ShopStrings::with_sysstrings(lookup);
        self.warehouse.view.strings = killer_ui::shop::ShopStrings::with_sysstrings(lookup);
        self.gm_store.view.strings = killer_ui::shop::ShopStrings::with_sysstrings(lookup);
        self.manor_shop.view.strings = killer_ui::shop::ShopStrings::with_sysstrings(lookup);
        self.delivery.view.strings = killer_ui::shop::ShopStrings::with_sysstrings(lookup);
        self.select_delivery.title = lookup(558).unwrap_or_default();
        self.select_delivery.ok = lookup(140).unwrap_or_default();
        self.select_delivery.cancel = lookup(141).unwrap_or_default();
        self.select_delivery.message = crate::ui2d::system_text(Some(tables), killer_ui::delivery::SELECT_MESSAGE, &[]);
        self.trade.view.title = lookup(killer_ui::trade::TITLE_SYSSTRING).unwrap_or_default();
        self.trade.view.ok_label = lookup(killer_ui::trade::OK_SYSSTRING).unwrap_or_default();
        self.trade.view.cancel_label = lookup(killer_ui::trade::CANCEL_SYSSTRING).unwrap_or_default();
        self.multisell.view = killer_ui::multisell::MultiSellView::new(lookup);
        self.manor_info = killer_ui::manor_info::ManorInfoView::new(lookup);
        self.manor_setting = crate::ui_manor_setting::ManorSettingWindow::new(lookup);
        self.hero_tower = crate::ui_hero_tower::HeroTowerWindow::new(lookup);
        self.empower = killer_ui::empower::EmpowerView::new(lookup);
        self.quest_info = killer_ui::quest_info::QuestInfoView::new(lookup);
        self.calc = killer_ui::calc_wnd::CalcView::new(lookup);
        self.observer = killer_ui::observer::ObserverView::new(lookup);
        self.gm = killer_ui::gm_wnd::GmView::new(lookup);
        self.gm_event = killer_ui::gm_event::GmEventView::new(lookup);
        self.gm_fence = killer_ui::gm_fence::GmFenceView::new(lookup);
        self.snoop = killer_ui::gm_snoop::SnoopManager::new(lookup);
        self.gm_inventory = killer_ui::gm_inventory::GmInventoryView::new(lookup);
        self.gm_status = killer_ui::gm_status::GmStatusView::new(lookup);
        self.gm_skills = killer_ui::gm_skills::GmSkillsView::new(lookup);
        self.gm_quest = killer_ui::gm_quest::GmQuestView::new(lookup);
        self.messenger.relabel(lookup);
        self.messenger.set_texts(tables);
        let previous = std::mem::replace(&mut self.gm_clan, killer_ui::gm_clan::GmClanView::new(lookup));
        self.gm_clan.pledge.name = previous.pledge.name;
        self.gm_clan.pledge.leader = previous.pledge.leader;
        self.gm_clan.pledge.level = previous.pledge.level;
        self.gm_clan.pledge.gm_castle = previous.pledge.gm_castle;
        self.gm_clan.pledge.gm_hideout = previous.pledge.gm_hideout;
        self.gm_clan.pledge.members = previous.pledge.members;
        self.gm_clan.clan_id = previous.clan_id;
        self.olympiad = crate::ui_olympiad::OlympiadWindows::new(lookup);
        let categories = self.petition.categories().to_vec();
        self.petition = crate::ui_petition::PetitionWindows::new(lookup, self.petition.line_height(), categories);
        self.manor_info.bottom_template = crate::ui2d::system_text(Some(tables), killer_ui::manor_info::BOTTOM_MESSAGE, &[]);
        self.train.view = killer_ui::train::TrainView::new(lookup);
        self.option = killer_ui::option::OptionView::new(lookup);
        self.henna = crate::ui_henna::HennaWindows::new(lookup);
        self.recipes = crate::ui_recipe::RecipeWindows::new(lookup);
        self.pet = crate::ui_pet::PetWindow::new(lookup);
        self.special_maps.town.frame.title = lookup(447).unwrap_or_default();
        self.special_maps.seal.frame.title = lookup(2).unwrap_or_default();
        self.macros.set_sysstrings(lookup);
        self.party_room_making.ok.label = lookup(140).unwrap_or_default();
        self.party_room_making.cancel.label = lookup(141).unwrap_or_default();
        self.command_channel.set_labels(ChannelLabels::from_sysstrings(lookup));
        self.party_room.set_labels(RoomLabels::from_sysstrings(lookup));
        let (main_tab, inventory_tab, skill_tab) = (self.main.tab, self.inventory.tab, self.skills.tab);
        self.main = MainWndView::with_sysstrings(lookup);
        self.main.tab = main_tab;
        let fresh = InventoryView::with_sysstrings(lookup);
        self.inventory.title = fresh.title;
        self.inventory.tab_names = fresh.tab_names;
        self.inventory.tab = inventory_tab;
        self.sysmenu = SysMenuView::with_sysstrings(lookup);
        self.skills.tab_names = SkillsView::with_sysstrings(lookup).tab_names;
        self.skills.tab = skill_tab;
        self.actions.headers = ActionsView::with_sysstrings(lookup).headers;
        self.pledge.texts = PledgeView::with_sysstrings(lookup).texts;
        let fresh = QuestView::with_sysstrings(lookup);
        self.quest.accepted_label = fresh.accepted_label;
        self.quest.location_label = fresh.location_label;
        self.quest.abort_label = fresh.abort_label;
        self.quest.labels = fresh.labels;
        let fresh = MinimapView::with_sysstrings(lookup);
        self.minimap.title = fresh.title;
        self.minimap.labels = fresh.labels;
        self.minimap.position_label = fresh.position_label;
        self.minimap.quest_status_label = fresh.quest_status_label;
        self.minimap.period_labels = fresh.period_labels;
        self.relocalize_actions(tables);
    }

    pub fn relocalize_actions(&mut self, tables: &DatTables) {
        self.action_social = tables.actions.iter().map(|a| (a.id as i32, a.kind)).collect();
        self.action_commands = tables.actions.iter().map(|a| (a.id as i32, a.cmd.clone())).collect();
        self.actions.actions = tables
            .actions
            .iter()
            .map(|a| ActionEntry { id: a.id as i32, category: a.category, icon: a.icon.clone(), name: a.name.clone(), desc: a.desc.clone(), ride: a.category2.first() == Some(&killer_ui::actions::CATEGORY2_RIDE) })
            .collect();
    }

    pub fn install(&self, desktop: &mut Desktop, windows_info: &killer_ui::ini::WindowsInfo, viewport: (i32, i32)) {
        let (mw, mh) = mainwnd::MAIN_SIZE;
        let (mx, my) = windows_info.position(WindowId::MAIN).unwrap_or_else(|| mainwnd::default_position(viewport));
        let mut main = Window::new(WindowId::MAIN, Rect::new(mx, my, mw, mh), mainwnd::grip())
            .with_initial_position(mainwnd::initial_position(viewport));
        main.visible = false;
        desktop.windows.push(main);
        let (iw, ih) = inventory::INVENTORY_SIZE;
        let (ix, iy) =
            windows_info.position(WindowId::INVENTORY).unwrap_or_else(|| inventory::default_position(viewport));
        let mut inv = Window::new(WindowId::INVENTORY, Rect::new(ix, iy, iw, ih), inventory::grip())
            .with_initial_position(inventory::initial_position(viewport));
        inv.visible = false;
        desktop.windows.push(inv);
        let (sw, sh) = sysmenu::SYSMENU_SIZE;
        let (sx, sy) = sysmenu::default_position(viewport);
        let mut menu = Window::new(SYSMENU_WINDOW, Rect::new(sx, sy, sw, sh), sysmenu::grip())
            .with_initial_position(sysmenu::initial_position(viewport));
        menu.visible = false;
        desktop.windows.push(menu);
        crate::ui_special_maps::SpecialMaps::create_windows(desktop, windows_info.position(WindowId::TOWN_MAP), windows_info.position(WindowId::XMAS_SEAL));
        let (mw, mh) = minimap::MINIMAP_SIZE;
        let (mx, my) = windows_info.position(WindowId::MAP_COVER).unwrap_or_else(|| minimap::default_position(viewport));
        let mut map = Window::new(WindowId::MAP_COVER, Rect::new(mx, my, mw, mh), minimap::grip())
            .with_initial_position(minimap::initial_position(viewport));
        map.visible = false;
        desktop.windows.push(map);
        let (x, y) = windows_info.position(WindowId::SHOP).unwrap_or_else(|| killer_ui::shop::default_position(viewport));
        let mut shop = Window::new(WindowId::SHOP, Rect::new(x, y, 256, 401), killer_ui::shop::grip())
            .with_initial_position(killer_ui::shop::initial_position(viewport));
        shop.visible = false;
        desktop.windows.push(shop);
        let (x, y) = windows_info.position(WindowId::PRIVATE_SHOP).unwrap_or_else(|| killer_ui::shop::private_default_position(viewport));
        let mut private_shop = Window::new(WindowId::PRIVATE_SHOP, Rect::new(x, y, 256, 409), killer_ui::shop::grip())
            .with_initial_position(killer_ui::shop::private_initial_position(viewport));
        private_shop.visible = false;
        desktop.windows.push(private_shop);
        let (x, y) = windows_info.position(WindowId::SHOP_PREVIEW).unwrap_or_else(|| killer_ui::shop::default_position(viewport));
        let mut shop_preview = Window::new(WindowId::SHOP_PREVIEW, Rect::new(x, y, 256, 401), killer_ui::shop::grip())
            .with_initial_position(killer_ui::shop::initial_position(viewport));
        shop_preview.visible = false;
        desktop.windows.push(shop_preview);
        for (id, height) in [(WindowId::DELIVERY, 401), (WindowId::SELECT_DELIVERY, 201)] {
            let (x, y) = windows_info.position(id).unwrap_or_else(|| killer_ui::shop::default_position(viewport));
            let mut window = Window::new(id, Rect::new(x, y, 256, height), killer_ui::shop::grip())
                .with_initial_position(killer_ui::shop::initial_position(viewport));
            window.visible = false;
            desktop.windows.push(window);
        }
        let (x, y) = windows_info.position(WindowId::MANOR_SHOP).unwrap_or_else(|| killer_ui::shop::default_position(viewport));
        let mut manor_shop = Window::new(WindowId::MANOR_SHOP, Rect::new(x, y, 256, 401), killer_ui::shop::grip())
            .with_initial_position(killer_ui::shop::initial_position(viewport));
        manor_shop.visible = false;
        desktop.windows.push(manor_shop);
        let (x, y) = windows_info.position(WindowId::WAREHOUSE).unwrap_or(killer_ui::shop::WAREHOUSE_DEFAULT_POS);
        let mut warehouse = Window::new(WindowId::WAREHOUSE, Rect::new(x, y, 256, 401), killer_ui::shop::grip())
            .with_initial_position(killer_ui::shop::WAREHOUSE_INITIAL_POSITION);
        warehouse.visible = false;
        desktop.windows.push(warehouse);
        let (x, y) = windows_info.position(WindowId::GM_STORE).unwrap_or(((viewport.0 as f32 - 209.0) as i32, 100));
        let mut gm_store = Window::new(WindowId::GM_STORE, Rect::new(x, y, 256, 401), killer_ui::shop::grip())
            .with_initial_position(killer_ui::initial_position::InitialPosition { enabled: true, horizontal_mode: 1, vertical_mode: 3, x: 209, y: 100 });
        gm_store.visible = false;
        desktop.windows.push(gm_store);
        let (x, y) = windows_info.position(WindowId::TRADE).unwrap_or_else(|| killer_ui::trade::default_position(viewport));
        let mut trade = Window::new(WindowId::TRADE, Rect::new(x, y, 256, 401), killer_ui::trade::grip())
            .with_initial_position(killer_ui::trade::initial_position(viewport));
        trade.visible = false;
        desktop.windows.push(trade);
        let (x, y) = windows_info.position(WindowId::MULTISELL).unwrap_or_else(|| killer_ui::multisell::default_position(viewport));
        let mut multisell = Window::new(WindowId::MULTISELL, Rect::new(x, y, killer_ui::multisell::SIZE.0, killer_ui::multisell::SIZE.1), killer_ui::multisell::grip())
            .with_initial_position(killer_ui::multisell::initial_position(viewport));
        multisell.visible = false;
        desktop.windows.push(multisell);
        let (x, y) = windows_info.position(WindowId::TRAIN).unwrap_or_else(|| killer_ui::train::default_position(viewport));
        let mut train = Window::new(WindowId::TRAIN, Rect::new(x, y, killer_ui::train::SIZE.0, killer_ui::train::SIZE.1), killer_ui::train::grip())
            .with_initial_position(killer_ui::train::initial_position(viewport));
        train.visible = false;
        desktop.windows.push(train);
        let (x, y) = windows_info.position(WindowId::ITEM_ENCHANT).unwrap_or_else(|| killer_ui::enchant_wnd::default_position(viewport));
        let mut enchant = Window::new(
            WindowId::ITEM_ENCHANT,
            Rect::new(x, y, killer_ui::enchant_wnd::SIZE.0, killer_ui::enchant_wnd::SIZE.1),
            killer_ui::enchant_wnd::grip(),
        ).with_initial_position(killer_ui::enchant_wnd::initial_position(viewport));
        enchant.visible = false;
        desktop.windows.push(enchant);
        let (ow, oh) = killer_ui::option::SIZE;
        let (ox, oy) = killer_ui::option::creation_position(viewport);
        let mut option = Window::new(
            WindowId::OPTION,
            Rect::new(ox, oy, ow, oh),
            killer_ui::option::grip(),
        ).with_initial_position(killer_ui::option::INITIAL_POSITION);
        option.visible = false;
        desktop.windows.push(option);
        for id in [WindowId::HENNA_LIST, WindowId::HENNA_INFO] {
            let (x, y) = windows_info.position(id).unwrap_or_else(|| killer_ui::henna::default_position(viewport));
            let size = killer_ui::henna::SIZE;
            let mut window = Window::new(id, Rect::new(x, y, size.0, size.1), killer_ui::henna::grip())
                .with_initial_position(killer_ui::henna::initial_position(viewport));
            window.visible = false;
            desktop.windows.push(window);
        }
        crate::ui_pet::create_window(desktop, windows_info.position(WindowId::PET_WINDOW));
        crate::ui_pet::create_summoned_window(desktop, windows_info.position(WindowId::SUMMONED_WINDOW));
        for id in [WindowId::RECIPE_BOOK, WindowId::RECIPE_MANUFACTURE, WindowId::RECIPE_TREE] {
            let (x, y) = windows_info.position(id).unwrap_or_else(|| killer_ui::recipe::book_default_position(viewport));
            let mut window = Window::new(id, Rect::new(x, y, killer_ui::recipe::SIZE.0, killer_ui::recipe::SIZE.1), killer_ui::recipe::grip())
                .with_initial_position(killer_ui::recipe::book_initial_position(viewport));
            window.visible = false;
            desktop.windows.push(window);
        }
        for id in [WindowId::RECIPE_SHOP, WindowId::RECIPE_SELL, WindowId::RECIPE_BUY] {
            let (x, y) = windows_info.position(id).unwrap_or_else(|| {
                let (x, y) = killer_ui::recipe::book_default_position(viewport);
                (x + 256, y)
            });
            let mut initial = killer_ui::recipe::book_initial_position(viewport);
            initial.x -= 256;
            let mut window = Window::new(id, Rect::new(x, y, 256, 401), killer_ui::recipe_shop::grip()).with_initial_position(initial);
            window.visible = false;
            desktop.windows.push(window);
        }


        let (x, y) = windows_info.position(WindowId::MACRO_LIST).unwrap_or_else(|| killer_ui::macros::list_default_position(viewport));
        let mut macro_list = Window::new(WindowId::MACRO_LIST, Rect::new(x, y, killer_ui::macros::LIST_SIZE.0, killer_ui::macros::LIST_SIZE.1), killer_ui::macros::list_grip())
            .with_initial_position(killer_ui::macros::list_initial_position(viewport));
        macro_list.visible = false;
        desktop.windows.push(macro_list);
        let (x, y) = windows_info.position(WindowId::MACRO_INFO).unwrap_or_else(|| killer_ui::macros::info_default_position(viewport));
        let mut macro_info = Window::new(WindowId::MACRO_INFO, Rect::new(x, y, killer_ui::macros::INFO_SIZE.0, killer_ui::macros::INFO_SIZE.1), killer_ui::macros::info_grip())
            .with_initial_position(killer_ui::macros::info_initial_position(viewport));
        macro_info.visible = false;
        desktop.windows.push(macro_info);
        let (x, y) = windows_info.position(WindowId::MACRO_EDIT).unwrap_or_else(|| killer_ui::macros::edit_default_position(viewport));
        let mut macro_edit = Window::new(WindowId::MACRO_EDIT, Rect::new(x, y, killer_ui::macros::EDIT_SIZE.0, killer_ui::macros::EDIT_SIZE.1), killer_ui::macros::edit_grip())
            .with_initial_position(killer_ui::macros::edit_initial_position(viewport));
        macro_edit.visible = false;
        desktop.windows.push(macro_edit);
        let (x, y) = windows_info.position(WindowId::PARTY_ROOM_MAKING).unwrap_or_else(|| party_room::making_default_position(viewport));
        let mut party_room_making = Window::new(WindowId::PARTY_ROOM_MAKING, Rect::new(x, y, party_room::MAKING_SIZE.0, party_room::MAKING_SIZE.1), party_room::grip())
            .with_initial_position(party_room::MAKING_INITIAL_POSITION);
        party_room_making.visible = false;
        desktop.windows.push(party_room_making);
        let (x, y) = windows_info.position(WindowId::PARTY_ROOM).unwrap_or(party_room::ROOM_POS);
        let mut party_room = Window::new(WindowId::PARTY_ROOM, Rect::new(x, y, party_room::ROOM_SIZE.0, party_room::ROOM_SIZE.1), party_room::room_grip())
            .with_initial_position(party_room::ROOM_INITIAL_POSITION);
        party_room.visible = false;
        desktop.windows.push(party_room);
        let (x, y) = windows_info.position(WindowId::PARTY_MATCH).unwrap_or((0, 76));
        let mut party_match = Window::new(WindowId::PARTY_MATCH, Rect::new(x, y, killer_ui::party_match::SIZE.0, killer_ui::party_match::SIZE.1), killer_ui::party_match::grip())
            .with_initial_position(killer_ui::party_match::INITIAL_POSITION);
        party_match.visible = false;
        desktop.windows.push(party_match);
        let (x, y) = windows_info.position(command_channel::WINDOW_ID).unwrap_or(command_channel::DEFAULT_POSITION);
        let mut channel = Window::new(command_channel::WINDOW_ID, Rect::new(x, y, command_channel::SIZE.0, command_channel::SIZE.1), CommandChannelView::grip())
            .with_initial_position(command_channel::INITIAL_POSITION);
        channel.visible = false;
        desktop.windows.push(channel);
        let id = crate::ui_siege::WINDOW_ID;
        let (x, y) = windows_info.position(id).unwrap_or_else(|| killer_ui::siege::default_position(viewport.1));
        let mut siege = Window::new(id, Rect::new(x, y, killer_ui::siege::SIZE.0, killer_ui::siege::SIZE.1), Rect::new(0, 0, 256, 20))
            .with_initial_position(killer_ui::siege::initial_position(viewport));
        siege.visible = false;
        desktop.windows.push(siege);
        let (x, y) = windows_info.position(WindowId::SSQ).unwrap_or_else(|| killer_ui::ssq::default_position(viewport));
        let mut ssq = Window::new(WindowId::SSQ, Rect::new(x, y, killer_ui::ssq::SIZE.0, killer_ui::ssq::SIZE.1), killer_ui::ssq::grip())
            .with_initial_position(killer_ui::ssq::initial_position(viewport));
        ssq.visible = false;
        desktop.windows.push(ssq);
        let id = killer_ui::manor_info::WINDOW_ID;
        let (x, y) = windows_info.position(id).unwrap_or_else(|| killer_ui::manor_info::default_position(viewport));
        let mut manor = Window::new(id, Rect::new(x, y, killer_ui::manor_info::SIZE.0, killer_ui::manor_info::SIZE.1), killer_ui::manor_info::grip())
            .with_initial_position(killer_ui::manor_info::INITIAL_POSITION);
        manor.visible = false;
        desktop.windows.push(manor);
        let (x, y) = windows_info.position(WindowId::MANOR_SEED_SETTING).unwrap_or_else(|| killer_ui::manor_setting::setting_default_position(viewport));
        let mut seed_setting = Window::new(WindowId::MANOR_SEED_SETTING, Rect::new(x, y, killer_ui::manor_setting::SETTING_SIZE.0, killer_ui::manor_setting::SETTING_SIZE.1), killer_ui::manor_setting::setting_grip())
            .with_initial_position(killer_ui::manor_setting::SETTING_INITIAL_POSITION);
        seed_setting.visible = false;
        desktop.windows.push(seed_setting);
        let (x, y) = windows_info.position(WindowId::MANOR_SEED_CHANGE).unwrap_or_else(|| killer_ui::manor_setting::change_default_position(viewport));
        let mut seed_change = Window::new(WindowId::MANOR_SEED_CHANGE, Rect::new(x, y, killer_ui::manor_setting::CHANGE_SIZE.0, killer_ui::manor_setting::CHANGE_SIZE.1), killer_ui::manor_setting::change_grip())
            .with_initial_position(killer_ui::manor_setting::CHANGE_INITIAL_POSITION);
        seed_change.visible = false;
        desktop.windows.push(seed_change);
        let (x, y) = windows_info.position(WindowId::MANOR_CROP_SETTING).unwrap_or_else(|| killer_ui::manor_crop_setting::setting_default_position(viewport));
        let mut crop_setting = Window::new(WindowId::MANOR_CROP_SETTING, Rect::new(x, y, killer_ui::manor_crop_setting::SETTING_SIZE.0, killer_ui::manor_crop_setting::SETTING_SIZE.1), killer_ui::manor_crop_setting::setting_grip())
            .with_initial_position(killer_ui::manor_crop_setting::SETTING_INITIAL_POSITION);
        crop_setting.visible = false;
        desktop.windows.push(crop_setting);
        let (x, y) = windows_info.position(WindowId::MANOR_CROP_CHANGE).unwrap_or_else(|| killer_ui::manor_crop_setting::change_default_position(viewport));
        let mut crop_change = Window::new(WindowId::MANOR_CROP_CHANGE, Rect::new(x, y, killer_ui::manor_crop_setting::CHANGE_SIZE.0, killer_ui::manor_crop_setting::CHANGE_SIZE.1), killer_ui::manor_crop_setting::change_grip())
            .with_initial_position(killer_ui::manor_crop_setting::CHANGE_INITIAL_POSITION);
        crop_change.visible = false;
        desktop.windows.push(crop_change);
        let (x, y) = windows_info.position(WindowId::MANOR_CROP_SELL).unwrap_or_else(|| killer_ui::manor_crop_sell::sell_default_position(viewport));
        let mut crop_sell = Window::new(WindowId::MANOR_CROP_SELL, Rect::new(x, y, killer_ui::manor_crop_sell::SELL_SIZE.0, killer_ui::manor_crop_sell::SELL_SIZE.1), killer_ui::manor_crop_sell::sell_grip())
            .with_initial_position(killer_ui::manor_crop_sell::SELL_INITIAL_POSITION);
        crop_sell.visible = false;
        desktop.windows.push(crop_sell);
        let (x, y) = windows_info.position(WindowId::MANOR_CROP_SELL_CHANGE).unwrap_or_else(|| killer_ui::manor_crop_sell::change_default_position(viewport));
        let mut procure_change = Window::new(WindowId::MANOR_CROP_SELL_CHANGE, Rect::new(x, y, killer_ui::manor_crop_sell::CHANGE_SIZE.0, killer_ui::manor_crop_sell::CHANGE_SIZE.1), killer_ui::manor_crop_sell::change_grip())
            .with_initial_position(killer_ui::manor_crop_sell::CHANGE_INITIAL_POSITION);
        procure_change.visible = false;
        desktop.windows.push(procure_change);
        let mut fishing = Window::new(WindowId::FISHING, Rect::new(killer_ui::fishing::POSITION.0, killer_ui::fishing::POSITION.1, killer_ui::fishing::SIZE.0, killer_ui::fishing::SIZE.1), Rect::new(0, 0, killer_ui::fishing::SIZE.0, 20))
            .with_initial_position(killer_ui::initial_position::DISABLED_INITIAL_POSITION);
        fishing.visible = false;
        desktop.windows.push(fishing);
        let (x, y) = windows_info.position(WindowId::HERO_TOWER).unwrap_or_else(|| killer_ui::hero_tower::tower_default_position(viewport));
        let mut hero_tower = Window::new(WindowId::HERO_TOWER, Rect::new(x, y, killer_ui::hero_tower::TOWER_SIZE.0, killer_ui::hero_tower::TOWER_SIZE.1), killer_ui::hero_tower::tower_grip())
            .with_initial_position(killer_ui::hero_tower::TOWER_INITIAL_POSITION);
        hero_tower.visible = false;
        desktop.windows.push(hero_tower);
        let (x, y) = windows_info.position(WindowId::HERO_TOWER_CONTENT).unwrap_or_else(|| killer_ui::hero_tower::content_default_position(viewport));
        let mut hero_content = Window::new(WindowId::HERO_TOWER_CONTENT, Rect::new(x, y, killer_ui::hero_tower::CONTENT_SIZE.0, killer_ui::hero_tower::CONTENT_SIZE.1), killer_ui::hero_tower::content_grip())
            .with_initial_position(killer_ui::hero_tower::CONTENT_INITIAL_POSITION);
        hero_content.visible = false;
        desktop.windows.push(hero_content);
        let (x, y) = windows_info.position(WindowId::EMPOWER).unwrap_or_else(|| killer_ui::empower::default_position(viewport));
        let mut empower = Window::new(WindowId::EMPOWER, Rect::new(x, y, killer_ui::empower::SIZE.0, killer_ui::empower::SIZE.1), killer_ui::empower::grip())
            .with_initial_position(killer_ui::empower::initial_position(viewport));
        empower.visible = false;
        desktop.windows.push(empower);
        {
            use killer_ui::olympiad;
            let mut add = |id: WindowId, default: (i32, i32), size: (i32, i32), grip: Rect, initial: killer_ui::initial_position::InitialPosition| {
                let (x, y) = windows_info.position(id).unwrap_or(default);
                let mut window = Window::new(id, Rect::new(x, y, size.0, size.1), grip).with_initial_position(initial);
                window.visible = false;
                desktop.windows.push(window);
            };
            add(olympiad::CONTROL_ID, olympiad::control_default_position(viewport), olympiad::CONTROL_SIZE, olympiad::control_grip(), olympiad::CONTROL_INITIAL_POSITION);
            for index in 0..2 {
                add(olympiad::PLAYER_IDS[index], olympiad::player_default_position(index, viewport), olympiad::PLAYER_SIZE, olympiad::player_grip(), olympiad::player_initial_position(index));
            }
            for index in 0..2 {
                add(olympiad::ABNORMAL_IDS[index], olympiad::abnormal_default_position(index, viewport), olympiad::ABNORMAL_SIZE, Rect::new(0, 0, killer_ui::abnormal::GRIP_SIZE.0, olympiad::ABNORMAL_SIZE.1), olympiad::abnormal_initial_position(index));
            }
            add(olympiad::TARGET_ID, olympiad::target_default_position(viewport), olympiad::TARGET_SIZE, olympiad::target_grip(), olympiad::TARGET_INITIAL_POSITION);
        }
        let (x, y) = windows_info.position(WindowId::GM_EVENT).unwrap_or_else(|| killer_ui::gm_event::default_position(viewport));
        let mut gm_event = Window::new(WindowId::GM_EVENT, Rect::new(x, y, killer_ui::gm_event::SIZE.0, killer_ui::gm_event::SIZE.1), killer_ui::gm_event::grip())
            .with_initial_position(killer_ui::gm_event::INITIAL_POSITION);
        gm_event.visible = false;
        desktop.windows.push(gm_event);
        let (x, y) = windows_info.position(WindowId::GM_FENCE).unwrap_or_else(|| killer_ui::gm_fence::default_position(viewport));
        let mut gm_fence = Window::new(WindowId::GM_FENCE, Rect::new(x, y, killer_ui::gm_fence::SIZE.0, killer_ui::gm_fence::SIZE.1), killer_ui::gm_fence::grip())
            .with_initial_position(killer_ui::gm_fence::initial_position(viewport));
        gm_fence.visible = false;
        desktop.windows.push(gm_fence);
        let (x, y) = windows_info.position(WindowId::GM_INVENTORY).unwrap_or_else(|| killer_ui::gm_inventory::default_position(viewport));
        let mut gm_inventory = Window::new(WindowId::GM_INVENTORY, Rect::new(x, y, killer_ui::gm_inventory::SIZE.0, killer_ui::gm_inventory::SIZE.1), killer_ui::gm_inventory::grip())
            .with_initial_position(killer_ui::gm_inventory::INITIAL_POSITION);
        gm_inventory.visible = false;
        desktop.windows.push(gm_inventory);
        let (x, y) = windows_info.position(WindowId::GM_STATUS).unwrap_or_else(|| killer_ui::gm_status::default_position(viewport));
        let mut gm_status = Window::new(WindowId::GM_STATUS, Rect::new(x, y, killer_ui::gm_status::SIZE.0, killer_ui::gm_status::SIZE.1), killer_ui::gm_status::GRIP)
            .with_initial_position(killer_ui::gm_status::INITIAL_POSITION);
        gm_status.visible = false;
        desktop.windows.push(gm_status);
        let (x, y) = windows_info.position(WindowId::GM_SKILLS).unwrap_or_else(|| killer_ui::gm_skills::default_position(viewport));
        let mut gm_skills = Window::new(WindowId::GM_SKILLS, Rect::new(x, y, killer_ui::gm_skills::SIZE.0, killer_ui::gm_skills::SIZE.1), killer_ui::gm_skills::GRIP)
            .with_initial_position(killer_ui::gm_skills::INITIAL_POSITION);
        gm_skills.visible = false;
        desktop.windows.push(gm_skills);
        let (x, y) = windows_info.position(WindowId::GM_QUEST).unwrap_or_else(|| killer_ui::gm_quest::default_position(viewport));
        let mut gm_quest = Window::new(WindowId::GM_QUEST, Rect::new(x, y, killer_ui::gm_quest::SIZE.0, killer_ui::gm_quest::SIZE.1), killer_ui::gm_quest::GRIP)
            .with_initial_position(killer_ui::gm_quest::INITIAL_POSITION);
        gm_quest.visible = false;
        desktop.windows.push(gm_quest);
        let (x, y) = windows_info.position(WindowId::GM_CLAN).unwrap_or_else(|| killer_ui::gm_clan::default_position(viewport));
        let mut gm_clan = Window::new(WindowId::GM_CLAN, Rect::new(x, y, killer_ui::gm_clan::SIZE.0, killer_ui::gm_clan::SIZE.1), killer_ui::gm_clan::GRIP)
            .with_initial_position(killer_ui::gm_clan::INITIAL_POSITION);
        gm_clan.visible = false;
        desktop.windows.push(gm_clan);
        let (x, y) = windows_info.position(WindowId::MESSENGER).unwrap_or(killer_ui::messenger::INITIAL_POSITION_XY);
        let mut messenger = Window::new(WindowId::MESSENGER, Rect::new(x, y, killer_ui::messenger::SIZE.0, killer_ui::messenger::SIZE.1), killer_ui::messenger::FRAME_RECT)
            .with_initial_position(killer_ui::initial_position::InitialPosition {
                enabled: true,
                horizontal_mode: 0,
                vertical_mode: 3,
                x: killer_ui::messenger::INITIAL_POSITION_XY.0,
                y: killer_ui::messenger::INITIAL_POSITION_XY.1,
            });
        messenger.visible = false;
        desktop.windows.push(messenger);
        for (id, rect) in [
            (WindowId::BROADCAST_OBSERVER, Rect::new(0, 0, 0, 0)),
            (WindowId::BROADCAST_GM_MSG, killer_ui::broadcast::gm_msg_rect(viewport, 0)),
            (WindowId::BROADCAST_SPECIAL_MSG, killer_ui::broadcast::special_rect(viewport)),
            (WindowId::BROADCAST_TOP, killer_ui::broadcast::top_rect(viewport.0)),
            (WindowId::BROADCAST_PLAYER_0, killer_ui::broadcast::player_rect(0, viewport.0)),
            (WindowId::BROADCAST_PLAYER_1, killer_ui::broadcast::player_rect(1, viewport.0)),
            (WindowId::BROADCAST_PARTY_0, killer_ui::broadcast::party_list_rect(0, viewport.0, 0)),
            (WindowId::BROADCAST_PARTY_1, killer_ui::broadcast::party_list_rect(1, viewport.0, 0)),
            (WindowId::BROADCAST_MSG, killer_ui::broadcast::msg_rect(viewport.0)),
        ] {
            let grip = Rect::new(0, 0, rect.w, rect.h);
            let initial = killer_ui::initial_position::InitialPosition { enabled: false, horizontal_mode: 0, vertical_mode: 3, x: rect.x, y: rect.y };
            let mut window = Window::new(id, rect, grip).with_initial_position(initial);
            window.visible = false;
            desktop.windows.push(window);
        }
        let (x, y) = windows_info.position(WindowId::GM).unwrap_or_else(|| killer_ui::gm_wnd::default_position(viewport));
        let mut gm = Window::new(WindowId::GM, Rect::new(x, y, killer_ui::gm_wnd::SIZE.0, killer_ui::gm_wnd::SIZE.1), killer_ui::gm_wnd::grip())
            .with_initial_position(killer_ui::gm_wnd::INITIAL_POSITION);
        gm.visible = false;
        desktop.windows.push(gm);
        let (x, y) = windows_info.position(WindowId::OBSERVER).unwrap_or_else(|| killer_ui::observer::default_position(viewport));
        let mut observer = Window::new(WindowId::OBSERVER, Rect::new(x, y, killer_ui::observer::SIZE.0, killer_ui::observer::SIZE.1), killer_ui::observer::grip())
            .with_initial_position(killer_ui::observer::INITIAL_POSITION);
        observer.visible = false;
        desktop.windows.push(observer);
        let (x, y) = windows_info.position(WindowId::CALC).unwrap_or_else(|| killer_ui::calc_wnd::default_position(viewport));
        let mut calc = Window::new(WindowId::CALC, Rect::new(x, y, killer_ui::calc_wnd::SIZE.0, killer_ui::calc_wnd::SIZE.1), killer_ui::calc_wnd::grip())
            .with_initial_position(killer_ui::calc_wnd::INITIAL_POSITION);
        calc.visible = false;
        desktop.windows.push(calc);
        let (x, y) = windows_info.position(WindowId::QUEST_LIST_INFO).unwrap_or_else(|| killer_ui::quest_info::default_position(viewport));
        let mut quest_info = Window::new(WindowId::QUEST_LIST_INFO, Rect::new(x, y, killer_ui::quest_info::SIZE.0, killer_ui::quest_info::SIZE.1), killer_ui::quest_info::grip())
            .with_initial_position(killer_ui::quest_info::initial_position(viewport));
        quest_info.visible = false;
        desktop.windows.push(quest_info);
        let (x, y) = windows_info.position(WindowId::PETITION_BEGIN).unwrap_or_else(|| killer_ui::petition_begin::default_position(viewport));
        let mut petition_begin = Window::new(WindowId::PETITION_BEGIN, Rect::new(x, y, killer_ui::petition_begin::SIZE.0, killer_ui::petition_begin::SIZE.1), killer_ui::petition_begin::grip())
            .with_initial_position(killer_ui::petition_begin::initial_position(viewport));
        petition_begin.visible = false;
        desktop.windows.push(petition_begin);
        let (x, y) = windows_info.position(WindowId::PETITION_GM).unwrap_or_else(|| killer_ui::petition_chat::gm_default_position(viewport));
        let mut petition_gm = Window::new(WindowId::PETITION_GM, Rect::new(x, y, killer_ui::petition_chat::GM_SIZE.0, killer_ui::petition_chat::GM_SIZE.1), killer_ui::petition_chat::gm_grip())
            .with_initial_position(killer_ui::petition_chat::gm_initial_position(viewport));
        petition_gm.visible = false;
        desktop.windows.push(petition_gm);
        let (x, y) = killer_ui::petition_chat::player_position(viewport);
        let mut petition = Window::new(WindowId::PETITION, Rect::new(x, y, killer_ui::petition_chat::SIZE.0, killer_ui::petition_chat::SIZE.1), killer_ui::petition_chat::grip())
            .with_initial_position(killer_ui::initial_position::DISABLED_INITIAL_POSITION);
        petition.visible = false;
        desktop.windows.push(petition);
        let (x, y) = killer_ui::petition_feedback::position(viewport);
        let mut petition_feedback = Window::new(WindowId::PETITION_FEEDBACK, Rect::new(x, y, killer_ui::petition_feedback::SIZE.0, killer_ui::petition_feedback::SIZE.1), killer_ui::petition_feedback::grip())
            .with_initial_position(killer_ui::initial_position::DISABLED_INITIAL_POSITION);
        petition_feedback.visible = false;
        desktop.windows.push(petition_feedback);
        let mut debug = Window::new(killer_ui::debug_wnd::ID, Rect::new(0, 0, viewport.0, killer_ui::debug_wnd::HEIGHT), Rect::new(0, 0, 0, 0))
            .with_initial_position(killer_ui::initial_position::DISABLED_INITIAL_POSITION);
        debug.visible = false;
        desktop.windows.push(debug);
        for id in killer_ui::gm_snoop::WINDOW_IDS {
            let (x, y) = killer_ui::gm_snoop::POSITION;
            let mut snoop = Window::new(id, Rect::new(x, y, killer_ui::gm_snoop::SIZE.0, killer_ui::gm_snoop::SIZE.1), killer_ui::gm_snoop::GRIP)
                .with_initial_position(killer_ui::initial_position::DISABLED_INITIAL_POSITION);
            snoop.visible = false;
            desktop.windows.push(snoop);
        }
    }

    fn list_sounds(&mut self) -> Vec<usize> {
        let manor = &mut self.manor_info;
        let setting = &mut self.manor_setting;
        let lists = [
            &mut manor.default.list,
            &mut manor.seed.list,
            &mut manor.crop.list,
            &mut setting.setting.list,
            &mut setting.crop_setting.list,
            &mut setting.crop_sell.list,
            &mut setting.procure_change.list,
            &mut self.quest_info.list,
        ];
        lists.into_iter().flat_map(|list| std::mem::take(&mut list.sounds)).collect()
    }

    pub fn take_commands(&mut self) -> Vec<UiCommand> {
        let sounds = self.list_sounds();
        self.pending.extend(sounds.into_iter().map(|index| UiCommand::InterfaceSound(interface_sound::TABLE[index])));
        self.pending.append(&mut self.trade.commands);
        self.pending.append(&mut self.multisell.commands);
        self.pending.append(&mut self.manor_setting.commands);
        let mut hero = std::mem::take(&mut self.hero_tower.commands);
        for command in &mut hero {
            if let UiCommand::Net(NetCommand::WriteHeroWords { words }) = command {
                *words = self.word_filter(words);
            }
        }
        self.pending.append(&mut hero);
        self.pending.append(&mut self.petition.commands);
        self.pending.append(&mut self.olympiad.commands);
        for id in self.petition.notices.drain(..) {
            self.dialogs.push(Dialog::Notice(SystemMessage { id: id as i32, args: Vec::new() }));
        }
        let requests: Vec<_> = self.manor_info.requests.drain(..).collect();
        self.pending.extend(requests.into_iter().map(|command| UiCommand::Net(NetCommand::Bypass { command })));
        self.pending.append(&mut self.train.commands);
        self.pending.append(&mut self.henna.commands);
        self.pending.append(&mut self.recipes.commands);
        self.pending.append(&mut self.recipes.shop.commands);
        self.pending.append(&mut self.pet.commands);
        self.pending.append(&mut self.macros.take_commands());
        let commands = std::mem::take(&mut self.pending);
        let mut commands: Vec<UiCommand> = commands
            .into_iter()
            .flat_map(|command| {
                let registered = matches!(command, UiCommand::Net(NetCommand::ShortcutReg { .. }));
                let sound = registered.then_some(UiCommand::InterfaceSound(interface_sound::TABLE[interface_sound::PICKUP]));
                std::iter::once(command).chain(sound)
            })
            .collect();
        let mut seen: Vec<&'static str> = Vec::new();
        commands.retain(|command| match command {
            UiCommand::InterfaceSound(name) if name == &interface_sound::TABLE[interface_sound::WINDOW_CLOSE] => {
                let first = !seen.contains(name);
                seen.push(name);
                first
            }
            _ => true,
        });
        commands
    }

    pub fn set_input_language(&mut self, language: killer_ui::ime::Language) {
        self.macros.edit.set_language(language);
        self.hero_tower.tower.set_language(language);
    }

    pub fn set_petition_categories(&mut self, categories: Vec<killer_ui::petition_begin::Category>) {
        let sysstrings = &self.sysstrings;
        let lookup = |id: u32| sysstrings.get(&id).cloned();
        self.petition = crate::ui_petition::PetitionWindows::new(lookup, self.petition.line_height(), categories);
    }

    pub fn show_quest_info(&mut self, tables: Option<&DatTables>, desktop: &mut Desktop) {
        let rows: Vec<killer_ui::quest_info::QuestInfoRow> = tables
            .map(|tables| {
                let mut listed: Vec<_> = tables.questname.values().filter(|quest| quest.valid != 0 && quest.state == killer_ui::quest_info::LISTED_STATE).collect();
                listed.sort_by_key(|quest| quest.index);
                listed
                    .into_iter()
                    .map(|quest| killer_ui::quest_info::QuestInfoRow {
                        quest_id: quest.id,
                        title: quest.title.clone(),
                        requirement: quest.requirement.clone(),
                        level: quest.level,
                        max_level: quest.quest_level,
                        quest_type: quest.quest_type,
                        npc_name: u32::try_from(quest.start_npc_id)
                            .ok()
                            .and_then(|id| tables.npcname.get(&id))
                            .map(|npc| npc.name.clone())
                            .unwrap_or_default(),
                        npc_loc: quest.start_npc_loc,
                        intro: quest.intro.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.quest_info.set_rows(&rows);
        if let Some(window) = desktop.window_mut(WindowId::QUEST_LIST_INFO) {
            window.visible = true;
        }
        if let Some(index) = desktop.windows.iter().position(|window| window.id == WindowId::QUEST_LIST_INFO) {
            let window = desktop.windows.remove(index);
            desktop.windows.push(window);
        }
    }

    pub fn tick_calc(&mut self, now: killer_platform::time::Instant) {
        if let Some(last) = self.calc_tick.replace(now) {
            let elapsed = now.saturating_duration_since(last).as_millis().min(u128::from(u32::MAX)) as u32;
            self.calc.tick(elapsed);
            self.gm.tick(elapsed);
            self.gm_fence.tick(elapsed);
            let actions = self.gm_event.tick(elapsed);
            self.pending_gm_event.extend(actions);
        }
    }

    pub fn answer_private_shop(&mut self, accept: bool, text: Option<&str>, desktop: &mut Desktop) {
        self.private_shop.answer_dialog(accept, text, desktop, &mut self.pending);
    }

    pub fn drain_shop_sounds(&mut self) {
        let shops = [&mut self.shop, &mut self.private_shop, &mut self.shop_preview, &mut self.warehouse, &mut self.gm_store, &mut self.manor_shop, &mut self.delivery];
        for shop in shops {
            for name in std::mem::take(&mut shop.sounds) {
                self.pending.push(UiCommand::InterfaceSound(name));
            }
            for id in std::mem::take(&mut shop.messages) {
                self.pending.push(UiCommand::SystemMessage(id));
            }
        }
    }

    pub fn tick_messenger(&mut self, now: killer_platform::time::Instant, desktop: &mut Desktop) {
        if let Some(last) = self.messenger_tick.replace(now) {
            let elapsed = now.saturating_duration_since(last).as_millis().min(60_000) as i32;
            self.messenger.tick(elapsed, desktop);
        }
        let commands = std::mem::take(&mut self.messenger.commands);
        self.pending.extend(commands);
        self.drain_shop_sounds();
    }

    pub fn tick_broadcast(&mut self, now: killer_platform::time::Instant, desktop: &mut Desktop) {
        if let Some(last) = self.broadcast_tick.replace(now) {
            let elapsed = now.saturating_duration_since(last).as_millis().min(60_000) as i32;
            self.broadcast.tick(elapsed, desktop);
        }
    }

    pub fn open_calc(&mut self, desktop: &mut Desktop, font: &killer_ui::font::BitmapFont) {
        self.calc.open(font);
        if let Some(window) = desktop.window_mut(WindowId::CALC) {
            window.visible = true;
        }
        if let Some(index) = desktop.windows.iter().position(|window| window.id == WindowId::CALC) {
            let window = desktop.windows.remove(index);
            desktop.windows.push(window);
        }
    }

    pub fn gm_action(&mut self, action: killer_ui::gm_wnd::GmAction, desktop: &mut Desktop) {
        use killer_ui::gm_wnd::GmAction;
        match action {
            GmAction::Close => set_visible(desktop, WindowId::GM, false),
            GmAction::Bypass(command) => self.send(NetCommand::BypassBuildCmd { command }),
            GmAction::ForcePeti { name } => {
                let command = format!("force_peti {name} {}", self.force_peti_text);
                self.send(NetCommand::BypassBuildCmd { command });
            }
            GmAction::Confirm { dialog_type, message_id, name, command } => {
                self.dialogs.push(Dialog::GmConfirm { dialog_type, message_id: message_id as i32, name, command });
            }
            GmAction::View { kind, name } => {
                use killer_ui::gm_wnd::ViewKind;
                let id = match kind {
                    ViewKind::Status => WindowId::GM_STATUS,
                    ViewKind::Clan => WindowId::GM_CLAN,
                    ViewKind::Skills => WindowId::GM_SKILLS,
                    ViewKind::Quest => WindowId::GM_QUEST,
                    ViewKind::Inventory => WindowId::GM_INVENTORY,
                    ViewKind::Warehouse => WindowId::GM_STORE,
                };
                if desktop.window(id).is_some_and(|window| window.visible) {
                    set_visible(desktop, id, false);
                    if kind == ViewKind::Skills {
                        self.gm_skills.clear();
                    }
                    return;
                }
                self.send(NetCommand::GmCommand { name, kind: kind as i32 });
                if kind == ViewKind::Quest {
                    self.gm_quest.reset();
                    self.send(NetCommand::RequestQuestList);
                }
                set_visible(desktop, id, true);
                if let Some(index) = desktop.windows.iter().position(|window| window.id == id) {
                    let window = desktop.windows.remove(index);
                    desktop.windows.push(window);
                }
            }
            GmAction::Notice(id) => self.pending_message(id, killer_ui::gm_wnd::NAME_LABEL_COLOR.0),
            GmAction::Search { kind, name } => self.gm_searches.push((kind, name)),
            GmAction::ServerTransfer { id } => self.pending.push(UiCommand::ServerTransfer { id }),
        }
    }

    pub fn snoop_event(&mut self, event: &killer_world::WorldEvent, font: &killer_ui::font::BitmapFont, desktop: &mut Desktop) {
        let killer_world::WorldEvent::Snoop(snoop) = event else { return };
        if self.snoop.open_snoop(snoop.first, &snoop.first_text) == killer_ui::gm_snoop::Opened::Full {
            self.dialogs.push(Dialog::GmNotice { message_id: killer_ui::gm_snoop::WINDOWS_FULL_SYSMSG as i32 });
        }
        let Some((line, color)) = killer_ui::gm_snoop::console_line(&snoop.second_text, &snoop.third_text, snoop.third) else { return };
        for index in self.snoop.add_line(snoop.first, &line, color, snoop.third, font) {
            set_visible(desktop, killer_ui::gm_snoop::WINDOW_IDS[index], true);
        }
    }

    pub fn debug_command(&mut self, window: bool, desktop: &mut Desktop) {
        if !window {
            self.debug.log_enabled = !self.debug.log_enabled;
            return;
        }
        let shown = desktop.window(killer_ui::debug_wnd::ID).is_some_and(|window| window.visible);
        set_visible(desktop, killer_ui::debug_wnd::ID, !shown);
    }

    pub fn snoop_mouse_down(&mut self, id: WindowId, x: i32, y: i32) {
        if let Some(index) = killer_ui::gm_snoop::WINDOW_IDS.iter().position(|candidate| *candidate == id) {
            self.snoop.windows[index].mouse_down(x, y);
        }
    }

    pub fn snoop_mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        let outside = (i32::MIN / 2, i32::MIN / 2);
        for (index, id) in killer_ui::gm_snoop::WINDOW_IDS.into_iter().enumerate() {
            let (x, y) = over.filter(|(over_id, _, _)| *over_id == id).map_or(outside, |(_, x, y)| (x, y));
            if let Some(snoop_id) = self.snoop.windows[index].mouse_up(x, y) {
                set_visible(desktop, id, false);
                self.send(NetCommand::GmSnoopEnd { snoop_id });
            }
        }
    }

    pub fn take_gm_event_actions(&mut self) -> Vec<killer_ui::gm_event::GmEventAction> {
        std::mem::take(&mut self.pending_gm_event)
    }

    pub fn warning_message(&mut self, id: u32, number: Option<i32>) {
        self.pending.push(UiCommand::WarningMessage { id, numbers: number.into_iter().collect() });
    }

    pub fn request_gm_event_file(&mut self, file: String, commands: bool) {
        self.pending.push(UiCommand::GmEventFile { file, commands });
    }

    pub fn sync_gm(&mut self, tables: Option<&DatTables>) {
        let searches = std::mem::take(&mut self.gm_searches);
        if let Some(tables) = tables {
            for (kind, name) in searches {
                use killer_ui::gm_wnd::{NPC_ID_OFFSET, SearchKind, search_result};
                let found = match kind {
                    SearchKind::Npc => tables.search_npc(&name).map(|id| id as i32 + NPC_ID_OFFSET),
                    SearchKind::Item => tables.search_item(&name).map(|id| id as i32),
                    SearchKind::Skill => tables.search_skill(&name).map(|id| id as i32),
                };
                if let Some(id) = found {
                    self.dialogs.push(Dialog::GmText { text: search_result(id, &name) });
                }
            }
        }
    }

    pub fn gm_target_selected(&mut self, world: &World, font: &killer_ui::font::BitmapFont) {
        let name = world.target.and_then(|id| world.entity(id)).filter(|entity| matches!(entity.kind, EntityKind::Player { .. })).map(|entity| entity.name.clone());
        if let Some(name) = name {
            self.gm.set_name(&name, font);
        }
    }

    pub fn load_help(&mut self, path: String) {
        self.pending.push(UiCommand::LoadHelp(path));
    }

    pub fn send(&mut self, command: NetCommand) {
        self.pending.push(UiCommand::Net(command));
    }

    pub fn pending_message(&mut self, id: u32, color: u32) {
        self.pending.push(UiCommand::ColoredSystemMessage { id, color });
    }

    pub fn key_state(&self, world: &World, chat_focused: bool, dialog_visible: bool) -> KeyState {
        KeyState {
            observer_mode: keys::OBSERVER_NORMAL,
            enter_chatting: self.enter_chatting,
            language_korean: false,
            dialog_visible,
            chat_focused,
            chat_locked: false,
            in_game: world.me.is_some(),
            tab_blocked: world.windows.buy.is_some() || world.windows.sell.is_some() || world.windows.multisell.is_some(),
            camera_fixed: world.camera_fixed,
            gm_level: 0,
            messenger: true,
            tooltips: false,
        }
    }

    pub fn hook(&mut self, msg: u32, vk: u32, lparam: u32, state: &KeyState, bar: &mut ShortcutView) -> Hooked {
        bar.enter_chatting = state.enter_chatting;
        bar.dialog_visible = state.dialog_visible;
        keys::hook_shortcut_key(msg, vk, lparam, state, bar)
    }

    pub fn apply_effects(
        &mut self,
        hooked: &Hooked,
        desktop: &mut Desktop,
        bar: &mut ShortcutView,
        world: &World,
    ) -> Vec<EffectResult> {
        let mut results = Vec::new();
        for effect in &hooked.effects {
            match effect {
                Effect::SelectTab(tab) => self.select_tab(*tab, desktop),
                Effect::ToggleMain(Some(tab)) => self.select_tab(*tab, desktop),
                Effect::ToggleMain(None) => {
                    let visible = desktop.window(WindowId::MAIN).is_some_and(|w| w.visible);
                    self.sound(if visible { interface_sound::CHARSTAT_CLOSE } else { interface_sound::CHARSTAT_OPEN });
                    toggle(desktop, WindowId::MAIN);
                }
                Effect::Inventory => self.toggle_inventory(desktop, world),
                Effect::SystemMenu => {
                    let visible = desktop.window(SYSMENU_WINDOW).is_some_and(|w| w.visible);
                    self.sound(if visible { interface_sound::SYSTEM_CLOSE } else { interface_sound::SYSTEM_OPEN });
                    toggle(desktop, SYSMENU_WINDOW);
                }
                Effect::Board => self.toggle_board(desktop),
                Effect::Minimap => self.send(NetCommand::ShowMinimap),
                Effect::HideUi => {
                    if desktop.window(WindowId::PARTY_ROOM).is_some_and(|window| window.visible)
                        && let Some(action) = self.party_room.leave_request()
                    {
                        self.party_room_actions(vec![action], desktop);
                    }
                    self.delivery.clear();
                    self.select_delivery.clear();
                    set_visible(desktop, WindowId::DELIVERY, false);
                    set_visible(desktop, WindowId::SELECT_DELIVERY, false);
                    self.warehouse.clear();
                    set_visible(desktop, WindowId::WAREHOUSE, false);
                    self.gm_store.clear();
                    set_visible(desktop, WindowId::GM_STORE, false);
                    self.manor_shop.clear();
                    set_visible(desktop, WindowId::MANOR_SHOP, false);
                    self.shop.clear();
                    set_visible(desktop, WindowId::SHOP, false);
                    if desktop.window(WindowId::PRIVATE_SHOP).is_some_and(|w| w.visible)
                        && let Some(command) = self.private_shop.private_quit()
                    {
                        self.send(command);
                    }
                    self.private_shop.clear();
                    set_visible(desktop, WindowId::PRIVATE_SHOP, false);
                    self.shop_preview.clear();
                    set_visible(desktop, WindowId::SHOP_PREVIEW, false);
                    self.multisell.clear(desktop);
                    self.train.view.clear();
                    set_visible(desktop, WindowId::TRAIN, false);
                    self.macros.clear(desktop);
                    self.hidden = !self.hidden;
                }
                Effect::CloseAll => {
                    self.delivery.clear();
                    self.select_delivery.clear();
                    set_visible(desktop, WindowId::DELIVERY, false);
                    set_visible(desktop, WindowId::SELECT_DELIVERY, false);
                    self.warehouse.clear();
                    set_visible(desktop, WindowId::WAREHOUSE, false);
                    self.gm_store.clear();
                    set_visible(desktop, WindowId::GM_STORE, false);
                    self.manor_shop.clear();
                    set_visible(desktop, WindowId::MANOR_SHOP, false);
                    self.shop.clear();
                    set_visible(desktop, WindowId::SHOP, false);
                    self.private_shop.clear();
                    set_visible(desktop, WindowId::PRIVATE_SHOP, false);
                    self.multisell.clear(desktop);
                    self.train.view.clear();
                    set_visible(desktop, WindowId::TRAIN, false);
                    self.macros.clear(desktop);
                    set_visible(desktop, WindowId::INVENTORY, false);
                }
                Effect::ShortcutPage(page) => bar.set_page(*page),
                Effect::TargetCancel => self.send(NetCommand::TargetCancel),
                Effect::FocusChat => results.push(EffectResult::FocusChat),
                Effect::ChatTab(direction) => results.push(EffectResult::ChatTab(*direction)),
                Effect::Macros => self.toggle_macro_list(desktop),
                Effect::MinFrame => self.pending.push(UiCommand::ToggleMinFrameRate),
                Effect::Messenger => self.messenger.toggle_messenger(desktop),
                Effect::GmWindow => {
                    let visible = desktop.window(WindowId::GM).is_some_and(|window| window.visible);
                    set_visible(desktop, WindowId::GM, !visible);
                }
                Effect::GmPetition => {
                    set_visible(desktop, WindowId::PETITION_GM, true);
                    if let Some(index) = desktop.windows.iter().position(|window| window.id == WindowId::PETITION_GM) {
                        let window = desktop.windows.remove(index);
                        desktop.windows.push(window);
                    }
                }
                Effect::ToggleChat => {
                    if desktop.window(WindowId::CHAT).is_some_and(|window| window.visible) {
                        set_visible(desktop, WindowId::CHAT, false);
                    } else {
                        set_visible(desktop, WindowId::CHAT, true);
                        results.push(EffectResult::FocusChat);
                    }
                }
                Effect::GroupCompact => results.push(EffectResult::GroupCompact),
                Effect::HideTooltips => self.tooltip = None,
                Effect::ImeReset | Effect::Olympiad(_) => {}
            }
        }
        self.drain_bar(bar, world);
        results
    }

    pub fn open_marked_quest(&mut self, quest_id: i32, desktop: &mut Desktop) {
        self.select_tab(MainTab::Quests, desktop);
        self.quest.marked = Some(quest_id);
    }

    fn select_tab(&mut self, tab: MainTab, desktop: &mut Desktop) {
        let visible = desktop.window(WindowId::MAIN).is_some_and(|w| w.visible);
        if visible && self.main.tab == tab {
            self.sound(interface_sound::CHARSTAT_CLOSE);
            set_visible(desktop, WindowId::MAIN, false);
            return;
        }
        self.sound(interface_sound::CHARSTAT_OPEN);
        let shown = !visible || self.main.tab != tab;
        self.main.tab = tab;
        set_visible(desktop, WindowId::MAIN, true);
        if shown && tab == MainTab::Quests {
            self.begin_quest_list();
        }
        if shown && tab == MainTab::Clan {
            self.pledge_show_pending = true;
        }
        if shown && tab == MainTab::Skills {
            self.send(NetCommand::RequestSkillList);
        }
    }

    fn begin_quest_list(&mut self) {
        self.quest.reset();
        self.send(NetCommand::RequestQuestList);
    }

    fn toggle_inventory(&mut self, desktop: &mut Desktop, world: &World) {
        if desktop.window(WindowId::SHOP).is_some_and(|w| w.visible) || world.trade.partner_id.is_some() {
            return;
        }
        if desktop.window(WindowId::INVENTORY).is_some_and(|w| w.visible) {
            self.sound(interface_sound::INVENTORY_CLOSE);
            set_visible(desktop, WindowId::INVENTORY, false);
            return;
        }
        self.send(NetCommand::RequestItemList);
        self.sound(interface_sound::INVENTORY_OPEN);
    }

    pub fn crest_file(&mut self, kind: killer_assets::crest::CrestKind, file: Option<&[u8]>) {
        use killer_assets::crest::{CrestError, CrestKind, SYSMSG_FILE_NOT_FOUND, prepare};
        let notice = |id: u32| UiCommand::ColoredSystemMessage { id, color: killer_ui::color::GOLD.0 };
        let Some(file) = file else {
            self.pending.push(notice(SYSMSG_FILE_NOT_FOUND));
            return;
        };
        match prepare(file, kind, 0) {
            Ok(data) => self.send(match kind {
                CrestKind::Clan => NetCommand::SetPledgeCrest { data },
                CrestKind::Ally => NetCommand::SetAllyCrest { data },
                CrestKind::Large => NetCommand::ExSetPledgeCrestLarge { data },
            }),
            Err(CrestError(id)) => self.pending.push(notice(id)),
        }
    }

    pub fn push_command(&mut self, command: UiCommand) {
        self.pending.push(command);
    }

    pub fn sound(&mut self, index: usize) {
        if let Some(name) = interface_sound::name(index) {
            self.pending.push(UiCommand::InterfaceSound(name));
        }
    }

    pub fn open_demo_windows(&mut self, desktop: &mut Desktop, extra: &str) {
        set_visible(desktop, WindowId::INVENTORY, true);
        self.main.tab = MainTab::Status;
        set_visible(desktop, WindowId::MAIN, true);
        for name in extra.split(',').map(str::trim) {
            match name {
                "minimap" => {
                    set_visible(desktop, WindowId::MAP_COVER, true);
                    self.minimap_recenter = true;
                }
                "clan" => {
                    self.main.tab = MainTab::Clan;
                    self.pledge_show_pending = true;
                }
                "quests" => self.main.tab = MainTab::Quests,
                "skills" => self.main.tab = MainTab::Skills,
                "actions" => self.main.tab = MainTab::Actions,
                "option" => set_visible(desktop, WindowId::OPTION, true),
                _ => {}
            }
        }
    }

    /// Um comando de macro de skill não avança por tempo: o stepper do original volta sem
    /// avançar enquanto o atraso do comando (`+0xf0`) for zero e o comando for de skill
    /// (`0x100ab2bb`..`0x100ab2f7`). Quem destrava é o `UGameEngine::OnReceiveMagicSkillUse`
    /// (`0x10480220`, `0x10480aa5`) quando o conjurador é o jogador local: ele chama o slot
    /// `0x3dc` do console (`0x10068580` → `0x100a89f0`) com `(skill_id, hit_time)`; o macro
    /// em execução cujo comando atual resolve para a mesma skill recebe a espera
    /// `hit_time / 1000 + 1` (divisão inteira; `hit_time == 0` dá `1.0`). Ver
    /// `docs/evidence/magic-skill-use.md`.
    pub fn macro_skill_started(&mut self, event: &WorldEvent, world: &World) {
        let WorldEvent::MagicSkillUse(use_skill) = event else { return };
        let Some(cast) = world.cast_accepted(use_skill) else { return };
        self.macro_cast_started(cast);
    }

    fn macro_cast_started(&mut self, cast: &killer_world::magic::AcceptedCast) {
        if !cast.caster_is_me {
            return;
        }
        let Some(run) = self.macro_run.as_mut() else { return };
        if run.waiting_skill != Some(cast.skill_id) {
            return;
        }
        run.waiting_skill = None;
        run.started = Instant::now();
        run.delay = killer_world::magic::macro_wait_seconds(cast.hit_time);
        log::info!("macro: magia {} começou, espera {:.2}s", cast.skill_id, run.delay);
    }

    /// `ReceiveInventoryUpdate` (`0x10402460`) chama, para as mudanças tipo 2 e 3, o slot
    /// `0x328` do engine → console `0x280` (`0x10068430`) → `0x10106180`: cada atalho de item
    /// com o `objectId` da mudança busca o item na janela de inventário; se não acha, o atalho
    /// é esvaziado (`0x10105aa0`: ocupado, recarga e realce a zero). Item que volta a existir
    /// (tipo 1 ou 2 com o mesmo `objectId`) deixa o atalho valer de novo.
    fn shortcut_item_refresh(&mut self, event: &WorldEvent) {
        match event {
            WorldEvent::InventoryUpdate(update) => {
                for change in &update.changes {
                    match change.change {
                        l2_net::game::packets::ITEM_REMOVED => {
                            self.removed_items.insert(change.item.object_id);
                        }
                        _ => {
                            self.removed_items.remove(&change.item.object_id);
                        }
                    }
                }
            }
            WorldEvent::ItemList(_) => self.removed_items.clear(),
            _ => {}
        }
    }

    /// Slot `0x3d4` do console (`0x100748d0`): com `reuse_delay / 1000 > 0` (truncado) o
    /// engine manda `(skill_id, level, segundos, 0)` e os atalhos de skill com o mesmo id e
    /// nível começam a contagem de recarga do zero.
    pub fn skill_reuse_started(&mut self, event: &WorldEvent, world: &World, now: Instant) {
        let WorldEvent::MagicSkillUse(use_skill) = event else { return };
        let Some(cast) = world.cast_accepted(use_skill).filter(|cast| cast.caster_is_me) else { return };
        let Some(seconds) = killer_world::magic::reuse_started(cast.reuse_delay) else { return };
        let seconds = seconds as f32;
        self.cooldowns.insert(cast.skill_id, Cooldown { level: cast.skill_level, started: now, reuse: seconds, remaining: seconds, flash: None, stopped: false });
    }

    /// Comando de macro correspondente a um slot da barra de atalhos, pra quando ele é solto
    /// numa linha do editor. O original monta **sempre** `"/useskill <nome do slot>"`
    /// (`0x100ae1b9`..`0x100ae1f2`): o verbo é fixo, o token `0x18`, e o nome vem de
    /// `célula + 0x14`. O campo `+0x1b14`, que decidiria pular a linha, nunca recebe valor
    /// diferente de zero em lugar nenhum do binário — só cópias e zeragem —, então não há
    /// restrição de tipo de slot. Quem resolve o que a linha vira é o Save, que procura o nome
    /// na tabela de skills: nome que não é skill vira `kind 1` com id 0.
    pub fn macro_command_from_shortcut(
        &self,
        world: &World,
        slot: usize,
    ) -> Option<killer_ui::macros::MacroCommandView> {
        let shortcut = world.shortcut(slot as i32)?;
        let name = match shortcut.kind {
            ShortcutKind::Skill { skill_id, level } => self
                .skillname
                .get(&(skill_id.max(0) as u32, level.max(0) as u32))
                .map(|skill| skill.name.clone()),
            ShortcutKind::Action { id } => {
                self.actions.actions.iter().find(|action| action.id == id).map(|action| action.name.clone())
            }
            ShortcutKind::Item { object_id } => (0..self.inventory.total_rows() as usize * 100)
                .filter_map(|index| self.inventory.grid_item(index))
                .find(|item| item.object_id == object_id)
                .map(|item| item.name.clone()),
            _ => None,
        }?;
        let verb = self.commandname.iter().find(|(_, token)| **token == MACRO_VERB_USESKILL).map(|(name, _)| name.clone())?;
        let label = format!("/{verb} {name}");
        if label.chars().count() > killer_ui::macros::MACRO_COMMAND_MAX_CHARS {
            return None;
        }
        let command = macro_command_from_line(&label, 0, &self.skillname, &self.commandname)?;
        Some(killer_ui::macros::MacroCommandView {
            kind: command.kind,
            skill_id: command.skill_id,
            shortcut_id: command.shortcut_id,
            command: command.command,
            label,
        })
    }

    fn word_filter(&self, text: &str) -> String {
        let replacement = self.sysstrings.get(&crate::ui2d::BAD_WORD_SYSSTRING).map_or("", String::as_str);
        l2_dat::obscene::mask(&self.obscene, text, replacement).0
    }

    fn set_macro_line(&mut self, line: usize, label: String) {
        if label.chars().count() > killer_ui::macros::MACRO_COMMAND_MAX_CHARS {
            return;
        }
        let Some(command) = macro_command_from_line(&label, 0, &self.skillname, &self.commandname) else { return };
        self.macros.edit.set_command(line, killer_ui::macros::MacroCommandView {
            kind: command.kind,
            skill_id: command.skill_id,
            shortcut_id: command.shortcut_id,
            command: command.command,
            label,
        });
    }

    fn macro_verb(&self, token: u32) -> Option<String> {
        self.commandname.iter().find(|(_, value)| **value == token).map(|(name, _)| name.clone())
    }

    fn drop_skill_on_macro_line(&mut self, id: i32, x: i32, y: i32) {
        let Some(line) = self.macros.edit.hit_line(x, y) else { return };
        self.macros.edit.clear_mark(line);
        let Some(skill) = self.skills.skills.iter().find(|skill| skill.id == id && !skill.passive) else { return };
        let Some(verb) = self.macro_verb(MACRO_VERB_USESKILL) else { return };
        let label = format!("/{verb} {}", skill.name);
        self.set_macro_line(line, label);
    }

    fn drop_action_on_macro_line(&mut self, id: i32, x: i32, y: i32) {
        let Some(line) = self.macros.edit.hit_line(x, y) else { return };
        self.macros.edit.clear_mark(line);
        if MACRO_FORBIDDEN_ACTIONS.contains(&id) || (0x29..=0x31).contains(&id) || id >= 1000 {
            self.pending.push(UiCommand::SystemMessage(MACRO_FORBIDDEN_ACTION_SYSMSG));
            return;
        }
        let Some(verb) = action_verb(&self.action_commands, &self.commandname, id) else { return };
        self.set_macro_line(line, format!("/{verb}"));
    }

    pub fn macro_key(&mut self, key: killer_ui::chat::InputKey, desktop: &mut Desktop) -> bool {
        self.macros.key(key, desktop)
    }

    pub fn set_option_applying(&mut self, applying: bool) {
        self.option_applying = applying;
        self.option_pressed = None;
        if applying { self.option.lose_combo_focus(); }
    }

    fn option_mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        if self.option_applying { return; }
        let released = match over {
            Some((WindowId::OPTION, x, y)) => self.option.hit(x, y),
            _ => None,
        };
        self.option.combo_mouse_up(released);
        self.option.initialize_pressed = false;
        let Some(pressed) = self.option_pressed.take() else { return };
        if released != Some(pressed) {
            return;
        }
        match pressed {
            killer_ui::option::OptionHit::Initialize => self.pending.push(UiCommand::InitializeInterface),
            killer_ui::option::OptionHit::Ok | killer_ui::option::OptionHit::Apply | killer_ui::option::OptionHit::Cancel | killer_ui::option::OptionHit::Close => {
                if matches!(pressed, killer_ui::option::OptionHit::Ok | killer_ui::option::OptionHit::Apply) {
                    self.pending.push(UiCommand::ApplyOptions { close: matches!(pressed, killer_ui::option::OptionHit::Ok) });
                }
                if matches!(pressed, killer_ui::option::OptionHit::Cancel | killer_ui::option::OptionHit::Close) {
                    self.option.lose_combo_focus();
                    set_visible(desktop, WindowId::OPTION, false);
                }
            }
            _ => {}
        }
    }

    /// Solta o botão da janela de encantar. O `OK` (`0xf0f012`, tratador `0x10095e90`) manda o
    /// id do item escolhido e **não** fecha a janela — quem fecha é o resultado do servidor. O
    /// `Cancel` (`0xf0f013`, `0x100940c0`) manda `-1` e fecha na hora.
    fn enchant_mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        let Some(pressed) = self.enchant.pressed.take() else { return };
        let released = match over {
            Some((WindowId::ITEM_ENCHANT, x, y)) => self.enchant.hit(x, y),
            _ => None,
        };
        if let killer_ui::enchant_wnd::EnchantHit::Scroll(hit) = pressed {
            let (first, visible, total) = (
                self.enchant.grid.scroll_row as i32,
                killer_ui::enchant_wnd::GRID_ROWS as i32,
                self.enchant.grid.total_rows(self.enchant.entries.len()),
            );
            let grid = &mut self.enchant.grid;
            grid.scroll.up_pressed = false;
            grid.scroll.down_pressed = false;
            grid.scroll.slider.mouse_up();
            if released == Some(pressed) {
                match hit {
                    ScrollHit::Up => grid.set_first_row(line_up(first, visible, total)),
                    ScrollHit::Down => grid.set_first_row(line_down(first, visible, total)),
                    ScrollHit::Slider(_) => {}
                }
            }
            return;
        }
        if released != Some(pressed) {
            return;
        }
        match pressed {
            killer_ui::enchant_wnd::EnchantHit::Ok => {
                if let Some(object_id) = self.enchant.selected().map(|entry| entry.object_id) {
                    self.send(NetCommand::EnchantItem { object_id });
                }
            }
            killer_ui::enchant_wnd::EnchantHit::Cancel => {
                self.send(NetCommand::EnchantItem { object_id: killer_ui::enchant_wnd::CANCEL_OBJECT_ID });
                self.hide_enchant(desktop);
            }
            _ => {}
        }
    }

    /// `NCItemEnchantWnd::Hide @ 0x100940f0` esvazia a grade (`FUN_100fe5e0`) antes de chamar o
    /// `Hide` da base, então fechar a janela sempre larga o que estava escolhido.
    fn hide_enchant(&mut self, desktop: &mut Desktop) {
        self.enchant.set_entries(Vec::new());
        self.enchant.pressed = None;
        set_visible(desktop, WindowId::ITEM_ENCHANT, false);
    }

    /// Preenche a grade com o inventário inteiro e põe o nome do pergaminho no título, como o
    /// abridor do original faz (`0x100666c0`: `Show`, foco, `FUN_10099370` copiando as células do
    /// inventário e `appSprintf("%s(%s)", …)` no título).
    fn sync_enchant(&mut self, tables: Option<&DatTables>, scroll_item_id: Option<i32>) {
        // O `FUN_10099370` preenche a grade com DOIS laços: primeiro a grade da aba de item do
        // inventário (`console+0x120 -> +0x10c`), depois o painel de equipamento
        // (`+0x114` = `NCEquipItemWnd`, os itens EQUIPADOS). Em ambos só adiciona o item cuja
        // categoria (`+0x1b14`) é 0, 1 ou 2 — arma, armadura, acessório —, descartando 3 (etcitem)
        // e 4 (quest). Os acessórios de cabelo (Circlet/Tiara) são `armorgrp` com
        // `body_part == HAIR (16)` e caem fora de {0,1,2}: não encantam. Por isso a arma equipada
        // (ex.: dual) precisa entrar também. Item fora do catálogo carregado fica na grade (não
        // escondemos um encantável só porque o ícone não foi achado).
        let excluded = |item: &killer_ui::inventory::InventoryItemView| {
            let Some(tables) = tables else { return false };
            let Ok(id) = u32::try_from(item.item_id) else { return false };
            if tables.item_icons.get(id).is_some_and(|icon| icon.kind == l2_dat::itemicons::ItemKind::Etc) {
                return true;
            }
            tables.armorgrp.get(&id).is_some_and(|armor| armor.body_part == l2_dat::armorgrp::body_part::HAIR)
        };
        let to_entry = |item: &killer_ui::inventory::InventoryItemView| killer_ui::item_grid::GridEntry {
            object_id: item.object_id,
            item_id: item.item_id,
            count: item.count,
            enchant: item.enchant,
            icon: item.icons.first().cloned().unwrap_or_default(),
            name: item.name.clone(),
            add_name: item.add_name.clone(),
            description: item.description.clone(),
            weight: item.weight,
        };
        let entries: Vec<_> = self
            .inventory
            .listed()
            .into_iter()
            .chain(self.inventory.equipped())
            .filter(|item| !excluded(item))
            .map(to_entry)
            .collect();
        self.enchant.set_entries(entries);
        let base = self.sysstrings.get(&killer_ui::enchant_wnd::TITLE_SYSSTRING).cloned().unwrap_or_default();
        let name = scroll_item_id
            .and_then(|id| u32::try_from(id).ok())
            .and_then(|id| tables.and_then(|t| t.itemname.get(&id)))
            .map(|item| item.name.clone())
            .unwrap_or_default();
        self.enchant.set_scroll(&base, &name);
    }

    pub fn macros_event(&mut self, event: &WorldEvent, world: &World, desktop: &mut Desktop) {
        let (skillname, commandname, action_commands) = (&self.skillname, &self.commandname, &self.action_commands);
        self.macros.world_event(event, world, |command| macro_line(command, skillname, commandname, action_commands), desktop);
        if let WorldEvent::SendMacroList(list) = event {
            let calls = if list.total_count > 0 && !list.macros.is_empty() { list.macros.len() } else { 1 };
            for _ in 0..calls {
                if self.macro_list_receiving {
                    self.macro_list_received = 0;
                    if list.total_count == 0 && list.macros.is_empty() {
                        self.update_macro_shortcuts(world, list.revision, list.flag);
                        return;
                    }
                    self.macro_list_receiving = false;
                }
                self.macro_list_received += 1;
                if self.macro_list_received >= i32::from(list.total_count) {
                    self.macro_list_receiving = true;
                    self.update_macro_shortcuts(world, list.revision, list.flag);
                }
            }
        }
    }

    fn update_macro_shortcuts(&mut self, world: &World, macro_id: i32, mode: u8) {
        if mode != MACRO_UPDATE_DELETE {
            return;
        }
        let doomed: Vec<i32> = world
            .shortcuts
            .iter()
            .filter(|shortcut| matches!(shortcut.kind, ShortcutKind::Macro { id } if id == macro_id))
            .map(|shortcut| shortcut.slot)
            .collect();
        for slot in doomed {
            self.send(NetCommand::ShortcutDel { slot });
        }
    }

    pub fn manor_event(&mut self, event: &WorldEvent, tables: Option<&DatTables>, desktop: &mut Desktop) {
        if self.manor_setting.world_event(event, tables, desktop) {
            return;
        }
        use killer_ui::manor_info::{ManorRow, ManorTable, WINDOW_ID};
        let item_name = |id: i32| u32::try_from(id).ok().and_then(|id| tables.and_then(|tables| tables.itemname.get(&id))).map(|item| item.name.clone()).unwrap_or_default();
        let lookup = |id: u32| self.sysstrings.get(&id).cloned();
        let shown = desktop.window(WINDOW_ID).is_some_and(|window| window.visible);
        match event {
            WorldEvent::SendManorList(list) => self.manor_info.set_castles(list.entries.iter().map(|entry| entry.id).collect()),
            WorldEvent::ShowSeedInfo(info) => {
                let table = ManorTable {
                    flag: info.flag == 1,
                    state: info.values[0],
                    rows: info.entries.iter().map(|row| ManorRow { item: row.values[0], numbers: row.values[1..].to_vec(), first: row.first.clone(), second: row.second.clone() }).collect(),
                };
                self.manor_info.show_seed(&table, &item_name, &lookup, shown);
            }
            WorldEvent::ShowCropInfo(info) => {
                let table = ManorTable {
                    flag: info.flag == 1,
                    state: info.values[0],
                    rows: info.entries.iter().map(|row| {
                        let mut numbers = row.values[1..].to_vec();
                        numbers.push(i32::from(row.flag as i8));
                        numbers.push(row.value);
                        ManorRow { item: row.values[0], numbers, first: row.first.clone(), second: row.second.clone() }
                    }).collect(),
                };
                self.manor_info.show_crop(&table, &item_name, &lookup);
            }
            WorldEvent::ShowManorDefaultInfo(info) => {
                let table = ManorTable {
                    flag: info.flag == 1,
                    state: 0,
                    rows: info.entries.iter().map(|row| ManorRow { item: row.values[0], numbers: row.values[1..].to_vec(), first: row.first.clone(), second: row.second.clone() }).collect(),
                };
                self.manor_info.show_default(&table, &item_name);
            }
            _ => return,
        }
        if matches!(event, WorldEvent::ShowSeedInfo(_) | WorldEvent::ShowCropInfo(_) | WorldEvent::ShowManorDefaultInfo(_)) {
            set_visible(desktop, WINDOW_ID, true);
            if let Some(index) = desktop.windows.iter().position(|window| window.id == WINDOW_ID) {
                let window = desktop.windows.remove(index);
                desktop.windows.push(window);
            }
        }
    }

    pub fn world_event(&mut self, event: &WorldEvent, desktop: &mut Desktop, world: &World) {
        if matches!(event, WorldEvent::QuestList(_)) {
            self.quest_dirty = true;
        }
        self.siege.world_event(event, world, desktop);
        if matches!(event, WorldEvent::CastleSiegeInfo(_)) {
            self.pending.push(UiCommand::InterfaceSound("itemsound.window_open"));
        }
        if let WorldEvent::SsqStatus(status) = event { crate::ui_ssq::receive(&mut self.ssq, status, desktop); }
        self.shortcut_item_refresh(event);
        self.recipes.world_event(event);
        self.pet.world_event(event, desktop);
        // `ChooseInventoryItem` abre a janela de encantar; o `EnchantResult` só a fecha
        // (`NConsoleWnd::OnEnchantResult @ 0x100667f0` chama `Hide` e mais nada).
        match event {
            WorldEvent::ChooseInventoryItem(_) => {
                self.enchant.grid.selected = None;
                self.enchant.pressed = None;
                self.enchant_refill = true;
                set_visible(desktop, WindowId::ITEM_ENCHANT, true);
            }
            WorldEvent::EnchantResult(_) => self.hide_enchant(desktop),
            _ => {}
        }
        match event {
            WorldEvent::MultiPartyChannelInfo(info) => self.command_channel.set_info(&ChannelInfo {
                owner: info.name.clone(),
                members: info.values[0],
                parties: info.values[1],
                rows: info.members.iter().map(|member| (member.name.clone(), member.value)).collect(),
            }),
            WorldEvent::PetStatusShow(_) => self.sound(interface_sound::WINDOW_OPEN),
            WorldEvent::PetDelete(_) => self.sound(interface_sound::WINDOW_CLOSE),
            WorldEvent::OpenMpcc => {
                set_visible(desktop, command_channel::WINDOW_ID, true);
                self.command_channel.show();
            }
            WorldEvent::CloseMpcc => {
                set_visible(desktop, command_channel::WINDOW_ID, false);
                self.command_channel.close();
            }
            _ => {}
        }
        self.party_room_event(event, desktop, world);
        if let WorldEvent::PartyMatchList(list) = event {
            self.party_match.receive(list.flag, &list.rooms.iter().map(|room| RoomData {
                id: room.id,
                title: room.title.clone(),
                location: room.location,
                min_level: room.min_level,
                max_level: room.max_level,
                members: room.members,
                max_members: room.max_members,
                owner_name: room.owner_name.clone(),
            }).collect::<Vec<_>>());
            self.minimized.sync_owner(desktop);
            self.minimized.hide_match();
            set_visible(desktop, WindowId::PARTY_MATCH, true);
            desktop.bring_to_front(WindowId::PARTY_MATCH);
        }
        if let WorldEvent::ItemList(list) = event {
            self.inventory.set_first_row(0);
            if list.show_window {
                set_visible(desktop, WindowId::INVENTORY, true);
            }
        }
        if let WorldEvent::SkillList(_) = event {
            self.skills.set_first_row(0);
        }
        if let WorldEvent::ShowMinimap(map) = event {
            self.minimap.period = i32::from(map.flag);
            let visible = desktop.window(WindowId::MAP_COVER).is_some_and(|w| w.visible);
            set_visible(desktop, WindowId::MAP_COVER, !visible);
            if !visible {
                self.minimap_recenter = true;
            }
        }
    }

    pub fn sync(&mut self, world: &World, tables: Option<&DatTables>, bar: &mut ShortcutView, now: Instant) {
        self.train.view.player_sp = world.me.as_ref().map(|_| world.stats.sp);
        self.train.view.player_exp = world.me.as_ref().and_then(|me| killer_ui::train::enchant_available_exp(me.level, me.exp));
        self.sync_inventory(world, tables);
        self.sync_skills(world, tables, now);
        self.sync_bar(world, tables, bar, now);
        self.sync_auto(world, tables, bar, now);
        self.sync_pledge(world, tables);
        self.sync_actions(world);
        self.target_player = player_target(world).map(|target| target.name.clone());
        self.sync_quests(world, tables);
        self.refresh_quest_counts(world);
        self.sync_minimap(world, now);
        self.pet.sync(world, tables);
        if self.pet.view.actions.is_empty() {
            self.pet.view.set_actions(&self.actions.actions);
        }
        if self.pet.summoned.actions.is_empty() {
            self.pet.summoned.set_actions(&self.actions.actions);
        }
        if let Some(id) = self.macros.run.take() {
            self.run_macro(world, id);
        }
        if let Some((id, name)) = self.macros.confirm_delete.take() {
            self.pending.push(UiCommand::ConfirmDeleteMacro { id, name });
        }
        // O original preenche a grade de encantar só no abridor (`0x100666c0` chama o
        // `FUN_10099370` uma vez); refazer isso a cada quadro apagava a escolha e a rolagem,
        // porque o `ItemGrid::clear` zera as duas.
        if std::mem::take(&mut self.enchant_refill) {
            self.sync_enchant(tables, world.windows.chosen_item_id);
        }
        self.tick_macro(world, now);
        self.abnormal = crate::ui2d::abnormal_view(world, tables, now);
    }

    fn zone_map_at(&mut self, world: (i32, i32, i32)) -> usize {
        let id = self.zones.lookup((world.0 as f32, world.1 as f32, world.2 as f32)).map_or(-1, |zone| zone.id as i32);
        minimap::zone_map(id)
    }

    fn minimap_action(&mut self, button: MinimapButton, world: &World) {
        let player = world.me.as_ref().map(|me| (me.position.x as i32, me.position.y as i32, me.position.z as i32));
        match button {
            MinimapButton::CurrentLoc => {
                if let Some(player) = player {
                    let map = self.zone_map_at(player);
                    self.minimap.center_world(player, map, true);
                }
            }
            MinimapButton::TargetLoc => {
                if let Some(mark) = &self.minimap.quest_target {
                    let point = (mark.world.0, mark.world.1, 0);
                    let map = self.zone_map_at(point);
                    self.minimap.center_world(point, map, true);
                }
            }
            MinimapButton::PartyMember => {
                let count = self.minimap.party.len();
                if count == 0 {
                    return;
                }
                self.party_mark = (self.party_mark + 1) % count;
                self.minimap.highlight = Some(self.party_mark);
                self.minimap.highlight_ms = minimap::PARTY_HIGHLIGHT_MS;
                let mark = self.minimap.party[self.party_mark].clone();
                let map = self.zone_map_at(mark.world);
                self.minimap.center_world(mark.world, map, true);
            }
            MinimapButton::ZoomIn(index) => {
                if let Some(player) = player {
                    self.minimap.zoom_in(index, (player.0, player.1));
                }
            }
            MinimapButton::ZoomOut => {
                if let Some(player) = player {
                    self.minimap.zoom_out(player);
                }
            }
        }
    }

    fn sync_minimap(&mut self, world: &World, now: Instant) {
        let me = world.me.as_ref();
        let map = self.minimap.map;
        self.minimap.player_name = me.map(|me| me.name.clone()).unwrap_or_default();
        self.minimap.player = me.map(|me| minimap::player_pos(map, me.position.x as i32, me.position.y as i32, me.position.z as i32));
        self.minimap.targets = world.windows.radar.map_points.iter().map(|p| (p[0], p[1])).collect();
        self.minimap.quest_target = self.quest.refresh_target_painted(self.quest_painted.take()).map(|target| minimap::QuestMark {
            world: (target.loc[0] as i32, target.loc[1] as i32, target.loc[2] as i32),
            name: target.name,
            title: target.title,
            id: target.id,
        });
        self.minimap.party = world
            .party
            .positions
            .iter()
            .map(|(id, p)| minimap::PartyMark {
                id: *id,
                world: (p[0], p[1], p[2]),
                in_window: world.party.members.iter().any(|m| m.object_id == *id),
                name: world.party.members.iter().find(|m| m.object_id == *id).map(|m| m.name.clone()).unwrap_or_default(),
            })
            .collect();
        self.minimap.hour = world.clock.map(|clock| clock.hour(now));
        if std::mem::take(&mut self.minimap_recenter)
            && let Some(me) = me
        {
            let position = (me.position.x as i32, me.position.y as i32, me.position.z as i32);
            let id = self.zones.lookup((me.position.x, me.position.y, me.position.z)).map_or(-1, |zone| zone.id as i32);
            self.minimap.show(id, position);
        }
    }

    fn sync_actions(&mut self, world: &World) {
        self.actions.context = killer_ui::actions::ActionContext {
            class_id: world.me.as_ref().and_then(|me| match me.kind {
                killer_world::world::EntityKind::Player { class_id, .. } => Some(class_id),
                _ => None,
            }),
            in_world: world.me.is_some(),
            riding: world.me.as_ref().is_some_and(|me| me.mount.is_some()),
            pet: world.pet.as_ref().map(|pet| killer_ui::actions::ActionPet { object_id: pet.info.object_id, mountable: pet.info.mountable }),
        };
    }

    fn sync_quests(&mut self, world: &World, tables: Option<&DatTables>) {
        if !self.quest_dirty || tables.is_none() {
            return;
        }
        self.quest_dirty = false;
        let mut quests = Vec::new();
        for state in &world.windows.quests {
            if let Some(node) = quest_node(tables, state.quest_id, state.cond) {
                quests.push(node);
            }
        }
        self.quest.quests = quests;
        self.quest.selected = None;
        self.quest.apply_marked();
    }

    fn refresh_quest_counts(&mut self, world: &World) {
        let Some((q, s)) = self.quest.current_step_index() else { return };
        let Some(step) = self.quest.quests.get_mut(q).and_then(|quest| quest.steps.get_mut(s)) else { return };
        for item in &mut step.items {
            item.current = world.inventory.items.iter().filter(|owned| owned.item_id == item.item_id).map(|owned| owned.count).sum();
        }
    }

    fn sync_pledge(&mut self, world: &World, tables: Option<&DatTables>) {
        let place = |id: i32| match tables {
            Some(tables) => u32::try_from(id).ok().and_then(|id| tables.castlename.get(&id)).map(|castle| castle.name.clone()).unwrap_or_default(),
            None => id.to_string(),
        };
        if std::mem::take(&mut self.pledge_show_pending)
            && let Some(rights) = pledge_rights(world)
        {
            self.pledge.show(&rights);
        }
        let pledge = &world.pledge_window;
        let info = pledge.info.as_ref().filter(|_| world.social.clan_id > 0);
        self.pledge.name = info.map(|i| i.name.clone());
        self.pledge.leader = info.map(|i| i.leader_name.clone()).unwrap_or_default();
        self.pledge.level = info.map_or(0, |i| i.level);
        self.pledge.dissolving = info.is_some_and(|i| i.dissolving == killer_ui::pledge::DISSOLVING_STATE);
        self.pledge.at_war = info.is_some_and(|i| i.at_war);
        self.pledge.castle = info.filter(|i| i.castle_id > 0).map(|i| place(i.castle_id));
        self.pledge.hideout = info.filter(|i| i.hideout_id > 0).map(|i| place(i.hideout_id));
        let incoming = pledge
            .members
            .iter()
            .enumerate()
            .map(|(i, m)| MemberRow { name: m.name.clone(), level: m.level, class_id: m.class_id, online: pledge.listed_online(i), object_id: m.online_object_id })
            .collect();
        let reset = self.pledge_list_reset != pledge.list_reset;
        self.pledge_list_reset = pledge.list_reset;
        self.pledge.apply_members(incoming, reset);
    }

    pub fn sync_gm_quest(&mut self, info: &l2_net::game::GmViewQuestInfo, tables: Option<&DatTables>) {
        self.gm_quest.reset();
        for &(quest_id, state) in &info.first {
            let known = tables.is_some_and(|t| (1..=killer_ui::quest::STATE_BITS).any(|state| t.questname.contains_key(&(quest_id, state))));
            if known {
                self.gm_quest.add_quest(quest_node(tables, quest_id, state));
            }
        }
        for item in &info.second {
            self.gm_quest.add_item(item[1], item[2]);
        }
    }

    pub fn sync_gm_skills(&mut self, entries: &[(bool, i32, i32)], clear: bool, tables: Option<&DatTables>) {
        if clear {
            self.gm_skills.clear();
        }
        for &(passive, level, id) in entries {
            let key = (id as u32, level as u32);
            self.gm_skills.add(SkillView {
                id,
                level,
                passive,
                icon: tables.and_then(|t| t.skillgrp.get(&key)).map(|s| s.icon.clone()).unwrap_or_default(),
                name: tables.and_then(|t| t.skillname.get(&key)).map(|n| n.name.clone()).unwrap_or_default(),
                description: tables.and_then(|t| t.skillname.get(&key)).map(|n| n.description.clone()).unwrap_or_default(),
                toggled: false,
                cool: None,
            });
        }
    }

    pub fn sync_gm_status(&mut self, info: &l2_net::game::GmViewCharacterInfo, clan: (Option<String>, i32, Option<String>)) {
        let v = &info.values;
        let status = killer_ui::detail_status::DetailStatusView {
            name: info.name.clone(),
            nick: info.title.clone(),
            karma: v[62],
            clan_name: clan.0,
            clan_crest_id: clan.1,
            clan_crest: clan.2,
            level: v[3],
            class_id: info.final_values[0].max(0) as u32,
            hp: (v[12], v[11]),
            mp: (v[14], v[13]),
            cp: (info.final_values[2], info.final_values[1]),
            exp: i64::from(v[4]),
            sp: v[15],
            load: v[16],
            max_load: v[17],
            p_atk: v[51],
            p_def: v[53],
            accuracy: v[55],
            critical: v[56],
            atk_spd: v[52],
            m_atk: v[57],
            m_def: v[60],
            evasion: v[54],
            speed: 0,
            cast_spd: v[58],
            stats: [v[5], v[6], v[7], v[8], v[9], v[10]],
            stat_bonus: [0; 6],
            pvp_kills: info.kills[1],
            pk_kills: info.kills[0],
            recom_have: i32::from(info.short_values[1]),
            recom_left: i32::from(info.short_values[0]),
            hero: false,
            noble: false,
        };
        self.gm_status.set(status, [v[63], v[64], v[65], v[66], v[69], v[70]], info.floats[0]);
    }

    pub fn sync_gm_hennas(&mut self, info: &l2_net::game::GmHennaInfo, tables: Option<&DatTables>) {
        self.gm_inventory.clear_hennas();
        for &(symbol_id, usable) in &info.entries {
            let Some(henna) = tables.and_then(|t| u32::try_from(symbol_id).ok().and_then(|id| t.hennagrp.get(&id))) else { continue };
            self.gm_inventory.add_henna(henna.icon.trim(), usable != 0);
        }
    }

    pub fn sync_gm_inventory(&mut self, list: &l2_net::game::GmViewItemList, tables: Option<&DatTables>) {
        let entry = |item_id: i32| tables.and_then(|t| u32::try_from(item_id).ok().and_then(|id| t.itemname.get(&id)));
        self.gm_inventory.clear();
        self.gm_inventory.limit = list.header_value;
        for item in list.items.iter().filter(|item| in_item_table(tables, item.values[1])) {
            let [object_id, item_id, count] = item.values;
            let icon = tables.and_then(|t| u32::try_from(item_id).ok().and_then(|id| t.item_icons.get(id)));
            self.gm_inventory.add_item(InventoryItemView {
                object_id,
                item_id,
                count,
                enchant: i32::from(item.tail[0]),
                body_part: item.value,
                equipped: item.shorts[2] != 0,
                quest: item.shorts[0] == QUEST_TYPE2,
                icons: icon.map(|icon| icon.icons.iter().map(|s| s.trim().to_string()).collect()).unwrap_or_default(),
                name: entry(item_id).map(|n| n.name.clone()).unwrap_or_default(),
                popup: entry(item_id).map_or(0, |n| n.popup),
                recipe: false,
                crystallizable: false,
                add_name: entry(item_id).map(|n| n.add_name.clone()).unwrap_or_default(),
                description: entry(item_id).map(|n| n.description.clone()).unwrap_or_default(),
                weight: icon.map_or(0, |i| i.weight as i32),
                tip: tables.map(|t| item_tip_data(t, item_id, i32::from(item.shorts[0]), item.value, i32::from(item.shorts[1]), i32::from(item.tail[1]))),
            });
        }
    }

    fn sync_inventory(&mut self, world: &World, tables: Option<&DatTables>) {
        self.macros.builder = world.builder_level != 0;
        let tables_id = tables.map_or(0, |t| std::ptr::from_ref(t) as usize);
        let unchanged = self.inventory_seen.as_ref().is_some_and(|(items, id)| *id == tables_id && *items == world.inventory.items);
        if !unchanged {
            self.inventory_seen = Some((world.inventory.items.clone(), tables_id));
            self.rebuild_inventory(world, tables);
        }
        self.sync_inventory_state(world, tables);
    }

    fn rebuild_inventory(&mut self, world: &World, tables: Option<&DatTables>) {
        let icons = |item_id: i32| -> Vec<String> {
            tables
                .and_then(|t| u32::try_from(item_id).ok().and_then(|id| t.item_icons.get(id)))
                .map(|icon| icon.icons.iter().map(|s| s.trim().to_string()).collect())
                .unwrap_or_default()
        };
        let previous: Vec<i32> = self.inventory.items.iter().filter(|item| !item.equipped).map(|item| item.object_id).collect();
        self.inventory.items = world
            .inventory
            .items
            .iter()
            .filter(|item| in_item_table(tables, item.item_id))
            .map(|item| InventoryItemView {
                tip: tables.map(|t| item_tip_data(t, item.item_id, i32::from(item.type2), item.body_part, i32::from(item.custom_type1), i32::from(item.custom_type2))),
                object_id: item.object_id,
                item_id: item.item_id,
                count: item.count,
                enchant: i32::from(item.enchant),
                body_part: item.body_part,
                equipped: item.equipped,
                quest: item.type2 == QUEST_TYPE2,
                icons: icons(item.item_id),
                name: tables
                    .and_then(|t| u32::try_from(item.item_id).ok().and_then(|id| t.itemname.get(&id)))
                    .map(|n| n.name.clone())
                    .unwrap_or_default(),
                popup: tables
                    .and_then(|t| u32::try_from(item.item_id).ok().and_then(|id| t.itemname.get(&id)))
                    .map_or(0, |n| n.popup),
                recipe: tables
                    .and_then(|t| u32::try_from(item.item_id).ok().and_then(|id| t.etcitemgrp.get(&id)))
                    .is_some_and(|e| e.family == ETCITEM_TYPE_RECIPE),
                crystallizable: tables.is_some_and(|t| u32::try_from(item.item_id).is_ok_and(|id| t.item_crystallizable(id))),
                add_name: tables
                    .and_then(|t| u32::try_from(item.item_id).ok().and_then(|id| t.itemname.get(&id)))
                    .map(|n| n.add_name.clone())
                    .unwrap_or_default(),
                description: tables
                    .and_then(|t| u32::try_from(item.item_id).ok().and_then(|id| t.itemname.get(&id)))
                    .map(|n| n.description.clone())
                    .unwrap_or_default(),
                weight: tables
                    .and_then(|t| u32::try_from(item.item_id).ok().and_then(|id| t.item_icons.get(id)))
                    .map_or(0, |i| i.weight as i32),
            })
            .collect();
        let world_index: HashMap<i32, usize> = world.inventory.items.iter().enumerate().map(|(index, item)| (item.object_id, index)).collect();
        self.inventory.items.sort_by_key(|item| {
            let known = previous.iter().position(|id| *id == item.object_id);
            let fresh = world_index.get(&item.object_id).copied().unwrap_or(0);
            match (item.equipped, item.quest, known) {
                (true, _, _) => (0, 0),
                (false, false, None) if previous.is_empty() => (2, fresh),
                (false, false, None) => (1, usize::MAX - fresh),
                (false, false, Some(at)) => (2, at),
                (false, true, Some(at)) => (3, at),
                (false, true, None) => (4, fresh),
            }
        });
        if previous.is_empty() {
            self.apply_item_order();
        }
    }

    pub fn track_inventory(&mut self, visible: bool) {
        if visible == self.inventory_shown {
            return;
        }
        self.inventory_shown = visible;
        if visible {
            self.read_item_order();
            self.apply_item_order();
        } else {
            self.save_item_order();
        }
    }

    fn item_page(&self) -> Vec<usize> {
        self.inventory.items.iter().enumerate().filter(|(_, item)| !item.equipped && !item.quest).map(|(index, _)| index).collect()
    }

    fn read_item_order(&mut self) {
        if self.character_name.is_empty() {
            return;
        }
        let file = killer_ui::item_order::file_name(&self.character_name);
        self.item_order = killer_platform::read_item_order(&file).map(|bytes| killer_ui::item_order::decode(&bytes)).unwrap_or_default();
    }

    fn apply_item_order(&mut self) {
        let positions = self.item_page();
        if positions.is_empty() || self.item_order.is_empty() {
            return;
        }
        let mut page: Vec<InventoryItemView> = positions.iter().map(|&index| self.inventory.items[index].clone()).collect();
        killer_ui::item_order::restore(&mut page, |item| item.item_id, &self.item_order);
        for (&index, item) in positions.iter().zip(page) {
            self.inventory.items[index] = item;
        }
    }

    fn save_item_order(&mut self) {
        let positions = self.item_page();
        if positions.is_empty() || self.character_name.is_empty() {
            return;
        }
        self.item_order = positions.iter().map(|&index| self.inventory.items[index].item_id).collect();
        let file = killer_ui::item_order::file_name(&self.character_name);
        if let Err(error) = killer_platform::write_item_order(&file, &killer_ui::item_order::encode(&self.item_order)) {
            log::warn!("ordem do inventário: {error}");
        }
    }

    fn sync_inventory_state(&mut self, world: &World, tables: Option<&DatTables>) {
        self.character_name = world.me.as_ref().map(|me| me.name.clone()).unwrap_or_default();
        self.inventory.adena = i64::from(world.inventory.adena());
        self.inventory.load = world.stats.load;
        self.inventory.max_load = world.stats.max_load;
        self.inventory.limit = world.inventory_limit;
        self.inventory.paperdoll_ids = PaperdollIds {
            r_ear: world.paperdoll[PAPERDOLL_R_EAR].object_id,
            l_ear: world.paperdoll[PAPERDOLL_L_EAR].object_id,
            r_finger: world.paperdoll[PAPERDOLL_R_FINGER].object_id,
            l_finger: world.paperdoll[PAPERDOLL_L_FINGER].object_id,
        };
        self.inventory.dwarven_craft = world.social.dwarven_craft;
        // A coluna de símbolos do `NCEquipItemWnd`: as molduras vêm dos espaços da classe
        // (`0x10074500`), não do campo `slots` do pacote, e os ícones do `symbol_icon` do
        // `hennagrp` pela `symbol_id` (`0x10098880` chama `GetHennaData` e descarta o que não
        // existir). Ver `docs/evidence/ui-henna.md` §7.
        self.inventory.henna_slots = world
            .me
            .as_ref()
            .and_then(|me| match &me.kind {
                killer_world::EntityKind::Player { class_id, .. } => killer_ui::inventory::henna_slots(*class_id),
                _ => None,
            })
            .unwrap_or(0);
        self.inventory.henna = world
            .henna
            .info
            .as_ref()
            .map(|info| {
                info.symbols
                    .iter()
                    .filter_map(|(symbol_id, dye_item_id)| {
                        let henna = tables?.hennagrp.get(&u32::try_from(*symbol_id).ok()?)?;
                        Some(killer_ui::inventory::HennaMark {
                            symbol_id: *symbol_id,
                            dye_item_id: *dye_item_id,
                            icon: henna.icon.clone(),
                            name: henna.name.clone(),
                            description: henna.description.clone(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.inventory.update_rows();
    }

    fn sync_cooldowns(&mut self, world: &World, now: Instant) {
        if world.cooldowns != self.cooldown_seen {
            self.cooldown_seen = world.cooldowns.clone();
            for reuse in world.cooldowns.iter().filter(|reuse| reuse.reuse > 0 && reuse.reuse >= reuse.remaining) {
                self.cooldowns.insert(
                    reuse.skill_id,
                    Cooldown { level: reuse.level, started: now, reuse: reuse.reuse as f32, remaining: reuse.remaining as f32, flash: None, stopped: false },
                );
            }
        }
        let ended: Vec<(i32, bool)> = self
            .cooldowns
            .iter()
            .filter(|(_, c)| c.flash.is_none() && !c.stopped)
            .filter_map(|(id, c)| {
                let elapsed = now.duration_since(c.started).as_secs_f64();
                let cool = killer_ui::cooltime::CoolTime { total: f64::from(c.reuse), remaining: f64::from(c.remaining) };
                (elapsed > cool.ends_at()).then_some((*id, cool.flash_window_open(elapsed)))
            })
            .collect();
        for (id, window_open) in ended {
            let painted = (self.skills_visible && self.skills.skills.iter().any(|skill| skill.id == id))
                || (self.bar_visible
                    && world.shortcuts.iter().any(|s| matches!(s.kind, ShortcutKind::Skill { skill_id, .. } if skill_id == id)));
            let Some(cooldown) = self.cooldowns.get_mut(&id) else { continue };
            if !window_open {
                cooldown.stopped = true;
            } else if painted {
                cooldown.flash = Some(now);
                self.sound(killer_ui::cooltime::SOUND_COOL_TIME_END);
            }
        }
        self.cooldowns.retain(|_, c| {
            !c.stopped && c.flash.is_none_or(|at| now.duration_since(at).as_secs_f64() <= killer_ui::cooltime::FLASH_SECONDS)
        });
    }

    pub fn set_cool_time_surfaces(&mut self, skills_visible: bool, bar_visible: bool) {
        self.skills_visible = skills_visible;
        self.bar_visible = bar_visible;
    }

    fn cool_frame(&self, skill_id: i32, level: i32, now: Instant) -> Option<killer_ui::cooltime::CoolFrame> {
        let cooldown = self.cooldowns.get(&skill_id).filter(|cooldown| cooldown.level == level)?;
        if let Some(at) = cooldown.flash {
            return killer_ui::cooltime::CoolTime::flash(now.duration_since(at).as_secs_f64());
        }
        killer_ui::cooltime::CoolTime { total: f64::from(cooldown.reuse), remaining: f64::from(cooldown.remaining) }
            .running(now.duration_since(cooldown.started).as_secs_f64())
    }

    fn is_toggle_skill(tables: Option<&DatTables>, skill_id: i32, level: i32) -> bool {
        tables.and_then(|t| t.skillgrp.get(&(skill_id as u32, level as u32))).map(|s| s.oper_type) == Some(TOGGLE_OPERATE_TYPE)
    }

    fn toggle_skill(tables: Option<&DatTables>, world: &World, skill_id: i32, level: i32) -> bool {
        Self::is_toggle_skill(tables, skill_id, level) && world.buffs.iter().any(|b| b.skill_id == skill_id)
    }

    fn cooldown_fraction(&self, skill_id: i32, level: i32, now: Instant) -> Option<f32> {
        let cooldown = self.cooldowns.get(&skill_id).filter(|cooldown| cooldown.level == level)?;
        let left = cooldown.remaining - now.duration_since(cooldown.started).as_secs_f32();
        (left > 0.0 && cooldown.reuse > 0.0).then(|| (left / cooldown.reuse).clamp(0.0, 1.0))
    }

    fn sync_skills(&mut self, world: &World, tables: Option<&DatTables>, now: Instant) {
        self.sync_cooldowns(world, now);
        let tables_id = tables.map_or(0, |t| std::ptr::from_ref(t) as usize);
        let unchanged = self.skills_seen.as_ref().is_some_and(|(skills, id)| *id == tables_id && *skills == world.skills);
        if unchanged {
            let mut views = std::mem::take(&mut self.skills.skills);
            for view in &mut views {
                view.toggled = Self::toggle_skill(tables, world, view.id, view.level);
                view.cool = self.cool_frame(view.id, view.level, now);
            }
            self.skills.skills = views;
            self.skills.update_rows();
            return;
        }
        self.skills_seen = Some((world.skills.clone(), tables_id));
        self.skills.skills = world
            .skills
            .iter()
            .map(|entry| {
                let key = (entry.id as u32, entry.level as u32);
                SkillView {
                    id: entry.id,
                    level: entry.level,
                    passive: entry.passive,
                    icon: tables.and_then(|t| t.skillgrp.get(&key)).map(|s| s.icon.clone()).unwrap_or_default(),
                    name: tables.and_then(|t| t.skillname.get(&key)).map(|n| n.name.clone()).unwrap_or_default(),
                    description: tables.and_then(|t| t.skillname.get(&key)).map(|n| n.description.clone()).unwrap_or_default(),
                    toggled: Self::toggle_skill(tables, world, entry.id, entry.level),
                    cool: self.cool_frame(entry.id, entry.level, now),
                }
            })
            .collect();
        self.skills.update_rows();
    }

    fn sync_bar(&mut self, world: &World, tables: Option<&DatTables>, bar: &mut ShortcutView, now: Instant) {
        let delta = self.bar_ticked.map_or(0, |last| now.saturating_duration_since(last).as_millis().min(u128::from(u32::MAX)) as u32);
        self.bar_ticked = Some(now);
        bar.clear();
        for shortcut in &world.shortcuts {
            let Ok(slot) = usize::try_from(shortcut.slot) else { continue };
            let entry = match shortcut.kind {
                ShortcutKind::Item { object_id } if self.removed_items.contains(&object_id) => continue,
                ShortcutKind::Item { object_id } if world.inventory.find(object_id).or_else(|| world.pet_inventory.find(object_id)).is_none() => continue,
                ShortcutKind::Item { object_id } => {
                    let item = world.inventory.find(object_id).or_else(|| world.pet_inventory.find(object_id));
                    let icon = item
                        .and_then(|i| {
                            tables.and_then(|t| u32::try_from(i.item_id).ok().and_then(|id| t.item_icons.texture(id)))
                        })
                        .unwrap_or_default();
                    let mut entry = ShortcutSlot::new(SlotKind::Item { object_id }, &icon);
                    entry.item_id = item.map_or(0, |i| i.item_id);
                    entry.soulshot = killer_ui::shortcut::is_soulshot_item(entry.item_id);
                    entry.disabled = world.combat.auto_soulshots.get(&entry.item_id).copied().unwrap_or(false);
                    entry
                }
                ShortcutKind::Skill { skill_id, level } => {
                    let icon = tables
                        .and_then(|t| t.skillgrp.get(&(skill_id as u32, level as u32)))
                        .map(|s| s.icon.clone())
                        .unwrap_or_default();
                    let mut entry = ShortcutSlot::new(SlotKind::Skill { id: skill_id, level }, &icon);
                    entry.toggle_skill = Self::is_toggle_skill(tables, skill_id, level);
                    entry.toggled = Self::toggle_skill(tables, world, skill_id, level);
                    entry.cooldown = self.cooldown_fraction(skill_id, level, now);
                    entry.cool = self.cool_frame(skill_id, level, now);
                    entry
                }
                ShortcutKind::Action { id } => {
                    let icon =
                        self.actions.actions.iter().find(|a| a.id == id).map(|a| a.icon.clone()).unwrap_or_default();
                    ShortcutSlot::new(SlotKind::Action { id }, &icon)
                }
                ShortcutKind::Macro { id } => {
                    let known = world.macros.macros.iter().find(|m| m.id == id);
                    let acronym = known.map(|m| m.acronym.clone()).unwrap_or_default();
                    let icon = format!("{}{}", killer_ui::macros::ICON_PREFIX, known.map_or(1, |m| i32::from(m.icon).clamp(1, 7)));
                    ShortcutSlot { macro_acronym: acronym, ..ShortcutSlot::new(SlotKind::Macro { id }, &icon) }
                }
                ShortcutKind::Recipe { id } => {
                    let recipe = tables.and_then(|t| t.recipes.get(&id));
                    let icon = recipe.and_then(|recipe| tables.and_then(|t| u32::try_from(recipe.product_id).ok().and_then(|id| t.item_icons.texture(id)))).unwrap_or_default();
                    let name = recipe.and_then(|recipe| tables.and_then(|t| u32::try_from(recipe.recipe_id).ok().and_then(|id| t.itemname.get(&id)))).map(|item| item.name.clone()).unwrap_or_default();
                    ShortcutSlot { name, ..ShortcutSlot::new(SlotKind::Recipe { object_id: id }, &icon) }
                }
                ShortcutKind::Other { .. } => continue,
            };
            bar.set(slot, entry);
        }
        bar.tick(delta);
    }

    pub fn dragged_item_id(&self) -> i32 {
        let Some(Press { source: Some(DragSource::Grid { object_id, .. } | DragSource::Paperdoll { object_id, .. }), .. }) = self.press else { return 0 };
        if !self.drag_moved {
            return 0;
        }
        self.inventory.items.iter().find(|item| item.object_id == object_id).map_or(0, |item| item.item_id)
    }

    fn sync_auto(&mut self, world: &World, tables: Option<&DatTables>, bar: &mut ShortcutView, now: Instant) {
        if bar.modern.is_none() {
            return;
        }
        for potion in &mut bar.auto.potions {
            potion.flash = potion.flash.saturating_sub(1);
        }
        let held = |item_id: i32| world.inventory.items.iter().filter(|item| item.item_id == item_id).map(|item| item.count).sum::<i32>();
        let icon = |item_id: i32| {
            tables
                .and_then(|t| u32::try_from(item_id).ok().and_then(|id| t.item_icons.texture(id)))
                .unwrap_or_default()
        };
        let name = |item_id: i32| {
            tables
                .and_then(|t| u32::try_from(item_id).ok().and_then(|id| t.itemname.get(&id)))
                .map(|item| item.name.clone())
                .unwrap_or_default()
        };
        for kind in ShotKind::ALL {
            let shot = &mut bar.auto.shots[kind.index()];
            if held(shot.item_id) < 1 {
                shot.item_id = kind.best(|id| held(id) > 0);
            }
            shot.count = held(shot.item_id);
            shot.icon = icon(shot.item_id);
            shot.name = name(shot.item_id);
            shot.on = world.combat.auto_soulshots.get(&shot.item_id).copied().unwrap_or(false);
        }
        let me = world.me.as_ref();
        for vital in Vital::ALL {
            let values = me.and_then(|me| match vital {
                Vital::Hp => me.hp,
                Vital::Cp => me.cp,
                Vital::Mp => me.mp,
            });
            let slot = vital.index();
            let potion = &mut bar.auto.potions[slot];
            potion.count = held(potion.item_id);
            potion.icon = icon(potion.item_id);
            potion.name = name(potion.item_id);
            if !potion.on || potion.count < 1 {
                continue;
            }
            let Some((value, max)) = values else { continue };
            let alive = me.and_then(|me| me.hp).is_some_and(|(hp, _)| hp > 0);
            if !alive || max <= 0 || i64::from(value) * 100 >= i64::from(max) * i64::from(potion.percent) {
                continue;
            }
            if self.auto_potion_at[slot].is_some_and(|at| now.duration_since(at) < AUTO_POTION_INTERVAL) {
                continue;
            }
            if let Some(item) = world.inventory.items.iter().find(|item| item.item_id == potion.item_id) {
                self.auto_potion_at[slot] = Some(now);
                bar.auto.potions[slot].flash = killer_ui::shortcut::AUTO_FLASH_FRAMES;
                self.send(NetCommand::UseItem { object_id: item.object_id, ctrl: false });
            }
        }
    }

    pub fn drain_bar(&mut self, bar: &mut ShortcutView, world: &World) {
        for event in bar.take_events() {
            match event {
                BarEvent::Use(slot) => self.use_shortcut(world, slot),
                BarEvent::Sound(_) => {}
                BarEvent::Register { kind, slot, id } => {
                    self.send(NetCommand::ShortcutReg { kind, slot: slot as i32, id, character_type: 1 })
                }
                BarEvent::Delete(slot) => self.send(NetCommand::ShortcutDel { slot: slot as i32 }),
                BarEvent::AutoSoulShot { object_id, enable } => {
                    if let Some(item) = world.inventory.find(object_id) {
                        self.send(NetCommand::AutoSoulShot { item_id: item.item_id, enable });
                    }
                }
                BarEvent::AutoShot { item_id, enable } => self.send(NetCommand::AutoSoulShot { item_id, enable }),
                BarEvent::PageChanged(_) => {}
            }
        }
    }

    fn use_action(&mut self, world: &World, id: i32, ctrl: bool, shift: bool) {
        let player = world.targeted().filter(|target| matches!(target.kind, EntityKind::Player { .. }));
        match id {
            2 => self.attack_current_target(world, ctrl, shift),
            3 => {
                if let Some(target) = world.targeted().filter(|target| !matches!(target.kind, EntityKind::Item { .. })) {
                    self.send(NetCommand::Trade { object_id: target.object_id });
                }
            }
            4 => self.target_next(world),
            5 => self.pick_up(world),
            6 => {
                if let Some(object_id) = world.assist_target() {
                    self.action_on(world, object_id);
                }
            }
            7 | 9 | 50 | 56 => {
                if let Some(target) = player {
                    let name = target.name.clone();
                    self.send(match id {
                        7 => NetCommand::JoinParty { name, item_distribution: self.party_looting },
                        9 => NetCommand::OustPartyMember { name },
                        50 => NetCommand::ChangePartyLeader { name },
                        _ => NetCommand::ExAskJoinMpcc { name },
                    });
                } else {
                    self.pending.push(UiCommand::ColoredSystemMessage {
                        id: match id { 7 => 185, 9 => 187, 50 => 1400, _ => 50 },
                        color: killer_ui::color::GOLD.0,
                    });
                }
            }
            8 => self.send(NetCommand::WithdrawParty),
            10 => self.send(NetCommand::PrivateStoreManageSell),
            11 => {
                self.send(NetCommand::PartyMatchList { level: 1, location: -1, mode: 1 });
                self.party_match.location.selected = 1;
                self.party_match.level.selected = 1;
            }
            12..=14 | 24..=26 | 29..=31 | 33..=35 => {
                if let Some(&action) = self.action_social.get(&id).filter(|&&action| action > 0) {
                    self.send(NetCommand::SocialAction { action });
                }
            }
            18 => self.pet_pick_up(world),
            28 => self.send(NetCommand::PrivateStoreManageBuy),
            40 => match player {
                Some(target) => self.send(NetCommand::VoteSociality { target_id: target.object_id }),
                None => self.pending.push(UiCommand::ColoredSystemMessage {
                    id: if world.targeted().is_some() { 144 } else { 242 },
                    color: killer_ui::color::GOLD.0,
                }),
            },
            55 => self.pending.push(UiCommand::ReplayRecord(ReplayRecord::Toggle)),
            _ => self.send(NetCommand::UseAction { id, ctrl: ctrl || self.ctrl, shift: shift || self.shift }),
        }
    }

    fn use_shortcut(&mut self, world: &World, slot: usize) {
        let Some(shortcut) = world.shortcut(slot as i32) else {
            log::info!("atalho {slot}: nenhum registro em world.shortcuts");
            self.sound(interface_sound::CLICK_FAILED);
            return;
        };
        log::info!("atalho {slot}: {:?}", shortcut.kind);
        match shortcut.kind {
            ShortcutKind::Item { object_id } if shortcut.character_type == SHORTCUT_OWNER_PET => {
                self.send(NetCommand::PetUseItem { object_id })
            }
            ShortcutKind::Item { object_id } => self.use_shortcut_item(world, object_id),
            ShortcutKind::Skill { skill_id, .. } => {
                self.send(NetCommand::UseSkill { skill_id, ctrl: self.ctrl, shift: self.shift })
            }
            ShortcutKind::Action { id } => self.use_action(world, id, self.ctrl, self.shift),
            ShortcutKind::Macro { id } => self.run_macro(world, id),
            ShortcutKind::Recipe { id } => self.send(NetCommand::RecipeItemMakeInfo { recipe_id: id }),
            ShortcutKind::Other { kind } => {
                self.pending.push(UiCommand::Log(format!("atalho de tipo {kind} desconhecido")))
            }
        }
    }

    /// Executa os comandos de um macro em sequência (`SendMacroList`/`Macro.commands`, campo
    /// `kind`). A versão anterior deste método chutava o significado de `kind` a partir de um
    /// cliente C# de terceiros (`gawric/Unity-Client-for-L2J`) sem confirmar contra o `.asm` do
    /// C4 — **estava errada**, achada via teste ao vivo do usuário (macro com "/TARGET nome"
    /// não fazia nada visível, e a skill do mesmo macro parecia ser "o único comando que
    /// funciona"). Desta vez o `.asm` real do stepper foi lido inteiro (`objdump` em
    /// `nwindow.dll`, sem Ghidra):
    /// - `NConsoleWnd::Tick @ 0x1007fa90`: a cada frame, para cada macro com a flag "rodando"
    ///   (`+0xa34 != 0`), chama o stepper `FUN_100ab100(macro_entry, deltaTime)`.
    /// - O stepper (`0x100ab100`) executa **um comando por vez**, não todos de uma vez: o campo
    ///   `+0xf8` (por comando, array em `+0xd8`, stride `0x70`) acumula `deltaTime` até estourar
    ///   um limite por comando (`+0xf0`), só então avança `+0xa48` (índice atual) pro próximo.
    ///   Ordem e atraso real NÃO estão portados aqui — os comandos ainda disparam todos de uma
    ///   vez nesta função, o que é uma aproximação sabidamente diferente do original.
    /// - O `kind` de cada comando (campo em `+0xd8` do array, ligado ao byte `kind` do pacote
    ///   `"ccdcS"`, `ReceiveMacroList @ 0x10409230`) só tem dois ramos explícitos no dispatcher:
    ///   `kind == 1` despacha a skill direto (`skill_id`) — **isso bate** com o que já estava
    ///   portado. `kind == 3` **não é uma "ação" com id numérico** como o oráculo externo
    ///   sugeria — é texto livre: o cliente chama um tokenizer (`FUN_1004bc10`/`FUN_1004bc70`
    ///   @ `0x1004bc10`/`0x1004bc70`) sobre a string do comando (ex.: "/TARGET loqiita"), que
    ///   por sua vez resolve placeholders de nome via uma tabela achada em `0x1023a910`
    ///   (`%target`, `%self`, `%pet`, `%party1`..`%party8`) antes de despachar o comando de
    ///   verdade. Esse tokenizer de texto (o que decide o que "/TARGET", "/DELAY" etc. fazem)
    ///   NÃO está decompilado/entendido ainda — precisa de mais engenharia reversa dedicada.
    ///   `kind == 4` também tem ramo próprio (`0x100ab585`), lido por completo: indexa um array
    ///   com o mesmo layout (stride `0x1cc0`, campo `+0x1b18`) da barra de atalhos (visto em
    ///   `NCShortCutWnd_PaintSlots_10106b10.c`), então a suposição antiga "kind 4 = índice de
    ///   atalho" tinha a ideia certa — mas o que exatamente esse ramo FAZ com a célula achada
    ///   ainda não foi seguido até o fim, então não virou código aqui (evitando reintroduzir o
    ///   mesmo tipo de chute que causou o bug original).
    ///
    /// Por isso, comandos de texto (a maioria dos macros com `/comando args`, exatamente o caso
    /// relatado) ficam com log explícito em vez de disparar uma ação errada (que era o bug real:
    /// o "único comando que funcionava" era sempre a skill de `kind == 1`, e os outros
    /// comandos, silenciosamente mal despachados como `UseAction`, não tinham efeito visível).
    /// Começa a rodar um macro: dispara o primeiro comando na hora e deixa o resto pro
    /// `tick_macro`, que avança um comando por vez. É o que o original faz — `NConsoleWnd::
    /// RunMacro @ 0x10074eb0` só liga a flag "rodando", e o stepper `FUN_100ab100` dispara o
    /// comando 0 no primeiro tick e depois espera o atraso de cada um pra passar pro próximo.
    /// Rodar um macro novo cancela o anterior, como no original (a flag é por macro e o
    /// `FUN_100a7860` zera tudo a cada tecla).
    fn run_macro(&mut self, world: &World, id: i32) {
        let Some(entry) = world.macros.macros.iter().find(|m| m.id == id) else {
            log::info!("macro {id} não está na lista recebida do servidor ({} conhecidos)", world.macros.macros.len());
            return;
        };
        log::info!("macro {id} \"{}\": {} comandos", entry.name, entry.commands.len());
        self.macro_run =
            Some(MacroRun { macro_id: id, index: 0, started: Instant::now(), delay: 0.0, waiting_skill: None });
        self.fire_macro_command(world, 0);
    }

    /// Avança a execução do macro em andamento: enquanto o atraso do comando atual não estoura,
    /// não faz nada; quando estoura, dispara o próximo. Chamado uma vez por frame pelo `sync`,
    /// como o `NConsoleWnd::Tick @ 0x1007fa90` faz com o stepper.
    fn tick_macro(&mut self, world: &World, now: Instant) {
        let Some(run) = self.macro_run.as_ref() else { return };
        if run.waiting_skill.is_some() {
            return;
        }
        if now.saturating_duration_since(run.started).as_secs_f32() < run.delay {
            return;
        }
        let (macro_id, next) = (run.macro_id, run.index + 1);
        let Some(count) = world.macros.macros.iter().find(|m| m.id == macro_id).map(|m| m.commands.len()) else {
            self.macro_run = None;
            return;
        };
        if next >= count {
            self.macro_run = None;
            return;
        }
        self.macro_run =
            Some(MacroRun { macro_id, index: next, started: now, delay: 0.0, waiting_skill: None });
        self.fire_macro_command(world, next);
    }


    /// Dispara o comando de índice `index` do macro em andamento e guarda o atraso dele.
    /// `kind` conferido contra `FUN_100a8120`, que lê o mesmo campo `+0xd8` do comando:
    /// `1` = skill (id no campo `+0xe0`, o nosso `skill_id`), `2` = ação (mesmo campo, id de
    /// `actionname-e.dat` — era isso que o oráculo externo chamava de "3 = action", errado),
    /// `3` = texto livre, `6` = espera pura (`fildl 0xe4(%edi); fstps 0xf0(%edi)` — converte o
    /// inteiro do campo `+0xe4`, o nosso `shortcut_id`, direto pra atraso em segundos).
    /// `4` (atalho) e `5` seguem sem porte: os dois leem campos cujo significado exato ainda não
    /// foi rastreado até o fim.
    fn fire_macro_command(&mut self, world: &World, index: usize) {
        let Some(command) = world
            .macros
            .macros
            .iter()
            .find(|m| Some(m.id) == self.macro_run.as_ref().map(|r| r.macro_id))
            .and_then(|m| m.commands.get(index))
            .cloned()
        else {
            log::info!("macro: comando {index} não existe");
            return;
        };
        log::info!(
            "macro: comando {index} tipo {} skill {} atalho {} texto {:?}",
            command.kind, command.skill_id, command.shortcut_id, command.command
        );
        let delay = match command.kind {
            1 => {
                self.send(NetCommand::UseSkill { skill_id: command.skill_id, ctrl: self.ctrl, shift: self.shift });
                if let Some(run) = self.macro_run.as_mut() {
                    run.waiting_skill = Some(command.skill_id);
                }
                0.0
            }
            2 => {
                self.use_action(world, command.skill_id, self.ctrl, self.shift);
                self.macro_action_delay(world, command.skill_id)
            }
            3 => {
                self.run_macro_text_command(world, &command.command);
                0.0
            }
            // O atraso vai no campo de 4 bytes do comando: o parser do Save grava
            // `índice, 6, <segundos>, 0, ""` (`0x100aaa24`..`0x100aaa3a`).
            6 => command.skill_id.max(0) as f32,
            kind => {
                self.pending.push(UiCommand::Log(format!(
                    "comando de macro \"{}\" (tipo {kind}) ainda não portado",
                    command.command
                )));
                0.0
            }
        };
        if let Some(run) = self.macro_run.as_mut() {
            run.delay = delay;
        }
    }

    /// Atraso de um comando de ação: só as ações que tocam animação social esperam, e o tempo é a
    /// duração da animação (ver `MACRO_ACTION_SOCIAL_ANIM`). "sitstand" escolhe a animação pelo
    /// estado atual — de pé toca sentar (social 0), sentado toca levantar (social 1).
    fn macro_action_delay(&self, world: &World, action: i32) -> f32 {
        let sitting = world.me.as_ref().is_some_and(|me| me.sitting);
        let anim = match action {
            MACRO_ACTION_SITSTAND => Some(usize::from(sitting)),
            _ => MACRO_ACTION_SOCIAL_ANIM.iter().find(|(id, _)| *id == action).map(|(_, anim)| *anim),
        };
        anim.and_then(|anim| self.social_durations.get(anim).copied()).filter(|d| *d > 0.0).unwrap_or(0.0)
    }

    /// Porte do despachante de comandos de texto do cliente original, `FUN_10058330`
    /// (`nwindow.dll`, já decompilado em `docs/evidence/decompiled/ui/FUN_10058330.c`) — é a
    /// mesma função que `NConsoleWnd::ExecuteCommand @ 0x1006d7d0` chama pra comandos digitados
    /// no chat e que o executor de macro chama pra comandos de `kind == 3`.
    ///
    /// O verbo (primeira palavra, sem a `/`) vira um token pela tabela real `commandname-e.dat`
    /// (`self.commandname`, 110 comandos), exatamente como `FL2GameData::GetCommandType`
    /// (`FL2GameData_GetCommandType_1044c310.asm`) faz. O `switch` sobre esse token abaixo segue
    /// caso a caso o do original; cada grupo está anotado com a chamada que ele faz lá.
    ///
    /// Os argumentos são o resto dos tokens com os placeholders de nome resolvidos e juntados
    /// por espaço — o original faz isso mesmo (`FUN_1004bc70` em cada token + `appStrcat`), por
    /// isso nomes com espaço funcionam.
    pub fn chat_command(&mut self, world: &World, line: &str) {
        let mut chars = line.chars();
        chars.next();
        let command = format!("/{}", chars.as_str());
        self.run_macro_text_command(world, &command);
    }

    pub fn chat_system(&mut self, id: u32) {
        self.pending.push(UiCommand::ColoredSystemMessage { id, color: CHAT_WARNING_COLOR });
    }

    pub fn run_macro_text_command(&mut self, world: &World, command: &str) {
        let mut words = command.split_whitespace();
        let Some(verb) = words.next() else { return };
        let key = verb.trim_start_matches('/').to_ascii_lowercase();
        if let Some(toggle_window) = killer_ui::debug_wnd::command(&key) {
            self.pending.push(UiCommand::DebugCommand { window: toggle_window });
            return;
        }
        let Some(token) = self.commandname.get(&key).copied() else {
            self.pending.push(UiCommand::Log(format!(
                "comando de macro \"{command}\" — verbo \"{key}\" desconhecido pro cliente original também"
            )));
            return;
        };
        let args: Vec<&str> = words.collect();
        let name = || -> Option<String> {
            let parts: Vec<String> = args.iter().filter_map(|a| resolve_macro_name_placeholder(world, a)).collect();
            (!parts.is_empty()).then(|| parts.join(" "))
        };
        let target_name = || name().or_else(|| world.target.and_then(|id| world.entity(id)).map(|e| e.name.clone()));

        if let Some((_, action)) = MACRO_ACTION_COMMANDS.iter().find(|(t, _)| *t == token) {
            self.use_action(world, *action, false, false);
            return;
        }
        if let Some((_, social)) = MACRO_SOCIAL_COMMANDS.iter().find(|(t, _)| *t == token) {
            self.send(NetCommand::SocialAction { action: *social });
            return;
        }
        match token {
            // FUN_10068560(0/1): sentar / levantar.
            0x4 => self.send(NetCommand::ChangeWaitType { standing: false }),
            0x5 => self.send(NetCommand::ChangeWaitType { standing: true }),
            // FUN_10068570(0/1): andar / correr.
            0x6 => self.send(NetCommand::ChangeMoveType { running: false }),
            0x7 => self.send(NetCommand::ChangeMoveType { running: true }),
            // attack/attackforce/attackstand: atacam o alvo atual; o original passa a posição do
            // jogador e difere só nos dois últimos flags (força / parado).
            0x8 | 0xa => self.action_on_target(world, token == 0xa),
            0x9 => self.attack_current_target(world, true, false),
            // FUN_100668e0(alvo->0x60): pedido de troca com o alvo atual.
            0xb => {
                if let Some(target) = world.target {
                    self.send(NetCommand::Trade { object_id: target });
                }
            }
            // target (0xc): `FUN_1006a8d0` acha o `User` pelo nome e `FUN_10071960` manda `Action` com o
            // Shift, sem tocar no alvo local (o servidor confirma com `MyTargetSelected`).
            0xc => {
                if let Some(name) = name() {
                    match world.me.iter().chain(world.entities.values()).find(|entity| entity.name.eq_ignore_ascii_case(&name)) {
                        Some(entity) => self.action_on(world, entity.object_id),
                        None => self.pending.push(UiCommand::Log(format!("comando de macro /target: \"{name}\" não está visível"))),
                    }
                }
            }
            // assist (0xf): `FUN_1007a2c0`, o alvo do alvo (ou o objeto estático correspondente).
            0xf => {
                if let Some(object_id) = world.assist_target() {
                    self.action_on(world, object_id);
                }
            }
            // FUN_10071a70(200, alvo_atual): pega o pawn mais próximo dentro do raio, pulando o
            // que já está selecionado.
            0xd => self.target_next(world),
            // FUN_100719e0(200, -1): pega o item no chão mais próximo (no original o passo
            // seguinte é o mesmo pacote `Action` de clicar nele).
            0xe => self.pick_up(world),
            // Invite: convida pelo nome dado, ou pelo alvo atual quando vem sem argumento.
            0x10 => match target_name() {
                Some(name) => self.send(NetCommand::JoinParty { name, item_distribution: self.party_looting }),
                None => self.pending.push(UiCommand::SystemMessage(INVITE_NEEDS_TARGET_SYSMSG)),
            },
            0x11 => self.send(NetCommand::WithdrawParty),
            0x12 => match target_name() {
                Some(name) => self.send(NetCommand::OustPartyMember { name }),
                None => self.pending.push(UiCommand::SystemMessage(SELECT_MEMBER_SYSMSG)),
            },
            // FUN_10069a00() / FUN_10069be0(): abrem a loja privada de venda / de compra.
            0x13 => self.send(NetCommand::PrivateStoreManageSell),
            0x27 => self.send(NetCommand::PrivateStoreManageBuy),
            // FUN_100692c0(): lista de GMs online.
            0x3 => self.send(NetCommand::RequestGmList),
            // allyinvite: convida o ALVO atual (o original empilha o object_id dele e avisa por
            // mensagem de sistema quando não há alvo válido — ids 0x1fe/0x1ff).
            0x1e => match world.target {
                Some(object_id) => {
                    self.pending.push(UiCommand::SystemMessage(ALLY_INVITE_SENT_SYSMSG));
                    self.send(NetCommand::JoinAlly { object_id });
                }
                None => self.pending.push(UiCommand::SystemMessage(ALLY_INVITE_NEEDS_TARGET_SYSMSG)),
            },
            // allyleave/allydissolve: o original abre um diálogo de confirmação antes de mandar
            // qualquer coisa (mensagens 0x200 "Do you really wish to withdraw from the
            // alliance?" e 0x202 "Do you really wish to dissolve the alliance?").
            0x1f => self.dialogs.push(Dialog::AllyConfirm { message_id: ALLY_LEAVE_SYSMSG, dissolve: false }),
            0x21 => self.dialogs.push(Dialog::AllyConfirm { message_id: ALLY_DISSOLVE_SYSMSG, dissolve: true }),
            // partymatching: o original empilha (1, -1, 1) e abre a janela de salas
            // (`FUN_1006e030` em `FUN_10058330`) — a mesma ordem de `RequestPartyMatchList`.
            // (a janela de salas abre sozinha quando a resposta `PartyMatchList` chega, em
            // `world_event` — o mesmo que o original faz ao abrir a janela depois do pedido)
            0x14 => self.send(NetCommand::PartyMatchList { level: 1, location: -1, mode: 1 }),
            // FUN_10068610(): manda o crest da aliança com (null, 0), ou seja, apaga.
            0x5b => self.send(NetCommand::SetAllyCrest { data: Vec::new() }),
            // FUN_10068920(): informação da aliança.
            0x23 => self.send(NetCommand::RequestAllyInfo),
            // friendinvite/friendlist/frienddel (FUN_10068af0/FUN_10068b20/FUN_10068b00).
            0x24 => {
                if let Some(name) = name() {
                    self.send(NetCommand::FriendInvite { name });
                }
            }
            0x25 => self.send(NetCommand::FriendList),
            0x26 => {
                if let Some(name) = name() {
                    self.send(NetCommand::FriendDel { name });
                }
            }
            // dwarvenmanufacture/generalmanufacture: `FUN_10068500(<id>, 0, 0)` é a mesma coisa
            // que `FUN_1007e980` — usar a ação de `actionname-e.dat` com esse id (0x25 = 37
            // "manufacture"/Dwarven Manufacture, 0x33 = 51 "manufacture2"/General Manufacture).
            0x3b => self.send(NetCommand::UseAction { id: 0x25, ctrl: self.ctrl, shift: self.shift }),
            0x65 => self.send(NetCommand::UseAction { id: 0x33, ctrl: self.ctrl, shift: self.shift }),
            // useskill/useskillforce/useskillstand: o original resolve o NOME da skill por
            // `FL2GameData::GetMSCommandData` e usa o id achado, com os flags força/parado.
            0x18..=0x1a => match name().and_then(|name| self.skill_id_by_name(&name)) {
                Some(skill_id) => self.send(NetCommand::UseSkill { skill_id, ctrl: token == 0x19, shift: token == 0x1a }),
                None => self.pending.push(UiCommand::Log(format!("macro: skill \"{command}\" não encontrada"))),
            },
            // useshortcut*: dois números (página 1..10, slot 1..12) viram o índice achatado
            // `(página - 1) * 12 + (slot - 1)`, exatamente a conta do original.
            0x1b..=0x1d => {
                if let Some(slot) = macro_shortcut_slot(&args) {
                    self.use_shortcut(world, slot);
                }
            }
            // equip: mesma conta de página+slot do `/useshortcut`, mas `FUN_1007a1e0` só age se a
            // célula da barra guardar um ITEM (checa `+0x1b14 == 1` antes de usar).
            0x50 => {
                if let Some(slot) = macro_shortcut_slot(&args)
                    && let Some(shortcut) = world.shortcut(slot as i32)
                    && let ShortcutKind::Item { object_id } = shortcut.kind
                {
                    self.send(NetCommand::UseItem { object_id, ctrl: self.ctrl });
                }
            }
            // FUN_10068b30(kind): lista/bloqueia/desbloqueia. Os ids vêm do próprio dispatcher.
            0x2b => self.send(NetCommand::Block { kind: 2, name: String::new() }),
            0x2c | 0x2d => {
                if let Some(name) = name() {
                    self.send(NetCommand::Block { kind: i32::from(token == 0x2d), name });
                }
            }
            0x4e => self.send(NetCommand::Block { kind: 3, name: String::new() }),
            0x4f => self.send(NetCommand::Block { kind: 4, name: String::new() }),
            // changepartyleader: pelo nome dado ou pelo alvo atual.
            0x55 => {
                if let Some(name) = target_name() {
                    self.send(NetCommand::ChangePartyLeader { name });
                }
            }
            // FUN_1006a160(): tempo restante (item de tempo limitado).
            0x35 => self.send(NetCommand::RequestRemainTime),
            // FUN_10069110(200, -1): o pet cata o item mais próximo. Sem pet não há o que fazer.
            0x42 => self.pet_pick_up(world),
            // `Gmcancel`: encaminha pro slot `0x160` do handler de rede
            // (`FUN_100692b0`), que é o `RequestPetitionCancel` (`0x80`, `cS`).
            0x2 => self.send(NetCommand::PetitionCancel { content: name().unwrap_or_default() }),
            0x20 => self.dialogs.push(Dialog::AllyDismissName),
            // `clanwarstart`/`clanwarstop`: confirmação `0x5fb`/`0x5fc` e, no sim,
            // `RequestStartPledgeWar` (`0x4d`, `cS`) / `RequestStopPledgeWar` (`0x4f`, `cS`),
            // os dois com o nome do clã. Sem nome não há o que mandar.
            0x56 | 0x57 => match target_name() {
                Some(pledge_name) => self.dialogs.push(Dialog::PledgeWarConfirm {
                    message_id: if token == 0x56 { PLEDGE_WAR_START_SYSMSG } else { PLEDGE_WAR_STOP_SYSMSG },
                    pledge_name,
                    start: token == 0x56,
                }),
                None => self.pending.push(UiCommand::SystemMessage(SELECT_MEMBER_SYSMSG)),
            },
            // `channelinvite`/`channelkick`: o argumento, ou o nome do alvo quando vem vazio
            // (`FUN_10073f80`); sem os dois, a mensagem dourada `0x5fa`. Vão pros slots `0x2f4`
            // e `0x2fc` do handler de rede, que são `RequestExAskJoinMPCC` (`0xd0:0x0d`) e
            // `RequestExOustFromMPCC` (`0xd0:0x0f`), os dois `chS`.
            0x5e | 0x5f => match target_name() {
                Some(name) if token == 0x5e => self.send(NetCommand::ExAskJoinMpcc { name }),
                Some(name) => self.send(NetCommand::ExOustFromMpcc { name }),
                None => self.pending.push(UiCommand::ColoredSystemMessage {
                    id: CHANNEL_NEEDS_TARGET_SYSMSG,
                    color: killer_ui::color::GOLD.0,
                }),
            },
            // Casos que o próprio cliente original deixa vazios nesta build (`break;` puro).
            0x28 | 0x29 | 0x2a | 0x44 | 0x61 => {}
            // Verbos que no original abrem janela ou mexem em coisa só do cliente, e que não
            // podem virar pacote: `GM` (abre a janela de GM com o texto, `console + 0x110`),
            // `allycrest` (confirmação `0x203` e diálogo de arquivo do brasão),
            // `findprivatestore` (janela de busca, `+0x94` da janela em `DAT[0x15] + 100`) e
            // Mandar user command aqui seria pacote errado. `start_videorecording`/`end_videorecording`/
            // `startend_videorecording` chamam `FL2ReplayManager::OnSaveStart/OnSaveStop/OnSaveStartStop`.
            0x1 => {
                let text = args.join(" ");
                self.pending.push(UiCommand::OpenPetition((!text.is_empty()).then_some(text)));
            }
            0x67 => self.pending.push(UiCommand::ReplayRecord(ReplayRecord::Start)),
            0x68 => self.pending.push(UiCommand::ReplayRecord(ReplayRecord::Stop)),
            0x69 => self.pending.push(UiCommand::ReplayRecord(ReplayRecord::Toggle)),
            0x22 | 0x54 => self.pending.push(UiCommand::Log(format!(
                "comando de macro \"{command}\": no original abre janela do cliente, não portada"
            ))),
            // `default:` do original: empilha o token e manda pro servidor como user command —
            // é assim que /loc, /time, /unstuck, /partyinfo, /siegestatus etc. funcionam.
            _ => self.send(NetCommand::UserCommand { command: token as i32 }),
        }
    }

    /// attack/attackforce/attackstand (`FUN_10071cb0`/`FUN_10068510` no dispatcher original):
    /// todos atacam o alvo atual; `force` ignora a checagem de alvo amigável e `stand` ataca sem
    /// sair do lugar. No nosso protocolo isso é o mesmo par `AttackRequest`/`Action` já usado
    /// pelo clique com ctrl.
    fn attack_current_target(&mut self, world: &World, force: bool, stand: bool) {
        let Some(object_id) = world.target else { return };
        let Some(me) = world.me.as_ref() else { return };
        let origin = [me.position.x as i32, me.position.y as i32, me.position.z as i32];
        match force {
            true => self.send(NetCommand::AttackRequest { object_id, origin, shift: stand }),
            false => self.send(NetCommand::Action { object_id, origin, shift: stand }),
        }
    }

    /// `NConsoleWnd::ActionOnTarget` (`0x10071cb0`): Ctrl pressionado vira ataque forçado, Shift
    /// pressionado vira o flag de ficar parado; depois o marcador de destino é apagado.
    fn action_on_target(&mut self, world: &World, stand: bool) {
        self.attack_current_target(world, self.ctrl, stand || self.shift);
        if world.target.is_some() && world.me.is_some() {
            self.pending.push(UiCommand::DeleteLandMark);
        }
    }

    /// `NConsoleWnd::RequestTargetNext` (`0x10071a70`): `GetNextEnemy` (`0x103f9a60`, rotação por
    /// distância de `GetNextObject` `0x104211d0`) e `Action` com a posição do jogador e o Shift.
    fn target_next(&mut self, world: &World) {
        let current = world.targeted().map_or(-1, |target| target.object_id);
        if let Some(object_id) = world.next_enemy(MACRO_SEARCH_RADIUS, current) {
            self.action_on(world, object_id);
        }
    }

    /// `NConsoleWnd::RequestPickUp` (`0x100719e0`): `GetNearestItem` (`0x103f99a0`) e `Action`.
    fn pick_up(&mut self, world: &World) {
        if let Some(object_id) = world.nearest_item(MACRO_SEARCH_RADIUS, -1) {
            self.action_on(world, object_id);
        }
    }

    /// `NConsoleWnd::PetPickUp` (`0x10069110`): `GetPetNearestItem` (`0x103f9960`, medido a partir do
    /// pet) e `RequestPetGetItem` com o id do item.
    fn pet_pick_up(&mut self, world: &World) {
        if let Some(object_id) = world.pet_nearest_item(MACRO_SEARCH_RADIUS, -1) {
            self.send(NetCommand::PetGetItem { object_id });
        }
    }

    fn action_on(&mut self, world: &World, object_id: i32) {
        let Some(me) = world.me.as_ref() else { return };
        let origin = [me.position.x as i32, me.position.y as i32, me.position.z as i32];
        self.send(NetCommand::Action { object_id, origin, shift: self.shift });
    }

    /// `/useskill <nome>`: o original resolve o nome por `FL2GameData::GetMSCommandData`; aqui a
    /// busca é na `skillname-e.dat` já carregada, pelo nome exato (sem diferenciar maiúsculas),
    /// preferindo o maior nível conhecido — que é o que o jogador espera ao digitar o nome.
    fn skill_id_by_name(&self, name: &str) -> Option<i32> {
        self.skillname
            .values()
            .filter(|skill| skill.name.eq_ignore_ascii_case(name))
            .max_by_key(|skill| skill.level)
            .and_then(|skill| i32::try_from(skill.id).ok())
    }

    fn delivery_selection_action(&mut self, action: killer_ui::delivery::SelectAction, desktop: &mut Desktop) {
        if let killer_ui::delivery::SelectAction::Request(target_id) = action {
            self.send(NetCommand::PackageSendableItemList { target_id });
        }
        self.select_delivery.clear();
        set_visible(desktop, WindowId::SELECT_DELIVERY, false);
    }

    pub fn handles(&self, id: WindowId) -> bool {
        crate::ui_key_routes::handles(id) || self.messenger.owns(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &self,
        id: WindowId,
        list: &mut DrawList,
        x: i32,
        y: i32,
        world: &World,
        font: &BitmapFont,
        alpha: f32,
    ) {
        match id {
            id if self.messenger.owns(id) => {
                self.messenger.paint(id, list, (x, y), font, alpha);
            }
            WindowId::MAIN => {
                self.main.paint_chrome(list, x, y, font, alpha);
                let (px, py) = (x + PAGE_RECT.x, y + PAGE_RECT.y);
                match self.main.tab {
                    MainTab::Status => {
                        let lookup = |id: u32| self.sysstrings.get(&id).cloned();
                        detail_status_view(world).paint(list, px, py, font, &Sysstrings(&lookup));
                    }
                    MainTab::Skills => self.skills.paint(list, px, py, font),
                    MainTab::Actions => self.actions.paint(list, px, py, font),
                    MainTab::Clan => self.pledge.paint(list, px, py, font),
                    MainTab::Quests => {
                        self.quest_painted.set(true);
                        self.quest.paint(list, px, py, font)
                    }
                }
            }
            WindowId::INVENTORY => self.inventory.paint(list, x, y, font, alpha),
            WindowId::SHOP => self.shop.view.paint(list, x, y, font, alpha),
            WindowId::SHOP_PREVIEW => self.shop_preview.view.paint(list, x, y, font, alpha),
            WindowId::PRIVATE_SHOP => {
                self.private_shop.view.private_paint(list, x, y, font, alpha);
            }
            WindowId::DELIVERY => self.delivery.view.paint(list, x, y, font, alpha),
            WindowId::SELECT_DELIVERY => self.select_delivery.paint(list, x, y, font),
            WindowId::WAREHOUSE => self.warehouse.view.paint(list, x, y, font, alpha),
            WindowId::GM_STORE => self.gm_store.view.paint(list, x, y, font, alpha),
            WindowId::MANOR_SHOP => self.manor_shop.view.paint(list, x, y, font, alpha),
            WindowId::TRADE => self.trade.view.paint(list, x, y, font, alpha),
            WindowId::MULTISELL => self.multisell.view.paint(list, x, y, font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            killer_ui::manor_info::WINDOW_ID => self.manor_info.paint(list, (x, y), font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            WindowId::MANOR_SEED_SETTING => self.manor_setting.setting.paint(list, (x, y), font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            WindowId::MANOR_SEED_CHANGE => self.manor_setting.change.paint(list, (x, y), font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            WindowId::MANOR_CROP_SETTING => self.manor_setting.crop_setting.paint(list, (x, y), font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            WindowId::MANOR_CROP_CHANGE => self.manor_setting.crop_change.paint(list, (x, y), font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            WindowId::MANOR_CROP_SELL => self.manor_setting.crop_sell.paint(list, (x, y), font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            WindowId::MANOR_CROP_SELL_CHANGE => self.manor_setting.procure_change.paint(list, (x, y), font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            WindowId::HERO_TOWER => self.hero_tower.tower.paint(list, (x, y), font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            WindowId::HERO_TOWER_CONTENT => self.hero_tower.content.paint(list, (x, y), font, alpha),
            WindowId::FISHING => killer_ui::fishing::paint(&self.fishing, list, (x, y), font, alpha),
            WindowId::EMPOWER => self.empower.paint(list, (x, y), font, alpha),
            WindowId::OLYMPIAD_CONTROL => self.olympiad.control.paint(list, (x, y), font, alpha),
            WindowId::OLYMPIAD_PLAYER_1 => self.olympiad.players[0].paint(list, (x, y), font, alpha),
            WindowId::OLYMPIAD_PLAYER_2 => self.olympiad.players[1].paint(list, (x, y), font, alpha),
            WindowId::OLYMPIAD_ABNORMAL_1 => abnormal::paint_abnormal(list, x, y, &self.olympiad.abnormal[0], font, alpha),
            WindowId::OLYMPIAD_ABNORMAL_2 => abnormal::paint_abnormal(list, x, y, &self.olympiad.abnormal[1], font, alpha),
            WindowId::OLYMPIAD_TARGET => self.olympiad.target.paint(list, (x, y), font, alpha),
            WindowId::GM => self.gm.paint(list, (x, y), font, alpha),
            WindowId::GM_EVENT => self.gm_event.paint(list, (x, y), font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            WindowId::GM_FENCE => self.gm_fence.paint(list, (x, y), font, alpha),
            killer_ui::debug_wnd::ID => self.debug.paint(list, (x, y), font),
            id if killer_ui::gm_snoop::WINDOW_IDS.contains(&id) => {
                if let Some(index) = killer_ui::gm_snoop::WINDOW_IDS.iter().position(|candidate| *candidate == id) {
                    self.snoop.windows[index].paint(list, (x, y), font, alpha);
                }
            }
            WindowId::GM_INVENTORY => self.gm_inventory.paint(list, (x, y), font, alpha),
            WindowId::GM_SKILLS => self.gm_skills.paint(list, (x, y), font, alpha),
            WindowId::GM_QUEST => self.gm_quest.paint(list, (x, y), font, alpha),
            WindowId::GM_CLAN => self.gm_clan.paint(list, (x, y), font, alpha),
            WindowId::BROADCAST_GM_MSG
            | WindowId::BROADCAST_SPECIAL_MSG
            | WindowId::BROADCAST_TOP
            | WindowId::BROADCAST_PLAYER_0
            | WindowId::BROADCAST_PLAYER_1
            | WindowId::BROADCAST_PARTY_0
            | WindowId::BROADCAST_PARTY_1
            | WindowId::BROADCAST_MSG => {
                self.broadcast.paint(id, list, (x, y), font, &|id| self.sysstrings.get(&id).cloned());
            }
            WindowId::GM_STATUS => self.gm_status.paint(list, (x, y), font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            WindowId::OBSERVER => self.observer.paint(list, (x, y), font, alpha),
            WindowId::CALC => self.calc.paint(list, (x, y), font, alpha),
            WindowId::QUEST_LIST_INFO => self.quest_info.paint(list, (x, y), font, alpha, &|id| self.sysstrings.get(&id).cloned()),
            WindowId::PETITION_BEGIN => self.petition.begin.paint(list, (x, y), font, alpha),
            WindowId::PETITION => self.petition.player.paint(list, (x, y), font, alpha),
            WindowId::PETITION_GM => self.petition.gm.paint(list, (x, y), font, alpha),
            WindowId::PETITION_FEEDBACK => self.petition.feedback.paint(list, (x, y), font, alpha),
            WindowId::TRAIN => self.train.view.paint(list, x, y, font, alpha),
            WindowId::HENNA_LIST => self.henna.list.paint(list, x, y, font, alpha),
            WindowId::HENNA_INFO => self.henna.info.paint(list, x, y, font, alpha),
            WindowId::RECIPE_BOOK => self.recipes.book.paint(list, x, y, font, alpha),
            WindowId::RECIPE_MANUFACTURE => self.recipes.manufacture.paint(list, x, y, font, alpha),
            WindowId::RECIPE_TREE => self.recipes.tree.paint(list, x, y, font, alpha),
            WindowId::RECIPE_SHOP => self.recipes.shop.view.paint(list, x, y, font, alpha),
            WindowId::RECIPE_SELL => self.recipes.sell.paint(list,x,y,font,alpha),
            WindowId::RECIPE_BUY => self.recipes.buy.paint(list,x,y,font,alpha),
            WindowId::PET_WINDOW => self.pet.view.paint(list, x, y, font, alpha),
            WindowId::SUMMONED_WINDOW => self.pet.summoned.paint(list, x, y, font, alpha),
            WindowId::ITEM_ENCHANT => self.enchant.paint(list, x, y, font, alpha),
            WindowId::OPTION => self.option.paint(list, x, y, font, alpha),
            WindowId::MACRO_LIST => self.macros.list.paint(list, x, y, font, alpha),
            WindowId::MACRO_INFO => self.macros.info.paint(list, x, y, font, alpha),
            WindowId::MACRO_EDIT => self.macros.edit.paint(list, x, y, font, alpha),
            WindowId::PARTY_ROOM_MAKING => self.party_room_making.paint(list, x, y, font, alpha),
            WindowId::PARTY_ROOM => self.party_room.paint(list, x, y, font, alpha),
            WindowId::PARTY_MATCH => self.party_match.paint(list, x, y, font, alpha),
            command_channel::WINDOW_ID => self.command_channel.paint(list, x, y, font, alpha),
            WindowId::SIEGE => self.siege.view.paint(list, (x, y), font, alpha),
            WindowId::SSQ => self.ssq.paint(list, x, y, font),
            SYSMENU_WINDOW => self.sysmenu.paint(list, x, y, font, alpha),
            WindowId::TOWN_MAP => self.special_maps.town.paint(list, x, y, font, alpha),
            WindowId::XMAS_SEAL => self.special_maps.seal.paint(list, x, y, font, alpha),
            WindowId::MAP_COVER => {
                self.minimap.paint(list, x, y, font, alpha);
                self.minimap.paint_quest_tooltip(list, x, y, font);
                self.minimap.paint_player_tooltip(list, x, y, font);
            }
            _ => {}
        }
    }

    pub fn mouse_down(&mut self, id: WindowId, x: i32, y: i32, desktop: &mut Desktop) -> bool {
        if matches!(id, WindowId::TOWN_MAP | WindowId::XMAS_SEAL) {
            self.special_maps.mouse_down(id, x, y, desktop);
            return true;
        }
        if id == WindowId::TRADE {
            self.trade.mouse_down(x, y);
            return true;
        }
        if id == WindowId::MULTISELL {
            self.multisell.mouse_down(x, y, desktop);
            return true;
        }
        if id == WindowId::TRAIN {
            self.train.mouse_down(x, y, desktop);
            return true;
        }
        if id == WindowId::HENNA_LIST {
            self.henna.list_mouse_down(x, y, desktop);
            return true;
        }
        if id == WindowId::RECIPE_SELL { self.recipes.sell_down(x,y); return true; }
        if id == WindowId::RECIPE_BUY { self.recipes.buy_down(x,y); return true; }
        if id == WindowId::RECIPE_SHOP {
            self.recipes.shop.mouse_down(x, y);
            return true;
        }
        if id == WindowId::RECIPE_TREE {
            self.recipes.tree.mouse_down(x, y);
            return true;
        }
        if id == WindowId::RECIPE_MANUFACTURE {
            self.recipes.manufacture_mouse_down(x, y);
            return true;
        }
        if id == WindowId::SUMMONED_WINDOW {
            if let Some(action) = self.pet.summoned_mouse_down(x, y) {
                self.press = Some(Press { window: id, source: Some(DragSource::Action { id: action }) });
            }
            return true;
        }
        if id == WindowId::PET_WINDOW {
            if let Some(action) = self.pet.mouse_down(x, y) {
                self.press = Some(Press { window: id, source: Some(DragSource::Action { id: action }) });
            } else if let Some(killer_ui::pet_wnd::PetHit::Item(index)) = self.pet.view.hit(x, y) {
                let object_id = self.pet.view.items[index].grid.object_id;
                self.sound(interface_sound::PICKUP);
                self.press = Some(Press { window: id, source: Some(DragSource::PetItem { object_id }) });
            }
            return true;
        }
        if id == WindowId::HENNA_INFO {
            self.henna.info_mouse_down(x, y, desktop);
            return true;
        }
        if id == WindowId::OPTION {
            if self.option_applying { return true; }
            self.option_pressed = self.option.hit(x, y);
            self.option.initialize_pressed = self.option_pressed == Some(killer_ui::option::OptionHit::Initialize);
            if !matches!(self.option_pressed, Some(killer_ui::option::OptionHit::Combo(_) | killer_ui::option::OptionHit::ComboArrow(_))) {
                self.option.lose_combo_focus();
            }
            match self.option_pressed {
                Some(killer_ui::option::OptionHit::Combo(key) | killer_ui::option::OptionHit::ComboArrow(key)) => {
                    let arrow = matches!(self.option_pressed, Some(killer_ui::option::OptionHit::ComboArrow(_)));
                    if let Some(notification) = self.option.combo_mouse_down(key, arrow)
                        && notification.page == killer_ui::option::TAB_VIDEO && notification.id == 0x1f08
                    {
                        self.pending.push(UiCommand::OptionResolutionChanged(notification.selection));
                    }
                }
                Some(killer_ui::option::OptionHit::Check(index)) => self.option.toggle(index),
                Some(killer_ui::option::OptionHit::VideoCheck(index)) => {
                    if let Some(notification) = self.option.toggle_video(index)
                        && notification.id == 0xf0f01465
                    {
                        self.pending.push(UiCommand::OptionKeepMinFrameRateChanged);
                    }
                }
                Some(killer_ui::option::OptionHit::Tab(index)) => {
                    for event in self.option.press_tab(index) {
                        if let killer_ui::tab_control::TabEvent::Sound(id) = event {
                            log::debug!("Settings: som de UI {id} pendente de backend de áudio");
                        }
                    }
                }
                _ => {}
            }
            return true;
        }
        if id == WindowId::ITEM_ENCHANT {
            self.enchant.pressed = self.enchant.hit(x, y);
            match self.enchant.pressed {
                Some(killer_ui::enchant_wnd::EnchantHit::Close) => {
                    self.enchant.pressed = None;
                    self.hide_enchant(desktop);
                }
                Some(killer_ui::enchant_wnd::EnchantHit::Slot(index)) => self.enchant.grid.selected = Some(index),
                Some(killer_ui::enchant_wnd::EnchantHit::Scroll(hit)) => {
                    let grid = &mut self.enchant.grid;
                    grid.scroll.up_pressed = hit == ScrollHit::Up;
                    grid.scroll.down_pressed = hit == ScrollHit::Down;
                    if let ScrollHit::Slider(pos) = hit {
                        grid.scroll.slider.mouse_down(pos);
                    }
                }
                _ => {}
            }
            return true;
        }
        if id == WindowId::MACRO_LIST || id == WindowId::MACRO_INFO {
            self.macros.mouse_down(id, x, y, desktop);
            return true;
        }
        if id == WindowId::MACRO_EDIT {
            self.macros.mouse_down(id, x, y, desktop);
            return true;
        }
        if id == WindowId::DELIVERY {
            if self.delivery.view.hit(x, y) == Some(killer_ui::shop::ShopHit::Close) {
                self.pending.push(UiCommand::InterfaceSound("itemsound.window_close"));
            }
            self.delivery.mouse_down(x, y, desktop);
            return true;
        }
        if id == WindowId::SELECT_DELIVERY {
            if let Some(action) = self.select_delivery.mouse_down(x, y) {
                self.delivery_selection_action(action, desktop);
            }
            return true;
        }
        if id == WindowId::WAREHOUSE {
            self.warehouse.mouse_down(x, y, desktop);
            return true;
        }
        if id == WindowId::GM_STORE {
            self.gm_store.mouse_down(x, y, desktop);
            return true;
        }
        if id == WindowId::MANOR_SHOP {
            self.manor_shop.mouse_down(x, y, desktop);
            return true;
        }
        if id == WindowId::SHOP {
            self.shop.mouse_down(x, y, desktop);
            return true;
        }
        if id == WindowId::PRIVATE_SHOP {
            let quit = self.private_shop.private_quit();
            self.private_shop.mouse_down(x, y, desktop);
            if let Some(command) = quit.filter(|_| !self.private_shop.private_manage) {
                self.send(command);
            }
            return true;
        }
        if id == WindowId::SHOP_PREVIEW {
            self.shop_preview.mouse_down(x, y, desktop);
            return true;
        }
        if id == WindowId::PARTY_ROOM_MAKING {
            self.party_room_making.mouse_down(x, y);
            return true;
        }
        if id == WindowId::PARTY_MATCH {
            self.party_match.mouse_down(x, y);
            return true;
        }
        if id == WindowId::SIEGE {
            if let Some(action) = self.siege.view.mouse_down(x, y) { self.siege_action(action, desktop); }
            return true;
        }
        if id == WindowId::SSQ {
            self.ssq.mouse_down(x, y);
            return true;
        }
        if id == WindowId::MANOR_SEED_SETTING {
            return self.manor_setting.mouse_down_setting(x, y, desktop);
        }
        if id == WindowId::MANOR_CROP_SETTING {
            return self.manor_setting.mouse_down_crop_setting(x, y, desktop);
        }
        if id == WindowId::MANOR_CROP_SELL {
            return self.manor_setting.mouse_down_sell(x, y, desktop);
        }
        if id == WindowId::HERO_TOWER {
            return true;
        }
        if id == WindowId::EMPOWER {
            self.empower.mouse_down(x, y);
            return true;
        }
        if id == WindowId::QUEST_LIST_INFO {
            self.quest_info.mouse_down(x, y);
            return true;
        }
        if id == WindowId::OBSERVER {
            self.observer.mouse_down(x, y);
            return true;
        }
        if self.olympiad.mouse_down(id, x, y) {
            return true;
        }
        if id == WindowId::HERO_TOWER_CONTENT {
            return self.hero_tower.mouse_down_content(x, y, desktop);
        }
        if matches!(id, WindowId::MANOR_SEED_CHANGE | WindowId::MANOR_CROP_CHANGE | WindowId::MANOR_CROP_SELL_CHANGE) {
            return true;
        }
        if id == killer_ui::manor_info::WINDOW_ID {
            if self.manor_info.mouse_down(x, y) == Some(killer_ui::manor_info::ManorHit::Close) {
                set_visible(desktop, id, false);
            }
            return true;
        }
        let source = match id {
            WindowId::MAIN => self.main_mouse_down(x, y, desktop),
            WindowId::INVENTORY => self.inventory_mouse_down(x, y, desktop),
            WindowId::MAP_COVER => match self.minimap.hit(x, y) {
                Some(MinimapHit::Close) => {
                    set_visible(desktop, WindowId::MAP_COVER, false);
                    None
                }
                Some(MinimapHit::Button(button)) => {
                    self.minimap.pressed = Some(button);
                    Some(DragSource::Minimap(button))
                }
                Some(MinimapHit::Cover | MinimapHit::Body) => {
                    self.minimap.press_map(x, y);
                    None
                }
                _ => None,
            },
            WindowId::RECIPE_BOOK => {
                if std::mem::take(&mut self.recipe_double_clicked) {
                    return true;
                }
                self.recipes.book_mouse_down(x, y, false, desktop).map(|id| DragSource::Recipe { id })
            }
            SYSMENU_WINDOW => {
                let button = sysmenu::button_at(x, y);
                self.sysmenu.pressed = button;
                button.map(DragSource::SysMenu)
            }
            _ => return false,
        };
        if matches!(source, Some(DragSource::Grid { .. } | DragSource::Paperdoll { .. } | DragSource::Action { .. } | DragSource::Skill { .. })) {
            self.sound(interface_sound::PICKUP);
        }
        self.press = Some(Press { window: id, source });
        true
    }

    fn main_mouse_down(&mut self, x: i32, y: i32, desktop: &mut Desktop) -> Option<DragSource> {
        if mainwnd::close_button().contains(x, y) {
            set_visible(desktop, WindowId::MAIN, false);
            return None;
        }
        if let Some(tab) = mainwnd::tab_at(x, y) {
            return Some(DragSource::Tab(tab));
        }
        let (px, py) = (x - PAGE_RECT.x, y - PAGE_RECT.y);
        match self.main.tab {
            MainTab::Skills => match self.skills.hit(px, py) {
                Some(SkillHit::Tab(tab)) => {
                    if self.skills.tab != tab {
                        self.sound(interface_sound::WINDOW_OPEN);
                    }
                    self.skills.tab = tab;
                    self.skills.update_rows();
                    None
                }
                Some(SkillHit::Scroll(hit)) => {
                    match hit {
                        ScrollHit::Up => self.skills.scroll.up_pressed = true,
                        ScrollHit::Down => self.skills.scroll.down_pressed = true,
                        ScrollHit::Slider(y) => {
                            self.skills.scroll.slider.mouse_down(y);
                        }
                    }
                    Some(DragSource::Scroll(hit))
                }
                Some(SkillHit::Slot(index)) => {
                    let skill = self.skills.slot_skill(index).map(|s| (s.id, s.level));
                    self.skills.selected = skill.map(|(id, _)| id);
                    self.skills.pressed = skill.is_some();
                    skill.map(|(id, level)| DragSource::Skill { id, level })
                }
                _ => None,
            },
            MainTab::Actions => {
                let action = self.actions.hit(px, py).map(|a| a.id);
                self.actions.selected = action;
                self.actions.pressed = action;
                action.map(|id| DragSource::Action { id })
            }
            MainTab::Quests => {
                match self.quest.hit(px, py) {
                    Some(hit @ (QuestHit::Quest(_) | QuestHit::Step(_, _))) => self.quest.toggle(hit),
                    Some(QuestHit::Abort) => self.quest.abort_pressed = true,
                    Some(QuestHit::Location) => self.quest.toggle_location(),
                    Some(QuestHit::Scroll) => {
                        self.quest.scroll_mouse_down(px, py);
                    }
                    _ => {}
                }
                None
            }
            MainTab::Clan => {
                if let Some(sound) = self.pledge.header_down(px, py) {
                    self.sound(sound);
                    return None;
                }
                if self.pledge.scroll_down(px, py) {
                    return None;
                }
                match self.pledge.hit(px, py) {
                    Some(PledgeHit::Button(button)) => {
                        self.pledge.pressed = Some(button);
                        Some(DragSource::Pledge(button))
                    }
                    Some(PledgeHit::Row(row)) => {
                        self.pledge.selected = Some(row);
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub fn gm_clan_action(&mut self, button: PledgeButton, world: &World, desktop: &mut Desktop) {
        match button {
            PledgeButton::Authorize => {}
            PledgeButton::Penalty | PledgeButton::Community | PledgeButton::DeleteInsignia | PledgeButton::SetInsignia => {
                self.pledge_action(button, world, desktop);
                return;
            }
            other => {
                self.pending.push(UiCommand::Log(format!("botão do clã do GM sem porte: {other:?}")));
                return;
            }
        }
        let leader = world.me.as_ref().is_some_and(|me| me.name == self.gm_clan.pledge.leader);
        let member = self.gm_clan.selected_member().filter(|member| !member.name.is_empty()).cloned();
        if let (true, Some(member)) = (leader, member) {
            if member.object_id > 0 {
                self.send(NetCommand::PledgePower { member_id: member.object_id, action: killer_ui::empower::MODE_EDIT, powers: [0; killer_ui::empower::POWER_BYTES] });
                self.open_empower(member.object_id, &member.name, killer_ui::empower::MODE_EDIT, desktop);
            } else {
                self.dialogs.push(Dialog::Notice(SystemMessage { id: killer_ui::empower::NOT_FOUND_SYSMSG as i32, args: Vec::new() }));
            }
        }
    }

    pub fn sync_gm_clan(&mut self, info: &l2_net::game::GmViewPledgeInfo) {
        let (_, clan_id, name, leader) = &info.header;
        let values = info.details.as_ref().map(|details| details.values).unwrap_or([0; 9]);
        self.gm_clan.apply_info(&killer_ui::gm_clan::GmPledgeInfo {
            clan_id: *clan_id,
            name: name.clone(),
            leader: leader.clone(),
            level: values[1],
            castle: values[2] != 0,
            hideout: values[3] != 0,
        });
        self.gm_clan.clear_members();
        for (member, ints) in &info.members {
            self.gm_clan.update_member(MemberRow { name: member.clone(), level: ints[0], class_id: ints[1], online: ints[4] > 0, object_id: ints[4] });
        }
    }

    fn pledge_action(&mut self, button: PledgeButton, world: &World, desktop: &mut Desktop) {
        match button {
            PledgeButton::Invite => match player_target(world) {
                Some(target) => self.send(NetCommand::JoinPledge { object_id: target.object_id }),
                None => self.pending.push(UiCommand::SystemMessage(INVITE_NEEDS_TARGET_SYSMSG)),
            },
            PledgeButton::Title => match player_target(world) {
                Some(target) => self.dialogs.push(Dialog::GiveTitle { target: target.name.clone() }),
                None => self.pending.push(UiCommand::SystemMessage(TARGET_NOT_FOUND_SYSMSG)),
            },
            PledgeButton::DeleteTitle => match player_target(world) {
                Some(target) => self.dialogs.push(Dialog::DeleteTitle { target: target.name.clone() }),
                None => self.pending.push(UiCommand::SystemMessage(TARGET_NOT_FOUND_SYSMSG)),
            },
            PledgeButton::DeclareWar => self.dialogs.push(Dialog::PledgeWarName { start: true }),
            PledgeButton::EndWar => self.dialogs.push(Dialog::PledgeWarName { start: false }),
            PledgeButton::DeleteCrest => self.dialogs.push(Dialog::DeleteCrest),
            PledgeButton::DeleteInsignia => self.dialogs.push(Dialog::DeleteInsignia),
            PledgeButton::SetCrest => self.dialogs.push(Dialog::CrestNotice { large: false }),
            PledgeButton::SetInsignia => self.dialogs.push(Dialog::CrestNotice { large: true }),
            PledgeButton::Penalty => self.send(NetCommand::UserCommand { command: PLEDGE_PENALTY_USER_COMMAND }),
            PledgeButton::Community => self.community_request = true,
            PledgeButton::Dismiss => match self.pledge.selected.and_then(|i| self.pledge.members.get(i)) {
                Some(member) => self.dialogs.push(Dialog::OustPledgeMember { name: member.name.clone() }),
                None => self.pending.push(UiCommand::SystemMessage(SELECT_MEMBER_SYSMSG)),
            },
            PledgeButton::Leave => {
                if let Some(me) = world.me.as_ref() {
                    if killer_ui::empower::may_authorize(me.relation) {
                        self.pending.push(UiCommand::SystemMessage(LEADER_CANNOT_WITHDRAW_SYSMSG));
                    } else {
                        let pledge_name = self.pledge.name.clone().unwrap_or_default();
                        self.dialogs.push(Dialog::WithdrawPledge { pledge_name });
                    }
                }
            }
            PledgeButton::Privileges => {
                if let Some(me) = world.me.as_ref() {
                    self.send(NetCommand::PledgePower { member_id: me.object_id, action: killer_ui::empower::MODE_VIEW, powers: [0; killer_ui::empower::POWER_BYTES] });
                    self.open_empower(me.object_id, &me.name, killer_ui::empower::MODE_VIEW, desktop);
                }
            }
            PledgeButton::Authorize => {
                let leader = world.me.as_ref().is_some_and(|me| killer_ui::empower::may_authorize(me.relation));
                let member = self.pledge.selected.and_then(|i| self.pledge.members.get(i)).filter(|member| !member.name.is_empty());
                if let (true, Some(member)) = (leader, member) {
                    if member.object_id > 0 {
                        let (id, name) = (member.object_id, member.name.clone());
                        self.send(NetCommand::PledgePower { member_id: id, action: killer_ui::empower::MODE_EDIT, powers: [0; killer_ui::empower::POWER_BYTES] });
                        self.open_empower(id, &name, killer_ui::empower::MODE_EDIT, desktop);
                    } else {
                        self.dialogs.push(Dialog::Notice(SystemMessage { id: killer_ui::empower::NOT_FOUND_SYSMSG as i32, args: Vec::new() }));
                    }
                }
            }
        }
    }

    fn open_empower(&mut self, target: i32, name: &str, mode: i32, desktop: &mut Desktop) {
        self.empower.reset();
        self.empower.set_target(target, name, mode);
        self.empower.show();
        if let Some(window) = desktop.window_mut(WindowId::EMPOWER) {
            window.visible = true;
        }
        if let Some(index) = desktop.windows.iter().position(|window| window.id == WindowId::EMPOWER) {
            let window = desktop.windows.remove(index);
            desktop.windows.push(window);
        }
    }

    fn empower_action(&mut self, action: killer_ui::empower::EmpowerAction, desktop: &mut Desktop) {
        if let killer_ui::empower::EmpowerAction::Write { target, powers } = action {
            self.send(NetCommand::PledgePower { member_id: target, action: killer_ui::empower::ACTION_WRITE, powers });
        }
        set_visible(desktop, WindowId::EMPOWER, false);
        self.empower.reset();
    }

    fn inventory_mouse_down(&mut self, x: i32, y: i32, desktop: &mut Desktop) -> Option<DragSource> {
        match self.inventory.hit(x, y)? {
            InventoryHit::Close => {
                set_visible(desktop, WindowId::INVENTORY, false);
                None
            }
            InventoryHit::Henna(_) => None,
            InventoryHit::Tab(tab) => {
                if self.inventory.tab != tab {
                    self.sound(interface_sound::WINDOW_OPEN);
                }
                self.inventory.tab = tab;
                self.inventory.update_rows();
                None
            }
            InventoryHit::Scroll(hit) => {
                match hit {
                    ScrollHit::Up => self.inventory.scroll.up_pressed = true,
                    ScrollHit::Down => self.inventory.scroll.down_pressed = true,
                    ScrollHit::Slider(y) => {
                        self.inventory.scroll.slider.mouse_down(y);
                    }
                }
                Some(DragSource::Scroll(hit))
            }
            InventoryHit::Grid(index) => {
                let item = self.inventory.grid_item(index).map(|i| (i.object_id, i.count));
                self.inventory.selected = item.map(|(id, _)| id);
                if std::mem::take(&mut self.grid_double_clicked) {
                    return None;
                }
                item.map(|(object_id, count)| DragSource::Grid { object_id, count })
            }
            InventoryHit::Paperdoll(slot) => {
                let item = self.inventory.paperdoll_item(slot).map(|i| (i.object_id, i.body_part));
                self.inventory.selected = item.map(|(id, _)| id);
                item.map(|(object_id, body_part)| DragSource::Paperdoll { object_id, body_part })
            }
            InventoryHit::Trash | InventoryHit::Recipe | InventoryHit::Weight | InventoryHit::Title | InventoryHit::Body => None,
        }
    }

    pub fn mouse_up(
        &mut self,
        over: Option<(WindowId, i32, i32)>,
        desktop: &mut Desktop,
        bar: &mut ShortcutView,
        world: &World,
    ) {
        self.macros.edit.mouse_up_box();
        self.macros.info.edit.mouse_up();
        self.macros.edit.scroll_up(over.filter(|(id, _, _)| *id == WindowId::MACRO_EDIT).map(|(_, x, y)| (x, y)));
        let pledge_over = over.filter(|(id, _, _)| *id == WindowId::MAIN).map(|(_, x, y)| (x - PAGE_RECT.x, y - PAGE_RECT.y));
        self.pledge.header_up(pledge_over.map_or(i32::MIN / 2, |(x, _)| x), pledge_over.map_or(i32::MIN / 2, |(_, y)| y));
        self.pledge.scroll_up(pledge_over);
        self.special_maps.town.mouse_up();
        self.inventory.over_trash = false;
        self.shop.mouse_up(over, desktop, &mut self.pending);
        self.private_shop.mouse_up(over, desktop, &mut self.pending);
        self.shop_preview.mouse_up(over, desktop, &mut self.pending);
        self.warehouse.mouse_up(over, desktop, &mut self.pending);
        self.gm_store.mouse_up(over, desktop, &mut self.pending);
        self.manor_shop.mouse_up(over, desktop, &mut self.pending);
        self.delivery.mouse_up(over, desktop, &mut self.pending);
        if let Some(action) = self.select_delivery.mouse_up(over.filter(|(id, _, _)| *id == WindowId::SELECT_DELIVERY).map(|(_, x, y)| (x, y))) {
            self.delivery_selection_action(action, desktop);
        }
        self.party_room_making_mouse_up(over, desktop);
        self.party_room_mouse_up(over, desktop);
        if let Some((WindowId::PARTY_MATCH, x, y)) = over {
            if let Some(action) = self.party_match.mouse_up(x, y) {
                self.party_match_action(action, world, desktop);
            }
        } else {
            self.party_match.mouse_up(i32::MIN / 2, i32::MIN / 2);
        }
        self.command_channel_mouse_up(over);
        self.manor_setting.mouse_up(over, desktop);
        self.hero_tower.mouse_up(over, desktop);
        let (ex, ey) = over.filter(|(id, _, _)| *id == WindowId::EMPOWER).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        self.olympiad.mouse_up(over, desktop);
        let (ox, oy) = over.filter(|(id, _, _)| *id == WindowId::OBSERVER).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        if self.observer.mouse_up(ox, oy).is_some() {
            self.send(NetCommand::ObserverEnd);
        }
        let (qx, qy) = over.filter(|(id, _, _)| *id == WindowId::QUEST_LIST_INFO).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        match self.quest_info.mouse_up(qx, qy) {
            Some(killer_ui::quest_info::QuestInfoAction::Close) => set_visible(desktop, WindowId::QUEST_LIST_INFO, false),
            Some(killer_ui::quest_info::QuestInfoAction::Location(target)) => self.quest.show_info(target),
            None => {}
        }
        if let Some(action) = self.empower.mouse_up(ex, ey) {
            self.empower_action(action, desktop);
        }
        match over.filter(|(id, _, _)| *id == killer_ui::manor_info::WINDOW_ID) {
            Some((_, x, y)) => { self.manor_info.mouse_up(x, y); }
            None => { self.manor_info.mouse_up(i32::MIN / 2, i32::MIN / 2); }
        }
        let (x, y) = over.filter(|(id, _, _)| *id == WindowId::SIEGE).map(|(_, x, y)| (x, y)).unwrap_or((-10000, -10000));
        if let Some(action) = self.siege.view.mouse_up(x, y) { self.siege_action(action, desktop); }
        let (x, y) = over.filter(|(id, _, _)| *id == WindowId::SSQ).map(|(_, x, y)| (x, y)).unwrap_or((-10000, -10000));
        if let Some(action) = self.ssq.mouse_up(x, y) { self.ssq_action(action, desktop); }
        self.trade.mouse_up(over, desktop);
        self.multisell.mouse_up(over, desktop);
        self.train.mouse_up(over, desktop);
        self.henna.list_mouse_up(over);
        self.henna.info_mouse_up(over, desktop);
        self.recipes.manufacture_mouse_up(over, desktop);
        self.recipes.tree_mouse_up(over, desktop);
        self.recipes.shop.mouse_up(over, desktop);
        self.recipes.sell_up(over,desktop);
        self.recipes.buy_up(over,desktop);
        self.recipes.book_mouse_up(over);
        self.pet.mouse_up(over);
        self.enchant_mouse_up(over, desktop);
        self.option_mouse_up(over, desktop);
        self.macros.shortcut_macros = bar.slots.iter().map(|slot| matches!(slot.kind, Some(SlotKind::Macro { .. }))).collect();
        match over {
            Some((id, x, y)) => {
                let tables = crate::ui_macros::MacroTables { skillname: &self.skillname, commandname: &self.commandname };
                self.macros.mouse_up(id, x, y, &tables, desktop);
            }
            None => self.macros.edit.pressed = None,
        }
        if over.is_some_and(|(id, _, _)| id != WindowId::MACRO_EDIT) {
            self.macros.edit.pressed = None;
        }
        self.inventory.over_recipe = false;
        self.actions.pressed = None;
        self.skills.pressed = false;
        self.skills.selected = None;
        self.pledge.pressed = None;
        self.minimap.pressed = None;
        self.minimap.release_map();
        self.quest.scroll_mouse_up();
        if std::mem::take(&mut self.quest.abort_pressed)
            && over.is_some_and(|(id, x, y)| id == WindowId::MAIN && self.quest.hit(x - PAGE_RECT.x, y - PAGE_RECT.y) == Some(QuestHit::Abort))
        {
            match self.quest.abort_target() {
                Some(quest_id) => self.dialogs.push(Dialog::AbortQuest { quest_id }),
                None => self.dialogs.push(Dialog::Notice(SystemMessage { id: killer_ui::quest::SELECT_QUEST_SYSMSG as i32, args: Vec::new() })),
            }
        }
        let pressed_button = self.sysmenu.pressed.take();
        self.drag_moved = false;
        let Some(press) = self.press.take() else { return };
        let Some(source) = press.source else { return };
        if bar.edit_locked() && over.is_some_and(|(id, _, _)| id == WindowId::SHORTCUT) { return; }
        match source {
            DragSource::Scroll(hit) => {
                let released = match over {
                    Some((WindowId::INVENTORY, x, y)) if press.window == WindowId::INVENTORY => self.inventory.scroll_hit(x, y),
                    Some((WindowId::MAIN, x, y)) if press.window == WindowId::MAIN => self.skills.scroll_hit(x - PAGE_RECT.x, y - PAGE_RECT.y),
                    _ => None,
                };
                let same = matches!((hit, released), (ScrollHit::Up, Some(ScrollHit::Up)) | (ScrollHit::Down, Some(ScrollHit::Down)));
                if press.window == WindowId::INVENTORY {
                    self.inventory.scroll.up_pressed = false;
                    self.inventory.scroll.down_pressed = false;
                    self.inventory.scroll.slider.mouse_up();
                    if same {
                        let (first, visible, total) = (self.inventory.scroll_row as i32, inventory::GRID_ROWS as i32, self.inventory.total_rows());
                        let next = if hit == ScrollHit::Up { line_up(first, visible, total) } else { line_down(first, visible, total) };
                        self.inventory.set_first_row(next);
                    }
                } else {
                    self.skills.scroll.up_pressed = false;
                    self.skills.scroll.down_pressed = false;
                    self.skills.scroll.slider.mouse_up();
                    if same {
                        let (first, visible, total) = (self.skills.scroll_row as i32, killer_ui::skills::LIST_ROWS as i32, self.skills.total_rows());
                        let next = if hit == ScrollHit::Up { line_up(first, visible, total) } else { line_down(first, visible, total) };
                        self.skills.set_first_row(next);
                    }
                }
            }
            DragSource::SysMenu(button) => {
                if over.is_some_and(|(id, x, y)| id == SYSMENU_WINDOW && sysmenu::button_at(x, y) == Some(button))
                    && pressed_button == Some(button)
                {
                    self.sysmenu_action(button, desktop);
                }
            }
            DragSource::Tab(tab) => {
                if over.is_some_and(|(id, x, y)| id == WindowId::MAIN && mainwnd::tab_at(x, y) == Some(tab)) {
                    let changed = self.main.tab != tab;
                    if changed {
                        self.sound(interface_sound::WINDOW_OPEN);
                    }
                    self.main.tab = tab;
                    if changed && tab == MainTab::Quests {
                        self.begin_quest_list();
                    }
                    if changed && tab == MainTab::Clan {
                        self.pledge_show_pending = true;
        }
                    if changed && tab == MainTab::Skills {
                        self.send(NetCommand::RequestSkillList);
                    }
                }
            }
            DragSource::Pledge(button) => {
                self.pledge.pressed = None;
                if over.is_some_and(|(id, x, y)| {
                    id == WindowId::MAIN && self.pledge.hit(x - PAGE_RECT.x, y - PAGE_RECT.y) == Some(PledgeHit::Button(button))
                }) {
                    self.pledge_action(button, world, desktop);
                }
            }
            DragSource::Minimap(button) => {
                if over.is_some_and(|(id, x, y)| id == WindowId::MAP_COVER && self.minimap.hit(x, y) == Some(MinimapHit::Button(button))) {
                    self.minimap_action(button, world);
                }
            }
            DragSource::Skill { id, .. } => match over {
                Some((WindowId::MAIN, x, y)) => {
                    let hit = self.skills.hit(x - PAGE_RECT.x, y - PAGE_RECT.y);
                    if matches!(hit, Some(SkillHit::Slot(index)) if self.skills.slot_skill(index).is_some_and(|s| s.id == id))
                    {
                        self.send(NetCommand::UseSkill { skill_id: id, ctrl: self.ctrl, shift: self.shift });
                    }
                }
                Some((WindowId::SHORTCUT, x, y)) => {
                    if let Some(slot) = bar.slot_at(x, y) {
                        self.send(NetCommand::ShortcutReg { kind: 2, slot: slot as i32, id, character_type: 1 });
                    }
                }
                Some((WindowId::MACRO_EDIT, x, y)) => self.drop_skill_on_macro_line(id, x, y),
                _ => {}
            },
            DragSource::Recipe { id } => {
                if let Some((WindowId::SHORTCUT, x, y)) = over
                    && let Some(slot) = bar.slot_at(x, y)
                {
                    self.send(NetCommand::ShortcutReg { kind: l2_net::game::SHORTCUT_RECIPE, slot: slot as i32, id, character_type: 1 });
                }
                self.recipes.dropped(id, over);
            }
            DragSource::Action { id } => match over {
                Some((WindowId::SUMMONED_WINDOW, x, y)) if self.pet.summoned.hit(x, y) == Some(killer_ui::pet_wnd::PetHit::Action(id)) => {
                    self.use_action(world, id, false, false);
                }
                Some((WindowId::PET_WINDOW, x, y)) if self.pet.view.hit(x, y) == Some(killer_ui::pet_wnd::PetHit::Action(id)) => {
                    self.use_action(world, id, false, false);
                }
                Some((WindowId::MAIN, x, y)) if self.actions.hit(x - PAGE_RECT.x, y - PAGE_RECT.y).is_some_and(|a| a.id == id) => {
                    self.use_action(world, id, false, false);
                }
                Some((WindowId::SHORTCUT, x, y)) => {
                    if let Some(slot) = bar.slot_at(x, y) {
                        let character_type = actions::shortcut_character_type(id);
                        self.send(NetCommand::ShortcutReg { kind: 3, slot: slot as i32, id, character_type });
                    }
                }
                Some((WindowId::MACRO_EDIT, x, y)) => self.drop_action_on_macro_line(id, x, y),
                _ => {}
            },
            DragSource::Grid { object_id, .. } | DragSource::Paperdoll { object_id, .. } if over.is_none() => {
                let item = self.inventory.items.iter().find(|item| item.object_id == object_id);
                let (item_id, name) = item.map_or((0, String::new()), |item| (item.item_id, item.name.clone()));
                let count = item.map_or(1, |item| item.count);
                self.pending.push(UiCommand::ItemDrop { object_id, item_id, count, name, target: ItemDropTarget::World });
            }
            DragSource::PetItem { object_id } => match over {
                Some((WindowId::PET_WINDOW, x, y)) => {
                    if let Some(target) = self.pet.view.slot_at(x, y) {
                        self.pet.move_item(object_id, target);
                    }
                }
                Some((WindowId::INVENTORY, x, y)) => {
                    if matches!(self.inventory.hit(x, y), Some(InventoryHit::Grid(_) | InventoryHit::Paperdoll(_)))
                        && let Some(item) = self.pet.view.items.iter().find(|item| item.grid.object_id == object_id)
                    {
                        let (item_id, count, name) = (item.grid.item_id, item.grid.count, item.grid.name.clone());
                        self.pending.push(UiCommand::ItemDrop { object_id, item_id, count, name, target: ItemDropTarget::FromPet });
                    }
                }
                Some((WindowId::SHORTCUT, x, y)) => {
                    if let Some(slot) = bar.slot_at(x, y) {
                        self.send(NetCommand::ShortcutReg { kind: 1, slot: slot as i32, id: object_id, character_type: 2 });
                    }
                }
                _ => {}
            },
            DragSource::Grid { object_id, count } if matches!(over, Some((WindowId::PET_WINDOW, _, _))) => {
                let item = self.inventory.items.iter().find(|item| item.object_id == object_id);
                let (item_id, name) = item.map_or((0, String::new()), |item| (item.item_id, item.name.clone()));
                self.pending.push(UiCommand::ItemDrop { object_id, item_id, count, name, target: ItemDropTarget::Pet });
            }
            DragSource::Grid { object_id, count } => match over {
                Some((WindowId::INVENTORY, x, y)) => match self.inventory.hit(x, y) {
                    Some(InventoryHit::Grid(target)) => {
                        self.inventory.move_listed(object_id, target);
                    }
                    Some(InventoryHit::Paperdoll(_)) => self.send(NetCommand::UseItem { object_id, ctrl: self.ctrl }),
                    Some(hit @ (InventoryHit::Trash | InventoryHit::Recipe)) => {
                        let item = self.inventory.items.iter().find(|item| item.object_id == object_id);
                        if hit == InventoryHit::Recipe && !item.is_some_and(|item| item.crystallizable) {
                            return;
                        }
                        let (item_id, name) = item.map_or((0, String::new()), |item| (item.item_id, item.name.clone()));
                        let target = if hit == InventoryHit::Trash { ItemDropTarget::Trash } else { ItemDropTarget::Crystallize };
                        self.pending.push(UiCommand::ItemDrop { object_id, item_id, count, name, target });
                    }
                    _ => {}
                },
                Some((WindowId::SHORTCUT, x, y)) => {
                    if let Some(hit) = bar.auto_at(x, y) {
                        let item_id = self.inventory.items.iter().find(|item| item.object_id == object_id).map_or(0, |item| item.item_id);
                        bar.assign_auto(hit, item_id);
                    } else if let Some(slot) = bar.slot_at(x, y) {
                        self.send(NetCommand::ShortcutReg {
                            kind: 1,
                            slot: slot as i32,
                            id: object_id,
                            character_type: 1,
                        });
                    }
                }
                _ => {}
            },
            DragSource::Paperdoll { object_id, body_part } => match over {
                Some((WindowId::INVENTORY, x, y)) => match self.inventory.hit(x, y) {
                    Some(InventoryHit::Grid(_)) => {
                        if let Some(mask) = unequip_mask(body_part, object_id, &self.inventory.paperdoll_ids) {
                            self.send(NetCommand::Unequip { body_part: mask });
                        }
                    }
                    Some(hit @ (InventoryHit::Trash | InventoryHit::Recipe)) => {
                        let item = self.inventory.items.iter().find(|item| item.object_id == object_id);
                        if hit == InventoryHit::Recipe && !item.is_some_and(|item| item.crystallizable) {
                            return;
                        }
                        let (item_id, count, name) = item.map_or((0, 1, String::new()), |item| (item.item_id, item.count, item.name.clone()));
                        let target = if hit == InventoryHit::Trash { ItemDropTarget::Trash } else { ItemDropTarget::Crystallize };
                        self.pending.push(UiCommand::ItemDrop { object_id, item_id, count, name, target });
                    }
                    _ => {}
                },
                Some((WindowId::PET_WINDOW, x, y))
                    if self.pet.view.page() == killer_ui::pet_wnd::PAGE_ITEMS && killer_ui::pet_wnd::PAGE_RECT.contains(x, y) =>
                {
                    if let Some(item) = self.inventory.items.iter().find(|item| item.object_id == object_id) {
                        let (item_id, count, name) = (item.item_id, item.count, item.name.clone());
                        self.pending.push(UiCommand::ItemDrop { object_id, item_id, count, name, target: ItemDropTarget::PetEquipped });
                    }
                }
                Some((WindowId::SHORTCUT, x, y)) => {
                    if let Some(hit) = bar.auto_at(x, y) {
                        let item_id = self.inventory.items.iter().find(|item| item.object_id == object_id).map_or(0, |item| item.item_id);
                        bar.assign_auto(hit, item_id);
                    } else if let Some(slot) = bar.slot_at(x, y) {
                        self.send(NetCommand::ShortcutReg {
                            kind: 1,
                            slot: slot as i32,
                            id: object_id,
                            character_type: 1,
                        });
                    }
                }
                _ => {}
            },
        }
        let _ = world;
    }

    fn party_room_making_mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        let Some(pressed) = self.party_room_making.pressed.take() else { return };
        let Some((id, x, y)) = over else { return };
        if id != WindowId::PARTY_ROOM_MAKING || self.party_room_making.hit(x, y) != Some(pressed) { return }
        match pressed {
            MakingHit::Ok => {
                let mut data = self.party_room_making.data();
                data.title = if data.title.is_empty() { self.party_match_title.clone() } else { self.word_filter(&data.title) };
                self.send(NetCommand::ManagePartyRoom {
                    room_id: data.room_id,
                    max_members: data.max_members,
                    min_level: data.min_level,
                    max_level: data.max_level,
                    loot_type: self.party_looting,
                    title: data.title,
                });
                set_visible(desktop, WindowId::PARTY_ROOM_MAKING, false);
            }
            MakingHit::Cancel => set_visible(desktop, WindowId::PARTY_ROOM_MAKING, false),
            _ => {}
        }
    }

    fn party_room_mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        let (x, y) = over.filter(|(id, _, _)| *id == WindowId::PARTY_ROOM).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        let actions = self.party_room.mouse_up(x, y);
        self.party_room_actions(actions, desktop);
    }

    pub fn party_room_mouse_down(&mut self, x: i32, y: i32, font: &killer_ui::font::BitmapFont) {
        self.party_room.mouse_down(x, y, font);
    }

    pub fn party_room_mouse_move(&mut self, x: i32, y: i32, font: &killer_ui::font::BitmapFont) {
        self.party_room.mouse_move(x, y, font);
    }

    fn party_room_actions(&mut self, actions: Vec<RoomAction>, desktop: &mut Desktop) {
        for action in actions {
            match action {
                RoomAction::Manage { room_id, title, max_members, min_level, max_level } => {
                    let current = self.party_room_making.data();
                    self.party_room_making.set_data(&killer_ui::party_room::MakingData {
                        room_id,
                        loot_type: current.loot_type,
                        location: current.location,
                        title,
                        max_members,
                        min_level,
                        max_level,
                    });
                    set_visible(desktop, WindowId::PARTY_ROOM_MAKING, true);
                    desktop.bring_to_front(WindowId::PARTY_ROOM_MAKING);
                }
                RoomAction::Oust { object_id } => self.send(NetCommand::OustFromPartyRoom { object_id }),
                RoomAction::Invite { name, loot_type } => self.send(NetCommand::JoinParty { name, item_distribution: loot_type }),
                RoomAction::Dismiss { room_id } => self.send(NetCommand::DismissPartyRoom { room_id }),
                RoomAction::Withdraw { room_id } => self.send(NetCommand::WithdrawPartyRoom { room_id }),
                RoomAction::Say { text } => self.send(NetCommand::Say { text, kind: party_room::SAY_PARTY_ROOM, target: String::new() }),
                RoomAction::Bypass(command) => self.send(NetCommand::BypassBuildCmd { command }),
                RoomAction::Hide => {
                    self.party_room.hide();
                    set_visible(desktop, WindowId::PARTY_ROOM, false);
                }
                RoomAction::Minimize => {
                    let title = self.sysstrings.get(&389).map(String::as_str).unwrap_or_default().to_owned();
                    self.minimized.minimize_room(desktop, &title);
                }
            }
        }
    }

    fn party_room_event(&mut self, event: &WorldEvent, desktop: &mut Desktop, world: &World) {
        let values = |id: i32, name: &str, values: &[i32; 4]| killer_ui::party_room::MemberValues {
            id,
            name: name.to_string(),
            class_id: values[0],
            level: values[1],
            location: values[2],
            role: values[3],
        };
        match event {
            WorldEvent::PartyMatchDetail(detail) => {
                if self.minimized.room_icon_visible() {
                    self.minimized.blink_room();
                } else {
                    set_visible(desktop, WindowId::PARTY_MATCH, false);
                    set_visible(desktop, WindowId::PARTY_ROOM, true);
                    desktop.bring_to_front(WindowId::PARTY_ROOM);
                }
                self.party_room.set_info(killer_ui::party_room::RoomInfo {
                    room_id: detail.id,
                    max_members: detail.max_members,
                    min_level: detail.min_level,
                    max_level: detail.max_level,
                    loot_type: detail.loot_type,
                    location: detail.location,
                    title: detail.title.clone(),
                });
            }
            WorldEvent::PartyRoomMember(list) => {
                let members = list.entries.iter().map(|entry| values(entry.id, &entry.name, &entry.values)).collect::<Vec<_>>();
                self.party_room.set_members(list.header[0], &members);
                self.minimized.blink_room();
            }
            WorldEvent::ManagePartyRoomMember(manage) => {
                let me = world.me.as_ref().map(|me| me.object_id);
                self.party_room.manage_member(manage.header[0], &values(manage.header[1], &manage.name, &manage.values), me);
                self.minimized.blink_room();
            }
            WorldEvent::ClosePartyRoom => {
                self.party_room.close_packet();
                set_visible(desktop, WindowId::PARTY_ROOM, false);
            }
            _ => {}
        }
    }

    pub fn command_channel_chat(&mut self, event: &WorldEvent, desktop: &mut Desktop, font: &killer_ui::font::BitmapFont) {
        if let WorldEvent::Say2(say) = event
            && matches!(say.kind, command_channel::SAY_ALL_MEMBERS | command_channel::SAY_LEADER)
            && let l2_net::game::Say2Body::Text { name, text } = &say.body
        {
            set_visible(desktop, command_channel::WINDOW_ID, true);
            self.command_channel.show();
            self.command_channel.receive_chat(say.kind, &format!("{name}: {text}"), font);
        }
    }

    pub fn party_room_chat(&mut self, event: &WorldEvent, font: &killer_ui::font::BitmapFont) {
        if let WorldEvent::Say2(say) = event
            && say.kind == party_room::SAY_PARTY_ROOM
            && let l2_net::game::Say2Body::Text { name, text } = &say.body
        {
            self.party_room.add_chat(&format!("{name}: {text}"), killer_ui::color::chat_color(say.kind), font);
            self.minimized.blink_room();
        }
    }

    fn party_match_action(&mut self, action: killer_ui::party_match::Action, world: &World, desktop: &mut Desktop) {
        use killer_ui::party_match::Action;
        match action {
            Action::RequestList { page, location, mode } => self.send(NetCommand::PartyMatchList { level: page, location, mode }),
            Action::Join { room_id, location } => self.send(NetCommand::JoinPartyRoomAt { room_id, location }),
            Action::Close => set_visible(desktop, WindowId::PARTY_MATCH, false),
            Action::Minimize => self.minimized.minimize_match(desktop, self.sysstrings.get(&389).map(String::as_str).unwrap_or_default()),
            Action::CreateRoom => {
                let (min_level, max_level) = killer_ui::party_match::create_levels(world.me.as_ref().map(|me| me.level));
                self.party_room_making.set_data(&killer_ui::party_room::MakingData {
                    room_id: 0, loot_type: 0, location: 0,
                    title: self.party_match_title.clone(),
                    max_members: 12, min_level, max_level,
                });
                set_visible(desktop, WindowId::PARTY_ROOM_MAKING, true);
                desktop.bring_to_front(WindowId::PARTY_ROOM_MAKING);
            }
        }
    }

    fn siege_action(&mut self, action: killer_ui::siege::SiegeAction, desktop: &mut Desktop) {
        let (command, dialog) = self.siege.action(action, desktop);
        if let Some(command) = command { self.send(command); }
        if let Some(dialog) = dialog { self.dialogs.push(dialog); }
    }

    fn ssq_action(&mut self, action: killer_ui::ssq::Action, desktop: &mut Desktop) {
        match action {
            killer_ui::ssq::Action::RequestStatus(page) => self.send(NetCommand::SsqStatus { page }),
            killer_ui::ssq::Action::Close => set_visible(desktop, WindowId::SSQ, false),
        }
    }

    pub fn minimized_mouse_up(&mut self, desktop: &mut Desktop) -> bool {
        self.minimized.mouse_up(desktop, &mut self.pending)
    }

    pub fn native_mouse_move(&mut self, id: WindowId, x: i32, y: i32) -> bool {
        match id {
            WindowId::PARTY_MATCH => { self.party_match.mouse_move(x, y); true }
            WindowId::PARTY_ROOM_MAKING => { self.party_room_making.mouse_move(x, y); true }
            WindowId::SIEGE => { self.siege.view.mouse_move(x, y); true }
            WindowId::SSQ => self.ssq.mouse_move(x, y),
            WindowId::QUEST_LIST_INFO => { self.quest_info.mouse_move(x, y); true }
            killer_ui::manor_info::WINDOW_ID => { self.manor_info.mouse_move(x, y); true }
            WindowId::MANOR_SEED_SETTING => { self.manor_setting.setting.mouse_move(x, y); true }
            WindowId::MANOR_CROP_SETTING => { self.manor_setting.crop_setting.mouse_move(x, y); true }
            WindowId::MANOR_CROP_SELL => { self.manor_setting.crop_sell.mouse_move(x, y); true }
            _ => false,
        }
    }

    pub fn party_room_key(&mut self, key: killer_ui::chat::InputKey, font: &killer_ui::font::BitmapFont, desktop: &mut Desktop) -> bool {
        if self.party_room_making.key(key) {
            return true;
        }
        if !desktop.window(WindowId::PARTY_ROOM).is_some_and(|window| window.visible) {
            return false;
        }
        let Some(actions) = self.party_room.key(key, font) else { return false };
        self.party_room_actions(actions, desktop);
        true
    }

    fn command_channel_mouse_up(&mut self, over: Option<(WindowId, i32, i32)>) {
        let Some((id, x, y)) = over.filter(|(id, _, _)| *id == command_channel::WINDOW_ID) else {
            self.command_channel.pressed = None;
            return;
        };
        let _ = id;
        if let Some(action) = self.command_channel.mouse_up(x, y) {
            self.command_channel_action(action);
        }
    }

    pub fn command_channel_mouse_down(&mut self, x: i32, y: i32, font: &killer_ui::font::BitmapFont) {
        self.command_channel.mouse_down(x, y, font);
    }

    pub fn command_channel_mouse_move(&mut self, x: i32, y: i32, font: &killer_ui::font::BitmapFont) {
        self.command_channel.mouse_move(x, y, font);
    }

    pub fn command_channel_key(&mut self, key: killer_ui::chat::InputKey, font: &killer_ui::font::BitmapFont) -> bool {
        let (consumed, action) = self.command_channel.key(key, font);
        if let Some(action) = action {
            self.command_channel_action(action);
        }
        consumed
    }

    fn command_channel_action(&mut self, action: ChannelAction) {
        match action {
            ChannelAction::Say { kind, text } => self.send(NetCommand::Say { text, kind, target: String::new() }),
            ChannelAction::UserCommand(command) => self.send(NetCommand::UserCommand { command }),
            ChannelAction::OustFromChannel(name) => self.send(NetCommand::ExOustFromMpcc { name }),
        }
    }

    pub fn tick_command_channel(&mut self, now: Instant) {
        let last = self.command_channel_tick.replace(now);
        self.party_match.tick(last.map_or(0.0, |last| now.saturating_duration_since(last).as_secs_f32()));
        let elapsed = last.map_or(0, |last| now.saturating_duration_since(last).as_millis().min(u128::from(u32::MAX)) as u32);
        self.command_channel.tick(elapsed);
        self.party_room.tick(elapsed);
        self.minimized.tick(elapsed);
        self.minimap.tick(elapsed.min(i32::MAX as u32) as i32);
    }

    fn grade_texture(&self, grade: usize) -> Option<String> {
        self.grade_textures.get(grade).cloned().flatten()
    }

    fn item_tooltip(&self, item: &InventoryItemView) -> Tooltip {
        let text = |id: u32| self.sysstrings.get(&id).cloned().unwrap_or_else(|| format!("sysstring {id}"));
        if let Some(tip) = item.tip.as_ref() {
            let lookup = |id: u32| self.sysstrings.get(&id).cloned();
            return Tooltip {
                enchant: if (0..=3).contains(&tip.type2) { item.enchant } else { 0 },
                name: item.name.clone(),
                add_name: item.add_name.clone(),
                grade: self.grade_texture(tip.grade),
                count: tip.counted().then_some(i64::from(item.count)),
                level: None,
                lines: killer_ui::item_tip::lines(tip, item.enchant, &lookup),
                description: item.description.clone(),
                item: true,
                row: false,
                row_style: 0,
                compact: false,
                anchor: None,
            };
        }
        Tooltip {
            enchant: item.enchant,
            name: item.name.clone(),
            add_name: item.add_name.clone(),
            grade: None,
            count: (item.count > 1).then_some(i64::from(item.count)),
            level: None,
            lines: vec![TooltipLine { label: text(WEIGHT_SYSSTRING), value: item.weight.to_string(), color: VALUE_COLOR }],
            description: item.description.clone(),
            ..Default::default()
        }
    }

    fn item_tooltip_slot(&self, item: &InventoryItemView, selected: bool, window: WindowId, slot: (i32, i32)) -> Tooltip {
        let mut tip = self.item_tooltip(item);
        if tip.item {
            tip.compact = !selected;
            tip.anchor = Some((window, slot));
        }
        tip
    }

    fn shortcut_item_tooltip(&self, item: &InventoryItemView) -> Tooltip {
        let counted = item.tip.as_ref().map_or(item.count > 1, |tip| tip.counted());
        let enchant = item.tip.as_ref().map_or(item.enchant, |tip| if (0..=3).contains(&tip.type2) { item.enchant } else { 0 });
        Tooltip {
            enchant,
            name: item.name.clone(),
            add_name: item.add_name.clone(),
            grade: item.tip.as_ref().and_then(|tip| self.grade_texture(tip.grade)),
            count: counted.then_some(i64::from(item.count)),
            ..Default::default()
        }
    }

    fn shortcut_skill_tooltip(&self, skill: &SkillView) -> Tooltip {
        let full = self.skill_tooltip(skill);
        Tooltip { name: full.name, level: full.level, ..Default::default() }
    }

    fn skill_tooltip(&self, skill: &SkillView) -> Tooltip {
        let text = |id: u32| self.sysstrings.get(&id).cloned().unwrap_or_else(|| format!("sysstring {id}"));
        let tip = u32::try_from(skill.id).ok().zip(u32::try_from(skill.level).ok()).and_then(|key| self.skill_tips.get(&key));
        let level = tip.map_or(skill.level, |tip| i32::try_from(tip.display_level(skill.level as u32)).unwrap_or(skill.level));
        let mut lines = Vec::new();
        if let Some(tip) = tip {
            lines.push(TooltipLine { label: String::new(), value: text(tip.type_sysstring), color: VALUE_COLOR });
            for (label, value) in tip.numbers() {
                lines.push(TooltipLine { label: text(label), value: value.to_string(), color: VALUE_COLOR });
            }
        }
        Tooltip {
            name: skill.name.clone(),
            level: Some((text(LV_SYSSTRING), level)),
            lines,
            description: skill.description.clone(),
            ..Default::default()
        }
    }

    /// Diferente de `skill_tooltip` (que resolve pela lista de skills conhecidas do próprio
    /// jogador, `self.skills.skills`), buffs vistos na barra de status podem ter sido lançados
    /// por outra classe ou por um inimigo (debuffs) e não estar nessa lista — por isso o nome
    /// vem direto da tabela `skillname-e.dat` (`self.skillname`, id+level), igual ao que
    /// `sync_skills` já faz para popular `SkillView::name`.
    fn abnormal_tooltip(&self, icon: &killer_ui::abnormal::BuffIcon) -> Option<Tooltip> {
        let key = (u32::try_from(icon.skill_id).ok()?, u32::try_from(icon.level).ok()?);
        let name = self.skillname.get(&key)?;
        let text = |id: u32| self.sysstrings.get(&id).cloned().unwrap_or_else(|| format!("sysstring {id}"));
        let level = self.skill_tips.get(&key).map_or(icon.level, |tip| i32::try_from(tip.display_level(key.1)).unwrap_or(icon.level));
        let lines = match icon.remaining {
            Some(seconds) if icon.shows_time_left() => vec![TooltipLine {
                label: text(abnormal::TIME_LEFT_SYSSTRING),
                value: abnormal::short_time_text(seconds, &text(abnormal::HOUR_SYSSTRING), &text(abnormal::MINUTE_SYSSTRING)),
                color: VALUE_COLOR,
            }],
            _ => Vec::new(),
        };
        Some(Tooltip {
            name: name.name.clone(),
            level: Some((text(LV_SYSSTRING), level)),
            lines,
            description: name.description.clone(),
            ..Default::default()
        })
    }

    pub fn drag_icon(&self, bar: &ShortcutView) -> Option<String> {
        for icon in [self.trade.drag_icon(), self.shop.drag_icon(), self.private_shop.drag_icon(), self.shop_preview.drag_icon(), self.warehouse.drag_icon(), self.gm_store.drag_icon(), self.gm_skills.drag_icon(), self.manor_shop.drag_icon(), self.delivery.drag_icon()] {
            if icon.is_some() { return icon; }
        }
        if let Some(icon) = self.recipes.shop.drag_icon() { return Some(icon); }
        if bar.dragging && self.drag_moved {
            return bar.pressed.and_then(|index| bar.slot(index)).map(|slot| slot.icon.clone()).filter(|icon| !icon.is_empty());
        }
        if !self.drag_moved {
            return None;
        }
        let icon = match self.press?.source? {
            DragSource::Grid { object_id, .. } | DragSource::Paperdoll { object_id, .. } => self
                .inventory
                .items
                .iter()
                .find(|item| item.object_id == object_id)
                .and_then(|item| item.icons.first().cloned()),
            DragSource::Skill { id, .. } => self.skills.skills.iter().find(|skill| skill.id == id).map(|skill| skill.icon.clone()),
            DragSource::Action { id } => self.actions.actions.iter().find(|action| action.id == id).map(|action| action.icon.clone()),
            DragSource::Recipe { id } => self.recipes.book.entries.iter().find(|entry| entry.recipe_id == id).map(|entry| entry.grid.icon.clone()),
            DragSource::PetItem { object_id } => self.pet.view.items.iter().find(|item| item.grid.object_id == object_id).map(|item| item.grid.icon.clone()),
            _ => None,
        };
        icon.filter(|icon| !icon.is_empty())
    }

    pub fn hover(&mut self, over: Option<(WindowId, i32, i32)>, bar: &ShortcutView) {
        self.recipes.shop.hover();
        self.trade.cursor_hover();
        self.shop.cursor_hover();
        self.private_shop.cursor_hover();
        self.shop_preview.cursor_hover();
        self.warehouse.cursor_hover();
        self.gm_store.cursor_hover();
        self.gm_skills.cursor_hover();
        self.delivery.cursor_hover();
        self.manor_shop.cursor_hover();
        if let Some((WindowId::SELECT_DELIVERY, x, y)) = over {
            self.select_delivery.combo.mouse_move(x, y);
        }
        if self.press.is_some_and(|press| matches!(press.source, Some(DragSource::Grid { .. } | DragSource::Paperdoll { .. } | DragSource::Skill { .. } | DragSource::Action { .. } | DragSource::Recipe { .. } | DragSource::PetItem { .. }))) {
            self.drag_moved = true;
        }
        let carrying_grid_item = matches!(self.press, Some(Press { source: Some(DragSource::Grid { .. }), .. }));
        let over_inventory = over.filter(|(id, _, _)| *id == WindowId::INVENTORY);
        self.inventory.over_trash = carrying_grid_item
            && over_inventory.is_some_and(|(_, x, y)| killer_ui::inventory::TRASH_RECT.contains(x, y));
        self.inventory.over_recipe = carrying_grid_item
            && over_inventory.is_some_and(|(_, x, y)| killer_ui::inventory::RECIPE_RECT.contains(x, y));
        if matches!(self.press, Some(Press { source: Some(DragSource::Skill { .. } | DragSource::Action { .. }), .. })) {
            match over {
                Some((WindowId::MACRO_EDIT, x, y)) => match self.macros.edit.hit_line(x, y) {
                    Some(line) => self.macros.edit.mark_line(line),
                    None => self.macros.edit.clear_marks(),
                },
                _ => self.macros.edit.clear_marks(),
            }
        }
        if bar.dragging {
            self.drag_moved = true;
        } else if self.press.is_none() {
            self.drag_moved = false;
        }
        if matches!(self.press, Some(Press { source: Some(DragSource::Recipe { .. }), .. })) {
            self.recipes.drag_over(over);
        }
        self.recipes.buy.tree_hover = self.recipes.buy.pressed == Some(killer_ui::recipe::ManufactureHit::Tree)
            && matches!(over, Some((WindowId::RECIPE_BUY, x, y)) if (206..=238).contains(&x) && (71..=103).contains(&y));
        self.recipes.manufacture.tree_hover = matches!(over, Some((WindowId::RECIPE_MANUFACTURE, x, y)) if (206..=238).contains(&x) && (71..=103).contains(&y));
        self.macros.list.hover = match over {
            Some((WindowId::MACRO_LIST, x, y)) if self.macros.list.dragging.is_some() => self.macros.list.hit(x, y),
            _ => None,
        };
        let broadcast_cell = match over {
            Some((id @ (WindowId::BROADCAST_PLAYER_0 | WindowId::BROADCAST_PLAYER_1), x, y)) => self.broadcast.hover_buff(id, x, y),
            _ => None,
        };
        self.tooltip = match over {
            Some((WindowId::RECIPE_BOOK, x, y)) if self.recipes.book.hit(x, y) == Some(killer_ui::recipe::BookHit::Trash) => Some(Tooltip {
                name: self.recipes.book.trash_tip.clone(),
                anchor: Some((WindowId::RECIPE_BOOK, (killer_ui::recipe::TRASH_RECT.x, killer_ui::recipe::TRASH_RECT.y))),
                ..Tooltip::default()
            }),
            Some((id @ (WindowId::RECIPE_MANUFACTURE | WindowId::RECIPE_BUY), x, y)) => {
                let view = if id == WindowId::RECIPE_MANUFACTURE { &self.recipes.manufacture } else { &self.recipes.buy };
                match view.hit(x, y) {
                    Some(killer_ui::recipe::ManufactureHit::Material(index)) => view.materials.get(index).map(|material| {
                        let item = &material.grid;
                        self.item_tooltip(&InventoryItemView {
                            enchant: item.enchant,
                            name: item.name.clone(),
                            add_name: item.add_name.clone(),
                            count: material.owned,
                            weight: item.weight,
                            description: item.description.clone(),
                            ..Default::default()
                        })
                    }),
                    _ => None,
                }
            }
            Some((WindowId::TRADE, x, y)) => match self.trade.view.hit(x, y) {
                Some(killer_ui::trade::TradeHit::Slot(which, index)) => self.trade.view.entries(which).get(index).map(|entry| {
                    let item = &entry.grid;
                    self.item_tooltip(&InventoryItemView {
                        enchant: item.enchant, name: item.name.clone(), add_name: item.add_name.clone(),
                        count: entry.quantity as i32, weight: item.weight, description: item.description.clone(),
                        tip: entry.tip.clone(),
                        ..Default::default()
                    })
                }),
                _ => None,
            },
            Some((id @ (WindowId::SHOP | WindowId::SHOP_PREVIEW | WindowId::PRIVATE_SHOP | WindowId::WAREHOUSE | WindowId::DELIVERY | WindowId::MANOR_SHOP), x, y)) => {
                let shop = if id == WindowId::SHOP { &self.shop } else if id == WindowId::SHOP_PREVIEW { &self.shop_preview } else if id == WindowId::PRIVATE_SHOP { &self.private_shop } else if id == WindowId::DELIVERY { &self.delivery } else if id == WindowId::MANOR_SHOP { &self.manor_shop } else if id == WindowId::GM_STORE { &self.gm_store } else { &self.warehouse };
                match shop.view.hit(x, y) {
                Some(killer_ui::shop::ShopHit::Slot(side, index)) => shop.view.entries(side).get(index).map(|entry| {
                    let item = &entry.grid;
                    let mut tooltip = self.item_tooltip(&InventoryItemView {
                        enchant: item.enchant,
                        name: item.name.clone(),
                        add_name: item.add_name.clone(),
                        count: entry.quantity as i32,
                        weight: item.weight,
                        description: item.description.clone(),
                        ..Default::default()
                    });
                    let price = killer_ui::shop::price_tip_lines(entry.tip_flags, entry.price, &|id| self.sysstrings.get(&id).cloned());
                    tooltip.lines.splice(0..0, price);
                    if entry.tip_flags & killer_ui::shop::TIP_NO_ENCHANT_BIT != 0 {
                        tooltip.enchant = 0;
                    }
                    if entry.tip_flags & killer_ui::shop::TIP_NO_COUNT_BIT != 0 {
                        tooltip.count = None;
                    }
                    tooltip
                }),
                _ => None,
                }
            },
            Some((WindowId::ITEM_ENCHANT, x, y)) => match self.enchant.hit(x, y) {
                Some(killer_ui::enchant_wnd::EnchantHit::Slot(index)) => self.enchant.entries.get(index).map(|item| {
                    self.item_tooltip(&InventoryItemView {
                        enchant: item.enchant,
                        name: item.name.clone(),
                        add_name: item.add_name.clone(),
                        count: item.count,
                        weight: item.weight,
                        description: item.description.clone(),
                        ..Default::default()
                    })
                }),
                _ => None,
            },
            Some((WindowId::INVENTORY, x, y)) => match self.inventory.hit(x, y) {
                Some(InventoryHit::Grid(index)) => {
                    let grid = self.inventory.grid_slot_origin(index);
                    self.inventory.grid_item(index).map(|i| self.item_tooltip_slot(i, self.inventory.selected == Some(i.object_id), WindowId::INVENTORY, grid))
                }
                Some(InventoryHit::Paperdoll(slot)) => {
                    let cell = killer_ui::inventory::paperdoll_tooltip_anchor(slot);
                    self.inventory.paperdoll_item(slot).map(|i| self.item_tooltip_slot(i, self.inventory.selected == Some(i.object_id), WindowId::INVENTORY, cell))
                }
                Some(InventoryHit::Weight) => Some(Tooltip {
                    name: killer_ui::inventory::weight_tip_text(self.inventory.load, self.inventory.max_load),
                    ..Tooltip::default()
                }),
                Some(InventoryHit::Henna(slot)) => self.inventory.henna_mark(slot).map(|mark| Tooltip {
                    name: mark.name.clone(),
                    description: mark.description.clone(),
                    ..Tooltip::default()
                }),
                _ => None,
            },
            Some((WindowId::MACRO_LIST, x, y)) => {
                let sysstring = |id: u32| self.sysstrings.get(&id).cloned().unwrap_or_default();
                match self.macros.list.hit(x, y) {
                    Some(killer_ui::macros::MacroHit::Macro(index)) => self.macros.list.macros.get(index).map(|entry| Tooltip {
                        name: entry.name.clone(),
                        description: entry.description.clone(),
                        ..Tooltip::default()
                    }),
                    Some(killer_ui::macros::MacroHit::EditIcon) => Some(Tooltip {
                        name: sysstring(killer_ui::macros::EDIT_ICON_SYSSTRING),
                        ..Tooltip::default()
                    }),
                    Some(killer_ui::macros::MacroHit::Trash) => Some(Tooltip {
                        name: sysstring(killer_ui::macros::TRASH_ICON_SYSSTRING),
                        ..Tooltip::default()
                    }),
                    _ => None,
                }
            }
            Some((WindowId::PARTY_ROOM, _, _)) => self.party_room.row_tooltip(),
            Some((WindowId::QUEST_LIST_INFO, _, _)) => self.quest_info.row_tooltip(&|id| self.sysstrings.get(&(id as u32)).cloned().unwrap_or_default()),
            Some((WindowId::MANOR_SEED_SETTING, _, _)) => self.manor_setting.setting.list.row_tip(killer_ui::list_ctrl::ROW_TIP_SEED_SETTING, "", &|id| self.sysstrings.get(&(id as u32)).cloned().unwrap_or_default()),
            Some((WindowId::MANOR_CROP_SETTING, _, _)) => self.manor_setting.crop_setting.list.row_tip(killer_ui::list_ctrl::ROW_TIP_CROP_SETTING, "", &|id| self.sysstrings.get(&(id as u32)).cloned().unwrap_or_default()),
            Some((WindowId::MANOR_CROP_SELL, _, _)) => self.manor_setting.crop_sell.list.row_tip(killer_ui::list_ctrl::ROW_TIP_CROP_SELL, "", &|id| self.sysstrings.get(&(id as u32)).cloned().unwrap_or_default()),
            Some((WindowId::MANOR_INFO, _, _)) => {
                let text = |id: i32| self.sysstrings.get(&(id as u32)).cloned().unwrap_or_default();
                match self.manor_info.page() {
                    Some(killer_ui::manor_info::ManorPage::Seed) => self.manor_info.seed.list.row_tip(killer_ui::list_ctrl::ROW_TIP_SEED_INFO, "", &text),
                    Some(killer_ui::manor_info::ManorPage::Crop) => self.manor_info.crop.list.row_tip(killer_ui::list_ctrl::ROW_TIP_CROP_INFO, "", &text),
                    Some(killer_ui::manor_info::ManorPage::Default) => self.manor_info.default.list.row_tip(killer_ui::list_ctrl::ROW_TIP_DEFAULT_INFO, "", &text),
                    None => None,
                }
            }
            Some((WindowId::MAIN, _, _)) if self.main.tab == MainTab::Clan => self.pledge.row_tooltip(),
            Some((WindowId::GM_SKILLS, x, y)) => self.gm_skills.slot_at(x, y).and_then(|index| self.gm_skills.view.slot_skill(index)).map(|s| self.skill_tooltip(s)),
            Some((WindowId::GM_INVENTORY, x, y)) => match self.gm_inventory.hit(x, y) {
                Some(killer_ui::gm_inventory::GmInventoryHit::Grid(index)) => self.gm_inventory.grid_item(index).map(|i| self.item_tooltip(i)),
                Some(killer_ui::gm_inventory::GmInventoryHit::Equipped(slot)) => self.gm_inventory.equipped_item(slot).map(|i| self.item_tooltip(i)),
                _ => None,
            },
            Some((WindowId::MAIN, x, y)) if self.main.tab == MainTab::Actions => {
                self.actions.hit(x - PAGE_RECT.x, y - PAGE_RECT.y).map(ActionEntry::tooltip)
            }
            Some((WindowId::MAIN, x, y)) if self.main.tab == MainTab::Skills => {
                match self.skills.hit(x - PAGE_RECT.x, y - PAGE_RECT.y) {
                    Some(SkillHit::Slot(index)) => self.skills.slot_skill(index).map(|s| self.skill_tooltip(s)),
                    _ => None,
                }
            }
            Some((WindowId::BROADCAST_PLAYER_0 | WindowId::BROADCAST_PLAYER_1, _, _)) => broadcast_cell.and_then(|cell| {
                self.abnormal_tooltip(&killer_ui::abnormal::BuffIcon {
                    skill_id: cell.skill,
                    level: 1,
                    remaining: Some(cell.time),
                    debuff: cell.debuff,
                    ..Default::default()
                })
            }),
            Some((WindowId::ABNORMAL_STATUS, x, y)) => match if bar.modern.is_some() {crate::ui_modern_buffs::hit(&self.abnormal,x,y)}else{abnormal::hit(&self.abnormal, x, y)} {
                Some(AbnormalHit::Buff(index)) => self.abnormal.buffs.get(index).and_then(|b| self.abnormal_tooltip(b)),
                Some(AbnormalHit::ShortBuff) => self.abnormal.short_buff.as_ref().and_then(|b| self.abnormal_tooltip(b)),
                Some(AbnormalHit::Etc(index)) => self.abnormal.etc_skills.get(index).and_then(|&(skill_id, level)| {
                    self.abnormal_tooltip(&killer_ui::abnormal::BuffIcon { skill_id, level, ..Default::default() })
                }),
                None => None,
            },
            Some((WindowId::SHORTCUT, x, y)) => bar.slot_at(x, y).and_then(|index| bar.slot(index).map(|slot| (index, slot))).and_then(|(index, slot)| (match slot.kind {
                Some(SlotKind::Item { object_id }) if bar.modern.is_none() => {
                    self.inventory.items.iter().find(|i| i.object_id == object_id).map(|i| self.shortcut_item_tooltip(i))
                }
                Some(SlotKind::Recipe { .. }) => Some(Tooltip { name: slot.name.clone(), ..Default::default() }),
                Some(SlotKind::Skill { id, level }) if bar.modern.is_none() => {
                    self.skills.skills.iter().find(|s| s.id == id && s.level == level).map(|s| self.shortcut_skill_tooltip(s))
                }
                Some(SlotKind::Action { id }) if bar.modern.is_none() => self
                    .actions
                    .actions
                    .iter()
                    .find(|action| action.id == id)
                    .map(|action| Tooltip { name: action.name.clone(), ..Default::default() }),
                Some(SlotKind::Macro { id }) if bar.modern.is_none() => self
                    .macros
                    .list
                    .macros
                    .iter()
                    .find(|entry| entry.id == id)
                    .map(|entry| Tooltip { name: entry.name.clone(), ..Default::default() }),
                Some(SlotKind::Item { object_id }) => {
                    self.inventory.items.iter().find(|i| i.object_id == object_id).map(|i| self.item_tooltip(i))
                }
                Some(SlotKind::Skill { id, level }) => {
                    self.skills.skills.iter().find(|s| s.id == id && s.level == level).map(|s| self.skill_tooltip(s))
                }
                Some(SlotKind::Action { id }) => self.actions.actions.iter().find(|action| action.id == id).map(ActionEntry::tooltip),
                Some(SlotKind::Macro { id }) => self
                    .macros
                    .list
                    .macros
                    .iter()
                    .find(|entry| entry.id == id)
                    .map(|entry| Tooltip { name: entry.name.clone(), ..Default::default() }),
                None => None,
            }).map(|mut tip| {
                if bar.modern.is_none() {
                    tip.anchor = Some((WindowId::SHORTCUT, bar.tooltip_anchor(index)));
                }
                tip
            })),
            _ => None,
        };
    }

    pub fn mouse_move(&mut self, id: WindowId, y: i32) -> bool {
        if id == WindowId::TRAIN { return self.train.view.mouse_move(y); }
        if id == WindowId::MULTISELL { return self.multisell.mouse_move(y); }
        if id == WindowId::GM_QUEST { self.gm_quest.view.scroll_mouse_move(y - killer_ui::gm_quest::PAGE_ORIGIN.1); }
        if id == WindowId::GM_CLAN { return self.gm_clan.mouse_move(y); }
        if id == WindowId::MESSENGER { self.messenger.view.mouse_move(y); }
        if id == WindowId::MACRO_EDIT { return self.macros.edit.scroll_move(y); }
        if id == WindowId::MAIN && self.pledge.scroll_move(y - PAGE_RECT.y) { return true; }
        if id == WindowId::MAIN && self.main.tab == MainTab::Quests {
            self.quest.scroll_mouse_move(y - PAGE_RECT.y);
        }
        if id == WindowId::RECIPE_SELL { return self.recipes.sell.mouse_move(y); }
        if id == WindowId::RECIPE_SHOP { return self.recipes.shop.mouse_move(y); }
        if id == WindowId::RECIPE_TREE { return self.recipes.tree.mouse_move(y); }
        if id == WindowId::RECIPE_BOOK { return self.recipes.book_mouse_move(y); }
        if id == WindowId::TRADE { return self.trade.mouse_move(y); }
        if id == WindowId::DELIVERY {
            return self.delivery.mouse_move(y);
        }
        if id == WindowId::WAREHOUSE {
            return self.warehouse.mouse_move(y);
        }
        if id == WindowId::GM_STORE {
            return self.gm_store.mouse_move(y);
        }
        if id == WindowId::MANOR_SHOP {
            return self.manor_shop.mouse_move(y);
        }
        if id == WindowId::SHOP {
            return self.shop.mouse_move(y);
        }
        if id == WindowId::PRIVATE_SHOP {
            return self.private_shop.mouse_move(y);
        }
        if id == WindowId::SHOP_PREVIEW {
            return self.shop_preview.mouse_move(y);
        }
        if id == WindowId::PET_WINDOW {
            return self.pet.mouse_move(y);
        }
        if id == WindowId::MACRO_INFO {
            self.macros.info.edit.mouse_move(y);
            return false;
        }
        if id == WindowId::ITEM_ENCHANT {
            if let Some(killer_ui::enchant_wnd::EnchantHit::Scroll(ScrollHit::Slider(_))) = self.enchant.pressed {
                let top = killer_ui::enchant_wnd::GRID_RECT.y + killer_ui::scroll::BUTTON_SIZE;
                if let Some(first) = self.enchant.grid.scroll.slider.mouse_move(y - top) {
                    self.enchant.grid.scroll_row = first.max(0) as usize;
                }
                return true;
            }
            return false;
        }
        let Some(Press { window, source: Some(DragSource::Scroll(ScrollHit::Slider(_))) }) = self.press else { return false };
        if window != id {
            return false;
        }
        match id {
            WindowId::INVENTORY => {
                let y = y - inventory::GRID_RECT.y - killer_ui::scroll::BUTTON_SIZE;
                if let Some(first) = self.inventory.scroll.slider.mouse_move(y) {
                    self.inventory.scroll_row = first.max(0) as usize;
                }
                true
            }
            WindowId::MAIN => {
                let y = y - PAGE_RECT.y - killer_ui::skills::LIST_RECT.y - killer_ui::scroll::BUTTON_SIZE;
                if let Some(first) = self.skills.scroll.slider.mouse_move(y) {
                    self.skills.scroll_row = first.max(0) as usize;
                }
                true
            }
            _ => false,
        }
    }

    pub fn right_click(&mut self, id: WindowId, x: i32, y: i32) -> bool {
        if id == WindowId::PET_WINDOW {
            return self.pet.double_click(x, y);
        }
        if id != WindowId::INVENTORY {
            return false;
        }
        match self.inventory.hit(x, y) {
            Some(InventoryHit::Grid(index)) => {
                self.use_grid_item(index);
                true
            }
            Some(InventoryHit::Paperdoll(slot)) => {
                self.unequip_paperdoll_slot(slot);
                true
            }
            _ => false,
        }
    }

    pub fn double_click(&mut self, id: WindowId, x: i32, y: i32, world: &World) -> bool {
        if id == WindowId::PARTY_MATCH {
            if !matches!(self.party_match.hit(x, y), Some(killer_ui::party_match::Hit::Row(_))) { return false; }
            if let Some(killer_ui::party_match::Action::Join { room_id, location }) = self.party_match.double_click(x, y) {
                self.send(NetCommand::JoinPartyRoomAt { room_id, location });
                return true;
            }
            return false;
        }
        if id == WindowId::RECIPE_SELL { self.recipes.sell_down(x,y); return true; }
        if id == WindowId::RECIPE_SHOP { return self.recipes.shop.double_click(x, y); }
        if id == WindowId::TRADE { return self.trade.double_click(x, y); }
        if id == WindowId::RECIPE_BOOK {
            self.press = None;
            self.recipe_double_clicked = true;
            let mut scratch = Desktop::new((0, 0));
            self.recipes.book_mouse_down(x, y, true, &mut scratch);
            return true;
        }
        if id == WindowId::DELIVERY {
            return self.delivery.double_click(x, y);
        }
        if id == WindowId::WAREHOUSE {
            return self.warehouse.double_click(x, y);
        }
        if id == WindowId::GM_STORE {
            return self.gm_store.double_click(x, y);
        }
        if id == WindowId::MANOR_SHOP {
            return self.manor_shop.double_click(x, y);
        }
        if id == WindowId::PET_WINDOW {
            return self.pet.double_click(x, y);
        }
        if id == WindowId::SHOP {
            return self.shop.double_click(x, y);
        }
        if id == WindowId::PRIVATE_SHOP {
            return self.private_shop.double_click(x, y);
        }
        if id == WindowId::SHOP_PREVIEW {
            return self.shop_preview.double_click(x, y);
        }
        if id != WindowId::INVENTORY {
            return false;
        }
        match self.inventory.hit(x, y) {
            Some(InventoryHit::Grid(index)) => {
                self.grid_double_clicked = true;
                self.press = None;
                self.use_grid_item(index);
                self.inventory.selected = None;
                let _ = world;
                true
            }
            Some(InventoryHit::Paperdoll(slot)) => {
                self.press = None;
                self.unequip_paperdoll_slot(slot);
                true
            }
            _ => false,
        }
    }

    fn use_grid_item(&mut self, index: usize) {
        let Some(item) = self.inventory.grid_item(index) else { return };
        let object_id = item.object_id;
        self.use_item_object(object_id);
    }

    fn use_shortcut_item(&mut self, world: &World, object_id: i32) {
        let dwarf = world.me.as_ref().is_some_and(|me| matches!(me.kind, EntityKind::Player { class_id, .. } if is_dwarven_class(class_id)));
        self.use_item_with(object_id, dwarf);
    }

    fn use_item_object(&mut self, object_id: i32) {
        self.use_item_with(object_id, true);
    }

    fn use_item_with(&mut self, object_id: i32, asks_recipe: bool) {
        let Some(item) = self.inventory.items.iter().find(|item| item.object_id == object_id) else {
            self.send(NetCommand::UseItem { object_id, ctrl: self.ctrl });
            return;
        };
        let (popup, recipe, name) = (item.popup, item.recipe && asks_recipe, item.name.clone());
        if recipe {
            self.dialogs.push(Dialog::RecipeRegister { object_id, item_name: name });
        } else if popup >= 1 {
            self.dialogs.push(Dialog::UseItemPopup { object_id, message_id: popup, item_name: name });
        } else {
            self.send(NetCommand::UseItem { object_id, ctrl: self.ctrl });
        }
    }

    fn unequip_paperdoll_slot(&mut self, slot: usize) {
        if let Some(item) = self.inventory.paperdoll_item(slot)
            && let Some(mask) = unequip_mask(item.body_part, item.object_id, &self.inventory.paperdoll_ids)
        {
            self.send(NetCommand::Unequip { body_part: mask });
        }
    }

    pub fn wheel(&mut self, id: WindowId, x: i32, y: i32, delta: i32) -> bool {
        if id == WindowId::SSQ { return self.ssq.wheel(x, y, delta); }
        if id == WindowId::PARTY_MATCH {
            self.party_match.wheel(delta);
            return true;
        }
        if id == WindowId::SIEGE {
            if let Some(part) = self.siege.view.active_part_mut() { part.scroll(delta / 120); }
            return true;
        }
        if id == WindowId::RECIPE_SELL { return self.recipes.sell.wheel(x,y,delta); }
        if id == WindowId::RECIPE_SHOP { return self.recipes.shop.wheel(x, y, delta); }
        if id == WindowId::RECIPE_TREE { return self.recipes.tree.wheel(x, y, delta); }
        if id == WindowId::RECIPE_BOOK { return self.recipes.book_wheel(x, y, delta); }
        if id == WindowId::MACRO_EDIT {
            if !killer_ui::macros::EDIT_COMMAND_LIST_RECT.contains(x, y) {
                return false;
            }
            self.macros.edit.scroll_commands(delta / 120 * 45);
            return true;
        }
        if id == WindowId::TRADE { return self.trade.wheel(x, y, delta); }
        if id == WindowId::GM_INVENTORY { return self.gm_inventory.wheel(x, y, delta); }
        if id == WindowId::GM_QUEST { return self.gm_quest.wheel(x, y, delta); }
        if self.messenger.owns(id) { return self.messenger.wheel(id, x, y, delta); }
        if id == WindowId::GM_CLAN { return self.gm_clan.wheel(x, y, delta); }
        if id == WindowId::GM_SKILLS {
            self.gm_skills.wheel(delta);
            return true;
        }
        if id == WindowId::QUEST_LIST_INFO {
            return self.quest_info.wheel(delta.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);
        }
        if self.petition.wheel(id, x, y, delta) {
            return true;
        }
        if self.manor_setting.wheel(id, delta.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16)
            || self.hero_tower.wheel(id, delta.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16)
        {
            return true;
        }
        if id == killer_ui::manor_info::WINDOW_ID {
            self.manor_info.wheel(delta.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);
            return true;
        }
        if id == WindowId::MULTISELL {
            return self.multisell.view.wheel(x, y, delta);
        }
        if id == WindowId::TRAIN {
            self.train.wheel(delta);
            return true;
        }
        if id == WindowId::HENNA_LIST {
            self.henna.wheel(delta);
            return true;
        }
        if id == WindowId::DELIVERY {
            return self.delivery.wheel(x, y, delta);
        }
        if id == WindowId::WAREHOUSE {
            return self.warehouse.wheel(x, y, delta);
        }
        if id == WindowId::GM_STORE {
            return self.gm_store.wheel(x, y, delta);
        }
        if id == WindowId::MANOR_SHOP {
            return self.manor_shop.wheel(x, y, delta);
        }
        if id == WindowId::SHOP {
            return self.shop.wheel(x, y, delta);
        }
        if id == WindowId::PRIVATE_SHOP {
            return self.private_shop.wheel(x, y, delta);
        }
        if id == WindowId::SHOP_PREVIEW {
            return self.shop_preview.wheel(x, y, delta);
        }
        if id == WindowId::PET_WINDOW {
            return self.pet.wheel(x, y, delta);
        }
        if id == WindowId::MACRO_INFO {
            if !killer_ui::macros::INFO_TEXT_RECT.contains(x, y) {
                return false;
            }
            self.macros.info.edit.wheel(delta);
            return true;
        }
        if id == WindowId::ITEM_ENCHANT {
            if !killer_ui::enchant_wnd::GRID_RECT.contains(x, y) {
                return false;
            }
            let grid = &self.enchant.grid;
            let first = item_wnd_wheel(grid.scroll_row as i32, grid.rows as i32, grid.total_rows(self.enchant.entries.len()), delta);
            self.enchant.grid.set_first_row(first);
            return true;
        }
        match id {
            WindowId::INVENTORY if inventory::GRID_RECT.contains(x, y) => {
                let visible = inventory::GRID_ROWS as i32;
                let total = self.inventory.total_rows();
                let first = item_wnd_wheel(self.inventory.scroll_row as i32, visible, total, delta);
                self.inventory.set_first_row(first);
                true
            }
            WindowId::MAIN if self.main.tab == MainTab::Skills && killer_ui::skills::LIST_RECT.contains(x - PAGE_RECT.x, y - PAGE_RECT.y) => {
                let visible = killer_ui::skills::LIST_ROWS as i32;
                let total = self.skills.total_rows();
                let first = item_wnd_wheel(self.skills.scroll_row as i32, visible, total, delta);
                self.skills.set_first_row(first);
                true
            }
            WindowId::MAIN if self.main.tab == MainTab::Quests && killer_ui::quest::TREE_RECT.contains(x - PAGE_RECT.x, y - PAGE_RECT.y) => {
                self.quest.wheel(delta);
                true
            }
            WindowId::MAIN if self.main.tab == MainTab::Clan && killer_ui::pledge::LIST_RECT.contains(x - PAGE_RECT.x, y - PAGE_RECT.y) => {
                self.pledge.wheel(delta);
                true
            }
            WindowId::PARTY_ROOM => self.party_room.wheel(x, y, delta),
            command_channel::WINDOW_ID => self.command_channel.wheel(x, y, delta),
            _ => false,
        }
    }

    /// Sincroniza os checkboxes da janela de opções com os valores atuais do `option.ini`
    /// carregado, para a janela abrir refletindo o estado real (não tudo desmarcado).
    pub fn sync_options(&mut self, options: &killer_ui::ini::Options) {
        self.option.set_checked(0xf0f01475, options.transparency);
        self.option.set_checked(0xf0f01459, options.show_my_name);
        self.option.set_checked(0xf0f01460, options.show_npc_name);
        self.option.set_checked(0xf0f01461, options.show_group_name);
        self.option.set_checked(0xf0f01462, options.show_pledge_member_name);
        self.option.set_checked(0xf0f01463, options.show_party_member_name);
        self.option.set_checked(0xf0f01464, options.show_other_pc_name);
        self.option.set_checked(0xf0f01470, options.auto_tracking_pawn);
        self.option.set_checked(0xf0f01476, options.arrow_mode);
        self.option.set_checked(0xf0f01478, options.show_zone_title);
        self.option.set_checked(0xf0f01471, options.enter_chatting);
        self.option.set_checked(0xf0f01479, options.system_msg.shown);
        self.option.set_checked(0xf0f01480, options.system_msg.damage);
        self.option.set_checked(0xf0f01481, options.system_msg.expendable_items);
    }

    pub(crate) fn sysmenu_action(&mut self, button: SysMenuButton, desktop: &mut Desktop) {
        match button {
            SysMenuButton::Restart if self.sysmenu.restart_locked => {}
            SysMenuButton::Restart => self.dialogs.push(Dialog::SystemMenu { restart: true }),
            SysMenuButton::ExitGame => self.dialogs.push(Dialog::SystemMenu { restart: false }),
            SysMenuButton::Community => self.toggle_board(desktop),
            SysMenuButton::Macro => self.toggle_macro_list(desktop),
            SysMenuButton::Options => {
                if desktop.window(WindowId::OPTION).is_some_and(|w| w.visible) {
                    self.sound(interface_sound::WINDOW_CLOSE);
                    set_visible(desktop, WindowId::OPTION, false);
                } else {
                    self.sound(interface_sound::WINDOW_OPEN);
                    set_visible(desktop, WindowId::OPTION, true);
                }
            }
            SysMenuButton::Help => self.pending.push(UiCommand::ToggleHelp { macro_help: false }),
            SysMenuButton::Petition => self.petition.toggle_begin(desktop),
        }
    }

    fn toggle_board(&mut self, desktop: &mut Desktop) {
        if desktop.window(WindowId::BOARD).is_some_and(|w| w.visible) {
            self.sound(interface_sound::WINDOW_CLOSE);
            set_visible(desktop, WindowId::BOARD, false);
        } else {
            self.send(NetCommand::ShowBoard);
            self.sound(interface_sound::WINDOW_OPEN);
        }
    }

    fn toggle_macro_list(&mut self, desktop: &mut Desktop) {
        if desktop.window(WindowId::MACRO_LIST).is_some_and(|w| w.visible) {
            self.sound(interface_sound::WINDOW_CLOSE);
            set_visible(desktop, WindowId::MACRO_LIST, false);
        } else {
            self.macros.open(desktop);
            self.sound(interface_sound::WINDOW_OPEN);
        }
    }

    pub fn system_menu_restart(&mut self, desktop: &mut Desktop) {
        set_visible(desktop, WindowId::MESSENGER, false);
        set_visible(desktop, SYSMENU_WINDOW, false);
    }

    pub fn bar_drop_target(over: Option<(WindowId, i32, i32)>, cursor: (i32, i32), bar: Rect) -> DropTarget {
        match over {
            Some((WindowId::SHORTCUT, x, y)) => DropTarget::Bar(x, y),
            _ => killer_ui::shortcut::outside_drop_target(bar, cursor.0, cursor.1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectResult {
    FocusChat,
    ChatTab(i32),
    GroupCompact,
}

pub fn clan_label(world: &World, clan_id: i32) -> (Option<String>, i32, Option<String>) {
    let Some(record) = world.pledges.get(&clan_id) else { return (None, 0, None) };
    let crest = (record.crest_id > 0 && world.pledge_window.crests.contains_key(&record.crest_id)).then(|| crate::ui2d::crest_texture(record.crest_id));
    (Some(record.name.clone()), record.crest_id, crest)
}

pub fn detail_status_view(world: &World) -> DetailStatusView {
    let me = world.me.as_ref();
    let stats = &world.stats;
    let class_id = match me.map(|m| &m.kind) {
        Some(killer_world::EntityKind::Player { class_id, .. }) => *class_id,
        _ => 0,
    };
    DetailStatusView {
        name: me.map(|m| m.name.clone()).unwrap_or_default(),
        nick: me.map(|m| m.title.clone()).unwrap_or_default(),
        karma: me.map_or(0, |m| m.karma),
        clan_name: clan_label(world, world.social.clan_id).0,
        clan_crest_id: clan_label(world, world.social.clan_id).1,
        clan_crest: clan_label(world, world.social.clan_id).2,
        level: me.map_or(0, |m| m.level),
        class_id: u32::try_from(class_id).unwrap_or(0),
        hp: me.and_then(|m| m.hp).unwrap_or((0, 0)),
        mp: me.and_then(|m| m.mp).unwrap_or((0, 0)),
        cp: me.and_then(|m| m.cp).unwrap_or((0, 0)),
        exp: me.map_or(0, |m| i64::from(m.exp)),
        sp: stats.sp,
        load: stats.load,
        max_load: stats.max_load,
        p_atk: stats.p_atk,
        p_def: stats.p_def,
        accuracy: stats.accuracy,
        critical: stats.critical,
        atk_spd: stats.atk_spd,
        m_atk: stats.m_atk,
        m_def: stats.m_def,
        evasion: stats.evasion,
        speed: me.map_or(0, |m| (m.run_speed as f64 * m.speed_multiplier) as i32),
        cast_spd: stats.cast_spd,
        stats: [stats.str_, stats.dex, stats.con, stats.int_, stats.wit, stats.men],
        // Os modificadores dos símbolos, que o `HennaInfo` (`0xe4`) manda: o
        // `NCDetailStatusWnd::OnPaint @ 0x10111790` lê `+0x14c` ao lado do `STR`, `+0x164` do
        // `DEX`, `+0x154` do `CON`, `+0x144` do `INT`, `+0x16c` do `WIT` e `+0x15c` do `MEN`.
        stat_bonus: world.henna.info.as_ref().map_or([0; 6], |h| {
            [h.str_, h.dex, h.con, h.int_, h.wit, h.men].map(i32::from)
        }),
        pvp_kills: world.social.pvp_kills,
        pk_kills: world.social.pk_kills,
        recom_have: i32::from(world.social.recom_have),
        recom_left: i32::from(world.social.recom_left),
        hero: world.social.hero,
        noble: world.social.noble,
    }
}

pub fn virtual_key(code: KeyCode) -> Option<u32> {
    let vk = match code {
        KeyCode::KeyA => 0x41,
        KeyCode::KeyB => 0x42,
        KeyCode::KeyC => 0x43,
        KeyCode::KeyD => 0x44,
        KeyCode::KeyE => 0x45,
        KeyCode::KeyF => 0x46,
        KeyCode::KeyG => 0x47,
        KeyCode::KeyH => 0x48,
        KeyCode::KeyI => 0x49,
        KeyCode::KeyJ => 0x4a,
        KeyCode::KeyK => 0x4b,
        KeyCode::KeyL => 0x4c,
        KeyCode::KeyM => 0x4d,
        KeyCode::KeyN => 0x4e,
        KeyCode::KeyO => 0x4f,
        KeyCode::KeyP => 0x50,
        KeyCode::KeyQ => 0x51,
        KeyCode::KeyR => 0x52,
        KeyCode::KeyS => 0x53,
        KeyCode::KeyT => 0x54,
        KeyCode::KeyU => 0x55,
        KeyCode::KeyV => 0x56,
        KeyCode::KeyW => 0x57,
        KeyCode::KeyX => 0x58,
        KeyCode::KeyY => 0x59,
        KeyCode::KeyZ => 0x5a,
        KeyCode::Digit0 => 0x30,
        KeyCode::Digit1 => 0x31,
        KeyCode::Digit2 => 0x32,
        KeyCode::Digit3 => 0x33,
        KeyCode::Digit4 => 0x34,
        KeyCode::Digit5 => 0x35,
        KeyCode::Digit6 => 0x36,
        KeyCode::Digit7 => 0x37,
        KeyCode::Digit8 => 0x38,
        KeyCode::Digit9 => 0x39,
        KeyCode::F1 => VK_F1,
        KeyCode::F2 => VK_F1 + 1,
        KeyCode::F3 => VK_F1 + 2,
        KeyCode::F4 => VK_F1 + 3,
        KeyCode::F5 => VK_F1 + 4,
        KeyCode::F6 => VK_F1 + 5,
        KeyCode::F7 => VK_F1 + 6,
        KeyCode::F8 => VK_F1 + 7,
        KeyCode::F9 => VK_F1 + 8,
        KeyCode::F10 => VK_F1 + 9,
        KeyCode::F11 => VK_F1 + 10,
        KeyCode::F12 => VK_F1 + 11,
        KeyCode::Tab => keys::VK_TAB,
        KeyCode::Enter | KeyCode::NumpadEnter => keys::VK_RETURN,
        KeyCode::Escape => keys::VK_ESCAPE,
        KeyCode::PageUp => keys::VK_PRIOR,
        KeyCode::PageDown => keys::VK_NEXT,
        KeyCode::End => keys::VK_END,
        KeyCode::Home => keys::VK_HOME,
        KeyCode::Minus => keys::VK_OEM_MINUS,
        KeyCode::Equal => keys::VK_OEM_PLUS,
        KeyCode::Backquote => keys::VK_OEM_3,
        KeyCode::ShiftLeft | KeyCode::ShiftRight => keys::VK_SHIFT,
        KeyCode::ControlLeft | KeyCode::ControlRight => keys::VK_CONTROL,
        KeyCode::AltLeft | KeyCode::AltRight => keys::VK_MENU,
        KeyCode::Space => 0x20,
        KeyCode::Backspace => 0x08,
        _ => return None,
    };
    Some(vk)
}

fn pledge_rights(world: &World) -> Option<killer_ui::pledge::PledgeRights> {
    let me = world.me.as_ref()?;
    let [invite, title, crest, war] = world.social.pledge_rights;
    Some(killer_ui::pledge::PledgeRights {
        leader: killer_ui::empower::may_authorize(me.relation),
        invite,
        title,
        crest,
        war,
        in_clan: me.clan_id > 0,
        noble: world.social.noble,
    })
}

const DEMO_PLEDGE_ID: i32 = 268_435_457;
const DEMO_QUESTS: [(i32, i32); 2] = [(1, 2), (2, 1)];

pub fn demo_fill(world: &mut World) {
    use l2_net::game::{InventoryItem, Shortcut, SkillEntry};
    world.inventory.items = DEMO_ITEMS
        .iter()
        .map(|&(item_id, object_id, count, equipped, body_part)| InventoryItem {
            type1: 0,
            object_id,
            item_id,
            count,
            type2: 0,
            custom_type1: 0,
            equipped,
            body_part,
            enchant: 0,
            custom_type2: 0,
        })
        .collect();
    world.inventory_limit = DEMO_INVENTORY_LIMIT;
    world.stats.load = DEMO_LOAD.0;
    world.stats.max_load = DEMO_LOAD.1;
    world.skills = DEMO_SKILLS.iter().map(|&(passive, level, id)| SkillEntry { passive, level, id }).collect();
    world.shortcuts = DEMO_SHORTCUTS
        .iter()
        .map(|&(slot, id)| {
            let kind = match slot {
                0 => ShortcutKind::Item { object_id: id },
                1 | 2 => ShortcutKind::Skill { skill_id: id, level: 1 },
                _ => ShortcutKind::Action { id },
            };
            Shortcut { slot, kind, character_type: 1 }
        })
        .collect();
    world.social.clan_id = DEMO_PLEDGE_ID;
    let member = |name: &str, level: i32, class_id: i32, online: i32| l2_net::game::PledgeMember {
        name: name.into(),
        level,
        class_id,
        unknown: 0,
        flag: 0,
        online_object_id: online,
    };
    world.pledge_window.members = vec![member("Luzal", 40, 2, 7), member("Kerchief", 31, 11, 0), member("Bremmer", 25, 22, 9)];
    world.pledge_window.info = Some(l2_net::game::PledgeShowMemberListAll {
        pledge_id: DEMO_PLEDGE_ID,
        name: "Killer".into(),
        leader_name: "Luzal".into(),
        crest_id: 0,
        level: 3,
        castle_id: 0,
        hideout_id: 0,
        my_level: 40,
        dissolving: 0,
        ally_id: 0,
        ally_name: String::new(),
        ally_crest_id: 0,
        at_war: false,
        members: world.pledge_window.members.clone(),
    });
    world.windows.quests = DEMO_QUESTS.iter().map(|&(quest_id, cond)| l2_net::game::QuestState { quest_id, cond }).collect();
    let _ = SHORTCUT_SLOTS_PER_PAGE;
}

/// `/useshortcut <página> <slot>` (e as variantes force/stand): o original lê os dois números,
/// valida página em 1..=10 e slot em 1..=12, e acha o índice achatado com
/// `slot + -0xd + página * 0xc`, isto é `(página - 1) * 12 + (slot - 1)`
/// (`FUN_10058330`, casos `0x1b`/`0x1c`/`0x1d`). Fora dessas faixas o original não faz nada.
fn macro_shortcut_slot(args: &[&str]) -> Option<usize> {
    let page: i64 = args.first()?.parse().ok()?;
    let slot: i64 = args.get(1)?.parse().ok()?;
    let inside = (1..=10).contains(&page) && (1..=12).contains(&slot);
    inside.then(|| usize::try_from((page - 1) * 12 + (slot - 1)).ok())?
}

/// Tabela de placeholders de nome achada em `0x1023a910` (`nwindow.dll`), usada pelo tokenizer
/// de comandos de texto de macro (`FUN_1004bc10`/`FUN_1004bc70` @ `0x1004bc10`/`0x1004bc70`)
/// para resolver o argumento de comandos como `/target`: `%target`, `%self`, `%pet`,
/// `%party1`..`%party8`. Qualquer outro texto é tratado como nome literal.
pub const CHAT_WARNING_COLOR: u32 = 0xFFB09B79;

const COMMAND_TOKEN_NAMES: [&str; 11] =
    ["%target", "%self", "%pet", "%party1", "%party2", "%party3", "%party4", "%party5", "%party6", "%party7", "%party8"];

pub fn expand_name_token(world: &World, token: &str) -> Option<String> {
    let which = COMMAND_TOKEN_NAMES.iter().position(|name| name.eq_ignore_ascii_case(token))?;
    match which {
        0 => world.target.and_then(|id| world.entity(id)).map(|e| e.name.clone()),
        1 => world.me.as_ref().map(|e| e.name.clone()),
        2 => world.pet.as_ref().map(|pet| pet.info.name.clone()).filter(|name| !name.is_empty()),
        _ => world.party.members.get(which - 3).map(|m| m.name.clone()),
    }
}

fn resolve_macro_name_placeholder(world: &World, arg: &str) -> Option<String> {
    match arg {
        "%target" => world.target.and_then(|id| world.entity(id)).map(|e| e.name.clone()),
        "%self" => world.me.as_ref().map(|e| e.name.clone()),
        "%pet" => world.pet.as_ref().and_then(|pet| world.entity(pet.info.object_id)).map(|e| e.name.clone()),
        _ if arg.starts_with("%party") => {
            let index: usize = arg["%party".len()..].parse().ok()?;
            world.party.members.get(index.checked_sub(1)?).map(|m| m.name.clone())
        }
        _ => Some(arg.to_string()),
    }
}

fn toggle(desktop: &mut Desktop, id: WindowId) {
    if let Some(window) = desktop.window_mut(id) {
        window.visible = !window.visible;
    }
}

pub(crate) fn item_tip_data(tables: &DatTables, item_id: i32, type2: i32, body_part: i32, custom1: i32, custom2: i32) -> killer_ui::item_tip::ItemTipData {
    let id = u32::try_from(item_id).unwrap_or(0);
    let mut data = killer_ui::item_tip::ItemTipData {
        type2,
        body_part,
        weight: tables.item_icons.get(id).map_or(0, |icon| icon.weight as i32),
        custom1,
        custom2,
        ..Default::default()
    };
    if let Some(weapon) = tables.weapongrp.get(&id) {
        data = data.with_weapon(weapon);
    } else if let Some(armor) = tables.armorgrp.get(&id) {
        data = data.with_armor(armor);
    } else if let Some(etc) = tables.etcitemgrp.get(&id) {
        data = data.with_etc(etc);
    }
    data
}

fn player_target(world: &World) -> Option<&killer_world::Entity> {
    world.targeted().filter(|target| matches!(target.kind, killer_world::EntityKind::Player { .. }))
}

fn quest_node(tables: Option<&DatTables>, quest_id: i32, cond: i32) -> Option<QuestNode> {
    let lookup = |st: i32| tables.and_then(|t| t.questname.get(&(quest_id, st)));
    let first = lookup(1).or_else(|| states_of(cond).into_iter().find_map(lookup))?;
    let steps = states_of(cond)
        .into_iter()
        .filter_map(|st| {
            lookup(st).map(|q| QuestStep {
                state: st,
                sub_name: if q.sub_name.chars().count() <= 1 { q.title.clone() } else { q.sub_name.clone() },
                description: q.description.clone(),
                target_name: q.target_name.clone(),
                journal_disp: q.journal_disp != 0,
                counts_disp: q.unknown != 0,
                target_loc: q.target_loc,
                minimap_only: q.minimap_only != 0,
                items: q
                    .item_ids
                    .iter()
                    .zip(&q.item_nums)
                    .take(QUEST_ITEMS_MAX)
                    .map(|(&item_id, &required)| QuestItem {
                        item_id,
                        icon: tables
                            .and_then(|t| u32::try_from(item_id).ok().and_then(|id| t.item_icons.get(id)))
                            .and_then(|icon| icon.icons.first().map(|name| name.trim().to_string()))
                            .unwrap_or_default(),
                        name: tables
                            .and_then(|t| u32::try_from(item_id).ok().and_then(|id| t.itemname.get(&id)))
                            .map(|name| name.name.clone())
                            .unwrap_or_default(),
                        required,
                        current: 0,
                    })
                    .collect(),
                expanded: false,
                selected: false,
            })
        })
        .collect();
    Some(QuestNode {
        quest_id,
        title: first.title.clone(),
        quest_type: first.quest_type,
        level: first.level,
        max_level: first.quest_level,
        steps,
        expanded: false,
        selected: false,
    })
}

fn set_visible(desktop: &mut Desktop, id: WindowId, visible: bool) {
    if let Some(window) = desktop.window_mut(id) {
        window.visible = visible;
    }
}

pub const APP_BOUND_KEYS: [KeyCode; 8] = [
    KeyCode::KeyW,
    KeyCode::KeyA,
    KeyCode::KeyS,
    KeyCode::KeyD,
    KeyCode::KeyF,
    KeyCode::KeyH,
    KeyCode::ShiftLeft,
    KeyCode::ShiftRight,
];

pub fn hooked_key(code: KeyCode, chat_focused: bool, alt: bool) -> Option<u32> {
    if !alt && !chat_focused && APP_BOUND_KEYS.contains(&code) {
        return None;
    }
    virtual_key(code)
}

pub fn key_message(code: KeyCode, alt: bool, pressed: bool, repeat: bool) -> (u32, u32) {
    let msg = match (alt, pressed) {
        (true, true) => keys::WM_SYSKEYDOWN,
        (true, false) => keys::WM_SYSKEYUP,
        (false, true) => keys::WM_KEYDOWN,
        (false, false) => keys::WM_KEYUP,
    };
    let scan = if code == KeyCode::F10 { keys::F10_SCAN_DOWN } else { 0 };
    let mut hi = scan;
    if alt {
        hi |= 0x2000;
    }
    if repeat || !pressed {
        hi |= 0x4000;
    }
    if !pressed {
        hi |= 0x8000;
    }
    (msg, hi << 16)
}

#[cfg(test)]
pub(crate) fn silent(commands: Vec<UiCommand>) -> Vec<UiCommand> {
    commands.into_iter().filter(|command| !matches!(command, UiCommand::InterfaceSound(_))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::ini::WindowsInfo;
    use killer_ui::keys::{WM_KEYDOWN, WM_SYSKEYDOWN};
    use killer_platform::time::Duration;
    use l2_net::game::{InventoryItem, ItemList, Shortcut};

    #[test]
    fn expand_token_1004bc30_matches_the_eleven_words_exactly_and_names_the_pet_from_its_info() {
        let mut world = World { me: Some(crate::ui2d::tests::me()), ..World::default() };
        let mut target = crate::ui2d::tests::me();
        target.object_id = 42;
        target.name = "Alvo".into();
        world.entities.insert(42, target);
        world.target = Some(42);
        world.party.members.push(l2_net::game::packets::PartyMember {
            object_id: 5, name: "Membro".into(), cp: 0, max_cp: 0, hp: 0, max_hp: 0, mp: 0, max_mp: 0, level: 1, class_id: 0,
        });
        world.pet = Some(killer_world::Pet { info: l2_net::game::packets::PetInfo { name: "Fido".into(), object_id: 9, ..Default::default() }, max_time: 0, remaining_time: 0 });
        let me = world.me.as_ref().unwrap().name.clone();
        assert_eq!(expand_name_token(&world, "%TARGET").as_deref(), Some("Alvo"));
        assert_eq!(expand_name_token(&world, "%Self"), Some(me));
        assert_eq!(expand_name_token(&world, "%pet").as_deref(), Some("Fido"));
        assert_eq!(expand_name_token(&world, "%party1").as_deref(), Some("Membro"));
        assert_eq!(expand_name_token(&world, "%party2"), None);
        assert_eq!(expand_name_token(&world, "%party+1"), None);
        assert_eq!(expand_name_token(&world, "%party01"), None);
        assert_eq!(expand_name_token(&world, "%party9"), None);
        assert_eq!(expand_name_token(&world, "alvo"), None);
    }

    #[test]
    fn request_action_use_1007e980_party_and_trade_send_specific_packets() {
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        world.me = Some(crate::ui2d::tests::me());
        let mut target = crate::ui2d::tests::me();
        target.object_id = 42;
        target.name = "Parceiro".into();
        world.entities.insert(42, target);
        world.target = Some(42);
        keys.main.tab = MainTab::Actions;
        keys.actions.actions = vec![
            ActionEntry { id: 3, category: 1, ..Default::default() },
            ActionEntry { id: 7, category: 2, ..Default::default() },
        ];
        for distribution in 0..5 {
            keys.party_looting = distribution;
            for (id, rect, expected) in [
                (3, actions::LIST_RECTS[0], NetCommand::Trade { object_id: 42 }),
                (7, actions::LIST_RECTS[1], NetCommand::JoinParty { name: "Parceiro".into(), item_distribution: distribution }),
            ] {
                let x = PAGE_RECT.x + rect.x + 4;
                let y = PAGE_RECT.y + rect.y + 4;
                assert!(keys.mouse_down(WindowId::MAIN, x, y, &mut desktop));
                keys.mouse_up(Some((WindowId::MAIN, x, y)), &mut desktop, &mut bar, &world);
                assert_eq!(silent(keys.take_commands()), vec![UiCommand::Net(expected.clone())]);
                world.shortcuts = vec![Shortcut { slot: 0, kind: ShortcutKind::Action { id }, character_type: 1 }];
                keys.use_shortcut(&world, 0);
                assert_eq!(keys.take_commands(), vec![UiCommand::Net(expected.clone())]);
                world.apply_world(&WorldEvent::SendMacroList(macro_with(vec![cmd(0, 2, id, 0)])));
                keys.run_macro(&world, 7);
                assert_eq!(keys.take_commands(), vec![UiCommand::Net(expected.clone())]);
                let bytes = killer_world::net::encode(&expected).unwrap();
                if id == 3 {
                    assert_eq!(bytes, [0x15, 42, 0, 0, 0]);
                } else {
                    assert_eq!(bytes[0], 0x29);
                    assert_eq!(&bytes[bytes.len()-4..], &distribution.to_le_bytes());
                }
            }
        }
        world.target = None;
        keys.use_action(&world, 7, false, false);
        keys.use_action(&world, 3, false, false);
        assert_eq!(keys.take_commands(), vec![UiCommand::ColoredSystemMessage { id: 185, color: killer_ui::color::GOLD.0 }]);
    }

    #[test]
    fn use_action_1007e8d0_party_matching_resets_both_combos_and_55_toggles_the_recording() {
        let (mut keys, _, _, world) = setup();
        keys.party_match.location.selected = 7;
        keys.party_match.level.selected = 4;
        keys.use_action(&world, 11, false, false);
        assert_eq!(keys.take_commands(), vec![UiCommand::Net(NetCommand::PartyMatchList { level: 1, location: -1, mode: 1 })]);
        assert_eq!((keys.party_match.location.selected, keys.party_match.level.selected), (1, 1));
        keys.use_action(&world, 55, false, false);
        assert_eq!(keys.take_commands(), vec![UiCommand::ReplayRecord(ReplayRecord::Toggle)]);
    }

    #[test]
    #[ignore]
    fn request_action_use_1007e980_routes_every_original_action() {
        let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let actions = l2_dat::actionname::read(&client.read_system_file("actionname-e.dat").unwrap()).unwrap();
        assert_eq!(actions.len(), 88);
        let (mut keys, _, _, mut world) = setup();
        keys.relocalize_actions(&DatTables { actions: actions.clone(), ..Default::default() });
        world.me = Some(crate::ui2d::tests::me());
        let mut target = crate::ui2d::tests::me();
        target.object_id = 42;
        target.name = "Parceiro".into();
        world.entities.insert(42, target);
        world.target = Some(42);
        for action in &actions {
            let id = action.id as i32;
            keys.use_action(&world, id, false, false);
            let commands = keys.take_commands();
            if matches!(id, 12..=14 | 24..=26 | 29..=31 | 33..=35) {
                assert_eq!(commands, vec![UiCommand::Net(NetCommand::SocialAction { action: action.kind })], "{}", action.name);
            } else if matches!(id, 2..=11 | 18 | 28 | 40 | 50 | 55 | 56) {
                assert!(!commands.iter().any(|command| matches!(command, UiCommand::Net(NetCommand::UseAction { .. }))), "{}", action.name);
            } else {
                assert_eq!(commands, vec![UiCommand::Net(NetCommand::UseAction { id, ctrl: false, shift: false })], "{}", action.name);
            }
        }
    }

    #[test]
    fn clicking_a_macro_runs_it_and_the_edit_icon_opens_the_editor() {
        let (mut keys, mut desktop, _, mut world) = setup();
        let list = l2_net::game::SendMacroList {
            flag: 0,
            revision: 1,
            total_count: 1,
            macros: vec![l2_net::game::packets::Macro {
                id: 10,
                name: "Meu macro".into(),
                description: "desc".into(),
                acronym: "MM".into(),
                icon: 1,
                commands: vec![
                    l2_net::game::packets::MacroCommand { index: 0, kind: 1, skill_id: 1177, shortcut_id: 1, command: String::new() },
                    l2_net::game::packets::MacroCommand { index: 1, kind: 3, skill_id: 0, shortcut_id: 0, command: "/target %self".into() },
                    l2_net::game::packets::MacroCommand { index: 2, kind: 6, skill_id: 5, shortcut_id: 0, command: String::new() },
                ],
            }],
        };
        // a janela monta e reconstrói os comandos pelo texto, então precisa das duas tabelas
        keys.commandname = [("useskill".to_string(), 0x18), ("delay".to_string(), 0x4c)].into_iter().collect();
        keys.skillname.insert(
            (1177, 1),
            l2_dat::skillname::SkillName {
                id: 1177,
                level: 1,
                name: "Wind Strike".into(),
                description: String::new(),
                desc_add1: String::new(),
                desc_add2: String::new(),
            },
        );
        let event = WorldEvent::SendMacroList(list);
        world.apply_world(&event);
        keys.macros_event(&event, &world, &mut desktop);
        keys.macros.open(&mut desktop);
        assert!(desktop.window(WindowId::MACRO_LIST).unwrap().visible);
        assert_eq!(keys.macros.list.macros.len(), 1, "macro list deveria ter 1 macro depois do SendMacroList");

        // clica na primeira célula da grade de macros (linha 0, coluna 0)
        let (cx, cy) = (killer_ui::macros::ARRAY_RECT.x + 6, killer_ui::macros::ARRAY_RECT.y + 6);
        keys.mouse_down(WindowId::MACRO_LIST, cx, cy, &mut desktop);
        assert_eq!(keys.macros.list.selected, Some(0), "clique na célula devia selecionar o macro");
        assert!(
            !desktop.window(WindowId::MACRO_INFO).unwrap().visible,
            "apertar na célula não abre janela nenhuma (0x100a9760 só seleciona)"
        );
        assert!(!desktop.window(WindowId::MACRO_EDIT).unwrap().visible, "nem o editor, ainda");
        assert_eq!(keys.macros.info.macro_entry.as_ref().map(|m| m.name.as_str()), Some("Meu macro"));

        // soltar sobre a MESMA célula executa o macro (0x100a9aec liga a flag +0xa34)
        keys.macros.mouse_up(WindowId::MACRO_LIST, cx, cy, &crate::ui_macros::MacroTables { skillname: &keys.skillname, commandname: &keys.commandname }, &mut desktop);
        assert_eq!(keys.macros.run, Some(10), "clicar na célula executa o macro");
        assert!(!desktop.window(WindowId::MACRO_EDIT).unwrap().visible, "e não abre janela nenhuma");
        assert!(!desktop.window(WindowId::MACRO_INFO).unwrap().visible);
        keys.macros.run = None;

        // arrastar pro ícone de edição é o que abre o editor (0x100a9e96)
        let icon = killer_ui::macros::EDIT_ICON_RECT;
        keys.mouse_down(WindowId::MACRO_LIST, cx, cy, &mut desktop);
        keys.macros.mouse_up(WindowId::MACRO_LIST, icon.x + 2, icon.y + 2, &crate::ui_macros::MacroTables { skillname: &keys.skillname, commandname: &keys.commandname }, &mut desktop);
        assert_eq!(keys.macros.run, None, "arrastar não executa");
        assert!(desktop.window(WindowId::MACRO_EDIT).unwrap().visible, "o ícone de edição abre MACRO_EDIT");

        // arrastar pra lixeira pede confirmação antes de apagar (0x100a9b47, mensagem 0x33c)
        let trash = killer_ui::macros::TRASH_ICON_RECT;
        keys.mouse_down(WindowId::MACRO_LIST, cx, cy, &mut desktop);
        keys.macros.mouse_up(WindowId::MACRO_LIST, trash.x + 2, trash.y + 2, &crate::ui_macros::MacroTables { skillname: &keys.skillname, commandname: &keys.commandname }, &mut desktop);
        assert_eq!(keys.macros.confirm_delete, Some((10, "Meu macro".to_string())));
        assert_eq!(keys.macros.edit.name.get_text().as_deref(), Some("Meu macro"), "editor devia vir com o nome do macro selecionado");
        assert_eq!(keys.macros.edit.acronym.get_text().as_deref(), Some("MM"));

        assert_eq!(keys.macros.edit.focus, killer_ui::macros::EditFocus::Command(0), "o editor abre com a primeira linha focada (0x100aaad0)");
        keys.macros.edit.focus(killer_ui::macros::EditFocus::Name);
        // digita no campo nome e salva com Enter
        keys.macros.edit.set_font(&crate::ui2d::tests::small_font());
        keys.macros.edit.key(killer_ui::chat::InputKey::Backspace);
        keys.macros.edit.key(killer_ui::chat::InputKey::Char('X'));
        assert_eq!(keys.macros.edit.name.get_text().as_deref(), Some("Meu macrX"), "digitar devia alterar o texto do campo ativo");
        keys.macro_key(killer_ui::chat::InputKey::Enter, &mut desktop);
        assert!(desktop.window(WindowId::MACRO_EDIT).unwrap().visible, "Enter nas caixas só avisa o pai, que não o trata");
        assert!(keys.take_commands().is_empty());
        let save = killer_ui::macros::EDIT_SAVE_RECT;
        keys.macros.mouse_down(WindowId::MACRO_EDIT, save.x + 2, save.y + 2, &mut desktop);
        assert!(keys.take_commands().is_empty(), "o botão age ao soltar");
        let tables = crate::ui_macros::MacroTables { skillname: &keys.skillname, commandname: &keys.commandname };
        keys.macros.mouse_up(WindowId::MACRO_EDIT, save.x + 2, save.y + 2, &tables, &mut desktop);
        assert!(!desktop.window(WindowId::MACRO_EDIT).unwrap().visible, "Salvar fecha o editor");
        let commands = keys.take_commands();
        assert_eq!(commands.len(), 1, "{commands:?}");
        assert!(matches!(&commands[0], UiCommand::Net(NetCommand::MakeMacro(m)) if m.name == "Meu macrX"), "{commands:?}");
        // O que o editor devolve pro servidor tem que ser o mesmo comando que veio dele: o
        // `RequestMakeMacro` (`ccdcS` por comando) leva tipo, skill e atalho, não só o texto.
        let UiCommand::Net(NetCommand::MakeMacro(saved)) = &commands[0] else { panic!("{commands:?}") };
        let sent: Vec<(u8, i32, u8, &str)> =
            saved.commands.iter().map(|c| (c.kind, c.skill_id, c.shortcut_id, c.command.as_str())).collect();
        assert_eq!(sent, [(1, 1177, 1, ""), (3, 0, 0, "/target %self"), (6, 5, 0, "")]);
        assert_eq!(saved.commands.iter().map(|c| c.index).collect::<Vec<_>>(), [1, 2, 3], "o número do comando começa em 1");
    }

    #[test]
    fn using_a_macro_shortcut_sends_a_command_per_recognized_kind() {
        let (mut keys, _desktop, mut bar, mut world) = setup();
        keys.commandname = [("target".to_string(), 0xc), ("delay".to_string(), 0x4c)].into_iter().collect();
        world.apply_world(&WorldEvent::SendMacroList(l2_net::game::SendMacroList {
            flag: 0,
            revision: 1,
            total_count: 1,
            macros: vec![l2_net::game::packets::Macro {
                id: 10,
                name: "Meu macro".into(),
                description: String::new(),
                acronym: "MM".into(),
                icon: 1,
                commands: vec![
                    l2_net::game::packets::MacroCommand { index: 0, kind: 1, skill_id: 1177, shortcut_id: 0, command: String::new() },
                    l2_net::game::packets::MacroCommand { index: 1, kind: 3, skill_id: 0, shortcut_id: 0, command: "/TARGET loqiita".into() },
                    l2_net::game::packets::MacroCommand { index: 2, kind: 3, skill_id: 0, shortcut_id: 0, command: "/DELAY 5".into() },
                    l2_net::game::packets::MacroCommand { index: 3, kind: 5, skill_id: 0, shortcut_id: 0, command: String::new() },
                ],
            }],
        }));
        world.shortcuts = vec![l2_net::game::Shortcut { slot: 0, kind: ShortcutKind::Macro { id: 10 }, character_type: 1 }];
        bar.slots = vec![ShortcutSlot::new(SlotKind::Macro { id: 10 }, "")];
        bar.use_slot(0);
        keys.drain_bar(&mut bar, &world);
        let commands = run_until_macro_ends(&mut keys, &world);
        assert_eq!(
            commands,
            vec![
                UiCommand::Net(NetCommand::UseSkill { skill_id: 1177, ctrl: false, shift: false }),
                UiCommand::Log("comando de macro /target: \"loqiita\" não está visível".into()),
                // "/delay" não tem caso próprio no dispatcher original: cai no `default:`, que
                // manda o token cru como user command.
                UiCommand::Net(NetCommand::UserCommand { command: 0x4c }),
                UiCommand::Log("comando de macro \"\" (tipo 5) ainda não portado".into()),
            ]
        );
    }

    #[test]
    fn a_macro_target_command_resolves_placeholders_before_selecting() {
        let (mut keys, _desktop, mut bar, mut world) = setup();
        keys.commandname = [("target".to_string(), 0xc)].into_iter().collect();
        world.me = Some(crate::ui2d::tests::me());
        world.target = Some(9);
        world.entities.insert(9, killer_world::Entity { object_id: 9, name: "Inimigo".into(), ..crate::ui2d::tests::me() });
        world.party.members = vec![l2_net::game::packets::PartyMember {
            object_id: 3,
            name: "Companheiro".into(),
            cp: 0,
            max_cp: 0,
            hp: 1,
            max_hp: 1,
            mp: 0,
            max_mp: 0,
            level: 1,
            class_id: 0,
        }];
        world.apply_world(&WorldEvent::SendMacroList(l2_net::game::SendMacroList {
            flag: 0,
            revision: 1,
            total_count: 1,
            macros: vec![l2_net::game::packets::Macro {
                id: 11,
                name: "Alvo".into(),
                description: String::new(),
                acronym: "AL".into(),
                icon: 1,
                commands: vec![
                    l2_net::game::packets::MacroCommand { index: 0, kind: 3, skill_id: 0, shortcut_id: 0, command: "/target %target".into() },
                    l2_net::game::packets::MacroCommand { index: 1, kind: 3, skill_id: 0, shortcut_id: 0, command: "/target %self".into() },
                    l2_net::game::packets::MacroCommand { index: 2, kind: 3, skill_id: 0, shortcut_id: 0, command: "/target %party1".into() },
                ],
            }],
        }));
        world.shortcuts = vec![l2_net::game::Shortcut { slot: 0, kind: ShortcutKind::Macro { id: 11 }, character_type: 1 }];
        bar.slots = vec![ShortcutSlot::new(SlotKind::Macro { id: 11 }, "")];
        bar.use_slot(0);
        keys.drain_bar(&mut bar, &world);
        assert_eq!(
            run_until_macro_ends(&mut keys, &world),
            vec![
                UiCommand::Net(NetCommand::Action { object_id: 9, origin: [0, 0, 0], shift: false }),
                UiCommand::Net(NetCommand::Action { object_id: world.me.as_ref().unwrap().object_id, origin: [0, 0, 0], shift: false }),
                UiCommand::Log("comando de macro /target: \"Companheiro\" não está visível".into()),
            ]
        );
    }

    #[test]
    fn deleting_a_macro_deletes_the_shortcut_slots_that_hold_it() {
        let (mut keys, mut desktop, _bar, mut world) = setup();
        world.shortcuts = vec![
            l2_net::game::Shortcut { slot: 3, kind: ShortcutKind::Macro { id: 42 }, character_type: 1 },
            l2_net::game::Shortcut { slot: 4, kind: ShortcutKind::Macro { id: 43 }, character_type: 1 },
        ];
        let event = WorldEvent::SendMacroList(l2_net::game::SendMacroList { flag: 0, revision: 42, total_count: 0, macros: vec![] });
        world.apply_world(&event);
        keys.macros_event(&event, &world, &mut desktop);
        assert_eq!(keys.take_commands(), vec![UiCommand::Net(NetCommand::ShortcutDel { slot: 3 })]);
    }

    /// Caminho real de produção do macro, do jeito que o jogo roda: o `sync` monta a barra a
    /// partir de `world.shortcuts`, o clique vira `BarEvent::Use`, o `drain_bar` chama o
    /// `run_macro` e os `sync` seguintes avançam os comandos. Os outros testes de macro montavam
    /// `bar.slots` na mão e chamavam `bar.use_slot`, pulando justamente esse trecho.
    #[test]
    fn a_click_on_a_macro_slot_of_the_bar_runs_the_whole_macro() {
        let (mut keys, _desktop, mut bar, mut world) = setup();
        keys.commandname = command_tokens();
        world.apply_world(&WorldEvent::SendMacroList(l2_net::game::SendMacroList {
            flag: 0,
            revision: 1,
            total_count: 1,
            macros: vec![l2_net::game::packets::Macro {
                id: 42,
                name: "Buffar".into(),
                description: String::new(),
                acronym: "BF".into(),
                icon: 1,
                commands: vec![
                    l2_net::game::packets::MacroCommand { index: 0, kind: 1, skill_id: 1177, shortcut_id: 0, command: String::new() },
                    l2_net::game::packets::MacroCommand { index: 1, kind: 1, skill_id: 1178, shortcut_id: 0, command: String::new() },
                ],
            }],
        }));
        world.shortcuts = vec![l2_net::game::Shortcut { slot: 0, kind: ShortcutKind::Macro { id: 42 }, character_type: 1 }];
        let now = Instant::now();
        keys.sync(&world, None, &mut bar, now);
        let slot = bar.slot(0).cloned().expect("o sync tem que pôr o macro na barra");
        assert_eq!(slot.kind, Some(SlotKind::Macro { id: 42 }));
        assert_eq!(slot.macro_acronym, "BF");
        assert!(slot.icon.starts_with(killer_ui::macros::ICON_PREFIX));
        let rect = bar.slot_rect(0);
        let point = (rect.x + rect.w / 2, rect.y + rect.h / 2);
        assert_eq!(bar.slot_at(point.0, point.1), Some(0));
        bar.mouse_down(point.0, point.1);
        keys.sync(&world, None, &mut bar, now);
        bar.mouse_up(killer_ui::shortcut::DropTarget::Bar(point.0, point.1));
        keys.drain_bar(&mut bar, &world);
        assert!(keys.macro_run.is_some(), "o clique tinha que começar o macro");
        let mut out = keys.take_commands();
        let mut now = now;
        for _ in 0..8 {
            if keys.macro_run.is_none() {
                break;
            }
            cast_pending_skill(&mut keys, &world);
            now += Duration::from_secs(1);
            keys.sync(&world, None, &mut bar, now);
            out.extend(keys.take_commands());
        }
        assert_eq!(
            out,
            vec![
                UiCommand::Net(NetCommand::UseSkill { skill_id: 1177, ctrl: false, shift: false }),
                UiCommand::Net(NetCommand::UseSkill { skill_id: 1178, ctrl: false, shift: false }),
            ]
        );
    }

    /// Avança o macro em andamento até acabar, juntando tudo que ele emitiu. Cada passo pula um
    /// minuto pra frente, então qualquer atraso já expirou — o objetivo aqui é a ORDEM, não o
    /// tempo (o tempo tem teste próprio).
    fn run_until_macro_ends(keys: &mut KeyWindows, world: &World) -> Vec<UiCommand> {
        let mut out = keys.take_commands();
        let mut now = Instant::now();
        for _ in 0..64 {
            if keys.macro_run.is_none() {
                break;
            }
            cast_pending_skill(keys, world);
            now += Duration::from_secs(60);
            keys.tick_macro(world, now);
            out.extend(keys.take_commands());
        }
        assert!(keys.macro_run.is_none(), "o macro devia ter terminado");
        out
    }

    /// O servidor confirmando a magia que o comando de skill pediu — é o que destrava o macro,
    /// como a `0x100748d0` faz no original.
    fn cast_pending_skill(keys: &mut KeyWindows, world: &World) {
        let Some(skill_id) = keys.macro_run.as_ref().and_then(|run| run.waiting_skill) else { return };
        let caster_id = world.me.as_ref().map_or(0, |me| me.object_id);
        keys.macro_cast_started(&accepted_cast(caster_id, caster_id, skill_id, 0));
    }

    fn accepted_cast(caster_id: i32, me: i32, skill_id: i32, hit_time: i32) -> killer_world::magic::AcceptedCast {
        killer_world::magic::AcceptedCast {
            caster_id,
            target_id: caster_id,
            skill_id,
            skill_level: 1,
            hit_time,
            reuse_delay: 0,
            caster_is_me: caster_id == me,
            variant: killer_world::magic::CastVariant::Pawn,
        }
    }

    /// Roda um comando de texto de macro isolado, com a tabela `commandname-e.dat` real do
    /// cliente quando ela estiver disponível (senão cai nos tokens fixos do teste).
    fn text_command(keys: &mut KeyWindows, world: &World, command: &str) -> Vec<UiCommand> {
        keys.run_macro_text_command(world, command);
        keys.take_commands()
    }

    fn open_options(keys: &mut KeyWindows, desktop: &mut Desktop) {
        keys.sysmenu_action(SysMenuButton::Options, desktop);
        keys.pending.retain(|command| !matches!(command, UiCommand::InterfaceSound(_)));
    }

    fn command_tokens() -> HashMap<String, u32> {
        [
            ("sit", 0x4), ("stand", 0x5), ("walk", 0x6), ("run", 0x7), ("attack", 0x8),
            ("attackforce", 0x9), ("attackstand", 0xa), ("trade", 0xb), ("targetnext", 0xd),
            ("pickup", 0xe), ("petcollect", 0x42), ("invite", 0x10), ("leave", 0x11), ("vendor", 0x13),
            ("socialhello", 0x15), ("useskill", 0x18), ("useshortcut", 0x1b),
            ("useshortcutforce", 0x1c), ("block", 0x2c), ("unblock", 0x2d), ("sitstand", 0x39),
            ("unsummon", 0x6c), ("allywarstart", 0x28), ("loc", 0x0), ("time", 0x4d),
        ]
        .into_iter()
        .map(|(name, token)| (name.to_string(), token))
        .collect()
    }

    #[test]
    fn a_skill_or_action_dropped_on_a_macro_edit_line_writes_its_command_there() {
        let (mut keys, _desktop, _bar, _world) = setup();
        keys.commandname = command_tokens();
        keys.action_commands = [(0, "sitstand"), (2, "attack"), (5, "Pickup"), (12, "socialhello")]
            .into_iter()
            .map(|(id, cmd)| (id, cmd.to_string()))
            .collect();
        keys.skills.skills.push(killer_ui::skills::SkillView { id: 1177, level: 1, name: "Wind Strike".into(), ..Default::default() });
        keys.skills.skills.push(killer_ui::skills::SkillView { id: 1204, level: 1, passive: true, name: "Passive".into(), ..Default::default() });
        keys.skillname.insert((1177, 1), l2_dat::skillname::SkillName { id: 1177, level: 1, name: "Wind Strike".into(), description: String::new(), desc_add1: String::new(), desc_add2: String::new() });
        let rect = killer_ui::macros::EDIT_COMMAND_LIST_RECT;
        let (x, y) = (rect.x + killer_ui::macros::EDIT_LINE_BOX.x + 4, rect.y + killer_ui::macros::EDIT_LINE_BOX.y + 2);
        keys.drop_skill_on_macro_line(1177, x, y);
        assert_eq!(keys.macros.edit.lines[0].get_text().as_deref(), Some("/useskill Wind Strike"));
        keys.drop_skill_on_macro_line(1204, x, y + killer_ui::macros::EDIT_COMMAND_STEP);
        assert_eq!(keys.macros.edit.lines[1].get_text().as_deref().unwrap_or(""), "");
        keys.drop_action_on_macro_line(0, x, y + killer_ui::macros::EDIT_COMMAND_STEP);
        assert_eq!(keys.macros.edit.lines[1].get_text().as_deref(), Some("/sitstand"));
        keys.take_commands();
        keys.drop_action_on_macro_line(2, x, y + 2 * killer_ui::macros::EDIT_COMMAND_STEP);
        assert_eq!(keys.macros.edit.lines[2].get_text().as_deref(), Some("/attack"));
        keys.drop_action_on_macro_line(5, x, y + 3 * killer_ui::macros::EDIT_COMMAND_STEP);
        assert_eq!(keys.macros.edit.lines[3].get_text().as_deref(), Some("/pickup"));
        keys.drop_action_on_macro_line(0x14, x, y + 4 * killer_ui::macros::EDIT_COMMAND_STEP);
        assert_eq!(keys.take_commands(), vec![UiCommand::SystemMessage(MACRO_FORBIDDEN_ACTION_SYSMSG)]);
        assert_eq!(keys.macros.edit.lines[4].get_text().as_deref().unwrap_or(""), "");
        keys.drop_skill_on_macro_line(1177, rect.x - 5, y);
    }

    #[test]
    fn macro_text_commands_follow_the_original_dispatcher_case_by_case() {
        let (mut keys, _desktop, _bar, mut world) = setup();
        keys.commandname = command_tokens();
        world.me = Some(crate::ui2d::tests::me());
        // FUN_10068560/FUN_10068570: sentar/levantar e andar/correr.
        assert_eq!(text_command(&mut keys, &world, "/sit"), vec![UiCommand::Net(NetCommand::ChangeWaitType { standing: false })]);
        assert_eq!(text_command(&mut keys, &world, "/stand"), vec![UiCommand::Net(NetCommand::ChangeWaitType { standing: true })]);
        assert_eq!(text_command(&mut keys, &world, "/walk"), vec![UiCommand::Net(NetCommand::ChangeMoveType { running: false })]);
        assert_eq!(text_command(&mut keys, &world, "/run"), vec![UiCommand::Net(NetCommand::ChangeMoveType { running: true })]);
        // FUN_1007e980(id, 0, 0): ação de actionname-e.dat com o id da tabela de evidência.
        assert_eq!(text_command(&mut keys, &world, "/sitstand"), vec![UiCommand::Net(NetCommand::UseAction { id: 0, ctrl: false, shift: false })]);
        assert_eq!(text_command(&mut keys, &world, "/unsummon"), vec![UiCommand::Net(NetCommand::UseAction { id: 0x34, ctrl: false, shift: false })]);
        // FUN_10068550(id): ação social.
        assert_eq!(text_command(&mut keys, &world, "/socialhello"), vec![UiCommand::Net(NetCommand::SocialAction { action: 2 })]);
        // Party e loja.
        assert_eq!(text_command(&mut keys, &world, "/leave"), vec![UiCommand::Net(NetCommand::WithdrawParty)]);
        assert_eq!(text_command(&mut keys, &world, "/vendor"), vec![UiCommand::Net(NetCommand::PrivateStoreManageSell)]);
        assert_eq!(
            text_command(&mut keys, &world, "/invite Fulano"),
            vec![UiCommand::Net(NetCommand::JoinParty { name: "Fulano".into(), item_distribution: 0 })]
        );
        assert_eq!(
            text_command(&mut keys, &world, "/block Fulano"),
            vec![UiCommand::Net(NetCommand::Block { kind: 0, name: "Fulano".into() })]
        );
        assert_eq!(
            text_command(&mut keys, &world, "/unblock Fulano"),
            vec![UiCommand::Net(NetCommand::Block { kind: 1, name: "Fulano".into() })]
        );
        // Casos que o próprio original deixa vazios nesta build.
        assert_eq!(text_command(&mut keys, &world, "/allywarstart"), vec![]);
        // `default:` do original — vira user command com o token cru.
        assert_eq!(text_command(&mut keys, &world, "/loc"), vec![UiCommand::Net(NetCommand::UserCommand { command: 0 })]);
        assert_eq!(text_command(&mut keys, &world, "/time"), vec![UiCommand::Net(NetCommand::UserCommand { command: 0x4d })]);
    }

    fn macro_with(commands: Vec<l2_net::game::packets::MacroCommand>) -> l2_net::game::SendMacroList {
        l2_net::game::SendMacroList {
            flag: 0,
            revision: 1,
            total_count: 1,
            macros: vec![l2_net::game::packets::Macro {
                id: 7,
                name: "M".into(),
                description: String::new(),
                acronym: "M".into(),
                icon: 1,
                commands,
            }],
        }
    }

    fn cmd(index: u8, kind: u8, skill_id: i32, shortcut_id: u8) -> l2_net::game::packets::MacroCommand {
        l2_net::game::packets::MacroCommand { index, kind, skill_id, shortcut_id, command: String::new() }
    }

    /// O original roda um comando por vez e só avança quando o atraso do atual acaba
    /// (`FUN_100ab100`). `kind == 6` é a espera pura: o inteiro do campo `+0xe4` vira o atraso em
    /// segundos (`fildl 0xe4(%edi); fstps 0xf0(%edi)` em `FUN_100a8120`).
    #[test]
    fn a_macro_waits_the_delay_command_before_running_the_next_one() {
        let (mut keys, _desktop, mut bar, mut world) = setup();
        world.me = Some(crate::ui2d::tests::me());
        world.apply_world(&WorldEvent::SendMacroList(macro_with(vec![
            cmd(0, 1, 1177, 0),
            cmd(1, 6, 3, 0), // /delay 3
            cmd(2, 1, 1204, 0),
        ])));
        world.shortcuts = vec![l2_net::game::Shortcut { slot: 0, kind: ShortcutKind::Macro { id: 7 }, character_type: 1 }];
        bar.slots = vec![ShortcutSlot::new(SlotKind::Macro { id: 7 }, "")];
        bar.use_slot(0);
        keys.drain_bar(&mut bar, &world);
        let start = Instant::now();
        // o primeiro comando sai na hora, sozinho
        assert_eq!(
            keys.take_commands(),
            vec![UiCommand::Net(NetCommand::UseSkill { skill_id: 1177, ctrl: false, shift: false })]
        );
        // o comando 0 é uma skill: o macro fica parado até o servidor confirmar a magia
        keys.tick_macro(&world, start + Duration::from_secs(60));
        assert!(keys.macro_run.as_ref().unwrap().waiting_skill.is_some(), "esperando a magia começar");
        assert_eq!(keys.take_commands(), vec![], "nada avança enquanto a magia não começa");
        cast_pending_skill(&mut keys, &world);
        let start = Instant::now();
        keys.tick_macro(&world, start);
        assert_eq!(keys.take_commands(), vec![], "o /delay em si não manda nada");
        // 1 s da espera do lançamento (hit_time 0 → 0 / 1000 + 1) e então os 3 s do /delay
        keys.tick_macro(&world, start + Duration::from_millis(1_100));
        assert_eq!(keys.take_commands(), vec![]);
        keys.tick_macro(&world, start + Duration::from_millis(4_000));
        assert_eq!(keys.take_commands(), vec![]);
        assert!(keys.macro_run.is_some(), "ainda esperando");
        keys.tick_macro(&world, start + Duration::from_millis(4_200));
        assert_eq!(
            keys.take_commands(),
            vec![UiCommand::Net(NetCommand::UseSkill { skill_id: 1204, ctrl: false, shift: false })]
        );
        keys.tick_macro(&world, start + Duration::from_secs(10));
        assert!(keys.macro_run.is_some(), "a última skill também espera a confirmação");
        cast_pending_skill(&mut keys, &world);
        keys.tick_macro(&world, start + Duration::from_secs(20));
        assert!(keys.macro_run.is_none(), "acabaram os comandos");
    }

    /// Os onze verbos que faltavam dos 110 do `commandname-e.dat`, comparando caso a caso com o
    /// `switch` de `FUN_10058330`. Os que no original abrem janela do cliente não viram pacote.
    #[test]
    fn the_last_eleven_macro_verbs_follow_the_original_case_by_case() {
        let (mut keys, _desktop, _bar, mut world) = setup();
        keys.commandname = [
            ("gm".to_string(), 0x1), ("gmcancel".to_string(), 0x2), ("allydismiss".to_string(), 0x20),
            ("allycrest".to_string(), 0x22), ("findprivatestore".to_string(), 0x54),
            ("clanwarstart".to_string(), 0x56), ("clanwarstop".to_string(), 0x57),
            ("channelinvite".to_string(), 0x5e), ("channelkick".to_string(), 0x5f),
            ("start_videorecording".to_string(), 0x67), ("end_videorecording".to_string(), 0x68),
        ]
        .into_iter()
        .collect();
        world.me = Some(crate::ui2d::tests::me());
        world.entities.insert(9, killer_world::Entity { object_id: 9, name: "Inimigo".into(), ..crate::ui2d::tests::me() });
        world.target = Some(9);

        // Gmcancel -> RequestPetitionCancel
        assert_eq!(
            text_command(&mut keys, &world, "/gmcancel motivo"),
            vec![UiCommand::Net(NetCommand::PetitionCancel { content: "motivo".into() })]
        );
        assert!(text_command(&mut keys, &world, "/allydismiss").is_empty());
        assert_eq!(keys.dialogs.pop(), Some(Dialog::AllyDismissName));
        // clanwarstart/stop -> confirmação 0x5fb/0x5fc com o nome do alvo
        assert!(text_command(&mut keys, &world, "/clanwarstart").is_empty());
        assert_eq!(
            keys.dialogs.pop(),
            Some(Dialog::PledgeWarConfirm {
                message_id: PLEDGE_WAR_START_SYSMSG,
                pledge_name: "Inimigo".into(),
                start: true
            })
        );
        assert!(text_command(&mut keys, &world, "/clanwarstop Outro").is_empty());
        assert_eq!(
            keys.dialogs.pop(),
            Some(Dialog::PledgeWarConfirm {
                message_id: PLEDGE_WAR_STOP_SYSMSG,
                pledge_name: "Outro".into(),
                start: false
            })
        );
        // channelinvite/kick usam o argumento, ou o alvo quando vem vazio
        assert_eq!(
            text_command(&mut keys, &world, "/channelinvite Fulano"),
            vec![UiCommand::Net(NetCommand::ExAskJoinMpcc { name: "Fulano".into() })]
        );
        assert_eq!(
            text_command(&mut keys, &world, "/channelkick"),
            vec![UiCommand::Net(NetCommand::ExOustFromMpcc { name: "Inimigo".into() })]
        );
        world.target = None;
        assert_eq!(
            text_command(&mut keys, &world, "/channelinvite"),
            vec![UiCommand::ColoredSystemMessage { id: CHANNEL_NEEDS_TARGET_SYSMSG, color: killer_ui::color::GOLD.0 }]
        );
        // os que abrem janela do cliente não viram pacote nenhum
        assert_eq!(text_command(&mut keys, &world, "/gm texto"), vec![UiCommand::OpenPetition(Some("texto".into()))]);
        assert_eq!(text_command(&mut keys, &world, "/gm"), vec![UiCommand::OpenPetition(None)]);
        assert_eq!(text_command(&mut keys, &world, "/start_videorecording"), vec![UiCommand::ReplayRecord(ReplayRecord::Start)]);
        assert_eq!(text_command(&mut keys, &world, "/end_videorecording"), vec![UiCommand::ReplayRecord(ReplayRecord::Stop)]);
        for verb in ["/allycrest", "/findprivatestore Fulano"] {
            let out = text_command(&mut keys, &world, verb);
            assert!(
                matches!(out.as_slice(), [UiCommand::Log(_)]),
                "{verb} devia só registrar, saiu {out:?}"
            );
        }
    }

    /// Porte do parser do Save (`FUN_100aa7a0`), ramo a ramo. O editor guarda só texto; é ele
    /// que decide o que cada linha vira no `RequestMakeMacro`.
    #[test]
    fn the_save_parser_turns_each_line_into_the_command_of_the_original() {
        let commandname: HashMap<String, u32> = [
            ("useskill".to_string(), 0x18),
            ("useskillforce".to_string(), 0x19),
            ("useshortcut".to_string(), 0x1b),
            ("delay".to_string(), 0x4c),
            ("attack".to_string(), 0x8),
            ("logout".to_string(), 0x6e),
        ]
        .into_iter()
        .collect();
        let mut skillname = HashMap::new();
        skillname.insert(
            (1397, 1),
            l2_dat::skillname::SkillName {
                id: 1397,
                level: 1,
                name: "Chant of Victory".into(),
                description: String::new(),
                desc_add1: String::new(),
                desc_add2: String::new(),
            },
        );
        let parse = |line: &str| {
            macro_command_from_line(line, 0, &skillname, &commandname)
                .map(|c| (c.kind, c.skill_id, c.shortcut_id, c.command))
        };
        // 0x18: junta o resto das palavras e procura o nome na tabela de skills
        assert_eq!(parse("/useskill Chant of Victory"), Some((1, 1397, 1, String::new())));
        assert_eq!(parse("/useskill Nao Existe"), Some((1, 0, 0, String::new())), "nome desconhecido vira id 0");
        // 0x19 e 0x1a caem no texto livre
        assert_eq!(parse("/useskillforce Chant of Victory"), Some((3, 0, 0, "/useskillforce Chant of Victory".into())));
        // 0x1b: duas faixas, 1..10 e 1..12, gravadas menos um
        assert_eq!(parse("/useshortcut 2 3"), Some((4, 1, 2, String::new())));
        assert_eq!(parse("/useshortcut 11 3"), None, "página fora da faixa descarta a linha");
        assert_eq!(parse("/useshortcut 2 13"), None, "slot fora da faixa também");
        // 0x4c: um número maior ou igual a 1, no campo de 4 bytes
        assert_eq!(parse("/delay 5"), Some((6, 5, 0, String::new())));
        assert_eq!(parse("/delay 0"), None);
        assert_eq!(parse("/delay"), None);
        // token abaixo de 0x6e vira texto livre, daí pra cima a linha é descartada
        assert_eq!(parse("/attack"), Some((3, 0, 0, "/attack".into())));
        assert_eq!(parse("/logout"), None);
        // verbo que a tabela não conhece também é texto livre
        assert_eq!(parse("/nada disso"), Some((3, 0, 0, "/nada disso".into())));
        assert_eq!(parse(""), Some((3, 0, 0, String::new())));
    }

    fn save_commandname() -> HashMap<String, u32> {
        [("useskill", 0x18), ("useshortcut", 0x1b), ("delay", 0x4c), ("attack", 0x8), ("logout", 0x6e)]
            .into_iter()
            .map(|(name, token)| (name.to_string(), token))
            .collect()
    }

    fn lines(texts: &[&str]) -> Vec<String> {
        let mut all: Vec<String> = texts.iter().map(|text| text.to_string()).collect();
        all.resize(12, String::new());
        all
    }

    #[test]
    fn macro_validate_lines_100aa260_follows_the_branches_of_the_dll() {
        let names = save_commandname();
        let none = |_: i32, _: i32| false;
        assert!(!macro_validate_lines(&lines(&[]), &names, &none, false), "a primeira linha não pode ser vazia");
        assert!(!macro_validate_lines(&lines(&["/delay 3", "", "/delay 3"]), &names, &none, false), "sem texto depois de linha vazia");
        assert!(!macro_validate_lines(&lines(&["/delay 0"]), &names, &none, false));
        assert!(!macro_validate_lines(&lines(&["/delay"]), &names, &none, false));
        assert!(macro_validate_lines(&lines(&["/delay 2", "/delay 9"]), &names, &none, false));
        assert!(!macro_validate_lines(&lines(&["/useshortcut 11 1"]), &names, &none, false));
        assert!(!macro_validate_lines(&lines(&["/useshortcut 1 13"]), &names, &none, false));
        assert!(!macro_validate_lines(&lines(&["/useshortcut 2 3"]), &names, &|page, slot| (page, slot) == (2, 3), false), "atalho que é macro");
        assert!(macro_validate_lines(&lines(&["/useshortcut 2 3", "/delay 1"]), &names, &none, false));
        assert!(!macro_validate_lines(&lines(&["//gmmenu"]), &names, &none, false));
        assert!(macro_validate_lines(&lines(&["//gmmenu"]), &names, &none, true));
        assert!(macro_validate_lines(&lines(&["/attack", "/delay 0", "/useshortcut 99 99"]), &names, &none, false), "o primeiro texto simples encerra a checagem com válido");
        assert!(macro_validate_lines(&lines(&["olá"]), &names, &none, false), "texto sem barra é conversa");
        assert!(!macro_validate_lines(&lines(&["/logout"]), &names, &none, false), "token 0x6e ou mais");
        assert_eq!(macro_line_code("///x", &names), MACRO_CODE_BUILDER_ALT);
        assert_eq!(macro_line_code("//x", &names), MACRO_CODE_BUILDER);
        assert_eq!(macro_line_code("//", &names), MACRO_CODE_NONE);
        assert_eq!(macro_line_code("/", &names), MACRO_CODE_NONE);
        assert_eq!(macro_line_code("delay 3", &names), MACRO_CODE_CHAT, "sem a barra o verbo não vale");
    }

    #[test]
    fn macro_push_lines_100aa600_numbers_from_one_counts_the_filled_prefix_and_keeps_empty_lines() {
        let names = save_commandname();
        let skills = HashMap::new();
        let (count, records) = macro_save_records(&lines(&["/useshortcut 99 1", "/attack", "/delay 4", "", "olá"]), &skills, &names, false);
        assert_eq!(count, 3);
        let sent: Vec<_> = records.iter().map(|r| (r.index, r.kind, r.skill_id, r.shortcut_id, r.command.as_str())).collect();
        assert_eq!(sent[0], (2, 3, 0, 0, "/attack"), "o useshortcut inválido sai, mas gasta o número 1");
        assert_eq!(sent[1], (3, 6, 4, 0, ""));
        assert_eq!(sent[2], (4, 3, 0, 0, ""), "a linha vazia segue como texto vazio");
        assert_eq!(sent.len(), 11, "{sent:?}");
        let (count, records) = macro_save_records(&lines(&["/attack", "//x", "/logout"]), &skills, &names, false);
        assert_eq!(count, 3);
        assert!(records.iter().all(|r| r.command != "//x" && r.command != "/logout"), "builder sem nível e token 0x6e saem");
        let (_, builder) = macro_save_records(&lines(&["//x"]), &skills, &names, true);
        assert_eq!(builder[0].command, "//x");
    }

    /// A janela de macro não mostra o campo de texto cru: o original remonta cada linha no
    /// `NCMacroArrayWnd::OnLButtonUp @ 0x100a9a40`, com a `/` de `0x1019f24c` na frente e o verbo
    /// vindo do `commandname-e.dat`. Um comando de skill não tem texto nenhum — era por isso que
    /// a lista de comandos aparecia vazia.
    #[test]
    fn the_editor_rebuilds_the_line_of_each_command_like_the_original() {
        let (mut keys, mut desktop, _, mut world) = setup();
        keys.commandname =
            [("useskill".to_string(), 0x18), ("useshortcut".to_string(), 0x1b), ("delay".to_string(), 0x4c), ("sitstand".to_string(), 0x39)]
                .into_iter()
                .collect();
        keys.action_commands = [(0, "sitstand".to_string())].into_iter().collect();
        keys.skillname.insert(
            (1397, 1),
            l2_dat::skillname::SkillName {
                id: 1397,
                level: 1,
                name: "Chant of Victory".into(),
                description: String::new(),
                desc_add1: String::new(),
                desc_add2: String::new(),
            },
        );
        let list = l2_net::game::SendMacroList {
            flag: 0,
            revision: 1,
            total_count: 1,
            macros: vec![l2_net::game::packets::Macro {
                id: 10,
                name: "AIOX 1".into(),
                description: String::new(),
                acronym: "AIO1".into(),
                icon: 1,
                commands: vec![
                    cmd(0, 1, 1397, 1),
                    cmd(1, 1, 9999, 1),
                    cmd(2, 2, 0x00, 0),
                    l2_net::game::packets::MacroCommand { index: 3, kind: 3, skill_id: 0, shortcut_id: 0, command: "/target %self".into() },
                    cmd(4, 4, 1, 2),
                    cmd(5, 6, 5, 0),
                ],
            }],
        };
        let event = WorldEvent::SendMacroList(list);
        world.apply_world(&event);
        keys.macros_event(&event, &world, &mut desktop);
        let lines: Vec<&str> =
            keys.macros.list.macros[0].commands.iter().map(|command| command.label.as_str()).collect();
        assert_eq!(
            lines,
            [
                "/useskill Chant of Victory",
                "/useskill 9999",
                "/sitstand",
                "/target %self",
                "/useshortcut 2 3",
                "/delay 5",
            ]
        );
    }

    /// Um comando de skill não avança por tempo: o stepper do original volta sem mexer no índice
    /// enquanto o atraso do comando for zero e o `kind` for 1 (`0x100ab2bb`..`0x100ab2f7`). Quem
    /// destrava é a magia começando — a `0x100748d0` casa o `skill_id` do comando atual com o da
    /// magia e só então grava o tempo de espera. Sem isso o macro mandava as doze skills em um
    /// segundo e o servidor descartava onze.
    #[test]
    fn a_skill_command_waits_for_the_cast_to_start_and_then_for_its_cast_time() {
        let (mut keys, _desktop, mut bar, mut world) = setup();
        world.me = Some(crate::ui2d::tests::me());
        let me = world.me.as_ref().unwrap().object_id;
        world.apply_world(&WorldEvent::SendMacroList(macro_with(vec![cmd(0, 1, 1397, 1), cmd(1, 1, 1048, 1)])));
        world.shortcuts = vec![l2_net::game::Shortcut { slot: 0, kind: ShortcutKind::Macro { id: 7 }, character_type: 1 }];
        bar.slots = vec![ShortcutSlot::new(SlotKind::Macro { id: 7 }, "")];
        bar.use_slot(0);
        keys.drain_bar(&mut bar, &world);
        assert_eq!(
            keys.take_commands(),
            vec![UiCommand::Net(NetCommand::UseSkill { skill_id: 1397, ctrl: false, shift: false })]
        );
        let start = Instant::now();
        keys.tick_macro(&world, start + Duration::from_secs(600));
        assert_eq!(keys.take_commands(), vec![], "o tempo sozinho não avança um comando de skill");
        // magia de outro jogador, ou de outra skill, não destrava
        keys.macro_cast_started(&accepted_cast(me + 1, me, 1397, 2_000));
        keys.macro_cast_started(&accepted_cast(me, me, 1048, 2_000));
        assert_eq!(keys.macro_run.as_ref().unwrap().waiting_skill, Some(1397));
        keys.macro_cast_started(&accepted_cast(me, me, 1397, 2_000));
        let started = Instant::now();
        assert_eq!(keys.macro_run.as_ref().unwrap().waiting_skill, None);
        assert!((keys.macro_run.as_ref().unwrap().delay - 3.0).abs() < 0.01, "espera hit_time / 1000 + 1");
        keys.tick_macro(&world, started + Duration::from_millis(2_900));
        assert_eq!(keys.take_commands(), vec![], "ainda lançando");
        keys.tick_macro(&world, started + Duration::from_millis(3_100));
        assert_eq!(
            keys.take_commands(),
            vec![UiCommand::Net(NetCommand::UseSkill { skill_id: 1048, ctrl: false, shift: false })]
        );
    }

    /// `kind == 2` é ação (o oráculo externo chamava isso de "3 = action", errado), e o atraso de
    /// uma ação social é a duração da animação (`APawn::GetSocialAnimDuration`, mapeada pela
    /// tabela de `0x100a87cc`): ação 12 "socialhello" -> animação social 2.
    #[test]
    fn a_macro_action_waits_the_social_animation_it_plays() {
        let (mut keys, _desktop, mut bar, mut world) = setup();
        world.me = Some(crate::ui2d::tests::me());
        keys.action_social.insert(12, 2);
        keys.social_durations = vec![1.0, 1.0, 2.5, 0.0, 0.0];
        world.apply_world(&WorldEvent::SendMacroList(macro_with(vec![cmd(0, 2, 12, 0), cmd(1, 1, 1177, 0)])));
        world.shortcuts = vec![l2_net::game::Shortcut { slot: 0, kind: ShortcutKind::Macro { id: 7 }, character_type: 1 }];
        bar.slots = vec![ShortcutSlot::new(SlotKind::Macro { id: 7 }, "")];
        bar.use_slot(0);
        keys.drain_bar(&mut bar, &world);
        let start = Instant::now();
        assert_eq!(
            keys.take_commands(),
            vec![UiCommand::Net(NetCommand::SocialAction { action: 2 })]
        );
        keys.tick_macro(&world, start + Duration::from_millis(2_400));
        assert_eq!(keys.take_commands(), vec![], "a animação social 2 dura 2.5 s");
        keys.tick_macro(&world, start + Duration::from_millis(2_600));
        assert_eq!(
            keys.take_commands(),
            vec![UiCommand::Net(NetCommand::UseSkill { skill_id: 1177, ctrl: false, shift: false })]
        );
        // uma ação que não toca social não espera nada
        assert_eq!(keys.macro_action_delay(&world, 2), 0.0, "\"attack\" não tem animação social");
        // sitstand escolhe a animação pelo estado: de pé senta (social 0), sentado levanta (1)
        assert_eq!(keys.macro_action_delay(&world, MACRO_ACTION_SITSTAND), 1.0);
    }

    /// Segunda leva de comandos, todos com pacote que o port já tinha: aliança, amigos, lista de
    /// GMs e as duas fabricações (que no original são `FUN_10068500(<id>,0,0)`, a mesma coisa que
    /// `FUN_1007e980` — usar a ação de `actionname-e.dat`).
    #[test]
    fn macro_ally_friend_and_manufacture_commands_follow_the_dispatcher() {
        let (mut keys, _desktop, _bar, mut world) = setup();
        keys.commandname = [
            ("gmlist", 0x3), ("allyinvite", 0x1e), ("allyleave", 0x1f), ("allydissolve", 0x21),
            ("allyinfo", 0x23), ("friendinvite", 0x24), ("friendlist", 0x25), ("frienddel", 0x26),
            ("dwarvenmanufacture", 0x3b), ("generalmanufacture", 0x65),
        ]
        .into_iter()
        .map(|(n, t)| (n.to_string(), t))
        .collect();
        world.me = Some(crate::ui2d::tests::me());

        assert_eq!(text_command(&mut keys, &world, "/gmlist"), vec![UiCommand::Net(NetCommand::RequestGmList)]);
        assert_eq!(text_command(&mut keys, &world, "/allyinfo"), vec![UiCommand::Net(NetCommand::RequestAllyInfo)]);
        assert_eq!(text_command(&mut keys, &world, "/friendlist"), vec![UiCommand::Net(NetCommand::FriendList)]);
        assert_eq!(
            text_command(&mut keys, &world, "/friendinvite Fulano"),
            vec![UiCommand::Net(NetCommand::FriendInvite { name: "Fulano".into() })]
        );
        assert_eq!(
            text_command(&mut keys, &world, "/frienddel Fulano"),
            vec![UiCommand::Net(NetCommand::FriendDel { name: "Fulano".into() })]
        );
        // ação 37 "manufacture" e ação 51 "manufacture2" de actionname-e.dat
        assert_eq!(
            text_command(&mut keys, &world, "/dwarvenmanufacture"),
            vec![UiCommand::Net(NetCommand::UseAction { id: 37, ctrl: false, shift: false })]
        );
        assert_eq!(
            text_command(&mut keys, &world, "/generalmanufacture"),
            vec![UiCommand::Net(NetCommand::UseAction { id: 51, ctrl: false, shift: false })]
        );
        // allyinvite usa o alvo atual, e avisa por mensagem de sistema quando não tem alvo
        assert_eq!(
            text_command(&mut keys, &world, "/allyinvite"),
            vec![UiCommand::SystemMessage(ALLY_INVITE_NEEDS_TARGET_SYSMSG)]
        );
        world.target = Some(55);
        assert_eq!(
            text_command(&mut keys, &world, "/allyinvite"),
            vec![UiCommand::SystemMessage(ALLY_INVITE_SENT_SYSMSG), UiCommand::Net(NetCommand::JoinAlly { object_id: 55 })]
        );
        keys.commandname.insert("partymatching".into(), 0x14);
        keys.commandname.insert("deletealliancecrest".into(), 0x5b);
        keys.commandname.insert("equip".into(), 0x50);
        assert_eq!(
            text_command(&mut keys, &world, "/partymatching"),
            vec![UiCommand::Net(NetCommand::PartyMatchList { level: 1, location: -1, mode: 1 })]
        );
        assert_eq!(
            text_command(&mut keys, &world, "/deletealliancecrest"),
            vec![UiCommand::Net(NetCommand::SetAllyCrest { data: Vec::new() })]
        );
        // /equip usa a mesma conta de página+slot, mas só quando a célula guarda um item
        world.shortcuts = vec![
            l2_net::game::Shortcut { slot: 0, kind: ShortcutKind::Item { object_id: 909 }, character_type: 1 },
            l2_net::game::Shortcut { slot: 1, kind: ShortcutKind::Skill { skill_id: 3, level: 1 }, character_type: 1 },
        ];
        assert_eq!(
            text_command(&mut keys, &world, "/equip 1 1"),
            vec![UiCommand::Net(NetCommand::UseItem { object_id: 909, ctrl: false })]
        );
        assert_eq!(text_command(&mut keys, &world, "/equip 1 2"), vec![], "slot com skill não equipa nada");

        // sair e dissolver a aliança passam por confirmação antes de mandar qualquer pacote
        assert_eq!(text_command(&mut keys, &world, "/allyleave"), vec![]);
        assert_eq!(text_command(&mut keys, &world, "/allydissolve"), vec![]);
        assert_eq!(
            keys.dialogs,
            vec![
                Dialog::AllyConfirm { message_id: ALLY_LEAVE_SYSMSG, dissolve: false },
                Dialog::AllyConfirm { message_id: ALLY_DISSOLVE_SYSMSG, dissolve: true },
            ]
        );
    }

    #[test]
    fn macro_attack_and_trade_use_the_current_target() {
        let (mut keys, _desktop, _bar, mut world) = setup();
        keys.commandname = command_tokens();
        world.me = Some(crate::ui2d::tests::me());
        world.target = Some(42);
        let origin = [0, 0, 0];
        assert_eq!(
            text_command(&mut keys, &world, "/attack"),
            vec![UiCommand::Net(NetCommand::Action { object_id: 42, origin, shift: false }), UiCommand::DeleteLandMark]
        );
        assert_eq!(
            text_command(&mut keys, &world, "/attackforce"),
            vec![UiCommand::Net(NetCommand::AttackRequest { object_id: 42, origin, shift: false })]
        );
        assert_eq!(
            text_command(&mut keys, &world, "/attackstand"),
            vec![UiCommand::Net(NetCommand::Action { object_id: 42, origin, shift: true }), UiCommand::DeleteLandMark]
        );
        assert_eq!(text_command(&mut keys, &world, "/trade"), vec![UiCommand::Net(NetCommand::Trade { object_id: 42 })]);
        world.target = None;
        assert_eq!(text_command(&mut keys, &world, "/attack"), vec![], "sem alvo o original não faz nada");
    }

    #[test]
    fn macro_targetnext_pickup_and_petcollect_follow_the_engine_scans() {
        let (mut keys, _desktop, _bar, mut world) = setup();
        keys.commandname = command_tokens();
        let me = crate::ui2d::tests::me();
        world.me = Some(me.clone());
        let at = |id: i32, x: f32| killer_world::Entity {
            object_id: id,
            position: glam::Vec3::new(x, 0.0, 0.0),
            kind: EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true },
            ..me.clone()
        };
        world.entities.insert(2, at(2, 50.0));
        world.entities.insert(3, at(3, 120.0));
        world.entities.insert(5, at(5, 500.0));
        let mut friendly = at(6, 10.0);
        friendly.kind = EntityKind::Npc { npc_id: 2, rhand: 0, chest: 0, lhand: 0, attackable: false };
        world.entities.insert(6, friendly);
        let mut ground = at(4, 10.0);
        ground.kind = EntityKind::Item { item_id: 57, count: 1, stackable: true };
        world.entities.insert(4, ground);
        let action = |object_id: i32| vec![UiCommand::Net(NetCommand::Action { object_id, origin: [0, 0, 0], shift: false })];
        assert_eq!(text_command(&mut keys, &world, "/targetnext"), action(2));
        world.target = Some(2);
        assert_eq!(text_command(&mut keys, &world, "/targetnext"), action(3));
        world.target = Some(3);
        assert_eq!(text_command(&mut keys, &world, "/targetnext"), action(2), "sem mais ninguém adiante, volta ao mais próximo");
        assert_eq!(text_command(&mut keys, &world, "/pickup"), action(4));
        assert_eq!(text_command(&mut keys, &world, "/petcollect"), vec![], "sem pet nada é pedido");
        world.entities.remove(&4);
        world.entities.remove(&2);
        world.entities.remove(&3);
        world.target = None;
        assert_eq!(text_command(&mut keys, &world, "/targetnext"), vec![]);
    }

    #[test]
    fn macro_attack_with_ctrl_forces_and_clears_the_destination_mark() {
        let (mut keys, _desktop, _bar, mut world) = setup();
        keys.commandname = command_tokens();
        world.me = Some(crate::ui2d::tests::me());
        world.target = Some(42);
        keys.ctrl = true;
        assert_eq!(
            text_command(&mut keys, &world, "/attack"),
            vec![UiCommand::Net(NetCommand::AttackRequest { object_id: 42, origin: [0, 0, 0], shift: false }), UiCommand::DeleteLandMark]
        );
        keys.ctrl = false;
        keys.shift = true;
        assert_eq!(
            text_command(&mut keys, &world, "/attack"),
            vec![UiCommand::Net(NetCommand::Action { object_id: 42, origin: [0, 0, 0], shift: true }), UiCommand::DeleteLandMark]
        );
        assert_eq!(
            text_command(&mut keys, &world, "/attackforce"),
            vec![UiCommand::Net(NetCommand::AttackRequest { object_id: 42, origin: [0, 0, 0], shift: false })]
        );
    }

    #[test]
    fn macro_useshortcut_flattens_page_and_slot_like_the_original() {
        assert_eq!(macro_shortcut_slot(&["1", "1"]), Some(0));
        assert_eq!(macro_shortcut_slot(&["1", "12"]), Some(11));
        assert_eq!(macro_shortcut_slot(&["2", "1"]), Some(12));
        assert_eq!(macro_shortcut_slot(&["10", "12"]), Some(119));
        // fora das faixas 1..=10 e 1..=12 o original não faz nada
        assert_eq!(macro_shortcut_slot(&["0", "1"]), None);
        assert_eq!(macro_shortcut_slot(&["11", "1"]), None);
        assert_eq!(macro_shortcut_slot(&["1", "13"]), None);
        assert_eq!(macro_shortcut_slot(&["1"]), None);
        assert_eq!(macro_shortcut_slot(&["a", "b"]), None);
    }

    #[test]
    fn macro_useskill_resolves_the_skill_by_name_from_skillname() {
        let (mut keys, _desktop, _bar, mut world) = setup();
        keys.commandname = command_tokens();
        world.me = Some(crate::ui2d::tests::me());
        let named = |id: u32, level: u32, name: &str| l2_dat::SkillName {
            id,
            level,
            name: name.into(),
            description: String::new(),
            desc_add1: String::new(),
            desc_add2: String::new(),
        };
        keys.skillname = [((1177, 1), named(1177, 1, "Wind Strike")), ((1177, 3), named(1177, 3, "Wind Strike"))]
            .into_iter()
            .collect();
        assert_eq!(
            text_command(&mut keys, &world, "/useskill Wind Strike"),
            vec![UiCommand::Net(NetCommand::UseSkill { skill_id: 1177, ctrl: false, shift: false })],
            "o nome é o resto da linha inteira, não só a primeira palavra"
        );
        assert!(matches!(text_command(&mut keys, &world, "/useskill Nao Existe").as_slice(), [UiCommand::Log(_)]));
    }

    #[test]
    fn on_warehouse_done_100800d0_closes_no_windows_and_the_failure_goes_through_the_table_notice() {
        let (mut keys, mut desktop, _, mut world) = setup();
        let list = l2_net::game::WarehouseList { warehouse_type: 1, adena: 500, items: vec![] };
        world.apply_world(&WorldEvent::WarehouseDepositList(list.clone()));
        desktop.window_mut(WindowId::WAREHOUSE).unwrap().visible = true;
        for success in [true, false, true] {
            let event = WorldEvent::WarehouseDone(l2_net::game::WarehouseDone { success });
            world.apply_world(&event);
            keys.world_event(&event, &mut desktop, &world);
            assert_eq!(world.windows.warehouse, Some(list.clone()));
            assert!(desktop.window(WindowId(18)).unwrap().visible);
            assert!(keys.take_commands().is_empty());
            assert_eq!(killer_world::notice_of(&event).map(|notice| (notice.id, notice.color)), (!success).then_some((307, None)));
        }
    }

    fn setup() -> (KeyWindows, Desktop, ShortcutView, World) {
        let keys = KeyWindows::new(&HashMap::new(), false);
        let mut desktop = Desktop::new((1558, 933));
        keys.install(&mut desktop, &WindowsInfo::from_ini(&killer_ui::ini::parse_ini("")), (1558, 933));
        desktop.windows.push(Window::new(WindowId::SHORTCUT, Rect::new(1478, 168, 46, 504), Rect::new(0, 0, 46, 12)));
        let mut world = World { me: Some(crate::ui2d::tests::me()), ..Default::default() };
        demo_fill(&mut world);
        (keys, desktop, ShortcutView::new(true), world)
    }

    fn alt(keys: &mut KeyWindows, desktop: &mut Desktop, bar: &mut ShortcutView, world: &World, vk: u32) -> Vec<EffectResult> {
        let state = keys.key_state(world, false, false);
        let hooked = keys.hook(WM_SYSKEYDOWN, vk, 0x2000_0000, &state, bar);
        keys.apply_effects(&hooked, desktop, bar, world)
    }

    #[test]
    fn virtual_keys_and_messages_follow_win32() {
        assert_eq!(virtual_key(KeyCode::KeyT), Some(0x54));
        assert_eq!(virtual_key(KeyCode::Digit0), Some(0x30));
        assert_eq!(virtual_key(KeyCode::F12), Some(0x7b));
        assert_eq!(virtual_key(KeyCode::Tab), Some(9));
        assert_eq!(virtual_key(KeyCode::Escape), Some(0x1b));
        assert_eq!(virtual_key(KeyCode::Minus), Some(0xbd));
        assert_eq!(virtual_key(KeyCode::Backquote), Some(0xc0));
        assert_eq!(virtual_key(KeyCode::ArrowUp), None);
        assert_eq!(key_message(KeyCode::KeyT, true, true, false), (WM_SYSKEYDOWN, 0x2000 << 16));
        assert_eq!(key_message(KeyCode::KeyT, false, true, false), (WM_KEYDOWN, 0));
        assert_eq!(key_message(KeyCode::F10, false, true, false), (WM_KEYDOWN, 0x44 << 16));
        assert_eq!(key_message(KeyCode::F10, false, true, true), (WM_KEYDOWN, 0x4044 << 16));
        assert_eq!(key_message(KeyCode::F10, false, false, false), (keys::WM_KEYUP, 0xc044 << 16));
        assert_eq!(hooked_key(KeyCode::KeyW, false, false), None);
        assert_eq!(hooked_key(KeyCode::KeyW, false, true), Some(0x57));
        assert_eq!(hooked_key(KeyCode::KeyW, true, false), Some(0x57));
        assert_eq!(hooked_key(KeyCode::KeyT, false, false), Some(0x54));
    }

    #[test]
    fn option_resolution_100c3ec0_emits_backend_request_on_selection_not_arrow_press() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        keys.option = killer_ui::option::OptionView::new(|id| Some(id.to_string()));
        keys.option.video_page.combos[0].entries = vec!["800*600 32bit".into(), "1024*768 32bit".into()];
        keys.option.video_page.combos[0].text_height = 11;
        let arrow = keys.option.video_page.combos[0].arrow_rect();
        let y = arrow.y + killer_ui::option::PAGE_RECT.y + 1;
        assert!(keys.mouse_down(WindowId::OPTION, arrow.x + 1, y, &mut desktop));
        assert!(keys.take_commands().is_empty());
        keys.mouse_up(Some((WindowId::OPTION, arrow.x + 1, y)), &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().is_empty());
        assert!(keys.option.video_page.combos[0].open);
        let y = 48 + 11 + 17 + 17 + 1;
        keys.option.combo_mouse_move(108, y);
        assert!(keys.mouse_down(WindowId::OPTION, 108, y, &mut desktop));
        assert_eq!(keys.take_commands(), [UiCommand::OptionResolutionChanged(1)]);
        assert_eq!(keys.option.video_page.combos[0].selected, 1);
        assert!(!keys.option.video_page.combos[0].open);
        keys.mouse_up(Some((WindowId::OPTION, 108, y)), &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().is_empty());
    }

    #[test]
    fn alt_with_the_f_keys_and_digits_from_winit_reach_the_hook_as_page_changes() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        let f_keys = [
            KeyCode::F1,
            KeyCode::F2,
            KeyCode::F3,
            KeyCode::F4,
            KeyCode::F5,
            KeyCode::F6,
            KeyCode::F7,
            KeyCode::F8,
            KeyCode::F9,
            KeyCode::F10,
        ];
        for (page, code) in f_keys.into_iter().enumerate() {
            let vk = hooked_key(code, false, true).unwrap();
            assert_eq!(vk, VK_F1 + page as u32);
            let (msg, lparam) = key_message(code, true, true, false);
            assert_eq!(msg, WM_SYSKEYDOWN);
            let state = keys.key_state(&world, false, false);
            let hooked = keys.hook(msg, vk, lparam, &state, &mut bar);
            assert_eq!(hooked.effects, vec![Effect::ShortcutPage(page as i32)], "{code:?}");
            assert!(hooked.handled);
            keys.apply_effects(&hooked, &mut desktop, &mut bar, &world);
            assert_eq!(bar.page, page as i32, "{code:?}");
            let (msg, lparam) = key_message(code, true, false, false);
            let hooked = keys.hook(msg, vk, lparam, &state, &mut bar);
            assert!(hooked.effects.is_empty() && !hooked.handled);
        }
        let digits = [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
            KeyCode::Digit0,
        ];
        for (page, code) in digits.into_iter().enumerate() {
            let vk = hooked_key(code, false, true).unwrap();
            let (msg, lparam) = key_message(code, true, true, false);
            let state = keys.key_state(&world, false, false);
            let hooked = keys.hook(msg, vk, lparam, &state, &mut bar);
            assert!(hooked.effects.is_empty(), "{code:?} sem EnterChatting");
            keys.enter_chatting = true;
            let state = keys.key_state(&world, false, false);
            let hooked = keys.hook(msg, vk, lparam, &state, &mut bar);
            assert_eq!(hooked.effects, vec![Effect::ShortcutPage(page as i32)], "{code:?}");
            keys.enter_chatting = false;
        }
    }

    #[test]
    fn alt_t_opens_the_status_tab_and_the_same_key_closes_it() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        assert!(!desktop.window(WindowId::MAIN).unwrap().visible);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x54);
        assert!(desktop.window(WindowId::MAIN).unwrap().visible);
        assert_eq!(keys.main.tab, MainTab::Status);
        keys.take_commands();
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x4b);
        assert!(desktop.window(WindowId::MAIN).unwrap().visible);
        assert_eq!(keys.main.tab, MainTab::Skills);
        assert!(keys.take_commands().contains(&UiCommand::Net(NetCommand::RequestSkillList)), "abrir a janela de skills pede a lista");
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x4b);
        assert!(!desktop.window(WindowId::MAIN).unwrap().visible);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x49);
        assert!(desktop.window(WindowId::MAIN).unwrap().visible);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x58);
        assert!(desktop.window(SYSMENU_WINDOW).unwrap().visible);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x48);
        assert!(keys.hidden);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x71);
        assert_eq!(bar.page, 1);
        assert!(silent(keys.take_commands()).is_empty());
    }

    #[test]
    fn alt_p_dispatches_the_native_min_frame_rate_toggle() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        assert!(alt(&mut keys, &mut desktop, &mut bar, &world, 0x50).is_empty());
        assert_eq!(keys.take_commands(), [UiCommand::ToggleMinFrameRate]);
    }

    #[test]
    fn alt_v_requests_the_item_list_and_the_window_opens_on_show_window() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x56);
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::Net(NetCommand::RequestItemList)]);
        assert!(!desktop.window(WindowId::INVENTORY).unwrap().visible);
        keys.world_event(&WorldEvent::ItemList(ItemList { show_window: false, items: vec![] }), &mut desktop, &world);
        assert!(!desktop.window(WindowId::INVENTORY).unwrap().visible);
        keys.world_event(&WorldEvent::ItemList(ItemList { show_window: true, items: vec![] }), &mut desktop, &world);
        assert!(desktop.window(WindowId::INVENTORY).unwrap().visible);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x56);
        assert!(!desktop.window(WindowId::INVENTORY).unwrap().visible);
        assert!(silent(keys.take_commands()).is_empty());
        let state = keys.key_state(&world, false, false);
        let hooked = keys.hook(WM_KEYDOWN, keys::VK_ESCAPE, 0, &state, &mut bar);
        keys.apply_effects(&hooked, &mut desktop, &mut bar, &world);
        assert_eq!(keys.take_commands(), vec![UiCommand::Net(NetCommand::TargetCancel)]);
    }

    #[test]
    fn an_inventory_update_changes_the_count_and_a_removal_empties_the_shortcut_slot() {
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        keys.sync(&world, None, &mut bar, Instant::now());
        let potion = world.inventory.find(107).cloned().unwrap();
        let apply = |keys: &mut KeyWindows, desktop: &mut Desktop, world: &mut World, change: u16, count: i32| {
            let event = WorldEvent::InventoryUpdate(l2_net::game::InventoryUpdate {
                changes: vec![l2_net::game::ItemChange { change, item: InventoryItem { count, ..potion } }],
            });
            world.apply_world(&event);
            keys.world_event(&event, desktop, world);
        };
        apply(&mut keys, &mut desktop, &mut world, l2_net::game::packets::ITEM_MODIFIED, potion.count - 1);
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!(keys.inventory.items.iter().find(|item| item.object_id == 107).map(|item| item.count), Some(potion.count - 1));
        assert!(bar.slot(0).is_some_and(|slot| slot.kind == Some(SlotKind::Item { object_id: 107 })));
        apply(&mut keys, &mut desktop, &mut world, l2_net::game::packets::ITEM_REMOVED, 0);
        keys.sync(&world, None, &mut bar, Instant::now());
        assert!(bar.slot(0).is_none(), "0x10106180 esvazia o atalho do item removido");
        apply(&mut keys, &mut desktop, &mut world, l2_net::game::packets::ITEM_ADDED, 5);
        keys.sync(&world, None, &mut bar, Instant::now());
        assert!(bar.slot(0).is_some_and(|slot| slot.kind == Some(SlotKind::Item { object_id: 107 })));
    }

    #[test]
    fn a_recipe_shortcut_takes_name_and_icon_from_the_recipe_table_and_a_missing_item_shortcut_is_not_set() {
        let (mut keys, _, mut bar, mut world) = setup();
        let mut tables = DatTables::default();
        tables.recipes.insert(
            1,
            l2_dat::recipe::Recipe {
                name: "mk_arrow".into(),
                id: 1,
                recipe_id: 1666,
                level: 1,
                product_id: 17,
                product_num: 500,
                mp_consume: 30,
                success_rate: 100,
                materials: Vec::new(),
            },
        );
        tables.item_icons.insert_all(vec![l2_dat::itemicons::ItemIcon {
            id: 17,
            kind: l2_dat::itemicons::ItemKind::Etc,
            drop: Default::default(),
            icons: std::array::from_fn(|_| "icon.etc_arrow_i00".into()),
            weight: 1,
            type1: 4,
        }]);
        tables.itemname.insert(
            1666,
            l2_dat::itemname::ItemName { id: 1666, name: "Recipe: Wooden Arrow".into(), add_name: String::new(), description: String::new(), popup: 0 },
        );
        world.shortcuts = vec![
            l2_net::game::Shortcut { slot: 3, kind: ShortcutKind::Recipe { id: 1 }, character_type: 1 },
            l2_net::game::Shortcut { slot: 4, kind: ShortcutKind::Item { object_id: 999_999 }, character_type: 1 },
        ];
        keys.sync(&world, Some(&tables), &mut bar, Instant::now());
        let slot = bar.slot(3).unwrap();
        assert_eq!((slot.name.as_str(), slot.icon.as_str()), ("Recipe: Wooden Arrow", "icon.etc_arrow_i00"));
        assert!(bar.slot(4).is_none());
    }

    #[test]
    fn skill_cool_times_with_a_zero_total_or_a_remaining_above_the_total_start_nothing() {
        let (mut keys, _, mut bar, mut world) = setup();
        world.cooldowns = vec![
            l2_net::game::SkillReuse { skill_id: 1, level: 1, reuse: 0, remaining: 0 },
            l2_net::game::SkillReuse { skill_id: 2, level: 1, reuse: 5, remaining: 9 },
            l2_net::game::SkillReuse { skill_id: 3, level: 1, reuse: 9, remaining: 5 },
        ];
        keys.sync(&world, None, &mut bar, Instant::now());
        assert!(keys.cooldowns.contains_key(&3) && !keys.cooldowns.contains_key(&1) && !keys.cooldowns.contains_key(&2));
    }

    #[test]
    fn a_local_cast_starts_the_shortcut_cooldown_and_a_stranger_does_not() {
        let (mut keys, _desktop, mut bar, mut world) = setup();
        world.me = Some(crate::ui2d::tests::me());
        let me = world.me.as_ref().unwrap().object_id;
        let cast = |caster_id: i32| {
            WorldEvent::MagicSkillUse(l2_net::game::MagicSkillUse {
                caster_id,
                target_id: me,
                skill_id: 3,
                skill_level: 1,
                hit_time: 1500,
                reuse_delay: 12_999,
                x: 0,
                y: 0,
                z: 0,
                extra: Vec::new(),
                positions: Vec::new(),
            })
        };
        let now = Instant::now();
        let event = cast(me);
        world.apply_world(&event);
        keys.skill_reuse_started(&event, &world, now);
        assert_eq!(keys.cooldowns.get(&3).map(|c| (c.level, c.reuse, c.remaining)), Some((1, 12.0, 12.0)));
        assert_eq!(keys.cooldown_fraction(3, 1, now), Some(1.0));
        assert_eq!(keys.cooldown_fraction(3, 2, now), None, "o atalho precisa do mesmo nível");
        keys.cooldowns.clear();
        let stranger = cast(me + 1);
        world.apply_world(&stranger);
        keys.skill_reuse_started(&stranger, &world, now);
        assert!(keys.cooldowns.is_empty());
        let _ = &mut bar;
    }

    #[test]
    fn escape_is_ignored_in_the_camera_fixed_mode_and_alt_pageup_pagedown_flip_the_chat_tabs() {
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        world.me = Some(crate::ui2d::tests::me());
        world.me.as_mut().unwrap().sitting = true;
        let state = keys.key_state(&world, false, false);
        let hooked = keys.hook(WM_KEYDOWN, keys::VK_ESCAPE, 0, &state, &mut bar);
        keys.apply_effects(&hooked, &mut desktop, &mut bar, &world);
        assert_eq!(keys.take_commands(), vec![UiCommand::Net(NetCommand::TargetCancel)]);
        world.camera_fixed = true;
        let state = keys.key_state(&world, false, false);
        let hooked = keys.hook(WM_KEYDOWN, keys::VK_ESCAPE, 0, &state, &mut bar);
        keys.apply_effects(&hooked, &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().is_empty());
        assert_eq!(alt(&mut keys, &mut desktop, &mut bar, &world, keys::VK_PRIOR), vec![EffectResult::ChatTab(0)]);
        assert_eq!(alt(&mut keys, &mut desktop, &mut bar, &world, keys::VK_NEXT), vec![EffectResult::ChatTab(1)]);
    }

    #[test]
    fn an_empty_shortcut_slot_plays_click_failed_and_a_registration_plays_pickup() {
        let (mut keys, _desktop, _bar, world) = setup();
        keys.use_shortcut(&world, 200);
        assert_eq!(keys.take_commands(), vec![UiCommand::InterfaceSound("itemsound.click_failed")]);
        keys.send(NetCommand::ShortcutReg { kind: 1, slot: 1, id: 107, character_type: 1 });
        assert_eq!(
            keys.take_commands(),
            vec![UiCommand::Net(NetCommand::ShortcutReg { kind: 1, slot: 1, id: 107, character_type: 1 }), UiCommand::InterfaceSound("itemsound.pickup")]
        );
    }

    #[test]
    fn alt_windows_play_the_open_and_close_sounds_of_the_nwindow_table() {
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        world.me = Some(crate::ui2d::tests::me());
        let sounds = |keys: &mut KeyWindows| -> Vec<&'static str> {
            keys.take_commands().into_iter().filter_map(|command| if let UiCommand::InterfaceSound(name) = command { Some(name) } else { None }).collect()
        };
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x58);
        assert_eq!(sounds(&mut keys), ["InterfaceSound.system_open_01"]);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x58);
        assert_eq!(sounds(&mut keys), ["InterfaceSound.system_close_01"]);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x54);
        assert_eq!(sounds(&mut keys), ["InterfaceSound.charstat_open_01"]);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x54);
        assert_eq!(sounds(&mut keys), ["InterfaceSound.charstat_close_01"]);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x56);
        assert_eq!(sounds(&mut keys), ["InterfaceSound.inventory_open_01"]);
        set_visible(&mut desktop, WindowId::INVENTORY, true);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x56);
        assert_eq!(sounds(&mut keys), ["InterfaceSound.inventory_close_01"]);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x52);
        assert_eq!(sounds(&mut keys), ["itemsound.window_open"]);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x52);
        assert_eq!(sounds(&mut keys), ["itemsound.window_close"]);
    }

    #[test]
    fn the_demo_world_fills_the_bar_and_f_keys_use_its_slots() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        keys.sync(&world, None, &mut bar, Instant::now());
        assert!(bar.slot(0).is_some_and(|s| s.kind == Some(SlotKind::Item { object_id: 107 })));
        assert!(bar.slot(1).is_some_and(|s| s.kind == Some(SlotKind::Skill { id: 3, level: 1 })));
        assert!(bar.slot(3).is_some_and(|s| s.kind == Some(SlotKind::Action { id: 2 })));
        assert_eq!(keys.inventory.items.len(), DEMO_ITEMS.len());
        assert_eq!(keys.inventory.equipped().count(), 6);
        assert_eq!(keys.skills.listed().len(), 3);
        let state = keys.key_state(&world, false, false);
        let hooked = keys.hook(WM_KEYDOWN, VK_F1, 0, &state, &mut bar);
        keys.apply_effects(&hooked, &mut desktop, &mut bar, &world);
        keys.ctrl = true;
        let hooked = keys.hook(WM_KEYDOWN, VK_F1 + 1, 0, &state, &mut bar);
        keys.apply_effects(&hooked, &mut desktop, &mut bar, &world);
        let hooked = keys.hook(WM_KEYDOWN, VK_F1 + 3, 0, &state, &mut bar);
        keys.apply_effects(&hooked, &mut desktop, &mut bar, &world);
        assert_eq!(
            keys.take_commands(),
            vec![
                UiCommand::Net(NetCommand::UseItem { object_id: 107, ctrl: false }),
                UiCommand::Net(NetCommand::UseSkill { skill_id: 3, ctrl: true, shift: false }),
            ]
        );
    }

    #[test]
    fn inventory_and_skill_views_are_rebuilt_only_when_the_world_changes() {
        let (mut keys, _desktop, mut bar, mut world) = setup();
        let now = Instant::now();
        keys.sync(&world, None, &mut bar, now);
        let count = keys.inventory.items.len();
        assert!(count > 0);
        keys.inventory.items[0].name = "marcado".into();
        keys.skills.skills[0].name = "marcada".into();
        keys.sync(&world, None, &mut bar, now);
        assert_eq!(keys.inventory.items[0].name, "marcado");
        assert_eq!(keys.skills.skills[0].name, "marcada");
        world.inventory.items.pop();
        world.skills.pop();
        keys.sync(&world, None, &mut bar, now);
        assert_eq!(keys.inventory.items.len(), count - 1);
        assert_ne!(keys.inventory.items[0].name, "marcado");
        assert_ne!(keys.skills.skills[0].name, "marcada");
    }

    #[test]
    fn inventory_clicks_send_use_unequip_and_shortcut_registration() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        keys.sync(&world, None, &mut bar, Instant::now());
        keys.world_event(&WorldEvent::ItemList(ItemList { show_window: true, items: vec![] }), &mut desktop, &world);
        assert!(keys.right_click(WindowId::INVENTORY, 10 + 37 * 2, 190));
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::Net(NetCommand::UseItem { object_id: 108, ctrl: false })]);
        assert!(keys.right_click(WindowId::INVENTORY, 8 + 7 + 5, 31 + 45 + 5));
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::Net(NetCommand::Unequip { body_part: inventory::BODY_PART_R_HAND })]);
        assert!(keys.mouse_down(WindowId::INVENTORY, 10 + 37, 190, &mut desktop));
        assert_eq!(keys.inventory.selected, Some(107));
        keys.mouse_up(Some((WindowId::SHORTCUT, 7, 70)), &mut desktop, &mut bar, &world);
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::Net(NetCommand::ShortcutReg { kind: 1, slot: 1, id: 107, character_type: 1 })]);
        assert!(keys.mouse_down(WindowId::INVENTORY, 10 + 37, 190, &mut desktop));
        keys.mouse_up(Some((WindowId::INVENTORY, 15, 40)), &mut desktop, &mut bar, &world);
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::Net(NetCommand::UseItem { object_id: 107, ctrl: false })]);
        assert!(keys.mouse_down(WindowId::INVENTORY, 8 + 7 + 5, 31 + 45 + 5, &mut desktop));
        keys.mouse_up(Some((WindowId::INVENTORY, 10, 190)), &mut desktop, &mut bar, &world);
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::Net(NetCommand::Unequip { body_part: inventory::BODY_PART_R_HAND })]);
        assert!(keys.mouse_down(WindowId::INVENTORY, 150, 170, &mut desktop));
        assert_eq!(keys.inventory.tab, killer_ui::inventory::InventoryTab::Quest);
        assert!(keys.mouse_down(WindowId::INVENTORY, 240, 10, &mut desktop));
        assert!(!desktop.window(WindowId::INVENTORY).unwrap().visible);
    }

    #[test]
    fn modern_lock_blocks_inventory_registration_into_extra_rows() {
        let (mut keys,mut desktop,mut bar,world)=setup();
        keys.sync(&world,None,&mut bar,Instant::now());
        bar.modern=Some(killer_ui::modern_shortcut::ModernShortcuts { count:5,locked:true,..Default::default() });
        let rect=bar.modern.unwrap().slot_rect(4,7);
        for locked in [true,false] {
            bar.modern.as_mut().unwrap().locked=locked;
            assert!(keys.mouse_down(WindowId::INVENTORY,47,190,&mut desktop));
            keys.mouse_up(Some((WindowId::SHORTCUT,rect.x+1,rect.y+1)),&mut desktop,&mut bar,&world);
            let expected=if locked {vec![]}else{vec![UiCommand::Net(NetCommand::ShortcutReg {kind:1,slot:55,id:107,character_type:1})]};
            assert_eq!(silent(keys.take_commands()),expected);
        }
    }

    #[test]
    fn menu_and_macro_help_buttons_queue_their_local_help_pages() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x58);
        let rect = SysMenuButton::Help.rect();
        assert!(keys.mouse_down(SYSMENU_WINDOW, rect.x+1, rect.y+1, &mut desktop));
        keys.mouse_up(Some((SYSMENU_WINDOW, rect.x+1, rect.y+1)), &mut desktop, &mut bar, &world);
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::ToggleHelp { macro_help: false }]);
        assert!(desktop.window(SYSMENU_WINDOW).unwrap().visible, "o menu de sistema não se esconde ao usar um botão");
        keys.macros.open(&mut desktop);
        assert!(keys.mouse_down(WindowId::MACRO_LIST, 11, 372, &mut desktop));
        keys.mouse_up(Some((WindowId::MACRO_LIST, 11, 372)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.take_commands(), vec![UiCommand::ToggleHelp { macro_help: true }]);
    }

    #[test]
    fn system_menu_buttons_restart_or_log_out_and_skills_fire_on_release() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        keys.sync(&world, None, &mut bar, Instant::now());
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x58);
        assert!(keys.mouse_down(SYSMENU_WINDOW, 20, 220, &mut desktop));
        assert_eq!(keys.sysmenu.pressed, Some(SysMenuButton::Restart));
        keys.mouse_up(Some((SYSMENU_WINDOW, 20, 222)), &mut desktop, &mut bar, &world);
        assert!(silent(keys.take_commands()).is_empty());
        assert_eq!(keys.dialogs, vec![Dialog::SystemMenu { restart: true }]);
        keys.dialogs.clear();
        assert!(desktop.window(SYSMENU_WINDOW).unwrap().visible);
        assert!(keys.mouse_down(SYSMENU_WINDOW, 20, 260, &mut desktop));
        keys.mouse_up(Some((SYSMENU_WINDOW, 100, 100)), &mut desktop, &mut bar, &world);
        assert!(silent(keys.take_commands()).is_empty());
        assert!(keys.mouse_down(SYSMENU_WINDOW, 20, 260, &mut desktop));
        keys.mouse_up(Some((SYSMENU_WINDOW, 20, 260)), &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().is_empty());
        assert_eq!(keys.dialogs, vec![Dialog::SystemMenu { restart: false }]);
        keys.system_menu_restart(&mut desktop);
        assert!(!desktop.window(SYSMENU_WINDOW).unwrap().visible);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x4b);
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::Net(NetCommand::RequestSkillList)]);
        let slot = (10 + 10, 66 + 41 + 10);
        assert!(keys.mouse_down(WindowId::MAIN, slot.0, slot.1, &mut desktop));
        assert_eq!(keys.skills.selected, Some(3));
        keys.mouse_up(Some((WindowId::MAIN, slot.0 + 2, slot.1 + 2)), &mut desktop, &mut bar, &world);
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::Net(NetCommand::UseSkill { skill_id: 3, ctrl: false, shift: false })]);
        assert!(keys.mouse_down(WindowId::MAIN, slot.0, slot.1, &mut desktop));
        keys.mouse_up(Some((WindowId::SHORTCUT, 7, 225)), &mut desktop, &mut bar, &world);
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::Net(NetCommand::ShortcutReg { kind: 2, slot: 5, id: 3, character_type: 1 })]);
        assert!(keys.mouse_down(WindowId::MAIN, 60, 30, &mut desktop));
        keys.mouse_up(Some((WindowId::MAIN, 60, 30)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.main.tab, MainTab::Skills);
        assert!(keys.mouse_down(WindowId::MAIN, 150, 30, &mut desktop));
        keys.mouse_up(Some((WindowId::MAIN, 150, 30)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.main.tab, MainTab::Clan);
    }

    #[test]
    fn option_tabs_1002e5a0_switch_on_mouse_down_without_saving() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        keys.option = killer_ui::option::OptionView::with_god(|id| Some(id.to_string()), false, killer_ui::option::GOD_FLAG);
        open_options(&mut keys, &mut desktop);
        assert_eq!(keys.option.active_page(), Some(killer_ui::option::TAB_VIDEO));
        let x = 12 + 75 * 2 + 1;
        assert!(keys.mouse_down(WindowId::OPTION, x, 26, &mut desktop));
        keys.mouse_up(Some((WindowId::OPTION, x, 26)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.option.active_page(), Some(killer_ui::option::TAB_VIDEO));
        assert!(!keys.option.tabs.control.entries[killer_ui::option::TAB_GAME].enabled);
        for tab in [1, 0, 1] {
            let x = 12 + 75 * tab + 1;
            assert!(keys.mouse_down(WindowId::OPTION, x, 26, &mut desktop));
            assert_eq!(keys.option.active_page(), Some(tab as usize));
            assert_eq!(keys.option.focused_page, Some(tab as usize));
            keys.mouse_up(Some((WindowId::OPTION, x, 26)), &mut desktop, &mut bar, &world);
            assert!(keys.take_commands().is_empty());
        }
        set_visible(&mut desktop, WindowId::OPTION, false);
        open_options(&mut keys, &mut desktop);
        assert_eq!(keys.option.active_page(), Some(killer_ui::option::TAB_AUDIO));
    }

    #[test]
    fn ncoptionwnd_100c49d0_cancel_does_not_save_and_apply_does_not_close() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        open_options(&mut keys, &mut desktop);
        keys.option.set_checked(0xf0f01475, true);
        let apply = killer_ui::option::APPLY_RECT;
        keys.mouse_down(WindowId::OPTION, apply.x + 1, apply.y + 1, &mut desktop);
        keys.mouse_up(Some((WindowId::OPTION, apply.x + 1, apply.y + 1)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.take_commands(), [UiCommand::ApplyOptions { close: false }]);
        assert!(keys.option.checked(0xf0f01475));
        assert!(desktop.window(WindowId::OPTION).unwrap().visible);
        keys.option.set_checked(0xf0f01475, false);
        let cancel = killer_ui::option::CANCEL_RECT;
        keys.mouse_down(WindowId::OPTION, cancel.x + 1, cancel.y + 1, &mut desktop);
        keys.mouse_up(Some((WindowId::OPTION, cancel.x + 1, cancel.y + 1)), &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().is_empty());
        assert!(!desktop.window(WindowId::OPTION).unwrap().visible);
    }

    #[test]
    fn option_game_100c7e90_and_100c8460_map_group_parent_and_general_child() {
        for group in [false, true] {
            for general in [false, true] {
                let (mut keys, mut desktop, mut bar, world) = setup();
                open_options(&mut keys, &mut desktop);
                let options = killer_ui::ini::Options { show_group_name: group, show_other_pc_name: general, ..Default::default() };
                keys.sync_options(&options);
                assert_eq!(keys.option.checked(0xf0f01461), group);
                assert_eq!(keys.option.checked(0xf0f01464), general);
                for id in [0xf0f01462, 0xf0f01463, 0xf0f01464] {
                    assert_eq!(keys.option.game_page.checks.iter().find(|c| c.id == id).unwrap().check.enabled, group);
                }
                let apply = killer_ui::option::APPLY_RECT;
                keys.mouse_down(WindowId::OPTION, apply.x + 1, apply.y + 1, &mut desktop);
                keys.mouse_up(Some((WindowId::OPTION, apply.x + 1, apply.y + 1)), &mut desktop, &mut bar, &world);
                let commands = keys.take_commands();
                assert_eq!(commands, [UiCommand::ApplyOptions { close: false }]);
                assert_eq!((keys.option.checked(0xf0f01461), keys.option.checked(0xf0f01464)), (group, general));
            }
        }
    }

    #[test]
    fn items_move_between_the_inventory_and_the_pet_window_like_100975e0_and_100de2d0() {
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        set_visible(&mut desktop, WindowId::INVENTORY, true);
        keys.sync(&world, None, &mut bar, Instant::now());
        let item = keys.inventory.grid_item(0).cloned().expect("demo tem item na grade");
        let (sx, sy) = killer_ui::inventory::grid_slot_pos(0);
        let (x, y) = (killer_ui::inventory::GRID_RECT.x + sx + 4, killer_ui::inventory::GRID_RECT.y + sy + 4);
        assert!(keys.mouse_down(WindowId::INVENTORY, x, y, &mut desktop));
        keys.mouse_up(Some((WindowId::PET_WINDOW, 50, 50)), &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().iter().any(|c| matches!(c, UiCommand::ItemDrop { object_id, target: ItemDropTarget::Pet, .. } if *object_id == item.object_id)));
        world.pet_inventory.items = vec![l2_net::game::InventoryItem { object_id: 9001, item_id: 2375, count: 3, ..Default::default() }];
        keys.sync(&world, None, &mut bar, Instant::now());
        keys.pet.view.press_tab(killer_ui::pet_wnd::PAGE_ITEMS);
        let (px, py) = (killer_ui::pet_wnd::PAGE_RECT.x + killer_ui::pet_wnd::ITEMS_RECT.x + 3, killer_ui::pet_wnd::PAGE_RECT.y + killer_ui::pet_wnd::ITEMS_RECT.y + 3);
        assert!(keys.mouse_down(WindowId::PET_WINDOW, px, py, &mut desktop));
        keys.hover(None, &bar);
        keys.mouse_up(Some((WindowId::INVENTORY, x, y)), &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().iter().any(|c| matches!(c, UiCommand::ItemDrop { object_id: 9001, count: 3, target: ItemDropTarget::FromPet, .. })));
    }

    #[test]
    fn inventory_item_100975e0_dropped_on_the_world_or_trash_asks_before_sending() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        set_visible(&mut desktop, WindowId::INVENTORY, true);
        keys.sync(&world, None, &mut bar, Instant::now());
        let item = keys.inventory.grid_item(0).cloned().expect("demo tem item na grade");
        let window = desktop.window(WindowId::INVENTORY).unwrap().rect;
        let (sx, sy) = killer_ui::inventory::grid_slot_pos(0);
        let (x, y) = (killer_ui::inventory::GRID_RECT.x + sx + 4, killer_ui::inventory::GRID_RECT.y + sy + 4);
        let _ = window;
        assert!(keys.mouse_down(WindowId::INVENTORY, x, y, &mut desktop));
        assert_eq!(keys.drag_icon(&bar), None);
        keys.hover(None, &bar);
        assert_eq!(keys.drag_icon(&bar), item.icons.first().cloned().filter(|icon| !icon.is_empty()));
        keys.mouse_up(None, &mut desktop, &mut bar, &world);
        assert_eq!(keys.drag_icon(&bar), None);
        assert!(keys.take_commands().iter().any(|c| matches!(c, UiCommand::ItemDrop { object_id, target: ItemDropTarget::World, .. } if *object_id == item.object_id)));
        let recipe = killer_ui::inventory::RECIPE_RECT;
        assert!(keys.mouse_down(WindowId::INVENTORY, x, y, &mut desktop));
        keys.mouse_up(Some((WindowId::INVENTORY, recipe.x + 2, recipe.y + 2)), &mut desktop, &mut bar, &world);
        assert!(silent(keys.take_commands()).is_empty(), "o botão de cristalizar só existe com o requisito do jogador");
        keys.inventory.dwarven_craft = true;
        assert!(keys.mouse_down(WindowId::INVENTORY, x, y, &mut desktop));
        keys.mouse_up(Some((WindowId::INVENTORY, recipe.x + 2, recipe.y + 2)), &mut desktop, &mut bar, &world);
        assert!(silent(keys.take_commands()).is_empty(), "e o item precisa ser cristalizável (item+0x74)");
        let index = keys.inventory.items.iter().position(|i| i.object_id == item.object_id).unwrap();
        keys.inventory.items[index].crystallizable = true;
        assert!(keys.mouse_down(WindowId::INVENTORY, x, y, &mut desktop));
        keys.mouse_up(Some((WindowId::INVENTORY, recipe.x + 2, recipe.y + 2)), &mut desktop, &mut bar, &world);
        let commands = keys.take_commands();
        assert!(commands.iter().any(|c| matches!(c, UiCommand::ItemDrop { target: ItemDropTarget::Crystallize, .. })), "{commands:?}");
        assert!(!commands.iter().any(|c| matches!(c, UiCommand::Net(NetCommand::CrystallizeItem { .. }))));
    }

    #[test]
    fn a_worn_item_dropped_on_the_trash_or_the_pet_asks_like_the_paperdoll_100964a0() {
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        set_visible(&mut desktop, WindowId::INVENTORY, true);
        keys.sync(&world, None, &mut bar, Instant::now());
        keys.inventory.dwarven_craft = true;
        let worn = keys.inventory.items.iter().position(|i| i.equipped).expect("o demo tem item vestido");
        let slot = (0..killer_ui::inventory::PAPERDOLL_SLOTS)
            .find(|slot| keys.inventory.paperdoll_item(*slot).is_some_and(|i| i.object_id == keys.inventory.items[worn].object_id))
            .expect("o item vestido ocupa uma casa");
        let (sx, sy) = killer_ui::inventory::paperdoll_slot_pos(slot);
        let (px, py) = (killer_ui::inventory::PAPERDOLL_RECT.x + sx + 4, killer_ui::inventory::PAPERDOLL_RECT.y + sy + 4);
        let trash = killer_ui::inventory::TRASH_RECT;
        assert!(keys.mouse_down(WindowId::INVENTORY, px, py, &mut desktop));
        keys.mouse_up(Some((WindowId::INVENTORY, trash.x + 2, trash.y + 2)), &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().iter().any(|c| matches!(c, UiCommand::ItemDrop { target: ItemDropTarget::Trash, .. })));
        world.pet_inventory.items.clear();
        keys.sync(&world, None, &mut bar, Instant::now());
        keys.pet.view.press_tab(killer_ui::pet_wnd::PAGE_ITEMS);
        assert!(keys.mouse_down(WindowId::INVENTORY, px, py, &mut desktop));
        let page = killer_ui::pet_wnd::PAGE_RECT;
        keys.mouse_up(Some((WindowId::PET_WINDOW, page.x + 5, page.y + 5)), &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().iter().any(|c| matches!(c, UiCommand::ItemDrop { target: ItemDropTarget::PetEquipped, .. })));
    }

    #[test]
    fn the_pet_items_reorder_in_their_grid_and_register_on_the_bar_with_the_pet_type() {
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        world.pet_inventory.items = (1..=3)
            .map(|id| l2_net::game::InventoryItem { object_id: 9000 + id, item_id: 2375, count: 1, ..Default::default() })
            .collect();
        keys.sync(&world, None, &mut bar, Instant::now());
        keys.pet.view.press_tab(killer_ui::pet_wnd::PAGE_ITEMS);
        let page = killer_ui::pet_wnd::PAGE_RECT;
        let items = killer_ui::pet_wnd::ITEMS_RECT;
        let cell = |index: i32| (page.x + items.x + 1 + 37 * index + 3, page.y + items.y + 1 + 3);
        let ids = |keys: &KeyWindows| keys.pet.view.items.iter().map(|i| i.grid.object_id).collect::<Vec<_>>();
        assert_eq!(ids(&keys), [9001, 9002, 9003]);
        let (x, y) = cell(0);
        assert!(keys.mouse_down(WindowId::PET_WINDOW, x, y, &mut desktop));
        let (tx, ty) = cell(2);
        keys.mouse_up(Some((WindowId::PET_WINDOW, tx, ty)), &mut desktop, &mut bar, &world);
        assert_eq!(ids(&keys), [9002, 9003, 9001]);
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!(ids(&keys), [9002, 9003, 9001], "a ordem do jogador sobrevive à sincronização");
        let (x, y) = cell(1);
        assert!(keys.mouse_down(WindowId::PET_WINDOW, x, y, &mut desktop));
        keys.mouse_up(Some((WindowId::SHORTCUT, 7, 225)), &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().iter().any(|c| matches!(
            c,
            UiCommand::Net(NetCommand::ShortcutReg { kind: 1, id: 9003, character_type: 2, .. })
        )));
    }

    #[test]
    fn the_settings_frame_close_button_hides_without_applying() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        open_options(&mut keys, &mut desktop);
        let close = killer_ui::option::close_rect();
        let (x, y) = (close.x + close.w / 2, close.y + close.h / 2);
        assert_eq!(keys.option.hit(x, y), Some(killer_ui::option::OptionHit::Close));
        keys.mouse_down(WindowId::OPTION, x, y, &mut desktop);
        keys.mouse_up(Some((WindowId::OPTION, x, y)), &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().is_empty());
        assert!(!desktop.window(WindowId::OPTION).unwrap().visible);
    }

    #[test]
    fn option_buttons_do_not_activate_when_released_elsewhere() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        open_options(&mut keys, &mut desktop);
        for rect in [killer_ui::option::OK_RECT, killer_ui::option::CANCEL_RECT, killer_ui::option::APPLY_RECT] {
            keys.mouse_down(WindowId::OPTION, rect.x + 1, rect.y + 1, &mut desktop);
            keys.mouse_up(None, &mut desktop, &mut bar, &world);
            assert!(keys.take_commands().is_empty());
            assert!(desktop.window(WindowId::OPTION).unwrap().visible);
        }
    }

    #[test]
    fn option_video_keep_min_frame_rate_dispatches_only_for_enabled_checkbox() {
        let (mut keys, mut desktop, _, _) = setup();
        open_options(&mut keys, &mut desktop);
        keys.option.tabs.control.current = Some(killer_ui::option::TAB_VIDEO);
        let check = killer_ui::option_video::CHECK_RECTS[0];
        let (x, y) = (killer_ui::option::PAGE_RECT.x + check.x + 1, killer_ui::option::PAGE_RECT.y + check.y + 1);
        assert!(keys.mouse_down(WindowId::OPTION, x, y, &mut desktop));
        assert!(matches!(keys.take_commands().as_slice(), [UiCommand::OptionKeepMinFrameRateChanged]));
        keys.option.video_page.checks[0].enabled = false;
        assert!(keys.mouse_down(WindowId::OPTION, x, y, &mut desktop));
        assert!(keys.take_commands().is_empty());
    }

    #[test]
    fn option_apply_wait_serializes_settings_input_until_resolution_confirmation() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        open_options(&mut keys, &mut desktop);
        let tab = keys.option.active_page();
        keys.set_option_applying(true);
        for (x, y) in [(163, 26), (12, 451), (92, 451), (172, 451)] {
            assert!(keys.mouse_down(WindowId::OPTION, x, y, &mut desktop));
            keys.mouse_up(Some((WindowId::OPTION, x, y)), &mut desktop, &mut bar, &world);
            assert!(keys.take_commands().is_empty());
            assert_eq!(keys.option.active_page(), tab);
            assert!(desktop.window(WindowId::OPTION).unwrap().visible);
        }
        keys.set_option_applying(false);
        keys.mouse_down(WindowId::OPTION, 172, 451, &mut desktop);
        keys.mouse_up(Some((WindowId::OPTION, 172, 451)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.take_commands(), [UiCommand::ApplyOptions { close: false }]);
    }

    #[test]
    fn the_option_button_opens_the_window_and_ok_saves() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        keys.sync(&world, None, &mut bar, Instant::now());
        open_options(&mut keys, &mut desktop);
        assert!(desktop.window(WindowId::OPTION).unwrap().visible, "Options abre a janela");
        keys.option.set_checked(0xf0f01475, true);
        let ok = killer_ui::option::OK_RECT;
        assert!(keys.mouse_down(WindowId::OPTION, ok.x + 4, ok.y + 4, &mut desktop));
        keys.mouse_up(Some((WindowId::OPTION, ok.x + 4, ok.y + 4)), &mut desktop, &mut bar, &world);
        let cmds = keys.take_commands();
        assert!(keys.option.checked(0xf0f01475));
        assert!(desktop.window(WindowId::OPTION).unwrap().visible, "OK aguarda a aplicação no App antes de fechar");
        assert_eq!(cmds, [UiCommand::ApplyOptions { close: true }]);
    }

    #[test]
    fn auto_potions_watch_hp_cp_and_mp_separately_throttled_and_stop_when_the_item_runs_out() {
        let (mut keys, _, mut bar, mut world) = setup();
        bar.modern = Some(Default::default());
        let item = |object_id, item_id, count| l2_net::game::InventoryItem { object_id, item_id, count, ..Default::default() };
        world.inventory.items = vec![item(777, 1060, 2), item(778, 5592, 1), item(779, 728, 1)];
        let me = world.me.as_mut().unwrap();
        me.hp = Some((900, 1000));
        me.cp = Some((100, 500));
        me.mp = Some((500, 500));
        for (potion, id) in bar.auto.potions.iter_mut().zip([1060, 5592, 728]) {
            potion.percent = 50;
            potion.item_id = id;
        }
        let used = |keys: &KeyWindows, object_id: i32| {
            keys.pending.iter().filter(|c| matches!(c, UiCommand::Net(NetCommand::UseItem { object_id: o, .. }) if *o == object_id)).count()
        };
        let start = Instant::now();
        keys.sync(&world, None, &mut bar, start);
        assert_eq!((used(&keys, 777), used(&keys, 778), used(&keys, 779)), (0, 1, 0));
        world.me.as_mut().unwrap().hp = Some((400, 1000));
        world.me.as_mut().unwrap().mp = Some((100, 500));
        keys.sync(&world, None, &mut bar, start + std::time::Duration::from_millis(500));
        assert_eq!((used(&keys, 777), used(&keys, 778), used(&keys, 779)), (1, 1, 1));
        keys.sync(&world, None, &mut bar, start + AUTO_POTION_INTERVAL);
        assert_eq!((used(&keys, 777), used(&keys, 778)), (1, 2));
        world.inventory.items.retain(|i| i.object_id != 778);
        keys.sync(&world, None, &mut bar, start + AUTO_POTION_INTERVAL * 4);
        assert_eq!(bar.auto.potions[1].item_id, 5592);
        assert_eq!(bar.auto.potions[1].count, 0);
        assert_eq!(used(&keys, 778), 2);
        world.me.as_mut().unwrap().hp = Some((0, 1000));
        keys.sync(&world, None, &mut bar, start + AUTO_POTION_INTERVAL * 8);
        assert_eq!(used(&keys, 777), 2);
    }

    #[test]
    fn the_shot_slots_fill_themselves_with_the_best_shot_of_their_kind() {
        let (mut keys, _, mut bar, mut world) = setup();
        bar.modern = Some(Default::default());
        let item = |object_id, item_id| l2_net::game::InventoryItem { object_id, item_id, count: 100, ..Default::default() };
        world.inventory.items = vec![item(1, 1463), item(2, 1465), item(3, 2511), item(4, 3949)];
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!((bar.auto.shots[0].item_id, bar.auto.shots[1].item_id), (1465, 3949));
        assert_eq!(bar.auto.shots[0].count, 100);
        world.inventory.items.retain(|i| i.item_id != 1465 && i.item_id != 3949);
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!((bar.auto.shots[0].item_id, bar.auto.shots[1].item_id), (1463, 2511));
        world.inventory.items.clear();
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!((bar.auto.shots[0].item_id, bar.auto.shots[1].item_id), (0, 0));
    }

    #[test]
    fn a_skill_list_decoded_in_entering_before_the_user_info_fills_the_skills_tab() {
        let mut body = vec![0x58u8];
        body.extend(3i32.to_le_bytes());
        for (passive, level, id) in [(0i32, 3i32, 9i32), (1, 1, 194), (0, 1, 1177)] {
            body.extend(passive.to_le_bytes());
            body.extend(level.to_le_bytes());
            body.extend(id.to_le_bytes());
        }
        let event = killer_world::session::decode_world(&body).unwrap().unwrap();
        let mut world = World::default();
        world.apply_world(&event);
        assert!(world.me.is_none());
        assert_eq!(world.skills.len(), 3);
        world.me = Some(crate::ui2d::tests::me());
        let (mut keys, mut desktop, mut bar, _) = setup();
        keys.sync(&world, None, &mut bar, Instant::now());
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x4b);
        assert_eq!(keys.main.tab, MainTab::Skills);
        assert_eq!(keys.skills.listed().iter().map(|s| (s.id, s.level)).collect::<Vec<_>>(), vec![(9, 3), (1177, 1)]);
        keys.skills.tab = killer_ui::skills::SkillTab::Passive;
        assert_eq!(keys.skills.listed().iter().map(|s| s.id).collect::<Vec<_>>(), vec![194]);
        let mut list = DrawList::default();
        keys.paint(WindowId::MAIN, &mut list, 0, 0, &world, &crate::ui2d::tests::small_font(), 1.0);
        assert!(!list.quads.iter().any(|q| q.texture == killer_ui::skills::ICON_BACK), "habilidade passiva não tem moldura");
        keys.skills.tab = killer_ui::skills::SkillTab::Active;
        let mut list = DrawList::default();
        keys.paint(WindowId::MAIN, &mut list, 0, 0, &world, &crate::ui2d::tests::small_font(), 1.0);
        assert!(list.quads.iter().any(|q| q.texture == killer_ui::skills::ICON_BACK));
    }

    #[test]
    fn new_items_show_first_on_the_item_page_and_known_ones_keep_the_order_the_player_gave() {
        let (mut keys, _, _, mut world) = setup();
        let item = |object_id: i32| l2_net::game::InventoryItem { object_id, item_id: 1, count: 1, ..Default::default() };
        world.inventory.items = vec![item(1), item(2), item(3)];
        keys.rebuild_inventory(&world, None);
        let ids = |keys: &KeyWindows| keys.inventory.listed().iter().map(|i| i.object_id).collect::<Vec<_>>();
        assert_eq!(ids(&keys), [1, 2, 3], "a lista inicial entra na ordem do servidor");
        assert!(keys.inventory.move_listed(1, 2));
        assert_eq!(ids(&keys), [2, 3, 1]);
        world.inventory.items.push(item(4));
        world.inventory.items[0].count = 9;
        keys.rebuild_inventory(&world, None);
        assert_eq!(ids(&keys), [4, 2, 3, 1]);
        world.inventory.items.remove(1);
        keys.rebuild_inventory(&world, None);
        assert_eq!(ids(&keys), [4, 3, 1]);
    }

    #[test]
    fn classic_shortcut_slots_show_the_one_line_tip_of_their_kind() {
        let (mut keys, _, _, _) = setup();
        let mut bar = ShortcutView::new(false);
        keys.inventory.items.push(InventoryItemView {
            object_id: 5,
            item_id: 1060,
            count: 12,
            enchant: 2,
            name: "Healing Potion".into(),
            description: "texto longo".into(),
            weight: 3,
            ..Default::default()
        });
        keys.skills.skills.push(killer_ui::skills::SkillView {
            id: 3,
            level: 4,
            name: "Power Strike".into(),
            description: "texto longo".into(),
            ..Default::default()
        });
        keys.actions.actions.push(ActionEntry { id: 2, category: 1, name: "Attack".into(), desc: "descricao".into(), ..Default::default() });
        bar.set(0, ShortcutSlot::new(SlotKind::Item { object_id: 5 }, "icon.potion"));
        bar.set(1, ShortcutSlot::new(SlotKind::Skill { id: 3, level: 4 }, "icon.skill0003"));
        bar.set(2, ShortcutSlot::new(SlotKind::Action { id: 2 }, "icon.action"));
        let tip = |keys: &mut KeyWindows, index: usize| {
            let rect = bar.slot_rect(index);
            keys.hover(Some((WindowId::SHORTCUT, rect.x + 2, rect.y + 2)), &bar);
            keys.tooltip.clone()
        };
        let item = tip(&mut keys, 0).unwrap();
        assert_eq!((item.name.as_str(), item.description.as_str(), item.lines.len()), ("Healing Potion", "", 0));
        let skill = tip(&mut keys, 1).unwrap();
        assert_eq!((skill.name.as_str(), skill.level.as_ref().map(|l| l.1), skill.description.as_str()), ("Power Strike", Some(4), ""));
        let action = tip(&mut keys, 2).unwrap();
        assert_eq!((action.name.as_str(), action.description.as_str()), ("Attack", ""));
    }

    #[test]
    fn carrying_an_inventory_item_over_the_trash_or_recipe_icon_shows_its_drag_picture() {
        let (mut keys, mut desktop, _, _) = setup();
        let bar = ShortcutView::new(false);
        keys.inventory.dwarven_craft = true;
        keys.inventory.items.push(InventoryItemView { object_id: 5, item_id: 1060, count: 1, name: "Potion".into(), ..Default::default() });
        set_visible(&mut desktop, WindowId::INVENTORY, true);
        let cell = killer_ui::inventory::GRID_RECT;
        assert!(keys.mouse_down(WindowId::INVENTORY, cell.x + 4, cell.y + 4, &mut desktop));
        let trash = killer_ui::inventory::TRASH_RECT;
        keys.hover(Some((WindowId::INVENTORY, trash.x + 3, trash.y + 3)), &bar);
        assert!(keys.inventory.over_trash && !keys.inventory.over_recipe);
        let recipe = killer_ui::inventory::RECIPE_RECT;
        keys.hover(Some((WindowId::INVENTORY, recipe.x + 3, recipe.y + 3)), &bar);
        assert!(!keys.inventory.over_trash && keys.inventory.over_recipe);
        keys.hover(Some((WindowId::INVENTORY, cell.x + 4, cell.y + 4)), &bar);
        assert!(!keys.inventory.over_trash && !keys.inventory.over_recipe);
    }

    #[test]
    fn closing_the_inventory_saves_the_item_order_and_opening_it_reads_and_applies_it() {
        let root = std::env::temp_dir().join(format!("killer-iwo-keys-{}", std::process::id()));
        killer_platform::set_item_order_root(root.clone());
        let (mut keys, _, _, mut world) = setup();
        let item = |object_id: i32, item_id: i32| l2_net::game::InventoryItem { object_id, item_id, count: 1, ..Default::default() };
        world.inventory.items = vec![item(1, 57), item(2, 1060), item(3, 1061)];
        keys.rebuild_inventory(&world, None);
        keys.sync_inventory_state(&world, None);
        let name = world.me.as_ref().unwrap().name.clone();
        keys.track_inventory(true);
        assert!(keys.inventory.move_listed(3, 0));
        keys.track_inventory(false);
        let file = killer_ui::item_order::file_name(&name);
        let bytes = killer_platform::read_item_order(&file).unwrap();
        assert_eq!(killer_ui::item_order::decode(&bytes), [1061, 57, 1060]);
        let (mut other, _, _, _) = setup();
        other.character_name = name;
        other.inventory.items = keys.inventory.items.clone();
        other.inventory.items.sort_by_key(|i| i.item_id);
        other.track_inventory(true);
        let ids: Vec<i32> = other.inventory.listed().iter().map(|i| i.item_id).collect();
        assert_eq!(ids, [1061, 57, 1060]);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn the_end_flash_is_born_only_when_the_icon_is_painted_within_half_a_second_after_the_end() {
        use std::time::Duration;
        let (mut keys, _, _, world) = setup();
        keys.skills.skills.push(killer_ui::skills::SkillView { id: 3, level: 1, ..Default::default() });
        let start = Instant::now();
        let cooldown = || Cooldown { level: 1, started: start, reuse: 2.0, remaining: 2.0, flash: None, stopped: false };
        keys.cooldowns.insert(3, cooldown());
        keys.set_cool_time_surfaces(false, false);
        keys.sync_cooldowns(&world, start + Duration::from_millis(2100));
        assert!(keys.cooldowns[&3].flash.is_none() && !keys.cooldowns[&3].stopped);
        assert_eq!(keys.cool_frame(3, 1, start + Duration::from_millis(2100)), None);
        keys.set_cool_time_surfaces(true, false);
        keys.take_commands();
        keys.sync_cooldowns(&world, start + Duration::from_millis(2300));
        assert!(keys.cooldowns[&3].flash.is_some());
        assert_eq!(keys.cool_frame(3, 1, start + Duration::from_millis(2300)), Some(killer_ui::cooltime::CoolFrame::Flash(0)));
        assert_eq!(keys.take_commands().len(), 1, "o som do fim toca uma vez");
        keys.cooldowns.insert(3, cooldown());
        keys.sync_cooldowns(&world, start + Duration::from_millis(2700));
        assert!(!keys.cooldowns.contains_key(&3), "sem pintura em meio segundo o ícone só para");
        keys.cooldowns.insert(3, cooldown());
        keys.sync_cooldowns(&world, start + Duration::from_millis(2300));
        keys.sync_cooldowns(&world, start + Duration::from_millis(2900));
        assert!(!keys.cooldowns.contains_key(&3), "o flash dura meio segundo");
    }

    #[test]
    fn a_pet_shortcut_item_goes_through_request_pet_use_item_100803a0() {
        let (mut keys, _, _, mut world) = setup();
        world.shortcuts = vec![Shortcut { slot: 0, kind: ShortcutKind::Item { object_id: 77 }, character_type: 2 }];
        keys.use_shortcut(&world, 0);
        assert_eq!(keys.take_commands(), vec![UiCommand::Net(NetCommand::PetUseItem { object_id: 77 })]);
        world.shortcuts = vec![Shortcut { slot: 0, kind: ShortcutKind::Item { object_id: 77 }, character_type: 1 }];
        keys.use_shortcut(&world, 0);
        assert_eq!(keys.take_commands(), vec![UiCommand::Net(NetCommand::UseItem { object_id: 77, ctrl: false })]);
    }

    #[test]
    fn a_shortcut_asks_dialog_21_for_a_recipe_item_only_for_the_dwarven_classes_100803a0() {
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        keys.sync(&world, None, &mut bar, Instant::now());
        keys.world_event(&WorldEvent::ItemList(ItemList { show_window: true, items: vec![] }), &mut desktop, &world);
        let item = keys.inventory.items.iter_mut().find(|i| i.object_id == 108).unwrap();
        item.recipe = true;
        item.name = "Recipe".into();
        world.shortcuts = vec![Shortcut { slot: 0, kind: ShortcutKind::Item { object_id: 108 }, character_type: 1 }];
        let mut me = crate::ui2d::tests::me();
        for (class_id, asks) in [(0x34, false), (0x35, true), (0x39, true), (0x3a, false), (0x75, true), (0x76, true), (0x77, false), (0, false)] {
            if let EntityKind::Player { class_id: current, .. } = &mut me.kind {
                *current = class_id;
            }
            world.me = Some(me.clone());
            keys.use_shortcut(&world, 0);
            let commands = keys.take_commands();
            if asks {
                assert!(commands.is_empty() && keys.dialogs == vec![Dialog::RecipeRegister { object_id: 108, item_name: "Recipe".into() }], "{class_id}");
            } else {
                assert_eq!(commands, vec![UiCommand::Net(NetCommand::UseItem { object_id: 108, ctrl: false })], "{class_id}");
                assert!(keys.dialogs.is_empty(), "{class_id}");
            }
            keys.dialogs.clear();
        }
    }

    #[test]
    fn items_with_a_popup_or_recipes_ask_before_being_used() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        keys.sync(&world, None, &mut bar, Instant::now());
        keys.world_event(&WorldEvent::ItemList(ItemList { show_window: true, items: vec![] }), &mut desktop, &world);
        let (x, y) = (10 + 37 * 2, 190);
        let index = 2;
        assert_eq!(keys.inventory.grid_item(index).map(|i| i.object_id), Some(108));
        let item = keys.inventory.items.iter_mut().find(|i| i.object_id == 108).unwrap();
        item.popup = 1080;
        item.name = "Facelifting Potion - A".into();
        assert!(keys.double_click(WindowId::INVENTORY, x, y, &world));
        assert!(keys.take_commands().is_empty());
        assert_eq!(
            keys.dialogs,
            vec![Dialog::UseItemPopup { object_id: 108, message_id: 1080, item_name: "Facelifting Potion - A".into() }]
        );
        keys.dialogs.clear();
        let item = keys.inventory.items.iter_mut().find(|i| i.object_id == 108).unwrap();
        item.popup = 0;
        item.recipe = true;
        assert!(keys.right_click(WindowId::INVENTORY, x, y));
        assert!(keys.take_commands().is_empty());
        assert_eq!(keys.dialogs, vec![Dialog::RecipeRegister { object_id: 108, item_name: "Facelifting Potion - A".into() }]);
        keys.dialogs.clear();
        let item = keys.inventory.items.iter_mut().find(|i| i.object_id == 108).unwrap();
        item.recipe = false;
        assert!(keys.right_click(WindowId::INVENTORY, x, y));
        assert_eq!(keys.take_commands(), vec![UiCommand::Net(NetCommand::UseItem { object_id: 108, ctrl: false })]);
        assert!(keys.dialogs.is_empty());
    }

    #[test]
    fn the_grid_scrolls_by_wheel_buttons_and_slider_like_ncscrollwnd() {
        use killer_ui::scroll::{BUTTON_SIZE, SCROLL_WIDTH};
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        world.inventory.items = (0..30)
            .map(|i| InventoryItem { type1: 0, object_id: 200 + i, item_id: 57, count: 1, type2: 0, custom_type1: 0, equipped: false, body_part: 0, enchant: 0, custom_type2: 0 })
            .collect();
        keys.sync(&world, None, &mut bar, Instant::now());
        keys.world_event(&WorldEvent::ItemList(ItemList { show_window: true, items: vec![] }), &mut desktop, &world);
        assert_eq!(keys.inventory.total_rows(), 5);
        assert!(keys.inventory.scroll.slider.shown);
        let grid = inventory::GRID_RECT;
        let scroll_x = grid.x + grid.w - SCROLL_WIDTH + 3;
        assert_eq!(keys.inventory.hit(scroll_x, grid.y + 3), Some(InventoryHit::Scroll(ScrollHit::Up)));
        assert_eq!(keys.inventory.hit(scroll_x, grid.y + grid.h - 3), Some(InventoryHit::Scroll(ScrollHit::Down)));
        assert_eq!(keys.inventory.hit(grid.x + 1 + 37 * 5 + 10, grid.y + 10), Some(InventoryHit::Grid(5)));
        assert!(keys.wheel(WindowId::INVENTORY, grid.x + 10, grid.y + 10, -120));
        assert_eq!(keys.inventory.scroll_row, 1);
        assert_eq!(keys.inventory.hit(grid.x + 10, grid.y + 10), Some(InventoryHit::Grid(6)));
        assert!(keys.wheel(WindowId::INVENTORY, grid.x + 10, grid.y + 10, -120 * 5));
        assert_eq!(keys.inventory.scroll_row, 1);
        assert!(keys.wheel(WindowId::INVENTORY, grid.x + 10, grid.y + 10, 120));
        assert_eq!(keys.inventory.scroll_row, 0);
        assert!(!keys.wheel(WindowId::INVENTORY, 10, 10, -120));
        assert!(keys.mouse_down(WindowId::INVENTORY, scroll_x, grid.y + grid.h - 3, &mut desktop));
        assert!(keys.inventory.scroll.down_pressed);
        keys.mouse_up(Some((WindowId::INVENTORY, scroll_x, grid.y + grid.h - 3)), &mut desktop, &mut bar, &world);
        assert!(!keys.inventory.scroll.down_pressed);
        assert_eq!(keys.inventory.scroll_row, 1);
        assert!(keys.mouse_down(WindowId::INVENTORY, scroll_x, grid.y + 3, &mut desktop));
        keys.mouse_up(Some((WindowId::INVENTORY, 100, 100)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.inventory.scroll_row, 1);
        assert!(keys.mouse_down(WindowId::INVENTORY, scroll_x, grid.y + 3, &mut desktop));
        keys.mouse_up(Some((WindowId::INVENTORY, scroll_x, grid.y + 3)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.inventory.scroll_row, 0);
        let slider_y = grid.y + BUTTON_SIZE;
        let thumb_y = keys.inventory.scroll.slider.thumb_y;
        assert!(keys.mouse_down(WindowId::INVENTORY, scroll_x, slider_y + thumb_y + 2, &mut desktop));
        assert!(keys.inventory.scroll.slider.dragging);
        assert!(keys.mouse_move(WindowId::INVENTORY, slider_y + thumb_y + 2 + 60));
        assert!(keys.inventory.scroll_row > 0, "{}", keys.inventory.scroll_row);
        keys.mouse_up(Some((WindowId::INVENTORY, scroll_x, slider_y + 62)), &mut desktop, &mut bar, &world);
        assert!(!keys.inventory.scroll.slider.dragging);
        world.inventory.items.truncate(6);
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!(keys.inventory.scroll_row, 1);
        keys.world_event(&WorldEvent::ItemList(ItemList { show_window: false, items: vec![] }), &mut desktop, &world);
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!(keys.inventory.scroll_row, 0);
        assert!(!keys.inventory.scroll.slider.shown);
        assert_eq!(keys.inventory.hit(scroll_x, grid.y + 50), Some(InventoryHit::Body));
        let mut list = DrawList::default();
        keys.paint(WindowId::INVENTORY, &mut list, 0, 0, &world, &crate::ui2d::tests::small_font(), 1.0);
        let up = list.quads.iter().find(|q| q.texture == killer_ui::scroll::UP_BTN).unwrap();
        assert_eq!((up.x, up.y), ((grid.x + grid.w - SCROLL_WIDTH) as f32, grid.y as f32));
    }

    #[test]
    fn clicking_a_clan_list_header_sorts_the_members_and_plays_the_click_sound() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        keys.main.tab = MainTab::Clan;
        set_visible(&mut desktop, WindowId::MAIN, true);
        let member = |name: &str| MemberRow { name: name.into(), level: 1, class_id: 1, online: true, object_id: 0 };
        keys.pledge.members = vec![member("b"), member("c"), member("a")];
        let (hx, hy) = (PAGE_RECT.x + killer_ui::pledge::LIST_RECT.x + 10, PAGE_RECT.y + killer_ui::pledge::LIST_RECT.y + 5);
        assert!(keys.mouse_down(WindowId::MAIN, hx, hy, &mut desktop));
        assert!(keys.pledge.headers[0].state == 1);
        keys.mouse_up(Some((WindowId::MAIN, hx, hy)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.pledge.members.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(), ["a", "b", "c"]);
        assert!(keys.take_commands().iter().any(|c| matches!(c, UiCommand::InterfaceSound(_))));
    }

    #[test]
    fn the_member_sync_keeps_the_sorted_order_until_the_list_is_sent_again() {
        let (mut keys, _desktop, mut bar, mut world) = setup();
        let member = |name: &str| l2_net::game::PledgeMember { name: name.into(), level: 5, class_id: 1, unknown: 0, flag: 0, online_object_id: 0 };
        world.social.clan_id = 33;
        world.pledge_window.members = vec![member("b"), member("c"), member("a")];
        world.pledge_window.list_reset = 1;
        keys.sync(&world, None, &mut bar, Instant::now());
        keys.pledge.selected = Some(2);
        keys.pledge.sort(0);
        assert_eq!(keys.pledge.members.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(), ["a", "b", "c"]);
        assert_eq!(keys.pledge.selected, Some(0));
        world.pledge_window.members.push(member("0"));
        world.pledge_window.members[1].level = 77;
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!(keys.pledge.members.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(), ["a", "b", "c", "0"]);
        assert_eq!(keys.pledge.members[2].level, 77);
        assert_eq!(keys.pledge.selected, Some(0));
        world.pledge_window.list_reset = 2;
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!(keys.pledge.members.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(), ["b", "c", "a", "0"]);
        assert_eq!(keys.pledge.selected, None);
    }

    #[test]
    fn opening_the_clan_page_runs_show_window_1004c680_with_the_rights_the_user_info_carries() {
        let (mut keys, _desktop, mut bar, mut world) = setup();
        let mut me = crate::ui2d::tests::me();
        me.clan_id = 33;
        me.relation = 0;
        world.me = Some(me);
        world.social.pledge_rights = [false, true, false, false];
        world.social.noble = false;
        keys.pledge_show_pending = true;
        keys.sync(&world, None, &mut bar, Instant::now());
        let enabled: Vec<_> = (0..15).filter(|i| keys.pledge.button_enabled(*i)).collect();
        assert_eq!(enabled, vec![1, 2, 4, 8, 10, 11, 14]);
        world.me.as_mut().unwrap().relation = 0x40;
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!((0..15).filter(|i| keys.pledge.button_enabled(*i)).count(), 7);
        keys.pledge_show_pending = true;
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!((0..15).filter(|i| keys.pledge.button_enabled(*i)).count(), 14);
        world.me = None;
        keys.pledge_show_pending = true;
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!((0..15).filter(|i| keys.pledge.button_enabled(*i)).count(), 14);
    }

    #[test]
    fn the_clan_tab_lists_the_members_and_its_buttons_invite_dismiss_and_leave() {
        use killer_ui::pledge::{button_rect, LIST_RECT};
        use l2_net::game::packets::PledgeMember;
        use l2_net::game::PledgeShowMemberListAll;
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        world.social.clan_id = 33;
        let member = |name: &str, online: i32| PledgeMember { name: name.into(), level: 40, class_id: 1, unknown: 0, flag: 0, online_object_id: online };
        world.pledge_window.info = Some(PledgeShowMemberListAll {
            pledge_id: 33,
            name: "Canastra".into(),
            leader_name: "Chefe".into(),
            crest_id: 0,
            level: 5,
            castle_id: 0,
            hideout_id: 0,
            my_level: 0,
            dissolving: 0,
            ally_id: 0,
            ally_name: String::new(),
            ally_crest_id: 0,
            at_war: false,
            members: vec![],
        });
        world.pledge_window.members = vec![member("Chefe", 5), member("Amigo", 0)];
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!(keys.pledge.name.as_deref(), Some("Canastra"));
        assert_eq!(keys.pledge.members.len(), 2);
        assert!(!keys.pledge.members[1].online);
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x4e);
        assert_eq!(keys.main.tab, MainTab::Clan);
        let page = |r: killer_ui::window::Rect| (PAGE_RECT.x + r.x + 5, PAGE_RECT.y + r.y + 5);
        let (ix, iy) = page(button_rect(0));
        assert!(keys.mouse_down(WindowId::MAIN, ix, iy, &mut desktop));
        assert_eq!(keys.pledge.pressed, Some(PledgeButton::Invite));
        keys.mouse_up(Some((WindowId::MAIN, ix, iy)), &mut desktop, &mut bar, &world);
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::SystemMessage(INVITE_NEEDS_TARGET_SYSMSG)]);
        let mut other = world.me.clone().unwrap();
        other.object_id = 9;
        world.entities.insert(9, other);
        world.target = Some(9);
        assert!(keys.mouse_down(WindowId::MAIN, ix, iy, &mut desktop));
        keys.mouse_up(Some((WindowId::MAIN, ix, iy)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.take_commands(), vec![UiCommand::Net(NetCommand::JoinPledge { object_id: 9 })]);
        let (dx, dy) = page(button_rect(3));
        assert!(keys.mouse_down(WindowId::MAIN, dx, dy, &mut desktop));
        keys.mouse_up(Some((WindowId::MAIN, dx, dy)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.take_commands(), vec![UiCommand::SystemMessage(SELECT_MEMBER_SYSMSG)]);
        let (rx, ry) = (PAGE_RECT.x + LIST_RECT.x + 10, PAGE_RECT.y + LIST_RECT.y + 19 + 17 + 3);
        assert!(keys.mouse_down(WindowId::MAIN, rx, ry, &mut desktop));
        assert_eq!(keys.pledge.selected, Some(1));
        keys.mouse_up(Some((WindowId::MAIN, rx, ry)), &mut desktop, &mut bar, &world);
        assert!(keys.mouse_down(WindowId::MAIN, dx, dy, &mut desktop));
        keys.mouse_up(Some((WindowId::MAIN, dx, dy)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.dialogs, vec![Dialog::OustPledgeMember { name: "Amigo".into() }]);
        keys.dialogs.clear();
        let (lx, ly) = page(button_rect(8));
        assert!(keys.mouse_down(WindowId::MAIN, lx, ly, &mut desktop));
        keys.mouse_up(Some((WindowId::MAIN, lx, ly)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.dialogs, vec![Dialog::WithdrawPledge { pledge_name: "Canastra".into() }]);
        keys.dialogs.clear();
        world.me.as_mut().unwrap().relation = 0x40;
        assert!(keys.mouse_down(WindowId::MAIN, lx, ly, &mut desktop));
        keys.mouse_up(Some((WindowId::MAIN, lx, ly)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.take_commands(), vec![UiCommand::SystemMessage(LEADER_CANNOT_WITHDRAW_SYSMSG)]);
        assert!(keys.mouse_down(WindowId::MAIN, lx, ly, &mut desktop));
        keys.mouse_up(Some((WindowId::MAIN, 5, 5)), &mut desktop, &mut bar, &world);
        assert!(keys.take_commands().is_empty());
        let mut list = DrawList::default();
        keys.paint(WindowId::MAIN, &mut list, 0, 0, &world, &crate::ui2d::tests::small_font(), 1.0);
        assert!(list.quads.iter().any(|q| q.texture == killer_ui::pledge::PLEDGE_BACK));
    }

    #[test]
    fn nc_map_cover_wnd_100bcd50_party_button_steps_the_member_index_modulo_the_count_and_highlights_it() {
        let (mut keys, _desktop, _bar, world) = setup();
        let mark = |id: i32, x: i32| killer_ui::minimap::PartyMark { id, world: (x, 148000, -3400), in_window: true, name: String::new() };
        keys.minimap.party = vec![mark(1, 83000), mark(2, 83400), mark(3, 83800)];
        for expected in [1, 2, 0, 1] {
            keys.minimap_action(MinimapButton::PartyMember, &world);
            assert_eq!(keys.minimap.highlight, Some(expected));
            assert_eq!(keys.minimap.highlight_ms, killer_ui::minimap::PARTY_HIGHLIGHT_MS);
        }
        keys.minimap.party.clear();
        keys.minimap.highlight = None;
        let before = keys.minimap.scroll;
        keys.minimap_action(MinimapButton::PartyMember, &world);
        assert_eq!(keys.minimap.scroll, before);
        assert_eq!(keys.minimap.highlight, None);
    }

    #[test]
    fn show_minimap_toggles_the_map_centred_on_the_player_and_the_buttons_recentre() {
        use killer_ui::minimap::{BUTTON_XS, BUTTON_Y, MINIMAP_SIZE};
        use l2_net::game::ShowMinimap;
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        world.me.as_mut().unwrap().position = glam::Vec3::new(83400.0, 148000.0, -3400.0);
        world.clock = Some(killer_world::clock::GameClock::new(13 * 60 + 30, 0, Instant::now()));
        assert!(!desktop.window(WindowId::MAP_COVER).unwrap().visible);
        keys.world_event(&WorldEvent::ShowMinimap(ShowMinimap { map_id: 1665, flag: 2 }), &mut desktop, &world);
        assert!(desktop.window(WindowId::MAP_COVER).unwrap().visible);
        assert_eq!(keys.minimap.period, 2);
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!(keys.minimap.player, Some((1073, 2052)));
        assert_eq!(keys.minimap.scroll, (1073 - 164, 2052 - 164));
        assert_eq!(keys.minimap.hour.map(|h| h as i32), Some(13));
        keys.minimap.scroll = (0, 0);
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!(keys.minimap.scroll, (0, 0));
        let (bx, by) = (BUTTON_XS[2] + 5, BUTTON_Y + 5);
        assert!(keys.mouse_down(WindowId::MAP_COVER, bx, by, &mut desktop));
        assert_eq!(keys.minimap.pressed, Some(MinimapButton::CurrentLoc));
        keys.mouse_up(Some((WindowId::MAP_COVER, bx, by)), &mut desktop, &mut bar, &world);
        assert_eq!(keys.minimap.scroll, (1073 - 164, 2052 - 164));
        assert_eq!(keys.minimap.pressed, None);
        let mut list = DrawList::default();
        keys.paint(WindowId::MAP_COVER, &mut list, 0, 0, &world, &crate::ui2d::tests::small_font(), 1.0);
        assert!(list.quads.iter().any(|q| q.texture == killer_ui::minimap::MINIMAP_POS));
        assert!(list.quads.iter().any(|q| q.texture == killer_ui::minimap::MAP_SUN));
        keys.world_event(&WorldEvent::ShowMinimap(ShowMinimap { map_id: 1665, flag: 2 }), &mut desktop, &world);
        assert!(!desktop.window(WindowId::MAP_COVER).unwrap().visible);
        keys.world_event(&WorldEvent::ShowMinimap(ShowMinimap { map_id: 1665, flag: 0 }), &mut desktop, &world);
        let close = killer_ui::minimap::close_button();
        assert!(keys.mouse_down(WindowId::MAP_COVER, close.x + 2, close.y + 2, &mut desktop));
        assert!(!desktop.window(WindowId::MAP_COVER).unwrap().visible);
        assert_eq!(desktop.window(WindowId::MAP_COVER).unwrap().rect.w, MINIMAP_SIZE.0);
    }

    #[test]
    fn minimap_buttons_100bcd50_and_100bb980_cycle_the_party_map_highlight_for_2s_and_centre_on_the_quest_mark() {
        use killer_ui::minimap::{BUTTON_XS, BUTTON_Y, PARTY_HIGHLIGHT_MS};
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        world.party.positions.insert(40, [70_000, 140_000, 0]);
        world.party.positions.insert(41, [-90_000, 200_000, 0]);
        world.windows.radar.map_points.push([70_730, 139_465, 0]);
        keys.sync(&world, None, &mut bar, Instant::now());
        assert_eq!(keys.minimap.party.iter().map(|mark| mark.id).collect::<Vec<_>>(), vec![40, 41]);
        assert_eq!(keys.minimap.targets, vec![(70_730, 139_465)]);
        let click = |keys: &mut KeyWindows, desktop: &mut Desktop, bar: &mut ShortcutView, world: &World, button: usize| {
            let (x, y) = (BUTTON_XS[button] + 5, BUTTON_Y + 5);
            keys.mouse_down(WindowId::MAP_COVER, x, y, desktop);
            keys.mouse_up(Some((WindowId::MAP_COVER, x, y)), desktop, bar, world);
        };
        click(&mut keys, &mut desktop, &mut bar, &world, 0);
        assert_eq!((keys.minimap.highlight, keys.minimap.highlight_ms), (Some(1), PARTY_HIGHLIGHT_MS));
        assert_eq!(keys.minimap.scroll, killer_ui::minimap::scroll_centered_on(killer_ui::minimap::world_to_map(-90_000, 200_000)));
        click(&mut keys, &mut desktop, &mut bar, &world, 0);
        assert_eq!(keys.minimap.highlight, Some(0));
        keys.minimap.tick(1999);
        assert_eq!(keys.minimap.highlight, Some(0));
        keys.minimap.tick(1);
        assert_eq!(keys.minimap.highlight, None);
        let before = keys.minimap.scroll;
        click(&mut keys, &mut desktop, &mut bar, &world, 1);
        assert_eq!(keys.minimap.scroll, before);
    }

    #[test]
    fn the_quest_tab_builds_the_tree_from_the_quest_list_and_questname() {
        use killer_ui::quest::{MASK_FLAG, PLUS_BTN, TREE_RECT};
        use l2_dat::QuestName;
        use l2_net::game::QuestState;
        let (mut keys, mut desktop, mut bar, mut world) = setup();
        let mut tables = DatTables::default();
        let quest = |state: i32, sub: &str| QuestName {
            index: 0,
            valid: 1,
            id: 1,
            state,
            title: "Letters of Love".into(),
            sub_name: sub.into(),
            description: format!("step {state}"),
            item_ids: vec![],
            item_nums: vec![],
            target_loc: [0.0; 3],
            level: 2,
            quest_level: 5,
            quest_type: 3,
            target_name: String::new(),
            minimap_only: 0,
            journal_disp: 1,
            unknown: 1,
            start_npc_id: 7048,
            start_npc_loc: [0.0; 3],
            requirement: String::new(),
            intro: String::new(),
        };
        tables.questname.insert((1, 1), quest(1, "Delivery"));
        tables.questname.insert((1, 2), quest(2, "Kerchief"));
        let list = |quests: Vec<QuestState>| WorldEvent::QuestList(l2_net::game::QuestList { quests, extras: Vec::new() });
        world.windows.quests = vec![QuestState { quest_id: 1, cond: 2 }, QuestState { quest_id: 999, cond: 1 }];
        keys.world_event(&list(world.windows.quests.clone()), &mut desktop, &world);
        keys.sync(&world, Some(&tables), &mut bar, Instant::now());
        assert_eq!(keys.quest.quests.len(), 1);
        assert_eq!(keys.quest.quests[0].steps.iter().map(|s| s.sub_name.as_str()).collect::<Vec<_>>(), vec!["Delivery", "Kerchief"]);
        world.windows.quests = vec![QuestState { quest_id: 1, cond: MASK_FLAG | 2 }];
        keys.sync(&world, Some(&tables), &mut bar, Instant::now());
        assert_eq!(keys.quest.quests[0].steps.len(), 2);
        keys.world_event(&list(world.windows.quests.clone()), &mut desktop, &world);
        keys.sync(&world, Some(&tables), &mut bar, Instant::now());
        assert_eq!(keys.quest.quests[0].steps.len(), 1);
        keys.take_commands();
        alt(&mut keys, &mut desktop, &mut bar, &world, 0x55);
        assert_eq!(keys.main.tab, MainTab::Quests);
        assert_eq!(silent(keys.take_commands()), vec![UiCommand::Net(NetCommand::RequestQuestList)]);
        assert!(keys.quest.quests.is_empty());
        keys.world_event(&list(world.windows.quests.clone()), &mut desktop, &world);
        keys.sync(&world, Some(&tables), &mut bar, Instant::now());
        assert_eq!(keys.quest.quests.len(), 1);
        let (x, y) = (PAGE_RECT.x + TREE_RECT.x + 20, PAGE_RECT.y + TREE_RECT.y + 5 + 2);
        assert!(keys.mouse_down(WindowId::MAIN, x, y, &mut desktop));
        assert!(keys.quest.quests[0].expanded);
        keys.mouse_up(Some((WindowId::MAIN, x, y)), &mut desktop, &mut bar, &world);
        keys.sync(&world, Some(&tables), &mut bar, Instant::now());
        assert!(keys.quest.quests[0].expanded);
        let mut list = DrawList::default();
        keys.paint(WindowId::MAIN, &mut list, 0, 0, &world, &crate::ui2d::tests::small_font(), 1.0);
        assert!(list.quads.iter().any(|q| q.texture == killer_ui::quest::MINUS_BTN));
        assert!(list.quads.iter().all(|q| q.texture != PLUS_BTN));
    }

    #[test]
    fn the_quest_location_checkbox_takes_its_label_from_the_sysstring_table() {
        let (mut keys, _desktop, _bar, _world) = setup();
        let mut tables = DatTables::default();
        tables.sysstring.insert(killer_ui::quest::LOCATION_SYSSTRING, "Location".into());
        keys.set_sysstrings(&tables);
        assert_eq!(keys.quest.location_label, "Location");
    }

    #[test]
    fn clicking_a_quest_title_through_the_whole_app_path_expands_it() {
        use killer_platform::time::Instant;
        let mut ui = crate::ui2d::fixture_ui((1024, 768));
        let world = World::default();
        ui.keys.main.tab = MainTab::Quests;
        for window in &mut ui.desktop.windows {
            window.visible = window.id == WindowId::MAIN;
        }
        let step = |name: &str| QuestStep { sub_name: name.into(), description: "d".into(), ..Default::default() };
        ui.keys.quest.quests = vec![killer_ui::quest::QuestNode {
            quest_id: 1,
            title: "An Arrogant Search".into(),
            level: 60,
            max_level: 75,
            steps: vec![step("one"), step("two")],
            ..Default::default()
        }];
        let rect = ui.desktop.window(WindowId::MAIN).unwrap().rect;
        let text_x = killer_ui::quest::ROOT_X + 19 + 20;
        for dx in [killer_ui::quest::ROOT_X + 5, text_x] {
            ui.keys.quest.quests[0].expanded = false;
            ui.keys.quest.quests[0].selected = false;
            let (x, y) = (rect.x + PAGE_RECT.x + killer_ui::quest::TREE_RECT.x + dx, rect.y + PAGE_RECT.y + killer_ui::quest::TREE_RECT.y + 8);
            assert!(ui.mouse_down_at(x, y, Instant::now(), &world), "dx={dx}");
            ui.mouse_up_in(&world);
            assert!(ui.keys.quest.quests[0].expanded, "dx={dx}");
        }
    }

    #[test]
    fn the_remaining_clan_buttons_open_their_dialogs_and_commands() {
        let (mut keys, mut desktop, _bar, mut world) = setup();
        keys.pledge_action(PledgeButton::Title, &world, &mut desktop);
        keys.pledge_action(PledgeButton::DeleteTitle, &world, &mut desktop);
        assert_eq!(keys.take_commands(), vec![UiCommand::SystemMessage(TARGET_NOT_FOUND_SYSMSG), UiCommand::SystemMessage(TARGET_NOT_FOUND_SYSMSG)]);
        let mut other = world.me.clone().unwrap();
        other.object_id = 9;
        other.name = "Amigo".into();
        world.entities.insert(9, other);
        world.target = Some(9);
        for button in [PledgeButton::Title, PledgeButton::DeleteTitle, PledgeButton::DeclareWar, PledgeButton::EndWar, PledgeButton::DeleteCrest, PledgeButton::DeleteInsignia] {
            keys.pledge_action(button, &world, &mut desktop);
        }
        assert_eq!(
            keys.dialogs,
            vec![
                Dialog::GiveTitle { target: "Amigo".into() },
                Dialog::DeleteTitle { target: "Amigo".into() },
                Dialog::PledgeWarName { start: true },
                Dialog::PledgeWarName { start: false },
                Dialog::DeleteCrest,
                Dialog::DeleteInsignia,
            ]
        );
        keys.pledge_action(PledgeButton::Penalty, &world, &mut desktop);
        assert_eq!(keys.take_commands(), vec![UiCommand::Net(NetCommand::UserCommand { command: 100 })]);
        assert!(!keys.community_request);
        keys.pledge_action(PledgeButton::Community, &world, &mut desktop);
        assert!(keys.community_request);
    }

    fn crest_bmp(width: usize, height: usize) -> Vec<u8> {
        let mut file = vec![0u8; 0x436 + width * height];
        file[0..2].copy_from_slice(b"BM");
        let length = file.len() as u32;
        file[2..6].copy_from_slice(&length.to_le_bytes());
        file[10..14].copy_from_slice(&0x436u32.to_le_bytes());
        file[14..18].copy_from_slice(&40u32.to_le_bytes());
        file[0x12..0x16].copy_from_slice(&(width as u32).to_le_bytes());
        file[0x16..0x1A].copy_from_slice(&(height as u32).to_le_bytes());
        file[0x1A] = 1;
        file[0x1C] = 8;
        for index in 0..256usize {
            file[0x36 + index * 4..0x36 + index * 4 + 3].copy_from_slice(&[index as u8, 255 - index as u8, 7]);
        }
        for (index, pixel) in file[0x436..].iter_mut().enumerate() {
            *pixel = (index * 5) as u8;
        }
        file
    }

    #[test]
    fn set_crest_and_set_insignia_open_the_notice_and_the_file_sends_the_dds_as_the_original() {
        use killer_assets::crest::CrestKind;
        let (mut keys, mut desktop, _bar, world) = setup();
        keys.pledge_action(PledgeButton::SetCrest, &world, &mut desktop);
        keys.gm_clan_action(PledgeButton::SetInsignia, &world, &mut desktop);
        assert_eq!(keys.dialogs, vec![Dialog::CrestNotice { large: false }, Dialog::CrestNotice { large: true }]);
        keys.crest_file(CrestKind::Clan, Some(&crest_bmp(16, 12)));
        let sent = keys.take_commands();
        let [UiCommand::Net(NetCommand::SetPledgeCrest { data })] = sent.as_slice() else { panic!("{sent:?}") };
        assert_eq!((data.len(), &data[..4], data[12], data[16]), (0x80 + 128, &b"DDS "[..], 16, 16));
        keys.crest_file(CrestKind::Large, Some(&crest_bmp(64, 64)));
        let sent = keys.take_commands();
        let [UiCommand::Net(NetCommand::ExSetPledgeCrestLarge { data })] = sent.as_slice() else { panic!("{sent:?}") };
        assert_eq!(data.len(), 0x80 + 2048);
        keys.crest_file(CrestKind::Clan, Some(&crest_bmp(32, 12)));
        keys.crest_file(CrestKind::Large, None);
        let colored = |id| UiCommand::ColoredSystemMessage { id, color: killer_ui::color::GOLD.0 };
        assert_eq!(keys.take_commands(), vec![colored(211), colored(528)]);
    }

    #[test]
    fn the_gm_clan_buttons_penalty_community_and_delete_insignia_act_like_the_players() {
        let (mut keys, mut desktop, _bar, world) = setup();
        keys.gm_clan_action(PledgeButton::Penalty, &world, &mut desktop);
        assert_eq!(keys.take_commands(), vec![UiCommand::Net(NetCommand::UserCommand { command: 100 })]);
        keys.gm_clan_action(PledgeButton::Community, &world, &mut desktop);
        assert!(keys.community_request);
        keys.gm_clan_action(PledgeButton::DeleteInsignia, &world, &mut desktop);
        assert_eq!(keys.dialogs, vec![Dialog::DeleteInsignia]);
    }

    #[test]
    fn abort_without_a_selection_shows_the_select_notice_and_with_one_asks_for_confirmation() {
        let (mut keys, mut desktop, mut bar, world) = setup();
        keys.main.tab = MainTab::Quests;
        keys.quest.quests = vec![killer_ui::quest::QuestNode { quest_id: 7, title: "t".into(), ..Default::default() }];
        let abort = killer_ui::quest::ABORT_RECT;
        let (x, y) = (PAGE_RECT.x + abort.x + 2, PAGE_RECT.y + abort.y + 2);
        for selected in [None, Some(0)] {
            keys.quest.selected = selected;
            keys.mouse_down(WindowId::MAIN, x, y, &mut desktop);
            keys.mouse_up(Some((WindowId::MAIN, x, y)), &mut desktop, &mut bar, &world);
        }
        assert_eq!(
            keys.dialogs,
            vec![
                Dialog::Notice(SystemMessage { id: 1201, args: Vec::new() }),
                Dialog::AbortQuest { quest_id: 7 },
            ]
        );
    }

    #[test]
    fn a_snoop_line_opens_its_window_and_the_fifth_snoop_asks_for_a_notice_0x10069bf0() {
        let (mut keys, mut desktop, _, _) = setup();
        let font = crate::ui2d::tests::font("f");
        let snoop = |id: i32, kind: i32| killer_world::WorldEvent::Snoop(l2_net::game::packets::Snoop {
            first: id,
            first_text: format!("alvo{id}"),
            second: 0,
            third: kind,
            second_text: "Ana".into(),
            third_text: "oi".into(),
        });
        let visible = |desktop: &Desktop, index: usize| desktop.window(killer_ui::gm_snoop::WINDOW_IDS[index]).is_some_and(|window| window.visible);
        assert!(!visible(&desktop, 0));
        keys.snoop_event(&snoop(10, 5), &font, &mut desktop);
        assert!(!visible(&desktop, 0) && keys.snoop.windows[0].snoop_id == 10);
        keys.snoop_event(&snoop(10, 0), &font, &mut desktop);
        assert!(visible(&desktop, 0));
        assert_eq!(keys.snoop.windows[0].list.lines()[0].text, "Ana: oi");
        for id in 11..14 {
            keys.snoop_event(&snoop(id, 0), &font, &mut desktop);
        }
        assert!(keys.dialogs.is_empty());
        keys.snoop_event(&snoop(99, 0), &font, &mut desktop);
        assert_eq!(keys.dialogs, vec![Dialog::GmNotice { message_id: 0x322 }]);
    }

    #[test]
    fn the_debug_commands_toggle_the_log_and_the_window_and_the_window_paints_its_lines_0x1006ba20() {
        let (mut keys, mut desktop, _, world) = setup();
        let font = crate::ui2d::tests::font("f");
        keys.debug.resize(1558);
        keys.debug.categories = killer_ui::debug_wnd::load_categories(&killer_ui::ini::parse_ini("[DebugInfo]\ncount=1\n[Library0]\nname0=Net\nIsEnable0=True\n"));
        let visible = |desktop: &Desktop| desktop.window(killer_ui::debug_wnd::ID).unwrap().visible;
        assert!(!visible(&desktop) && !keys.debug.log_enabled);
        keys.chat_command(&world, "/L2DebugWindow");
        keys.chat_command(&world, "/l2debug");
        for command in keys.take_commands() {
            if let UiCommand::DebugCommand { window } = command {
                keys.debug_command(window, &mut desktop);
            }
        }
        assert!(visible(&desktop) && keys.debug.log_enabled);
        keys.debug.debug_logf(0, "ola");
        let mut list = DrawList::default();
        keys.paint(killer_ui::debug_wnd::ID, &mut list, 0, 0, &world, &font, 1.0);
        assert_eq!(list.quads[0].texture, killer_ui::debug_wnd::BACK_TEXTURE);
        assert_eq!(keys.debug.list.items[0].text, "[Net]ola");
        keys.debug_command(true, &mut desktop);
        assert!(!visible(&desktop));
    }

    #[test]
    fn the_detail_status_view_reads_the_world() {
        let (_, _, _, mut world) = setup();
        world.stats.p_atk = 77;
        world.social.pk_kills = 4;
        let view = detail_status_view(&world);
        assert_eq!(view.name, world.me.as_ref().unwrap().name);
        assert_eq!(view.p_atk, 77);
        assert_eq!(view.pk_kills, 4);
        assert_eq!(view.load, DEMO_LOAD.0);
        let _ = Shortcut { slot: 0, kind: ShortcutKind::Action { id: 0 }, character_type: 1 };
        let _ = InventoryItem { type1: 0, object_id: 0, item_id: 0, count: 0, type2: 0, custom_type1: 0, equipped: false, body_part: 0, enchant: 0, custom_type2: 0 };
    }
}
