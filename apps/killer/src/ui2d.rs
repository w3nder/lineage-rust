use killer_world::NetCommand;
use crate::characters::{LABEL_ALLY_CREST_WIDTH, LABEL_CREST_HEIGHT, LABEL_CREST_V, LABEL_PLEDGE_CREST_WIDTH, Label};
use crate::ui_keys::{EffectResult, KeyWindows, UiCommand};
use killer_assets::client::{AssetError, Client, DatTables};
use killer_ui::abnormal::{
    ABNORMAL_DEFAULT_POS, ABNORMAL_SIZE, AbnormalView, BuffIcon, GRIP_SIZE as ABNORMAL_GRIP, paint_abnormal,
};
use killer_ui::chat::{self, CHAT_HEADER, ChatEntry, ChatView, FilterHit, InputKey, TAB_SYSSTRING_IDS};
use killer_ui::color::Argb as ChatColor;
use killer_ui::color::Argb;
use killer_ui::dialog::{self, DIALOG_SIZE, DialogButton, DialogKind, DialogView, paint_dialog};
use killer_ui::draw::DrawList;
use killer_ui::font::{BitmapFont, FontSet, Glyphs};
use killer_ui::frame::{FrameCtrl, FrameMode};
use killer_ui::gauge::{GaugeView, paint_gauges};
use killer_ui::html::{bypass_command, link_target};
use killer_ui::ini::{ChatFilter, IniFile, Options, WindowsInfo, frame_alpha, parse_ini};
use killer_ui::keys::{Effect, Hooked};
use killer_ui::loading::{loading_texture, paint_loading};
use killer_ui::menu::{self, MENU_SIZE, MenuButton, paint_menu};
use killer_ui::npc_viewer::{
    self, CHAT_TITLE_SYSSTRING, DEFAULT_TITLE, NPC_VIEWER_SIZE, NpcViewer, HTML_WHEEL_STEP,
};
use killer_ui::party::{self, PARTY_DEFAULT_POS, PARTY_WIDTH, PartyHit, PartyMemberView, PartyView, paint_party};
use killer_ui::pet_status::{
    self, PET_STATUS_DEFAULT_POS, PET_STATUS_SIZE, PetHit, PetStatusView, ThirdBar, paint_pet_status_sized,
};
use killer_ui::radar::{RADAR_SIZE, RadarView};
use killer_ui::restart::{self, RESTART_SIZE, RestartView, paint_restart};
use killer_ui::shortcut::{PageButton, ShortcutView};
use killer_ui::status::{STATUS_SIZE, StatusView, paint_status_sized};
use killer_ui::sysmsg::{self, SYSTEM_MSG_SIZE, SystemMsgView};
use killer_ui::target::{TARGET_SIZE, TargetKind, TargetView, paint_target_sized};
use killer_ui::tooltip;
use killer_ui::window::{Desktop, Rect, Window, WindowId};
use killer_ui::window_registry::{RegistryError, WindowRegistry};
use killer_ui::zone_title::{self, TrueTypeFont, ZoneTitleView, paint_zone_title};
use killer_world::chat::CHAT_SYSTEM;
use killer_world::{ChatLine, Dialog, Entity, EntityKind, World};
use l2_net::game::packets::{NpcHtmlMessage, SUMMON_PET};
use l2_net::game::{SystemMessage, SystemMessageArg};
use std::collections::HashMap;
use killer_platform::time::Instant;
use winit::keyboard::{Key, NamedKey};

pub const STATUS_DEFAULT_POS: (i32, i32) = (0, 0);
pub const SHORTCUT_DEFAULT_ROTATE: bool = false;
pub const ENGINE_NAME_RISE: i32 = 15;
pub const DEFAULT_CHAT_HEIGHT: i32 = 187;
pub const NPC_NAME_BASE: i32 = 1_000_000;
const HTML_PREVIEW_CHARS: usize = 120;
const LOCALIZATION_INI: &str = "localization.ini";
const WINDOWS_INFO_INI: &str = "WindowsInfo.ini";
const OPTION_INI: &str = "option.ini";
const CHAT_FILTER_INI: &str = "chatfilter.ini";
const TTFONTINFO_INI: &str = "ttfontinfo.ini";
pub const PARTY_INVITE_SYSMSG: [u32; 5] = [572, 573, 967, 968, 969];
pub const PLEDGE_INVITE_SYSMSG: u32 = 67;
/// "Are you sure you wish to delete the macro?" — `pushl $0x33c` em `0x100a9b47`.
pub const DELETE_MACRO_SYSMSG: u32 = 0x33c;
pub const ALLIANCE_INVITE_SYSMSG: u32 = 527;
pub const TRADE_REQUEST_SYSMSG: u32 = 100;
/// systemmsg 516 = "$s1 has invited you as a Friend." — o `ReceiveFriendAddRequest @
/// 0x10406c20` empilha o nome (`"S"`) e abre o diálogo; o `$s1` é esse nome. Confirmado no
/// `systemmsg` do cliente.
pub const FRIEND_INVITE_SYSMSG: u32 = 516;
pub const DIALOG_TYPE_PARTY: i32 = 10;
pub const DIALOG_TYPE_PLEDGE: i32 = 14;
pub const DIALOG_TYPE_ALLIANCE: i32 = 51;
pub const DIALOG_TYPE_CONFIRM: i32 = 70;
pub const DIALOG_TYPE_NOTICE: i32 = 31;
pub const DIALOG_TYPE_START_ALLIANCE_WAR: i32 = 0x34;
pub const DIALOG_TYPE_STOP_ALLIANCE_WAR: i32 = 0x35;
pub const START_ALLIANCE_WAR_SYSMSG: u32 = 665;
pub const STOP_ALLIANCE_WAR_SYSMSG: u32 = 666;
pub const DIALOG_TYPE_ABORT_QUEST: i32 = 0x1e;
pub const ABORT_QUEST_SYSMSG: u32 = 182;
pub const DIALOG_TYPE_SYSMENU_RESTART: i32 = 0x1b;
pub const DIALOG_TYPE_SYSMENU_EXIT: i32 = 0x1c;
pub const DIALOG_TYPE_DISCONNECTED: i32 = 0x1d;
pub const DISCONNECTED_RETRY_SYSMSG: u32 = 127;
pub const CONFIRM_NOT_CANCELLED_SYSMSG: i32 = 1510;
pub const SYSMENU_RESTART_SYSMSG: u32 = 126;
pub const SYSMENU_EXIT_SYSMSG: u32 = 125;
pub const DIALOG_TYPE_USE_ITEM_POPUP: i32 = 0x45;
pub const DIALOG_TYPE_RECIPE_REGISTER: i32 = 0x21;
pub const DROP_DIALOG_TYPE: i32 = 4;
pub const DESTROY_DIALOG_TYPE: i32 = 5;
pub const CRYSTALLIZE_DIALOG_TYPE: i32 = 6;
pub const PET_NAME_DIALOG_TYPE: i32 = 0x39;
pub const PET_NAME_SYSMSG: u32 = 0x217;
pub const PET_NAME_TOO_LONG_SYSMSG: u32 = 0x50;
pub const PET_NAME_INVALID_SYSMSG: u32 = 0x24f;
pub const PET_NAME_MAX_BYTES: usize = 0x10;
pub const PET_NAME_MESSAGE_COLOR: u32 = 0xffb09b79;
pub const PET_GIVE_DIALOG_TYPE: i32 = 0x36;
pub const PET_GET_DIALOG_TYPE: i32 = 0x37;
pub const PET_MOVE_COUNT_SYSMSG: u32 = 0x48;
pub const PET_EQUIPPED_COUNT_SYSMSG: u32 = 0x23F;
pub const DROP_COUNT_SYSMSG: u32 = 71;
pub const DROP_CONFIRM_SYSMSG: u32 = 400;
pub const DROP_PLACE_SYSMSG: u32 = 651;
pub const DROP_PLACE_FAMILY: u32 = 8;
pub const DESTROY_COUNT_SYSMSG: u32 = 0x49;
pub const DESTROY_CONFIRM_SYSMSG: u32 = 0x4a;
pub const CRYSTALLIZE_SYSMSG: u32 = 0x150;
pub const RECIPE_REGISTER_SYSMSG: u32 = 798;
pub const DIALOG_TYPE_OUST_PLEDGE_MEMBER: i32 = 0x18;
pub const DIALOG_TYPE_GIVE_TITLE: i32 = 0xf;
pub const DIALOG_TYPE_PLEDGE_WAR_START: i32 = 0x12;
pub const DIALOG_TYPE_PLEDGE_WAR_STOP: i32 = 0x14;
pub const DIALOG_TYPE_DELETE_TITLE: i32 = 0x48;
pub const DIALOG_TYPE_ALLY_DISMISS_NAME: i32 = 0x32;
pub const DIALOG_TYPE_ALLY_DISMISS: i32 = 0x4d;
pub const DIALOG_TYPE_JOIN_MPCC: i32 = 0x52;
pub const DIALOG_TYPE_HTML_LINK: i32 = 0x3c;
pub const JOIN_MPCC_SYSMSG: u32 = 0x5f9;
pub const DIALOG_TYPE_DELETE_CREST: i32 = 0x49;
pub const DIALOG_TYPE_DELETE_INSIGNIA: i32 = 0x55;
pub const DIALOG_TYPE_CREST_NOTICE: i32 = 0x10;
pub const DIALOG_TYPE_CREST_FILE: i32 = 0x11;
pub const DIALOG_TYPE_INSIGNIA_NOTICE: i32 = 0x53;
pub const DIALOG_TYPE_INSIGNIA_FILE: i32 = 0x54;
pub const CREST_FILE_SYSMSG: u32 = 208;
pub const GIVE_TITLE_SYSMSG: u32 = 256;
pub const DELETE_TITLE_SYSMSG: u32 = 1181;
pub const DELETE_CREST_SYSMSG: u32 = 1182;
pub const PLEDGE_WAR_NAME_START_SYSMSG: u32 = 1531;
pub const PLEDGE_WAR_NAME_STOP_SYSMSG: u32 = 1532;
pub const TITLE_INVALID_SYSMSG: u32 = 204;
pub const TITLE_LENGTH_SYSMSG: u32 = 80;
pub const TITLE_MAX_BYTES: usize = 0x10;
pub const TITLE_MESSAGE_COLOR: u32 = 0xffb0_9b79;
pub const OUST_PLEDGE_MEMBER_SYSMSG: u32 = 69;
pub const DIALOG_TYPE_WITHDRAW_PLEDGE: i32 = 0x1a;
pub const WITHDRAW_PLEDGE_SYSMSG: u32 = 68;
pub const NOTICE_SYSMSG_GROUP: u32 = 4;
pub const DOUBLE_CLICK_TIME_MS: u128 = 500;
pub const CREST_PACKAGE: &str = "crest";
pub const DOUBLE_CLICK_RECT: (i32, i32) = (4, 4);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    Ignored,
    Consumed,
    Say(String),
    Bypass(String),
    LinkHtml(String),
    Dialog(DialogAnswer, bool),
    Select(i32),
    Assist(i32),
    RestartPoint(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlSource {
    Npc,
    Item,
    Board,
    Tutorial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogAnswer {
    Siege(killer_world::state::SiegeDecision),
    Shop,
    GmStore,
    ShopPreview,
    PrivateShop,
    RecipeShop,
    Warehouse,
    ManorShop,
    Delivery,
    TradeCount,
    MultiSellCount,
    ManorSetting,
    JoinPledge,
    JoinParty,
    JoinAlliance,
    Confirm { message_id: i32 },
    Notice,
    Trade,
    Friend,
    UseItem { object_id: i32 },
    OustPledgeMember { name: String },
    AbortQuest { quest_id: i32 },
    SystemMenu { restart: bool },
    Disconnected,
    GmConfirm { command: String },
    WithdrawPledge,
    AllyConfirm { dissolve: bool },
    AllyDismissName,
    AllyDismiss { pledge_name: String },
    JoinMpcc,
    DeleteMacro { id: i32 },
    DeleteRecipe { recipe_id: i32 },
    PetName,
    DropItem { object_id: i32, count: i32, position: [i32; 3], kind: killer_world::ItemDropKind },
    PledgeWar { pledge_name: String, start: bool },
    GiveTitle { target: String },
    DeleteTitle { target: String },
    PledgeWarName { start: bool },
    DeleteCrest,
    DeleteInsignia,
    HtmlLink { action: String, source: HtmlSource },
    CrestNotice { large: bool },
    CrestFile { large: bool },
}

fn magic_skill_data_accepted(world: &World, cast: &l2_net::game::MagicSkillUse, tables: Option<&DatTables>) -> bool {
    world.cast_accepted(cast).is_some()
        && tables.is_none_or(|tables| {
            u32::try_from(cast.skill_id)
                .ok()
                .zip(u32::try_from(cast.skill_level).ok())
                .is_some_and(|key| tables.skillgrp.contains_key(&key))
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveDialog {
    pub view: DialogView,
    pub answer: DialogAnswer,
    pub window: WindowId,
    opened: Instant,
    pressed: Option<DialogButton>,
}

impl ActiveDialog {
    pub fn elapsed_ms(&self, now: Instant) -> i32 {
        now.saturating_duration_since(self.opened).as_millis().min(i32::MAX as u128) as i32
    }
}

pub const OLYMPIAD_MAX_SKILL_ID: i32 = 1999;
pub const OLYMPIAD_USE_SYSMSG: u32 = 46;
pub const OLYMPIAD_AVOIDED_SYSMSG: u32 = 42;
pub const OLYMPIAD_CRITICAL_SYSMSG: u32 = 44;
pub const OLYMPIAD_SHIELD_SYSMSG: u32 = 111;
pub const OLYMPIAD_HIT_MISS: u8 = 0x80;
pub const OLYMPIAD_HIT_SHIELD: u8 = 0x40;
pub const OLYMPIAD_HIT_CRITICAL: u8 = 0x20;

pub fn system_text(tables: Option<&DatTables>, id: u32, args: &[SystemMessageArg]) -> String {
    let Some(tables) = tables else { return format!("systemmsg {id}") };
    system_line(tables, &SystemMessage { id: id as i32, args: args.to_vec() })
        .map_or_else(|| format!("systemmsg {id}"), |line| line.text)
}

pub fn link_message_text(tables: Option<&DatTables>, message: &str) -> String {
    match killer_ui::html::message_code(message) {
        Some((id, argument)) => system_text(tables, id as u32, &[SystemMessageArg::Text(argument)]),
        None => message.to_string(),
    }
}

pub fn dialog_request(
    dialog: &Dialog,
    world: &World,
    tables: Option<&DatTables>,
) -> (DialogView, DialogAnswer, WindowId) {
    let text = |arg: &str| SystemMessageArg::Text(arg.to_string());
    let name_of =
        |object_id: i32| world.entities.get(&object_id).map_or_else(|| object_id.to_string(), |e| e.name.clone());
    let typed = |mut view: DialogView, dialog_type: i32| {
        view.dialog_type = dialog_type;
        view
    };
    match dialog {
        Dialog::Siege { message_id, dialog_type, parameter, decision } => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, *message_id, &[text(parameter)]));
            (typed(view, *dialog_type), DialogAnswer::Siege(decision.clone()), WindowId::DIALOG)
        }
        Dialog::JoinParty(ask) => {
            let message = usize::try_from(ask.item_distribution)
                .ok()
                .and_then(|index| PARTY_INVITE_SYSMSG.get(index))
                .map_or_else(String::new, |id| system_text(tables, *id, &[text(&ask.requestor_name)]));
            let view = DialogView::new(DialogKind::YesNo, &message);
            (typed(view, DIALOG_TYPE_PARTY), DialogAnswer::JoinParty, WindowId::DIALOG_BOTTOM)
        }
        Dialog::JoinPledge(ask) => {
            let args = [text(&name_of(ask.requestor_id)), text(&ask.pledge_name)];
            let view = DialogView::new(DialogKind::YesNo, &system_text(tables, PLEDGE_INVITE_SYSMSG, &args));
            (typed(view, DIALOG_TYPE_PLEDGE), DialogAnswer::JoinPledge, WindowId::DIALOG_BOTTOM)
        }
        Dialog::JoinAlliance(ask) => {
            let args = [text(&ask.requestor_ally_name), text(&ask.requestor_name)];
            let view = DialogView::new(DialogKind::YesNo, &system_text(tables, ALLIANCE_INVITE_SYSMSG, &args));
            (typed(view, DIALOG_TYPE_ALLIANCE), DialogAnswer::JoinAlliance, WindowId::DIALOG_BOTTOM)
        }
        Dialog::Confirm(message) => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, message.id as u32, &message.args));
            (typed(view, DIALOG_TYPE_CONFIRM), DialogAnswer::Confirm { message_id: message.id }, WindowId::DIALOG)
        }
        Dialog::Notice(message) => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, message.id as u32, &message.args));
            (typed(view, DIALOG_TYPE_NOTICE), DialogAnswer::Notice, WindowId::DIALOG)
        }
        Dialog::TradeRequest { requestor_id } => {
            let view = DialogView::new(
                DialogKind::YesNo,
                &system_text(tables, TRADE_REQUEST_SYSMSG, &[text(&name_of(*requestor_id))]),
            );
            (view, DialogAnswer::Trade, WindowId::DIALOG_BOTTOM)
        }
        Dialog::FriendRequest { requestor_name } => {
            let view = DialogView::new(
                DialogKind::YesNo,
                &system_text(tables, FRIEND_INVITE_SYSMSG, &[text(requestor_name)]),
            );
            (view, DialogAnswer::Friend, WindowId::DIALOG_BOTTOM)
        }
        Dialog::UseItemPopup { object_id, message_id, item_name } => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, *message_id as u32, &[text(item_name)]));
            (typed(view, DIALOG_TYPE_USE_ITEM_POPUP), DialogAnswer::UseItem { object_id: *object_id }, WindowId::DIALOG)
        }
        Dialog::OustPledgeMember { name } => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, OUST_PLEDGE_MEMBER_SYSMSG, &[text(name)]));
            (typed(view, DIALOG_TYPE_OUST_PLEDGE_MEMBER), DialogAnswer::OustPledgeMember { name: name.clone() }, WindowId::DIALOG)
        }
        Dialog::AbortQuest { quest_id } => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, ABORT_QUEST_SYSMSG, &[]));
            (typed(view, DIALOG_TYPE_ABORT_QUEST), DialogAnswer::AbortQuest { quest_id: *quest_id }, WindowId::DIALOG)
        }
        Dialog::Disconnected { reason } => {
            let text = if (0..=2).contains(reason) { system_text(tables, DISCONNECTED_RETRY_SYSMSG, &[]) } else { String::new() };
            let view = DialogView::new(DialogKind::OkCancel, &text);
            (typed(view, DIALOG_TYPE_DISCONNECTED), DialogAnswer::Disconnected, WindowId::DIALOG)
        }
        Dialog::SystemMenu { restart } => {
            let (message, dialog_type) = if *restart {
                (SYSMENU_RESTART_SYSMSG, DIALOG_TYPE_SYSMENU_RESTART)
            } else {
                (SYSMENU_EXIT_SYSMSG, DIALOG_TYPE_SYSMENU_EXIT)
            };
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, message, &[]));
            (typed(view, dialog_type), DialogAnswer::SystemMenu { restart: *restart }, WindowId::DIALOG)
        }
        Dialog::GmConfirm { dialog_type, message_id, name, command } => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, *message_id as u32, &[text(name)]));
            (typed(view, *dialog_type), DialogAnswer::GmConfirm { command: command.clone() }, WindowId::DIALOG)
        }
        Dialog::GmText { text } => {
            let view = DialogView::new(DialogKind::OkCancel, text);
            (typed(view, DIALOG_TYPE_NOTICE), DialogAnswer::Notice, WindowId::DIALOG)
        }
        Dialog::GmNotice { message_id } => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, *message_id as u32, &[]));
            (typed(view, DIALOG_TYPE_NOTICE), DialogAnswer::Notice, WindowId::DIALOG)
        }
        Dialog::WithdrawPledge { pledge_name } => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, WITHDRAW_PLEDGE_SYSMSG, &[text(pledge_name)]));
            (typed(view, DIALOG_TYPE_WITHDRAW_PLEDGE), DialogAnswer::WithdrawPledge, WindowId::DIALOG)
        }
        Dialog::AllyConfirm { message_id, dissolve } => {
            let view = DialogView::new(DialogKind::YesNo, &system_text(tables, *message_id, &[]));
            (typed(view, DIALOG_TYPE_CONFIRM), DialogAnswer::AllyConfirm { dissolve: *dissolve }, WindowId::DIALOG)
        }
        Dialog::JoinMpcc { name } => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, JOIN_MPCC_SYSMSG, &[text(name)]));
            (typed(view, DIALOG_TYPE_JOIN_MPCC), DialogAnswer::JoinMpcc, WindowId::DIALOG)
        }
        Dialog::AllyDismissName => {
            let view = DialogView::with_text(DialogKind::OkCancel, &system_text(tables, crate::ui_keys::ALLY_DISMISS_SYSMSG, &[]));
            (typed(view, DIALOG_TYPE_ALLY_DISMISS_NAME), DialogAnswer::AllyDismissName, WindowId::DIALOG)
        }
        Dialog::AllyDismissConfirm { pledge_name } => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, crate::ui_keys::ALLY_DISMISS_CONFIRM_SYSMSG, &[text(pledge_name)]));
            (typed(view, DIALOG_TYPE_ALLY_DISMISS), DialogAnswer::AllyDismiss { pledge_name: pledge_name.clone() }, WindowId::DIALOG)
        }
        Dialog::AllianceWar { start, first, second } => {
            let (message, dialog_type) = if *start { (START_ALLIANCE_WAR_SYSMSG, DIALOG_TYPE_START_ALLIANCE_WAR) } else { (STOP_ALLIANCE_WAR_SYSMSG, DIALOG_TYPE_STOP_ALLIANCE_WAR) };
            let view = DialogView::new(DialogKind::YesNo, &system_text(tables, message, &[text(first), text(second)]));
            (typed(view, dialog_type), DialogAnswer::Notice, WindowId::DIALOG_BOTTOM)
        }
        Dialog::PledgeWarConfirm { message_id, pledge_name, start } => {
            let view = DialogView::new(DialogKind::YesNo, &system_text(tables, *message_id, &[text(pledge_name)]));
            (
                typed(view, DIALOG_TYPE_CONFIRM),
                DialogAnswer::PledgeWar { pledge_name: pledge_name.clone(), start: *start },
                WindowId::DIALOG,
            )
        }
        Dialog::DeleteMacro { id, name } => {
            let view = DialogView::new(DialogKind::YesNo, &system_text(tables, DELETE_MACRO_SYSMSG, &[text(name)]));
            (typed(view, DIALOG_TYPE_CONFIRM), DialogAnswer::DeleteMacro { id: *id }, WindowId::DIALOG)
        }
        Dialog::DeleteRecipe { recipe_id, name } => {
            let view = DialogView::new(DialogKind::YesNo, &system_text(tables, killer_ui::recipe::DELETE_SYSMSG, &[text(name)]));
            (typed(view, killer_ui::recipe::DELETE_DIALOG_TYPE), DialogAnswer::DeleteRecipe { recipe_id: *recipe_id }, WindowId::DIALOG)
        }
        Dialog::GiveTitle { target } => {
            let view = DialogView::with_text(DialogKind::OkCancel, &system_text(tables, GIVE_TITLE_SYSMSG, &[]));
            (typed(view, DIALOG_TYPE_GIVE_TITLE), DialogAnswer::GiveTitle { target: target.clone() }, WindowId::DIALOG)
        }
        Dialog::DeleteTitle { target } => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, DELETE_TITLE_SYSMSG, &[]));
            (typed(view, DIALOG_TYPE_DELETE_TITLE), DialogAnswer::DeleteTitle { target: target.clone() }, WindowId::DIALOG)
        }
        Dialog::PledgeWarName { start } => {
            let (message, dialog_type) = if *start { (PLEDGE_WAR_NAME_START_SYSMSG, DIALOG_TYPE_PLEDGE_WAR_START) } else { (PLEDGE_WAR_NAME_STOP_SYSMSG, DIALOG_TYPE_PLEDGE_WAR_STOP) };
            let view = DialogView::with_text(DialogKind::OkCancel, &system_text(tables, message, &[]));
            (typed(view, dialog_type), DialogAnswer::PledgeWarName { start: *start }, WindowId::DIALOG)
        }
        Dialog::DeleteCrest => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, DELETE_CREST_SYSMSG, &[]));
            (typed(view, DIALOG_TYPE_DELETE_CREST), DialogAnswer::DeleteCrest, WindowId::DIALOG)
        }
        Dialog::CrestNotice { large } => {
            let (message, dialog_type) = if *large { (killer_assets::crest::SYSMSG_LARGE_FORMAT, DIALOG_TYPE_INSIGNIA_NOTICE) } else { (killer_assets::crest::SYSMSG_CLAN_FORMAT, DIALOG_TYPE_CREST_NOTICE) };
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, message, &[]));
            (typed(view, dialog_type), DialogAnswer::CrestNotice { large: *large }, WindowId::DIALOG)
        }
        Dialog::CrestFile { large } => {
            let view = DialogView::with_text(DialogKind::OkCancel, &system_text(tables, CREST_FILE_SYSMSG, &[]));
            (typed(view, if *large { DIALOG_TYPE_INSIGNIA_FILE } else { DIALOG_TYPE_CREST_FILE }), DialogAnswer::CrestFile { large: *large }, WindowId::DIALOG)
        }
        Dialog::DeleteInsignia => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, DELETE_CREST_SYSMSG, &[]));
            (typed(view, DIALOG_TYPE_DELETE_INSIGNIA), DialogAnswer::DeleteInsignia, WindowId::DIALOG)
        }
        Dialog::PetName => {
            let view = DialogView::with_text(DialogKind::OkCancel, &system_text(tables, PET_NAME_SYSMSG, &[]));
            (typed(view, PET_NAME_DIALOG_TYPE), DialogAnswer::PetName, WindowId::DIALOG)
        }
        Dialog::DropItem { object_id, name, message_id, count, position, kind } => {
            let message = system_text(tables, *message_id, &[text(name)]);
            let view = match count {
                Some(max) => DialogView::with_count(DialogKind::OkCancel, &message, i64::from(*max)),
                None => DialogView::new(DialogKind::OkCancel, &message),
            };
            let dialog_type = match kind {
                killer_world::ItemDropKind::GiveToPet => PET_GIVE_DIALOG_TYPE,
                killer_world::ItemDropKind::GetFromPet => PET_GET_DIALOG_TYPE,
                killer_world::ItemDropKind::Drop => DROP_DIALOG_TYPE,
                killer_world::ItemDropKind::Destroy => DESTROY_DIALOG_TYPE,
                killer_world::ItemDropKind::Crystallize => CRYSTALLIZE_DIALOG_TYPE,
            };
            (typed(view, dialog_type), DialogAnswer::DropItem { object_id: *object_id, count: 1, position: *position, kind: *kind }, WindowId::DIALOG)
        }
        Dialog::RecipeRegister { object_id, item_name } => {
            let view = DialogView::new(DialogKind::OkCancel, &system_text(tables, RECIPE_REGISTER_SYSMSG, &[text(item_name)]));
            (typed(view, DIALOG_TYPE_RECIPE_REGISTER), DialogAnswer::UseItem { object_id: *object_id }, WindowId::DIALOG)
        }
    }
}

pub fn html_marker(tables: Option<&DatTables>, marker: killer_ui::html::Marker, id: i32) -> Option<String> {
    use killer_ui::html::Marker;
    let tables = tables?;
    let id = u32::try_from(id).ok();
    match marker {
        Marker::SysString => tables.sysstring.get(&id?).cloned(),
        Marker::CastleName => tables.castlename.get(&id?).map(|castle| castle.name.clone()),
        Marker::CastleLocation => tables.castlename.get(&id?).map(|castle| castle.location.clone()),
        Marker::CastleDescription => tables.castlename.get(&id?).map(|castle| castle.description.clone()),
        Marker::NpcName => tables.npcname.get(&id?).map(|npc| npc.name.clone()),
        Marker::ItemName => tables.itemname.get(&id?).map(|item| item.name.clone()),
    }
}

pub fn html_action_result(action: &str, handled: bool) -> i32 {
    let mut tokens = killer_ui::html::split_message(action).1.split_whitespace();
    if !tokens.next().is_some_and(|word| word.eq_ignore_ascii_case("bypass")) {
        return i32::from(handled);
    }
    let mut sent = false;
    for token in tokens {
        if token == "-h" {
            return 2;
        }
        sent = true;
    }
    i32::from(sent)
}

pub fn html_action(action: &str) -> Option<KeyAction> {
    if let Some(command) = killer_ui::html::cmd_target(action) {
        return Some(KeyAction::Say(command.to_string()));
    }
    bypass_command(action).map(KeyAction::Bypass).or_else(|| link_target(action).map(KeyAction::LinkHtml))
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WorldViews {
    pub abnormal: AbnormalView,
    pub pet: Option<(WindowId, i32, PetStatusView)>,
    pub restart: Option<RestartView>,
    pub radar: RadarView,
    pub party: PartyView,
    pub gauges: Vec<GaugeView>,
    pub item_label: Option<ItemLabel>,
    pub primitives: Vec<PrimitiveMark>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveMark {
    pub screen: [f32; 2],
    pub text: String,
    pub color: Argb,
    pub point: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemLabel {
    pub screen: [f32; 2],
    pub text: String,
}

pub const ITEM_LABEL_COLOR: Argb = Argb(0xFFDC_DCDC);

pub fn world_views(world: &World, tables: Option<&DatTables>, now: Instant) -> WorldViews {
    WorldViews {
        abnormal: abnormal_view(world, tables, now),
        pet: pet_view(world, tables),
        restart: restart_view(world, tables),
        radar: radar_view(world, (0, 0)),
        party: party_view(world, tables),
        gauges: gauge_views(world, now),
        item_label: None,
        primitives: Vec::new(),
    }
}

pub fn gauge_views(world: &World, now: Instant) -> Vec<GaugeView> {
    world
        .gauge
        .filter(|g| g.remaining(now) > 0)
        .map(|g| GaugeView { kind: g.setup.color, remaining_ms: g.remaining(now), max_ms: g.setup.time })
        .into_iter()
        .collect()
}

pub fn party_view(world: &World, tables: Option<&DatTables>) -> PartyView {
    // NCPartyWnd::Rearrange (docs/evidence/decompiled/ui/NCPartyWnd::Rearrange_10111320.c,
    // 0x10111320) itera 8 células fixas: a própria janela é só para os OUTROS membros do
    // grupo, já que HP/MP/CP do próprio jogador vive no player-status orb. O pacote
    // PartySmallWindowAll/Add inclui a própria entrada, então sem filtrar por is_me() o
    // 9º membro real (grupo cheio) ficava de fora das 8 células.
    let members = world
        .party
        .members
        .iter()
        .filter(|m| !world.is_me(m.object_id))
        .map(|m| PartyMemberView {
            object_id: m.object_id,
            name: m.name.clone(),
            cp: (m.cp, m.max_cp),
            hp: (m.hp, m.max_hp),
            mp: (m.mp, m.max_mp),
            leader: m.object_id == world.party.leader_id,
            buffs: world
                .party
                .spelled
                .get(&m.object_id)
                .map(|effects| {
                    effects
                        .iter()
                        .map(|b| BuffIcon {
                            skill_id: b.skill_id,
                            level: i32::from(b.level),
                            icon: skill_icon(tables, b.skill_id, i32::from(b.level)),
                            remaining: Some(b.duration),
                            debuff: skill_is_debuff(tables, b.skill_id, i32::from(b.level)),
                            fade_ticks: 0,
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect();
    PartyView { members, show_buffs: false, loot_rule: world.party.loot_distribution }
}

pub fn radar_view(world: &World, camera: (i32, i32)) -> RadarView {
    let Some(me) = world.me.as_ref() else { return RadarView::default() };
    let delta = |x: f32, y: f32| [x - me.position.x, y - me.position.y];
    let party = world
        .party
        .positions
        .iter()
        .filter(|(id, _)| **id > 0 && **id != me.object_id)
        .map(|(id, p)| match world.entities.get(id) {
            Some(pawn) => delta(pawn.position.x, pawn.position.y),
            None => delta(p[0] as f32, p[1] as f32),
        })
        .collect();
    let target = world.targeted().map(|e| delta(e.position.x, e.position.y));
    let radar = &world.windows.radar;
    let arrow_to = radar.enabled.then(|| {
        let t = radar.target.unwrap_or_default();
        [t[0] - me.position.x, t[1] - me.position.y, t[2] - me.position.z]
    });
    let points = radar.points.iter().map(|p| delta(p[0] as f32, p[1] as f32)).collect();
    RadarView { camera_pitch: camera.1, camera_yaw: camera.0, camera_roll: 0, party, target, arrow_to, points, show_time: radar.show_time }
}

pub fn restart_view(world: &World, tables: Option<&DatTables>) -> Option<RestartView> {
    let die = world.death.as_ref()?;
    let mut view =
        RestartView::new([die.to_village, die.to_hideout, die.to_castle, die.to_siege_hq, die.to_fixed_spot]);
    if let Some(tables) = tables {
        for (label, id) in view.labels.iter_mut().zip(restart::BUTTON_SYSSTRINGS) {
            if let Some(text) = tables.sysstring.get(&id) {
                *label = text.clone();
            }
        }
    }
    Some(view)
}

pub fn pet_view(world: &World, tables: Option<&DatTables>) -> Option<(WindowId, i32, PetStatusView)> {
    let pet = world.pet.as_ref()?;
    let info = &pet.info;
    let window = if info.summon_type == SUMMON_PET { WindowId::PET_STATUS } else { WindowId::SUMMONED_STATUS };
    let third = if info.summon_type == SUMMON_PET {
        ThirdBar::Fed { fed: info.fed, max_fed: info.max_fed }
    } else {
        ThirdBar::Time { remaining: pet.remaining_time, max: pet.max_time }
    };
    let buffs = world
        .party
        .spelled
        .get(&info.object_id)
        .map(|effects| effects.iter().map(|b| skill_icon(tables, b.skill_id, i32::from(b.level))).collect())
        .unwrap_or_default();
    let view = PetStatusView {
        name: info.name.clone(),
        hp: (info.hp, info.max_hp),
        mp: (info.mp, info.max_mp),
        third,
        buffs,
        show_buffs: false,
    };
    Some((window, info.object_id, view))
}

impl GameUi {
    pub fn tick_dialog(&mut self, world: &mut World, tables: Option<&DatTables>, now: Instant) -> Option<KeyAction> {
        let inventory_visible = self.desktop.window(WindowId::INVENTORY).is_some_and(|window| window.visible);
        self.keys.track_inventory(inventory_visible);
        let main_visible = self.desktop.window(WindowId::MAIN).is_some_and(|window| window.visible);
        let bar_visible = self.desktop.window(WindowId::SHORTCUT).is_some_and(|window| window.visible);
        self.keys.set_cool_time_surfaces(main_visible && self.keys.main.tab == killer_ui::keys::MainTab::Skills, bar_visible);
        if std::mem::take(&mut self.keys.multisell.dismiss_dialog) && self.dialog.as_ref().is_some_and(|d| d.answer == DialogAnswer::MultiSellCount) {
            self.close_dialog();
        }
        if let Some(answer) = self.death_answer.take() {
            return Some(answer);
        }
        if let Some(timed_out) = self.dialog_timed_out(now) {
            return Some(timed_out);
        }
        let replaceable = self.dialog.as_ref().is_some_and(|d| {
            matches!(d.answer, DialogAnswer::RecipeShop | DialogAnswer::Shop | DialogAnswer::PrivateShop | DialogAnswer::Warehouse | DialogAnswer::GmStore | DialogAnswer::ManorShop | DialogAnswer::Delivery | DialogAnswer::TradeCount | DialogAnswer::ManorSetting)
        });
        let incoming_window = self
            .keys
            .dialogs
            .first()
            .or_else(|| world.dialogs.pending.first())
            .map(|incoming| dialog_request(incoming, world, tables).2);
        if let Some(window) = incoming_window {
            let replaced = if self.dialog.as_ref().is_some_and(|dialog| dialog.window == window) {
                self.close_dialog()
            } else if self.parked_dialog.as_ref().is_some_and(|dialog| dialog.window == window) {
                self.parked_dialog.take()
            } else {
                None
            };
            if let Some(replaced) = replaced {
                let keeps_silent = replaced.answer == DialogAnswer::Confirm { message_id: CONFIRM_NOT_CANCELLED_SYSMSG };
                return (!keeps_silent).then_some(KeyAction::Dialog(replaced.answer, false));
            }
        }
        for dialog in [self.dialog.as_mut(), self.parked_dialog.as_mut()].into_iter().flatten() {
            if dialog.view.timer_elapsed_ms.is_some() {
                dialog.view.timer_elapsed_ms = Some(dialog.elapsed_ms(now));
            }
        }
        if self.dialog.is_some() {
            if self.parked_dialog.is_none() && incoming_window.is_some() {
                let pending = match self.keys.dialogs.is_empty() {
                    true => world.dialogs.take(),
                    false => Some(self.keys.dialogs.remove(0)),
                };
                if let Some(pending) = pending {
                    self.open_dialog(&pending, world, tables, now);
                    return None;
                }
            }
            if !replaceable {
                return None;
            }
        }
        if let Some(view) = self.keys.recipes.shop.take_dialog(tables) {
            self.show_dialog_view(view, DialogAnswer::RecipeShop, WindowId::DIALOG, now);
            return None;
        }
        if let Some(view) = self.keys.shop.take_dialog(tables) {
            self.show_dialog_view(view, DialogAnswer::Shop, WindowId::DIALOG, now);
            return None;
        }
        if let Some(view) = self.keys.shop_preview.take_dialog(tables) {
            self.show_dialog_view(view, DialogAnswer::ShopPreview, WindowId::DIALOG, now);
            return None;
        }
        if let Some(view) = self.keys.private_shop.take_dialog(tables) {
            self.show_dialog_view(view, DialogAnswer::PrivateShop, WindowId::DIALOG, now);
            return None;
        }
        if let Some(view) = self.keys.delivery.take_dialog(tables) {
            self.show_dialog_view(view, DialogAnswer::Delivery, WindowId::DIALOG, now);
            return None;
        }
        if let Some(view) = self.keys.warehouse.take_dialog(tables) {
            self.show_dialog_view(view, DialogAnswer::Warehouse, WindowId::DIALOG, now);
            return None;
        }
        if let Some(view) = self.keys.gm_store.take_dialog(tables) {
            self.show_dialog_view(view, DialogAnswer::GmStore, WindowId::DIALOG, now);
            return None;
        }
        if let Some(view) = self.keys.manor_shop.take_dialog(tables) {
            self.show_dialog_view(view, DialogAnswer::ManorShop, WindowId::DIALOG, now);
            return None;
        }
        if let Some(view) = self.keys.trade.take_dialog(tables) {
            self.show_dialog_view(view, DialogAnswer::TradeCount, WindowId::DIALOG, now);
            return None;
        }
        if let Some(view) = self.keys.multisell.take_dialog(tables) {
            self.show_dialog_view(view, DialogAnswer::MultiSellCount, WindowId::DIALOG, now);
            return None;
        }
        if let Some(view) = self.keys.manor_setting.take_dialog(tables) {
            self.show_dialog_view(view, DialogAnswer::ManorSetting, WindowId::DIALOG, now);
            return None;
        }
        if self.dialog.is_some() {
            return None;
        }
        let pending = match self.keys.dialogs.is_empty() {
            true => world.dialogs.take()?,
            false => self.keys.dialogs.remove(0),
        };
        self.open_dialog(&pending, world, tables, now);
        None
    }
}

pub fn skill_icon(tables: Option<&DatTables>, skill_id: i32, level: i32) -> Option<String> {
    let table = &tables?.skillgrp;
    let skill = table.lookup(u32::try_from(skill_id).ok()?, u32::try_from(level).ok()?)?;
    Some(skill.icon.clone())
}

pub fn skill_is_debuff(tables: Option<&DatTables>, skill_id: i32, level: i32) -> bool {
    tables
        .and_then(|t| t.skillgrp.lookup(u32::try_from(skill_id).ok()?, u32::try_from(level).ok()?))
        .is_some_and(|s| s.is_debuff())
}

pub fn remaining_seconds(duration: i32, since: Option<Instant>, now: Instant) -> Option<i32> {
    if duration < 0 {
        return None;
    }
    let elapsed = since.map_or(0, |at| now.saturating_duration_since(at).as_secs() as i32);
    Some((duration - elapsed).max(0))
}

pub fn fade_ticks(duration: i32, since: Option<Instant>, now: Instant) -> u32 {
    if duration < 0 {
        return 0;
    }
    let elapsed = since.map_or(0, |at| now.saturating_duration_since(at).as_millis() as u64);
    killer_ui::abnormal::blink_ticks(duration, elapsed)
}

const ETC_STATUS_SKILLS: [(i32, bool); 5] = [(4271, false), (4270, false), (4269, true), (4268, true), (4267, true)];
const ETC_STATUS_LIMIT: usize = 10;

pub fn etc_icons(values: [i32; 5]) -> Vec<(i32, i32)> {
    ETC_STATUS_SKILLS
        .iter()
        .zip(values)
        .filter(|(_, value)| *value > 0)
        .map(|((skill_id, level_one), value)| (*skill_id, if *level_one { 1 } else { value }))
        .take(ETC_STATUS_LIMIT)
        .collect()
}

pub fn abnormal_view(world: &World, tables: Option<&DatTables>, now: Instant) -> AbnormalView {
    let buffs = world
        .buffs
        .iter()
        .map(|b| BuffIcon {
            skill_id: b.skill_id,
            level: i32::from(b.level),
            icon: skill_icon(tables, b.skill_id, i32::from(b.level)),
            remaining: remaining_seconds(b.duration, world.buffs_at, now),
            debuff: skill_is_debuff(tables, b.skill_id, i32::from(b.level)),
            fade_ticks: fade_ticks(b.duration, world.buffs_at, now),
        })
        .collect();
    let short_buff = world.short_buff.map(|b| BuffIcon {
        skill_id: b.skill_id,
        level: b.level,
        icon: skill_icon(tables, b.skill_id, b.level),
        remaining: remaining_seconds(b.duration, world.short_buff_at, now),
        debuff: skill_is_debuff(tables, b.skill_id, b.level),
        fade_ticks: fade_ticks(b.duration, world.short_buff_at, now),
    });
    let etc_skills = world.etc_status.map(|status| etc_icons(status.values)).unwrap_or_default();
    let etc = etc_skills.iter().map(|&(skill_id, level)| skill_icon(tables, skill_id, level)).collect();
    AbnormalView { buffs, short_buff, etc, etc_skills }
}

pub struct GameUi {
    pub chat_second: Option<killer_platform::time::Instant>,
    pub chat_seen: u64,
    pub chat_origin: Option<(i32, i32)>,
    pub desktop: Desktop,
    pub texture_bank: killer_assets::textures::TextureBank,
    pub window_registry: WindowRegistry,
    pub small: BitmapFont,
    pub large: BitmapFont,
    pub options: Options,
    pub windows_info: WindowsInfo,
    pub alpha: f32,
    pub status_visible: bool,
    pub chat: ChatView,
    pub system_msg: SystemMsgView,
    pub option_synced: bool,
    pub audio_volumes: Option<[f32; 4]>,
    pub option_game_runtime: Option<crate::ui_options::GameOptionRuntime>,
    pub shortcut: ShortcutView,
    pub npc_viewer: Option<NpcViewer>,
    pub item_viewer: Option<NpcViewer>,
    pub npc_title: String,
    pub board: killer_ui::board::BoardView,
    pub help: crate::ui_help::HelpWindow,
    board_command: Option<KeyAction>,
    board_favorite_pressed: bool,
    pub dialog: Option<ActiveDialog>,
    pub parked_dialog: Option<ActiveDialog>,
    pub button_labels: HashMap<u32, String>,
    pub warning: killer_ui::dialog::WarningDialog,
    warning_pressed: bool,
    pub warning_exit: bool,
    pub zone_title_font: Option<TrueTypeFont>,
    pub true_fonts: killer_ui::true_font::TrueFonts,
    pub zone_title: Option<(ZoneTitleView, Instant)>,
    zone_tracker: zone_title::ZoneTracker,
    zone_clock: Option<Instant>,
    html_clock: Option<(Instant, u32)>,
    pub loading_texture: Option<String>,
    pub pet_buffs_shown: bool,
    pub party_buffs_shown: bool,
    death_answer: Option<KeyAction>,
    pet_pressed: Option<PetHit>,
    party_pressed: Option<PartyHit>,
    restart_pressed: Option<usize>,
    pub keys: KeyWindows,
    pub(crate) cursor: (i32, i32),
    last_left_press: Option<(Instant, (i32, i32))>,
    pub target_clicked: bool,
    pub status_clicked: bool,
    pet_clicked: bool,
    party_cache: PartyView,
    party_clicked: Option<usize>,
    pub target_expanded: bool,
    target_labels: [String; 4],
    rotate_pressed: bool,
    pad_pressed: bool,
    status_resize: crate::ui_status_resize::StatusResize,
    page_pressed: Option<PageButton>,
    link_pressed: Option<String>,
    menu_pressed: Option<MenuButton>,
    pub modern_menu_open: bool,
    pub modern_chat_rows: std::cell::Cell<[usize;2]>,
    pub button_map: std::cell::RefCell<Vec<(WindowId, killer_ui::draw::ButtonRegion)>>,
    close_pressed: Option<WindowId>,
    deferred_close: Option<(WindowId, i32, i32, Instant)>,
    replaying_close: bool,
    button_held: Option<(i32, i32)>,
    pub modern_system_open: bool,
    modern_menu_pressed: Option<usize>,
    pub tutorial: crate::ui_tutorial::TutorialWindows,
    pub html_tables: Option<std::sync::Arc<DatTables>>,
}

#[cfg(target_arch = "wasm32")]
pub fn load_lobby_font(client: &mut Client) -> Result<BitmapFont, Box<dyn std::error::Error + Send + Sync>> {
    let localization = parse_ini(&client.read_system_text(LOCALIZATION_INI)?);
    let fonts = FontSet::from_localization(&localization).ok_or_else(|| AssetError::NotFound(format!("{LOCALIZATION_INI}: fontes da língua")))?;
    Ok(load_font(client, &fonts.small)?)
}

impl GameUi {
    pub fn radar_visible(&self, world: &World) -> bool {
        world.me.is_some() && self.desktop.window(WindowId::RADAR).is_some_and(|window| window.visible)
    }

    pub fn load(client: &mut Client, size: (i32, i32)) -> Result<GameUi, Box<dyn std::error::Error + Send + Sync>> {
        let localization = parse_ini(&client.read_system_text(LOCALIZATION_INI)?);
        let fonts = FontSet::from_localization(&localization)
            .ok_or_else(|| AssetError::NotFound(format!("{LOCALIZATION_INI}: fontes da língua")))?;
        let small = load_font(client, &fonts.small)?;
        let large = load_font(client, &fonts.large)?;
        let windows_info = WindowsInfo::from_ini(&optional_ini(client, WINDOWS_INFO_INI));
        let option_ini = optional_ini(client, OPTION_INI);
        let options = Options::from_ini(&option_ini);
        let filter = ChatFilter::from_ini(&parse_ini(&client.read_system_text(CHAT_FILTER_INI)?));
        let mut ui = GameUi::new(small, large, options, windows_info, filter, size)?;
        log::debug!("interface: {} registros instalados em {} raízes", ui.window_registry.manager.count, ui.window_registry.manager.roots.len());
        let font_table = killer_ui::true_font::TrueFontTable::from_ini(&optional_ini(client, TTFONTINFO_INI));
        ui.zone_title_font = TrueTypeFont::zone_title(&optional_ini(client, TTFONTINFO_INI));
        ui.true_fonts = killer_ui::true_font::TrueFonts::load(font_table, |name| client.read_system_file(name).ok());
        ui.keys.broadcast.true_fonts = ui.true_fonts.clone();
        ui.loading_texture = loading_texture(&localization);
        ui.keys.set_petition_categories(killer_ui::petition_begin::categories(&localization));
        let language = localization.get("LanguageSet", "Language").and_then(|code| code.trim().parse::<i32>().ok()).and_then(killer_ui::ime::Language::from_code);
        ui.keys.set_input_language(language.unwrap_or(killer_ui::ime::Language::English));
        let system = optional_ini(client, "l2.ini");
        let runtime = crate::ui_options::GameOptionRuntime::new(option_ini, localization, &system, false, None);
        ui.shortcut.joypad_visible = runtime.state.can_use_joystick != 0 && runtime.state.using_joystick != 0;
        ui.option_game_runtime = Some(runtime);
        for path in [killer_ui::option::HEADER, killer_ui::option::BACK, killer_ui::option::AUDIO_BACK, killer_ui::option::GAME_BACK] {
            ui.texture_bank.texture(client.catalog(), path);
        }
        Ok(ui)
    }

    pub fn new(
        small: BitmapFont,
        large: BitmapFont,
        options: Options,
        windows_info: WindowsInfo,
        filter: ChatFilter,
        size: (i32, i32),
    ) -> Result<GameUi, RegistryError> {
        let viewport = options.viewport.unwrap_or(size);
        let mut desktop = Desktop::new(viewport);
        let loaded_width = |id: WindowId, base: i32| {
            let width = windows_info.width(id).unwrap_or(base);
            crate::ui_status_resize::status_width_after(base, width.wrapping_sub(base))
        };
        let (sw, sh) = (loaded_width(WindowId::STATUS, STATUS_SIZE.0), STATUS_SIZE.1);
        let (sx, sy) = windows_info.position(WindowId::STATUS).unwrap_or(STATUS_DEFAULT_POS);
        let status_grip = FrameCtrl::new(FrameMode::VerticalGrip).grip(sw, sh);
        desktop.windows.push(
            Window::new(WindowId::STATUS, Rect::new(sx, sy, sw, sh), status_grip)
                .with_initial_position(killer_ui::initial_position::STATUS_INITIAL_POSITION),
        );
        let (tw, th) = (loaded_width(WindowId::TARGET, TARGET_SIZE.0), TARGET_SIZE.1);
        let (tx, ty) = windows_info.position(WindowId::TARGET).unwrap_or_else(|| killer_ui::target::default_position(viewport));
        let target_grip = FrameCtrl::new(FrameMode::VerticalGrip).grip(tw, th);
        desktop.windows.push(
            Window::new(WindowId::TARGET, Rect::new(tx, ty, tw, th), target_grip)
                .with_initial_position(killer_ui::initial_position::TARGET_INITIAL_POSITION),
        );
        let shortcut = ShortcutView::new(!windows_info.shortcut_rotate.unwrap_or(SHORTCUT_DEFAULT_ROTATE));
        let (bw, bh) = shortcut.size();
        let (bx, by) = windows_info.position(WindowId::SHORTCUT).unwrap_or((viewport.0 - bw, viewport.1 / 2 - bh / 2));
        desktop.windows.push(Window::new(WindowId::SHORTCUT, Rect::new(bx, by, bw, bh), shortcut.grip())
            .with_initial_position(killer_ui::shortcut::initial_position(viewport)));
        let mut chat = ChatView::new(windows_info.chat_height.unwrap_or(DEFAULT_CHAT_HEIGHT), filter);
        chat.old_chat = options.old_chatting;
        chat.prefiltered = true;
        let (cw, ch) = chat.size();
        let (cx, cy) = (0, viewport.1 - ch);
        desktop.windows.push(Window::new(WindowId::CHAT, Rect::new(cx, cy, cw, ch), Rect::new(0, 0, cw, CHAT_HEADER)).with_initial_position(chat::initial_position(viewport)));
        let (sx, sy) = sysmsg::position_over_chat(Rect::new(cx, cy, cw, ch));
        let (smw, smh) = SYSTEM_MSG_SIZE;
        let mut system_window = Window::new(WindowId::SYSTEM_MSG, Rect::new(sx, sy, smw, smh), Rect::new(0, 0, 0, 0)).with_initial_position(sysmsg::INITIAL_POSITION);
        system_window.visible = options.system_msg.shown;
        desktop.windows.push(system_window);
        let mut system_msg = SystemMsgView::new(options.system_msg);
        system_msg.stored = true;
        let (nw, nh) = NPC_VIEWER_SIZE;
        let (nx, ny) = windows_info
            .position(WindowId::NPC_HTML)
            .unwrap_or_else(|| npc_viewer::default_position(viewport));
        let mut npc_window = Window::new(WindowId::NPC_HTML, Rect::new(nx, ny, nw, nh), npc_viewer::grip())
            .with_initial_position(npc_viewer::initial_position(viewport));
        npc_window.visible = false;
        desktop.windows.push(npc_window);
        let (ix, iy) = windows_info
            .position(WindowId::ITEM_HTML)
            .unwrap_or_else(|| npc_viewer::default_position(viewport));
        let mut item_window = Window::new(WindowId::ITEM_HTML, Rect::new(ix, iy, nw, nh), npc_viewer::grip())
            .with_initial_position(npc_viewer::initial_position(viewport));
        item_window.visible = false;
        desktop.windows.push(item_window);
        let (bw, bh) = killer_ui::board::SIZE;
        let (bx, by) = windows_info
            .position(WindowId::BOARD)
            .unwrap_or_else(|| killer_ui::board::default_position(viewport));
        let mut board_window = Window::new(WindowId::BOARD, Rect::new(bx, by, bw, bh), killer_ui::board::grip())
            .with_initial_position(killer_ui::board::INITIAL_POSITION);
        board_window.visible = false;
        desktop.windows.push(board_window);
        crate::ui_tutorial::create_windows(&mut desktop, viewport);
        crate::ui_help::create_window(&mut desktop, &windows_info, viewport);
        let (aw, ah) = ABNORMAL_SIZE;
        let (ax, ay) = windows_info.position(WindowId::ABNORMAL_STATUS).unwrap_or(ABNORMAL_DEFAULT_POS);
        let abnormal_grip = Rect::new(0, 0, ABNORMAL_GRIP.0, ABNORMAL_GRIP.1);
        desktop.windows.push(Window::new(WindowId::ABNORMAL_STATUS, Rect::new(ax, ay, aw, ah), abnormal_grip)
            .with_initial_position(killer_ui::abnormal::ABNORMAL_INITIAL_POSITION));
        let (mw, mh) = MENU_SIZE;
        let (mx, my) = windows_info.position(WindowId::MENU).unwrap_or_else(|| menu::default_position(viewport));
        desktop.windows.push(Window::new(WindowId::MENU, Rect::new(mx, my, mw, mh), menu::grip())
            .with_initial_position(menu::initial_position(viewport)));
        let party_w = PARTY_WIDTH;
        let (party_x, party_y) = windows_info.position(WindowId::PARTY_STATUS).unwrap_or(PARTY_DEFAULT_POS);
        let mut party_window =
            Window::new(WindowId::PARTY_STATUS, Rect::new(party_x, party_y, party_w, 0), party::grip(0))
                .with_initial_position(killer_ui::party::PARTY_INITIAL_POSITION);
        party_window.visible = false;
        desktop.windows.push(party_window);
        let (pw, ph) = PET_STATUS_SIZE;
        for id in [WindowId::PET_STATUS, WindowId::SUMMONED_STATUS] {
            let (px, py) = windows_info.position(id).unwrap_or(PET_STATUS_DEFAULT_POS);
            let mut window = Window::new(id, Rect::new(px, py, pw, ph), pet_status::grip())
                .with_initial_position(killer_ui::pet_status::PET_STATUS_INITIAL_POSITION);
            window.visible = false;
            desktop.windows.push(window);
        }
        let (radar_w, radar_h) = RADAR_SIZE;
        let (radar_x, radar_y) = killer_ui::radar::default_position(viewport);
        desktop.windows.push(Window::new(
            WindowId::RADAR,
            Rect::new(radar_x, radar_y, radar_w, radar_h),
            Rect::new(0, 0, 0, 0),
        ).with_initial_position(killer_ui::radar::initial_position(viewport)));
        let mut zone_window = Window::new(
            WindowId::ZONE_TITLE,
            Rect::new(0, 0, 0, 0),
            Rect::new(0, 0, 0, 0),
        ).with_initial_position(killer_ui::initial_position::DISABLED_INITIAL_POSITION);
        zone_window.visible = false;
        desktop.windows.push(zone_window);
        let (rw, rh) = RESTART_SIZE;
        let (rx, ry) = restart::default_position(viewport);
        let mut restart_window = Window::new(WindowId::RESTART_MENU, Rect::new(rx, ry, rw, rh), Rect::new(0, 0, 0, 0))
            .with_initial_position(restart::RESTART_INITIAL_POSITION);
        restart_window.visible = false;
        desktop.windows.push(restart_window);
        let (dw, dh) = DIALOG_SIZE;
        for (id, bottom) in [(WindowId::DIALOG, false), (WindowId::DIALOG_BOTTOM, true)] {
            let (dx, dy) = windows_info.position(id).unwrap_or_else(|| dialog::default_position(bottom, viewport));
            let mut window = Window::new(id, Rect::new(dx, dy, dw, dh), Rect::new(0, 0, 0, 0))
                .with_initial_position(dialog::initial_position(bottom, viewport));
            window.visible = false;
            desktop.windows.push(window);
        }
        let (warning_x, warning_y) = dialog::warning_position(viewport);
        let mut warning_window = Window::new(WindowId::WARNING_DIALOG, Rect::new(warning_x, warning_y, dw, dh), Rect::new(0, 0, 0, 0))
            .with_initial_position(killer_ui::initial_position::DISABLED_INITIAL_POSITION);
        warning_window.visible = false;
        desktop.windows.push(warning_window);
        let mut keys = KeyWindows::new(&HashMap::new(), options.enter_chatting);
        keys.option.set_font(&small);
        keys.petition.set_font(&small);
        keys.hero_tower.tower.set_font(&small);
        keys.macros.edit.set_font(&small);
        keys.macros.info.set_font(&small);
        keys.quest.set_font(&small);
        keys.gm_quest.set_font(&small);
        keys.line_height = small.line_height as i32;
        keys.ssq.set_font(&small);
        keys.siege.view.date.text_height = small.line_height as i32;
        keys.install(&mut desktop, &windows_info, viewport);
        keys.debug.resize(viewport.0);
        keys.broadcast.set_viewport(viewport);
        keys.quest.screen = viewport;
        keys.quest_info.screen = viewport;
        keys.messenger.set_viewport(viewport);
        for window in &mut desktop.windows {
            window.owner = killer_ui::window_owner::original_owner(window.id);
            window.registration = killer_ui::window_owner::original_registration(window.id);
        }
        desktop.sort_by_creation_order();
        desktop.rearrange(size);
        keys.minimized.sync_owner(&desktop);
        let window_registry = WindowRegistry::from_windows(&desktop.windows)?;
        let alpha = frame_alpha(&options);
        let mut ui = GameUi {
            chat_second: None,
            chat_seen: 0,
            chat_origin: None,
            desktop,
            texture_bank: killer_assets::textures::TextureBank::new(),
            window_registry,
            small,
            large,
            options,
            windows_info,
            alpha,
            status_visible: true,
            chat,
            system_msg,
            option_synced: false,
            audio_volumes: None,
            option_game_runtime: None,
            shortcut,
            npc_viewer: None,
            item_viewer: None,
            board: killer_ui::board::BoardView::new(|_| None),
            help: crate::ui_help::HelpWindow::default(),
            board_command: None,
            board_favorite_pressed: false,
            npc_title: DEFAULT_TITLE.to_string(),
            dialog: None,
            parked_dialog: None,
            button_labels: HashMap::new(),
            warning: killer_ui::dialog::WarningDialog::default(),
            warning_pressed: false,
            warning_exit: false,
            zone_title_font: None,
            true_fonts: Default::default(),
            zone_title: None,
            zone_tracker: Default::default(),
            zone_clock: None,
            html_clock: None,
            loading_texture: None,
            pet_buffs_shown: false,
            party_buffs_shown: false,
            death_answer: None,
            pet_pressed: None,
            party_pressed: None,
            restart_pressed: None,
            keys,
            cursor: (0, 0),
            last_left_press: None,
            target_clicked: false,
            status_clicked: false,
            pet_clicked: false,
            party_cache: PartyView::default(),
            party_clicked: None,
            target_expanded: false,
            target_labels: killer_ui::target::info_labels(|_| None),
            rotate_pressed: false,
            pad_pressed: false,
            status_resize: Default::default(),
            page_pressed: None,
            link_pressed: None,
            menu_pressed: None,
            modern_menu_open: false,
            modern_chat_rows: std::cell::Cell::new([0;2]),
            button_map: std::cell::RefCell::new(Vec::new()),
            close_pressed: None,
            deferred_close: None,
            replaying_close: false,
            button_held: None,
            modern_system_open: false,
            modern_menu_pressed: None,
            tutorial: crate::ui_tutorial::TutorialWindows::default(),
            html_tables: None,
        };
        ui.follow_chat();
        Ok(ui)
    }

    #[cfg(test)]
    pub fn open_npc_html(&mut self, message: &NpcHtmlMessage) {
        self.open_npc_html_with(message, None);
    }

    pub fn open_npc_html_with(&mut self, message: &NpcHtmlMessage, tables: Option<&DatTables>) {
        if message.item_id < 0 {
            return;
        }
        self.keys.sound(killer_ui::interface_sound::WINDOW_OPEN);
        let title = if message.item_id > 0 {
            tables.and_then(|tables| tables.itemname.get(&message.item_id.unsigned_abs())).map(|item| item.name.clone()).unwrap_or_default()
        } else {
            self.npc_title.clone()
        };
        let viewer = NpcViewer::with_markers(&title, &message.html, &self.small, &|marker, id| html_marker(tables, marker, id));
        let preview: String = message.html.chars().take(HTML_PREVIEW_CHARS).collect();
        log::debug!(
            "html do NPC {} ({} chars, {} ações, {} linhas): {preview:?}",
            message.npc_object_id,
            message.html.chars().count(),
            viewer.doc.actions().len(),
            viewer.lines.len()
        );
        let id = if message.item_id > 0 { WindowId::ITEM_HTML } else { WindowId::NPC_HTML };
        *self.html_viewer_slot(id) = Some(viewer);
        if let Some(window) = self.desktop.window_mut(id) {
            window.visible = true;
        }
    }
    fn html_viewer_slot(&mut self, id: WindowId) -> &mut Option<NpcViewer> {
        if id == WindowId::ITEM_HTML { &mut self.item_viewer } else { &mut self.npc_viewer }
    }

    pub fn html_viewer(&self, id: WindowId) -> Option<&NpcViewer> {
        if id == WindowId::ITEM_HTML { self.item_viewer.as_ref() } else { self.npc_viewer.as_ref() }
    }


    pub fn show_board(&mut self, board: Option<(&[String], &str)>) {
        self.close_html_question(HtmlSource::Board);
        use killer_ui::board::BoardWindowEvent;
        let tables = self.html_tables.clone();
        self.board.receive_with(board, &self.small, &mut |event| match event {
            BoardWindowEvent::Show | BoardWindowEvent::Hide => {
                if let Some(window) = self.desktop.window_mut(WindowId::BOARD) {
                    window.visible = event == BoardWindowEvent::Show;
                }
            }
            BoardWindowEvent::Focus => {
                if let Some(index) = self.desktop.windows.iter().position(|w| w.id == WindowId::BOARD) {
                    let window = self.desktop.windows.remove(index);
                    self.desktop.windows.push(window);
                }
            }
        }, &|marker, id| html_marker(tables.as_deref(), marker, id));
    }

    pub fn open_community(&mut self) {
        self.open_board_tab(killer_ui::board::COMMUNITY_TAB, true);
    }

    fn open_board_tab(&mut self, tab: usize, press: bool) {
        if let Some(window) = self.desktop.window_mut(WindowId::BOARD) {
            window.visible = true;
        }
        if let Some(index) = self.desktop.windows.iter().position(|w| w.id == WindowId::BOARD) {
            let window = self.desktop.windows.remove(index);
            self.desktop.windows.push(window);
        }
        let mut action = None;
        let mut host = |command: &str| {
            action = html_action(command);
            html_action_result(command, action.is_some())
        };
        if press { self.board.press_tab(tab, &mut host) } else { self.board.run_tab(tab, &mut host) }
        self.board_command = action;
    }

    pub fn take_status_click(&mut self, world: &World) -> Option<KeyAction> {
        let me = world.me.as_ref()?;
        let origin = me.position.to_array().map(|value| value as i32);
        let shift = self.keys.shift;
        let target = if std::mem::take(&mut self.status_clicked) {
            Some(me.object_id)
        } else if std::mem::take(&mut self.pet_clicked) {
            world.pet.as_ref().map(|pet| pet.info.object_id)
        } else if let Some(index) = self.party_clicked.take() {
            let member = party_view(world, None).members.get(index).map(|member| member.object_id)?;
            let command = if self.keys.ctrl {
                killer_world::NetCommand::AttackRequest { object_id: member, origin, shift }
            } else {
                killer_world::NetCommand::Action { object_id: member, origin, shift }
            };
            self.keys.send(command);
            return Some(KeyAction::Consumed);
        } else {
            None
        }?;
        self.keys.send(killer_world::NetCommand::Action { object_id: target, origin, shift });
        Some(KeyAction::Consumed)
    }

    pub fn take_board_command(&mut self) -> Option<KeyAction> {
        self.board_command.take()
    }

    pub fn board_released(&mut self) -> Option<KeyAction> {
        if !std::mem::take(&mut self.board_favorite_pressed) {
            return None;
        }
        let window = self.desktop.window(WindowId::BOARD).filter(|w| w.visible)?;
        let (x, y) = (self.cursor.0 - window.rect.x, self.cursor.1 - window.rect.y);
        if self.board.hit(x, y, &self.small) != Some(killer_ui::board::BoardHit::Favorite) {
            return None;
        }
        let mut action = None;
        self.board.favorite(&mut |command: &str| {
            action = html_action(command);
            html_action_result(command, action.is_some())
        });
        action
    }

    fn board_link_at(&self, x: i32, y: i32) -> Option<String> {
        let window = self.desktop.window(WindowId::BOARD).filter(|w| w.visible)?;
        match self.board.hit(x - window.rect.x, y - window.rect.y, &self.small)? {
            killer_ui::board::BoardHit::Link(action) => Some(action),
            _ => None,
        }
    }

    fn close_html_question(&mut self, source: HtmlSource) {
        if self.dialog.as_ref().is_some_and(|dialog| matches!(dialog.answer, DialogAnswer::HtmlLink { source: shown, .. } if shown == source)) {
            self.close_dialog();
        }
    }

    fn ask_html_link(&mut self, action: String, message: &str, source: HtmlSource) {
        let text = killer_ui::html::expand_system_msg_code(message, |id| self.html_tables.as_ref().and_then(|tables| tables.systemmsg.get(&id)).map(|message| message.text.clone()));
        let mut view = DialogView::new(DialogKind::OkCancel, &text);
        view.dialog_type = DIALOG_TYPE_HTML_LINK;
        self.show_dialog_view(view, DialogAnswer::HtmlLink { action, source }, WindowId::DIALOG, Instant::now());
    }

    pub fn close_npc_html(&mut self) {
        self.close_html_window(WindowId::NPC_HTML);
    }

    pub fn close_item_html(&mut self) {
        self.close_html_window(WindowId::ITEM_HTML);
    }

    fn close_html_window(&mut self, id: WindowId) {
        *self.html_viewer_slot(id) = None;
        self.link_pressed = None;
        self.close_html_question(if id == WindowId::ITEM_HTML { HtmlSource::Item } else { HtmlSource::Npc });
        if let Some(window) = self.desktop.window_mut(id) {
            window.visible = false;
        }
    }

    pub fn html_confirm(&mut self, tables: Option<&DatTables>, action: &str, now: Instant) {
        let (message, command) = killer_ui::html::split_message(action);
        let Some(message) = message else { return };
        let view = DialogView::new(DialogKind::YesNo, &link_message_text(tables, message));
        self.show_dialog_view(view, DialogAnswer::HtmlLink { action: command.to_string(), source: HtmlSource::Npc }, WindowId::DIALOG, now);
    }

    pub fn open_dialog(&mut self, dialog: &Dialog, world: &World, tables: Option<&DatTables>, now: Instant) {
        let (mut view, answer, window) = dialog_request(dialog, world, tables);
        view.timer_elapsed_ms = (!matches!(dialog, Dialog::AllianceWar { start: false, .. })).then_some(0);
        self.show_dialog_view(view, answer, window, now);
    }

    fn show_dialog_view(&mut self, mut view: DialogView, answer: DialogAnswer, window: WindowId, now: Instant) {
        let (first, second) = view.kind.sysstrings();
        if let (Some(a), Some(b)) = (self.button_labels.get(&first), self.button_labels.get(&second)) {
            view.labels = (a.clone(), b.clone());
        }
        let width = view.width();
        if let Some(previous) = self.dialog.take()
            && previous.window != window
        {
            self.parked_dialog = Some(previous);
        }
        self.dialog = Some(ActiveDialog { view, answer, window, opened: now, pressed: None });
        let parked = self.parked_dialog.as_ref().map(|dialog| dialog.window);
        for id in [WindowId::DIALOG, WindowId::DIALOG_BOTTOM] {
            if let Some(w) = self.desktop.window_mut(id) {
                w.visible = id == window || Some(id) == parked;
                if id == window {
                    w.rect.w = width;
                }
            }
        }
        if let Some(index) = self.desktop.windows.iter().position(|w| w.id == window) {
            let window = self.desktop.windows.remove(index);
            self.desktop.windows.push(window);
        }
    }

    pub fn show_warning(&mut self, text: &str, exit: bool) {
        self.warning.show(Some(text), exit);
        self.warning_pressed = false;
        if let Some(window) = self.desktop.window_mut(WindowId::WARNING_DIALOG) {
            window.visible = true;
        }
        if let Some(index) = self.desktop.windows.iter().position(|w| w.id == WindowId::WARNING_DIALOG) {
            let window = self.desktop.windows.remove(index);
            self.desktop.windows.push(window);
        }
    }

    fn warning_local(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let window = self.desktop.window(WindowId::WARNING_DIALOG).filter(|window| window.visible && self.warning.shown)?;
        Some((x - window.rect.x, y - window.rect.y))
    }

    pub fn warning_released(&mut self) -> Option<KeyAction> {
        if !self.warning.shown {
            return None;
        }
        let pressed = std::mem::take(&mut self.warning_pressed);
        let (lx, ly) = self.warning_local(self.cursor.0, self.cursor.1)?;
        if pressed && dialog::WarningDialog::hit(lx, ly) {
            match self.warning.ok() {
                dialog::WarningOutcome::Exit => self.warning_exit = true,
                dialog::WarningOutcome::Hide => {
                    if let Some(window) = self.desktop.window_mut(WindowId::WARNING_DIALOG) {
                        window.visible = false;
                    }
                }
            }
        }
        Some(KeyAction::Consumed)
    }

    pub fn close_dialog(&mut self) -> Option<ActiveDialog> {
        let closed = self.dialog.take();
        if let Some(window) = closed.as_ref().map(|dialog| dialog.window)
            && let Some(w) = self.desktop.window_mut(window)
        {
            w.visible = false;
        }
        self.dialog = self.parked_dialog.take();
        closed
    }

    pub fn dialog_released(&mut self) -> Option<KeyAction> {
        let dialog = self.dialog.as_mut()?;
        let pressed = dialog.pressed.take()?;
        let window = self.desktop.window(dialog.window)?;
        let released = dialog.view.button_at(self.cursor.0 - window.rect.x, self.cursor.1 - window.rect.y)?;
        if released != pressed {
            return None;
        }
        if let DialogButton::Key(key) = pressed {
            if let Some(input) = dialog.view.input.as_mut() {
                input.key(key);
            }
            return Some(KeyAction::Consumed);
        }
        Some(self.finish_dialog(pressed == DialogButton::First))
    }

    fn finish_dialog(&mut self, accept: bool) -> KeyAction {
        if accept
            && let Some(dialog) = self.dialog.as_mut()
            && dialog.answer == DialogAnswer::PetName
        {
            let name = dialog.view.input.as_ref().map(|input| input.text.clone()).unwrap_or_default();
            if name.is_empty() || name.chars().any(|c| !self.small.has_glyph(c)) {
                if let Some(input) = dialog.view.input.as_mut() { input.text.clear(); }
                self.keys.pending_message(PET_NAME_INVALID_SYSMSG, PET_NAME_MESSAGE_COLOR);
                return KeyAction::Consumed;
            }
            if name.chars().count() > PET_NAME_MAX_BYTES {
                self.keys.pending_message(PET_NAME_TOO_LONG_SYSMSG, PET_NAME_MESSAGE_COLOR);
                return KeyAction::Consumed;
            }
            self.close_dialog();
            self.keys.send(killer_world::NetCommand::ChangePetName { name });
            return KeyAction::Consumed;
        }
        if accept
            && let Some(dialog) = self.dialog.as_mut()
            && let DialogAnswer::GiveTitle { target } = dialog.answer.clone()
        {
            let title = dialog.view.input.as_ref().map(|input| input.text.clone()).unwrap_or_default();
            if self.keys.target_player.as_deref() != Some(target.as_str()) {
                self.close_dialog();
                return KeyAction::Consumed;
            }
            if title.is_empty() {
                if let Some(input) = dialog.view.input.as_mut() { input.text.clear(); }
                self.keys.pending_message(TITLE_INVALID_SYSMSG, TITLE_MESSAGE_COLOR);
                return KeyAction::Consumed;
            }
            if title.chars().count() > TITLE_MAX_BYTES {
                self.keys.pending_message(TITLE_LENGTH_SYSMSG, TITLE_MESSAGE_COLOR);
                return KeyAction::Consumed;
            }
            self.close_dialog();
            self.keys.send(killer_world::NetCommand::GiveNickName { target, title });
            return KeyAction::Consumed;
        }
        let Some(dialog) = self.close_dialog() else { return KeyAction::Consumed };
        if let DialogAnswer::HtmlLink { action, .. } = &dialog.answer {
            if !accept {
                return KeyAction::Consumed;
            }
            if html_action_result(action, html_action(action).is_some()) == 2 {
                self.close_npc_html();
            }
            return html_action(action).unwrap_or(KeyAction::Consumed);
        }
        if dialog.answer == DialogAnswer::AllyDismissName {
            let pledge_name = dialog.view.input.as_ref().map(|input| input.text.clone()).unwrap_or_default();
            if accept && !pledge_name.is_empty() {
                self.keys.dialogs.push(Dialog::AllyDismissConfirm { pledge_name });
            }
            return KeyAction::Consumed;
        }
        if let DialogAnswer::CrestNotice { large } = dialog.answer {
            if accept {
                self.keys.dialogs.push(Dialog::CrestFile { large });
            }
            return KeyAction::Consumed;
        }
        if let DialogAnswer::CrestFile { large } = dialog.answer {
            if accept {
                let path = dialog.view.input.as_ref().map(|input| input.text.clone()).unwrap_or_default();
                let kind = if large { killer_assets::crest::CrestKind::Large } else { killer_assets::crest::CrestKind::Clan };
                self.keys.push_command(crate::ui_keys::UiCommand::CrestFile { kind, path });
            }
            return KeyAction::Consumed;
        }
        if let DialogAnswer::PledgeWarName { start } = dialog.answer {
            if accept {
                let pledge_name = dialog.view.input.as_ref().map(|input| input.text.clone()).unwrap_or_default();
                self.keys.send(if start { killer_world::NetCommand::StartPledgeWar { pledge_name } } else { killer_world::NetCommand::StopPledgeWar { pledge_name } });
            }
            return KeyAction::Consumed;
        }
        if matches!(dialog.answer, DialogAnswer::Siege(killer_world::state::SiegeDecision::SetTime { .. })) {
            if accept && let Some(command) = self.keys.siege.confirm_time() { self.keys.send(command); }
            KeyAction::Consumed
        } else if let DialogAnswer::HtmlLink { action, source } = &dialog.answer {
            if !accept {
                return KeyAction::Consumed;
            }
            match source {
                HtmlSource::Tutorial => {
                    if let Some(command) = crate::ui_tutorial::TutorialWindows::link_command(action) {
                        self.keys.send(command);
                    }
                    KeyAction::Consumed
                }
                HtmlSource::Npc | HtmlSource::Item | HtmlSource::Board => {
                    if html_action_result(action, html_action(action).is_some()) == 2 {
                        match source {
                            HtmlSource::Npc => self.close_npc_html(),
                            HtmlSource::Item => self.close_item_html(),
                            _ => {}
                        }
                    }
                    html_action(action).unwrap_or(KeyAction::Consumed)
                }
            }
        } else if dialog.answer == DialogAnswer::Shop {
            let text = dialog.view.input.as_ref().filter(|_| accept).map(|input| input.text.as_str());
            self.keys.shop.answer_text(text);
            KeyAction::Consumed
        } else if dialog.answer == DialogAnswer::RecipeShop {
            let text = dialog.view.input.as_ref().filter(|_| accept).map(|input| input.text.as_str());
            self.keys.recipes.shop.answer_text(text);
            KeyAction::Consumed
        } else if dialog.answer == DialogAnswer::ShopPreview {
            if let Some(command) = self.keys.shop_preview.answer_confirm(accept, &mut self.desktop) {
                self.keys.send(command);
            }
            KeyAction::Consumed
        } else if dialog.answer == DialogAnswer::PrivateShop {
            let text = dialog.view.input.as_ref().filter(|_| accept).map(|input| input.text.as_str());
            self.keys.answer_private_shop(accept, text, &mut self.desktop);
            KeyAction::Consumed
        } else if dialog.answer == DialogAnswer::Delivery {
            let text = dialog.view.input.as_ref().filter(|_| accept).map(|input| input.text.as_str());
            self.keys.delivery.answer_text(text);
            KeyAction::Consumed
        } else if dialog.answer == DialogAnswer::Warehouse {
            let text = dialog.view.input.as_ref().filter(|_| accept).map(|input| input.text.as_str());
            self.keys.warehouse.answer_text(text);
            KeyAction::Consumed
        } else if dialog.answer == DialogAnswer::GmStore {
            let text = dialog.view.input.as_ref().filter(|_| accept).map(|input| input.text.as_str());
            self.keys.gm_store.answer_text(text);
            KeyAction::Consumed
        } else if dialog.answer == DialogAnswer::ManorShop {
            let text = dialog.view.input.as_ref().filter(|_| accept).map(|input| input.text.as_str());
            self.keys.manor_shop.answer_text(text);
            KeyAction::Consumed
        } else if dialog.answer == DialogAnswer::TradeCount {
            let text = dialog.view.input.as_ref().filter(|_| accept).map(|input| input.text.as_str());
            self.keys.trade.answer_text(text);
            KeyAction::Consumed
        } else if let DialogAnswer::DropItem { object_id, position, kind, .. } = dialog.answer
            && let Some(input) = dialog.view.input.as_ref()
        {
            match input.value(false).and_then(|value| i32::try_from(value).ok()).filter(|value| accept && *value > 0) {
                Some(count) => KeyAction::Dialog(DialogAnswer::DropItem { object_id, count, position, kind }, true),
                None => KeyAction::Consumed,
            }
        } else if dialog.answer == DialogAnswer::MultiSellCount {
            let text = dialog.view.input.as_ref().filter(|_| accept).map(|input| input.text.as_str());
            self.keys.multisell.answer_text(text);
            KeyAction::Consumed
        } else if dialog.answer == DialogAnswer::ManorSetting {
            self.keys.manor_setting.answer(accept);
            KeyAction::Consumed
        } else {
            KeyAction::Dialog(dialog.answer, accept)
        }
    }

    pub fn dialog_timed_out(&mut self, now: Instant) -> Option<KeyAction> {
        let expired = |dialog: &ActiveDialog| dialog.view.timer_elapsed_ms.is_some() && dialog.elapsed_ms(now) >= dialog::TIMER_TOTAL_MS;
        if self.parked_dialog.as_ref().is_some_and(expired) {
            let dialog = self.parked_dialog.take()?;
            if let Some(w) = self.desktop.window_mut(dialog.window) {
                w.visible = false;
            }
            return Some(KeyAction::Dialog(dialog.answer, false));
        }
        if !self.dialog.as_ref().is_some_and(expired) {
            return None;
        }
        let dialog = self.close_dialog()?;
        Some(KeyAction::Dialog(dialog.answer, false))
    }

    fn html_link_message_at(&self, id: WindowId, x: i32, y: i32) -> Option<String> {
        let window = self.desktop.window(id).filter(|w| w.visible)?;
        let viewer = self.html_viewer(id)?;
        viewer.link_message_at(x - window.rect.x, y - window.rect.y).map(str::to_string)
    }

    fn html_link_at(&self, id: WindowId, x: i32, y: i32) -> Option<String> {
        let window = self.desktop.window(id).filter(|w| w.visible)?;
        let viewer = self.html_viewer(id)?;
        viewer.link_at(x - window.rect.x, y - window.rect.y, &self.small).map(str::to_string)
    }

    pub fn resize_cursor(&self) -> Option<u8> {
        if self.status_resize.window().is_some() {
            return Some(killer_ui::resize_frame::CURSOR_HORIZONTAL as u8);
        }
        let (x, y) = self.cursor;
        let (id, _, _) = self.window_under(x, y)?;
        let window = self.desktop.window(id)?;
        crate::ui_status_resize::hover_cursor(window, x, y).map(|index| index as u8)
    }

    pub fn toggle_target_info(&mut self) {
        self.target_expanded = !self.target_expanded;
        let height = killer_ui::target::target_height(self.target_expanded);
        if let Some(window) = self.desktop.window_mut(WindowId::TARGET) {
            window.rect.h = height;
            window.grip = FrameCtrl::new(FrameMode::VerticalGrip).grip(window.rect.w, height);
        }
    }

    pub fn system_menu_restart(&mut self) {
        self.keys.system_menu_restart(&mut self.desktop);
    }

    pub fn show_pet_window(&mut self, views: &WorldViews) {
        self.party_cache = views.party.clone();
        let shown = views.pet.as_ref().map(|(id, _, _)| *id);
        for id in [WindowId::PET_STATUS, WindowId::SUMMONED_STATUS] {
            if let Some(window) = self.desktop.window_mut(id) {
                window.visible = shown == Some(id);
            }
        }
        if let Some(window) = self.desktop.window_mut(WindowId::RESTART_MENU) {
            window.visible = views.restart.is_some();
        }
        if let Some(window) = self.desktop.window_mut(WindowId::PARTY_STATUS) {
            let height = views.party.height();
            window.visible = height > 0;
            window.rect.h = height;
            window.grip = party::grip(height);
        }
        if let Some(window) = self.desktop.window_mut(WindowId::ABNORMAL_STATUS) {
            let modern=self.shortcut.modern.is_some();
            let (w, h) = if modern {crate::ui_modern_buffs::size(&views.abnormal)}else{views.abnormal.size()};
            window.visible = !views.abnormal.is_empty();
            window.rect.w = w;
            window.rect.h = h;
            window.grip = if modern {Rect::new(0,0,12,h)}else{views.abnormal.grip()};
        }
    }

    pub fn loading_frame(&self) -> DrawList {
        let mut list = DrawList::default();
        if let Some(texture) = &self.loading_texture {
            let (w, h) = self.desktop.size;
            paint_loading(&mut list, 0, 0, w, h, texture);
        }
        list
    }

    pub fn show_zone_title(&mut self, zone: &str, now: Instant) {
        if !self.options.show_zone_title {
            return;
        }
        self.zone_title = Some((ZoneTitleView::shown(zone), now));
        if let Some(window) = self.desktop.window_mut(WindowId::ZONE_TITLE) {
            window.visible = false;
        }
        self.tick_zone_title(now);
    }

    pub fn take_zone_texture_requests(&mut self) -> Vec<(i32, i32)> {
        self.keys.zones.take_requests()
    }

    pub fn set_zone_texture(&mut self, cell: (i32, i32), texture: Option<zone_title::ZoneTexture>) {
        self.keys.zones.set_texture(cell, texture);
    }

    pub fn update_zone(&mut self, tables: &DatTables, position: (f32, f32, f32), now: Instant) {
        let delta = self.zone_clock.map_or(0.0, |last| now.saturating_duration_since(last).as_secs_f32());
        self.zone_clock = Some(now);
        self.keys.zones.ensure(&tables.zonename);
        let found = self.keys.zones.lookup(position).cloned();
        if let Some(title) = self.zone_tracker.tick(delta, found.as_ref()) {
            self.show_zone_title(&title, now);
        }
        self.keys.minimap.zone = self.zone_tracker.current.clone().unwrap_or_default();
        self.keys.minimap.zone_id = self.zone_tracker.current_id.map_or(-1, |id| id as i32);
    }

    pub fn reset_zone(&mut self) {
        self.zone_tracker.reset();
    }

    pub fn tick_command_channel(&mut self, now: Instant) {
        self.keys.minimized.sync_owner(&self.desktop);
        self.keys.tick_command_channel(now);
    }

    pub fn tick_html_forms(&mut self, now: Instant) {
        let (last, ms) = self.html_clock.unwrap_or((now, 0));
        let elapsed = now.saturating_duration_since(last).as_millis().min(u128::from(u32::MAX)) as u32;
        let ms = ms.saturating_add(elapsed);
        self.html_clock = Some((now, ms));
        self.board.html_form.tick(elapsed);
        self.board.tick(elapsed);
        if let Some(viewer) = self.npc_viewer.as_mut() {
            viewer.form.tick(elapsed);
            viewer.set_clock_ms(ms);
        }
    }

    pub fn tick_edit_boxes(&mut self, now: Instant) {
        self.tick_html_forms(now);
        self.keys.multisell.tick(now);
        self.keys.manor_setting.tick(now);
        self.keys.hero_tower.tick(now);
        self.keys.macros.tick(now);
        self.keys.petition.tick(now);
        self.keys.tick_calc(now);
        self.keys.tick_broadcast(now, &mut self.desktop);
        self.keys.tick_messenger(now, &mut self.desktop);
    }

    pub fn shift_changed(&mut self, pressed: bool) -> bool {
        let seed = self.keys.manor_setting.shift(pressed);
        let hero = self.keys.hero_tower.shift(pressed);
        let macros = self.keys.macros.edit.shift(pressed);
        self.keys.multisell.view.quantity_shift(pressed) || seed || hero || macros
    }

    pub fn tick_tutorial(&mut self, now: Instant) {
        self.tutorial.tick(now, &mut self.desktop);
    }

    pub fn tick_zone_title(&mut self, now: Instant) {
        let screen = self.desktop.size;
        if let Some((view, last)) = self.zone_title.as_mut()
            && let Some(window) = self.desktop.window_mut(WindowId::ZONE_TITLE)
        {
            let delta = now.saturating_duration_since(*last).as_secs_f32();
            *last = now;
            let zone_font: &dyn killer_ui::true_font::LineFont = self.true_fonts.for_type(zone_title::TRUETYPE_FONT_TYPE).map_or(&self.large, |font| font);
            zone_title::tick_zone_title(&mut zone_title::ZoneTitleFrame { view, window, font: zone_font, screen }, delta);
            if view.lines.iter().all(String::is_empty) {
                self.zone_title = None;
            }
        }
    }

    fn start_tab_drag(&mut self, tab: usize, press: (i32, i32), cursor: (i32, i32)) {
        let Some(chat) = self.desktop.window(WindowId::CHAT).map(|w| w.rect) else { return };
        let (width, height) = self.chat.split_size();
        let rect = Rect::new(chat.x + cursor.0 - press.0, chat.y + cursor.1 - press.1, width, height);
        let id = WindowId(ChatView::split_window_id(tab));
        self.chat.detach(tab);
        match self.desktop.window_mut(id) {
            Some(window) => {
                window.rect = rect;
                window.grip = self.chat.split_grip(tab);
                window.visible = true;
            }
            None => self.desktop.windows.push(Window::new(id, rect, self.chat.split_grip(tab))),
        }
        self.desktop.bring_to_front(id);
        self.desktop.begin_drag(cursor.0, cursor.1);
    }

    fn merge_dragged_tab(&mut self, dragged: Option<WindowId>) {
        let Some(tab) = dragged.and_then(|id| ChatView::split_tab_of(id.0)) else { return };
        let (Some(window), Some(chat)) = (
            self.desktop.window(WindowId(ChatView::split_window_id(tab))).map(|w| w.rect),
            self.desktop.window(WindowId::CHAT).map(|w| w.rect),
        ) else {
            return;
        };
        let grip = self.chat.split_grip(tab);
        let moved = grip.offset(window.x, window.y);
        let saved = Rect::new(chat.x + killer_ui::chat::TAB_X[tab], chat.y + chat.h - killer_ui::chat::CHAT_FOOTER, grip.w, grip.h);
        if ChatView::merge_touches(moved, saved) {
            if let Some(window) = self.desktop.window_mut(WindowId(ChatView::split_window_id(tab))) {
                window.visible = false;
            }
            self.chat.attach(tab);
        }
    }

    pub fn follow_chat(&mut self) {
        let Some(chat) = self.desktop.window(WindowId::CHAT).map(|w| w.rect) else { return };
        let previous = self.chat_origin.replace((chat.x, chat.y));
        if let Some((px, py)) = previous.filter(|origin| *origin != (chat.x, chat.y)) {
            for tab in 1..5 {
                if let Some(window) = self.desktop.window_mut(WindowId(ChatView::split_window_id(tab))) {
                    window.rect.x += chat.x - px;
                    window.rect.y += chat.y - py;
                }
            }
        }
        let (sx, sy) = sysmsg::position_over_chat(chat);
        if let Some(window) = self.desktop.window_mut(WindowId::SYSTEM_MSG) {
            window.rect.x = sx;
            window.rect.y = sy;
        }
    }

    fn paint_settings(&self, list: &mut DrawList, rect: Rect) {
        use killer_ui::canvas_clip::CanvasClip;
        use killer_ui::option_paint::{OptionDrawBackend, OptionPaintWindow, paint_option};
        use killer_ui::ui_device::{TextureResources, UiDevice};

        let option = &self.keys.option;
        option.paint_with_body(list, (rect.x, rect.y), &self.small, self.alpha, |list| {
            let window = OptionPaintWindow {
                position: [rect.x, rect.y],
                size: [rect.w as f32, rect.h as f32],
                header: Some(killer_ui::option::HEADER),
                background: Some(option.background()),
            };
            let mut device = UiDevice { origin: [0; 2], minimum_y: 0, maximum_y: 0, transparency_state: 0 };
            let mut canvas = CanvasClip {
                flags: 0, origin: [0.0; 2], size: [0.0; 2], cursor: [0.0; 2],
                saved_cursor: [0.0; 2], saved_origin: [0.0; 2], saved_size: [0.0; 2],
            };
            let mut base_x = 0;
            let mut backend = OptionDrawBackend {
                window: &window, device: &mut device, canvas: &mut canvas, base_x: &mut base_x,
                depth: 0.0, list,
                resources: |path: &str| {
                    (self.texture_bank.texture_available(path) == Some(true))
                        .then_some(TextureResources { primary: true, secondary: false })
                },
            };
            match paint_option(&mut backend) {
                Ok(_) => {}
                Err(never) => match never {},
            }
        });
    }

    pub fn frame(
        &self,
        world: &World,
        labels: &[Label],
        chat: &[ChatEntry],
        target: Option<&TargetView>,
        views: &WorldViews,
    ) -> DrawList {
        let mut list = DrawList::default();
        self.paint_labels(&mut list, labels);
        if self.shortcut.modern.is_none()
            && let (Some(me), false) = (world.me.as_ref(), views.gauges.is_empty())
            && let Some(label) = labels.iter().find(|l| l.name == me.name)
        {
            let focal = (self.desktop.size.0 as f32 / 2.0) / (crate::camera::ENGINE_DEFAULT_FOV_X_DEG.to_radians() / 2.0).tan();
            paint_gauges(&mut list, label.screen[0].round() as i32, label.screen[1].round() as i32, killer_ui::gauge::stack_step(focal), &views.gauges);
        }
        for mark in &views.primitives {
            paint_primitive_mark(&mut list, mark, &self.small);
        }
        if let Some(item) = &views.item_label {
            paint_item_label(&mut list, item, &self.small);
        }
        if self.keys.hidden {
            return list;
        }
        let mut button_map = self.button_map.borrow_mut();
        button_map.clear();
        let style = |id: WindowId| killer_ui::window_owner::original_registration(id).map(|registration| registration.flags_argument);
        let front = self.desktop.windows.iter().rev().find(|w| w.visible && style(w.id).is_some_and(|flags| flags & 0x800000 != 0)).or_else(|| self.desktop.windows.iter().rev().find(|w| w.visible)).map(|w| w.id);
        for window in &self.desktop.windows {
            if window.id == WindowId::PARTY_MATCH && self.shortcut.modern.is_none() {
                self.keys.minimized.paint(&mut list, &self.small, self.desktop.size.0);
            }
            if !window.visible { continue; }
            if self.shortcut.modern.is_some() && (matches!(window.id, WindowId::STATUS | WindowId::CHAT | WindowId::SYSTEM_MSG | WindowId::SHORTCUT) || crate::ui_modern_windows::supports(window.id)) { continue; }
            let first = list.buttons.len();
            list.transparent = self.options.transparency && front != Some(window.id) && style(window.id).is_some_and(|flags| flags & 0x80000 == 0);
            self.paint_window(&mut list,window,world,chat,target,views);
            list.transparent = false;
            button_map.extend(list.buttons[first..].iter().map(|button| (window.id, *button)));
        }
        drop(button_map);
        if self.shortcut.modern.is_none() && let Some(window) = self.desktop.window(WindowId::CHAT).filter(|w| w.visible) {
            self.chat.paint_filter(&mut list, window.rect.x, window.rect.y, &self.small);
        }
        let party_tip = self.desktop.window(WindowId::MAP_COVER).filter(|window| window.visible).and_then(|_| {
            let (name, (lx, ly)) = self.keys.minimap.party_tooltip()?;
            let tip = killer_ui::tooltip::Tooltip { name, compact: true, anchor: Some((WindowId::MAP_COVER, (lx, ly))), ..Default::default() };
            Some(tip)
        });
        if let Some(tooltip) = self.keys.tooltip.as_ref().or(party_tip.as_ref())
            && self.shortcut.modern.is_none() {
            let slot = tooltip.anchor.and_then(|(id, (lx, ly))| self.desktop.window(id).map(|window| (window.rect.x + lx, window.rect.y + ly)));
            let size = tooltip.size(&self.small);
            let (x, y) = match slot {
                Some(slot) => tooltip::place_slot(slot, size, self.desktop.size, tooltip.compact),
                None => tooltip::place(self.cursor, size, self.desktop.size),
            };
            tooltip.paint(&mut list, x, y, &self.small);
        }
        if let Some((hx, hy)) = self.button_held {
            let (cx, cy) = self.cursor;
            let held: Vec<usize> = self.button_map.borrow().iter()
                .filter(|(_, button)| button.contains(hx, hy) && button.contains(cx, cy))
                .map(|(_, button)| button.quad)
                .collect();
            for quad in held {
                let Some(face) = list.quads.get(quad).and_then(|q| killer_ui::button_pushed::pushed_face(q.texture.as_str())) else { continue };
                let face = list.intern(face);
                list.quads[quad].texture = face;
            }
        } else {
            let (cx, cy) = self.cursor;
            let hovered = self.window_under(cx, cy).and_then(|(under, _, _)| killer_ui::button_over::hovered_quad(&self.button_map.borrow(), under, cx, cy));
            if let Some(quad) = hovered
                && let Some(face) = list.quads.get(quad).and_then(|q| killer_ui::button_over::over_face(q.texture.as_str()))
            {
                let face = list.intern(&face);
                list.quads[quad].texture = face;
            }
        }
        list
    }

    pub fn paint_window(&self, list: &mut DrawList, window: &killer_ui::window::Window, world: &World, chat: &[ChatEntry], target: Option<&TargetView>, views: &WorldViews) {
            let (x, y) = (window.rect.x, window.rect.y);
            match window.id {
                WindowId::STATUS => {
                    if let Some(me) = world.me.as_ref().filter(|_| self.status_visible) {
                        paint_status_sized(list, x, y, window.rect.w, &StatusView { regen: world.regen_fill.amount, ..status_view(me) }, &self.small, self.alpha);
                    }
                }
                WindowId::TARGET => {
                    if let Some(view) = target {
                        let view = TargetView { expanded: self.target_expanded, labels: self.target_labels.clone(), ..view.clone() };
                        paint_target_sized(list, x, y, window.rect.w, &view, &self.small, self.alpha);
                    }
                }
                WindowId::SHORTCUT => self.shortcut.paint(list, x, y, &self.small, self.alpha),
                WindowId::CHAT => self.chat.paint(list, x, y, self.chat.active_list(), &self.small, self.alpha),
                WindowId::SYSTEM_MSG => self.system_msg.paint(list, x, y, chat, &self.small, self.alpha),
                id if ChatView::split_tab_of(id.0).is_some() => {
                    if let Some(tab) = ChatView::split_tab_of(id.0) {
                        self.chat.paint_split(list, x, y, tab, &self.small, self.alpha);
                    }
                }
                WindowId::NPC_HTML | WindowId::ITEM_HTML => {
                    if let Some(viewer) = self.html_viewer(window.id) {
                        viewer.paint(list, x, y, &self.small, self.alpha);
                    }
                }
                WindowId::BOARD => {
                    let pressed = self.board_favorite_pressed
                        && self.board.hit(self.cursor.0 - x, self.cursor.1 - y, &self.small) == Some(killer_ui::board::BoardHit::Favorite);
                    self.board.paint(list, x, y, pressed, &self.small, self.alpha);
                }
                WindowId::RADAR => list.radar_at = Some(list.quads.len()),
                WindowId::HELP => self.help.paint(list, x, y, &self.small, self.alpha),
                WindowId::TUTORIAL_VIEWER => self.tutorial.viewer.paint(list, x, y, &self.small, self.alpha),
                WindowId::TUTORIAL_BUTTON => {
                    let hovered = window.rect.contains(self.cursor.0, self.cursor.1);
                    self.tutorial.button.paint(list, x, y, self.tutorial.button_pressed && hovered, hovered, self.alpha);
                }
                WindowId::MAIL_BUTTON => {
                    let hovered = window.rect.contains(self.cursor.0, self.cursor.1);
                    self.tutorial.mail_button.paint_mail(list, x, y, self.tutorial.mail_pressed && hovered, hovered, self.alpha);
                }
                WindowId::QUEST_BUTTON => {
                    let hovered = window.rect.contains(self.cursor.0, self.cursor.1);
                    self.tutorial.quest_button.paint_quest(list, x, y, self.tutorial.quest_pressed && hovered, hovered, self.alpha);
                }
                WindowId::ABNORMAL_STATUS => paint_abnormal(list, x, y, &views.abnormal, &self.small, self.alpha),
                WindowId::MENU => paint_menu(list, x, y, self.menu_pressed, &self.small, self.alpha),
                WindowId::DIALOG | WindowId::DIALOG_BOTTOM => {
                    if let Some(dialog) = [self.dialog.as_ref(), self.parked_dialog.as_ref()].into_iter().flatten().find(|d| d.window == window.id) {
                        paint_dialog(list, x, y, &dialog.view, dialog.pressed, &self.small);
                    }
                }
                WindowId::WARNING_DIALOG if self.warning.shown => {
                    let label = self.button_labels.get(&dialog::OK_SYSSTRING).map_or("OK", String::as_str);
                    self.warning.paint(list, x, y, label, self.warning_pressed, &self.small);
                }
                WindowId::ZONE_TITLE => {
                    if let Some((view, _)) = &self.zone_title {
                        paint_zone_title(list, x, y, view, self.true_fonts.for_type(zone_title::TRUETYPE_FONT_TYPE).map_or(&self.large as &dyn killer_ui::true_font::LineFont, |font| font));
                    }
                }
                WindowId::RESTART_MENU => {
                    if let Some(view) = &views.restart {
                        paint_restart(list, x, y, view, self.restart_pressed, &self.small);
                    }
                }
                WindowId::PARTY_STATUS => {
                    let view = PartyView { show_buffs: self.party_buffs_shown, ..views.party.clone() };
                    paint_party(list, x, y, window.rect.w, &view, &self.small, self.alpha);
                }
                WindowId::PET_STATUS | WindowId::SUMMONED_STATUS => {
                    if let Some((_, _, view)) = views.pet.as_ref().filter(|(id, _, _)| *id == window.id) {
                        let view = PetStatusView { show_buffs: self.pet_buffs_shown, ..view.clone() };
                        paint_pet_status_sized(list, x, y, window.rect.w, &view, &self.small, self.alpha);
                    }
                }
                WindowId::OPTION => self.paint_settings(list, window.rect),
                WindowId::MAP_COVER => {
                    self.keys.minimap.yaw.set(views.radar.camera_yaw);
                    self.keys.paint(window.id, list, x, y, world, &self.small, self.alpha);
                }
                id if self.keys.handles(id) => self.keys.paint(id, list, x, y, world, &self.small, self.alpha),
                _ => {}
            }
    }

    fn npc_form_key(&mut self, id: WindowId, key: InputKey) -> KeyAction {
        let Some(viewer) = (if id == WindowId::ITEM_HTML { self.item_viewer.as_mut() } else { self.npc_viewer.as_mut() }) else { return KeyAction::Consumed };
        match key {
            InputKey::Escape => viewer.form.focus = None,
            InputKey::Enter => {
                let Some(action) = viewer.form.focused_action().map(str::to_string) else { return KeyAction::Consumed };
                let (action, messages) = killer_ui::html_form::expand_action(&action, &viewer.form, &self.small);
                for id in messages {
                    self.keys.pending_message(id, killer_ui::html_form::MESSAGE_COLOR);
                }
                if action.is_empty() {
                    return KeyAction::Consumed;
                }
                if html_action_result(&action, html_action(&action).is_some()) == 2 {
                    self.close_html_window(id);
                }
                return html_action(&action).unwrap_or(KeyAction::Consumed);
            }
            other => {
                viewer.form.key(other, &self.small);
            }
        }
        KeyAction::Consumed
    }

    pub fn ingest_chat(&mut self, log: &killer_world::chat::ChatLog) {
        let pushed = log.pushed();
        if pushed < self.chat_seen {
            self.chat.clear_lists();
            self.system_msg.list.clear();
            self.chat_seen = 0;
        }
        let fresh = (pushed - self.chat_seen).min(log.len() as u64) as usize;
        for line in log.iter().skip(log.len() - fresh) {
            let entry = chat_entry(line);
            let shown = self.chat.shows_in(self.chat.active_tab, &entry);
            self.chat.capture.message(&entry.line(), shown);
            self.chat.ingest(&entry);
            self.system_msg.ingest(&entry);
        }
        self.chat_seen = pushed;
    }

    pub fn tick_text_capture(&mut self, stamp: killer_ui::text_capture::Stamp) -> Option<(String, Vec<u8>)> {
        if let Some(text) = self.chat.capture.update(stamp) {
            let mut entry = ChatEntry::new(killer_world::chat::CHAT_SYSTEM, "", text);
            entry.color = Some(Argb(killer_ui::text_capture::CAPTURE_COLOR));
            let tab = self.chat.active_tab.min(self.chat.lists.len() - 1);
            self.chat.lists[tab].push(entry);
        }
        self.chat.capture.take_write()
    }

    pub fn chat_line(&mut self, line: &str, world: &World) -> Vec<killer_ui::chat_line::LineEffect> {
        let context = killer_ui::chat_line::LineContext { tab: self.chat.active_tab, old_chat: self.chat.old_chat, builder_level: world.builder_level };
        let expand = |word: &str| crate::ui_keys::expand_name_token(world, word);
        killer_ui::chat_line::process_line(line, &context, &mut self.chat.limiter, &mut self.chat.tells, &expand)
    }

    pub fn tick_chat_limiter(&mut self, now: killer_platform::time::Instant) {
        let due = self.chat_second.is_none_or(|last| now.saturating_duration_since(last).as_secs_f32() >= 1.0);
        if !due {
            return;
        }
        self.chat_second = Some(now);
        if self.chat.limiter.tick() {
            self.keys.chat_system(killer_ui::chat_line::FLOOD_RELEASED_MESSAGE);
        }
    }

    pub fn sync_windows(&mut self, world: &World, tables: Option<&DatTables>, now: killer_platform::time::Instant) {
        self.tick_chat_limiter(now);
        self.sync_fishing(world, tables);
        if let Some(runtime) = &self.option_game_runtime {
            self.keys.party_looting = runtime.console_looting;
        }
        self.keys.sync(world, tables, &mut self.shortcut, now);
        if let (Some(tables), Some(me)) = (tables, world.me.as_ref()) {
            self.update_zone(tables, (me.position.x, me.position.y, me.position.z), now);
        }
        self.shortcut.auto.carried = self.keys.dragged_item_id();
        self.keys.sync_gm(tables);
        let timed = self.keys.take_gm_event_actions();
        self.gm_event_actions(timed, world);
        self.keys.recipes.sync(world, tables, &mut self.desktop);
        self.keys.recipes.tree.measure(&self.small);
        self.sync_option_window();
    }

    pub fn option_open(&self) -> bool {
        self.desktop.window(WindowId::OPTION).is_some_and(|w| w.visible)
    }

    pub fn sync_option_window(&mut self) {
        // Ao abrir a janela de opções, sincroniza os checkboxes com o option.ini atual uma vez
        // (na borda de subida), sem sobrescrever os cliques do usuário nos quadros seguintes.
        let open = self.desktop.window(WindowId::OPTION).is_some_and(|w| w.visible);
        if open && !self.option_synced && !self.keys.option_applying {
            if let Some(runtime) = &mut self.option_game_runtime {
                runtime.state.auto_tracking = i32::from(self.options.auto_tracking_pawn);
                runtime.show(&mut self.keys.option.game_page, &mut self.shortcut, |id| self.keys.sysstrings.get(&id).cloned());
            } else {
                let options = self.options.clone();
                self.keys.sync_options(&options);
            }
            self.keys.option.audio_page.clear();
            self.keys.option.audio_page.fill(self.audio_volumes, |id| self.keys.sysstrings.get(&id).cloned());
        }
        self.option_synced = open;
    }

    pub fn world_event(&mut self, event: &killer_world::WorldEvent, world: &World) {
        self.keys.messenger.me_name = world.me.as_ref().map(|me| me.name.clone()).unwrap_or_default();
        self.keys.messenger.world_event(event, world, &mut self.desktop, &self.small);
        if let killer_world::WorldEvent::ShowBoard(board) = event {
            self.show_board(board.as_ref().map(|board| (board.links.as_slice(), board.html.as_str())));
        }
        if matches!(event, killer_world::WorldEvent::CastleSiegeInfo(_) | killer_world::WorldEvent::PartyMatchList(_))
            || matches!(event, killer_world::WorldEvent::SsqStatus(status) if matches!(status.body, l2_net::game::packets::SsqPage::Summary(_)))
        {
            self.chat.focused = false;
            self.board.html_form.focus = None;
            self.keys.party_room_making.focus = None;
            self.keys.party_room_making.lose_combo_focus();
            self.keys.option.lose_combo_focus();
            self.keys.select_delivery.combo.lose_focus();
        }
        if matches!(event, killer_world::WorldEvent::TutorialShowHtml(_) | killer_world::WorldEvent::TutorialCloseHtml(_)) {
            self.close_html_question(HtmlSource::Tutorial);
        }
        if matches!(event, killer_world::WorldEvent::MyTargetSelected(_)) && self.desktop.window(WindowId::GM).is_some_and(|window| window.visible) {
            self.keys.gm_target_selected(world, &self.small);
        }
        if let killer_world::WorldEvent::Die(die) = event
            && world.is_me(die.object_id)
        {
            match self.dialog.as_ref().map(|dialog| dialog.window) {
                Some(WindowId::DIALOG) => {
                    self.close_dialog();
                }
                Some(WindowId::DIALOG_BOTTOM) => {
                    self.death_answer = self.close_dialog().map(|dialog| KeyAction::Dialog(dialog.answer, false));
                }
                _ => {}
            }
        }
        self.tutorial.world_event(event, &mut self.desktop, &self.small, self.html_tables.as_deref());
        self.keys.world_event(event, &mut self.desktop, world);
        self.keys.party_room_chat(event, &self.small);
        self.keys.command_channel_chat(event, &mut self.desktop, &self.small);
    }

    pub fn shop_event(&mut self, event: &killer_world::WorldEvent, tables: Option<&DatTables>) {
        if matches!(event, killer_world::WorldEvent::BuyList(_) | killer_world::WorldEvent::SellList(_)) {
            self.keys.shop_preview.clear();
            if let Some(window) = self.desktop.window_mut(WindowId::SHOP_PREVIEW) {
                window.visible = false;
            }
            if self.dialog.as_ref().is_some_and(|d| d.answer == DialogAnswer::ShopPreview) {
                self.close_dialog();
            }
        }
        if self.keys.shop_preview.preview_event(event, tables, &mut self.desktop) {
            self.close_npc_html();
            self.keys.shop.clear();
            if self.dialog.as_ref().is_some_and(|d| matches!(d.answer, DialogAnswer::Shop | DialogAnswer::ShopPreview)) {
                self.close_dialog();
            }
        }
        if let killer_world::WorldEvent::PackageToList(list) = event
            && !list.targets.is_empty()
        {
            self.keys.select_delivery.open(list.targets.iter().map(|target| (target.object_id, target.name.clone())));
            self.keys.select_delivery.message = system_text(tables, killer_ui::delivery::SELECT_MESSAGE, &[]);
            if let Some(window) = self.desktop.window_mut(WindowId::SELECT_DELIVERY) { window.visible = true; }
            if let Some(index) = self.desktop.windows.iter().position(|window| window.id == WindowId::SELECT_DELIVERY) {
                let window = self.desktop.windows.remove(index);
                self.desktop.windows.push(window);
            }
        }
        if self.keys.delivery.world_event(event, tables, &mut self.desktop)
            && self.dialog.as_ref().is_some_and(|d| d.answer == DialogAnswer::Delivery)
        {
            self.close_dialog();
        }
        if self.keys.gm_store.world_event(event, tables, &mut self.desktop) {
            return;
        }
        if self.keys.warehouse.world_event(event, tables, &mut self.desktop) {
            self.close_npc_html();
            if self.dialog.as_ref().is_some_and(|d| d.answer == DialogAnswer::Warehouse) {
                self.close_dialog();
            }
        }
        if self.keys.manor_shop.world_event(event, tables, &mut self.desktop) {
            self.close_npc_html();
            if self.dialog.as_ref().is_some_and(|d| d.answer == DialogAnswer::ManorShop) {
                self.close_dialog();
            }
        }
        if self.keys.shop.world_event(event, tables, &mut self.desktop) {
            self.close_npc_html();
            if self.dialog.as_ref().is_some_and(|d| d.answer == DialogAnswer::Shop) {
                self.close_dialog();
            }
        }
    }

    pub fn private_shop_event(&mut self, event: &killer_world::WorldEvent, tables: Option<&DatTables>) {
        if self.keys.private_shop.private_manage_event(event, tables, &mut self.desktop) {
            self.close_npc_html();
            return;
        }
        if self.keys.private_shop.private_event(event, tables, &mut self.desktop) {
            self.close_npc_html();
            if self.dialog.as_ref().is_some_and(|d| d.answer == DialogAnswer::PrivateShop) {
                self.close_dialog();
            }
        }
    }

    pub fn manor_event(&mut self, event: &killer_world::WorldEvent, tables: Option<&DatTables>) {
        self.keys.manor_event(event, tables, &mut self.desktop);
    }

    pub fn route_system_line(&mut self, line: ChatLine) -> Option<ChatLine> {
        if line.group != crate::ui_petition::SYSTEM_GROUP_PETITION {
            return Some(line);
        }
        self.keys.petition.system_message(&line.text, &self.small, &mut self.desktop);
        None
    }

    pub fn snoop_event(&mut self, event: &killer_world::WorldEvent) {
        self.keys.snoop_event(event, &self.small, &mut self.desktop);
    }

    pub fn petition_event(&mut self, event: &killer_world::WorldEvent) {
        self.keys.petition.world_event(event);
        if let killer_world::WorldEvent::Say2(message) = event
            && let l2_net::game::Say2Body::Text { name, text } = &message.body
        {
            self.keys.petition.chat_message(message.kind, name, text, &self.small, &mut self.desktop);
        }
    }

    pub fn cast_event(&mut self, event: &killer_world::WorldEvent, world: &World, tables: Option<&DatTables>) {
        let killer_world::WorldEvent::MagicSkillUse(cast) = event else { return };
        if !magic_skill_data_accepted(world, cast, tables) {
            return;
        }
        self.keys.skill_reuse_started(event, world, killer_platform::time::Instant::now());
        self.keys.macro_skill_started(event, world);
    }

    pub fn olympiad_event(&mut self, event: &killer_world::WorldEvent, world: &World, tables: Option<&DatTables>) {
        match event {
            killer_world::WorldEvent::MagicSkillUse(cast) => {
                if !magic_skill_data_accepted(world, cast, tables) || !(0..=OLYMPIAD_MAX_SKILL_ID).contains(&cast.skill_id) {
                    return;
                }
                let name = tables
                    .and_then(|tables| tables.skillname.get(&(u32::try_from(cast.skill_id).unwrap_or(0), 1)))
                    .map(|skill| skill.name.clone())
                    .unwrap_or_default();
                for player in &mut self.keys.olympiad.players {
                    if player.object_id() == cast.caster_id {
                        player.add_message(&system_text(tables, OLYMPIAD_USE_SYSMSG, &[SystemMessageArg::Text(name.clone())]));
                    }
                }
                return;
            }
            killer_world::WorldEvent::Attack(attack) => {
                let Some(hit) = attack.hits.first() else { return };
                let Some(attacker_entity) = world.entity(attack.attacker_id) else { return };
                if world.entity(hit.target_id).is_none() || attack.attacker_id == hit.target_id || attacker_entity.dead {
                    return;
                }
                let attacker = world.entity(attack.attacker_id).map(|entity| entity.name.clone()).unwrap_or_default();
                for player in &mut self.keys.olympiad.players {
                    let id = player.object_id();
                    if id == attack.attacker_id && hit.flags & OLYMPIAD_HIT_CRITICAL != 0 {
                        player.add_message(&system_text(tables, OLYMPIAD_CRITICAL_SYSMSG, &[]));
                    } else if id == hit.target_id && hit.flags & OLYMPIAD_HIT_MISS != 0 {
                        player.add_message(&system_text(tables, OLYMPIAD_AVOIDED_SYSMSG, &[SystemMessageArg::Text(attacker.clone())]));
                    } else if id == hit.target_id && hit.flags & OLYMPIAD_HIT_SHIELD != 0 {
                        player.add_message(&system_text(tables, OLYMPIAD_SHIELD_SYSMSG, &[]));
                    }
                }
                return;
            }
            _ => {}
        }
        if let killer_world::WorldEvent::OlympiadSpelledInfo(info) = event {
            if let Some(index) = self.keys.olympiad.spelled_target(info.values[0]) {
                let buffs = info
                    .entries
                    .iter()
                    .take(killer_ui::abnormal::MAX_BUFFS)
                    .map(|entry| BuffIcon {
                        skill_id: entry.value,
                        level: i32::from(entry.level),
                        icon: skill_icon(tables, entry.value, i32::from(entry.level)),
                        remaining: (entry.amount >= 0).then_some(entry.amount),
                        debuff: skill_is_debuff(tables, entry.value, i32::from(entry.level)),
                        fade_ticks: 0,
                    })
                    .collect();
                self.keys.olympiad.abnormal[index] = AbnormalView { buffs, ..Default::default() };
            }
            return;
        }
        self.keys.olympiad.world_event(event, &mut self.desktop);
    }

    fn show_window(&mut self, id: WindowId, visible: bool) {
        if let Some(window) = self.desktop.window_mut(id) {
            window.visible = visible;
        }
        if visible && let Some(index) = self.desktop.windows.iter().position(|window| window.id == id) {
            let window = self.desktop.windows.remove(index);
            self.desktop.windows.push(window);
        }
    }

    pub fn show_html(&mut self, title: &str, html: &str) {
        self.npc_viewer = Some(NpcViewer::new(title, html, &self.small));
        self.show_window(WindowId::NPC_HTML, true);
    }

    pub fn gm_event_actions(&mut self, actions: Vec<killer_ui::gm_event::GmEventAction>, world: &World) {
        use killer_ui::gm_event::{Ed, GmEventAction};
        for action in actions {
            match action {
                GmEventAction::Command(command) => self.keys.send(NetCommand::BypassBuildCmd { command }),
                GmEventAction::Notice(id, number) => self.keys.warning_message(id, number),
                GmEventAction::SelectTarget(team) => {
                    let edit = if team == 0 { Ed::Leader1 } else { Ed::Leader2 };
                    self.keys.gm_event.set_edit(edit, "", &self.small);
                    let name = world.target.and_then(|id| world.entity(id)).map(|entity| entity.name.clone());
                    if let Some(name) = name {
                        self.keys.gm_event.set_edit(edit, &name, &self.small);
                    }
                }
                GmEventAction::ReadOptions(file) => self.keys.request_gm_event_file(file, false),
                GmEventAction::ReadCommandFile(file) => self.keys.request_gm_event_file(file, true),
                GmEventAction::OpenFence => {
                    self.keys.gm_fence.reset();
                    self.show_window(WindowId::GM_FENCE, true);
                }
                GmEventAction::Close => self.show_window(WindowId::GM_EVENT, false),
                GmEventAction::Minimize => {
                    let title = self.keys.sysstrings.get(&killer_ui::minimized::GM_EVENT_TIP_SYSSTRING).cloned().unwrap_or_default();
                    self.keys.minimized.minimize_gm(&mut self.desktop, &title);
                }
            }
        }
    }

    fn gm_fence_action(&mut self, action: killer_ui::gm_fence::FenceAction, world: &World) {
        use killer_ui::gm_event::Ed;
        use killer_ui::gm_fence::FenceAction;
        match action {
            FenceAction::Close => self.show_window(WindowId::GM_FENCE, false),
            FenceAction::Notice(id) => self.keys.warning_message(id, None),
            FenceAction::MyCoordinates => {
                if let Some(me) = &world.me {
                    let point = [me.position.x as i32, me.position.y as i32, me.position.z as i32];
                    self.keys.gm_fence.set_coordinates(point, &self.small);
                }
            }
            FenceAction::Create { point, width, height } => {
                let team1 = self.keys.gm_event.edit_text(Ed::Team1);
                let team2 = self.keys.gm_event.edit_text(Ed::Team2);
                let command = format!(
                    "{} create 1 {team1} {team2} {} {} {} {width} {height}",
                    killer_ui::gm_event::COMMAND_PREFIX,
                    point[0],
                    point[1],
                    point[2]
                );
                self.keys.send(NetCommand::BypassBuildCmd { command });
                self.keys.gm_event.fence_created();
                self.show_window(WindowId::GM_FENCE, false);
            }
        }
    }

    pub fn broadcast_event(&mut self, event: &killer_world::WorldEvent, tables: Option<&DatTables>, target: Option<i32>) {
        self.keys.broadcast.world_event(event, &mut self.desktop, &mut self.keys.olympiad, tables, target);
    }

    pub fn gm_clan_event(&mut self, event: &killer_world::WorldEvent) {
        if let killer_world::WorldEvent::GmViewPledgeInfo(info) = event {
            self.keys.sync_gm_clan(info);
        }
    }

    pub fn gm_quest_event(&mut self, event: &killer_world::WorldEvent, tables: Option<&DatTables>) {
        if let killer_world::WorldEvent::GmViewQuestInfo(info) = event {
            self.keys.sync_gm_quest(info, tables);
        }
    }

    pub fn gm_skills_event(&mut self, event: &killer_world::WorldEvent, tables: Option<&DatTables>) {
        match event {
            killer_world::WorldEvent::GmViewSkillInfo(info) => {
                let entries: Vec<(bool, i32, i32)> = info.entries.iter().map(|[kind, level, id]| (*kind == 1, *level, *id)).collect();
                self.keys.sync_gm_skills(&entries, true, tables);
            }
            killer_world::WorldEvent::GmViewMagicInfo(info) => {
                let entries: Vec<(bool, i32, i32)> = info.entries.iter().map(|[level, id]| (false, *level, *id)).collect();
                self.keys.sync_gm_skills(&entries, false, tables);
            }
            _ => {}
        }
    }

    pub fn gm_status_event(&mut self, event: &killer_world::WorldEvent, world: &World) {
        if let killer_world::WorldEvent::GmViewCharacterInfo(info) = event {
            let clan = info.tail_values[0];
            self.keys.sync_gm_status(info, crate::ui_keys::clan_label(world, clan));
        }
    }

    pub fn gm_inventory_event(&mut self, event: &killer_world::WorldEvent, tables: Option<&DatTables>) {
        if let killer_world::WorldEvent::GmHennaInfo(info) = event {
            self.keys.sync_gm_hennas(info, tables);
        }
        if let killer_world::WorldEvent::GmViewItemList(list) = event {
            self.keys.sync_gm_inventory(list, tables);
            if list.header[0] != 0 {
                self.show_window(WindowId::GM_INVENTORY, true);
            }
        }
    }

    pub fn gm_event_world(&mut self, event: &killer_world::WorldEvent) {
        use killer_world::WorldEvent;
        let sysstrings = self.keys.sysstrings.clone();
        let class_name = |class: i32| u32::try_from(class).ok().and_then(|class| sysstrings.get(&(killer_ui::gm_event::CLASS_NAME_BASE + class)).cloned()).unwrap_or_default();
        match event {
            WorldEvent::EventMatchManage(manage) => {
                self.show_window(WindowId::GM_EVENT, true);
                self.keys.gm_event.manage_header(manage.value, manage.flags, &manage.first_name, manage.middle_flag, &manage.second_name, manage.last_flag, &self.small);
                for (index, entry) in manage.entries.iter().enumerate() {
                    self.keys.gm_event.manage_entry(
                        manage.value,
                        index,
                        entry.flags[0],
                        entry.flags[1],
                        entry.value,
                        &entry.name,
                        entry.values[0],
                        entry.values[1],
                        &class_name(entry.values[0]),
                        &self.small,
                    );
                }
            }
            WorldEvent::EventMatchGmTest => self.show_window(WindowId::GM_EVENT, true),
            WorldEvent::EventMatchCreate(create) => self.keys.gm_event.game_created(create.value),
            WorldEvent::EventMatchTeamInfo(info) if self.desktop.window(WindowId::GM_EVENT).is_some_and(|window| window.visible) => {
                self.keys.gm_event.team_info_start(info.header[0], info.header[1]);
                for (index, entry) in info.entries.iter().enumerate() {
                    self.keys.gm_event.team_info_entry(index, entry.id, &entry.name, entry.values);
                }
                self.keys.gm_event.team_info_end(&class_name);
            }
            _ => {}
        }
    }

    pub fn calc_event(&mut self, event: &killer_world::WorldEvent) {
        if let killer_world::WorldEvent::ShowCalculator(calc) = event
            && u32::try_from(calc.calculator_id).is_ok_and(|id| id == killer_ui::calc_wnd::SHOW_ARGUMENT)
        {
            self.keys.open_calc(&mut self.desktop, &self.small);
        }
    }

    fn sync_fishing(&mut self, world: &World, tables: Option<&DatTables>) {
        use killer_world::fishing::FishText;
        let state = &world.fishing_window;
        if let Some(window) = self.desktop.window_mut(WindowId::FISHING) {
            window.visible = state.visible;
        }
        if !state.visible {
            return;
        }
        let sysstring = |id: u32| self.keys.sysstrings.get(&id).cloned().unwrap_or_default();
        let text = |message: FishText| match message {
            FishText::SysString(id) => sysstring(id),
            FishText::SystemMessage { id, penalty } => system_text(tables, id, &[SystemMessageArg::Number(penalty)]),
        };
        self.keys.fishing = killer_ui::fishing::FishPaint {
            started: state.started,
            time: state.time,
            bar_width: killer_ui::fishing::bar_width(state.hp, state.start_hp),
            effect_alpha: state.effect_on.then_some(state.effect_alpha),
            message: state.message.filter(|_| state.message_timer > 0.0).map(|message| killer_ui::fishing::FishMessage {
                text: text(message),
                value: state.value,
                alpha: state.message_alpha(),
                positive: state.message_positive,
            }),
            hint: state.hint_shown().then(|| {
                let exhausted = state.mode == 0;
                let id = if exhausted { killer_world::fishing::EXHAUSTED_SYSMSG } else { killer_world::fishing::RESISTING_SYSMSG };
                (exhausted, system_text(tables, id, &[]))
            }),
            viewport: world.fishing_view.param.is_some(),
        };
    }

    pub fn observer_event(&mut self, event: &killer_world::WorldEvent) {
        let visible = match event {
            killer_world::WorldEvent::ObserverStart(_) => true,
            killer_world::WorldEvent::ObserverEnd(_) => false,
            _ => return,
        };
        self.keys.olympiad.mark_ops.push(if visible { crate::ui_olympiad::MarkOp::SaveZero } else { crate::ui_olympiad::MarkOp::Restore });
        if let Some(window) = self.desktop.window_mut(WindowId::OBSERVER) {
            window.visible = visible;
        }
        if visible && let Some(index) = self.desktop.windows.iter().position(|window| window.id == WindowId::OBSERVER) {
            let window = self.desktop.windows.remove(index);
            self.desktop.windows.push(window);
        }
    }

    pub fn quest_info_event(&mut self, event: &killer_world::WorldEvent, tables: Option<&DatTables>) {
        if matches!(event, killer_world::WorldEvent::ShowQuestInfo) {
            self.keys.show_quest_info(tables, &mut self.desktop);
        }
    }

    pub fn empower_event(&mut self, event: &killer_world::WorldEvent) {
        if let killer_world::WorldEvent::ManagePledgePower(power) = event {
            self.keys.empower.set_powers(power.powers);
        }
    }

    pub fn hero_event(&mut self, event: &killer_world::WorldEvent, world: &World) {
        let sysstrings = &self.keys.sysstrings;
        let lookup = |id: u32| sysstrings.get(&id).cloned();
        self.keys.hero_tower.world_event(event, world, &lookup, &mut self.desktop);
    }

    pub fn multisell_event(&mut self, event: &killer_world::WorldEvent, world: &World, tables: Option<&DatTables>) {
        if matches!(event, killer_world::WorldEvent::MultiSellList(_)) {
            self.keys.multisell.world_event(event, world, tables, &mut self.desktop);
        }
    }

    pub fn train_event(&mut self, event: &killer_world::WorldEvent, world: &World, tables: Option<&DatTables>) {
        if matches!(event, killer_world::WorldEvent::AcquireSkillList(_) | killer_world::WorldEvent::AcquireSkillInfo(_) | killer_world::WorldEvent::AcquireSkillDone(_) | killer_world::WorldEvent::EnchantSkillList(_) | killer_world::WorldEvent::EnchantSkillInfo(_)) {
            self.keys.train.world_event(event, world, tables, &mut self.desktop);
        }
    }

    /// Os quatro pacotes de henna: os cabeçalhos de lista abrem a lista e as informações abrem a
    /// janela de informação, cada um esconde a outra janela (`0x10070d70`, `0x10071120`,
    /// `0x10070fa0` e `0x10071390`).
    pub fn henna_event(&mut self, event: &killer_world::WorldEvent, tables: Option<&DatTables>) {
        use killer_world::WorldEvent::{HennaEquipList, HennaItemInfo, HennaUnequipInfo, HennaUnequipList};
        if matches!(event, HennaEquipList(_) | HennaUnequipList(_) | HennaItemInfo(_) | HennaUnequipInfo(_)) {
            self.keys.henna.world_event(event, tables, &mut self.desktop);
        }
    }

    pub fn macros_event(&mut self, event: &killer_world::WorldEvent, world: &World) {
        if matches!(event, killer_world::WorldEvent::SendMacroList(_)) {
            self.keys.macros_event(event, world, &mut self.desktop);
        }
    }

    pub fn macro_key(&mut self, key: killer_ui::chat::InputKey) -> bool { self.keys.macro_key(key, &mut self.desktop) }

    pub fn trade_event(&mut self, event: &killer_world::WorldEvent, world: &World, tables: Option<&DatTables>) {
        if self.keys.trade.world_event(event, world, tables, &mut self.desktop)
            && self.dialog.as_ref().is_some_and(|d| d.answer == DialogAnswer::TradeCount)
        {
            self.close_dialog();
        }
    }

    pub fn take_commands(&mut self) -> Vec<UiCommand> {
        self.keys.take_commands()
    }

    pub fn window_under(&self, x: i32, y: i32) -> Option<(WindowId, i32, i32)> {
        if self.keys.minimized.hit(&self.desktop, x, y) { return None; }
        if let Some(window) = self.desktop.window(WindowId::SELECT_DELIVERY).filter(|window| window.visible)
            && self.keys.select_delivery.combo.open
            && self.keys.select_delivery.combo.expanded_rect().contains(x - window.rect.x, y - window.rect.y)
        { return Some((WindowId::SELECT_DELIVERY, x - window.rect.x, y - window.rect.y)); }
        if let Some(window) = self.desktop.window(WindowId::OPTION).filter(|window| window.visible)
            && self.keys.option.open_combo_hit(x - window.rect.x, y - window.rect.y).is_some()
        { return Some((WindowId::OPTION, x - window.rect.x, y - window.rect.y)); }
        let id = self.desktop.window_at(x, y)?;
        let window = self.desktop.window(id)?;
        Some((id, x - window.rect.x, y - window.rect.y))
    }

    pub fn hook_key(&mut self, msg: u32, vk: u32, lparam: u32, world: &World) -> bool {
        if self.dialog.as_ref().is_some_and(|d| d.view.input.is_some()) {
            return false;
        }
        let mut state = self.keys.key_state(world, self.chat.focused, self.dialog.is_some());
        state.tab_blocked = self.desktop.window(WindowId::SHOP).is_some_and(|w| w.visible);
        let hooked = self.keys.hook(msg, vk, lparam, &state, &mut self.shortcut);
        self.apply_hooked(&hooked, world);
        hooked.handled
    }

    pub fn menu_button(&mut self, button: MenuButton, world: &World) {
        let effect = match button {
            MenuButton::CharacterStatus => Effect::ToggleMain(None),
            MenuButton::Inventory => Effect::Inventory,
            MenuButton::Map => Effect::Minimap,
            MenuButton::System => Effect::SystemMenu,
        };
        self.apply_hooked(&Hooked { effects: vec![effect], handled: true }, world);
    }

    fn apply_hooked(&mut self, hooked: &Hooked, world: &World) {
        if hooked.effects.iter().any(|effect|matches!(effect,Effect::CloseAll)) {
            self.modern_menu_open=false;
            self.modern_system_open=false;
            self.layout_modern_menu();
        }
        let mut modern_hooked=hooked.clone();
        if self.shortcut.modern.is_some() {
            modern_hooked.effects.retain(|effect|!matches!(effect,Effect::SystemMenu));
            if hooked.effects.iter().any(|effect|matches!(effect,Effect::SystemMenu)) {
                self.keys.sound(if self.modern_system_open { killer_ui::interface_sound::SYSTEM_CLOSE } else { killer_ui::interface_sound::SYSTEM_OPEN });
                self.modern_system_open = !self.modern_system_open;
                self.modern_menu_open=self.modern_system_open;
                if let Some(window)=self.desktop.window_mut(WindowId::SYSTEM_MENU) {window.visible=false;}
                self.layout_modern_menu();
            }
        }
        let results = self.keys.apply_effects(&modern_hooked, &mut self.desktop, &mut self.shortcut, world);
        for result in results {
            match result {
                EffectResult::FocusChat => self.chat.focused = true,
                EffectResult::ChatTab(direction) => self.chat.cycle_tab(direction),
                EffectResult::GroupCompact => {
                    self.party_buffs_shown = !self.party_buffs_shown;
                    self.pet_buffs_shown = !self.pet_buffs_shown;
                }
            }
        }
    }

    pub fn right_released(&mut self, x: i32, y: i32, world: &World) {
        let Some(window) = self.desktop.window(WindowId::PARTY_STATUS).filter(|window| window.visible) else { return };
        let Some(PartyHit::Member(index)) = self.party_hit_at(window.rect, x, y) else { return };
        let member = party_view(world, None).members.get(index).map(|member| member.object_id);
        let assisted = member.and_then(|id| world.entity(id)).and_then(|entity| entity.target);
        if let (Some(object_id), Some(me)) = (assisted, world.me.as_ref()) {
            let origin = me.position.to_array().map(|value| value as i32);
            self.keys.send(killer_world::NetCommand::Action { object_id, origin, shift: self.keys.shift });
        }
    }

    pub fn right_click(&mut self, x: i32, y: i32, world: &World) -> bool {
        let Some((id, lx, ly)) = self.window_under(x, y) else { return false };
        if id == WindowId::SHORTCUT {
            self.shortcut.right_click(lx, ly);
            self.keys.drain_bar(&mut self.shortcut, world);
            return true;
        }
        self.keys.right_click(id, lx, ly)
    }

    fn paint_labels(&self, list: &mut DrawList, labels: &[Label]) {
        let line_height = self.large.line_height as i32;
        for label in labels {
            let first = list.quads.len();
            let x = label.screen[0].round() as i32;
            let y = label.screen[1].round() as i32;
            let name_y = y - ENGINE_NAME_RISE;
            let crests = label.crests_width();
            let name_w = label.modern_text_size.map_or_else(||self.large.measure(&label.name) as i32,|size|size[0][0].round() as i32);
            let width = name_w + crests;
            if let Some(mark) = label.mark {
                let name_x = x - width / 2 + crests;
                let (w, h) = (crate::characters::TARGET_MARK_WIDTH, crate::characters::TARGET_MARK_HEIGHT);
                let texture = mark.texture();
                list.draw_texture(name_x - w - label.mark_shift(), name_y - 1, w, h, 0, 0, w, h, texture, 1.0);
                list.draw_texture(name_x + name_w, name_y - 1, w, h, 0, 0, w, h, texture, 1.0);
            }
            if let Some(icon) = &label.broadcast_icon {
                let (ix, iy) = icon.origin(x, y, name_y);
                let side = crate::characters::BROADCAST_ICON_SIZE;
                list.draw_texture(ix, iy, side, side, 0, 0, side, side, &icon.texture(), 1.0);
            }
            let title_w = if label.title.is_empty() { 0 } else { self.large.measure(&label.title) as i32 };
            let relation = label.relation_origin(x, y, title_w, line_height);
            if let Some((icon_x, icon_y)) = relation {
                let (w, h) = (crate::characters::RELATION_ICON_WIDTH, crate::characters::RELATION_ICON_HEIGHT);
                for texture in [label.relation_icons.base, label.relation_icons.leader].into_iter().flatten() {
                    list.draw_texture(icon_x, icon_y, w, h, 0, 0, w, h, texture, 1.0);
                }
            }
            if !label.name_visible {
                for quad in &mut list.quads[first..] {quad.depth=label.depth;}
                continue;
            }
            let mut left = x - width / 2;
            if let Some(ally) = &label.ally_crest {
                let (w, h) = (LABEL_ALLY_CREST_WIDTH, LABEL_CREST_HEIGHT);
                list.draw_texture(left, name_y, w, h, 0, LABEL_CREST_V, w, h, ally, 1.0);
                left += w;
            }
            if let Some(crest) = &label.clan_crest {
                let (w, h) = (LABEL_PLEDGE_CREST_WIDTH, LABEL_CREST_HEIGHT);
                list.draw_texture(left, name_y, w, h, 0, LABEL_CREST_V, w, h, crest, 1.0);
                left += w;
            }
            if self.shortcut.modern.is_none() {list.draw_text(left, name_y, rgb(label.name_color), &label.name, &self.large);}
            let title = &label.title;
            if !title.is_empty() && self.shortcut.modern.is_none() {
                list.draw_text(label.title_left(x, title_w, relation), y - 2 * line_height, rgb(label.title_color), title, &self.large);
            }
            for quad in &mut list.quads[first..] {quad.depth=label.depth;}
        }
        for label in labels {
            if let Some(sign) = &label.store_sign { sign.paint(list, &self.small); }
            if self.shortcut.modern.is_none() && let Some(speech)=&label.speech {
                let first=list.quads.len();
                speech.paint_speech(list,&self.small);
                for quad in &mut list.quads[first..] {quad.depth=label.depth;}
            }
        }
    }

    pub fn set_tab_names(&mut self, tables: &DatTables) {
        self.board.relabel(|id| tables.sysstring.get(&id).cloned());
        self.help.title = tables.sysstring.get(&145).cloned().unwrap_or_default();
        self.keys.set_sysstrings(tables);
        self.keys.sysmenu.restart_locked = tables.login_ini.auto_logon.is_some();
        self.keys.ssq.set_font(&self.small);
        self.keys.siege.view.date.text_height = self.small.line_height as i32;
        self.keys.option.set_font(&self.small);
        self.keys.petition.set_font(&self.small);
        self.keys.hero_tower.tower.set_font(&self.small);
        self.keys.macros.edit.set_font(&self.small);
        self.keys.quest.set_font(&self.small);
        self.keys.gm_quest.set_font(&self.small);
        for (name, id) in self.chat.tab_names.iter_mut().zip(TAB_SYSSTRING_IDS) {
            if let Some(text) = tables.sysstring.get(&id) {
                *name = text.clone();
            }
        }
        for (label, (id, _, _, _)) in self.chat.filter_labels.iter_mut().zip(killer_ui::chat::FILTER_ROWS) {
            if let Some(text) = tables.sysstring.get(&id) {
                *label = text.clone();
            }
        }
        for (slot, id) in [
            (&mut self.chat.filter_ok, killer_ui::chat::FILTER_OK_SYSSTRING),
            (&mut self.chat.filter_cancel, killer_ui::chat::FILTER_CANCEL_SYSSTRING),
            (&mut self.chat.option_tooltip, killer_ui::chat::OPTION_TOOLTIP_SYSSTRING),
            (&mut self.chat.messenger_tooltip, killer_ui::chat::MESSENGER_TOOLTIP_SYSSTRING),
        ] {
            if let Some(text) = tables.sysstring.get(&id) {
                *slot = text.clone();
            }
        }
        if let Some(text) = tables.sysstring.get(&CHAT_TITLE_SYSSTRING) {
            self.npc_title = text.clone();
        }
        self.target_labels = killer_ui::target::info_labels(|id| tables.sysstring.get(&id).cloned());
        for id in [dialog::OK_SYSSTRING, dialog::CANCEL_SYSSTRING, dialog::YES_SYSSTRING, dialog::NO_SYSSTRING] {
            if let Some(text) = tables.sysstring.get(&id) {
                self.button_labels.insert(id, text.clone());
            }
        }
    }

    #[cfg(test)]
    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        self.mouse_down_at(x, y, Instant::now(), &World::default())
    }

    fn take_double_click(&mut self, x: i32, y: i32, now: Instant) -> bool {
        let double = self.last_left_press.is_some_and(|(at, (px, py))| {
            now.saturating_duration_since(at).as_millis() <= DOUBLE_CLICK_TIME_MS
                && (x - px).abs() * 2 <= DOUBLE_CLICK_RECT.0
                && (y - py).abs() * 2 <= DOUBLE_CLICK_RECT.1
        });
        self.last_left_press = if double { None } else { Some((now, (x, y))) };
        double
    }

    pub fn mouse_down_at(&mut self, x: i32, y: i32, now: Instant, world: &World) -> bool {
        self.cursor = (x, y);
        if let Some((lx, ly)) = self.warning_local(x, y) {
            self.warning_pressed = dialog::WarningDialog::hit(lx, ly);
            return true;
        }
        if self.modern_menu_open && self.window_under(x,y).is_none_or(|(id,_,_)|id!=WindowId::MENU) {
            self.modern_menu_open=false;
            self.modern_system_open=false;
            self.layout_modern_menu();
        }
        if self.window_under(x, y).map(|(id, _, _)| id) != Some(WindowId::SELECT_DELIVERY) {
            self.keys.select_delivery.combo.lose_focus();
        }
        if self.window_under(x, y).map(|(id, _, _)| id) != Some(WindowId::OPTION) {
            self.keys.option.lose_combo_focus();
        }
        if self.window_under(x, y).map(|(id, _, _)| id) != Some(WindowId::MULTISELL) {
            self.keys.multisell.view.quantity_blur();
        }
        self.keys.manor_setting.blur_outside(self.window_under(x, y).map(|(id, _, _)| id));
        self.keys.hero_tower.blur_outside(self.window_under(x, y).map(|(id, _, _)| id));
        self.keys.minimized.sync_owner(&self.desktop);
        if self.keys.minimized.mouse_down(&mut self.desktop, x, y) { return true; }
        if let Some((id, _, _)) = self.window_under(x, y)
            && let Some(window) = self.desktop.window(id)
            && self.status_resize.press(window, x, y)
        {
            return true;
        }
        let double = self.take_double_click(x, y, now);
        if self.chat_filter_down(x, y) {
            return true;
        }
        if let Some((under, _, _)) = self.window_under(x, y)
            && let Some((enabled, close, silent)) = self.button_map.borrow().iter().rev().find(|(id, button)| *id == under && button.contains(x, y)).map(|(_, button)| (button.enabled, button.close, button.silent))
        {
            if !self.replaying_close && !silent {
                self.keys.sound(if enabled { killer_ui::interface_sound::CLICK } else { killer_ui::interface_sound::CLICK_FAILED });
            }
            self.close_pressed = close.then_some(under);
            self.button_held = Some((x, y));
            if close && enabled && !self.replaying_close {
                self.deferred_close = Some((under, x, y, now));
                return true;
            }
        } else {
            self.close_pressed = None;
            self.button_held = None;
        }
        let Some((id, _, _)) = self.window_under(x, y) else {
            self.chat.focused = false;
            self.desktop.clear_focus();
            return false;
        };
        self.desktop.focus_window(id);
        if id != WindowId::CHAT && !(1..5).any(|tab| id == WindowId(ChatView::split_window_id(tab))) {
            self.chat.focused = false;
        }
        if double
            && id == WindowId::HERO_TOWER
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            if self.keys.hero_tower.double_click(x - wx, y - wy, &mut self.desktop) {
                return true;
            }
        }
        if double
            && matches!(id, WindowId::MANOR_SEED_SETTING | WindowId::MANOR_CROP_SETTING | WindowId::MANOR_CROP_SELL)
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            if self.keys.manor_setting.double_click(id, x - wx, y - wy, &mut self.desktop) {
                return true;
            }
        }
        if double
            && id == WindowId::MESSENGER
            && let Some((wx, wy)) = self.desktop.window(id).map(|window| (window.rect.x, window.rect.y))
            && self.keys.messenger.double_click(id, x - wx, y - wy, &mut self.desktop)
        {
            return true;
        }
        if double
            && self.keys.handles(id)
            && let Some(window) = self.desktop.window(id)
            && self.keys.double_click(id, x - window.rect.x, y - window.rect.y, world)
        {
            return true;
        }
        if (id == WindowId::NPC_HTML || id == WindowId::ITEM_HTML)
            && let Some(window) = self.desktop.window(id)
            && npc_viewer::close_button().offset(window.rect.x, window.rect.y).contains(x, y)
        {
            self.close_html_window(id);
            return true;
        }
        if id == WindowId::HELP && let Some(window) = self.desktop.window(id) {
            let (hx,hy) = (x-window.rect.x,y-window.rect.y);
            self.help.mouse_down(&mut self.desktop,hx,hy,&self.small);
            if npc_viewer::close_button().contains(hx,hy) {
                return true;
            }
        }
        if id == WindowId::BOARD
            && let Some(window) = self.desktop.window(id)
        {
            let (bx, by) = (x - window.rect.x, y - window.rect.y);
            if killer_ui::board::close_button().contains(bx, by) {
                self.show_board(None);
                return true;
            }
            let hit = self.board.hit(bx, by, &self.small);
            if !matches!(hit, Some(killer_ui::board::BoardHit::Edit(_))) {
                self.board.html_form.focus = None;
            }
            match hit {
                Some(killer_ui::board::BoardHit::Edit(var)) => {
                    self.board.html_form.focus_var(&var);
                    return true;
                }
                Some(killer_ui::board::BoardHit::Tab(index)) => {
                    let mut action = None;
                    self.board.press_tab(index, &mut |command: &str| {
                        action = html_action(command);
                        html_action_result(command, action.is_some())
                    });
                    self.board_command = action;
                    return true;
                }
                Some(killer_ui::board::BoardHit::Favorite) => {
                    self.board_favorite_pressed = true;
                    return true;
                }
                Some(killer_ui::board::BoardHit::Link(action)) => {
                    self.link_pressed = Some(action);
                }
                None => {}
            }
        }
        if id == WindowId::MULTISELL
            && let Some(window) = self.desktop.window(id)
        {
            self.keys.multisell.view.quantity_mouse_down(x - window.rect.x, y - window.rect.y, &self.small);
        }
        if matches!(id, WindowId::GM_EVENT | WindowId::GM_FENCE)
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            if id == WindowId::GM_EVENT {
                self.keys.gm_event.mouse_down(x - wx, y - wy, &self.small);
            } else {
                self.keys.gm_fence.mouse_down(x - wx, y - wy, &self.small);
            }
            if self.desktop.window(id).is_some_and(|window| window.visible) {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if id == WindowId::GM
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            self.keys.gm.mouse_down(x - wx, y - wy, &self.small);
            if self.desktop.window(id).is_some_and(|window| window.visible) {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if id == WindowId::GM_INVENTORY
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            if self.keys.gm_inventory.mouse_down(x - wx, y - wy).is_none() {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if self.keys.messenger.owns(id)
            && let Some((wx, wy)) = self.desktop.window(id).map(|window| (window.rect.x, window.rect.y))
        {
            let drag = self.keys.messenger.mouse_down(id, x - wx, y - wy, &self.small);
            if drag {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if matches!(id, WindowId::BROADCAST_PARTY_0 | WindowId::BROADCAST_PARTY_1 | WindowId::BROADCAST_PLAYER_0 | WindowId::BROADCAST_PLAYER_1)
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            self.keys.broadcast.mouse_down(id, x - wx, y - wy, None);
            if !self.keys.broadcast.observer.locked {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if id == WindowId::GM_CLAN
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            let on_frame = y - wy < killer_ui::gm_clan::GRIP.h;
            self.keys.gm_clan.mouse_down(x - wx, y - wy);
            if on_frame && self.desktop.window(id).is_some_and(|window| window.visible) {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if id == WindowId::GM_QUEST
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            if !self.keys.gm_quest.mouse_down(x - wx, y - wy) {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if id == WindowId::GM_SKILLS
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            if !self.keys.gm_skills.mouse_down(x - wx, y - wy) {
                self.desktop.begin_drag(x, y);
            }
            if self.keys.gm_skills.take_pickup() {
                self.keys.sound(killer_ui::interface_sound::PICKUP);
            }
            return true;
        }
        if id == WindowId::GM_STATUS
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            if !self.keys.gm_status.mouse_down(x - wx, y - wy) {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if id == WindowId::CALC
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            self.keys.calc.mouse_down(x - wx, y - wy, &self.small);
            if self.desktop.window(id).is_some_and(|window| window.visible) {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if id == killer_ui::command_channel::WINDOW_ID
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            self.keys.command_channel_mouse_down(x - wx, y - wy, &self.small);
            self.desktop.begin_drag(x, y);
            return true;
        }
        if id == WindowId::PARTY_ROOM
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            self.keys.party_room_mouse_down(x - wx, y - wy, &self.small);
            self.desktop.begin_drag(x, y);
            return true;
        }
        if killer_ui::gm_snoop::WINDOW_IDS.contains(&id)
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            self.keys.snoop_mouse_down(id, x - wx, y - wy);
            self.desktop.begin_drag(x, y);
            return true;
        }
        if matches!(id, WindowId::PETITION_BEGIN | WindowId::PETITION | WindowId::PETITION_GM | WindowId::PETITION_FEEDBACK)
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            self.keys.petition.mouse_down(id, x - wx, y - wy, &self.small, &mut self.desktop);
            if self.desktop.window(id).is_some_and(|window| window.visible) {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if id == WindowId::HERO_TOWER
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            self.keys.hero_tower.mouse_down_tower(x - wx, y - wy, &self.small, &mut self.desktop);
            if self.desktop.window(id).is_some_and(|window| window.visible) {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if matches!(id, WindowId::MANOR_SEED_CHANGE | WindowId::MANOR_CROP_CHANGE | WindowId::MANOR_CROP_SELL_CHANGE)
            && let Some(window) = self.desktop.window(id)
        {
            let (wx, wy) = (window.rect.x, window.rect.y);
            match id {
                WindowId::MANOR_SEED_CHANGE => self.keys.manor_setting.mouse_down_change(x - wx, y - wy, &self.small, &mut self.desktop),
                WindowId::MANOR_CROP_CHANGE => self.keys.manor_setting.mouse_down_crop_change(x - wx, y - wy, &self.small, &mut self.desktop),
                _ => self.keys.manor_setting.mouse_down_procure(x - wx, y - wy, &self.small, &mut self.desktop),
            };
            if self.desktop.window(id).is_some_and(|window| window.visible) {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if self.keys.handles(id)
            && let Some(window) = self.desktop.window(id)
            && self.keys.mouse_down(id, x - window.rect.x, y - window.rect.y, &mut self.desktop)
        {
            if self.desktop.window(id).is_some_and(|window| window.visible) {
                self.desktop.begin_drag(x, y);
            }
            return true;
        }
        if id == WindowId::TARGET
            && let Some(window) = self.desktop.window(id)
            && killer_ui::target::info_button().offset(window.rect.x, window.rect.y).contains(x, y)
        {
            self.toggle_target_info();
            return true;
        }
        self.desktop.begin_drag(x, y);
        if id == WindowId::TARGET
            && let Some(window) = self.desktop.window(id)
            && !window.grip_rect().contains(x, y)
            && !killer_ui::target::close_button_sized(window.rect.w).offset(window.rect.x, window.rect.y).contains(x, y)
        {
            self.target_clicked = true;
        }
        if id == WindowId::TARGET
            && let Some(window) = self.desktop.window(id)
            && killer_ui::target::close_button().offset(window.rect.x, window.rect.y).contains(x, y)
        {
            self.keys.send(killer_world::NetCommand::TargetUnselect);
            self.keys.sound(killer_ui::interface_sound::WINDOW_CLOSE);
            self.desktop.end_drag();
            return true;
        }
        if id == WindowId::STATUS
            && let Some(window) = self.desktop.window(id)
            && !window.grip_rect().contains(x, y)
        {
            self.status_clicked = true;
        }
        if id == WindowId::SHORTCUT {
            self.rotate_pressed = self.over_rotate_button(x, y);
            self.pad_pressed = self.over_pad_button(x, y);
            self.page_pressed = self.page_button_under(x, y);
            if let Some(window) = self.desktop.window(id) {
                self.shortcut.mouse_down(x - window.rect.x, y - window.rect.y);
            }
        }
        if id == WindowId::MENU && self.shortcut.modern.is_some() {
            self.modern_menu_pressed=self.modern_menu_hit(x,y);
            return true;
        }
        if id == WindowId::MENU {
            self.menu_pressed = self.menu_button_at(x, y);
        }
        if (id == WindowId::PET_STATUS || id == WindowId::SUMMONED_STATUS)
            && let Some(window) = self.desktop.window(id)
        {
            let hit = pet_status::hit_sized(x - window.rect.x, y - window.rect.y, window.rect.w);
            self.pet_clicked = hit == Some(PetHit::Select);
            self.pet_pressed = hit.filter(|hit| *hit != PetHit::Select);
        }
        if id == WindowId::PARTY_STATUS
            && let Some(window) = self.desktop.window(id)
        {
            let hit = self.party_hit_at(window.rect, x, y);
            let last_ten_pixels = x > window.rect.x + window.rect.w - 10;
            if let Some(PartyHit::Member(index)) = hit {
                self.party_clicked = (!last_ten_pixels).then_some(index);
                self.party_pressed = None;
            } else {
                self.party_pressed = hit;
            }
        }
        if id == WindowId::RESTART_MENU
            && let Some(window) = self.desktop.window(id)
        {
            self.restart_pressed = RestartView::new([true; 5]).button_at(x - window.rect.x, y - window.rect.y);
        }
        if self.parked_dialog.as_ref().is_some_and(|parked| parked.window == id) {
            std::mem::swap(&mut self.dialog, &mut self.parked_dialog);
        }
        if (id == WindowId::DIALOG || id == WindowId::DIALOG_BOTTOM)
            && let Some(window) = self.desktop.window(id)
            && let Some(dialog) = self.dialog.as_mut().filter(|d| d.window == id)
        {
            dialog.pressed = dialog.view.button_at(x - window.rect.x, y - window.rect.y);
        }
        if let Some(window) = self.desktop.window(id) {
            self.tutorial.mouse_down(id, x - window.rect.x, y - window.rect.y, &self.small);
        }
        if (id == WindowId::NPC_HTML || id == WindowId::ITEM_HTML)
            && let Some(window) = self.desktop.window(id)
        {
            let (lx, ly) = (x - window.rect.x, y - window.rect.y);
            let font = self.small.clone();
            if let Some(viewer) = self.html_viewer_slot(id).as_mut() {
                viewer.pointer = Some((lx, ly));
                viewer.form.focus = None;
                if viewer.combo_press(lx, ly) || viewer.scroll_press(lx, ly) {
                    self.link_pressed = None;
                } else {
                    if let Some(var) = viewer.edit_var_at(lx, ly).map(str::to_string) {
                        viewer.form.focus_var(&var);
                        if let Some(top) = viewer.edit_top_at(lx, ly) {
                            viewer.scroll_to_edit(top);
                        }
                    }
                    if viewer.edit_var_at(lx, ly).is_none() && viewer.button_at(lx, ly).is_none() {
                        viewer.stop_text_anim();
                    }
                    viewer.pressed = viewer.button_at(lx, ly);
                    self.link_pressed = viewer.link_at(lx, ly, &font).map(str::to_string);
                }
            }
            log::debug!("clique na janela do NPC em ({lx}, {ly}): {:?}", self.link_pressed);
        }
        if id == WindowId::SYSTEM_MSG && self.shortcut.modern.is_none() && let Some(window) = self.desktop.window(id) {
            let (lx, ly) = (x - window.rect.x, y - window.rect.y);
            self.system_msg.scroll_press(lx, ly);
        }
        if id == WindowId::CHAT {
            self.chat.focused = true;
            if let Some(window) = self.desktop.window(id) {
                let (lx, ly) = (x - window.rect.x, y - window.rect.y);
                if self.shortcut.modern.is_none() && self.chat.size_press(lx, ly) {
                    self.desktop.end_drag();
                } else if self.shortcut.modern.is_none() && self.chat.scroll_press(lx, ly) {
                } else if self.shortcut.modern.is_none() && self.chat.messenger_rect().contains(lx, ly) {
                    self.keys.messenger.toggle_messenger(&mut self.desktop);
                } else if (if self.shortcut.modern.is_some() {crate::ui_modern::chat_option_rect(self.chat.height)}else{self.chat.option_rect()}).contains(lx, ly) {
                    self.chat.toggle_filter();
                } else if let Some(tab) = if self.shortcut.modern.is_some() { crate::ui_modern::chat_tab_at(lx,ly) }else{self.chat.tab_at(lx, ly)} {
                    if self.chat.active_tab != tab {
                        self.keys.sound(killer_ui::interface_sound::WINDOW_OPEN);
                    }
                    if !self.chat.detached[tab] {
                        self.chat.tab_press = (tab > 0).then_some((tab, x, y));
                    }
                    self.chat.select_tab(tab);
                }
            }
        }
        true
    }

    /// O painel de filtro fica acima da janela de chat, fora do retângulo dela, então o clique
    /// nele é testado antes do resto. As linhas mudam na descida; `OK` e `Cancel` só valem na
    /// subida, pra arte de pressionado aparecer.
    fn chat_filter_down(&mut self, x: i32, y: i32) -> bool {
        let Some(hit) = self.chat_filter_hit(x, y) else { return false };
        match hit {
            FilterHit::Row(_) => {
                self.chat.filter_click(hit);
            }
            FilterHit::Ok | FilterHit::Cancel => self.chat.pressed = Some(hit),
        }
        true
    }

    fn chat_filter_hit(&self, x: i32, y: i32) -> Option<FilterHit> {
        let window = self.desktop.window(WindowId::CHAT).filter(|w| w.visible)?;
        self.chat.filter_hit(x - window.rect.x, y - window.rect.y)
    }

    /// Solta `OK` ou `Cancel`. Devolve o texto do `chatfilter.ini` quando o `OK` gravou, pra
    /// quem tem o cliente salvar o arquivo como o original faz em `0x100555a0`.
    pub fn chat_filter_released(&mut self) -> Option<String> {
        let pressed = self.chat.pressed.take()?;
        let released = self.chat_filter_hit(self.cursor.0, self.cursor.1)?;
        if released != pressed {
            return None;
        }
        self.chat.filter_click(released)?;
        Some(self.chat.filter.to_ini())
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) -> bool {
        self.cursor = (x, y);
        if let Some(id) = self.status_resize.window() {
            if let Some(window) = self.desktop.window_mut(id) {
                self.status_resize.drag(window, x, y);
            }
            return true;
        }
        if let Some(window) = self.desktop.window(WindowId::SHORTCUT) {
            self.shortcut.auto_drag(x - window.rect.x);
        }
        if let Some(window) = self.desktop.window(WindowId::MULTISELL).filter(|window| window.visible) {
            self.keys.multisell.view.quantity_mouse_move(x - window.rect.x, &self.small);
        }
        self.keys.minimized.sync_owner(&self.desktop);
        if self.keys.minimized.mouse_move(&self.desktop, x, y) { self.keys.tooltip = None; return true; }
        for id in [WindowId::MANOR_SEED_CHANGE, WindowId::MANOR_CROP_CHANGE, WindowId::MANOR_CROP_SELL_CHANGE] {
            if let Some(window) = self.desktop.window(id).filter(|window| window.visible) {
                self.keys.manor_setting.mouse_move(id, x - window.rect.x, y - window.rect.y, &self.small);
            }
        }
        for id in [WindowId::PETITION_BEGIN, WindowId::PETITION, WindowId::PETITION_GM, WindowId::PETITION_FEEDBACK] {
            if let Some(window) = self.desktop.window(id).filter(|window| window.visible) {
                self.keys.petition.mouse_move(id, x - window.rect.x, y - window.rect.y, &self.small);
            }
        }
        if let Some(window) = self.desktop.window(WindowId::PARTY_ROOM).filter(|window| window.visible) {
            let (wx, wy) = (window.rect.x, window.rect.y);
            self.keys.party_room_mouse_move(x - wx, y - wy, &self.small);
        }
        if let Some(window) = self.desktop.window(killer_ui::command_channel::WINDOW_ID).filter(|window| window.visible) {
            let (wx, wy) = (window.rect.x, window.rect.y);
            self.keys.command_channel_mouse_move(x - wx, y - wy, &self.small);
        }
        if let Some(window) = self.desktop.window(WindowId::HERO_TOWER).filter(|window| window.visible) {
            self.keys.hero_tower.mouse_move(WindowId::HERO_TOWER, x - window.rect.x, y - window.rect.y, &self.small);
        }
        if let Some(window) = self.desktop.window(WindowId::MACRO_EDIT).filter(|window| window.visible) {
            self.keys.macros.edit.mouse_move_box(x - window.rect.x);
        }
        for id in [WindowId::NPC_HTML, WindowId::ITEM_HTML] {
            if let Some(window) = self.desktop.window(id).filter(|window| window.visible)
                && let Some(viewer) = (if id == WindowId::ITEM_HTML { self.item_viewer.as_mut() } else { self.npc_viewer.as_mut() })
            {
                let (lx, ly) = (x - window.rect.x, y - window.rect.y);
                viewer.pointer = Some((lx, ly));
                viewer.combo_move(lx, ly);
                viewer.scroll_move(ly);
            }
        }
        for id in [WindowId::PARTY_MATCH, WindowId::PARTY_ROOM_MAKING, WindowId::SIEGE, WindowId::SSQ, WindowId::QUEST_LIST_INFO, WindowId::MANOR_INFO, WindowId::MANOR_SEED_SETTING, WindowId::MANOR_CROP_SETTING, WindowId::MANOR_CROP_SELL] {
            if let Some(window) = self.desktop.window(id).filter(|window| window.visible) {
                self.keys.native_mouse_move(id, x - window.rect.x, y - window.rect.y);
            }
        }
        if let Some(window) = self.desktop.window(WindowId::OPTION).filter(|window| window.visible) {
            self.keys.option.combo_mouse_move(x - window.rect.x, y - window.rect.y);
        }
        if let Some((tab, px, py)) = self.chat.tab_press.filter(|(_, px, py)| (*px, *py) != (x, y)) {
            self.chat.tab_press = None;
            self.start_tab_drag(tab, (px, py), (x, y));
        }
        let dragging = self.desktop.dragging().is_some();
        if dragging {
            self.desktop.drag(x, y);
            self.follow_chat();
        }
        let map_dragged = self.desktop.window(WindowId::TOWN_MAP).filter(|w| w.visible)
            .is_some_and(|w| self.keys.special_maps.town.mouse_move(x - w.rect.x, y - w.rect.y));
        let scrolled = [WindowId::TRAIN, WindowId::MACRO_EDIT, WindowId::INVENTORY, WindowId::MAIN, WindowId::SHOP, WindowId::PRIVATE_SHOP, WindowId::SHOP_PREVIEW, WindowId::WAREHOUSE, WindowId::DELIVERY, WindowId::TRADE, WindowId::PET_WINDOW, WindowId::RECIPE_BOOK, WindowId::RECIPE_TREE, WindowId::RECIPE_SHOP, WindowId::RECIPE_SELL, WindowId::GM_QUEST, WindowId::GM_CLAN, WindowId::GM_STORE, WindowId::MESSENGER].into_iter().any(|id| {
            self.desktop.window(id).is_some_and(|w| w.visible)
                && self.desktop.window(id).is_some_and(|w| self.keys.mouse_move(id, y - w.rect.y))
        });
        if let Some((wx, wy)) = self.desktop.window(WindowId::MAP_COVER).filter(|w| w.visible).map(|w| (w.rect.x, w.rect.y)) {
            self.keys.minimap.mouse_move(x - wx, y - wy);
        }
        if let Some((wx, wy)) = self.desktop.window(WindowId::MAIN).filter(|w| w.visible).map(|w| (w.rect.x, w.rect.y)) {
            if self.keys.main.tab == killer_ui::keys::MainTab::Quests {
                let page = killer_ui::mainwnd::PAGE_RECT;
                self.keys.quest.mouse_move(x - wx - page.x, y - wy - page.y);
            } else if self.keys.main.tab == killer_ui::keys::MainTab::Clan {
                let page = killer_ui::mainwnd::PAGE_RECT;
                self.keys.pledge.header_move(x - wx - page.x, y - wy - page.y);
                self.keys.quest.tooltip_hover = false;
            } else {
                self.keys.quest.tooltip_hover = false;
            }
        }
        let chat_ids: Vec<WindowId> = self.keys.messenger.chats.iter().map(|chat| chat.id).collect();
        for id in chat_ids {
            if let Some(window) = self.desktop.window(id).filter(|w| w.visible) {
                let (wx, wy) = (window.rect.x, window.rect.y);
                self.keys.messenger.mouse_move(id, x - wx, y - wy, &self.small);
            }
        }
        let chat_resized = self.chat_resize_move(x, y);
        let list_dragged = {
            let chat = self.desktop.window(WindowId::CHAT).map(|w| w.rect.y);
            let system = self.desktop.window(WindowId::SYSTEM_MSG).map(|w| w.rect.y);
            chat.is_some_and(|top| self.chat.scroll_drag_to(y - top)) | system.is_some_and(|top| self.system_msg.scroll_drag_to(y - top))
        };
        let over = if dragging { None } else { self.window_under(x, y) };
        self.keys.hover(over, &self.shortcut);
        if let Some((WindowId::CHAT, lx, ly)) = over
            && (if self.shortcut.modern.is_some() {crate::ui_modern::chat_option_rect(self.chat.height)}else{self.chat.option_rect()}).contains(lx, ly)
            && !self.chat.option_tooltip.is_empty()
        {
            self.keys.tooltip = Some(killer_ui::tooltip::Tooltip { name: self.chat.option_tooltip.clone(), ..Default::default() });
        }
        if let Some((WindowId::CHAT, lx, ly)) = over
            && self.shortcut.modern.is_none()
            && self.chat.messenger_rect().contains(lx, ly)
            && !self.chat.messenger_tooltip.is_empty()
        {
            self.keys.tooltip = Some(killer_ui::tooltip::Tooltip { name: self.chat.messenger_tooltip.clone(), ..Default::default() });
        }
        if let Some((WindowId::PARTY_STATUS, lx, ly)) = over
            && let Some(window) = self.desktop.window(WindowId::PARTY_STATUS)
            && self.party_cache.leader_icon_at(lx, ly, window.rect.w, &self.small)
        {
            let text = |id: u32| self.keys.sysstrings.get(&id).cloned().unwrap_or_default();
            let rule = usize::try_from(self.party_cache.loot_rule).ok().and_then(|rule| party::LOOT_RULE_SYSSTRINGS.get(rule));
            if let Some(rule) = rule {
                let name = format!("{} : {}", text(party::LOOT_RULE_LABEL_SYSSTRING), text(*rule));
                self.keys.tooltip = Some(killer_ui::tooltip::Tooltip { name, ..Default::default() });
            }
        }
        if let Some((pet_id @ (WindowId::PET_STATUS | WindowId::SUMMONED_STATUS), lx, ly)) = over
            && let Some(window) = self.desktop.window(pet_id)
            && pet_status::hit_sized(lx, ly, window.rect.w) == Some(PetHit::BuffButton)
        {
            let name = self.keys.sysstrings.get(&pet_status::BUFF_BUTTON_TOOLTIP_SYSSTRING).cloned().unwrap_or_default();
            self.keys.tooltip = Some(killer_ui::tooltip::Tooltip { name, ..Default::default() });
        }
        dragging || scrolled || map_dragged || list_dragged || chat_resized
    }

    fn chat_resize_move(&mut self, x: i32, y: i32) -> bool {
        let Some(window) = self.desktop.window(WindowId::CHAT).filter(|w| w.visible) else { return self.chat.size_frame.dragging };
        let (lx, ly) = (x - window.rect.x, y - window.rect.y);
        let Some(step) = self.chat.size_move(lx, ly) else { return self.chat.size_frame.dragging };
        let height = self.chat.height;
        if let Some(window) = self.desktop.window_mut(WindowId::CHAT) {
            window.rect.y = window.rect.y.wrapping_sub(step);
            window.rect.h = height;
        }
        self.follow_chat();
        true
    }

    pub fn mouse_up(&mut self) -> Option<String> {
        self.mouse_up_in(&World::default())
    }

    pub fn mouse_up_in(&mut self, world: &World) -> Option<String> {
        if let Some((id, x, y, now)) = self.deferred_close.take() {
            let (cx, cy) = self.cursor;
            let inside = self.button_map.borrow().iter().any(|(window, button)| *window == id && button.close && button.contains(cx, cy));
            if inside {
                self.replaying_close = true;
                self.mouse_down_at(x, y, now, world);
                self.replaying_close = false;
            } else {
                self.close_pressed = None;
            }
        }
        self.button_held = None;
        if let Some(pressed) = self.close_pressed.take() {
            let (x, y) = self.cursor;
            if self.button_map.borrow().iter().any(|(id, button)| *id == pressed && button.close && button.contains(x, y)) {
                self.keys.sound(killer_ui::interface_sound::WINDOW_CLOSE);
            }
        }
        self.keys.multisell.view.quantity.mouse_up();
        if self.keys.minimized_mouse_up(&mut self.desktop) { return None; }
        self.status_resize.release();
        self.chat.tab_press = None;
        let dragged = self.desktop.dragging();
        self.desktop.end_drag();
        self.merge_dragged_tab(dragged);
        self.chat.size_release();
        for viewer in [self.npc_viewer.as_mut(), self.item_viewer.as_mut()].into_iter().flatten() {
            viewer.scroll_release();
            viewer.pressed = None;
            if let Some(window) = self.desktop.window(WindowId::NPC_HTML) {
                let (x, y) = self.cursor;
                viewer.combo_release(x - window.rect.x, y - window.rect.y);
            }
        }
        self.chat.scroll_pressed = None;
        self.chat.scroll_grab = None;
        self.system_msg.scroll_pressed = None;
        self.system_msg.scroll_grab = None;
        if std::mem::take(&mut self.rotate_pressed) && self.over_rotate_button(self.cursor.0, self.cursor.1) {
            self.rotate_shortcut();
        }
        if std::mem::take(&mut self.pad_pressed) && self.over_pad_button(self.cursor.0, self.cursor.1) {
            self.pad_shortcut();
        }
        if let Some(button) = self.page_pressed.take()
            && self.page_button_under(self.cursor.0, self.cursor.1) == Some(button)
        {
            self.shortcut.press_page_button(button);
            if self.shortcut.modern.is_some() && let Some(window) = self.desktop.window_mut(WindowId::SHORTCUT) {
                let (width, height) = self.shortcut.size();
                if let Some(position) = window.initial_position.as_mut() { position.y += height-window.rect.h; }
                window.rect.y = (window.rect.y + window.rect.h - height).max(0);
                window.rect.w = width;
                window.rect.h = height;
                window.grip = self.shortcut.grip();
            }
        }
        let over = self.window_under(self.cursor.0, self.cursor.1);
        if self.drop_shortcut_on_macro_line(over, world) {
            self.shortcut.cancel_drag();
        } else {
            let bar = self.desktop.window(WindowId::SHORTCUT).map_or(Rect::new(0, 0, 0, 0), |window| window.rect);
            self.shortcut.mouse_up(KeyWindows::bar_drop_target(over, self.cursor, bar));
        }
        self.keys.drain_bar(&mut self.shortcut, world);
        self.keys.mouse_up(over, &mut self.desktop, &mut self.shortcut, world);
        if std::mem::take(&mut self.keys.community_request) {
            self.open_community();
        }
        self.keys.petition.mouse_up(over, &self.small, &mut self.desktop);
        self.keys.snoop_mouse_up(over, &mut self.desktop);
        let outside = (i32::MIN / 2, i32::MIN / 2);
        let (ex, ey) = over.filter(|(id, _, _)| *id == WindowId::GM_EVENT).map_or(outside, |(_, x, y)| (x, y));
        let actions = self.keys.gm_event.mouse_up(ex, ey, &self.small);
        self.gm_event_actions(actions, world);
        let (fx, fy) = over.filter(|(id, _, _)| *id == WindowId::GM_FENCE).map_or(outside, |(_, x, y)| (x, y));
        if let Some(action) = self.keys.gm_fence.mouse_up(fx, fy) {
            self.gm_fence_action(action, world);
        }
        let (gx, gy) = over.filter(|(id, _, _)| *id == WindowId::GM).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        if let Some(action) = self.keys.gm.mouse_up(gx, gy) {
            self.keys.gm_action(action, &mut self.desktop);
        }
        let (ix, iy) = over.filter(|(id, _, _)| *id == WindowId::GM_INVENTORY).map_or(outside, |(_, x, y)| (x, y));
        if self.keys.gm_inventory.mouse_up(ix, iy) == Some(true) {
            self.show_window(WindowId::GM_INVENTORY, false);
        }
        let local = over.and_then(|(id, ox, oy)| self.desktop.window(id).map(|_| (id, ox, oy)));
        self.keys.messenger.mouse_up(local, &mut self.desktop);
        let (lx, ly) = over.filter(|(id, _, _)| *id == WindowId::GM_CLAN).map_or(outside, |(_, x, y)| (x, y));
        match self.keys.gm_clan.mouse_up(lx, ly) {
            killer_ui::gm_clan::GmClanRelease::Close => self.show_window(WindowId::GM_CLAN, false),
            killer_ui::gm_clan::GmClanRelease::Button(button) => self.keys.gm_clan_action(button, world, &mut self.desktop),
            killer_ui::gm_clan::GmClanRelease::None => {}
        }
        let (qx, qy) = over.filter(|(id, _, _)| *id == WindowId::GM_QUEST).map_or(outside, |(_, x, y)| (x, y));
        if self.keys.gm_quest.mouse_up(qx, qy) {
            self.show_window(WindowId::GM_QUEST, false);
        }
        let (kx, ky) = over.filter(|(id, _, _)| *id == WindowId::GM_SKILLS).map_or(outside, |(_, x, y)| (x, y));
        if self.keys.gm_skills.mouse_up(kx, ky) {
            self.show_window(WindowId::GM_SKILLS, false);
            self.keys.gm_skills.clear();
        }
        let (sx, sy) = over.filter(|(id, _, _)| *id == WindowId::GM_STATUS).map_or(outside, |(_, x, y)| (x, y));
        if self.keys.gm_status.mouse_up(sx, sy) {
            self.show_window(WindowId::GM_STATUS, false);
        }
        let (cx, cy) = over.filter(|(id, _, _)| *id == WindowId::CALC).map_or((i32::MIN / 2, i32::MIN / 2), |(_, x, y)| (x, y));
        if self.keys.calc.mouse_up(cx, cy, &self.small).is_some()
            && let Some(window) = self.desktop.window_mut(WindowId::CALC)
        {
            window.visible = false;
        }
        if let Some(path) = self.help.mouse_up(over, &self.small) {
            self.keys.load_help(path);
        }
        for command in self.tutorial.mouse_up(over, &mut self.desktop, &self.small) {
            self.keys.send(command);
        }
        if let Some((action, message)) = self.tutorial.link_question.take() {
            self.ask_html_link(action, &message, HtmlSource::Tutorial);
        }
        if std::mem::take(&mut self.tutorial.mail_click) {
            self.open_board_tab(killer_ui::tutorial::MAIL_BOARD_TAB, false);
        }
        if let Some(quest_id) = self.tutorial.quest_click.take() {
            self.keys.open_marked_quest(quest_id, &mut self.desktop);
        }
        let pressed = self.link_pressed.take()?;
        if let Some(released) = self.board_link_at(self.cursor.0, self.cursor.1)
            && released == pressed
            && !self.board.locked
            && let Some(result) = killer_ui::html_form::write_fields(&released, &self.board.html_form, &self.small)
        {
            match result {
                Ok(fields) => {
                    self.board.begin_link();
                    self.board.locked = true;
                    self.keys.send(killer_world::NetCommand::BbsWrite { fields });
                }
                Err(Some(id)) => self.keys.pending_message(id, killer_ui::html_form::MESSAGE_COLOR),
                Err(None) => {}
            }
            return None;
        }
        let html_released = [WindowId::ITEM_HTML, WindowId::NPC_HTML]
            .into_iter()
            .find_map(|id| self.html_link_at(id, self.cursor.0, self.cursor.1).map(|link| (id, link)));
        if let Some((html_id, released)) = html_released {
            if pressed != released {
                return None;
            }
            let (released, messages) = match self.html_viewer(html_id) {
                Some(viewer) => killer_ui::html_form::expand_action(&released, &viewer.form, &self.small),
                None => (released, Vec::new()),
            };
            for id in messages {
                self.keys.pending_message(id, killer_ui::html_form::MESSAGE_COLOR);
            }
            let source = if html_id == WindowId::ITEM_HTML { HtmlSource::Item } else { HtmlSource::Npc };
            if let Some(message) = self.html_link_message_at(html_id, self.cursor.0, self.cursor.1) {
                self.ask_html_link(released, &message, source);
                return None;
            }
            if killer_ui::html::split_message(&released).0.is_some() {
                return Some(released);
            }
            let is_button = self.html_viewer(html_id).is_some_and(|viewer| {
                let (x, y) = viewer.pointer.unwrap_or_default();
                viewer.button_at(x, y).is_some()
            });
            if !is_button {
                self.keys.sound(killer_ui::interface_sound::CLICK);
            }
            if html_action_result(&released, html_action(&released).is_some()) == 2 {
                self.close_html_window(html_id);
            }
            return Some(released);
        }
        let released = self.board_link_at(self.cursor.0, self.cursor.1)?;
        if pressed != released {
            return None;
        }
        let window = self.desktop.window(WindowId::BOARD).filter(|w| w.visible)?;
        if !self.board.begin_link() {
            return None;
        }
        if let Some(message) = self.board.link_message_at(self.cursor.0 - window.rect.x, self.cursor.1 - window.rect.y).map(str::to_string) {
            self.ask_html_link(released, &message, HtmlSource::Board);
            return None;
        }
        if !released.is_empty() {
            let result = html_action_result(&released, html_action(&released).is_some());
            if result == 2 {
                if let Some(window) = self.desktop.window_mut(WindowId::BOARD) {
                    window.visible = false;
                }
            } else {
                self.board.end_link(result);
            }
        }
        self.keys.sound(killer_ui::interface_sound::CLICK);
        Some(released)
    }

    /// Soltar um slot da barra de atalhos numa linha da lista de comandos do editor de macro é
    /// como um comando entra no macro. O `NCShortCutWnd::OnLButtonUp @ 0x100adf60` confere que a
    /// janela sob o cursor é o editor (`console + 0x154`, em `0x100ae171`), pega o índice da
    /// linha (`FUN_100a9fc0`, em `0x100ae18d`) e escreve nela o texto montado com o verbo do
    /// `commandname-e.dat` — o mesmo formato com que a janela mostra os comandos.
    fn drop_shortcut_on_macro_line(&mut self, over: Option<(WindowId, i32, i32)>, world: &World) -> bool {
        let Some(slot) = self.shortcut.pressed else { return false };
        let Some((WindowId::MACRO_EDIT, x, y)) = over else { return false };
        if !self.desktop.window(WindowId::MACRO_EDIT).is_some_and(|w| w.visible) {
            return false;
        }
        let Some(line) = self.keys.macros.edit.command_line_at(x, y) else {
            log::info!("macro: atalho {slot} solto no editor em ({x}, {y}), fora da lista de comandos");
            return false;
        };
        let Some(command) = self.keys.macro_command_from_shortcut(world, slot) else {
            log::info!("macro: atalho {slot} não virou comando (sem nome na tabela, ou texto grande demais)");
            return false;
        };
        log::info!("macro: atalho {slot} virou \"{}\" na linha {line}", command.label);
        self.keys.macros.edit.set_command(line, command)
    }

    fn party_hit_at(&self, rect: Rect, x: i32, y: i32) -> Option<PartyHit> {
        let cells = rect.h / party::CELL_STEP;
        let probe = PartyView {
            members: (0..cells)
                .map(|i| PartyMemberView {
                    object_id: i,
                    name: String::new(),
                    cp: (0, 0),
                    hp: (0, 0),
                    mp: (0, 0),
                    leader: false,
                    buffs: Vec::new(),
                })
                .collect(),
            show_buffs: false,
            loot_rule: 0,
        };
        probe.hit(x - rect.x, y - rect.y, rect.w)
    }

    pub fn party_released(&mut self, views: &WorldViews) -> Option<KeyAction> {
        let pressed = self.party_pressed.take()?;
        let window = self.desktop.window(WindowId::PARTY_STATUS)?;
        let released = views.party.hit(self.cursor.0 - window.rect.x, self.cursor.1 - window.rect.y, window.rect.w)?;
        if released != pressed {
            return None;
        }
        match released {
            PartyHit::BuffButton => {
                self.party_buffs_shown = !self.party_buffs_shown;
                Some(KeyAction::Consumed)
            }
            PartyHit::Member(_) | PartyHit::Grip => None,
        }
    }

    pub fn party_assist_at(&self, x: i32, y: i32, views: &WorldViews) -> Option<KeyAction> {
        let (id, lx, ly) = self.window_under(x, y)?;
        if id != WindowId::PARTY_STATUS || lx < party::CELL_X || ly < 0 {
            return None;
        }
        let index = usize::try_from(ly / party::CELL_STEP).ok()?;
        views.party.members.get(index).map(|member| KeyAction::Assist(member.object_id))
    }

    pub fn restart_released(&mut self, views: &WorldViews) -> Option<KeyAction> {
        let pressed = self.restart_pressed.take()?;
        let view = views.restart.as_ref()?;
        let window = self.desktop.window(WindowId::RESTART_MENU)?;
        let released = view.button_at(self.cursor.0 - window.rect.x, self.cursor.1 - window.rect.y)?;
        if released != pressed {
            return None;
        }
        Some(view.restart_point(released).map_or(KeyAction::Consumed, KeyAction::RestartPoint))
    }

    pub fn pet_released(&mut self, views: &WorldViews) -> Option<KeyAction> {
        let pressed = self.pet_pressed.take()?;
        let (id, _, _) = views.pet.as_ref()?;
        let window = self.desktop.window(*id)?;
        let released = pet_status::hit_sized(self.cursor.0 - window.rect.x, self.cursor.1 - window.rect.y, window.rect.w)?;
        if released != pressed {
            return None;
        }
        match released {
            PetHit::BuffButton => {
                self.pet_buffs_shown = !self.pet_buffs_shown;
                Some(KeyAction::Consumed)
            }
            PetHit::Select | PetHit::Grip => None,
        }
    }

    pub fn layout_modern_menu(&mut self) {
        if self.shortcut.modern.is_none() {return;}
        let (width,height)=if self.modern_menu_open {(224,268+if self.modern_system_open {168}else{0})} else {(112,36)};
        let (vw,vh)=self.desktop.size;
        if self.modern_menu_open && let Some(index)=self.desktop.windows.iter().position(|window|window.id==WindowId::MENU) {
            let window=self.desktop.windows.remove(index);
            self.desktop.windows.push(window);
        }
        if let Some(window)=self.desktop.window_mut(WindowId::MENU) {
            window.rect=Rect::new((vw-width-16).max(0),(vh-height-16).max(0),width,height);
            window.grip=Rect::new(0,0,0,0);
        }
    }

    fn modern_menu_hit(&self,x:i32,y:i32)->Option<usize> {
        let window=self.desktop.window(WindowId::MENU).filter(|window|window.visible && window.rect.contains(x,y))?;
        let y=y-window.rect.y;
        if y>=window.rect.h-36 {return (x>=window.rect.x+window.rect.w-112).then_some(6);}
        let exit_y=196+if self.modern_system_open {168}else{0};
        if (exit_y..exit_y+32).contains(&y) {return Some(13);}
        if self.modern_system_open && (196..364).contains(&y) {return Some(7+((y-196)/28) as usize);}
        (self.modern_menu_open && (4..196).contains(&y)).then(||((y-4)/32) as usize)
    }

    pub fn menu_released(&mut self, world: &World) -> Option<KeyAction> {
        if self.shortcut.modern.is_some() {
            let pressed=self.modern_menu_pressed.take()?;
            if self.modern_menu_hit(self.cursor.0,self.cursor.1)!=Some(pressed) {return None;}
            if pressed==6 {
                self.modern_menu_open = !self.modern_menu_open;
                if !self.modern_menu_open {self.modern_system_open=false;}
            }
            else if pressed==5 {self.modern_system_open = !self.modern_system_open;}
            else if let Some(button)=pressed.checked_sub(7).and_then(|index|killer_ui::sysmenu::SysMenuButton::ALL.get(index)).copied() {
                self.modern_menu_open=false;
                self.modern_system_open=false;
                self.keys.sysmenu_action(button,&mut self.desktop);
            }
            else {
                self.modern_menu_open=false;
            self.modern_system_open=false;
                let effect=match pressed {
                    0=>Some(Effect::ToggleMain(Some(killer_ui::keys::MainTab::Status))),
                    1=>Some(Effect::ToggleMain(Some(killer_ui::keys::MainTab::Skills))),
                    2=>Some(Effect::Inventory),
                    3=>Some(Effect::Minimap),
                    4=>{if let Some(window)=self.desktop.window_mut(WindowId::RADAR) {window.visible = !window.visible;}None},
                    5=>Some(Effect::SystemMenu),
                    _=>None,
                };
                if let Some(effect)=effect {self.apply_hooked(&Hooked {effects:vec![effect],handled:true},world);}
            }
            self.layout_modern_menu();
            return Some(KeyAction::Consumed);
        }
        let pressed = self.menu_pressed.take()?;
        let released = self.menu_button_at(self.cursor.0, self.cursor.1)?;
        if released != pressed {
            return None;
        }
        self.menu_button(pressed, world);
        Some(KeyAction::Consumed)
    }

    fn menu_button_at(&self, x: i32, y: i32) -> Option<MenuButton> {
        let window = self.desktop.window(WindowId::MENU).filter(|w| w.visible)?;
        menu::button_at(x - window.rect.x, y - window.rect.y)
    }

    pub fn wheel_over(&mut self, x: i32, y: i32, ticks: f32, chat: &[ChatEntry]) -> bool {
        if self.keys.minimized.hit(&self.desktop, x, y) { return true; }
        if let Some((id, lx, ly)) = self.window_under(x, y)
            && self.keys.handles(id)
        {
            return self.keys.wheel(id, lx, ly, (ticks * killer_ui::inventory::WHEEL_DELTA as f32).round() as i32);
        }
        if let Some((WindowId::SHORTCUT, lx, ly)) = self.window_under(x, y)
            && self.shortcut.auto_wheel(lx, ly, ticks.round() as i32)
        {
            return true;
        }
        let under=self.desktop.window_at(x,y);
        if self.shortcut.modern.is_none() && under==Some(WindowId::CHAT) {
            self.chat.scroll_by(ticks.round() as i32);
            return true;
        }
        if self.shortcut.modern.is_some() && matches!(under,Some(WindowId::CHAT | WindowId::SYSTEM_MSG)) {
            let system=under==Some(WindowId::SYSTEM_MSG);
            let (count,height,scroll)=if system {
                (self.modern_chat_rows.get()[1],killer_ui::sysmsg::SYSTEM_MSG_SIZE.1,&mut self.system_msg.scroll)
            } else {
                (self.modern_chat_rows.get()[0],self.chat.height,&mut self.chat.scroll)
            };
            let max=crate::ui_modern::chat_scroll_limit(count,height,system);
            *scroll=(*scroll as i64+i64::from(ticks.round() as i32)).clamp(0,max as i64) as usize;
            return true;
        }
        match under {
            Some(id @ (WindowId::NPC_HTML | WindowId::ITEM_HTML)) => {
                if let Some(viewer) = self.html_viewer_slot(id).as_mut() {
                    viewer.scroll_by((-ticks * HTML_WHEEL_STEP as f32).round() as i32);
                }
                true
            }
            Some(WindowId::HELP) => {
                self.help.wheel((-ticks * HTML_WHEEL_STEP as f32).round() as i32);
                true
            }
            Some(WindowId::BOARD) => {
                self.board.scroll_by((-ticks * HTML_WHEEL_STEP as f32).round() as i32);
                true
            }
            Some(WindowId::SYSTEM_MSG) => {
                self.system_msg.scroll_by(chat, &self.small, ticks.round() as i32);
                true
            }
            Some(id) if ChatView::split_tab_of(id.0).is_some() => {
                if let Some(tab) = ChatView::split_tab_of(id.0) {
                    let rows = self.chat.rows(&self.chat.lists[tab], &self.small).len();
                    self.chat.split_scroll_by(tab, ticks.round() as i32, rows);
                }
                true
            }
            Some(WindowId::TUTORIAL_VIEWER) => {
                self.tutorial.viewer.scroll_by((-ticks * killer_ui::tutorial::HTML_WHEEL_STEP as f32).round() as i32);
                true
            }
            _ => false,
        }
    }

    fn page_button_under(&self, x: i32, y: i32) -> Option<PageButton> {
        let window = self.desktop.window(WindowId::SHORTCUT)?;
        self.shortcut.page_button_at(x - window.rect.x, y - window.rect.y)
    }

    fn over_rotate_button(&self, x: i32, y: i32) -> bool {
        !self.shortcut.pad_mode
            && self.desktop
                .window(WindowId::SHORTCUT)
                .is_some_and(|window| self.shortcut.rotate_button().offset(window.rect.x, window.rect.y).contains(x, y))
    }

    fn over_pad_button(&self, x: i32, y: i32) -> bool {
        (self.shortcut.joypad_visible || self.shortcut.pad_mode)
            && self.desktop
                .window(WindowId::SHORTCUT)
                .is_some_and(|window| self.shortcut.joypad_button().offset(window.rect.x, window.rect.y).contains(x, y))
    }

    pub fn pad_shortcut(&mut self) {
        if let Some(window) = self.desktop.window_mut(WindowId::SHORTCUT)
            && let Some(rect) = self.shortcut.pad_clicked(window.rect)
        {
            window.rect = rect;
            window.grip = self.shortcut.grip();
        }
    }

    pub fn rotate_shortcut(&mut self) {
        if let Some(window) = self.desktop.window_mut(WindowId::SHORTCUT) {
            window.rect = self.shortcut.rotate(window.rect);
            window.grip = self.shortcut.grip();
        }
    }

    pub fn resized(&mut self, size: (i32, i32)) {
        self.desktop.rearrange(size);
        self.keys.broadcast.set_viewport(size);
        self.keys.debug.resize(size.0);
        if let Some(window) = self.desktop.window_mut(killer_ui::debug_wnd::ID) {
            window.rect.w = size.0;
        }
        self.keys.quest.screen = size;
        self.keys.quest_info.screen = size;
        self.keys.messenger.set_viewport(size);
        self.keys.broadcast.layout(&mut self.desktop);
        self.keys.minimized.sync_owner(&self.desktop);
        self.follow_chat();
    }

    pub fn chat_focused(&self) -> bool {
        self.chat.focused
    }

    pub fn unfocus_chat(&mut self) {
        self.chat.focused = false;
    }

    pub fn key(&mut self, key: InputKey) -> KeyAction {
        if let Some(id) = [WindowId::ITEM_HTML, WindowId::NPC_HTML].into_iter().find(|id| {
            self.desktop.window(*id).is_some_and(|w| w.visible) && self.html_viewer(*id).is_some_and(|viewer| viewer.form.focus.is_some())
        }) {
            return self.npc_form_key(id, key);
        }
        if self.board.html_form.focus.is_some() && self.desktop.window(WindowId::BOARD).is_some_and(|w| w.visible) {
            if key == InputKey::Escape {
                self.board.html_form.focus = None;
            } else {
                self.board.html_form.key(key, &self.small);
            }
            return KeyAction::Consumed;
        }
        if self.keys.party_room_key(key, &self.small, &mut self.desktop) { return KeyAction::Consumed; }
        if self.keys.command_channel_key(key, &self.small) { return KeyAction::Consumed; }
        if self.macro_key(key) { return KeyAction::Consumed; }
        if self.keys.multisell.key(key, &self.small, &self.desktop) { return KeyAction::Consumed; }
        if self.keys.manor_setting.key(key, &self.small, &self.desktop) { return KeyAction::Consumed; }
        if self.keys.hero_tower.key(key, &self.small, &self.desktop) { return KeyAction::Consumed; }
        if self.keys.petition.key(key, &self.small, &mut self.desktop) { return KeyAction::Consumed; }
        if !self.chat.focused && self.keys.broadcast.key(key, &mut self.desktop) { return KeyAction::Consumed; }
        if self.keys.messenger.key(key, &self.desktop, &self.small) { return KeyAction::Consumed; }
        if self.desktop.window(WindowId::GM_FENCE).is_some_and(|window| window.visible) && self.keys.gm_fence.key(key, &self.small) { return KeyAction::Consumed; }
        if self.desktop.window(WindowId::GM_EVENT).is_some_and(|window| window.visible) && self.keys.gm_event.key(key, &self.small) { return KeyAction::Consumed; }
        if self.desktop.window(WindowId::GM).is_some_and(|window| window.visible) && self.keys.gm.key(key, &self.small) { return KeyAction::Consumed; }
        if self.desktop.window(WindowId::CALC).is_some_and(|window| window.visible) && self.keys.calc.key(key, &self.small) { return KeyAction::Consumed; }
        if let Some(dialog) = self.dialog.as_mut() {
            if key == InputKey::Enter {
                let accept = dialog.view.enter_button() == DialogButton::First;
                return self.finish_dialog(accept);
            }
            if let Some(input) = dialog.view.input.as_mut() {
                match key {
                    InputKey::Char(c) => input.push(c),
                    InputKey::Backspace => { input.text.pop(); }
                    _ => {}
                }
                return KeyAction::Consumed;
            }
        }
        if !self.chat.focused {
            if let InputKey::Char(c) = key
                && !self.keys.enter_chatting
                && !self.keys.alt
                && !self.keys.ctrl
                && c.is_ascii()
                && !c.is_ascii_control()
                && !"wasdfhWASDFH".contains(c)
            {
                self.chat.focused = true;
                self.chat.press(key);
                return KeyAction::Consumed;
            }
            if key != InputKey::Enter {
                return KeyAction::Ignored;
            }
            self.chat.focused = true;
            return KeyAction::Consumed;
        }
        let sent = self.chat.press(key);
        match key {
            InputKey::Enter => match sent {
                Some(text) => {
                    if self.keys.enter_chatting {
                        self.chat.focused = false;
                    }
                    return KeyAction::Say(text);
                }
                None => self.chat.focused = false,
            },
            InputKey::Escape => self.chat.focused = false,
            _ => {}
        }
        KeyAction::Consumed
    }
}

pub fn input_keys(key: &Key) -> Vec<InputKey> {
    match key {
        Key::Named(NamedKey::Enter) => vec![InputKey::Enter],
        Key::Named(NamedKey::Backspace) => vec![InputKey::Backspace],
        Key::Named(NamedKey::ArrowLeft) => vec![InputKey::Left],
        Key::Named(NamedKey::ArrowRight) => vec![InputKey::Right],
        Key::Named(NamedKey::Home) => vec![InputKey::Home],
        Key::Named(NamedKey::End) => vec![InputKey::End],
        Key::Named(NamedKey::Delete) => vec![InputKey::Delete],
        Key::Named(NamedKey::ArrowUp) => vec![InputKey::Up],
        Key::Named(NamedKey::ArrowDown) => vec![InputKey::Down],
        Key::Named(NamedKey::Escape) => vec![InputKey::Escape],
        Key::Named(NamedKey::Space) => vec![InputKey::Char(' ')],
        Key::Character(text) => text.chars().map(|c| if matches!(c, '\n' | '\r') { InputKey::Enter } else { InputKey::Char(c) }).collect(),
        _ => Vec::new(),
    }
}


pub fn status_view(me: &Entity) -> StatusView {
    StatusView {
        regen: 0.0,
        name: me.name.clone(),
        level: me.level,
        hp: me.hp.unwrap_or((0, 0)),
        mp: me.mp.unwrap_or((0, 0)),
        cp: me.cp.unwrap_or((0, 0)),
        exp: i64::from(me.exp),
    }
}

pub fn crest_texture(crest_id: i32) -> String {
    format!("{CREST_PACKAGE}.crest_{crest_id}")
}

pub fn ally_crest_texture(crest_id: i32) -> String {
    format!("{CREST_PACKAGE}.ally_{crest_id}")
}

pub fn static_target_view(world: &World, tables: Option<&DatTables>) -> Option<TargetView> {
    let target = world.targeted_static()?;
    let name = tables
        .and_then(|tables| tables.staticobject.get(&u32::try_from(target.static_object_id).ok()?))
        .cloned()
        .unwrap_or_default();
    let kind = if target.hp.is_some() { TargetKind::Door } else { TargetKind::Npc };
    Some(TargetView::new(name, kind, target.hp, None))
}

pub fn target_view(world: &World, entity: &Entity, name: String, tables: Option<&DatTables>) -> TargetView {
    let npc = |npc_id: i32| tables.and_then(|t| u32::try_from(npc_id).ok().and_then(|id| t.npcgrp.get(id)));
    let own_pet = world.pet.as_ref().is_some_and(|pet| pet.info.object_id == entity.object_id);
    let kind = match (&entity.kind, world.me.as_ref()) {
        (_, Some(me)) if me.object_id == entity.object_id => TargetKind::Me,
        (EntityKind::Player { .. }, _) => TargetKind::Player,
        (EntityKind::Npc { .. }, _) if own_pet => TargetKind::Barred,
        (EntityKind::Npc { npc_id, .. }, _) if npc(*npc_id).is_some_and(|record| record.shows_bars != 0) => TargetKind::Barred,
        (EntityKind::Npc { attackable: true, .. }, _) => TargetKind::Monster,
        (EntityKind::Npc { .. } | EntityKind::Item { .. }, _) => TargetKind::Npc,
    };
    let mut view = TargetView::new(name, kind, entity.hp, entity.mp);
    if let EntityKind::Npc { npc_id, .. } = &entity.kind {
        view.skill_icons = npc(*npc_id)
            .map(|record| record.skills.iter().filter_map(|&skill| skill_icon(tables, i32::try_from(skill).ok()?, 1)).collect())
            .unwrap_or_default();
    }
    if entity.clan_id > 0 {
        view.clan = world.pledge_name(entity.clan_id).map(str::to_string);
        view.ally = world.ally_name(entity.clan_id).map(str::to_string);
        view.clan_crest = (entity.clan_crest_id > 0 && world.pledge_window.crests.contains_key(&entity.clan_crest_id))
            .then(|| crest_texture(entity.clan_crest_id));
        view.ally_crest = (entity.ally_crest_id > 0 && world.pledge_window.ally_crests.contains_key(&entity.ally_crest_id))
            .then(|| ally_crest_texture(entity.ally_crest_id));
    }
    view
}

pub fn chat_entry(line: &ChatLine) -> ChatEntry {
    ChatEntry {
        kind: line.kind,
        name: line.sender.clone(),
        text: line.text.clone(),
        group: line.group,
        color: line.color.map(ChatColor),
    }
}

pub fn is_notice(tables: &DatTables, message: &SystemMessage) -> bool {
    u32::try_from(message.id)
        .ok()
        .and_then(|id| tables.systemmsg.get(&id))
        .is_some_and(|r| r.group == NOTICE_SYSMSG_GROUP)
}

pub fn system_sound(tables: &DatTables, message: &SystemMessage) -> Option<String> {
    let record = tables.systemmsg.get(&u32::try_from(message.id).ok()?)?;
    (!record.sound.is_empty() && record.sound != "None").then(|| record.sound.clone())
}

pub fn system_voice(tables: &DatTables, message: &SystemMessage) -> Option<String> {
    let record = tables.systemmsg.get(&u32::try_from(message.id).ok()?)?;
    (!record.second_sound.is_empty() && record.second_sound != "None").then(|| record.second_sound.clone())
}

pub const BAD_WORD_SYSSTRING: u32 = 740;

pub fn say_line(tables:&DatTables,message:&l2_net::game::Say2)->Option<ChatLine> {
    let (sender,text)=match &message.body {
        l2_net::game::Say2Body::Text {name,text}=>(name.clone(),text.clone()),
        l2_net::game::Say2Body::Tables {sys_string_id,sys_msg_id}=>{
            let sender=u32::try_from(*sys_string_id).ok().and_then(|id|tables.sysstring.get(&id));
            let text=u32::try_from(*sys_msg_id).ok().and_then(|id|tables.systemmsg.get(&id));
            let (Some(sender),Some(text))=(sender,text) else {
                log::warn!("SayPacket2: tabelas ausentes sysstring={sys_string_id}, systemmsg={sys_msg_id}");
                return None;
            };
            (sender.clone(),text.text.clone())
        },
    };
    let text=l2_dat::obscene::mask(&tables.obscene,&text,tables.sysstring.get(&BAD_WORD_SYSSTRING).map_or("",String::as_str)).0;
    let sender=if message.kind==SAY2_ANNOUNCEMENT {tables.sysstring.get(&ANNOUNCEMENT_SYSSTRING).cloned().unwrap_or_default()} else {sender};
    Some(ChatLine::new(message.kind,sender,text))
}

pub const SAY2_ANNOUNCEMENT: i32 = 10;
pub const ANNOUNCEMENT_SYSSTRING: u32 = 328;
pub const SAY2_PETITION_GM: i32 = 7;

pub fn npc_say_line(tables: &DatTables, say: &l2_net::game::NpcSay) -> Option<ChatLine> {
    if say.text.is_empty() {
        return None;
    }
    let name = u32::try_from(say.npc_id).ok().and_then(|id| tables.npcname.get(&id)).map_or_else(|| "(null)".to_string(), |npc| npc.name.clone());
    let (kind, color, name) = match say.kind {
        1 => (1, 0xFFFF7200, name),
        2 => (2, 0xFFFF00FF, name),
        3 => (3, 0xFF00FF00, name),
        4 => (4, 0xFF7D77FF, name),
        8 => (8, 0xFFEAA5F5, name),
        9 => (9, 0xFF77FF99, name),
        6 | 7 => (say.kind, 0xFF80FFFF, name),
        SAY2_ANNOUNCEMENT => (SAY2_PETITION_GM, 0xFF80FFFF, tables.sysstring.get(&ANNOUNCEMENT_SYSSTRING).cloned().unwrap_or_default()),
        15 => (SAY2_PETITION_GM, 0xFFFFE87B, name),
        16 | 18 => (SAY2_PETITION_GM, 0xFF7B7DF2, name),
        _ => (0, 0xFFDCDCDC, name),
    };
    let mut line = ChatLine::new(kind, name, say.text.clone());
    line.color = Some(color);
    Some(line)
}

pub fn channel_say_text(tables: Option<&DatTables>, event: &killer_world::WorldEvent) -> Option<killer_world::WorldEvent> {
    let killer_world::WorldEvent::Say2(message) = event else { return None };
    if !matches!(message.kind, killer_ui::command_channel::SAY_ALL_MEMBERS | killer_ui::command_channel::SAY_LEADER | killer_ui::party_room::SAY_PARTY_ROOM) {
        return None;
    }
    if !matches!(message.body, l2_net::game::Say2Body::Tables { .. }) {
        return None;
    }
    let line = say_line(tables?, message)?;
    Some(killer_world::WorldEvent::Say2(l2_net::game::Say2::text(message.object_id, message.kind, &line.sender, &line.text)))
}

pub const SYSTEM_MESSAGE_COUNT: i32 = 3000;
pub const UNKNOWN_SYSTEM_MESSAGE_COLOR: u32 = 0xFFDCDCDC;
const PARTICLE_I: char = '\u{C774}';
const PARTICLE_EUL: char = '\u{C744}';
const PARTICLE_EUN: char = '\u{C740}';
const PARTICLE_WA: char = '\u{C640}';

pub fn system_line(tables: &DatTables, message: &SystemMessage) -> Option<ChatLine> {
    let record = u32::try_from(message.id)
        .ok()
        .filter(|_| message.id < SYSTEM_MESSAGE_COUNT)
        .and_then(|id| tables.systemmsg.get(&id))
        .filter(|record| record.present != 0);
    let Some(record) = record else {
        let text = format!("\u{C54C} \u{C218} \u{C5C6}\u{B294} System Message format : {}", message.id);
        return Some(ChatLine { kind: CHAT_SYSTEM, sender: String::new(), text, group: 0, color: Some(UNKNOWN_SYSTEM_MESSAGE_COLOR), system: None, tables: None });
    };
    let params: Vec<Option<String>> = message.args.iter().map(|arg| argument(tables, arg)).collect();
    let text = format_system_message(&record.text, &params);
    if text.is_empty() {
        return None;
    }
    Some(ChatLine { kind: CHAT_SYSTEM, sender: String::new(), text, group: record.group, color: Some(record.argb()), system: None, tables: None })
}

fn has_no_final_consonant(c: char) -> bool {
    let c = c as u32;
    !(0xAC00..=0xD7A3).contains(&c) || ((c - 0xAC00) % (21 * 28)).is_multiple_of(28)
}

pub fn format_system_message(format: &str, params: &[Option<String>]) -> String {
    let format: Vec<char> = format.chars().collect();
    let at = |index: usize| format.get(index).copied().unwrap_or('\0');
    let mut out = String::new();
    let mut last: Option<usize> = None;
    let mut i = 0;
    while i < format.len() {
        let c = format[i];
        let is_particle = matches!(at(i + 1), PARTICLE_I | PARTICLE_EUL | PARTICLE_EUN | PARTICLE_WA);
        if (c != '$' && c != '%') || (c == '%' && !is_particle) {
            out.push(c);
            i += 1;
        } else if c == '$' {
            last = (at(i + 2) as u32).checked_sub('1' as u32).map(|index| index as usize);
            if let Some(Some(text)) = last.and_then(|index| params.get(index)) {
                out.push_str(text);
            }
            i += 3;
        } else {
            let particle = at(i + 1);
            match last {
                None => {
                    out.push(match particle {
                        PARTICLE_I => '\u{C774}',
                        PARTICLE_EUL => '\u{B97C}',
                        PARTICLE_EUN => '\u{B294}',
                        _ => '\u{C640}',
                    });
                    i += 2;
                }
                Some(index) => match params.get(index) {
                    Some(Some(text)) => {
                        let no_final = has_no_final_consonant(text.chars().last().unwrap_or('\0'));
                        out.push(match (particle, no_final) {
                            (PARTICLE_I, true) => '\u{AC00}',
                            (PARTICLE_I, false) => '\u{C774}',
                            (PARTICLE_EUL, true) => '\u{B97C}',
                            (PARTICLE_EUL, false) => '\u{C744}',
                            (PARTICLE_EUN, true) => '\u{B294}',
                            (PARTICLE_EUN, false) => '\u{C740}',
                            (_, true) => '\u{C640}',
                            (_, false) => '\u{ACFC}',
                        });
                        i += 2;
                    }
                    _ => i += 1,
                },
            }
        }
    }
    out
}

fn argument(tables: &DatTables, arg: &SystemMessageArg) -> Option<String> {
    match arg {
        SystemMessageArg::Text(text) => Some(text.clone()),
        SystemMessageArg::Number(n) => Some(n.to_string()),
        SystemMessageArg::NpcName(id) => u32::try_from(id.wrapping_sub(NPC_NAME_BASE))
            .ok()
            .and_then(|npc_id| tables.npcname.get(&npc_id))
            .map(|npc| npc.name.clone()),
        SystemMessageArg::ItemName(id) => u32::try_from(*id).ok().and_then(|id| tables.itemname.get(&id)).map(|item| item.name.clone()),
        SystemMessageArg::SkillName { id, level } => tables
            .skillname
            .get(&(u32::try_from(*id).unwrap_or_default(), u32::try_from(*level).unwrap_or_default()))
            .map(|skill| skill.name.clone()),
        SystemMessageArg::CastleName(id) => u32::try_from(*id)
            .ok()
            .and_then(|id| tables.castlename.get(&id))
            .filter(|castle| castle.present != 0 && castle.index < 100)
            .map(|castle| castle.name.clone()),
        SystemMessageArg::Unknown { .. } => None,
    }
}

pub const PRIMITIVE_POINT_HALF: f32 = 2.0;

pub fn paint_primitive_mark(list: &mut DrawList, mark: &PrimitiveMark, font: &BitmapFont) {
    if !mark.text.is_empty() {
        let width = font.measure(&mark.text) as i32;
        let x = (mark.screen[0] - (width / 2) as f32) as i32;
        let y = (mark.screen[1] - font.line_height as f32) as i32;
        list.draw_text(x, y, mark.color, &mark.text, font);
    }
    if mark.point {
        let left = (mark.screen[0] - PRIMITIVE_POINT_HALF) as i32;
        let top = (mark.screen[1] - PRIMITIVE_POINT_HALF) as i32;
        let side = (PRIMITIVE_POINT_HALF * 2.0) as i32;
        list.draw_fill(left, top, side, side, mark.color);
    }
}

pub fn paint_item_label(list: &mut DrawList, item: &ItemLabel, font: &BitmapFont) {
    let width = font.measure(&item.text) as i32;
    let x = (item.screen[0] - (width / 2) as f32) as i32;
    let y = (item.screen[1] - font.line_height as f32) as i32;
    list.draw_text(x, y, ITEM_LABEL_COLOR, &item.text, font);
}

fn rgb(color: [u8; 3]) -> Argb {
    Argb(0xFF00_0000 | (u32::from(color[0]) << 16) | (u32::from(color[1]) << 8) | u32::from(color[2]))
}

pub fn load_named_font(client: &mut Client, texture: &str, file: &str) -> Result<BitmapFont, AssetError> {
    load_font(client, &(texture.to_string(), file.to_string()))
}

fn load_font(client: &mut Client, font: &(String, String)) -> Result<BitmapFont, AssetError> {
    let bytes = client.read_system_file(&font.1)?;
    let glyphs = Glyphs::parse(&bytes).map_err(|e| AssetError::Package(format!("{}: {e}", font.1)))?;
    Ok(BitmapFont::new(font.0.clone(), glyphs))
}

fn optional_ini(client: &mut Client, name: &str) -> IniFile {
    match client.read_system_text(name) {
        Ok(text) => parse_ini(&text),
        Err(e) => {
            log::warn!("{name}: {e}; usando os padrões");
            parse_ini("")
        }
    }
}

pub fn dialog_command(answer: DialogAnswer, accept: bool) -> Option<NetCommand> {
    let command = match answer {
        DialogAnswer::Siege(decision) => {
            if !accept { return None; }
            match decision {
                killer_world::state::SiegeDecision::Join { castle_id, attacker, join } => NetCommand::JoinCastleSiege { castle_id, attacker, join },
                killer_world::state::SiegeDecision::Approve { castle_id, pledge_id, approved } => NetCommand::ConfirmCastleSiegeWaitingList { castle_id, pledge_id, approved },
                killer_world::state::SiegeDecision::SetTime { castle_id, time } => NetCommand::SetCastleSiegeTime { castle_id, time },
            }
        }
        DialogAnswer::JoinPledge => NetCommand::AnswerJoinPledge { accept },
        DialogAnswer::JoinParty => NetCommand::AnswerJoinParty { accept },
        DialogAnswer::JoinAlliance => NetCommand::AnswerJoinAlly { accept },
        DialogAnswer::JoinMpcc => NetCommand::ExAcceptJoinMpcc { accept },
        DialogAnswer::Confirm { message_id } => NetCommand::DlgAnswer { message_id, answer: i32::from(accept) },
        DialogAnswer::Notice | DialogAnswer::HtmlLink { .. } | DialogAnswer::RecipeShop | DialogAnswer::Shop | DialogAnswer::ShopPreview | DialogAnswer::PrivateShop | DialogAnswer::Warehouse | DialogAnswer::GmStore | DialogAnswer::ManorShop | DialogAnswer::Delivery | DialogAnswer::TradeCount | DialogAnswer::MultiSellCount | DialogAnswer::ManorSetting => return None,
        DialogAnswer::Trade => NetCommand::AnswerTrade { accept },
        DialogAnswer::Friend => NetCommand::AnswerFriendInvite { accept },
        DialogAnswer::UseItem { object_id } => {
            if !accept {
                return None;
            }
            NetCommand::UseItem { object_id, ctrl: false }
        }
        DialogAnswer::OustPledgeMember { name } => {
            if !accept {
                return None;
            }
            NetCommand::OustPledgeMember { name }
        }
        DialogAnswer::AbortQuest { quest_id } => {
            if !accept {
                return None;
            }
            NetCommand::QuestAbort { quest_id }
        }
        DialogAnswer::SystemMenu { restart } => {
            if !accept {
                return None;
            }
            if restart { NetCommand::Restart } else { NetCommand::Logout }
        }
        DialogAnswer::Disconnected => return None,
        DialogAnswer::GmConfirm { command } => {
            if !accept {
                return None;
            }
            NetCommand::BypassBuildCmd { command }
        }
        DialogAnswer::WithdrawPledge => {
            if !accept {
                return None;
            }
            NetCommand::WithdrawPledge
        }
        DialogAnswer::GiveTitle { .. } | DialogAnswer::PledgeWarName { .. } | DialogAnswer::AllyDismissName | DialogAnswer::CrestNotice { .. } | DialogAnswer::CrestFile { .. } => return None,
        DialogAnswer::AllyDismiss { pledge_name } => {
            if !accept {
                return None;
            }
            NetCommand::AllyDismiss { pledge_name }
        }
        DialogAnswer::DeleteTitle { target } => {
            if !accept {
                return None;
            }
            NetCommand::GiveNickName { target, title: String::new() }
        }
        DialogAnswer::DeleteCrest => {
            if !accept {
                return None;
            }
            NetCommand::SetPledgeCrest { data: Vec::new() }
        }
        DialogAnswer::DeleteInsignia => {
            if !accept {
                return None;
            }
            NetCommand::ExSetPledgeCrestLarge { data: Vec::new() }
        }
        DialogAnswer::PledgeWar { pledge_name, start } => {
            if !accept {
                return None;
            }
            match start {
                true => NetCommand::StartPledgeWar { pledge_name },
                false => NetCommand::StopPledgeWar { pledge_name },
            }
        }
        DialogAnswer::DropItem { object_id, count, position, kind } => {
            if !accept {
                return None;
            }
            match kind {
                killer_world::ItemDropKind::Drop => NetCommand::DropItem { object_id, count, position },
                killer_world::ItemDropKind::Destroy => NetCommand::DestroyItem { object_id, count },
                killer_world::ItemDropKind::Crystallize => NetCommand::CrystallizeItem { object_id, count },
                killer_world::ItemDropKind::GiveToPet => NetCommand::GiveItemToPet { object_id, count },
                killer_world::ItemDropKind::GetFromPet => NetCommand::GetItemFromPet { object_id, count },
            }
        }
        DialogAnswer::PetName => return None,
        DialogAnswer::DeleteRecipe { recipe_id } => {
            if !accept {
                return None;
            }
            NetCommand::RecipeItemDelete { recipe_id }
        }
        DialogAnswer::DeleteMacro { id } => {
            if !accept {
                return None;
            }
            NetCommand::DeleteMacro { id }
        }
        DialogAnswer::AllyConfirm { dissolve } => {
            if !accept {
                return None;
            }
            match dissolve {
                true => NetCommand::DismissAlly,
                false => NetCommand::AllyLeave,
            }
        }
    };
    Some(command)
}


#[cfg(test)]
pub(crate) fn fixture_ui(size: (i32, i32)) -> GameUi {
    tests::game_ui(size)
}

#[cfg(test)]
pub(crate) mod tests {
    #[test]
    fn modern_system_submenu_keeps_the_toggle_fixed_and_opens_options() {
        let mut ui=game_ui((800,600));
        ui.shortcut.modern=Some(Default::default());
        ui.layout_modern_menu();
        let closed=ui.desktop.window(WindowId::MENU).unwrap().rect;
        let point=(closed.x+closed.w-20,closed.y+closed.h-18);
        ui.mouse_down(point.0,point.1);
        ui.menu_released(&world());
        assert_eq!(ui.modern_menu_hit(point.0,point.1),Some(6));
        let rect=ui.desktop.window(WindowId::MENU).unwrap().rect;
        ui.mouse_down(rect.x+30,rect.y+4+5*32+16);
        ui.menu_released(&world());
        assert!(ui.modern_menu_open && ui.modern_system_open);
        assert_eq!(ui.desktop.windows.last().unwrap().id,WindowId::MENU);
        assert_eq!(ui.desktop.window_at(rect.x+30,rect.y+20),Some(WindowId::MENU));
        assert_eq!(ui.modern_menu_hit(point.0,point.1),Some(6));
        let rect=ui.desktop.window(WindowId::MENU).unwrap().rect;
        ui.mouse_down(rect.x+40,rect.y+196+4*28+14);
        ui.menu_released(&world());
        assert!(ui.desktop.window(WindowId::OPTION).unwrap().visible);
        assert!(!ui.modern_menu_open && !ui.modern_system_open);
        ui.menu_button(MenuButton::System,&world());
        assert!(ui.modern_menu_open && ui.modern_system_open);
        assert!(!ui.desktop.window(WindowId::SYSTEM_MENU).unwrap().visible);
        for system in [false,true] {
            ui.modern_menu_open=true;
            ui.modern_system_open=system;
            ui.layout_modern_menu();
            let rect=ui.desktop.window(WindowId::MENU).unwrap().rect;
            ui.mouse_down(rect.x+30,rect.y+196+if system {168+16}else{16});
            ui.menu_released(&world());
            assert_eq!(ui.keys.dialogs.pop(),Some(killer_world::Dialog::SystemMenu{restart:false}));
            assert!(!ui.modern_menu_open);
        }
    }

    #[test]
    fn modern_menu_uses_real_actions_and_closes_after_selection() {
        let mut ui=game_ui((800,600));
        ui.shortcut.modern=Some(Default::default());
        ui.layout_modern_menu();
        let world=world();
        for row in [2,4] {
            let rect=ui.desktop.window(WindowId::MENU).unwrap().rect;
            assert!(ui.mouse_down(rect.x+20,rect.y+rect.h-18));
            assert_eq!(ui.menu_released(&world),Some(KeyAction::Consumed));
            assert!(ui.modern_menu_open);
            let rect=ui.desktop.window(WindowId::MENU).unwrap().rect;
            let id=if row==2 {WindowId::INVENTORY}else{WindowId::RADAR};
            let before=ui.desktop.window(id).unwrap().visible;
            assert!(ui.mouse_down(rect.x+20,rect.y+4+row*32+16));
            assert_eq!(ui.menu_released(&world),Some(KeyAction::Consumed));
            assert!(!ui.modern_menu_open);
            if row==2 {assert!(ui.take_commands().iter().any(|command|matches!(command,UiCommand::Net(NetCommand::RequestItemList))));}
            else {assert_ne!(ui.desktop.window(id).unwrap().visible,before);}
            assert_eq!(ui.desktop.window(WindowId::MENU).unwrap().rect.h,36);
        }
    }

    #[test]
    #[ignore]
    fn the_relation_icon_textures_exist_in_the_original_client_with_room_for_16_by_12() {
        use crate::characters::*;
        let root = std::env::var_os("KILLER_CLIENT_DIR").unwrap();
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let mut bank = killer_assets::textures::TextureBank::new();
        for name in [
            WAR_SINGLE_ICON, WAR_DOUBLE_ICON, BLUE_SHIELD_ICON, BLUE_SWORD_ICON, BLUE_LEADER_ICON,
            BLUE_ATTACK_LEADER_ICON, RED_SHIELD_ICON, RED_SWORD_ICON, RED_LEADER_ICON, RED_ATTACK_LEADER_ICON,
        ] {
            bank.texture(client.catalog(), name);
            let [w, h] = bank.size(client.catalog(), name);
            assert!(w >= RELATION_ICON_WIDTH as f32 && h >= RELATION_ICON_HEIGHT as f32, "{name} {w}x{h}");
        }
        assert_eq!(bank.missing().collect::<Vec<_>>(), Vec::<&str>::new());
    }

    #[test]
    #[ignore]
    fn macro_boxes_and_store_signs_render_with_original_client_assets() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let ui = GameUi::load(&mut client, (1024, 480)).unwrap();
        let mut view = killer_ui::macros::MacroEditView::new(|id| tables.sysstring.get(&id).cloned());
        view.name.set_text("Evasion");
        view.acronym.set_text("EV");
        for line in 0..12 {
            view.set_command(line, killer_ui::macros::MacroCommandView { label: format!("/delay {}", line + 1), ..Default::default() });
        }
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &ui.small, 1.0);
        view.scroll_commands(-3);
        view.paint(&mut list, 256, 0, &ui.small, 1.0);
        for (index, kind) in [1, 3, 5, 7, 8].into_iter().enumerate() {
            let (id, color, background) = killer_ui::store_sign::style(kind, true, 0).unwrap();
            let prefix = tables.sysstring.get(&id).unwrap();
            killer_ui::store_sign::StoreSign { text: if kind == 7 { prefix.clone() } else { format!("{prefix}\nLoja de materiais") }, color, background, screen: [750.0, 55.0 + index as f32 * 80.0] }.paint(&mut list, &ui.small);
        }
        let mut bank = ui.texture_bank;
        for quad in &list.quads { bank.texture(client.catalog(), &quad.texture); }
        assert!(bank.missing().next().is_none(), "{:?}", bank.missing().collect::<Vec<_>>());
        let mut source = killer_assets::textures::BankSource { bank: &mut bank, catalog: client.catalog() };
        let mut headless = killer_render::headless::Headless::new(1024, 480).unwrap();
        headless.renderer_mut().set_ui(&list, &mut source);
        let camera = killer_render::camera::Camera { eye: glam::Vec3::new(0.0, -100.0, 50.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 1000.0 };
        let rgba = headless.render(&camera, &killer_render::environment::Environment::default()).unwrap();
        killer_assets::textures::write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-macro-store-audit.png"), 1024, 480, &rgba).unwrap();
    }
    #[test]
    #[ignore]
    fn janelas_ref_a1_capture_the_windows_changed_in_this_pass() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let ui = GameUi::load(&mut client, (1024, 480)).unwrap();
        let lookup = |id: u32| tables.sysstring.get(&id).cloned();
        let mut list = DrawList::default();

        let mut train = killer_ui::train::TrainView::new(lookup);
        let row = |level: i32, name: &str| killer_ui::train::LearnableView {
            enchant_description: None, id: 1, level, sp_cost: 1200, icon: "NWindow.BlackTexture".into(), name: name.into(),
        };
        train.set_list(vec![row(0, "Not learned"), row(3, "Learned"), row(0, "Not learned 2")]);
        train.player_sp = Some(12345);
        train.paint(&mut list, 0, 0, &ui.small, 1.0);

        let mut macro_edit = killer_ui::macros::MacroEditView::new(lookup);
        macro_edit.name.set_text("Evasion");
        macro_edit.acronym.set_text("EV");
        for (line, text) in ["/useskill Chant", "/delay 3", "/useshortcut 2 3", "//gmmenu"].into_iter().enumerate() {
            macro_edit.set_command(line, killer_ui::macros::MacroCommandView { label: text.into(), ..Default::default() });
        }
        macro_edit.paint(&mut list, 266, 0, &ui.small, 1.0);

        let mut multisell = killer_ui::multisell::MultiSellView::new(lookup);
        let offer = |stackable| killer_ui::multisell::OfferView {
            item: killer_ui::item_grid::GridEntry { icon: "NWindow.BlackTexture".into(), count: 1, ..Default::default() }, stackable, ..Default::default()
        };
        multisell.set_offers(1, vec![offer(false), offer(true)]);
        multisell.select(0);
        multisell.paint(&mut list, 0, 420, &ui.small, 1.0, &lookup);

        let pet = killer_ui::pet_status::PetStatusView {
            name: "Wolf".into(), hp: (50, 100), mp: (30, 60),
            third: killer_ui::pet_status::ThirdBar::Fed { fed: 25, max_fed: 100 },
            buffs: (0..12).map(|_| Some("NWindow.BlackTexture".to_string())).collect(), show_buffs: true,
        };
        killer_ui::pet_status::paint_pet_status_sized(&mut list, 550, 20, 372, &pet, &ui.small, 1.0);

        let mut bank = ui.texture_bank;
        for quad in &list.quads { bank.texture(client.catalog(), &quad.texture); }
        let mut source = killer_assets::textures::BankSource { bank: &mut bank, catalog: client.catalog() };
        let mut headless = killer_render::headless::Headless::new(1024, 900).unwrap();
        headless.renderer_mut().set_ui(&list, &mut source);
        let camera = killer_render::camera::Camera { eye: glam::Vec3::new(0.0, -100.0, 50.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 1000.0 };
        let rgba = headless.render(&camera, &killer_render::environment::Environment::default()).unwrap();
        killer_assets::textures::write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-ref-a1-janelas.png"), 1024, 900, &rgba).unwrap();
    }

    #[test]
    fn settings_frame_uses_loaded_resources_native_dimensions_and_secondary_root_alpha() {
        use killer_ui::option::{BACK, HEADER, TAB_VIDEO};
        let mut ui = game_ui((1024, 768));
        ui.keys.option.press_tab(TAB_VIDEO);
        let window = ui.desktop.window_mut(WindowId::OPTION).unwrap();
        window.visible = true;
        window.rect = Rect { x: 31, y: 47, w: 120, h: 200 };
        let paint = |ui: &GameUi| {
            ui.frame(&World::default(), &[], &[], None, &WorldViews::default()).quads
                .into_iter().filter(|q| q.texture == HEADER || q.texture == BACK).collect::<Vec<_>>()
        };
        assert!(paint(&ui).is_empty());
        for path in [HEADER, BACK] {
            ui.texture_bank.insert(path, killer_assets::textures::TextureData {
                width: 1, height: 1, format: killer_assets::textures::TextureFormatKind::Rgba8,
                mips: vec![vec![255; 4]],
            });
        }
        ui.alpha = 0.25;
        let quads = paint(&ui);
        assert_eq!(quads.len(), 2);
        assert_eq!([quads[0].x, quads[0].y, quads[0].w, quads[0].h], [31.0, 67.0, 120.0, 28.0]);
        assert_eq!([quads[1].x, quads[1].y, quads[1].w, quads[1].h], [31.0, 95.0, 120.0, 152.0]);
        assert!(quads.iter().all(|q| q.color[3] == 1.0));
    }

    #[test]
    #[ignore]
    fn settings_backgrounds_are_loaded_before_paint_and_shared_without_fallback() {
        let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let mut ui = super::GameUi::load(&mut client, (1024, 768)).unwrap();
        let tables = DatTables {
            sysstring: l2_dat::sysstring::read(&client.read_system_file("sysstring-e.dat").unwrap()).unwrap()
                .into_iter().map(|row| (row.id, row.text)).collect(),
            ..DatTables::default()
        };
        ui.set_tab_names(&tables);
        let mut renderer_bank = killer_assets::textures::TextureBank::new();
        for (path, data) in ui.texture_bank.textures() {
            assert_eq!(ui.texture_bank.texture_available(&path), Some(true), "{path}");
            renderer_bank.insert_shared(&path, data);
        }
        assert_eq!(ui.texture_bank.textures().count(), 4);
        for (path, data) in ui.texture_bank.textures() {
            let renderer_data = renderer_bank.texture(client.catalog(), &path);
            assert!(std::sync::Arc::ptr_eq(&data, &renderer_data));
            assert!(data.width > 0 && data.height > 0 && !data.mips.is_empty());
        }
        for window in &mut ui.desktop.windows {
            window.visible = window.id == WindowId::OPTION;
        }
        let mut list = DrawList::default();
        for (page, background) in [killer_ui::option::BACK, killer_ui::option::AUDIO_BACK, killer_ui::option::GAME_BACK].into_iter().enumerate() {
            ui.keys.option.press_tab(page);
            ui.desktop.window_mut(WindowId::OPTION).unwrap().rect = Rect { x: page as i32 * 256, y: 0, w: 256, h: 480 };
            let frame = ui.frame(&World::default(), &[], &[], None, &WorldViews::default());
            let body: Vec<_> = frame.quads.iter().filter(|q| q.texture == background).collect();
            assert_eq!(body.len(), 1);
            assert_eq!([body[0].x, body[0].y, body[0].w, body[0].h, body[0].color[3]], [page as f32 * 256.0, 48.0, 256.0, 432.0, 1.0]);
            list.quads.extend(frame.quads);
        }
        let mut headless = killer_render::headless::Headless::new(768, 480).unwrap();
        let mut source = killer_assets::textures::BankSource { bank: &mut renderer_bank, catalog: client.catalog() };
        headless.renderer_mut().set_ui(&list, &mut source);
        assert!(renderer_bank.missing().next().is_none());
        let camera = killer_render::camera::Camera {
            eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO,
            fov_x_deg: 60.0, near: 1.0, far: 5000.0,
        };
        let rgba = headless.render(&camera, &killer_render::environment::Environment::default()).unwrap();
        killer_assets::textures::write_png(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-option-app.png"),
            768, 480, &rgba,
        ).unwrap();
    }

    use super::*;
    use killer_ui::abnormal::{ABNORMAL_BACK, DEBUFF_OVERLAY};
    use killer_ui::chat::{CHAT_WIDTH, CHATTING_BACK2};
    use killer_ui::color::{TEXT, chat_color};
    use killer_ui::draw::Quad;
    use killer_ui::font::Glyphs;
    use killer_ui::frame::SMALLBAR_1;
    use killer_ui::npc_viewer::NPC_HTML_BACK;
    use killer_ui::pet_status::{FATIGUE_BAR, FATIGUE_BAR_BACK, PET_BUTTON_DOWN};
    use killer_ui::shortcut::SHORTCUT_BACKV;
    use killer_ui::status::{PS_HPBAR, PS_LEVELBACK};
    use killer_ui::target::{HP_BAR, HP_BAR_BACK, MP_BAR_BACK};
    use killer_world::Pet;
    use killer_world::world::WHITE_NAME_COLOR;
    use killer_world::{EntityKind, World};
    use l2_dat::{NpcName, Skill, SystemMessage as SystemMessageRecord};
    use l2_net::game::packets::{Buff, PartyBuff, PetInfo, SUMMON_SERVITOR, ShortBuffStatusUpdate};
    use l2_net::game::packets::{CHAT_ALL, CHAT_SHOUT, VISIBLE_EQUIP_SLOTS};
    use l2_net::game::{SystemMessage, SystemMessageArg};
    use killer_platform::time::Duration;
    use winit::keyboard::{Key, NamedKey};

    const WINDOWS_INFO: &str = include_str!("../../../crates/killer-ui/tests/fixtures/windowsinfo.ini");
    const OPTION: &str = include_str!("../../../crates/killer-ui/tests/fixtures/option.ini");
    const CHAT_FILTER: &str = include_str!("../../../crates/killer-ui/tests/fixtures/chatfilter.ini");

    fn gly() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend(1024u32.to_le_bytes());
        b.extend(128u32.to_le_bytes());
        b.extend(1u32.to_le_bytes());
        b.extend(0x20u32.to_le_bytes());
        b.extend(95u32.to_le_bytes());
        for i in 0..95i32 {
            let width = if i == 0 { 3 } else { 6 };
            for v in [i * 8, width, 0, 13] {
                b.extend(v.to_le_bytes());
            }
        }
        b
    }

    pub(crate) fn font(atlas: &str) -> BitmapFont {
        BitmapFont::new(atlas.into(), Glyphs::parse(&gly()).unwrap())
    }

    pub(crate) fn small_font() -> BitmapFont {
        font("L2Font-e.SmallFont-e")
    }

    pub(crate) fn small_font_for_lobby() -> BitmapFont {
        small_font()
    }

    pub(crate) fn game_ui(size: (i32, i32)) -> GameUi {
        GameUi::new(
            font("L2Font-e.SmallFont-e"),
            font("L2Font-e.LargeFont-e"),
            Options::from_ini(&parse_ini(OPTION)),
            WindowsInfo::from_ini(&parse_ini(WINDOWS_INFO)),
            ChatFilter::from_ini(&parse_ini(CHAT_FILTER)),
            size,
        ).unwrap()
    }

    #[test]
    fn load_window_positions_10073530_skips_the_chat_and_restores_the_item_html_window() {
        let ini = format!("[{}]\nposX=300\nposY=400\nHeight=200\n[{}]\nposX=321\nposY=123\n", WindowId::CHAT.0, WindowId::ITEM_HTML.0);
        let ui = GameUi::new(
            small_font(), font("L2Font-e.LargeFont-e"), Options::default(),
            WindowsInfo::from_ini(&parse_ini(&ini)),
            ChatFilter::from_ini(&parse_ini(CHAT_FILTER)), (1558, 933),
        ).unwrap();
        let chat = ui.desktop.window(WindowId::CHAT).unwrap().rect;
        assert_eq!((chat.x, chat.y + chat.h), (0, 933));
        assert_eq!(chat.h, ui.chat.size().1);
        let item = ui.desktop.window(WindowId::ITEM_HTML).unwrap().rect;
        assert_eq!((item.x, item.y), (321, 123));
    }

    #[test]
    fn commerce_creation_descriptors_preserve_saved_positions_and_private_shop_starts_on_right() {
        for saved in [false, true] {
            let ini = if saved {
                "[21]\nposX=123\nposY=234\n[18]\nposX=123\nposY=234\n[24]\nposX=123\nposY=234\n[40]\nposX=123\nposY=234\n[23]\nposX=123\nposY=234\n[33]\nposX=123\nposY=234\n[11]\nposX=123\nposY=234\n"
            } else {
                ""
            };
            let ui = GameUi::new(
                small_font(), font("L2Font-e.LargeFont-e"), Options::default(),
                WindowsInfo::from_ini(&parse_ini(ini)),
                ChatFilter::from_ini(&parse_ini(CHAT_FILTER)), (1558, 933),
            ).unwrap();
            for (id, default, descriptor) in [
                (WindowId::SHOP, (0, 214), killer_ui::shop::initial_position((1558, 933))),
                (WindowId::WAREHOUSE, (80, 156), killer_ui::shop::WAREHOUSE_INITIAL_POSITION),
                (WindowId::TRADE, (1256, 214), killer_ui::trade::initial_position((1558, 933))),
                (WindowId::PRIVATE_SHOP, (1256, 214), killer_ui::shop::private_initial_position((1558, 933))),
                (WindowId::MULTISELL, (0, 214), killer_ui::multisell::initial_position((1558, 933))),
                (WindowId::TRAIN, (0, 214), killer_ui::train::initial_position((1558, 933))),
                (WindowId::ITEM_ENCHANT, (1256, 214), killer_ui::enchant_wnd::initial_position((1558, 933))),
            ] {
                let window = ui.desktop.window(id).unwrap();
                assert_eq!((window.rect.x, window.rect.y), if saved { (123, 234) } else { default });
                assert_eq!(window.initial_position, Some(descriptor));
                assert!(!window.visible);
            }
        }
    }

    #[test]
    fn the_command_channel_window_opens_with_mpcc_and_takes_type_15_and_16_chat() {
        use l2_net::game::{MultiPartyChannelInfo, MultiPartyChannelMember, Say2};
        use killer_ui::command_channel as channel;
        use killer_world::WorldEvent;
        let mut ui = game_ui((1558, 933));
        let window = ui.desktop.window(channel::WINDOW_ID).unwrap();
        assert!(!window.visible);
        assert_eq!((window.rect.x, window.rect.y, window.rect.w, window.rect.h), (151, 182, 348, 187));
        assert_eq!(window.initial_position, Some(channel::INITIAL_POSITION));
        let world = world();
        ui.world_event(&WorldEvent::OpenMpcc, &world);
        assert!(ui.desktop.window(channel::WINDOW_ID).unwrap().visible);
        ui.world_event(&WorldEvent::MultiPartyChannelInfo(MultiPartyChannelInfo {
            name: "Chefe".into(),
            values: [11, 2],
            members: vec![MultiPartyChannelMember { name: "Ana".into(), value: 9 }],
        }), &world);
        assert_eq!((ui.keys.command_channel.members, ui.keys.command_channel.parties), (11, 2));
        ui.world_event(&WorldEvent::CloseMpcc, &world);
        assert!(!ui.desktop.window(channel::WINDOW_ID).unwrap().visible);
        ui.world_event(&WorldEvent::Say2(Say2::text(1, channel::SAY_ALL_MEMBERS, "Ana", "oi")), &world);
        assert!(ui.desktop.window(channel::WINDOW_ID).unwrap().visible);
        assert_eq!(ui.keys.command_channel.chat.lines().len(), 1);
        assert_eq!(ui.keys.command_channel.chat.lines()[0].text, "Ana: oi");
    }

    #[test]
    fn party_room_packets_buttons_chat_and_hide_ui_reach_the_network() {
        use killer_ui::party_room as room;
        use killer_world::{NetCommand, WorldEvent};
        use l2_net::game::{ManagePartyRoomMember, PartyMatchDetail, PartyRoomMember, PartyRoomMemberEntry, Say2};
        let mut ui = game_ui((1558, 933));
        let world = world();
        let net = |ui: &mut GameUi| ui.take_commands().into_iter().filter_map(|command| match command { UiCommand::Net(net) => Some(net), _ => None }).collect::<Vec<_>>();
        ui.desktop.window_mut(WindowId::PARTY_MATCH).unwrap().visible = true;
        ui.world_event(&WorldEvent::PartyMatchDetail(PartyMatchDetail {
            id: 7, max_members: 12, min_level: 1, max_level: 78, loot_type: 2, location: 3, title: "Sala".into(),
        }), &world);
        assert!(ui.desktop.window(WindowId::PARTY_ROOM).unwrap().visible);
        assert!(!ui.desktop.window(WindowId::PARTY_MATCH).unwrap().visible);
        assert_eq!(ui.keys.party_room.info.room_id, 7);
        let entry = |id: i32, role: i32| PartyRoomMemberEntry { id, name: format!("m{id}"), values: [1, 40, 1, role] };
        ui.world_event(&WorldEvent::PartyRoomMember(PartyRoomMember { header: [1, 2], entries: vec![entry(1, 1), entry(5, 2)] }), &world);
        assert_eq!(ui.keys.party_room.members.len(), 2);
        ui.world_event(&WorldEvent::ManagePartyRoomMember(ManagePartyRoomMember { header: [2, 5], name: String::new(), values: [0; 4] }), &world);
        assert_eq!(ui.keys.party_room.members.len(), 1);
        ui.world_event(&WorldEvent::ManagePartyRoomMember(ManagePartyRoomMember { header: [0, 6], name: "m6".into(), values: [1, 30, 1, 2] }), &world);
        let rect = ui.desktop.window(WindowId::PARTY_ROOM).unwrap().rect;
        let row_y = rect.y + room::ROOM_LIST_RECT.y + room::ROOM_HEADER_HEIGHT + room::ROOM_ROW_HEIGHT + 2;
        assert!(ui.mouse_down(rect.x + 20, row_y));
        ui.mouse_up();
        assert!(ui.mouse_down(rect.x + 380, rect.y + 410));
        ui.mouse_up();
        assert!(ui.mouse_down(rect.x + 450, rect.y + 410));
        ui.mouse_up();
        assert_eq!(net(&mut ui), vec![
            NetCommand::OustFromPartyRoom { object_id: 6 },
            NetCommand::JoinParty { name: "m6".into(), item_distribution: 2 },
        ]);
        ui.world_event(&WorldEvent::Say2(Say2::text(3, room::SAY_PARTY_ROOM, "Ana", "oi")), &world);
        assert_eq!(ui.keys.party_room.chat.lines()[0].text, "Ana: oi");
        let hooked = killer_ui::keys::Hooked { effects: vec![killer_ui::keys::Effect::HideUi], handled: true };
        let mut bar = killer_ui::shortcut::ShortcutView::new(true);
        ui.keys.apply_effects(&hooked, &mut ui.desktop, &mut bar, &world);
        assert_eq!(net(&mut ui), vec![NetCommand::DismissPartyRoom { room_id: 7 }]);
        ui.keys.apply_effects(&hooked, &mut ui.desktop, &mut bar, &world);
        assert_eq!(net(&mut ui), vec![NetCommand::DismissPartyRoom { room_id: 7 }]);
        assert!(!ui.keys.hidden);
        assert!(ui.mouse_down(rect.x + 530, rect.y + 410));
        ui.mouse_up();
        assert_eq!(net(&mut ui), vec![NetCommand::DismissPartyRoom { room_id: 7 }]);
        assert!(!ui.desktop.window(WindowId::PARTY_ROOM).unwrap().visible);
        ui.desktop.window_mut(WindowId::PARTY_ROOM).unwrap().visible = true;
        ui.world_event(&WorldEvent::ClosePartyRoom, &world);
        assert!(!ui.desktop.window(WindowId::PARTY_ROOM).unwrap().visible);
        assert!(ui.keys.party_room.chat.lines().is_empty());
        ui.keys.apply_effects(&hooked, &mut ui.desktop, &mut bar, &world);
        assert!(net(&mut ui).is_empty());
    }

    #[test]
    fn party_room_minimizes_to_its_own_icon_blinks_and_restores() {
        use killer_ui::party_room as room;
        use killer_world::WorldEvent;
        use l2_net::game::{PartyMatchDetail, Say2};
        let mut ui = game_ui((1558, 933));
        let world = world();
        let detail = WorldEvent::PartyMatchDetail(PartyMatchDetail { id: 7, max_members: 12, min_level: 1, max_level: 78, loot_type: 2, location: 3, title: "Sala".into() });
        ui.world_event(&detail, &world);
        let rect = ui.desktop.window(WindowId::PARTY_ROOM).unwrap().rect;
        assert!(ui.mouse_down(rect.x + 565, rect.y + 10));
        ui.mouse_up();
        assert!(!ui.desktop.window(WindowId::PARTY_ROOM).unwrap().visible);
        let icon = ui.keys.minimized.party_room.rect;
        assert!(ui.keys.minimized.party_room.visible);
        assert!(!ui.keys.minimized.party_match.visible);
        ui.world_event(&detail, &world);
        assert!(!ui.desktop.window(WindowId::PARTY_ROOM).unwrap().visible);
        assert!(ui.keys.minimized.party_room.blinking);
        ui.world_event(&WorldEvent::Say2(Say2::text(3, room::SAY_PARTY_ROOM, "Ana", "oi")), &world);
        assert_eq!(ui.keys.minimized.party_room.timers.len(), 2);
        ui.keys.minimized.tick(600);
        assert!(!ui.keys.minimized.party_room.blink_phase);
        assert!(ui.mouse_down(icon.x + 5, icon.y + 5));
        ui.mouse_up();
        assert!(ui.desktop.window(WindowId::PARTY_ROOM).unwrap().visible);
        assert!(!ui.keys.minimized.party_room.visible);
        assert_eq!(ui.keys.minimized.party_room.timers.len(), 1);
    }

    #[test]
    fn channel_and_room_chat_by_tables_is_resolved_for_their_windows_and_971_is_the_long_text() {
        use killer_world::WorldEvent;
        use l2_net::game::{Say2, Say2Body};
        let mut tables = DatTables::default();
        tables.sysstring.insert(5, "Ana".into());
        let record = |id: u32, text: &str| l2_dat::systemmsg::SystemMessage { id, present: 1, text: text.into(), group: 0, color: [0; 3], alpha: 255, sound: String::new(), second_sound: String::new() };
        tables.systemmsg.insert(9, record(9, "oi"));
        tables.systemmsg.insert(971, record(971, "longo demais"));
        let tabled = |kind: i32| WorldEvent::Say2(Say2 { object_id: 1, kind, body: Say2Body::Tables { sys_string_id: 5, sys_msg_id: 9 } });
        for kind in [14, 15, 16] {
            let resolved = channel_say_text(Some(&tables), &tabled(kind)).unwrap();
            assert_eq!(resolved, WorldEvent::Say2(Say2::text(1, kind, "Ana", "oi")));
        }
        assert!(channel_say_text(Some(&tables), &tabled(2)).is_none());
        assert!(channel_say_text(None, &tabled(15)).is_none());
        assert!(channel_say_text(Some(&tables), &WorldEvent::Say2(Say2::text(1, 15, "A", "b"))).is_none());
        let mut ui = game_ui((1558, 933));
        ui.keys.set_sysstrings(&tables);
        assert_eq!(ui.keys.command_channel.too_long, "longo demais");
    }

    #[test]
    fn command_channel_edit_box_chat_and_scroll_are_reached_through_the_desktop() {
        use killer_ui::command_channel as channel;
        let mut ui = game_ui((1558, 933));
        let world = world();
        ui.world_event(&killer_world::WorldEvent::OpenMpcc, &world);
        let rect = ui.desktop.window(channel::WINDOW_ID).unwrap().rect;
        assert!(ui.mouse_down(rect.x + channel::INPUT_RECT.x + 3, rect.y + channel::INPUT_RECT.y + 3));
        ui.mouse_up();
        assert!(ui.keys.command_channel.edit.focused);
        ui.keys.command_channel.edit.set_text("oi");
        assert_eq!(ui.key(killer_ui::chat::InputKey::Enter), KeyAction::Consumed);
        assert!(ui.take_commands().contains(&UiCommand::Net(killer_world::NetCommand::Say { text: "oi".into(), kind: channel::SAY_LEADER, target: String::new() })));
        for index in 0..12 {
            ui.world_event(&killer_world::WorldEvent::Say2(l2_net::game::Say2::text(1, channel::SAY_ALL_MEMBERS, "Ana", &format!("linha {index}"))), &world);
        }
        assert_eq!(ui.keys.command_channel.chat.lines().len(), 12);
        let first = ui.keys.command_channel.chat.first();
        assert!(ui.wheel_over(rect.x + channel::CHAT_RECT.x + 20, rect.y + channel::CHAT_RECT.y + 20, 1.0, &[]));
        assert!(ui.keys.command_channel.chat.first() != first || first == 0);
    }

    #[test]
    fn command_channel_reaches_paint_and_mouse_dispatch_in_both_themes() {
        use killer_ui::command_channel as channel;
        for modern in [false, true] {
            let mut ui = game_ui((1558, 933));
            for window in &mut ui.desktop.windows { window.visible = false; }
            ui.shortcut.modern = modern.then(Default::default);
            let world = world();
            ui.world_event(&killer_world::WorldEvent::OpenMpcc, &world);
            let window = ui.desktop.window(channel::WINDOW_ID).unwrap();
            let rect = window.rect;
            let mut draw = DrawList { capture_text: modern, ..Default::default() };
            ui.paint_window(&mut draw, window, &world, &[], None, &WorldViews::default());
            assert!(draw.quads.iter().any(|quad| quad.texture == channel::BACK));
            let tab = channel::TAB_RECTS[1];
            assert!(ui.mouse_down(rect.x + tab.x + 2, rect.y + tab.y + 2));
            ui.mouse_up();
            assert_eq!(ui.keys.command_channel.tab, 1);
            assert!(ui.take_commands().contains(&UiCommand::Net(killer_world::NetCommand::UserCommand {
                command: channel::USER_COMMAND_REFRESH,
            })));
        }
    }

    #[test]
    fn every_key_window_with_paint_reaches_the_application_dispatcher() {
        let ui = game_ui((1558, 933));
        let world = world();
        let mut checked = 0;
        for window in &ui.desktop.windows {
            if window.id == WindowId::OPTION { continue; }
            let mut direct = DrawList::default();
            ui.keys.paint(window.id, &mut direct, window.rect.x, window.rect.y, &world, &ui.small, ui.alpha);
            if direct.quads.is_empty() { continue; }
            let mut routed = DrawList::default();
            ui.paint_window(&mut routed, window, &world, &[], None, &WorldViews::default());
            assert_eq!(direct.quads, routed.quads, "janela {:?} perdida no dispatcher", window.id);
            checked += 1;
        }
        assert!(checked > 0);
    }

    #[test]
    #[ignore]
    fn command_channel_capture_uses_app_dispatch_and_original_textures() {
        use killer_ui::command_channel as channel;
        let root = std::env::var_os("KILLER_CLIENT_DIR").unwrap();
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let mut ui = game_ui((720, 220));
        ui.small = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap());
        ui.set_tab_names(&tables);
        for window in &mut ui.desktop.windows { window.visible = false; }
        let world = world();
        ui.world_event(&killer_world::WorldEvent::OpenMpcc, &world);
        let small = ui.small.clone();
        ui.keys.command_channel.receive_chat(channel::SAY_ALL_MEMBERS, "Ana: Ready.", &small);
        ui.keys.command_channel.set_info(&channel::ChannelInfo {
            owner: "Chefe".into(), members: 18, parties: 2,
            rows: vec![("Ana".into(), 9), ("Pedro".into(), 9)],
        });
        let mut draw = DrawList::default();
        for tab in 0..2 {
            ui.keys.command_channel.tab = tab;
            ui.keys.command_channel.info_visible = tab == 1;
            ui.keys.command_channel.chat_visible = tab == 0;
            ui.desktop.window_mut(channel::WINDOW_ID).unwrap().rect = Rect::new(6 + tab as i32 * 360, 12, channel::SIZE.0, channel::SIZE.1);
            let frame = ui.frame(&world, &[], &[], None, &WorldViews::default());
            assert!(frame.quads.iter().any(|quad| quad.texture == channel::BACK));
            draw.quads.extend(frame.quads);
        }
        let mut gpu = killer_render::headless::Headless::new(720, 220).unwrap();
        let mut bank = killer_assets::textures::TextureBank::new();
        gpu.renderer_mut().set_ui(&draw, &mut killer_assets::textures::BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(bank.missing().count(), 0);
        let camera = killer_render::camera::Camera {
            eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO,
            fov_x_deg: 60.0, near: 1.0, far: 5000.0,
        };
        let rgba = gpu.render(&camera, &killer_render::environment::Environment::default()).unwrap();
        killer_assets::textures::write_png(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-command-channel-app.png"),
            720, 220, &rgba,
        ).unwrap();
    }

    #[test]
    fn macro_creation_descriptors_are_separate_from_saved_positions_and_visibility() {
        for saved in [false, true] {
            let ini = if saved {
                "[29]\nposX=17\nposY=123\n[30]\nposX=18\nposY=124\n[31]\nposX=19\nposY=125\n"
            } else {
                ""
            };
            let ui = GameUi::new(
                small_font(), font("L2Font-e.LargeFont-e"), Options::default(),
                WindowsInfo::from_ini(&parse_ini(ini)),
                ChatFilter::from_ini(&parse_ini(CHAT_FILTER)), (1558, 933),
            ).unwrap();
            for (id, default, saved_position, descriptor) in [
                (WindowId::MACRO_LIST, (0, 214), (17, 123), killer_ui::macros::list_initial_position((1558, 933))),
                (WindowId::MACRO_INFO, (20, 294), (18, 124), killer_ui::macros::info_initial_position((1558, 933))),
                (WindowId::MACRO_EDIT, (0, 214), (19, 125), killer_ui::macros::edit_initial_position((1558, 933))),
            ] {
                let window = ui.desktop.window(id).unwrap();
                assert_eq!((window.rect.x, window.rect.y), if saved { saved_position } else { default });
                assert_eq!(window.initial_position, Some(descriptor));
                assert!(!window.visible);
            }
        }
    }

    #[test]
    fn henna_and_party_room_creation_preserve_ini_and_room_bottom_is_interactive() {
        for saved in [false, true] {
            let ini = if saved {
                "[51]\nposX=123\nposY=234\n[52]\nposX=123\nposY=234\n[35]\nposX=123\nposY=234\n[36]\nposX=123\nposY=234\n"
            } else {
                ""
            };
            let mut ui = GameUi::new(
                small_font(), font("L2Font-e.LargeFont-e"), Options::default(),
                WindowsInfo::from_ini(&parse_ini(ini)),
                ChatFilter::from_ini(&parse_ini(CHAT_FILTER)), (1558, 933),
            ).unwrap();
            for (id, default, descriptor) in [
                (WindowId::HENNA_LIST, (0, 214), killer_ui::henna::initial_position((1558, 933))),
                (WindowId::HENNA_INFO, (0, 214), killer_ui::henna::initial_position((1558, 933))),
                (WindowId::PARTY_ROOM_MAKING, (601, 396), killer_ui::party_room::MAKING_INITIAL_POSITION),
                (WindowId::PARTY_ROOM, (0, 76), killer_ui::party_room::ROOM_INITIAL_POSITION),
            ] {
                let window = ui.desktop.window(id).unwrap();
                assert_eq!((window.rect.x, window.rect.y), if saved { (123, 234) } else { default });
                assert_eq!(window.initial_position, Some(descriptor));
                assert!(!window.visible);
            }
            let room = ui.desktop.window_mut(WindowId::PARTY_ROOM).unwrap();
            room.visible = true;
            let rect = room.rect;
            assert_eq!((rect.w, rect.h), (600, 431));
            assert!(rect.contains(rect.x + 530, rect.y + 422));
            assert_eq!(ui.desktop.window_at(rect.x + 530, rect.y + 422), Some(WindowId::PARTY_ROOM));
            assert_eq!(ui.keys.party_room.hit(530, 422), Some(killer_ui::party_room::RoomHit::Leave));
        }
    }

    #[test]
    fn hud_initial_descriptors_preserve_saved_widths_positions_and_dynamic_party_height() {
        let mut ui = GameUi::new(
            small_font(), font("L2Font-e.LargeFont-e"), Options::default(),
            WindowsInfo::from_ini(&parse_ini("[14]\nposX=300\nposY=100\n[15]\nposX=310\nposY=110\nwidth=202\n[16]\nposX=320\nposY=120\n[17]\nposX=330\nposY=130\n")),
            ChatFilter::from_ini(&parse_ini(CHAT_FILTER)), (1558, 933),
        ).unwrap();
        for (id, position, descriptor) in [
            (WindowId::ABNORMAL_STATUS, (300, 100), killer_ui::abnormal::ABNORMAL_INITIAL_POSITION),
            (WindowId::PARTY_STATUS, (310, 110), killer_ui::party::PARTY_INITIAL_POSITION),
            (WindowId::PET_STATUS, (320, 120), killer_ui::pet_status::PET_STATUS_INITIAL_POSITION),
            (WindowId::SUMMONED_STATUS, (330, 130), killer_ui::pet_status::PET_STATUS_INITIAL_POSITION),
        ] {
            let window = ui.desktop.window(id).unwrap();
            assert_eq!((window.rect.x, window.rect.y), position);
            assert_eq!(window.initial_position, Some(descriptor));
        }
        let party = ui.desktop.window_mut(WindowId::PARTY_STATUS).unwrap();
        assert_eq!((party.rect.w, party.rect.h), (172, 0));
        party.rect.h = 46 * 8;
        assert_eq!(party.initial_position, Some(killer_ui::party::PARTY_INITIAL_POSITION));
    }

    #[test]
    fn npc_and_menus_capture_creation_independently_of_saved_positions() {
        for saved in [false, true] {
            let ini = if saved { "[7]\nposX=900\nposY=200\n[25]\nposX=700\nposY=300\n" } else { "" };
            let ui = GameUi::new(
                small_font(), font("L2Font-e.LargeFont-e"), Options::default(),
                WindowsInfo::from_ini(&parse_ini(ini)),
                ChatFilter::from_ini(&parse_ini(CHAT_FILTER)), (1558, 933),
            ).unwrap();
            for (id, expected, descriptor, visible) in [
                (WindowId::NPC_HTML, if saved { (900, 200) } else { (0, 214) }, killer_ui::npc_viewer::initial_position((1558, 933)), false),
                (WindowId::MENU, if saved { (700, 300) } else { (1385, 887) }, killer_ui::menu::initial_position((1558, 933)), true),
                (crate::ui_keys::SYSMENU_WINDOW, (1386, 592), killer_ui::sysmenu::initial_position((1558, 933)), false),
            ] {
                let window = ui.desktop.window(id).unwrap();
                assert_eq!((window.rect.x, window.rect.y), expected);
                assert_eq!(window.initial_position, Some(descriptor));
                assert_eq!(window.visible, visible);
            }
        }
    }

    #[test]
    fn map_and_shortcut_creation_descriptors_do_not_follow_saved_orientation() {
        for rotate in [false, true] {
            let ini = format!("[32]\nposX=600\nposY=300\nrotate={rotate}\n[37]\nposX=400\nposY=200\n");
            let mut ui = GameUi::new(
                small_font(), font("L2Font-e.LargeFont-e"), Options::default(),
                WindowsInfo::from_ini(&parse_ini(&ini)),
                ChatFilter::from_ini(&parse_ini(CHAT_FILTER)), (1558, 933),
            ).unwrap();
            let map = ui.desktop.window(WindowId::MAP_COVER).unwrap();
            assert_eq!((map.rect.x, map.rect.y), (400, 200));
            assert_eq!(map.initial_position, Some(killer_ui::minimap::initial_position((1558, 933))));
            let bar = ui.desktop.window(WindowId::SHORTCUT).unwrap();
            assert_eq!((bar.rect.x, bar.rect.y), (600, 300));
            assert_eq!((bar.rect.w, bar.rect.h), if rotate { (504, 46) } else { (46, 504) });
            let initial = killer_ui::shortcut::initial_position((1558, 933));
            assert_eq!(bar.initial_position, Some(initial));
            assert_eq!((initial.horizontal_mode, initial.vertical_mode, initial.x, initial.y), (1, 2, 46, 504));
            let rotated = ui.shortcut.rotate(bar.rect);
            let bar = ui.desktop.window_mut(WindowId::SHORTCUT).unwrap();
            bar.rect = rotated;
            assert_eq!(bar.initial_position, Some(initial));
        }
    }

    #[test]
    fn the_warning_box_is_modal_centred_and_closes_or_exits_on_ok_as_0x1000b120_10008f90_100091a0() {
        let mut ui = GameUi::new(
            small_font(), font("L2Font-e.LargeFont-e"), Options::default(),
            WindowsInfo::from_ini(&parse_ini("")),
            ChatFilter::from_ini(&parse_ini(CHAT_FILTER)), (1558, 933),
        ).unwrap();
        let window = ui.desktop.window(WindowId::WARNING_DIALOG).unwrap();
        assert_eq!((window.rect.x, window.rect.y, window.rect.w, window.rect.h), (651, 412, 256, 108));
        assert!(!window.visible);
        ui.show_warning("aviso", false);
        assert!(ui.desktop.window(WindowId::WARNING_DIALOG).unwrap().visible);
        assert_eq!(ui.desktop.windows.last().unwrap().id, WindowId::WARNING_DIALOG);
        let world = World::default();
        assert!(ui.mouse_down_at(10, 10, Instant::now(), &world));
        assert_eq!(ui.warning_released(), Some(KeyAction::Consumed));
        assert!(ui.warning.shown);
        let (ox, oy) = (651 + 100, 412 + 80);
        assert!(ui.mouse_down_at(ox, oy, Instant::now(), &world));
        assert_eq!(ui.warning_released(), Some(KeyAction::Consumed));
        assert!(!ui.warning.shown && !ui.desktop.window(WindowId::WARNING_DIALOG).unwrap().visible);
        assert!(!ui.warning_exit);
        assert_eq!(ui.warning_released(), None);
        ui.show_warning("fatal", true);
        assert!(ui.mouse_down_at(ox, oy, Instant::now(), &world));
        ui.warning_released();
        assert!(ui.warning_exit && ui.warning.shown);
    }

    #[test]
    fn dialog_creation_descriptors_survive_ini_and_numeric_panel_width() {
        for saved in [false, true] {
            let ini = if saved { "[27]\nposX=123\nposY=234\n[28]\nposX=345\nposY=456\n" } else { "" };
            let mut ui = GameUi::new(
                small_font(), font("L2Font-e.LargeFont-e"), Options::default(),
                WindowsInfo::from_ini(&parse_ini(ini)),
                ChatFilter::from_ini(&parse_ini(CHAT_FILTER)), (1558, 933),
            ).unwrap();
            for (id, bottom, expected) in [
                (WindowId::DIALOG, false, if saved { (123, 234) } else { (651, 412) }),
                (WindowId::DIALOG_BOTTOM, true, if saved { (345, 456) } else { (347, 823) }),
            ] {
                let window = ui.desktop.window(id).unwrap();
                assert_eq!((window.rect.x, window.rect.y), expected);
                assert_eq!(window.initial_position, Some(dialog::initial_position(bottom, (1558, 933))));
                assert!(!window.visible);
            }
            ui.show_dialog_view(
                DialogView::with_count(DialogKind::OkCancel, "", 100),
                DialogAnswer::Shop, WindowId::DIALOG, Instant::now(),
            );
            let window = ui.desktop.window(WindowId::DIALOG).unwrap();
            assert_eq!(window.rect.w, DIALOG_SIZE.0 + dialog::KEYPAD_WIDTH);
            assert_eq!(window.initial_position, Some(dialog::initial_position(false, (1558, 933))));
            assert!(window.visible);
            assert!(!ui.desktop.window(WindowId::DIALOG_BOTTOM).unwrap().visible);
        }
    }

    #[test]
    fn manor_packets_open_the_window_on_the_page_and_tab_presses_send_bypass_commands() {
        use killer_ui::manor_info::{ManorHit, ManorPage};
        use l2_net::game::{CropInfo, ManorDefaultEntry, ManorDefaultInfo, ManorEntry, ManorList, SeedInfo, SeedInfoEntry};
        let mut ui = fixture_ui((1558, 933));
        let id = killer_ui::manor_info::WINDOW_ID;
        assert!(!ui.desktop.window(id).unwrap().visible);
        assert_eq!(ui.desktop.window(id).unwrap().rect.w, 600);
        assert_eq!(ui.desktop.window(id).unwrap().rect.h, 326);
        ui.manor_event(&killer_world::WorldEvent::SendManorList(ManorList { entries: vec![ManorEntry { id: 3, name: "Aden".into() }] }), None);
        ui.manor_event(&killer_world::WorldEvent::ShowManorDefaultInfo(ManorDefaultInfo {
            flag: 0,
            entries: vec![ManorDefaultEntry { values: [5, 6, 7, 8], first: vec![1], second: vec![] }],
        }), None);
        assert!(ui.desktop.window(id).unwrap().visible);
        assert_eq!(ui.keys.manor_info.page(), Some(ManorPage::Default));
        assert_eq!(ui.keys.manor_info.default.list.count(), 1);
        ui.manor_event(&killer_world::WorldEvent::ShowSeedInfo(SeedInfo {
            flag: 1,
            values: [3, 0],
            entries: vec![SeedInfoEntry { values: [1, 2, 3, 4, 5], first: vec![], second: vec![] }],
        }), None);
        assert_eq!(ui.keys.manor_info.page(), Some(ManorPage::Seed));
        assert!(ui.keys.manor_info.seed.flag);
        assert_eq!(ui.keys.manor_info.seed.committed, -1);
        ui.manor_event(&killer_world::WorldEvent::ShowCropInfo(CropInfo { flag: 0, values: [3, 0], entries: Vec::new() }), None);
        assert_eq!(ui.keys.manor_info.page(), Some(ManorPage::Crop));
        let rect = ui.desktop.window(id).unwrap().rect;
        let tab = ui.keys.manor_info.tabs.buttons[2].paint.rect.offset(12, 25);
        assert!(ui.keys.mouse_down(id, tab.x + 2, tab.y + 2, &mut ui.desktop));
        let commands = ui.keys.take_commands();
        assert!(commands.iter().any(|command| matches!(command, UiCommand::Net(NetCommand::Bypass { command }) if command == killer_ui::manor_info::ASK_TAB_DEFAULT)));
        let close = (rect.w - 23 + 2, 5);
        assert_eq!(ui.keys.manor_info.hit(close.0, close.1), Some(ManorHit::Close));
        assert!(ui.keys.mouse_down(id, close.0, close.1, &mut ui.desktop));
        assert!(!ui.desktop.window(id).unwrap().visible);
    }

    #[test]
    fn special_primary_windows_have_known_restore_descriptors() {
        let ui = game_ui((1558, 933));
        let radar = ui.desktop.window(WindowId::RADAR).unwrap();
        assert_eq!(radar.initial_position, Some(killer_ui::radar::initial_position(ui.desktop.size)));
        let restart = ui.desktop.window(WindowId::RESTART_MENU).unwrap();
        assert_eq!(restart.initial_position, Some(restart::RESTART_INITIAL_POSITION));
        let zone = ui.desktop.window(WindowId::ZONE_TITLE).unwrap();
        assert_eq!(zone.initial_position, Some(killer_ui::initial_position::DISABLED_INITIAL_POSITION));
        assert_eq!(zone.rect, Rect::new(0, 0, 0, 0));
        assert_eq!(ui.desktop.window(WindowId::OPTION).unwrap().initial_position, Some(killer_ui::option::INITIAL_POSITION));
        let mut unmapped: Vec<_> = ui.desktop.windows.iter()
            .filter(|window| window.initial_position.is_none()).map(|window| window.id).collect();
        unmapped.sort_by_key(|id| id.0);
        assert_eq!(ui.desktop.window(WindowId::CHAT).unwrap().initial_position, Some(chat::initial_position((1558, 933))));
        assert_eq!(ui.desktop.window(WindowId::SYSTEM_MSG).unwrap().initial_position, Some(sysmsg::INITIAL_POSITION));
        assert!(unmapped.is_empty());
        use killer_ui::window_owner::{WindowOwner, WindowParent};
        assert_eq!(ui.desktop.window(WindowId::SYSTEM_MSG).unwrap().owner, Some(WindowOwner { parent: WindowParent::Window(WindowId::CHAT), slot: 0x1a0 }));
        assert_eq!(ui.desktop.window(WindowId::OPTION).unwrap().owner, Some(WindowOwner { parent: WindowParent::SecondaryRoot, slot: 0xf4 }));
        assert_eq!(ui.desktop.windows.iter().filter(|w| w.owner.is_some_and(|owner| owner.parent == WindowParent::PrimaryRoot)).count(), 92);
        let virtual_root = killer_ui::window_owner::VIRTUAL_ROOT_CHILDREN;
        assert!(ui.desktop.windows.iter().all(|w| w.owner.is_some() || virtual_root.contains(&w.id)));
        assert!(ui.desktop.windows.iter().all(|w| w.registration.is_some() || virtual_root.contains(&w.id)));
        let instances: std::collections::HashSet<_> = ui.desktop.windows.iter().map(Window::instance).collect();
        assert_eq!(instances.len(), 109);
        for window in &ui.desktop.windows {
            assert_eq!(ui.desktop.instance(&window.instance()).unwrap().id, window.id);
        }
        assert_eq!(ui.window_registry.manager.count, 99);
        assert_eq!(ui.window_registry.manager.roots.len(), 5);
        for window in ui.desktop.windows.iter().filter(|window| !virtual_root.contains(&window.id)) {
            let instance = window.instance();
            let node = ui.window_registry.node(&instance).unwrap();
            let record = ui.window_registry.manager.records.get(node.record).unwrap();
            assert_eq!(record.window.as_ref(), Some(&instance));
            assert!(record.manager_bound);
            let expected_parent = match window.owner.unwrap().parent {
                WindowParent::PrimaryRoot => Some(ui.window_registry.primary.clone()),
                WindowParent::SecondaryRoot => Some(ui.window_registry.secondary.clone()),
                WindowParent::Window(id) => Some(ui.desktop.window(id).unwrap().instance()),
                WindowParent::ConsoleManager => None,
            };
            assert_eq!(ui.window_registry.parent_instance(&instance), expected_parent.as_ref());
        }
        let primary = ui.window_registry.node(&ui.window_registry.primary).unwrap();
        let mut node = primary.children.first();
        let mut calls = Vec::new();
        while let Some(current) = node {
            let record = primary.children.record(current).unwrap();
            let instance = ui.window_registry.manager.records.get(record).unwrap().window.as_ref().unwrap();
            calls.push(ui.desktop.instance(instance).unwrap().registration.unwrap().call_address);
            node = primary.children.next(current);
        }
        assert_eq!(calls.len(), 92);
        assert!(calls.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn creation_descriptors_survive_saved_positions_widths_and_dragging() {
        use killer_ui::initial_position::{STATUS_INITIAL_POSITION, TARGET_INITIAL_POSITION};

        let mut ui = GameUi::new(
            small_font(),
            font("L2Font-e.LargeFont-e"),
            Options::default(),
            WindowsInfo::from_ini(&parse_ini("[12]\nposX=740\nposY=420\nwidth=300\n[13]\nposX=900\nposY=450\n[1]\nposX=123\nposY=234\n[10]\nposX=345\nposY=456\n")),
            ChatFilter::from_ini(&parse_ini(CHAT_FILTER)),
            (1558, 933),
        ).unwrap();
        let status = ui.desktop.window(WindowId::STATUS).unwrap();
        assert_eq!((status.rect.x, status.rect.y, status.rect.w), (740, 420, 300));
        assert_eq!(status.initial_position, Some(STATUS_INITIAL_POSITION));
        let target = ui.desktop.window(WindowId::TARGET).unwrap();
        assert_eq!((target.rect.x, target.rect.y), (900, 450));
        assert_eq!(target.initial_position, Some(TARGET_INITIAL_POSITION));
        for (id, position, descriptor) in [
            (WindowId::MAIN, (123, 234), killer_ui::mainwnd::initial_position((1558, 933))),
            (WindowId::INVENTORY, (345, 456), killer_ui::inventory::initial_position((1558, 933))),
        ] {
            let window = ui.desktop.window(id).unwrap();
            assert_eq!((window.rect.x, window.rect.y), position);
            assert_eq!(window.initial_position, Some(descriptor));
        }
        assert!(ui.desktop.begin_drag(741, 421));
        ui.desktop.drag(500, 300);
        ui.desktop.end_drag();
        assert_eq!(ui.desktop.window(WindowId::STATUS).unwrap().initial_position, Some(STATUS_INITIAL_POSITION));
        assert_eq!(ui.desktop.window(WindowId::OPTION).unwrap().initial_position, Some(killer_ui::option::INITIAL_POSITION));
    }

    pub(crate) fn me() -> Entity {
        Entity {
            item_origin: None,
            abnormal_effect: 0,
            object_id: 1,
            kind: EntityKind::Player {
                class_id: 0,
                race: 0,
                sex: 0,
                equipped: [0; VISIBLE_EQUIP_SLOTS],
                hair_style: 0,
                hair_color: 0,
                face: 0,
                enchant_effect: 0,
                hero: false,
            },
            name: "Killer".into(),
            title: String::new(),
            position: glam::Vec3::ZERO,
            heading: 0,
            dest: None,
            stop_heading: None,
            desired_heading: 0,
            turn_rate: None,
            run_speed: 120,
            walk_speed: 80,
            speed_multiplier: 1.0,
            attack_speed_multiplier: 1.0,
            m_atk_speed: 0,
            skill_aim: [0; 3],
            p_atk_speed: 0,
            speed_bonus: 0.0,
            speed_bonus_left: 0.0,
            blocked: false,
            follow: None,
            running: true,
            sitting: false,
            fake_dead: false,
            dead: false,
            walk_anim_frame_in_move: 12,
            hp: Some((450, 900)),
            mp: Some((200, 300)),
            cp: Some((100, 100)),
            level: 52,
            exp: 52262490,
            collision_radius: 8.0,
            collision_height: 24.0,
            base_collision: [0.0; 2],
            karma: 0,
            pvp_flag: 0,
            relation: 0, auto_attack: false, surrender_war_id: 0,
            target: None,
            casting: None,
            magic: Default::default(),
            anim: Default::default(),
            spawned: 0,
            name_color: WHITE_NAME_COLOR,
            recommendations: 0,
            name_blink: false,
            name_blink_elapsed: 0.0,
            clan_id: 0,
            clan_crest_id: 0,
            ally_id: 0,
            ally_crest_id: 0,
            store_type: 0,
            in_combat: false,
            fishing: 0,
            mount: None,
            vehicle: None,
            chair: None,
            rotation: None,
        }
    }

    fn world() -> World {
        World { me: Some(me()), ..Default::default() }
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    fn tables() -> DatTables {
        DatTables::default()
    }

    fn skill(id: u32, level: u32, extra_eff: u32) -> Skill {
        Skill {
            id,
            level,
            oper_type: 1,
            mp_consume: 0,
            cast_range: 0,
            cast_style: 0,
            hit_time: 0.0,
            is_magic: 1,
            ani_char: String::new(),
            visual_effect: String::new(),
            icon: format!("icon.skill{id:04}"),
            extra_eff,
            is_ench: 0,
            ench_skill_id: 0,
            hp_consume: 0,
            rumble_self: 0,
            rumble_target: 0,
        }
    }

    fn skill_tables() -> DatTables {
        let mut tables = tables();
        tables.skillgrp =
            [skill(1204, 1, 0), skill(1204, 2, 0), skill(1069, 1, 1), skill(2274, 1, 0)].into_iter().collect();
        tables
    }

    fn message(id: u32, text: &str) -> SystemMessageRecord {
        SystemMessageRecord { id, present: 1, text: text.into(), group: 0, color: [0; 3], alpha: 0xFF, sound: String::new(), second_sound: String::new() }
    }

    fn dialog_tables() -> DatTables {
        let mut tables = tables();
        tables
            .systemmsg
            .insert(572, message(572, "Do you wish to join $s1's party? (Item distribution: Finders Keepers)"));
        tables.systemmsg.insert(573, message(573, "Do you wish to join $s1's party? (Item distribution: Random)"));
        tables.systemmsg.insert(67, message(67, "$s1 has invited you to the $s2 Clan. Do you want to join?"));
        tables.systemmsg.insert(100, message(100, "$s1 requests a trade. Do you want to trade?"));
        tables
            .systemmsg
            .insert(1510, message(1510, "$s1 is making an attempt at resurrection. Do you want to continue?"));
        tables.sysstring.insert(140, "OK".into());
        tables.sysstring.insert(141, "Cancel".into());
        tables.sysstring.insert(184, "Sim".into());
        tables.sysstring.insert(185, "Não".into());
        tables
    }

    #[test]
    fn dialogs_open_at_the_bottom_or_centered_with_the_systemmsg_text_and_the_sysstring_labels() {
        let tables = dialog_tables();
        let mut ui = game_ui((1558, 933));
        ui.set_tab_names(&tables);
        let mut world = world();
        let mut other = me();
        other.object_id = 3;
        other.name = "Luzal".into();
        world.entities.insert(3, other);
        let now = Instant::now();
        assert!(!ui.desktop.window(WindowId::DIALOG).unwrap().visible);
        assert_eq!(ui.desktop.window(WindowId::DIALOG).unwrap().rect, Rect::new(635, 361, 256, 108));
        assert_eq!(ui.desktop.window(WindowId::DIALOG_BOTTOM).unwrap().rect, Rect::new(267, 718, 256, 108));
        world.dialogs.push(Dialog::JoinParty(l2_net::game::packets::AskJoinParty {
            requestor_name: "Luzal".into(),
            item_distribution: 1,
        }));
        assert_eq!(ui.tick_dialog(&mut world, Some(&tables), now), None);
        let dialog = ui.dialog.as_ref().unwrap();
        assert_eq!(dialog.view.message, "Do you wish to join Luzal's party? (Item distribution: Random)");
        assert_eq!(dialog.view.labels, ("Sim".to_string(), "Não".to_string()));
        assert_eq!(
            (dialog.answer.clone(), dialog.window, dialog.view.dialog_type),
            (DialogAnswer::JoinParty, WindowId::DIALOG_BOTTOM, 10)
        );
        assert!(ui.desktop.window(WindowId::DIALOG_BOTTOM).unwrap().visible);
        assert!(!ui.desktop.window(WindowId::DIALOG).unwrap().visible);
        let list = ui.frame(&world, &[], &[], None, &WorldViews::default());
        assert_eq!(with_texture(&list.quads, killer_ui::dialog::SYSTEM_BACK)[0].x, 267.0);
        assert!(ui.mouse_down(267 + 60, 718 + 80));
        assert_eq!(ui.dialog.as_ref().unwrap().pressed, Some(DialogButton::First));
        assert_eq!(ui.mouse_up(), None);
        assert_eq!(ui.dialog_released(), Some(KeyAction::Dialog(DialogAnswer::JoinParty, true)));
        assert!(ui.dialog.is_none());
        assert!(!ui.desktop.window(WindowId::DIALOG_BOTTOM).unwrap().visible);
        world.dialogs.push(Dialog::TradeRequest { requestor_id: 3 });
        world
            .dialogs
            .push(Dialog::Confirm(SystemMessage { id: 1510, args: vec![SystemMessageArg::Text("Luzal".into())] }));
        ui.tick_dialog(&mut world, Some(&tables), now);
        assert_eq!(ui.dialog.as_ref().unwrap().view.message, "Luzal requests a trade. Do you want to trade?");
        assert_eq!(ui.tick_dialog(&mut world, Some(&tables), now + Duration::from_secs(3)), None);
        let parked = ui.parked_dialog.as_ref().unwrap();
        assert_eq!((parked.answer.clone(), parked.view.timer_elapsed_ms), (DialogAnswer::Trade, Some(3000)));
        assert!(ui.desktop.window(WindowId::DIALOG_BOTTOM).unwrap().visible && ui.desktop.window(WindowId::DIALOG).unwrap().visible);
        let dialog = ui.dialog.as_ref().unwrap();
        assert_eq!((dialog.window, dialog.view.kind), (WindowId::DIALOG, DialogKind::OkCancel));
        assert_eq!(dialog.view.labels.0, "OK");
        assert_eq!((dialog.answer.clone(), dialog.view.dialog_type), (DialogAnswer::Confirm { message_id: 1510 }, 70));
        assert_eq!(
            ui.tick_dialog(&mut world, Some(&tables), now + Duration::from_secs(11)),
            Some(KeyAction::Dialog(DialogAnswer::Trade, false))
        );
        assert!(ui.parked_dialog.is_none() && !ui.desktop.window(WindowId::DIALOG_BOTTOM).unwrap().visible);
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Dialog(DialogAnswer::Confirm { message_id: 1510 }, false));
        assert!(ui.dialog.is_none());
        assert_eq!(ui.tick_dialog(&mut world, Some(&tables), now), None);
        let (view, answer, window) = dialog_request(
            &Dialog::JoinPledge(l2_net::game::packets::AskJoinPledge {
                requestor_id: 3,
                pledge_name: "Killers".into(),
            }),
            &world,
            Some(&tables),
        );
        assert_eq!(
            (view.message.as_str(), answer, window),
            (
                "Luzal has invited you to the Killers Clan. Do you want to join?",
                DialogAnswer::JoinPledge,
                WindowId::DIALOG_BOTTOM
            )
        );
        assert_eq!(
            dialog_request(&Dialog::FriendRequest { requestor_name: "x".into() }, &world, None).0.message,
            "systemmsg 516"
        );
    }

    #[test]
    fn alliance_war_dialogs_1007a880_ask_yes_no_with_both_names_and_send_nothing() {
        let world = World::default();
        let (view, answer, window) = dialog_request(&Dialog::AllianceWar { start: true, first: "A".into(), second: "B".into() }, &world, None);
        assert_eq!((view.dialog_type, view.kind, answer, window), (0x34, DialogKind::YesNo, DialogAnswer::Notice, WindowId::DIALOG_BOTTOM));
        assert_eq!(view.message, "systemmsg 665");
        let (view, _, _) = dialog_request(&Dialog::AllianceWar { start: false, first: "A".into(), second: "B".into() }, &world, None);
        assert_eq!((view.dialog_type, view.message.as_str()), (0x35, "systemmsg 666"));
    }

    fn pet_info(summon_type: i32) -> PetInfo {
        PetInfo {
            summon_type,
            object_id: 9,
            npc_id: 12077,
            attackable: false,
            x: 0,
            y: 0,
            z: 0,
            heading: 0,
            m_atk_speed: 0,
            p_atk_speed: 0,
            run_speed: 0,
            walk_speed: 0,
            speed_multiplier: 1.0,
            attack_speed_multiplier: 1.0,
            collision_radius: 8.0,
            collision_height: 20.0,
            rhand: 0,
            chest: 0,
            lhand: 0,
            has_owner: true,
            running: true,
            in_combat: false,
            alike_dead: false,
            spawn_animation: 0,
            name: "Wolf".into(),
            title: "Killer".into(),
            pvp_flag: 0,
            karma: 0,
            fed: 80,
            max_fed: 100,
            hp: 300,
            max_hp: 600,
            mp: 45,
            max_mp: 90,
            sp: 0,
            level: 25,
            exp: 0,
            exp_this_level: 0,
            exp_next_level: 0,
            load: 0,
            max_load: 0,
            p_atk: 0,
            p_def: 0,
            m_atk: 0,
            m_def: 0,
            accuracy: 0,
            evasion: 0,
            critical: 0,
            speed: 0,
            atk_spd: 0,
            cast_spd: 0,
            abnormal_effect: 0,
            mountable: 0,
            team: 0,
            soulshots_per_hit: 0,
            spiritshots_per_hit: 0,
        }
    }

    #[test]
    fn the_pet_window_follows_the_summon_type_and_clicks_select_or_expand_the_buffs() {
        let tables = skill_tables();
        let mut ui = game_ui((1558, 933));
        let mut world = world();
        assert_eq!(ui.desktop.window(WindowId::PET_STATUS).unwrap().rect, Rect::new(0, 72, 172, 46));
        assert_eq!(ui.desktop.window(WindowId::SUMMONED_STATUS).unwrap().grip, Rect::new(0, 17, 12, 29));
        assert!(!ui.desktop.window(WindowId::PET_STATUS).unwrap().visible);
        assert_eq!(pet_view(&world, Some(&tables)), None);
        world.pet = Some(Pet { info: pet_info(SUMMON_PET), max_time: 0, remaining_time: 0 });
        world.party.spelled.insert(9, vec![PartyBuff { skill_id: 1204, level: 2, duration: 100 }]);
        let views = world_views(&world, Some(&tables), Instant::now());
        let (id, object_id, view) = views.pet.clone().unwrap();
        assert_eq!(
            (id, object_id, view.name.as_str(), view.hp, view.mp),
            (WindowId::PET_STATUS, 9, "Wolf", (300, 600), (45, 90))
        );
        assert_eq!(view.third, ThirdBar::Fed { fed: 80, max_fed: 100 });
        assert_eq!(view.buffs, vec![Some("icon.skill1204".to_string())]);
        ui.desktop.raise_newly_shown();
        ui.show_pet_window(&views);
        ui.desktop.raise_newly_shown();
        assert!(ui.desktop.window(WindowId::PET_STATUS).unwrap().visible);
        assert!(!ui.desktop.window(WindowId::SUMMONED_STATUS).unwrap().visible);
        let list = ui.frame(&world, &[], &[], None, &views);
        let fed = with_texture(&list.quads, FATIGUE_BAR);
        assert_eq!((fed[0].x, fed[0].y, fed[0].w), (16.0, 72.0 + 33.0, 120.0));
        assert!(with_texture(&list.quads, "icon.skill1204").is_empty());
        assert!(ui.mouse_down(5, 72 + 10));
        assert_eq!(ui.mouse_up(), None);
        assert_eq!(ui.pet_released(&views), Some(KeyAction::Consumed));
        assert!(ui.pet_buffs_shown);
        let list = ui.frame(&world, &[], &[], None, &views);
        assert_eq!(with_texture(&list.quads, "icon.skill1204")[0].x, 173.0);
        assert_eq!(with_texture(&list.quads, PET_BUTTON_DOWN).len(), 1);
        ui.keys.sysstrings.insert(881, "Buffs".into());
        ui.mouse_move(5, 72 + 10);
        assert_eq!(ui.keys.tooltip.as_ref().map(|t| t.name.as_str()), Some("Buffs"));
        assert!(ui.mouse_down(100, 72 + 30));
        assert_eq!(ui.desktop.dragging(), None);
        ui.mouse_up();
        assert_eq!(ui.pet_released(&views), None);
        assert_eq!(ui.take_status_click(&world), Some(KeyAction::Consumed));
        assert!(ui.keys.take_commands().iter().any(|c| matches!(c, UiCommand::Net(NetCommand::Action { object_id: 9, .. }))));
        assert!(ui.mouse_down(5, 72 + 30));
        assert_eq!(ui.desktop.dragging(), Some(WindowId::PET_STATUS));
        ui.mouse_up();
        assert_eq!(ui.pet_released(&views), None);
        world.pet = Some(Pet { info: pet_info(SUMMON_SERVITOR), max_time: 1200, remaining_time: 300 });
        let views = world_views(&world, Some(&tables), Instant::now());
        let (id, _, view) = views.pet.clone().unwrap();
        assert_eq!((id, view.third), (WindowId::SUMMONED_STATUS, ThirdBar::Time { remaining: 300, max: 1200 }));
        ui.show_pet_window(&views);
        assert!(ui.desktop.window(WindowId::SUMMONED_STATUS).unwrap().visible);
        assert!(!ui.desktop.window(WindowId::PET_STATUS).unwrap().visible);
        let list = ui.frame(&world, &[], &[], None, &views);
        assert_eq!(with_texture(&list.quads, FATIGUE_BAR_BACK)[0].h, 6.0);
        assert_eq!(with_texture(&list.quads, FATIGUE_BAR)[0].w, 38.0);
        world.pet = None;
        ui.show_pet_window(&world_views(&world, Some(&tables), Instant::now()));
        assert!(!ui.desktop.window(WindowId::SUMMONED_STATUS).unwrap().visible);
    }

    #[test]
    fn dying_opens_the_centered_restart_menu_whose_buttons_ask_for_the_restart_point() {
        let mut tables = tables();
        tables.sysstring.insert(372, "Para a Vila".into());
        let mut ui = game_ui((1558, 933));
        let mut world = world();
        let window = ui.desktop.window(WindowId::RESTART_MENU).unwrap();
        assert_eq!((window.rect, window.visible), (Rect::new(715, 402, 128, 128), false));
        assert_eq!(restart_view(&world, Some(&tables)), None);
        world.death = Some(l2_net::game::packets::Die {
            object_id: 1,
            to_village: true,
            to_hideout: false,
            to_castle: false,
            to_siege_hq: false,
            sweepable: false,
            to_fixed_spot: true,
        });
        let views = world_views(&world, Some(&tables), Instant::now());
        let view = views.restart.clone().unwrap();
        assert_eq!(view.enabled, [true, false, false, false, true]);
        assert_eq!(view.labels[0], "Para a Vila");
        assert_eq!(view.labels[1], "To Hideaway");
        ui.show_pet_window(&views);
        assert!(ui.desktop.window(WindowId::RESTART_MENU).unwrap().visible);
        let list = ui.frame(&world, &[], &[], None, &views);
        let back = with_texture(&list.quads, killer_ui::restart::RESTART_BACK);
        assert_eq!((back[0].x, back[0].y), (715.0, 402.0));
        assert!(ui.mouse_down(715 + 20, 402 + 10));
        assert_eq!(ui.mouse_up(), None);
        assert_eq!(ui.restart_released(&views), Some(KeyAction::RestartPoint(0)));
        assert!(ui.mouse_down(715 + 20, 402 + 34));
        ui.mouse_up();
        assert_eq!(ui.restart_released(&views), None);
        assert!(ui.mouse_down(715 + 20, 402 + 106));
        ui.mouse_move(715 + 20, 402 + 10);
        ui.mouse_up();
        assert_eq!(ui.restart_released(&views), None);
        world.death = None;
        ui.show_pet_window(&world_views(&world, Some(&tables), Instant::now()));
        assert!(!ui.desktop.window(WindowId::RESTART_MENU).unwrap().visible);
    }

    #[test]
    fn the_chat_size_control_resizes_the_window_through_the_resize_frame() {
        let mut ui = game_ui((1558, 933));
        let window = ui.desktop.window(WindowId::CHAT).unwrap().rect;
        let bottom = window.y + window.h;
        assert!(ui.mouse_down(window.x + 10, window.y + 2));
        assert_eq!(ui.desktop.dragging(), None);
        assert!(ui.mouse_move(window.x + 10, window.y + 2 + 15));
        let resized = ui.desktop.window(WindowId::CHAT).unwrap().rect;
        assert_eq!((resized.h, resized.y + resized.h), (window.h + 15, bottom));
        assert_eq!(ui.chat.height, resized.h);
        ui.mouse_up();
        assert!(!ui.chat.size_frame.dragging);
    }

    #[test]
    fn modern_chat_selects_all_channels_from_the_top_tabs() {
        let mut ui=game_ui((1558,933));
        for window in &mut ui.desktop.windows { window.visible=false; }
        ui.shortcut.modern=Some(killer_ui::modern_shortcut::ModernShortcuts::default());
        let window=ui.desktop.window_mut(WindowId::CHAT).unwrap();
        window.rect=Rect::new(20,650,348,262);
        window.visible=true;
        for index in 0..5 {
            let tab=crate::ui_modern::chat_tab_rect(index);
            ui.chat.scroll=12;
            assert!(ui.mouse_down(20+tab.x+tab.w/2,650+tab.y+tab.h/2));
            ui.mouse_up();
            assert_eq!(ui.chat.active_tab,index);
            assert_eq!(ui.chat.scroll,0);
            assert!(ui.chat.focused);
        }
    }

    #[test]
    fn modern_bar_buttons_expand_upward_select_each_page_and_keep_keyboard_on_primary() {
        use killer_ui::modern_shortcut::{ModernBarButton,ModernShortcuts};
        let mut ui=game_ui((1558,933));
        ui.shortcut.modern=Some(ModernShortcuts::default());
        let (width,height)=ui.shortcut.size();
        let window=ui.desktop.window_mut(WindowId::SHORTCUT).unwrap();
        window.rect=Rect::new(600,933-height-16,width,height);
        window.grip=ui.shortcut.grip();
        let bottom=window.rect.y+window.rect.h;
        for count in 4..=5 {
            let rect=ui.desktop.window(WindowId::SHORTCUT).unwrap().rect;
            let button=ui.shortcut.modern.unwrap().button_rect(ModernBarButton::Expand);
            let (x,y)=(rect.x+button.x+2,rect.y+button.y+2);
            assert!(ui.mouse_down(x,y));ui.mouse_move(x,y);ui.mouse_up();
            assert_eq!(ui.shortcut.modern.unwrap().count,count);
            let rect=ui.desktop.window(WindowId::SHORTCUT).unwrap().rect;
            assert_eq!(rect.y+rect.h,bottom);
        }
        let rect=ui.desktop.window(WindowId::SHORTCUT).unwrap().rect;
        let button=ui.shortcut.modern.unwrap().button_rect(ModernBarButton::Next(3));
        let (x,y)=(rect.x+button.x+2,rect.y+button.y+2);
        assert!(ui.mouse_down(x,y));ui.mouse_move(x,y);ui.mouse_up();
        assert_eq!(ui.shortcut.modern.unwrap().pages[2],4);
        assert_eq!(ui.shortcut.page,0);
        ui.keys.alt=true;
        assert!(ui.hook_key(killer_ui::keys::WM_SYSKEYDOWN,killer_ui::keys::VK_F1+9,0,&World::default()));
        assert_eq!(ui.shortcut.page,9);
        assert_eq!(ui.shortcut.modern.unwrap().pages[2],4);
    }

    #[test]
    fn special_maps_and_trainer_receive_pointer_movement_through_game_ui() {
        use killer_world::WorldEvent;
        let mut ui = game_ui((1558, 933));
        let event = WorldEvent::ShowTownMap(l2_net::game::ShowTownMap { texture: "town_map.test".into(), x: 256, y: 256 });
        ui.keys.special_maps.world_event(&event, None, &mut ui.desktop);
        let rect = ui.desktop.window(WindowId::TOWN_MAP).unwrap().rect;
        assert!(ui.mouse_down(rect.x + 100, rect.y + 100));
        assert!(ui.mouse_move(rect.x + 50, rect.y + 70));
        assert_eq!(ui.keys.special_maps.town.scroll, (142,122));
        ui.mouse_up();
        ui.mouse_move(rect.x + 100, rect.y + 100);
        assert_eq!(ui.keys.special_maps.town.scroll, (142,122));
        let close = killer_ui::special_maps::CLOSE;
        let under = ui.desktop.window_mut(WindowId::MAIN).unwrap();
        under.visible = true;
        under.rect.x = rect.x + close.x;
        under.rect.y = rect.y + close.y;
        assert!(ui.mouse_down(rect.x + close.x, rect.y + close.y));
        assert!(!ui.desktop.window(WindowId::TOWN_MAP).unwrap().visible);
        assert_eq!(ui.desktop.dragging(), None);
        ui.mouse_up();
        let world = World::default();
        ui.train_event(&WorldEvent::AcquireSkillList(l2_net::game::AcquireSkillList {
            mode: 0, skills: vec![l2_net::game::LearnableSkill { id: 3, next_level: 1, max_level: 1, sp_cost: 0, requirements: 0 }; 30],
        }), &world, None);
        let rect = ui.desktop.window(WindowId::TRAIN).unwrap().rect;
        let grid = killer_ui::train::LIST_RECT;
        let x = rect.x + grid.x + grid.w - 5;
        let y = rect.y + grid.y + killer_ui::scroll::BUTTON_SIZE + 2;
        assert!(ui.mouse_down(x,y));
        assert!(ui.mouse_move(x,y+80));
        assert!(ui.keys.train.view.first_row > 0);
        ui.mouse_up();
    }

    #[test]
    fn the_radar_sits_in_the_top_right_corner_and_maps_party_target_and_destination_around_me() {
        let mut ui = game_ui((1558, 933));
        let window = ui.desktop.window(WindowId::RADAR).unwrap();
        assert_eq!((window.rect, window.grip), (Rect::new(1456, 0, 102, 102), Rect::new(0, 0, 0, 0)));
        assert!(ui.mouse_down(1500, 50));
        assert_eq!(ui.desktop.dragging(), None);
        ui.mouse_up();
        let mut world = world();
        let me = world.me.as_mut().unwrap();
        me.position = glam::Vec3::new(1000.0, 2000.0, 10.0);
        me.dest = Some(glam::Vec3::new(1300.0, 2000.0, 0.0));
        world.party.positions.insert(1, [1000, 2000, 0]);
        world.party.positions.insert(9, [1500, 2000, 0]);
        world.party.positions.insert(7, [1000, 2600, 0]);
        let mut gremlin = super::tests::me();
        gremlin.object_id = 2;
        gremlin.position = glam::Vec3::new(1000.0, 1700.0, 0.0);
        world.entities.insert(2, gremlin);
        let mut walker = super::tests::me();
        walker.object_id = 9;
        walker.position = glam::Vec3::new(800.0, 2000.0, 0.0);
        world.entities.insert(9, walker);
        world.select(2);
        assert!(radar_view(&world, (0, 0)).arrow_to.is_none());
        world.windows.radar.apply_control(0, 0, [1100, 2000, 0]);
        let view = radar_view(&world, (0x4000, -0x1800));
        assert_eq!((view.camera_yaw, view.camera_pitch, view.camera_roll), (0x4000, -0x1800, 0));
        assert_eq!(view.party, vec![[-200.0, 0.0], [0.0, 600.0]]);
        assert_eq!(view.target, Some([0.0, -300.0]));
        assert_eq!(view.arrow_to, Some([100.0, 0.0, -10.0]));
        assert_eq!(view.points, vec![[100.0, 0.0]]);
        world.windows.radar.apply_control(1, 0, [1100, 2000, 0]);
        assert!(radar_view(&world, (0, 0)).points.is_empty());
        world.windows.radar.apply_control(2, 0, [0, 0, 0]);
        assert!(radar_view(&world, (0, 0)).arrow_to.is_none());
        assert!(!radar_view(&world, (0, 0)).show_time);
        world.windows.radar.show_time = true;
        world.windows.radar.enabled = true;
        let shown = radar_view(&world, (0, 0));
        assert!(shown.show_time);
        assert_eq!(shown.arrow_to, Some([100.0, 0.0, -10.0]));
        world.windows.radar.enabled = false;
        assert_eq!(radar_view(&World::default(), (0, 0)), RadarView::default());
        assert!(ui.radar_visible(&world));
        assert!(!ui.radar_visible(&World::default()));
        ui.desktop.window_mut(WindowId::RADAR).unwrap().visible = false;
        assert!(!ui.radar_visible(&world));
        let list = ui.frame(&world, &[], &[], None, &WorldViews { radar: view, ..WorldViews::default() });
        assert!(list.quads.iter().all(|quad| !quad.texture.contains(".Radar.")));
    }

    #[test]
    fn the_system_message_panel_sits_130_above_the_chat_follows_it_and_scrolls_with_the_wheel() {
        let mut ui = game_ui((1558, 933));
        let window = ui.desktop.window(WindowId::SYSTEM_MSG).unwrap();
        assert_eq!((window.rect, window.visible), (Rect::new(0, 616, 348, 125), true));
        let system = |text: &str, group: u32| ChatEntry {
            kind: killer_world::chat::CHAT_SYSTEM,
            name: String::new(),
            text: text.into(),
            group,
            color: Some(Argb(0xFFB09B79)),
        };
        let lines: Vec<ChatEntry> = (0..20).map(|i| system(&format!("m{i}"), if i % 2 == 0 { 0 } else { 3 })).collect();
        lines.iter().for_each(|line| ui.system_msg.ingest(line));
        let list = ui.frame(&world(), &[], &lines, None, &WorldViews::default());
        let back = with_texture(&list.quads, killer_ui::sysmsg::SYSTEM_MSG_BACK);
        assert_eq!((back[0].x, back[0].y), (0.0, 616.0));
        let text: Vec<&Quad> =
            list.quads.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e" && q.y >= 616.0 && q.y < 746.0).collect();
        assert_eq!(text.len(), 7 * 3);
        assert!(ui.wheel_over(100, 650, 2.0, &lines));
        assert_eq!(ui.system_msg.scroll, 2);
        assert!(!ui.wheel_over(800, 400, 2.0, &lines));
        assert!(ui.mouse_down(100, 750));
        ui.mouse_move(150, 700);
        ui.mouse_up();
        assert_eq!(ui.desktop.window(WindowId::CHAT).unwrap().rect.x, 50);
        assert_eq!(ui.desktop.window(WindowId::SYSTEM_MSG).unwrap().rect, Rect::new(50, 566, 348, 125));
        let hidden = GameUi::new(
            font("s"),
            font("l"),
            Options::default(),
            WindowsInfo::default(),
            ChatFilter::from_ini(&parse_ini(CHAT_FILTER)),
            (1280, 720),
        ).unwrap();
        assert!(!hidden.desktop.window(WindowId::SYSTEM_MSG).unwrap().visible);
        assert_eq!(hidden.desktop.window(WindowId::SYSTEM_MSG).unwrap().rect, Rect::new(0, 720 - 187 - 130, 348, 125));
    }

    #[test]
    fn the_zone_title_shows_the_tile_zone_centered_at_the_top_and_fades_out() {
        let mut tables = tables();
        let zone = |id, color, grid: (u32, u32), name: &str| l2_dat::ZoneName {
            id,
            color_id: color,
            grid_x: grid.0,
            grid_y: grid.1,
            top_z: 30000.0,
            bottom_z: -30000.0,
            name: name.into(),
        };
        tables.zonename = vec![zone(41, 253, (22, 22), "Arena"), zone(43, 255, (22, 22), "Giran Castle Town")];
        let mut ui = game_ui((1558, 933));
        let start = Instant::now();
        let now = start + Duration::from_secs(9);
        assert!(!ui.desktop.window(WindowId::ZONE_TITLE).unwrap().visible);
        ui.update_zone(&tables, (82157.0, 147057.0, -134.0), start);
        assert_eq!(ui.take_zone_texture_requests(), vec![(22, 22)]);
        assert!(ui.zone_title.is_none());
        ui.set_zone_texture((22, 22), Some(killer_ui::zone_title::ZoneTexture { width: 2, height: 2, pixels: vec![255; 4] }));
        ui.update_zone(&tables, (82157.0, 147057.0, -134.0), now);
        assert_eq!(ui.keys.minimap.zone, "Giran Castle Town");
        let (view, _) = ui.zone_title.clone().unwrap();
        assert_eq!(view.lines[0], "Giran Castle Town");
        let window = ui.desktop.window(WindowId::ZONE_TITLE).unwrap();
        let width = ui.large.measure("Giran Castle Town") as i32;
        assert_eq!(width, 96);
        assert_eq!((window.rect, window.visible), (Rect::new(779 - width / 2, 933 / 5, width, ui.large.line_height as i32), true));
        ui.tick_zone_title(now + Duration::from_secs(1));
        let list = ui.frame(&world(), &[], &[], None, &WorldViews::default());
        let deco = with_texture(&list.quads, killer_ui::zone_title::FONT_DECO);
        assert_eq!((deco[0].x, deco[0].y), ((779 - width / 2 - 100) as f32, (933 / 5 - 80) as f32));
        assert!(list.quads.iter().any(|q| q.texture == "L2Font-e.LargeFont-e" && q.y == (933 / 5) as f32));
        ui.tick_zone_title(now + Duration::from_millis(4500));
        assert_eq!(ui.zone_title.as_ref().unwrap().0.alpha, 191);
        ui.tick_zone_title(now + Duration::from_secs(5));
        assert_eq!(ui.zone_title.as_ref().unwrap().0.alpha, 127);
        ui.tick_zone_title(now + Duration::from_secs(8));
        assert!(ui.zone_title.is_some());
        ui.tick_zone_title(now + Duration::from_millis(8001));
        assert!(ui.zone_title.is_none());
        assert!(!ui.desktop.window(WindowId::ZONE_TITLE).unwrap().visible);
        ui.update_zone(&tables, (0.0, 0.0, 0.0), now);
        assert_eq!(ui.take_zone_texture_requests(), vec![(20, 18)]);
        assert!(ui.zone_title.is_none());
        let mut off = game_ui((1558, 933));
        off.options.show_zone_title = false;
        off.show_zone_title("Giran", now);
        assert!(off.zone_title.is_none());
        assert!(ui.zone_title_font.is_none());
    }

    #[test]
    fn the_radar_pass_is_ordered_by_the_position_of_the_radar_window_in_the_ui_list() {
        let mut ui = game_ui((1558, 933));
        let world = world();
        let list = ui.frame(&world, &[], &[], None, &WorldViews::default());
        let at = list.radar_at.unwrap();
        assert!(at <= list.quads.len());
        ui.desktop.window_mut(WindowId::RADAR).unwrap().visible = false;
        assert_eq!(ui.frame(&world, &[], &[], None, &WorldViews::default()).radar_at, None);
    }

    #[test]
    fn the_party_window_grows_with_the_members_and_clicks_select_them_or_show_their_buffs() {
        let tables = skill_tables();
        let mut ui = game_ui((1558, 933));
        let mut world = world();
        let window = ui.desktop.window(WindowId::PARTY_STATUS).unwrap();
        assert_eq!((window.rect, window.visible), (Rect::new(0, 141, 172, 0), false));
        let member = |id: i32, name: &str| l2_net::game::packets::PartyMember {
            object_id: id,
            name: name.into(),
            cp: 50,
            max_cp: 100,
            hp: 300,
            max_hp: 600,
            mp: 45,
            max_mp: 90,
            level: 40,
            class_id: 0,
        };
        world.party.leader_id = 7;
        world.party.members = vec![member(7, "Luzal"), member(8, "Kain")];
        world.party.spelled.insert(8, vec![PartyBuff { skill_id: 1069, level: 1, duration: 10 }]);
        let views = world_views(&world, Some(&tables), Instant::now());
        assert_eq!(views.party.members.len(), 2);
        assert!(views.party.members[0].leader && !views.party.members[1].leader);
        assert_eq!(
            views.party.members[1].buffs,
            vec![BuffIcon { skill_id: 1069, level: 1, icon: Some("icon.skill1069".into()), remaining: Some(10), debuff: true, ..Default::default() }]
        );
        ui.show_pet_window(&views);
        let window = ui.desktop.window(WindowId::PARTY_STATUS).unwrap();
        assert_eq!(
            (window.rect, window.grip, window.visible),
            (Rect::new(0, 141, 172, 92), Rect::new(0, 18, 12, 74), true)
        );
        let list = ui.frame(&world, &[], &[], None, &views);
        let leader = with_texture(&list.quads, killer_ui::party::LEADER_ICON);
        assert_eq!((leader.len(), leader[0].x, leader[0].y), (1, 12.0 + killer_ui::party::leader_icon_x(160, ui.small.measure("Luzal") as i32) as f32, 149.0));
        ui.keys.sysstrings.insert(489, "Loot".into());
        ui.keys.sysstrings.insert(487, "Finders".into());
        ui.keys.sysstrings.insert(798, "Random".into());
        ui.party_cache.loot_rule = 2;
        ui.mouse_move(66 + 5, 141 + 8 + 5);
        let rule = killer_ui::party::LOOT_RULE_SYSSTRINGS[ui.party_cache.loot_rule as usize];
        assert_eq!(rule, 798);
        assert_eq!(ui.keys.tooltip.as_ref().map(|t| t.name.as_str()), Some("Loot : Random"));
        ui.mouse_move(120, 141 + 8 + 5);
        assert!(ui.keys.tooltip.is_none());
        assert_eq!(with_texture(&list.quads, killer_ui::party::HP_BAR_BACK)[1].y, 141.0 + 46.0 + 28.0);
        assert!(with_texture(&list.quads, "icon.skill1069").is_empty());
        assert!(ui.mouse_down(5, 141 + 10));
        ui.mouse_up();
        assert_eq!(ui.party_released(&views), Some(KeyAction::Consumed));
        assert!(ui.party_buffs_shown);
        let list = ui.frame(&world, &[], &[], None, &views);
        assert_eq!(with_texture(&list.quads, "icon.skill1069")[0].x, 12.0 + 161.0);
        assert!(ui.mouse_down(100, 141 + 60));
        assert_eq!(ui.desktop.dragging(), None);
        ui.mouse_up();
        assert_eq!(ui.party_released(&views), None);
        assert_eq!(ui.take_status_click(&world), Some(KeyAction::Consumed));
        assert!(ui.keys.take_commands().iter().any(|c| matches!(c, UiCommand::Net(NetCommand::Action { object_id: 8, .. }))));
        ui.keys.ctrl = true;
        assert!(ui.mouse_down(100, 141 + 60));
        ui.mouse_up();
        ui.take_status_click(&world);
        assert!(ui.keys.take_commands().iter().any(|c| matches!(c, UiCommand::Net(NetCommand::AttackRequest { object_id: 8, .. }))));
        ui.keys.ctrl = false;
        assert!(ui.mouse_down(171, 141 + 60));
        ui.mouse_up();
        assert_eq!(ui.take_status_click(&world), None, "os dez últimos pixels da direita não miram");
        assert!(ui.mouse_down(5, 141 + 60));
        assert_eq!(ui.desktop.dragging(), Some(WindowId::PARTY_STATUS));
        ui.mouse_up();
        assert_eq!(ui.party_released(&views), None);
        assert_eq!(ui.party_assist_at(100, 141 + 60, &views), Some(KeyAction::Assist(8)));
        assert_eq!(ui.party_assist_at(5, 141 + 60, &views), None);
        world.party.members.clear();
        ui.show_pet_window(&world_views(&world, Some(&tables), Instant::now()));
        assert!(!ui.desktop.window(WindowId::PARTY_STATUS).unwrap().visible);
    }

    #[test]
    fn the_party_window_excludes_myself_and_shows_all_eight_other_members_when_the_group_is_full() {
        let mut world = world();
        let member = |id: i32| l2_net::game::packets::PartyMember {
            object_id: id,
            name: format!("m{id}"),
            cp: 0,
            max_cp: 0,
            hp: 1,
            max_hp: 1,
            mp: 0,
            max_mp: 0,
            level: 1,
            class_id: 0,
        };
        world.party.leader_id = 1;
        world.party.members = (1..=9).map(member).collect();
        let view = party_view(&world, None);
        assert_eq!(view.members.len(), 8, "a janela mostra só os outros 8, o próprio jogador vem do player-status orb");
        assert!(view.members.iter().all(|m| m.object_id != 1));
        assert_eq!(view.members.iter().map(|m| m.object_id).collect::<Vec<_>>(), (2..=9).collect::<Vec<_>>());
    }

    #[test]
    fn the_loading_screen_stretches_the_localized_texture_over_the_viewport() {
        let mut ui = game_ui((1558, 933));
        assert!(ui.loading_frame().quads.is_empty());
        ui.loading_texture =
            loading_texture(&parse_ini(include_str!("../../../crates/killer-ui/tests/fixtures/localization.ini")));
        let list = ui.loading_frame();
        let q = &list.quads[0];
        assert_eq!((q.texture.as_str(), q.w, q.h, q.uw, q.vh), ("L2Font-e.loading02-e", 1558.0, 933.0, 1024.0, 768.0));
    }

    #[test]
    fn the_cast_gauge_is_drawn_under_my_label_while_the_setup_gauge_runs() {
        let ui = game_ui((1558, 933));
        let now = Instant::now();
        let mut world = world();
        world.gauge = Some(killer_world::Gauge {
            setup: l2_net::game::packets::SetupGauge { color: 0, time: 5000, max_time: 5000 },
            started: now,
        });
        let views = world_views(&world, None, now);
        assert_eq!(views.gauges.len(), 1);
        assert_eq!((views.gauges[0].kind, views.gauges[0].max_ms), (0, 5000));
        let label = Label {
            object_id: 1,
            name: "Killer".into(),
            title: String::new(),
            store_sign: None, speech: None, modern_text_size: None, name_visible: true,
            name_color: [255; 3],
            title_color: [255; 3],
            screen: [400.0, 300.0], depth: 0.5,
            clan_crest: None,
            ally_crest: None,
            mark: None,
            relation_icons: Default::default(),
            broadcast_icon: None,
        };
        let list = ui.frame(&world, &[label], &[], None, &views);
        let back = with_texture(&list.quads, killer_ui::gauge::GAUGE_BACK[0]);
        assert_eq!((back[0].x, back[0].y), (350.0, 256.0));
        assert_eq!(with_texture(&list.quads, killer_ui::gauge::GAUGE_FILL[0]).len(), 1);
        assert!(gauge_views(&world, now + Duration::from_secs(6)).is_empty());
        let list = ui.frame(&world, &[], &[], None, &views);
        assert!(with_texture(&list.quads, killer_ui::gauge::GAUGE_BACK[0]).is_empty());
    }

    #[test]
    fn option_game_show_100c98b0_uses_runtime_ini_instead_of_the_old_checkbox_copy() {
        let mut ui = game_ui((1558, 933));
        ui.option_game_runtime = Some(crate::ui_options::GameOptionRuntime::new(
            parse_ini("[Game]\nTransparencyMode=False\nOldChatting=False\nUseJoystick=False\nPartyLooting=3\n"),
            parse_ini("[LanguageSet]\nLanguage=1\n"), &parse_ini(""), false, None,
        ));
        ui.desktop.window_mut(WindowId::OPTION).unwrap().visible = true;
        ui.sync_windows(&World::default(), None, Instant::now());
        assert_eq!(ui.keys.option.game_page.language.entries, ["English"]);
        assert_eq!(ui.keys.option.game_page.looting.selected, 3);
        assert!(!ui.keys.option.checked(0xf0f01475));
        assert!(!ui.keys.option.checked(0xf0f01472));
        assert!(!ui.shortcut.joypad_visible);
        ui.keys.option.game_page.looting.selected = 1;
        ui.sync_windows(&World::default(), None, Instant::now());
        assert_eq!(ui.keys.option.game_page.looting.selected, 1);
        ui.desktop.window_mut(WindowId::OPTION).unwrap().visible = false;
        ui.sync_windows(&World::default(), None, Instant::now());
        ui.desktop.window_mut(WindowId::OPTION).unwrap().visible = true;
        ui.sync_windows(&World::default(), None, Instant::now());
        assert_eq!(ui.keys.option.game_page.looting.selected, 3);
        assert_eq!(ui.keys.option.game_page.looting.entries.len(), 5);
    }

    #[test]
    fn option_show_100c98b0_refills_audio_once_per_open_without_a_fake_backend() {
        let mut ui = game_ui((1558, 933));
        for id in killer_ui::option_audio::CHOICE_IDS { ui.keys.sysstrings.insert(id, id.to_string()); }
        let world = World::default();
        for _ in 0..3 {
            ui.desktop.window_mut(WindowId::OPTION).unwrap().visible = true;
            ui.sync_windows(&world, None, Instant::now());
            for combo in &ui.keys.option.audio_page.combos {
                assert_eq!(combo.entries, killer_ui::option_audio::CHOICE_IDS.map(|id| id.to_string()));
                assert_eq!(combo.selected, -1);
                assert!(!combo.enabled);
            }
            ui.keys.option.audio_page.combos[0].selected = 3;
            ui.sync_windows(&world, None, Instant::now());
            assert_eq!(ui.keys.option.audio_page.combos[0].selected, 3);
            ui.desktop.window_mut(WindowId::OPTION).unwrap().visible = false;
            ui.sync_windows(&world, None, Instant::now());
        }
    }

    #[test]
    fn option_combo_10005d70_routes_hover_selection_and_focus_through_the_app() {
        let mut ui = game_ui((1558, 933));
        ui.keys.option = killer_ui::option::OptionView::with_god(|id| Some(id.to_string()), false, 0);
        ui.keys.option.set_font(&ui.small);
        ui.keys.option.press_tab(killer_ui::option::TAB_GAME);
        ui.keys.option.game_page.append_choices(Some(1), 1, |id| Some(id.to_string()));
        let window = ui.desktop.window_mut(WindowId::OPTION).unwrap();
        window.visible = true;
        window.rect.x = 400;
        window.rect.y = 100;
        let rect = window.rect;
        let key = killer_ui::option::OptionCombo { page: killer_ui::option::TAB_GAME, index: 1 };
        assert!(ui.mouse_down(rect.x + 100, rect.y + 425));
        ui.mouse_up();
        assert!(ui.keys.option.combo(key).unwrap().open);
        ui.mouse_move(rect.x + 100, rect.y + 510);
        assert_eq!(ui.window_under(rect.x + 100, rect.y + 510), Some((WindowId::OPTION, 100, 510)));
        assert!(ui.mouse_down(rect.x + 100, rect.y + 510));
        ui.mouse_up();
        assert_eq!(ui.keys.option.game_page.looting.selected, 4);
        assert!(!ui.keys.option.game_page.looting.open);
        assert!(ui.take_commands().is_empty());
        ui.mouse_down(rect.x + 100, rect.y + 425);
        ui.mouse_up();
        ui.mouse_down(0, 0);
        assert!(!ui.keys.option.game_page.looting.open);
        assert_eq!(ui.keys.option.game_page.looting.selected, 4);
    }

    #[test]
    fn pressing_a_painted_button_plays_click_01_and_a_disabled_one_click_failed() {
        let mut ui = game_ui((1558, 933));
        ui.frame(&world(), &[], &[], None, &WorldViews::default());
        assert!(ui.button_map.borrow().iter().any(|(id, _)| *id == WindowId::MENU));
        assert!(ui.mouse_down(1351 + 60, 781 + 10));
        let sounds: Vec<_> = ui.take_commands().into_iter().filter(|command| matches!(command, UiCommand::InterfaceSound(_))).collect();
        assert_eq!(sounds, vec![UiCommand::InterfaceSound("InterfaceSound.click_01")]);
        ui.mouse_up();
        ui.take_commands();
        assert!(ui.mouse_down(1351 + 168, 781 + 40));
        assert!(!ui.take_commands().iter().any(|command| matches!(command, UiCommand::InterfaceSound(_))));
    }

    #[test]
    fn the_target_close_sends_request_target_cancel_one_and_a_frame_close_plays_window_close_on_release() {
        let mut ui = game_ui((1558, 933));
        if let Some(window) = ui.desktop.window_mut(WindowId::TARGET) { window.visible = true; }
        let rect = ui.desktop.window(WindowId::TARGET).unwrap().rect;
        let close = killer_ui::target::close_button().offset(rect.x, rect.y);
        ui.frame(&world(), &[], &[], None, &WorldViews::default());
        assert!(ui.mouse_down(close.x + 2, close.y + 2));
        let commands = ui.take_commands();
        assert!(commands.contains(&UiCommand::Net(killer_world::NetCommand::TargetUnselect)));
        assert!(commands.contains(&UiCommand::InterfaceSound("itemsound.window_close")));
        ui.mouse_up();
        if let Some(window) = ui.desktop.window_mut(WindowId::MAIN) { window.visible = true; }
        let main = ui.desktop.window(WindowId::MAIN).unwrap().rect;
        let close = killer_ui::mainwnd::close_button().offset(main.x, main.y);
        ui.frame(&world(), &[], &[], None, &WorldViews::default());
        assert!(ui.mouse_down(close.x + 2, close.y + 2));
        ui.take_commands();
        ui.mouse_up();
        let sounds: Vec<_> = ui.take_commands().into_iter().filter(|command| matches!(command, UiCommand::InterfaceSound(_))).collect();
        assert_eq!(sounds, vec![UiCommand::InterfaceSound("itemsound.window_close")]);
    }

    #[test]
    fn the_menu_window_sits_at_the_ini_position_and_reports_the_released_button() {
        let mut ui = game_ui((1558, 933));
        let window = ui.desktop.window(WindowId::MENU).unwrap();
        assert_eq!(window.rect, Rect::new(1351, 781, 173, 46));
        assert_eq!(window.grip, Rect::new(0, 0, 12, 46));
        let bare = GameUi::new(
            font("s"),
            font("l"),
            Options::default(),
            WindowsInfo::default(),
            ChatFilter::from_ini(&parse_ini(CHAT_FILTER)),
            (1280, 720),
        ).unwrap();
        assert_eq!(bare.desktop.window(WindowId::MENU).unwrap().rect, Rect::new(1280 - 173, 720 - 46, 173, 46));
        let list = ui.frame(&world(), &[], &[], None, &WorldViews::default());
        let button = with_texture(&list.quads, killer_ui::menu::BUTTON_TEXTURES[1]);
        assert_eq!((button[0].x, button[0].y), (1351.0 + 56.0, 781.0 + 6.0));
        assert!(ui.mouse_down(1351 + 60, 781 + 10));
        assert_eq!(ui.desktop.dragging(), None);
        let list = ui.frame(&world(), &[], &[], None, &WorldViews::default());
        assert_eq!(with_texture(&list.quads, "L2UI_ch3.MenuIcon.MenuButton2_down").len(), 1);
        assert_eq!(ui.mouse_up(), None);
        let world = world();
        assert_eq!(ui.menu_released(&world), Some(KeyAction::Consumed));
        assert_eq!(crate::ui_keys::silent(ui.take_commands()), vec![UiCommand::Net(killer_world::NetCommand::RequestItemList)]);
        assert_eq!(ui.menu_released(&world), None);
        assert!(ui.mouse_down(1351 + 60, 781 + 10));
        ui.mouse_move(1351 + 100, 781 + 10);
        ui.mouse_up();
        assert_eq!(ui.menu_released(&world), None);
        assert!(ui.mouse_down(1351 + 5, 781 + 20));
        assert_eq!(ui.desktop.dragging(), Some(WindowId::MENU));
        ui.mouse_move(1346, 801);
        ui.mouse_up();
        assert_eq!(ui.menu_released(&world), None);
        assert_eq!(ui.desktop.window(WindowId::MENU).unwrap().rect, Rect::new(1341, 781, 173, 46));
        assert!(!ui.desktop.window(WindowId::MAIN).unwrap().visible);
        ui.menu_button(MenuButton::CharacterStatus, &world);
        assert!(ui.desktop.window(WindowId::MAIN).unwrap().visible);
        ui.menu_button(MenuButton::CharacterStatus, &world);
        assert!(!ui.desktop.window(WindowId::MAIN).unwrap().visible);
        assert!(!ui.desktop.window(crate::ui_keys::SYSMENU_WINDOW).unwrap().visible);
        ui.menu_button(MenuButton::System, &world);
        assert!(ui.desktop.window(crate::ui_keys::SYSMENU_WINDOW).unwrap().visible);
        ui.menu_button(MenuButton::Map, &world);
        assert_eq!(crate::ui_keys::silent(ui.take_commands()), vec![UiCommand::Net(killer_world::NetCommand::ShowMinimap)]);
    }

    #[test]
    fn the_abnormal_window_starts_at_the_ini_position_with_a_twelve_by_twenty_six_grip() {
        let ui = game_ui((1558, 933));
        let window = ui.desktop.window(WindowId::ABNORMAL_STATUS).unwrap();
        assert_eq!(window.rect, Rect::new(381, 17, 252, 50));
        assert_eq!(window.grip, Rect::new(0, 0, 12, 26));
        let bare = GameUi::new(
            font("s"),
            font("l"),
            Options::default(),
            WindowsInfo::default(),
            ChatFilter::from_ini(&parse_ini(CHAT_FILTER)),
            (1280, 720),
        ).unwrap();
        assert_eq!(bare.desktop.window(WindowId::ABNORMAL_STATUS).unwrap().rect, Rect::new(172, 0, 252, 50));
        let list = ui.frame(&world(), &[], &[], None, &WorldViews::default());
        assert!(with_texture(&list.quads, ABNORMAL_BACK).is_empty());
    }

    #[test]
    fn the_abnormal_view_takes_icons_and_debuffs_from_skillgrp_and_counts_the_seconds_down() {
        let tables = skill_tables();
        let now = Instant::now();
        let mut world = world();
        world.buffs = vec![
            Buff { skill_id: 1204, level: 2, duration: 1200 },
            Buff { skill_id: 1069, level: 1, duration: 30 },
            Buff { skill_id: 9999, level: 1, duration: -1 },
        ];
        world.buffs_at = Some(now);
        world.short_buff = Some(ShortBuffStatusUpdate { skill_id: 2274, level: 1, duration: 15 });
        world.short_buff_at = Some(now);
        let view = abnormal_view(&world, Some(&tables), now + Duration::from_secs(10));
        assert_eq!(view.buffs.len(), 3);
        assert_eq!(
            view.buffs[0],
            BuffIcon { skill_id: 1204, level: 2, icon: Some("icon.skill1204".into()), remaining: Some(1190), debuff: false, ..Default::default() }
        );
        assert_eq!(
            view.buffs[1],
            BuffIcon { skill_id: 1069, level: 1, icon: Some("icon.skill1069".into()), remaining: Some(20), debuff: true, fade_ticks: 100 }
        );
        assert_eq!(view.buffs[2], BuffIcon { skill_id: 9999, level: 1, icon: None, remaining: None, debuff: false, ..Default::default() });
        assert_eq!(
            view.short_buff,
            Some(BuffIcon {
                skill_id: 2274,
                level: 1,
                icon: Some("icon.skill2274".into()),
                remaining: Some(5),
                debuff: false,
                fade_ticks: 100,
            })
        );
        assert!(view.etc.is_empty());
        assert_eq!(remaining_seconds(30, Some(now), now + Duration::from_secs(45)), Some(0));
        assert_eq!(remaining_seconds(30, None, now), Some(30));
        assert_eq!(skill_icon(Some(&tables), 1204, 7).as_deref(), Some("icon.skill1204"));
        assert_eq!(skill_icon(None, 1204, 1), None);
        let views = world_views(&world, Some(&tables), now);
        assert_eq!(views.abnormal.buffs.len(), 3);
        let mut ui = game_ui((1558, 933));
        ui.show_pet_window(&views);
        let window = ui.desktop.window(WindowId::ABNORMAL_STATUS).unwrap();
        assert_eq!(
            (window.rect, window.grip, window.visible),
            (Rect::new(381, 17, 90, 51), Rect::new(0, 0, 12, 51), true)
        );
        let list = ui.frame(&world, &[], &[], None, &views);
        let backs = with_texture(&list.quads, ABNORMAL_BACK);
        assert_eq!((backs.len(), backs[0].x, backs[0].y), (4, 393.0, 17.0));
        assert_eq!(with_texture(&list.quads, "icon.skill1204")[0].x, 394.0);
        assert_eq!(with_texture(&list.quads, DEBUFF_OVERLAY)[0].x, 394.0 + 26.0);
        assert_eq!(with_texture(&list.quads, "icon.skill2274")[0].y, 17.0 + 25.0 + 1.0);
    }

    #[test]
    fn hovering_a_buff_icon_shows_its_name_from_skillname_even_for_a_debuff_not_in_my_skill_list() {
        let mut tables = skill_tables();
        tables.skillname.insert(
            (1204, 2),
            l2_dat::SkillName {
                id: 1204,
                level: 2,
                name: "Haste".into(),
                description: "Increases attack speed.".into(),
                desc_add1: String::new(),
                desc_add2: String::new(),
            },
        );
        tables.skillname.insert(
            (1069, 1),
            l2_dat::SkillName {
                id: 1069,
                level: 1,
                name: "Poison".into(),
                description: "Poisons the target.".into(),
                desc_add1: String::new(),
                desc_add2: String::new(),
            },
        );
        let now = Instant::now();
        let mut world = world();
        world.buffs = vec![Buff { skill_id: 1204, level: 2, duration: 1200 }, Buff { skill_id: 1069, level: 1, duration: 30 }];
        world.buffs_at = Some(now);
        let mut ui = game_ui((1558, 933));
        ui.set_tab_names(&tables);
        ui.sync_windows(&world, Some(&tables), now);
        ui.show_pet_window(&world_views(&world, Some(&tables), now));
        let window = ui.desktop.window(WindowId::ABNORMAL_STATUS).unwrap().rect;
        ui.mouse_move(window.x + 20, window.y + 10);
        let tip = ui.keys.tooltip.clone().expect("hover no primeiro buff devia mostrar o tooltip");
        assert_eq!(tip.name, "Haste");
        assert_eq!(tip.description, "Increases attack speed.");
        ui.mouse_move(window.x + 20 + 26, window.y + 10);
        let tip = ui.keys.tooltip.clone().expect("hover no segundo buff (debuff, de outra fonte) devia mostrar o tooltip");
        assert_eq!(tip.name, "Poison");
        ui.mouse_move(window.x + 5, window.y + 5);
        assert!(ui.keys.tooltip.is_none(), "fora dos ícones não deve haver tooltip");
    }

    const MERCHANT: &str = include_str!("../../../crates/killer-ui/tests/fixtures/npc-7829.htm");
    const HEINE_PET: &str = include_str!("../../../crates/killer-ui/tests/fixtures/npc-heine-8067.htm");
    const AGIT_SALE: &str = include_str!("../../../crates/killer-ui/tests/fixtures/npc-agitsale1.htm");

    fn npc_html(html: &str, item_id: i32) -> NpcHtmlMessage {
        NpcHtmlMessage { npc_object_id: 2, html: html.to_string(), item_id }
    }

    fn recipe_scroll_ui(count: usize) -> GameUi {
        let mut ui = game_ui((1558, 933));
        for window in &mut ui.desktop.windows {
            window.visible = window.id == WindowId::RECIPE_BOOK;
        }
        for id in 0..count {
            ui.keys.recipes.book.push(killer_ui::recipe::BookEntry {
                recipe_id: id as i32 + 1,
                grid: killer_ui::item_grid::GridEntry { icon: format!("recipe{id}"), ..Default::default() },
            });
        }
        ui
    }

    #[test]
    fn recipe_book_1001ea50_scrolls_every_grid_size_and_selects_the_visible_recipe_through_game_ui() {
        use killer_ui::recipe::BOOK_GRID_RECT;
        for count in 0usize..=160 {
            let mut ui = recipe_scroll_ui(count);
            let rect = ui.desktop.window(WindowId::RECIPE_BOOK).unwrap().rect;
            let (x, y) = (rect.x + BOOK_GRID_RECT.x + 4, rect.y + BOOK_GRID_RECT.y + 4);
            let last = count.div_ceil(6).saturating_sub(8);
            assert!(ui.wheel_over(x, y, -100.0, &[]));
            assert_eq!(ui.keys.recipes.book.grid.scroll_row, last, "count {count}");
            let row = ui.keys.recipes.book.grid.scroll_row;
            assert!(ui.wheel_over(x, y, 0.5, &[]));
            assert_eq!(ui.keys.recipes.book.grid.scroll_row, row);
            if count > 0 {
                assert!(ui.keys.double_click(WindowId::RECIPE_BOOK, BOOK_GRID_RECT.x + 4, BOOK_GRID_RECT.y + 4, &World::default()));
                assert_eq!(ui.take_commands(), vec![UiCommand::Net(killer_world::NetCommand::RecipeItemMakeInfo { recipe_id: (last * 6 + 1) as i32 })]);
            }
            assert!(ui.wheel_over(x, y, 100.0, &[]));
            assert_eq!(ui.keys.recipes.book.grid.scroll_row, 0);
            assert!(!ui.wheel_over(rect.x + 10, rect.y + 35, -1.0, &[]));
        }
    }

    #[test]
    fn recipe_book_1001eca0_1001ece0_1002c270_arrows_and_slider_receive_real_mouse_events() {
        use killer_ui::recipe::BOOK_GRID_RECT;
        let mut ui = recipe_scroll_ui(160);
        let world = World::default();
        let rect = ui.desktop.window(WindowId::RECIPE_BOOK).unwrap().rect;
        let grid = BOOK_GRID_RECT.offset(rect.x, rect.y);
        let x = grid.x + grid.w - 4;
        let now = Instant::now();
        assert!(ui.mouse_down_at(x, grid.y + grid.h - 4, now, &world));
        assert!(ui.keys.recipes.book.grid.scroll.down_pressed);
        assert_eq!(ui.keys.recipes.book.grid.scroll_row, 0);
        ui.mouse_move(x - 40, grid.y + grid.h - 4);
        ui.mouse_up();
        assert_eq!(ui.keys.recipes.book.grid.scroll_row, 0);
        assert!(!ui.keys.recipes.book.grid.scroll.down_pressed);
        assert!(ui.mouse_down_at(x, grid.y + grid.h - 4, now + Duration::from_secs(1), &world));
        ui.mouse_up();
        assert_eq!(ui.keys.recipes.book.grid.scroll_row, 1);
        assert!(ui.mouse_down_at(x, grid.y + 4, now + Duration::from_secs(2), &world));
        ui.mouse_up();
        assert_eq!(ui.keys.recipes.book.grid.scroll_row, 0);
        let y = grid.y + killer_ui::scroll::BUTTON_SIZE + 2;
        assert!(ui.mouse_down_at(x, y, now + Duration::from_secs(3), &world));
        assert!(ui.keys.recipes.book.grid.scroll.slider.dragging);
        assert!(ui.mouse_move(rect.x - 50, y + 1000));
        assert_eq!(ui.keys.recipes.book.grid.scroll_row, 19);
        ui.mouse_up();
        assert!(!ui.keys.recipes.book.grid.scroll.slider.dragging);
        ui.mouse_move(x, y);
        assert_eq!(ui.keys.recipes.book.grid.scroll_row, 19);
        assert!(ui.take_commands().is_empty());
    }

    #[test]
    fn pet_and_private_shop_sliders_1002c270_receive_game_ui_mouse_move() {
        for id in [WindowId::PET_WINDOW, WindowId::PRIVATE_SHOP] {
            let mut ui = game_ui((1558, 933));
            for window in &mut ui.desktop.windows { window.visible = window.id == id; }
            let local = if id == WindowId::PET_WINDOW {
                ui.keys.pet.view.press_tab(killer_ui::pet_wnd::PAGE_ITEMS);
                ui.keys.pet.view.grid.update_rows(30);
                ui.keys.pet.view.grid.rect.offset(killer_ui::pet_wnd::PAGE_RECT.x, killer_ui::pet_wnd::PAGE_RECT.y)
            } else {
                ui.keys.private_shop.view.top_grid.update_rows(60);
                ui.keys.private_shop.view.top_grid.rect
            };
            let rect = ui.desktop.window(id).unwrap().rect;
            let (x, y) = (rect.x + local.x + local.w - 4, rect.y + local.y + killer_ui::scroll::BUTTON_SIZE + 2);
            assert!(ui.mouse_down_at(x, y, Instant::now(), &World::default()));
            assert!(ui.mouse_move(x, y + 1000));
            let grid = if id == WindowId::PET_WINDOW { &ui.keys.pet.view.grid } else { &ui.keys.private_shop.view.top_grid };
            assert!(grid.scroll.slider.dragging);
            assert!(grid.scroll_row > 0);
            ui.mouse_up();
            let grid = if id == WindowId::PET_WINDOW { &ui.keys.pet.view.grid } else { &ui.keys.private_shop.view.top_grid };
            assert!(!grid.scroll.slider.dragging);
        }
    }

    #[test]
    #[ignore]
    fn recipe_shop_10070a60_10070ae0_10070c00_paints_and_transacts_every_client_recipe() {
        let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let mut ui = GameUi::load(&mut client, (1024, 768)).unwrap();
        ui.set_tab_names(&tables);
        for window in &mut ui.desktop.windows { window.visible = false; }
        for id in [WindowId::RECIPE_SHOP, WindowId::RECIPE_SELL, WindowId::RECIPE_BUY] {
            let w=ui.desktop.window_mut(id).unwrap(); (w.rect.x,w.rect.y)=(0,0);
        }
        let mut world=World::default();
        let mut bank = killer_assets::textures::TextureBank::new();
        for (path,data) in ui.texture_bank.textures() { bank.insert_shared(&path,data); }
        let mut captures=DrawList::default();
        let mut seen=std::collections::HashSet::new();
        let mut now=Instant::now();
        for (batch,chunk) in tables.recipe_order.chunks(100).enumerate() {
            let packet=l2_net::game::RecipeShopManageList { common:batch%2==0,mp:100,max_mp:200,recipes:chunk.iter().map(|id|(*id,0)).collect(),items:Vec::new() };
            world.apply_world(&killer_world::WorldEvent::RecipeShopManageList(packet.clone()));
            ui.keys.recipes.world_event(&killer_world::WorldEvent::RecipeShopManageList(packet));
            ui.sync_windows(&world,Some(&tables),now);
            assert_eq!(ui.keys.recipes.shop.view.top.len(),chunk.len());
            if batch==0 { ui.keys.recipes.shop.view.paint(&mut captures,0,0,&ui.small,1.0); }
            for (i,id) in chunk.iter().enumerate() {
                ui.keys.recipes.shop.view.top_grid.set_first_row((i/6).min(chunk.len().div_ceil(6).saturating_sub(4)) as i32);
                let first=ui.keys.recipes.shop.view.top_grid.scroll_row;
                let (sx,sy)=killer_ui::inventory::grid_slot_pos_cols(i-first*6,6);
                assert!(ui.keys.recipes.shop.double_click(11+sx,50+sy));
                ui.tick_dialog(&mut world,Some(&tables),now);
                assert_eq!(ui.dialog.as_ref().unwrap().answer,DialogAnswer::RecipeShop);
                assert!(!ui.mouse_down_at(500,500,now,&world));
                assert_eq!(ui.dialog.as_ref().unwrap().answer,DialogAnswer::RecipeShop);
                ui.dialog.as_mut().unwrap().view.input.as_mut().unwrap().text=(i*1001).to_string();
                assert_eq!(ui.key(InputKey::Enter),KeyAction::Consumed);
                assert_eq!((ui.keys.recipes.shop.view.bottom[i].recipe_id,ui.keys.recipes.shop.view.bottom[i].cost),(*id,(i*1001) as i32));
                seen.insert(*id);
                now+=Duration::from_secs(1);
            }
            let mut list=DrawList::default();
            for row in 0..=chunk.len().div_ceil(6).saturating_sub(4) {
                ui.keys.recipes.shop.view.top_grid.set_first_row(row as i32);
                ui.keys.recipes.shop.view.bottom_grid.set_first_row(row as i32);
                ui.keys.recipes.shop.view.paint(&mut list,0,0,&ui.small,1.0);
            }
            if batch==0 { ui.keys.recipes.shop.view.paint(&mut captures,256,0,&ui.small,1.0); }
            ui.keys.recipes.shop.mouse_down(180,380);
            ui.keys.recipes.shop.mouse_up(Some((WindowId::RECIPE_SHOP,180,380)),&mut ui.desktop);
            assert!(ui.take_commands().iter().any(|c|matches!(c,UiCommand::Net(killer_world::NetCommand::RecipeShopListSet { items }) if items.len()==chunk.len())));
            let sell=l2_net::game::RecipeShopSellList { manufacturer_id:77,mp:100,max_mp:200,adena:1234567,items:chunk.iter().enumerate().map(|(i,id)|l2_net::game::ManufactureItem { recipe_id:*id,unknown:0,cost:(i*1001) as i32 }).collect() };
            world.apply_world(&killer_world::WorldEvent::RecipeShopSellList(sell.clone()));
            ui.keys.recipes.world_event(&killer_world::WorldEvent::RecipeShopSellList(sell));
            ui.sync_windows(&world,Some(&tables),now);
            for row in 0..=chunk.len().saturating_sub(5) {
                ui.keys.recipes.sell.paint(&mut list,0,0,&ui.small,1.0);
                assert_eq!(ui.keys.recipes.sell.first,row);
                ui.wheel_over(20,65,-1.0,&[]);
            }
            if batch==0 { ui.keys.recipes.sell.paint(&mut captures,512,0,&ui.small,1.0); }
            for id in chunk {
                let info=l2_net::game::RecipeShopItemInfo { manufacturer_id:77,recipe_id:*id,mp:100,max_mp:200,status:1,cost:4321 };
                world.apply_world(&killer_world::WorldEvent::RecipeShopItemInfo(info));
                ui.keys.recipes.world_event(&killer_world::WorldEvent::RecipeShopItemInfo(info));
                ui.sync_windows(&world,Some(&tables),now);
                assert_eq!(ui.keys.recipes.buy.materials.len(),tables.recipes[id].materials.len());
                assert_eq!(ui.keys.recipes.buy.frame.title,tables.sysstring[&663]);
                ui.keys.recipes.buy.paint(&mut list,0,0,&ui.small,1.0);
                ui.keys.recipes.buy_down(20,380);
                ui.keys.recipes.buy_up(Some((WindowId::RECIPE_BUY,20,380)),&mut ui.desktop);
                assert!(ui.take_commands().iter().any(|c|matches!(c,UiCommand::Net(killer_world::NetCommand::RecipeShopMakeDo { manufacturer_id:77,recipe_id,cost:4321 }) if recipe_id==id)));
            }
            if batch==0 { ui.keys.recipes.buy.paint(&mut captures,768,0,&ui.small,1.0); }
            ui.desktop.window_mut(WindowId::RECIPE_BUY).unwrap().visible=false;
            for quad in &list.quads { bank.texture(client.catalog(),&quad.texture); }
        }
        assert_eq!(seen.len(),tables.recipes.len());
        for quad in &captures.quads { bank.texture(client.catalog(),&quad.texture); }
        assert!(bank.missing().next().is_none(),"{:?}",bank.missing().collect::<Vec<_>>());
        let mut headless=killer_render::headless::Headless::new(1024,401).unwrap();
        let mut source=killer_assets::textures::BankSource { bank:&mut bank,catalog:client.catalog() };
        headless.renderer_mut().set_ui(&captures,&mut source);
        let camera=killer_render::camera::Camera { eye:glam::vec3(0.0,-300.0,0.0),target:glam::Vec3::ZERO,fov_x_deg:60.0,near:1.0,far:5000.0 };
        let rgba=headless.render(&camera,&killer_render::environment::Environment::default()).unwrap();
        killer_assets::textures::write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-recipe-shop.png"),1024,401,&rgba).unwrap();
        eprintln!("{} receitas: preço por diálogo GameUi, iniciar loja, lista de compra e fabricação; todas as texturas presentes",seen.len());
    }

    #[test]
    #[ignore]
    fn recipe_tree_100f4ea0_paints_every_client_tree_and_scrolls_through_game_ui() {
        fn expand(node: &mut killer_ui::recipe_tree::RecipeNode) -> usize {
            node.expanded = !node.children.is_empty();
            1 + node.children.iter_mut().map(expand).sum::<usize>()
        }
        let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let mut ui = GameUi::load(&mut client, (1024, 768)).unwrap();
        ui.set_tab_names(&tables);
        assert!(!ui.keys.recipes.tree.close.label.is_empty());
        let world = World::default();
        for window in &mut ui.desktop.windows { window.visible = window.id == WindowId::RECIPE_MANUFACTURE; }
        for id in [WindowId::RECIPE_MANUFACTURE, WindowId::RECIPE_TREE] {
            let rect = &mut ui.desktop.window_mut(id).unwrap().rect;
            (rect.x, rect.y) = (0, 0);
        }
        let mut bank = killer_assets::textures::TextureBank::new();
        for (path, data) in ui.texture_bank.textures() { bank.insert_shared(&path, data); }
        let mut most = (0, killer_ui::recipe_tree::RecipeTreeView::new(|_| None));
        let mut nodes = 0;
        for id in &tables.recipe_order {
            ui.desktop.window_mut(WindowId::RECIPE_TREE).unwrap().visible = false;
            ui.keys.recipes.manufacture.recipe_id = *id;
            ui.keys.recipes.manufacture_mouse_down(220, 85);
            ui.keys.recipes.manufacture_mouse_up(Some((WindowId::RECIPE_MANUFACTURE, 220, 85)), &mut ui.desktop);
            ui.sync_windows(&world, Some(&tables), Instant::now());
            assert!(ui.desktop.window(WindowId::RECIPE_TREE).unwrap().visible);
            let count = expand(ui.keys.recipes.tree.root.as_mut().unwrap());
            nodes += count;
            ui.keys.recipes.tree.update_scroll();
            if count > most.0 { most = (count, ui.keys.recipes.tree.clone()); }
            loop {
                let mut list = DrawList::default();
                ui.keys.recipes.tree.paint(&mut list, 0, 0, &ui.small, 1.0);
                for quad in &list.quads { bank.texture(client.catalog(), &quad.texture); }
                let before = ui.keys.recipes.tree.offset;
                assert!(ui.wheel_over(30, 120, -1.0, &[]));
                if ui.keys.recipes.tree.offset == before { break; }
            }
        }
        assert_eq!(nodes, 15991);
        assert!(bank.missing().next().is_none(), "{:?}", bank.missing().collect::<Vec<_>>());
        ui.keys.recipes.tree = most.1;
        let mut captures = DrawList::default();
        ui.keys.recipes.tree.paint(&mut captures, 0, 0, &ui.small, 1.0);
        let (x, y) = (245, 125);
        assert!(ui.mouse_down_at(x, y, Instant::now(), &world));
        assert!(ui.keys.recipes.tree.scroll.slider.dragging);
        assert!(ui.mouse_move(x, 10000));
        ui.mouse_up();
        assert_eq!(ui.keys.recipes.tree.offset, ui.keys.recipes.tree.scroll.slider.total - 254);
        ui.keys.recipes.tree.paint(&mut captures, 256, 0, &ui.small, 1.0);
        ui.wheel_over(30, 120, 10000.0, &[]);
        ui.mouse_down_at(23, 114, Instant::now() + Duration::from_secs(2), &world);
        ui.mouse_up();
        assert!(!ui.keys.recipes.tree.root.as_ref().unwrap().expanded);
        ui.keys.recipes.tree.paint(&mut captures, 512, 0, &ui.small, 1.0);
        ui.mouse_down_at(110, 380, Instant::now() + Duration::from_secs(3), &world);
        ui.mouse_up();
        assert!(!ui.desktop.window(WindowId::RECIPE_TREE).unwrap().visible);
        assert!(ui.keys.recipes.tree.root.is_none());
        let mut headless = killer_render::headless::Headless::new(768, 401).unwrap();
        let mut source = killer_assets::textures::BankSource { bank: &mut bank, catalog: client.catalog() };
        headless.renderer_mut().set_ui(&captures, &mut source);
        let camera = killer_render::camera::Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let rgba = headless.render(&camera, &killer_render::environment::Environment::default()).unwrap();
        killer_assets::textures::write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-recipe-tree.png"), 768, 401, &rgba).unwrap();
        eprintln!("{} árvores, {nodes} nós, todas as texturas e rolagem pela GameUi; maior árvore {} nós", tables.recipes.len(), most.0);
    }

    #[test]
    #[ignore]
    fn recipe_book_100f5fa0_scrolls_and_paints_every_client_recipe() {
        let root = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let mut ids: Vec<_> = tables.recipes.keys().copied().collect();
        ids.sort_unstable();
        assert!(!ids.is_empty());
        let mut ui = GameUi::load(&mut client, (1024, 768)).unwrap();
        ui.set_tab_names(&tables);
        for window in &mut ui.desktop.windows { window.visible = window.id == WindowId::RECIPE_BOOK; }
        ui.desktop.window_mut(WindowId::RECIPE_MANUFACTURE).unwrap().rect.x = 0;
        ui.desktop.window_mut(WindowId::RECIPE_MANUFACTURE).unwrap().rect.y = 0;
        let mut world = World::default();
        let mut seen = std::collections::HashSet::new();
        let mut captures = DrawList::default();
        let mut captured = 0;
        let mut renderer_bank = killer_assets::textures::TextureBank::new();
        for (path, data) in ui.texture_bank.textures() { renderer_bank.insert_shared(&path, data); }
        for chunk in ids.chunks(160) {
            let book = l2_net::game::RecipeBookItemList {
                common: false, max_mp: 300, recipes: chunk.iter().map(|id| (*id, 0)).collect(),
            };
            world.recipes.book = Some(book.clone());
            ui.keys.recipes.world_event(&killer_world::WorldEvent::RecipeBookItemList(book));
            ui.keys.recipes.sync(&world, Some(&tables), &mut ui.desktop);
            assert_eq!(ui.keys.recipes.book.grid.scroll_row, 0);
            let last = chunk.len().div_ceil(6).saturating_sub(8);
            for row in 0..=last {
                let mut list = DrawList::default();
                ui.keys.recipes.book.paint(&mut list, 0, 0, &ui.small, 1.0);
                for (index, entry) in ui.keys.recipes.book.entries.iter().enumerate().skip(row * 6).take(48) {
                    assert_eq!(entry.recipe_id, chunk[index]);
                    assert!(!entry.grid.icon.is_empty());
                    assert!(list.quads.iter().any(|q| q.texture == entry.grid.icon));
                    let (sx, sy) = killer_ui::inventory::grid_slot_pos_cols(index - row * 6, 6);
                    assert_eq!(ui.keys.recipes.book.hit(9 + sx + 2, 48 + sy + 2), Some(killer_ui::recipe::BookHit::Slot(index)));
                    seen.insert(entry.recipe_id);
                }
                for quad in &list.quads { renderer_bank.texture(client.catalog(), &quad.texture); }
                if captured < 3 && (row == 0 || row == last) {
                    ui.keys.recipes.book.paint(&mut captures, captured * 256, 0, &ui.small, 1.0);
                    captured += 1;
                }
                assert!(ui.wheel_over(15, 55, -1.0, &[]));
                assert_eq!(ui.keys.recipes.book.grid.scroll_row, (row + 1).min(last));
            }
        }
        assert_eq!(seen.len(), ids.len());
        assert!(renderer_bank.missing().next().is_none());
        let mut headless = killer_render::headless::Headless::new(768, 401).unwrap();
        let mut source = killer_assets::textures::BankSource { bank: &mut renderer_bank, catalog: client.catalog() };
        headless.renderer_mut().set_ui(&captures, &mut source);
        let camera = killer_render::camera::Camera {
            eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO,
            fov_x_deg: 60.0, near: 1.0, far: 5000.0,
        };
        let rgba = headless.render(&camera, &killer_render::environment::Environment::default()).unwrap();
        killer_assets::textures::write_png(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-recipe-scroll.png"),
            768, 401, &rgba,
        ).unwrap();
        eprintln!("{} receitas percorridas, pintadas e com hit-test conferido", seen.len());
    }

    #[test]
    fn the_pet_name_dialog_100dcf30_takes_text_and_checks_the_length_like_1000a6ac() {
        let mut ui = game_ui((1558, 933));
        let world = World::default();
        ui.open_dialog(&Dialog::PetName, &world, None, Instant::now());
        let dialog = ui.dialog.as_ref().unwrap();
        assert_eq!((dialog.view.dialog_type, dialog.view.width()), (PET_NAME_DIALOG_TYPE, 256));
        assert!(!dialog.view.keypad());
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Consumed);
        assert!(ui.dialog.is_some());
        assert!(ui.take_commands().iter().any(|c| matches!(c, UiCommand::ColoredSystemMessage { id: PET_NAME_INVALID_SYSMSG, .. })));
        for c in "Wo\u{4e2d}".chars() { ui.key(InputKey::Char(c)); }
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Consumed);
        assert!(ui.dialog.is_some());
        assert!(ui.dialog.as_ref().unwrap().view.input.as_ref().unwrap().text.is_empty());
        assert!(ui.take_commands().iter().any(|c| matches!(c, UiCommand::ColoredSystemMessage { id: PET_NAME_INVALID_SYSMSG, .. })));
        for c in "Wolfie The Seventeen".chars() { ui.key(InputKey::Char(c)); }
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Consumed);
        assert!(ui.dialog.is_some());
        assert!(ui.take_commands().iter().any(|c| matches!(c, UiCommand::ColoredSystemMessage { id: PET_NAME_TOO_LONG_SYSMSG, .. })));
        for _ in 0.."The Seventeen".len() { ui.key(InputKey::Backspace); }
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Consumed);
        assert!(ui.dialog.is_none());
        assert!(ui.take_commands().iter().any(|c| matches!(c, UiCommand::Net(killer_world::NetCommand::ChangePetName { name }) if name == "Wolfie ")));
    }

    #[test]
    fn the_clan_title_dialog_checks_the_target_and_the_length_before_sending() {
        let mut ui = game_ui((1558, 933));
        let world = World::default();
        ui.keys.target_player = Some("Amigo".into());
        ui.open_dialog(&Dialog::GiveTitle { target: "Amigo".into() }, &world, None, Instant::now());
        assert_eq!(ui.dialog.as_ref().unwrap().view.dialog_type, DIALOG_TYPE_GIVE_TITLE);
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Consumed);
        assert!(ui.dialog.is_some());
        assert!(ui.take_commands().iter().any(|c| matches!(c, UiCommand::ColoredSystemMessage { id: TITLE_INVALID_SYSMSG, .. })));
        for c in "A title that is far too long".chars() { ui.key(InputKey::Char(c)); }
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Consumed);
        assert!(ui.dialog.is_some());
        assert!(ui.take_commands().iter().any(|c| matches!(c, UiCommand::ColoredSystemMessage { id: TITLE_LENGTH_SYSMSG, .. })));
        for _ in 0.."A title that is far too long".len() - 5 { ui.key(InputKey::Backspace); }
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Consumed);
        assert!(ui.dialog.is_none());
        assert!(ui.take_commands().iter().any(|c| matches!(c, UiCommand::Net(killer_world::NetCommand::GiveNickName { target, title }) if target == "Amigo" && title == "A tit")));
        ui.keys.target_player = None;
        ui.open_dialog(&Dialog::GiveTitle { target: "Amigo".into() }, &world, None, Instant::now());
        for c in "Hi".chars() { ui.key(InputKey::Char(c)); }
        ui.key(InputKey::Enter);
        assert!(ui.dialog.is_none());
        assert!(ui.take_commands().iter().all(|c| !matches!(c, UiCommand::Net(_))));
    }

    #[test]
    fn the_clan_war_name_dialogs_and_the_delete_dialogs_send_their_packets() {
        let mut ui = game_ui((1558, 933));
        let world = World::default();
        for start in [true, false] {
            ui.open_dialog(&Dialog::PledgeWarName { start }, &world, None, Instant::now());
            for c in "Rivals".chars() { ui.key(InputKey::Char(c)); }
            ui.key(InputKey::Enter);
            let expected = if start { killer_world::NetCommand::StartPledgeWar { pledge_name: "Rivals".into() } } else { killer_world::NetCommand::StopPledgeWar { pledge_name: "Rivals".into() } };
            assert!(ui.dialog.is_none());
            assert!(ui.take_commands().contains(&UiCommand::Net(expected)));
        }
        assert_eq!(dialog_command(DialogAnswer::DeleteTitle { target: "Amigo".into() }, true), Some(NetCommand::GiveNickName { target: "Amigo".into(), title: String::new() }));
        assert_eq!(dialog_command(DialogAnswer::DeleteCrest, true), Some(NetCommand::SetPledgeCrest { data: Vec::new() }));
        assert_eq!(dialog_command(DialogAnswer::DeleteInsignia, true), Some(NetCommand::ExSetPledgeCrestLarge { data: Vec::new() }));
        assert_eq!(dialog_command(DialogAnswer::DeleteCrest, false), None);
        assert_eq!(dialog_command(DialogAnswer::SystemMenu { restart: true }, true), Some(NetCommand::Restart));
        assert_eq!(dialog_command(DialogAnswer::SystemMenu { restart: false }, true), Some(NetCommand::Logout));
        assert_eq!(dialog_command(DialogAnswer::SystemMenu { restart: true }, false), None);
    }

    #[test]
    fn the_crest_dialogs_ask_for_the_file_after_the_notice_and_hand_the_typed_path_to_the_app() {
        let mut ui = game_ui((1558, 933));
        let world = World::default();
        for (large, notice, file) in [(false, DIALOG_TYPE_CREST_NOTICE, DIALOG_TYPE_CREST_FILE), (true, DIALOG_TYPE_INSIGNIA_NOTICE, DIALOG_TYPE_INSIGNIA_FILE)] {
            ui.open_dialog(&Dialog::CrestNotice { large }, &world, None, Instant::now());
            assert_eq!(ui.dialog.as_ref().unwrap().view.dialog_type, notice);
            ui.finish_dialog(true);
            assert_eq!(ui.keys.dialogs, vec![Dialog::CrestFile { large }]);
            ui.keys.dialogs.clear();
            ui.open_dialog(&Dialog::CrestFile { large }, &world, None, Instant::now());
            assert_eq!(ui.dialog.as_ref().unwrap().view.dialog_type, file);
            for c in "crest.bmp".chars() { ui.key(InputKey::Char(c)); }
            ui.key(InputKey::Enter);
            let kind = if large { killer_assets::crest::CrestKind::Large } else { killer_assets::crest::CrestKind::Clan };
            assert_eq!(ui.take_commands(), vec![UiCommand::CrestFile { kind, path: "crest.bmp".into() }]);
        }
    }

    #[test]
    fn allydismiss_asks_the_name_then_confirms_before_request_oust_ally_1000a5a0() {
        let mut ui = game_ui((1558, 933));
        let world = World::default();
        ui.open_dialog(&Dialog::AllyDismissName, &world, None, Instant::now());
        assert_eq!(ui.dialog.as_ref().unwrap().view.dialog_type, DIALOG_TYPE_ALLY_DISMISS_NAME);
        ui.key(InputKey::Enter);
        assert!(ui.dialog.is_none() && ui.keys.dialogs.is_empty());
        ui.open_dialog(&Dialog::AllyDismissName, &world, None, Instant::now());
        for c in "Rivals".chars() { ui.key(InputKey::Char(c)); }
        ui.key(InputKey::Enter);
        assert!(ui.take_commands().iter().all(|c| !matches!(c, UiCommand::Net(_))));
        assert_eq!(ui.keys.dialogs, vec![Dialog::AllyDismissConfirm { pledge_name: "Rivals".into() }]);
        assert_eq!(dialog_command(DialogAnswer::AllyDismiss { pledge_name: "Rivals".into() }, true), Some(NetCommand::AllyDismiss { pledge_name: "Rivals".into() }));
        assert_eq!(dialog_command(DialogAnswer::AllyDismiss { pledge_name: "Rivals".into() }, false), None);
        assert_eq!(dialog_command(DialogAnswer::JoinMpcc, true), Some(NetCommand::ExAcceptJoinMpcc { accept: true }));
        assert_eq!(dialog_command(DialogAnswer::JoinMpcc, false), Some(NetCommand::ExAcceptJoinMpcc { accept: false }));
        ui.open_dialog(&Dialog::JoinMpcc { name: "Leader".into() }, &world, None, Instant::now());
        assert_eq!(ui.dialog.as_ref().unwrap().view.dialog_type, DIALOG_TYPE_JOIN_MPCC);
    }

    #[test]
    fn server_close_in_the_game_asks_dialog_1d_with_message_127_and_ok_only_10076ae0() {
        let world = World::default();
        let mut tables = tables();
        tables.systemmsg.insert(127, message(127, "Your connection has been disconnected from the server. Please try again."));
        for reason in 0..=2 {
            let (view, answer, window) = dialog_request(&Dialog::Disconnected { reason }, &world, Some(&tables));
            assert_eq!((view.dialog_type, window), (DIALOG_TYPE_DISCONNECTED, WindowId::DIALOG));
            assert_eq!(view.message, "Your connection has been disconnected from the server. Please try again.");
            assert_eq!(answer, DialogAnswer::Disconnected);
            assert!(killer_ui::dialog::SINGLE_BUTTON_TYPES.contains(&view.dialog_type));
        }
        let (view, _, _) = dialog_request(&Dialog::Disconnected { reason: 5 }, &world, Some(&tables));
        assert_eq!(view.message, "");
        assert_eq!(dialog_command(DialogAnswer::Disconnected, true), None);
    }

    #[test]
    fn a_dialog_for_the_same_box_cancels_the_open_one_like_the_gara_dialog_set_info_10008b90() {
        let mut tables = tables();
        tables.systemmsg.insert(TRADE_REQUEST_SYSMSG, message(TRADE_REQUEST_SYSMSG, "$s1 requests a trade."));
        let mut ui = game_ui((1558, 933));
        let mut world = world();
        let now = Instant::now();
        world.dialogs.push(Dialog::TradeRequest { requestor_id: 3 });
        assert_eq!(ui.tick_dialog(&mut world, Some(&tables), now), None);
        assert_eq!(ui.dialog.as_ref().unwrap().answer, DialogAnswer::Trade);
        world.dialogs.push(Dialog::Confirm(SystemMessage { id: 1510, args: vec![SystemMessageArg::Text("x".into())] }));
        assert_eq!(ui.tick_dialog(&mut world, Some(&tables), now), None);
        assert_eq!(ui.dialog.as_ref().unwrap().answer, DialogAnswer::Confirm { message_id: 1510 }, "caixas diferentes coexistem");
        assert_eq!(ui.parked_dialog.as_ref().unwrap().answer, DialogAnswer::Trade);
        ui.close_dialog();
        assert_eq!(ui.dialog.as_ref().unwrap().answer, DialogAnswer::Trade, "a outra caixa volta a ser a ativa");
        assert!(ui.parked_dialog.is_none() && ui.desktop.window(WindowId::DIALOG_BOTTOM).unwrap().visible);
        world.dialogs.pending.clear();
        world.dialogs.push(Dialog::FriendRequest { requestor_name: "A".into() });
        assert_eq!(ui.tick_dialog(&mut world, Some(&tables), now), Some(KeyAction::Dialog(DialogAnswer::Trade, false)));
        assert!(ui.dialog.is_none());
        assert_eq!(ui.tick_dialog(&mut world, Some(&tables), now), None);
        assert_eq!(ui.dialog.as_ref().unwrap().answer, DialogAnswer::Friend);
        world.dialogs.push(Dialog::Confirm(SystemMessage { id: 1510, args: vec![SystemMessageArg::Text("x".into())] }));
        ui.close_dialog();
        assert_eq!(ui.tick_dialog(&mut world, Some(&tables), now), None);
        assert_eq!(ui.dialog.as_ref().unwrap().answer, DialogAnswer::Confirm { message_id: 1510 });
        world.dialogs.push(Dialog::Confirm(SystemMessage { id: 1510, args: vec![SystemMessageArg::Text("y".into())] }));
        assert_eq!(ui.tick_dialog(&mut world, Some(&tables), now), None, "o 0x46 com 0x5E6 é trocado sem cancelar");
        assert!(ui.dialog.is_none());
    }

    #[test]
    fn auto_log_on_locks_the_restart_button_of_the_system_menu() {
        let mut ui = game_ui((1558, 933));
        let mut locked = tables();
        locked.login_ini = killer_assets::login_ini::LoginIni::parse("", "[AutoLogOn]\nIsL2AutoLogOn=True\nL2ID=a\nL2Passwd=b\n");
        ui.set_tab_names(&locked);
        assert!(ui.keys.sysmenu.restart_locked);
        ui.keys.sysmenu_action(killer_ui::sysmenu::SysMenuButton::Restart, &mut ui.desktop);
        assert!(ui.keys.dialogs.is_empty());
        ui.keys.sysmenu_action(killer_ui::sysmenu::SysMenuButton::ExitGame, &mut ui.desktop);
        assert_eq!(ui.keys.dialogs, vec![Dialog::SystemMenu { restart: false }]);
        ui.set_tab_names(&tables());
        assert!(!ui.keys.sysmenu.restart_locked);
    }

    #[test]
    fn the_community_button_opens_the_board_and_presses_its_clan_tab() {
        let mut ui = game_ui((1024, 768));
        assert!(!ui.desktop.window(WindowId::BOARD).unwrap().visible);
        ui.open_community();
        assert!(ui.desktop.window(WindowId::BOARD).unwrap().visible);
        assert_eq!(ui.desktop.windows.last().map(|w| w.id), Some(WindowId::BOARD));
    }

    #[test]
    fn generated_board_receive_keeps_partial_pages_replaces_commands_and_focuses_each_packet() {
        use killer_world::WorldEvent;
        use l2_net::game::ShowBoard;
        let mut ui = game_ui((1024, 768));
        let world = World::default();
        let links: Vec<String> = (0..8).map(|i| format!("bypass command-{i}")).collect();
        ui.world_event(&WorldEvent::ShowBoard(Some(ShowBoard { links: links.clone(), html: "101\u{8}<html><body>First".into() })), &world);
        assert_eq!(ui.board.pages.list_count, 1);
        assert_eq!(ui.board.script.m_command.as_slice(), links);
        ui.board.script.m_b_btn_lock = true;
        ui.world_event(&WorldEvent::ShowBoard(None), &world);
        assert!(!ui.desktop.window(WindowId::BOARD).unwrap().visible);
        assert!(ui.board.script.m_b_btn_lock);
        assert_eq!(ui.board.pages.list_count, 1);
        assert_eq!(ui.board.script.m_command.as_slice(), links);
        for html in ["102\u{8} Second", "103\u{8} Third</body></html>", ""] {
            let at = ui.desktop.windows.iter().position(|w| w.id == WindowId::HELP).unwrap();
            let mut above = ui.desktop.windows.remove(at);
            above.visible = true;
            ui.desktop.windows.push(above);
            ui.board.script.m_b_btn_lock = true;
            ui.world_event(&WorldEvent::ShowBoard(Some(ShowBoard { links: vec![String::new();8], html: html.into() })), &world);
            assert!(ui.desktop.window(WindowId::BOARD).unwrap().visible);
            assert_eq!(ui.desktop.windows.last().unwrap().id, WindowId::BOARD);
            assert!(!ui.board.script.m_b_btn_lock);
            assert!(ui.board.script.m_command.iter().all(String::is_empty));
        }
        assert_eq!(ui.board.pages.list_count, 0);
        let text: String = ui.board.lines.iter().flat_map(|line| &line.spans).map(|span| span.text.as_str()).collect();
        assert_eq!(text, "First Second Third");
    }

    #[test]
    fn the_community_board_opens_on_show_board_and_sends_tab_favorite_and_link_actions() {
        let mut ui = game_ui((1558, 933));
        let board = ui.desktop.window(WindowId::BOARD).unwrap();
        assert!(!board.visible);
        assert_eq!(board.rect, Rect::new(441, 156, 646, 530));
        let mut links: Vec<String> = vec![String::new(); 8];
        links[7] = "bypass _bbsaddfav_List".into();
        ui.show_board(Some((&links, "<html><body><a action=\"bypass _bbsread_1\">Post</a></body></html>")));
        let window = ui.desktop.window(WindowId::BOARD).unwrap().clone();
        assert!(window.visible);
        assert_eq!(ui.desktop.windows.last().map(|w| w.id), Some(WindowId::BOARD));
        let list = ui.frame(&World::default(), &[], &[], None, &WorldViews::default());
        assert_eq!(with_texture(&list.quads, killer_ui::board::BACK_BODY).len(), 1);
        let (wx, wy) = (window.rect.x, window.rect.y);
        let tab = ui.board.tabs.buttons[3].paint.rect.offset(killer_ui::board::TABS_POS.0, killer_ui::board::TABS_POS.1);
        assert!(ui.mouse_down(wx + tab.x + 5, wy + tab.y + 5));
        assert_eq!(ui.take_board_command(), Some(KeyAction::Bypass("_bbsclan".into())));
        ui.mouse_up();
        assert!(ui.board.script.m_b_btn_lock);
        let favorite = killer_ui::board::FAVORITE_RECT;
        assert!(ui.mouse_down(wx + favorite.x + 5, wy + favorite.y + 5));
        assert_eq!(ui.board_released(), None);
        ui.show_board(Some((&links, "")));
        assert!(ui.mouse_down(wx + favorite.x + 5, wy + favorite.y + 5));
        assert_eq!(ui.board_released(), Some(KeyAction::Bypass("_bbsaddfav_List".into())));
        let line = &ui.board.lines[0];
        let content = killer_ui::board::content_rect();
        let (lx, ly) = (wx + content.x + line.spans[0].x + 2, wy + content.y + line.y + 2);
        assert!(ui.mouse_down(lx, ly));
        assert_eq!(ui.mouse_up().as_deref(), Some("bypass _bbsread_1"));
        ui.show_board(None);
        assert!(!ui.desktop.window(WindowId::BOARD).unwrap().visible);
        ui.show_board(Some((&links, "")));
        let close = killer_ui::board::close_button();
        assert!(ui.mouse_down(wx + close.x + 2, wy + close.y + 2));
        ui.mouse_move(wx + 100, wy + 200);
        ui.mouse_up();
        assert!(ui.desktop.window(WindowId::BOARD).unwrap().visible, "soltar fora do botão não fecha");
        assert!(ui.mouse_down(wx + close.x + 2, wy + close.y + 2));
        assert!(ui.desktop.window(WindowId::BOARD).unwrap().visible, "o botão de fechar só age ao soltar");
        ui.mouse_up();
        assert!(!ui.desktop.window(WindowId::BOARD).unwrap().visible);
    }

    #[test]
    fn the_community_board_form_fills_types_and_sends_bbs_write() {
        let mut ui = game_ui((1558, 933));
        let links = vec![String::new(); 8];
        let template = "1001\u{8}<html><body><edit var=\"Title\" width=300 height=15><br><multiedit var=\"Content\" width=400 height=60><br><button value=\"Write\" action=\"Write Notice Set _ Title Content Content\" width=70 height=20></body></html>";
        ui.show_board(Some((&links, template)));
        let values = ["0", "0", "0", "0", "0", "0", "0", "0", "0", "0", " ", "Velho", "Corpo"].join("\u{8}");
        ui.show_board(Some((&links, &format!("1002\u{8}{values}"))));
        assert_eq!(ui.board.html_form.fields.iter().map(|f| f.text()).collect::<Vec<_>>(), ["Velho", "Corpo"]);
        let window = ui.desktop.window(WindowId::BOARD).unwrap().rect;
        let content = killer_ui::board::content_rect();
        let widget = |kind: &dyn Fn(&killer_ui::html::WidgetKind) -> bool| {
            ui.board.lines.iter().find_map(|line| line.widgets.iter().find(|w| kind(&w.kind)).map(|w| (window.x + content.x + w.x + 3, window.y + content.y + line.y + 3))).unwrap()
        };
        let edit = widget(&|k| matches!(k, killer_ui::html::WidgetKind::Edit { multi: false, .. }));
        let button = widget(&|k| matches!(k, killer_ui::html::WidgetKind::Button { .. }));
        assert!(ui.mouse_down(edit.0, edit.1));
        assert_eq!(ui.board.html_form.focus, Some(0));
        assert_eq!(ui.key(InputKey::Char('!')), KeyAction::Consumed);
        ui.mouse_up();
        assert!(ui.mouse_down(button.0, button.1));
        assert_eq!(ui.board.html_form.focus, None);
        assert_eq!(ui.mouse_up(), None);
        let fields = ["Notice", "Set", "_", "Velho!", "Corpo", "Corpo"].map(String::from);
        assert!(ui.take_commands().contains(&UiCommand::Net(killer_world::NetCommand::BbsWrite { fields: Box::new(fields) })));
        assert!(ui.board.locked);
        ui.board.locked = false;
        ui.board.html_form.fields[0].clear();
        assert!(ui.mouse_down(button.0, button.1));
        ui.mouse_up();
        assert!(ui.take_commands().contains(&UiCommand::ColoredSystemMessage { id: 1207, color: 0xffb0_9b79 }));
    }

    #[test]
    fn closing_help_does_not_start_dragging_the_window_underneath() {
        let mut ui = game_ui((1024, 768));
        for window in &mut ui.desktop.windows { window.visible = false; }
        let rect = Rect::new(20, 20, 310, 311);
        let npc = ui.desktop.window_mut(WindowId::NPC_HTML).unwrap();
        npc.rect = rect;
        npc.visible = true;
        ui.desktop.window_mut(WindowId::HELP).unwrap().rect = rect;
        ui.help.toggle(&mut ui.desktop, &ui.small, crate::ui_help::HELP_FILE, "<html>Help</html>", &|_, _| None);
        assert!(ui.mouse_down(rect.x + 290, rect.y + 5));
        assert!(!ui.desktop.window(WindowId::HELP).unwrap().visible);
        assert_eq!(ui.desktop.dragging(), None);
        assert_eq!(ui.desktop.window(WindowId::NPC_HTML).unwrap().rect, rect);
    }

    #[test]
    #[ignore]
    fn converted_html_family_uses_real_help_pages_and_client_textures() {
        use killer_world::WorldEvent;
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_render::{camera::Camera, environment::Environment, headless::Headless};
        use std::collections::BTreeSet;
        let root = std::env::var_os("KILLER_CLIENT_DIR").unwrap();
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap());
        let mut ui = game_ui((1000, 640));
        ui.small = font;
        ui.set_tab_names(&tables);
        let mut pending = vec![crate::ui_help::HELP_FILE.to_owned(),crate::ui_help::MACRO_FILE.to_owned()];
        let mut visited = BTreeSet::new();
        let mut links = 0;
        while let Some(path) = pending.pop() {
            if !visited.insert(path.clone()) { continue; }
            let text = client.read_help_text(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
            assert!(!text.is_empty(),"{path}");
            ui.help.load(&mut ui.desktop,&ui.small,&text,&|_,_|None);
            for action in killer_ui::html::parse(&text).actions() {
                if let Some(link) = killer_ui::html::local_link_target(action) {
                    pending.push(link.to_owned());
                    links += 1;
                }
            }
        }
        assert!(visited.len()>2);
        assert!(links>visited.len());
        eprintln!("help: {} complete files, {} links",visited.len(),links);
        let text=client.read_help_text(crate::ui_help::HELP_FILE).unwrap();
        ui.help.toggle(&mut ui.desktop,&ui.small,crate::ui_help::HELP_FILE,&text,&|_,_|None);
        let board_links=vec![String::new();8];
        ui.show_board(Some((&board_links,"<html><body><a action=\"bypass _bbsread_1\">Community</a></body></html>")));
        ui.world_event(&WorldEvent::TutorialShowHtml(l2_net::game::TutorialShowHtml {html:"<html><body>Tutorial<br><a action=\"link tutorial_1.htm\">Next</a></body></html>".into()}),&world());
        let mut draw=DrawList::default();
        ui.board.paint(&mut draw,10,10,false,&ui.small,1.0);
        ui.help.paint(&mut draw,680,10,&ui.small,1.0);
        ui.tutorial.viewer.paint(&mut draw,680,340,&ui.small,1.0);
        let mut headless=Headless::new(1000,640).unwrap();
        let mut bank=TextureBank::new();
        headless.renderer_mut().set_ui(&draw,&mut BankSource {bank:&mut bank,catalog:client.catalog()});
        assert_eq!(bank.missing().count(),0);
        let camera=Camera {eye:glam::vec3(0.0,-300.0,0.0),target:glam::Vec3::ZERO,fov_x_deg:60.0,near:1.0,far:5000.0};
        let rgba=headless.render(&camera,&Environment::default()).unwrap();
        write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-html-family.png"),1000,640,&rgba).unwrap();
    }

    #[test]
    #[ignore]
    fn community_board_1003f9e0_uses_client_sysstrings_and_textures_in_a_gpu_capture() {
        use killer_assets::textures::{BankSource, TextureBank, write_png};
        use killer_render::camera::Camera;
        use killer_render::environment::Environment;
        use killer_render::headless::Headless;
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let tables = client.tables().unwrap();
        for id in killer_ui::board::TAB_SYSSTRINGS.into_iter().chain([killer_ui::board::FAVORITE_SYSSTRING, killer_ui::board::TITLE_SYSSTRING]) {
            assert!(tables.sysstring.get(&id).is_some_and(|text| !text.is_empty()), "sysstring {id}");
        }
        let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&client.read_system_file("SmallFont-e.gly").unwrap()).unwrap());
        let mut ui = game_ui((1000, 800));
        ui.small = font;
        ui.set_tab_names(&tables);
        let links = vec![String::new(); 8];
        ui.show_board(Some((&links, "101\u{8}<html><body><br><center>Community Board</center><br><table width=610>")));
        ui.show_board(Some((&links, "102\u{8}<tr><td><a action=\"bypass _bbsread_1\">Regras do servidor</a></td></tr>")));
        ui.show_board(Some((&links, "103\u{8}</table></body></html>")));
        assert!(ui.board.lines.iter().flat_map(|line| &line.spans).any(|span| span.action.as_deref() == Some("bypass _bbsread_1")));
        let mut draw = DrawList::default();
        ui.board.paint(&mut draw, 10, 10, false, &ui.small, 1.0);
        for (index, button) in ui.board.tabs.buttons.iter().enumerate() {
            assert_eq!(button.paint.label, tables.sysstring[&killer_ui::board::TAB_SYSSTRINGS[index]]);
        }
        let mut headless = Headless::new(666, 550).unwrap();
        let mut bank = TextureBank::new();
        headless.renderer_mut().set_ui(&draw, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(bank.missing().count(), 0);
        let camera = Camera { eye: glam::vec3(0.0, -300.0, 0.0), target: glam::Vec3::ZERO, fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let rgba = headless.render(&camera, &Environment::default()).unwrap();
        write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-board.png"), 666, 550, &rgba).unwrap();
        ui.show_board(Some((&links, "1001\u{8}<html><body><table width=610><tr><td>Title</td><td><edit var=\"Title\" width=400 height=13></td></tr><tr><td>Content</td><td><multiedit var=\"Content\" width=400 height=200></td></tr></table><button value=\"Write\" action=\"Write Notice Set _ Title Content Content\" width=70 height=20 back=\"L2UI_CH3.Button.SmallButton2_down\" fore=\"L2UI_CH3.Button.SmallButton2\"></body></html>")));
        let values = ["0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "GM", "Bem-vindo", "Primeira linha\nSegunda linha"].join("\u{8}");
        ui.show_board(Some((&links, &format!("1002\u{8}{values}"))));
        ui.board.html_form.focus = Some(0);
        let mut draw = DrawList::default();
        ui.board.paint(&mut draw, 10, 10, false, &ui.small, 1.0);
        let mut bank = TextureBank::new();
        headless.renderer_mut().set_ui(&draw, &mut BankSource { bank: &mut bank, catalog: client.catalog() });
        assert_eq!(bank.missing().count(), 0);
        let rgba = headless.render(&camera, &Environment::default()).unwrap();
        write_png(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/renders/ui-board-write.png"), 666, 550, &rgba).unwrap();
    }

    #[test]
    fn the_npc_viewer_starts_hidden_at_the_ini_position_and_opens_on_the_html_message() {
        let mut ui = game_ui((1558, 933));
        let window = ui.desktop.window(WindowId::NPC_HTML).unwrap();
        assert!(!window.visible);
        assert_eq!(window.rect, Rect::new(987, 494, 310, 401));
        assert_eq!(window.grip, Rect::new(0, 0, 310, 20));
        assert!(ui.npc_viewer.is_none());
        let list = ui.frame(&world(), &[], &[], None, &WorldViews::default());
        assert!(with_texture(&list.quads, NPC_HTML_BACK).is_empty());
        assert!(!ui.mouse_down(990, 500));
        ui.open_npc_html(&npc_html(MERCHANT, 0));
        assert!(ui.desktop.window(WindowId::NPC_HTML).unwrap().visible);
        let viewer = ui.npc_viewer.as_ref().unwrap();
        assert_eq!(viewer.title, "Chat");
        assert_eq!(viewer.lines[0].spans[0].text, "Pet Manager Cooper:");
        let list = ui.frame(&world(), &[], &[], None, &WorldViews::default());
        let back = with_texture(&list.quads, NPC_HTML_BACK);
        assert_eq!((back[0].x, back[0].y, back[0].color[3]), (987.0, 514.0, killer_ui::draw::TRANSPARENT_ALPHA));
        let mut item_tables = tables();
        item_tables.itemname.insert(57, l2_dat::itemname::ItemName { id: 57, name: "Adena".into(), add_name: String::new(), description: String::new(), popup: -1 });
        ui.open_npc_html_with(&npc_html("<html><body>x</body></html>", 57), Some(&item_tables));
        assert_eq!(ui.item_viewer.as_ref().unwrap().title, "Adena");
        assert_eq!(ui.npc_viewer.as_ref().unwrap().title, "Chat");
        ui.open_npc_html_with(&npc_html("<html><body>y</body></html>", -3), Some(&item_tables));
        assert_eq!(ui.item_viewer.as_ref().unwrap().title, "Adena");
        let bare = GameUi::new(
            font("s"),
            font("l"),
            Options::default(),
            WindowsInfo::default(),
            ChatFilter::from_ini(&parse_ini(CHAT_FILTER)),
            (1280, 720),
        ).unwrap();
        assert_eq!(bare.desktop.window(WindowId::NPC_HTML).unwrap().rect, Rect::new(0, 360 - 252, 310, 401));
    }

    #[test]
    fn an_item_page_opens_its_own_window_beside_the_npc_page_10081520() {
        let mut ui = game_ui((1558, 933));
        let item = ui.desktop.window(WindowId::ITEM_HTML).unwrap();
        assert!(!item.visible);
        assert_eq!(item.rect.w, 310);
        assert_eq!(item.rect.h, 401);
        let mut item_tables = tables();
        item_tables.itemname.insert(57, l2_dat::itemname::ItemName { id: 57, name: "Adena".into(), add_name: String::new(), description: String::new(), popup: -1 });
        ui.open_npc_html(&npc_html("<html><body><a action=\"bypass -h npc_1_Talk\">npc</a></body></html>", 0));
        ui.open_npc_html_with(&npc_html("<html><body><a action=\"bypass -h item_1_Read\">item</a></body></html>", 57), Some(&item_tables));
        assert!(ui.desktop.window(WindowId::NPC_HTML).unwrap().visible);
        assert!(ui.desktop.window(WindowId::ITEM_HTML).unwrap().visible);
        assert_eq!(ui.npc_viewer.as_ref().unwrap().title, "Chat");
        assert_eq!(ui.item_viewer.as_ref().unwrap().title, "Adena");
        assert!(ui.npc_viewer.as_ref().unwrap().doc.actions().iter().any(|action| action.contains("npc_1_Talk")));
        assert!(ui.item_viewer.as_ref().unwrap().doc.actions().iter().any(|action| action.contains("item_1_Read")));
        ui.open_npc_html_with(&npc_html("<html><body>again</body></html>", 0), Some(&item_tables));
        assert_eq!(ui.item_viewer.as_ref().unwrap().title, "Adena");
        let rect = ui.desktop.window(WindowId::ITEM_HTML).unwrap().rect;
        let close = npc_viewer::close_button().offset(rect.x, rect.y);
        assert!(ui.mouse_down(close.x + 1, close.y + 1));
        assert!(ui.item_viewer.is_none());
        assert!(!ui.desktop.window(WindowId::ITEM_HTML).unwrap().visible);
        assert!(ui.npc_viewer.is_some());
        assert!(ui.desktop.window(WindowId::NPC_HTML).unwrap().visible);
    }

    #[test]
    fn a_bypass_from_the_item_window_closes_only_the_item_window_1007_2560() {
        let mut ui = game_ui((1558, 933));
        let mut item_tables = tables();
        item_tables.itemname.insert(57, l2_dat::itemname::ItemName { id: 57, name: "Adena".into(), add_name: String::new(), description: String::new(), popup: -1 });
        ui.open_npc_html(&npc_html("<html><body>npc</body></html>", 0));
        ui.open_npc_html_with(&npc_html("<html><body><a action=\"bypass -h item_1_Read\">item</a></body></html>", 57), Some(&item_tables));
        let window = ui.desktop.window(WindowId::ITEM_HTML).unwrap().rect;
        let content = killer_ui::npc_viewer::content_rect();
        let viewer = ui.item_viewer.as_ref().unwrap();
        let (top, widget) = viewer
            .lines
            .iter()
            .flat_map(|line| line.widgets.iter().map(move |widget| (line.y, widget)))
            .find(|(_, widget)| widget.action.as_deref() == Some("bypass -h item_1_Read"))
            .unwrap();
        let (px, py) = (window.x + content.x + widget.x + 2, window.y + content.y + top + 2);
        ui.cursor = (px, py);
        assert!(ui.mouse_down(px, py));
        assert_eq!(ui.mouse_up().as_deref(), Some("bypass -h item_1_Read"));
        assert!(ui.item_viewer.is_none(), "bypass -h fecha a página que o enviou");
        assert!(ui.npc_viewer.is_some(), "a outra janela de HTML fica aberta");
    }

    #[test]
    fn an_action_link_with_a_message_asks_dialog_3c_first_and_ok_runs_the_action_1004baa0() {
        let mut ui = game_ui((1558, 933));
        let mut tables = tables();
        tables.systemmsg.insert(1700, message(1700, "Pay $s1 adena?"));
        ui.html_tables = Some(std::sync::Arc::new(tables));
        ui.open_npc_html(&npc_html("<html><body><a action=\"bypass -h npc_2_Pay 1\" msg=\"1700;500\">pay</a><br><a action=\"bypass -h npc_2_Free 1\">free</a></body></html>", 0));
        let window = ui.desktop.window(WindowId::NPC_HTML).unwrap().rect;
        let content = killer_ui::npc_viewer::content_rect();
        let viewer = ui.npc_viewer.as_ref().unwrap();
        let spot = |action: &str| {
            let (top, widget) = viewer
                .lines
                .iter()
                .flat_map(|line| line.widgets.iter().map(move |widget| (line.y, widget)))
                .find(|(_, widget)| widget.action.as_deref() == Some(action))
                .unwrap();
            (window.x + content.x + widget.x + 2, window.y + content.y + top + 2)
        };
        let (px, py) = spot("bypass -h npc_2_Pay 1");
        ui.cursor = (px, py);
        assert!(ui.mouse_down(px, py));
        assert_eq!(ui.mouse_up(), None, "com mensagem o clique só abre a caixa");
        let dialog = ui.dialog.as_ref().expect("a caixa 0x3c");
        assert_eq!((dialog.view.dialog_type, dialog.view.message.as_str(), dialog.window), (0x3c, "Pay 500 adena?", WindowId::DIALOG));
        assert_eq!(ui.finish_dialog(false), KeyAction::Consumed);
        assert!(ui.npc_viewer.is_some());
        assert!(ui.mouse_down(px, py));
        ui.mouse_up();
        assert_eq!(ui.finish_dialog(true), KeyAction::Bypass("npc_2_Pay 1".into()));
        assert!(ui.npc_viewer.is_none(), "bypass -h fecha a página (resultado 2)");
        ui.open_npc_html(&npc_html("<html><body><a action=\"bypass -h npc_2_Pay 1\" msg=\"1700;500\">pay</a></body></html>", 0));
        let viewer = ui.npc_viewer.as_ref().unwrap();
        assert!(viewer.lines.iter().flat_map(|line| line.widgets.iter()).any(|w| w.message.is_some()));
        let (fx, fy) = {
            let window = ui.desktop.window(WindowId::NPC_HTML).unwrap().rect;
            (window.x + content.x + 4, window.y + content.y + 4)
        };
        ui.cursor = (fx, fy);
        ui.mouse_down(fx, fy);
        ui.mouse_up();
        assert!(ui.dialog.is_some());
        ui.close_npc_html();
        assert!(ui.dialog.is_none(), "NCHtmlViewer::Clear esconde a caixa 0x3c");
    }

    #[test]
    fn the_chat_title_comes_from_sysstring_444() {
        let mut ui = game_ui((1558, 933));
        let mut tables = tables();
        tables.sysstring.insert(444, "Conversa".into());
        ui.set_tab_names(&tables);
        ui.open_npc_html(&npc_html(MERCHANT, 0));
        assert_eq!(ui.npc_viewer.as_ref().unwrap().title, "Conversa");
    }

    /// O martelo abre o painel de filtro acima da janela de chat, as caixas mudam no clique e o
    /// `OK` grava o `chatfilter.ini` inteiro; o `Cancel` não grava nada.
    #[test]
    fn the_chat_hammer_opens_the_filter_panel_and_ok_writes_the_ini() {
        let mut ui = game_ui((1558, 933));
        let mut tables = tables();
        for (id, text) in [(0x164u32, "Sistema"), (0x8c, "Confirmar"), (0x8d, "Cancelar"), (0x381, "Opções do chat")] {
            tables.sysstring.insert(id, text.into());
        }
        ui.set_tab_names(&tables);
        assert_eq!(ui.chat.filter_labels[0], "Sistema");
        assert_eq!(ui.chat.filter_labels[1], "Damage", "sem sysstring fica o nome do original");
        assert_eq!((ui.chat.filter_ok.as_str(), ui.chat.filter_cancel.as_str()), ("Confirmar", "Cancelar"));
        let chat = ui.desktop.window(WindowId::CHAT).unwrap().rect;
        let (hx, hy) = (chat.x + 5 + 7, chat.y + chat.h - 20 + 7);
        ui.mouse_move(hx, hy);
        assert_eq!(ui.keys.tooltip.as_ref().map(|t| t.name.as_str()), Some("Opções do chat"));
        assert!(ui.mouse_down(hx, hy));
        assert!(ui.chat.filter_open);
        assert!(ui.chat.filter_edit.iter().all(|on| *on));
        let row = |index: usize| {
            let rect = killer_ui::chat::FILTER_ROWS[index].3;
            (chat.x + rect.x + 2, chat.y - 191 + rect.y + 2)
        };
        let (sx, sy) = row(killer_ui::chat::FILTER_SYSTEM);
        assert!(ui.mouse_down(sx, sy));
        assert!(!ui.chat.filter_edit[killer_ui::chat::FILTER_SYSTEM]);
        assert!(!ui.chat.filter_enabled(1));
        let (dx, dy) = row(1);
        assert!(ui.mouse_down(dx, dy));
        assert!(ui.chat.filter_edit[1], "linha desligada não muda");
        let cancel = (chat.x + 131 + 10, chat.y - 191 + 160 + 10);
        assert!(ui.mouse_down(cancel.0, cancel.1));
        assert_eq!(ui.chat.pressed, Some(killer_ui::chat::FilterHit::Cancel));
        ui.mouse_move(cancel.0, cancel.1);
        assert_eq!(ui.chat_filter_released(), None, "cancel não grava");
        assert!(!ui.chat.filter_open);
        assert!(ui.chat.filter.tabs[0].shows("system"), "cancel joga fora");
        assert!(ui.mouse_down(hx, hy));
        assert!(ui.mouse_down(sx, sy));
        let ok = (chat.x + 51 + 10, chat.y - 191 + 160 + 10);
        assert!(ui.mouse_down(ok.0, ok.1));
        ui.mouse_move(ok.0 + 200, ok.1);
        assert_eq!(ui.chat_filter_released(), None, "soltar fora do botão não vale");
        assert!(ui.chat.filter_open);
        assert!(ui.mouse_down(ok.0, ok.1));
        ui.mouse_move(ok.0, ok.1);
        let text = ui.chat_filter_released().expect("o ok grava o ini");
        assert!(!ui.chat.filter_open);
        assert!(!ui.chat.filter.tabs[0].shows("system"));
        assert!(text.starts_with("[entire_tab]\r\nsystem=False\r\ndamage=True"));
        assert!(text.contains("[market_tab]\r\nsystem=False"));
        let list = ui.frame(&World::default(), &[], &[], None, &WorldViews::default());
        assert!(list.quads.iter().all(|q| q.texture != killer_ui::chat::CHATTING_BACK5));
        assert!(ui.mouse_down(hx, hy));
        let list = ui.frame(&World::default(), &[], &[], None, &WorldViews::default());
        let back = list.quads.iter().find(|q| q.texture == killer_ui::chat::CHATTING_BACK5).unwrap();
        assert_eq!((back.x, back.y), (chat.x as f32, (chat.y - 191) as f32));
    }

    /// O resto do `HennaInfo` (`0xe4`): os seis modificadores aparecem ao lado dos atributos na
    /// página de estado, no formato do `NCDetailStatusWnd::OnPaint @ 0x10111790` — `"%d (+%d)"`
    /// quando positivo, `"%d(%d)"` quando negativo e `"%d"` quando zero —, cada um no seu lugar:
    /// `+0x14c` no `STR`, `+0x164` no `DEX`, `+0x154` no `CON`, `+0x144` no `INT`, `+0x16c` no
    /// `WIT` e `+0x15c` no `MEN`.
    #[test]
    fn the_status_page_shows_the_symbol_modifiers_of_henna_info() {
        let mut ui = game_ui((1558, 933));
        let mut world = World { me: Some(me()), ..Default::default() };
        world.stats.str_ = 40;
        world.stats.dex = 30;
        world.stats.con = 43;
        world.stats.int_ = 21;
        world.stats.wit = 11;
        world.stats.men = 25;
        let info = l2_net::game::HennaInfo {
            int_: 0,
            str_: 2,
            con: -2,
            men: 0,
            dex: 3,
            wit: 0,
            slots: 3,
            symbols: Vec::new(),
        };
        world.apply_world(&killer_world::WorldEvent::HennaInfo(info));
        ui.sync_windows(&world, None, killer_platform::time::Instant::now());
        let view = crate::ui_keys::detail_status_view(&world);
        assert_eq!(view.stat_bonus, [2, 3, -2, 0, 0, 0], "STR, DEX, CON, INT, WIT, MEN");
        assert_eq!(killer_ui::detail_status::stat_text(view.stats[0], view.stat_bonus[0]), "40 (+2)");
        assert_eq!(killer_ui::detail_status::stat_text(view.stats[2], view.stat_bonus[2]), "43(-2)");
        assert_eq!(killer_ui::detail_status::stat_text(view.stats[3], view.stat_bonus[3]), "21");
    }

    #[test]
    fn my_target_selected_fills_the_gm_name_line_only_while_the_window_is_shown() {
        let mut ui = game_ui((1558, 933));
        let mut world = World { me: Some(me()), ..Default::default() };
        let mut target = me();
        target.object_id = 7;
        target.name = "Luzal".into();
        world.entities.insert(7, target);
        world.target = Some(7);
        let event = killer_world::WorldEvent::MyTargetSelected(l2_net::game::MyTargetSelected { object_id: 7, color: 0 });
        ui.world_event(&event, &world);
        assert_eq!(ui.keys.gm.name(), "");
        ui.desktop.window_mut(WindowId::GM).unwrap().visible = true;
        ui.world_event(&event, &world);
        assert_eq!(ui.keys.gm.name(), "Luzal");
    }

    #[test]
    fn dying_cancels_the_bottom_dialog_like_a_timeout_and_resets_the_main_one_silently() {
        let mut ui = game_ui((1558, 933));
        let mut world = World { me: Some(me()), ..Default::default() };
        let now = Instant::now();
        ui.open_dialog(&Dialog::JoinParty(l2_net::game::AskJoinParty { requestor_name: "Luzal".into(), item_distribution: 0 }), &world, None, now);
        let die = killer_world::WorldEvent::Die(l2_net::game::Die {
            object_id: 1,
            to_village: false,
            to_hideout: false,
            to_castle: false,
            to_siege_hq: false,
            sweepable: false,
            to_fixed_spot: false,
        });
        ui.world_event(&die, &world);
        assert!(ui.dialog.is_none());
        assert_eq!(ui.tick_dialog(&mut world, None, now), Some(KeyAction::Dialog(DialogAnswer::JoinParty, false)));
        ui.open_dialog(&Dialog::Confirm(l2_net::game::SystemMessage { id: 1510, args: vec![] }), &world, None, now);
        ui.world_event(&die, &world);
        assert!(ui.dialog.is_none());
        assert_eq!(ui.tick_dialog(&mut world, None, now), None);
    }

    #[test]
    fn a_party_invitation_with_an_unknown_distribution_has_an_empty_text() {
        let world = World::default();
        let ask = |item_distribution| Dialog::JoinParty(l2_net::game::AskJoinParty { requestor_name: "Luzal".into(), item_distribution });
        assert_eq!(dialog_request(&ask(5), &world, None).0.message, "");
        assert_eq!(dialog_request(&ask(-1), &world, None).0.message, "");
        assert_ne!(dialog_request(&ask(4), &world, None).0.message, "");
    }

    /// A janela de encantar pelo caminho real: a grade recebe os itens **listados** na aba aberta
    /// (o `FUN_10099370` clona as células da grade do inventário, não o inventário inteiro), e a
    /// escolha sobrevive aos quadros seguintes — o original só preenche a grade no abridor
    /// (`0x100666c0`), e refazer isso a cada quadro apagava a escolha e a rolagem.
    #[test]
    fn the_enchant_grid_keeps_the_choice_across_frames_and_lists_only_the_open_tab() {
        let mut ui = game_ui((1558, 933));
        let mut tables = tables();
        tables.item_icons.insert_all(
            (101..=104)
                .map(|id| l2_dat::itemicons::ItemIcon {
                    id,
                    kind: l2_dat::itemicons::ItemKind::Armor,
                    drop: Default::default(),
                    icons: std::array::from_fn(|_| "icon.armor".into()),
                    weight: 1,
                    type1: 1,
                })
                .collect(),
        );
        let mut world = World { me: Some(me()), ..Default::default() };
        let item = |object_id: i32, equipped: bool, type2: i16| l2_net::game::InventoryItem {
            type1: 0,
            object_id,
            item_id: 100 + object_id,
            count: 1,
            type2,
            custom_type1: 0,
            equipped,
            body_part: 0,
            enchant: 0,
            custom_type2: 0,
        };
        world.inventory.items = vec![item(1, false, 0), item(2, true, 0), item(3, false, 3), item(4, false, 0)];
        world.windows.chosen_item_id = Some(729);
        let now = killer_platform::time::Instant::now();
        ui.sync_windows(&world, Some(&tables), now);
        ui.world_event(&killer_world::WorldEvent::ChooseInventoryItem(l2_net::game::ChooseInventoryItem { item_id: 729 }), &world);
        ui.sync_windows(&world, Some(&tables), now);

        let listed: Vec<i32> = ui.keys.enchant.entries.iter().map(|e| e.object_id).collect();
        assert_eq!(listed, [1, 4, 2], "aba de itens (sem quest) e depois os equipados, como os dois laços de FUN_10099370");

        let rect = ui.desktop.window(WindowId::ITEM_ENCHANT).unwrap().rect;
        let grid = killer_ui::enchant_wnd::GRID_RECT;
        assert!(ui.mouse_down(rect.x + grid.x + 2, rect.y + grid.y + 2));
        ui.mouse_up();
        assert_eq!(ui.keys.enchant.grid.selected, Some(0));
        // vários quadros depois a escolha continua lá
        for _ in 0..3 {
            ui.sync_windows(&world, Some(&tables), now);
        }
        assert_eq!(ui.keys.enchant.grid.selected, Some(0), "o quadro seguinte não apaga a escolha");

        let ok = killer_ui::enchant_wnd::OK_RECT;
        assert!(ui.mouse_down(rect.x + ok.x + 2, rect.y + ok.y + 2));
        ui.mouse_up();
        let commands = ui.keys.take_commands();
        assert!(
            commands
                .iter()
                .any(|c| matches!(c, crate::ui_keys::UiCommand::Net(killer_world::NetCommand::EnchantItem { object_id: 1 }))),
            "o OK manda o id do item escolhido: {commands:?}"
        );
    }

    /// Caminho real da coluna de símbolos no inventário: o `HennaInfo` (`0xe4`) entra no mundo, o
    /// `sync_windows` monta a lista pelo `hennagrp`, e o `frame` pinta as molduras e os ícones no
    /// painel de equipamento. Nós não mostrávamos nada aqui — foi o que o usuário viu faltando.
    #[test]
    fn the_inventory_shows_the_character_symbols_through_the_game_ui() {
        let mut ui = game_ui((1558, 933));
        let mut tables = tables();
        for (symbol_id, icon) in [(145u32, "icon.etc_str_symbol_i01"), (151, "icon.etc_int_symbol_i01")] {
            tables.hennagrp.insert(
                symbol_id,
                l2_dat::HennaGrp {
                    symbol_id,
                    dye_item_id: 4444 + symbol_id,
                    name: "Greater Symbol".into(),
                    icon: icon.into(),
                    description: String::new(),
                    add_name: String::new(),
                },
            );
        }
        let info = l2_net::game::HennaInfo {
            int_: 11,
            str_: 30,
            con: 43,
            men: 10,
            dex: 20,
            wit: 25,
            slots: 3,
            symbols: vec![(145, 4589), (151, 0)],
        };
        let mut world = World::default();
        // classe 2 (segunda profissão) tem três espaços pela tabela do `0x10074500`
        let mut player = me();
        if let killer_world::EntityKind::Player { class_id, .. } = &mut player.kind {
            *class_id = 2;
        }
        world.me = Some(player);
        world.apply_world(&killer_world::WorldEvent::HennaInfo(info));
        ui.sync_windows(&world, Some(&tables), killer_platform::time::Instant::now());
        assert_eq!(ui.keys.inventory.henna_slots, 3);
        assert_eq!(ui.keys.inventory.henna.len(), 2);

        if let Some(window) = ui.desktop.window_mut(WindowId::INVENTORY) {
            window.visible = true;
        }
        let rect = ui.desktop.window(WindowId::INVENTORY).unwrap().rect;
        let quads = ui.frame(&world, &[], &[], None, &WorldViews::default()).quads;
        let paperdoll = killer_ui::inventory::PAPERDOLL_RECT;
        let (px, py) = ((rect.x + paperdoll.x) as f32, (rect.y + paperdoll.y) as f32);
        let marks: Vec<_> =
            quads.iter().filter(|q| q.texture == killer_ui::inventory::HENNA_MARK).map(|q| (q.x, q.y)).collect();
        assert_eq!(marks, [(px + 215.0, py + 8.0), (px + 215.0, py + 35.0), (px + 215.0, py + 62.0)]);
        assert!(quads.iter().any(|q| q.texture == "icon.etc_str_symbol_i01" && (q.x, q.y) == (px + 216.0, py + 9.0)));
        assert!(quads.iter().any(|q| q.texture == "icon.etc_int_symbol_i01" && (q.x, q.y) == (px + 216.0, py + 36.0)));
        let empty: Vec<_> = quads
            .iter()
            .filter(|q| q.texture == killer_ui::inventory::HENNA_EMPTY)
            .map(|q| (q.x, q.y))
            .collect();
        assert_eq!(empty, [(px + 217.0, py + 36.0)], "só o símbolo sem corante é tapado");
    }

    /// Caminho real das janelas de henna pela `GameUi`: o pacote abre a lista, o clique numa linha
    /// pede a informação do símbolo, a resposta troca a lista pela janela de informação e o `OK`
    /// manda o pedido do modo. Os testes do `ui_henna` chamam a janela direto e pulam este trecho.
    #[test]
    fn the_henna_windows_work_through_the_game_ui() {
        let mut ui = game_ui((1558, 933));
        let mut tables = tables();
        tables.itemname.insert(
            4589,
            l2_dat::ItemName {
                id: 4589,
                name: "Greater Dye of STR".into(),
                add_name: "Str+2 Con-2".into(),
                description: String::new(),
                popup: 0,
            },
        );
        tables.hennagrp.insert(
            145,
            l2_dat::HennaGrp {
                symbol_id: 145,
                dye_item_id: 4589,
                name: "Greater Symbol of Strength".into(),
                icon: "icon.etc_str_symbol_i01".into(),
                description: "Str+2 Con-2".into(),
                add_name: "Str+2 Con-2".into(),
            },
        );
        let offer = l2_net::game::HennaOffer {
            symbol_id: 145,
            dye_item_id: 4589,
            dye_amount: 10,
            price: 33000,
            available: true,
        };
        let list = l2_net::game::HennaEquipList { adena: 5000, slots: 3, offers: vec![offer] };
        ui.henna_event(&killer_world::WorldEvent::HennaEquipList(list), Some(&tables));
        assert!(ui.desktop.window(WindowId::HENNA_LIST).unwrap().visible);

        let rect = ui.desktop.window(WindowId::HENNA_LIST).unwrap().rect;
        let quads = ui.frame(&World::default(), &[], &[], None, &WorldViews::default()).quads;
        assert!(quads.iter().any(|q| q.texture == killer_ui::henna::LIST_BACK), "o fundo da lista é pintado");

        let cell = (rect.x + killer_ui::henna::LIST_RECT.x + 2, rect.y + killer_ui::henna::LIST_RECT.y + killer_ui::henna::ROW_TOP + 2);
        assert!(ui.mouse_down(cell.0, cell.1));
        ui.mouse_up();
        let commands = ui.keys.take_commands();
        assert!(
            commands.iter().any(|c| matches!(c, crate::ui_keys::UiCommand::Net(killer_world::NetCommand::HennaItemInfo { symbol_id: 145 }))),
            "clicar na linha pede a informação do símbolo: {commands:?}"
        );

        let info = l2_net::game::HennaItemInfo {
            symbol_id: 145,
            dye_item_id: 4589,
            dye_amount: 10,
            price: 33000,
            available: true,
            adena: 5000,
            int_: (11, 12),
            str_: (30, 32),
            con: (43, 41),
            men: (10, 10),
            dex: (20, 20),
            wit: (25, 25),
        };
        ui.henna_event(&killer_world::WorldEvent::HennaItemInfo(info), Some(&tables));
        assert!(!ui.desktop.window(WindowId::HENNA_LIST).unwrap().visible);
        assert!(ui.desktop.window(WindowId::HENNA_INFO).unwrap().visible);
        let quads = ui.frame(&World::default(), &[], &[], None, &WorldViews::default()).quads;
        assert!(quads.iter().any(|q| q.texture == killer_ui::henna::INFO_BACK));
        assert!(quads.iter().any(|q| q.texture == "icon.etc_str_symbol_i01"), "o ícone do símbolo é pintado");

        let rect = ui.desktop.window(WindowId::HENNA_INFO).unwrap().rect;
        let ok = killer_ui::henna::OK_BTN_RECT;
        let point = (rect.x + ok.x + 2, rect.y + ok.y + 2);
        assert!(ui.mouse_down(point.0, point.1));
        ui.mouse_up();
        let commands = ui.keys.take_commands();
        assert!(
            commands.iter().any(|c| matches!(c, crate::ui_keys::UiCommand::Net(killer_world::NetCommand::HennaEquip { symbol_id: 145 }))),
            "o OK manda RequestHennaEquip: {commands:?}"
        );
        assert!(!ui.desktop.window(WindowId::HENNA_INFO).unwrap().visible);
    }

    /// Caminho real do arrasto na janela de macro, do jeito que o jogo roda: `mouse_down_at`,
    /// `mouse_move` e `mouse_up_in` da `GameUi`. Os testes anteriores chamavam a `MacroWindow`
    /// direto e pulavam justamente esse trecho.
    #[test]
    fn dragging_a_macro_to_the_edit_icon_through_the_game_ui_opens_the_editor() {
        let mut ui = game_ui((1558, 933));
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
                commands: Vec::new(),
            }],
        };
        let event = killer_world::WorldEvent::SendMacroList(list);
        let mut world = World::default();
        world.apply_world(&event);
        ui.macros_event(&event, &world);
        ui.keys.macros.open(&mut ui.desktop);
        let rect = ui.desktop.window(WindowId::MACRO_LIST).unwrap().rect;
        let cell = (rect.x + killer_ui::macros::ARRAY_RECT.x + 6, rect.y + killer_ui::macros::ARRAY_RECT.y + 6);
        let icon = killer_ui::macros::EDIT_ICON_RECT;
        let drop = (rect.x + icon.x + 4, rect.y + icon.y + 4);

        assert!(ui.mouse_down(cell.0, cell.1));
        assert_eq!(ui.keys.macros.list.dragging, Some(0), "apertar na célula começa o arrasto");
        ui.mouse_move(drop.0, drop.1);
        assert_eq!(
            ui.keys.macros.list.hover,
            Some(killer_ui::macros::MacroHit::EditIcon),
            "passar por cima do ícone acende a arte de arrastar"
        );
        ui.mouse_up();
        assert!(ui.desktop.window(WindowId::MACRO_EDIT).unwrap().visible, "soltar no ícone abre o editor");
        assert_eq!(ui.keys.macros.edit.name.get_text().as_deref(), Some("Meu macro"));
        assert_eq!(ui.keys.macros.run, None, "arrastar não executa o macro");

        // e clicar sem arrastar executa (com o editor fechado, que senão fica por cima da célula)
        if let Some(window) = ui.desktop.window_mut(WindowId::MACRO_EDIT) {
            window.visible = false;
        }
        assert!(ui.mouse_down(cell.0, cell.1));
        ui.mouse_move(cell.0, cell.1);
        ui.mouse_up();
        assert_eq!(ui.keys.macros.run, Some(10));
    }

    /// Como um comando entra num macro: arrastando um slot da barra de atalhos pra uma linha da
    /// lista de comandos do editor (`NCShortCutWnd::OnLButtonUp @ 0x100adf60`, que confere a
    /// janela de destino em `0x100ae171` e acha a linha em `0x100ae18d`).
    #[test]
    fn dropping_a_shortcut_on_an_editor_line_writes_the_command() {
        let mut ui = game_ui((1558, 933));
        let mut tables = tables();
        tables.skillname.insert(
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
        tables.commandname.insert("useskill".into(), 0x18);
        ui.set_tab_names(&tables);

        let world = World {
            me: Some(me()),
            shortcuts: vec![l2_net::game::Shortcut {
                slot: 0,
                kind: l2_net::game::ShortcutKind::Skill { skill_id: 1397, level: 1 },
                character_type: 1,
            }],
            ..Default::default()
        };
        ui.sync_windows(&world, Some(&tables), killer_platform::time::Instant::now());

        ui.keys.macros.edit.set_macro(None);
        show_window(&mut ui, WindowId::MACRO_EDIT, true);
        let editor = ui.desktop.window(WindowId::MACRO_EDIT).unwrap().rect;
        let bar = ui.desktop.window(WindowId::SHORTCUT).unwrap().rect;
        let slot = ui.shortcut.slot_rect(0);
        let from = (bar.x + slot.x + slot.w / 2, bar.y + slot.y + slot.h / 2);
        let rows = killer_ui::macros::EDIT_COMMAND_LIST_RECT;
        let to = (editor.x + rows.x + 20, editor.y + rows.y + 4);

        assert!(ui.mouse_down(from.0, from.1));
        assert_eq!(ui.shortcut.pressed, Some(0), "apertar no slot começa o arrasto");
        ui.mouse_move(to.0, to.1);
        ui.mouse_up_in(&world);

        let commands = &ui.keys.macros.edit.macro_entry.commands;
        assert_eq!(commands.len(), 1, "a linha em que soltou recebe o comando");
        assert_eq!(commands[0].label, "/useskill Chant of Victory");
        assert_eq!((commands[0].kind, commands[0].skill_id, commands[0].shortcut_id), (1, 1397, 1));
        // e o atalho não foi registrado em outro slot nem usado
        assert_eq!(ui.shortcut.pressed, None);

        // salvar tem que mandar esse comando no pacote, não só mostrá-lo na janela
        ui.keys.macros.edit.focus(killer_ui::macros::EditFocus::Name);
        for c in "Buff".chars() {
            ui.macro_key(killer_ui::chat::InputKey::Char(c));
        }
        let save = killer_ui::macros::EDIT_SAVE_RECT;
        assert!(ui.mouse_down(editor.x + save.x + 2, editor.y + save.y + 2));
        ui.mouse_up_in(&world);
        let sent: Vec<_> = ui
            .take_commands()
            .into_iter()
            .filter_map(|c| match c {
                crate::ui_keys::UiCommand::Net(killer_world::NetCommand::MakeMacro(m)) => Some(m),
                _ => None,
            })
            .collect();
        let saved = sent.first().expect("Salvar envia o macro");
        assert_eq!(saved.commands.len(), 1, "o comando solto entra no pacote");
        assert_eq!((saved.commands[0].kind, saved.commands[0].skill_id), (1, 1397));
    }

    /// No cliente o `WindowsInfo.ini` põe a lista de macro e o editor no mesmo ponto (ids 29 e
    /// 31, os dois `posX=0 posY=173`), e os dois desenham o fundo em `(0, 20, 256, 381)`
    /// (`0x100a8e2a` e `0x100aa44a`). Quer dizer que o editor tapa a lista inteira — então o
    /// fundo dele tem que sair depois de tudo que a lista desenha.
    #[test]
    fn the_macro_editor_paints_over_the_list_that_sits_at_the_same_place() {
        let mut ui = game_ui((1558, 933));
        let list = ui.desktop.window(WindowId::MACRO_LIST).unwrap().rect;
        let editor = ui.desktop.window(WindowId::MACRO_EDIT).unwrap().rect;
        assert_eq!((list.x, list.y), (editor.x, editor.y), "as duas janelas ficam no mesmo ponto");
        ui.desktop.raise_newly_shown();
        show_window(&mut ui, WindowId::MACRO_LIST, true);
        ui.desktop.raise_newly_shown();
        show_window(&mut ui, WindowId::MACRO_EDIT, true);
        ui.desktop.raise_newly_shown();
        let quads = ui.frame(&World::default(), &[], &[], None, &WorldViews::default()).quads;
        let editor_back = quads
            .iter()
            .position(|q| q.texture == "l2ui_ch3.macrownd.macro1_back")
            .expect("o editor desenha o fundo dele");
        let last_list = quads
            .iter()
            .rposition(|q| q.texture == killer_ui::macros::LIST_BACK || q.texture == killer_ui::macros::EDIT)
            .expect("a lista desenha");
        assert!(last_list < editor_back, "o fundo do editor tem que vir depois do que a lista desenha");
    }

    /// O botão de adicionar abre o editor vazio, e os dois campos de texto aceitam digitação —
    /// nome e sigla, cada um com o seu limite (`FUN_1000b790(0xc)` e `FUN_1000b790(4)`).
    #[test]
    fn the_add_button_opens_an_empty_editor_and_both_text_fields_take_typing() {
        let mut ui = game_ui((1558, 933));
        ui.keys.macros.open(&mut ui.desktop);
        let list = ui.desktop.window(WindowId::MACRO_LIST).unwrap().rect;
        let add = killer_ui::macros::ADD_RECT;
        let point = (list.x + add.x + 10, list.y + add.y + 10);
        assert!(ui.mouse_down(point.0, point.1));
        ui.mouse_move(point.0, point.1);
        ui.mouse_up();
        assert!(ui.desktop.window(WindowId::MACRO_EDIT).unwrap().visible, "o botão de adicionar abre o editor");
        assert!(ui.keys.macros.edit.name.get_text().is_none(), "e ele vem vazio");

        let editor = ui.desktop.window(WindowId::MACRO_EDIT).unwrap().rect;
        let name = killer_ui::macros::EDIT_NAME_RECT;
        assert!(ui.mouse_down(editor.x + name.x + 4, editor.y + name.y + 4));
        for c in "Buff".chars() {
            ui.macro_key(killer_ui::chat::InputKey::Char(c));
        }
        assert_eq!(ui.keys.macros.edit.name.get_text().as_deref(), Some("Buff"));

        let acronym = killer_ui::macros::EDIT_ACRONYM_RECT;
        assert!(ui.mouse_down(editor.x + acronym.x + 4, editor.y + acronym.y + 4));
        assert_eq!(
            ui.keys.macros.edit.focus,
            killer_ui::macros::EditFocus::Acronym,
            "clicar na sigla muda o campo com foco"
        );
        for c in "BF".chars() {
            ui.macro_key(killer_ui::chat::InputKey::Char(c));
        }
        assert_eq!(ui.keys.macros.edit.acronym.get_text().as_deref(), Some("BF"));
        assert_eq!(ui.keys.macros.edit.name.get_text().as_deref(), Some("Buff"), "e o nome fica intacto");

        // e as linhas de comando também são caixas de texto: dá pra digitar o comando
        let rows = killer_ui::macros::EDIT_COMMAND_LIST_RECT;
        assert!(ui.mouse_down(editor.x + rows.x + 30, editor.y + rows.y + 4));
        assert_eq!(ui.keys.macros.edit.focus, killer_ui::macros::EditFocus::Command(0));
        for c in "/delay 5".chars() {
            ui.macro_key(killer_ui::chat::InputKey::Char(c));
        }
        assert_eq!(ui.keys.macros.edit.macro_entry.commands[0].label, "/delay 5");
        // o limite de 32 caracteres da caixa (FUN_1000b790(0x20)) vale
        for _ in 0..40 {
            ui.macro_key(killer_ui::chat::InputKey::Char('x'));
        }
        assert_eq!(ui.keys.macros.edit.macro_entry.commands[0].label.chars().count(), 32);
    }

    #[test]
    fn macro_scroll_capture_reaches_all_twelve_commands_through_game_ui() {
        let mut ui = game_ui((1558, 933));
        show_window(&mut ui, WindowId::MACRO_EDIT, true);
        let rect = ui.desktop.window(WindowId::MACRO_EDIT).unwrap().rect;
        for line in 0..12 {
            if line >= 9 {
                let (x, y) = (rect.x + 240, rect.y + 355);
                ui.mouse_move(x, y);
                assert!(ui.mouse_down(x, y));
                ui.mouse_up();
            }
            let row = line - ui.keys.macros.edit.command_scroll;
            let (x, y) = (rect.x + 35, rect.y + 134 + row as i32 * 26);
            ui.mouse_move(x, y);
            assert!(ui.mouse_down(x, y));
            ui.mouse_up();
            assert_eq!(ui.keys.macros.edit.focus, killer_ui::macros::EditFocus::Command(line));
            for c in format!("/delay {}", line + 1).chars() { ui.macro_key(InputKey::Char(c)); }
        }
        assert_eq!(ui.keys.macros.edit.macro_entry.commands.len(), 12);
        for (line, command) in ui.keys.macros.edit.macro_entry.commands.iter().enumerate() {
            assert_eq!(command.label, format!("/delay {}", line + 1));
        }
        ui.keys.macros.edit.focus(killer_ui::macros::EditFocus::Name);
        assert!(ui.wheel_over(rect.x + 35, rect.y + 134, 0.25, &[]));
        assert_eq!(ui.keys.macros.edit.command_scroll, 3);
        assert!(ui.wheel_over(rect.x + 35, rect.y + 134, 1.0, &[]));
        assert_eq!(ui.keys.macros.edit.command_scroll, 0);
        assert!(ui.wheel_over(rect.x + 35, rect.y + 134, -1.0, &[]));
        assert_eq!(ui.keys.macros.edit.command_scroll, 3);
    }

    /// Fluxo da janela de encantar, no caminho real: o `ChooseInventoryItem` abre, o clique
    /// escolhe o item, o `OK` manda o id e **não** fecha, e o `EnchantResult` é que fecha.
    #[test]
    fn the_enchant_window_opens_on_choose_item_and_sends_the_chosen_object() {
        let mut ui = game_ui((1558, 933));
        let mut tables = tables();
        tables.itemname.insert(
            729,
            l2_dat::itemname::ItemName {
                id: 729,
                name: "Scroll: Enchant Weapon (Grade A)".into(),
                add_name: String::new(),
                description: String::new(),
                popup: 0,
            },
        );
        ui.set_tab_names(&tables);
        let mut world = World { me: Some(me()), ..Default::default() };
        tables.item_icons.insert_all(vec![
            l2_dat::itemicons::ItemIcon { id: 100, kind: l2_dat::itemicons::ItemKind::Weapon, drop: Default::default(), icons: std::array::from_fn(|_| "icon.weapon".into()), weight: 1, type1: 0 },
            l2_dat::itemicons::ItemIcon { id: 101, kind: l2_dat::itemicons::ItemKind::Etc, drop: Default::default(), icons: std::array::from_fn(|_| "icon.etc".into()), weight: 1, type1: 0 },
            l2_dat::itemicons::ItemIcon { id: 102, kind: l2_dat::itemicons::ItemKind::Armor, drop: Default::default(), icons: std::array::from_fn(|_| "icon.tiara".into()), weight: 1, type1: 0 },
        ]);
        let armor = |id: u32, body_part: u32| l2_dat::armorgrp::Armor {
            drop_sound:String::new(),
            equip_sound: String::new(),
            boots_sounds: vec![],
            id, body_part, rows: vec![], npc_meshes: vec![], npc_textures: vec![], damage_effect: String::new(), attributes_58c: [0; 6], crystallizable: false,
        };
        tables.armorgrp.insert(102, armor(102, l2_dat::armorgrp::body_part::HAIR));
        world.inventory.items = vec![
            l2_net::game::InventoryItem {
                type1: 0, object_id: 501, item_id: 100, count: 1, type2: 0, custom_type1: 0,
                equipped: false, body_part: 0, enchant: 3, custom_type2: 0,
            },
            l2_net::game::InventoryItem {
                type1: 0, object_id: 502, item_id: 101, count: 1, type2: 0, custom_type1: 0,
                equipped: false, body_part: 0, enchant: 0, custom_type2: 0,
            },
            l2_net::game::InventoryItem {
                type1: 0, object_id: 503, item_id: 102, count: 1, type2: 0, custom_type1: 0,
                equipped: false, body_part: 0, enchant: 0, custom_type2: 0,
            },
        ];

        let event = killer_world::WorldEvent::ChooseInventoryItem(l2_net::game::packets::ChooseInventoryItem { item_id: 729 });
        world.apply_world(&event);
        ui.world_event(&event, &world);
        assert!(ui.desktop.window(WindowId::ITEM_ENCHANT).unwrap().visible, "o pacote abre a janela");
        ui.sync_windows(&world, Some(&tables), killer_platform::time::Instant::now());
        assert_eq!(ui.keys.enchant.entries.len(), 1, "arma entra; etcitem e acessório de cabelo (Tiara/Circlet) ficam de fora");
        assert_eq!(ui.keys.enchant.entries[0].object_id, 501);
        assert!(ui.keys.enchant.title.ends_with("(Scroll: Enchant Weapon (Grade A))"), "saiu {}", ui.keys.enchant.title);

        let rect = ui.desktop.window(WindowId::ITEM_ENCHANT).unwrap().rect;
        let grid = killer_ui::enchant_wnd::GRID_RECT;
        let slot = (rect.x + grid.x + 2, rect.y + grid.y + 2);
        assert!(ui.mouse_down(slot.0, slot.1));
        assert_eq!(ui.keys.enchant.selected().map(|e| e.object_id), Some(501));

        let ok = killer_ui::enchant_wnd::OK_RECT;
        assert!(ui.mouse_down(rect.x + ok.x + 4, rect.y + ok.y + 4));
        ui.mouse_move(rect.x + ok.x + 4, rect.y + ok.y + 4);
        ui.mouse_up_in(&world);
        let sent: Vec<_> = ui.take_commands();
        assert!(
            sent.contains(&crate::ui_keys::UiCommand::Net(killer_world::NetCommand::EnchantItem { object_id: 501 })),
            "saiu {sent:?}"
        );
        assert!(ui.desktop.window(WindowId::ITEM_ENCHANT).unwrap().visible, "o OK não fecha a janela");

        // o resultado do servidor é que fecha (0x100667f0 só chama Hide)
        let result = killer_world::WorldEvent::EnchantResult(l2_net::game::packets::EnchantResult { result: 1 });
        ui.world_event(&result, &world);
        assert!(!ui.desktop.window(WindowId::ITEM_ENCHANT).unwrap().visible);

        // e o Cancel manda -1 e fecha na hora
        ui.world_event(&event, &world);
        let cancel = killer_ui::enchant_wnd::CANCEL_RECT;
        assert!(ui.mouse_down(rect.x + cancel.x + 4, rect.y + cancel.y + 4));
        ui.mouse_move(rect.x + cancel.x + 4, rect.y + cancel.y + 4);
        ui.mouse_up_in(&world);
        let sent = ui.take_commands();
        assert!(
            sent.contains(&crate::ui_keys::UiCommand::Net(killer_world::NetCommand::EnchantItem { object_id: -1 })),
            "saiu {sent:?}"
        );
        assert!(!ui.desktop.window(WindowId::ITEM_ENCHANT).unwrap().visible);
    }

    #[test]
    fn the_enchant_grid_includes_equipped_weapons_like_a_dual() {
        let mut ui = game_ui((1558, 933));
        let mut tables = tables();
        tables.item_icons.insert_all(vec![
            l2_dat::itemicons::ItemIcon { id: 200, kind: l2_dat::itemicons::ItemKind::Weapon, drop: Default::default(), icons: std::array::from_fn(|_| "icon.dual".into()), weight: 1, type1: 0 },
        ]);
        let mut world = World { me: Some(me()), ..Default::default() };
        world.inventory.items = vec![l2_net::game::InventoryItem {
            type1: 0, object_id: 900, item_id: 200, count: 1, type2: 0, custom_type1: 0,
            equipped: true, body_part: l2_net::game::packets::BODY_PART_LR_HAND, enchant: 0, custom_type2: 0,
        }];
        let event = killer_world::WorldEvent::ChooseInventoryItem(l2_net::game::packets::ChooseInventoryItem { item_id: 729 });
        world.apply_world(&event);
        ui.world_event(&event, &world);
        ui.sync_windows(&world, Some(&tables), killer_platform::time::Instant::now());
        assert_eq!(ui.keys.enchant.entries.len(), 1, "a arma equipada (dual) entra na grade de enchant");
        assert_eq!(ui.keys.enchant.entries[0].object_id, 900);
    }

    #[test]
    fn hovering_an_enchant_item_shows_its_name_tooltip() {
        let mut ui = game_ui((1558, 933));
        let mut tables = tables();
        tables.item_icons.insert_all(vec![
            l2_dat::itemicons::ItemIcon { id: 200, kind: l2_dat::itemicons::ItemKind::Weapon, drop: Default::default(), icons: std::array::from_fn(|_| "icon.dual".into()), weight: 3, type1: 0 },
        ]);
        tables.itemname.insert(200, l2_dat::itemname::ItemName {
            id: 200, name: "Dual Swords".into(), add_name: String::new(), description: "A pair.".into(), popup: 0,
        });
        let mut world = World { me: Some(me()), ..Default::default() };
        world.inventory.items = vec![l2_net::game::InventoryItem {
            type1: 0, object_id: 900, item_id: 200, count: 1, type2: 0, custom_type1: 0,
            equipped: false, body_part: 0, enchant: 4, custom_type2: 0,
        }];
        let event = killer_world::WorldEvent::ChooseInventoryItem(l2_net::game::packets::ChooseInventoryItem { item_id: 729 });
        world.apply_world(&event);
        ui.world_event(&event, &world);
        ui.sync_windows(&world, Some(&tables), killer_platform::time::Instant::now());
        let rect = ui.desktop.window(WindowId::ITEM_ENCHANT).unwrap().rect;
        let grid = killer_ui::enchant_wnd::GRID_RECT;
        ui.mouse_move(rect.x + grid.x + 2, rect.y + grid.y + 2);
        let tip = ui.keys.tooltip.as_ref().expect("passar o mouse no item da grade de enchant mostra o tooltip");
        assert_eq!(tip.name, "Dual Swords");
        assert_eq!(tip.enchant, 4);
    }

    fn show_window(ui: &mut GameUi, id: WindowId, visible: bool) {
        if let Some(window) = ui.desktop.window_mut(id) {
            window.visible = visible;
        }
    }

    #[test]
    fn clicking_a_link_reports_its_action_on_release_over_the_same_link() {
        let mut ui = game_ui((1558, 933));
        ui.open_npc_html(&npc_html(MERCHANT, 0));
        let sell_y = ui.npc_viewer.as_ref().unwrap().lines.iter().find(|l| l.spans[0].text == "Sell item").unwrap().y;
        let (x, y) = (987 + 7 + 5, 494 + 30 + sell_y + 5);
        assert!(ui.mouse_down(x, y));
        assert_eq!(ui.desktop.dragging(), None);
        assert_eq!(ui.mouse_up().as_deref(), Some("bypass -h npc_%objectId%_Sell"));
        assert!(!ui.desktop.window(WindowId::NPC_HTML).unwrap().visible);
        assert_eq!(ui.mouse_up(), None);
        ui.open_npc_html(&npc_html(MERCHANT, 0));
        assert!(ui.mouse_down(x, y));
        ui.mouse_move(x, y - 40);
        assert_eq!(ui.mouse_up(), None);
        assert!(ui.mouse_down(987 + 150, 494 + 30 + sell_y + 5));
        assert_eq!(ui.mouse_up(), None);
        assert!(ui.mouse_down(987 + 100, 494 + 30 + 2));
        assert_eq!(ui.mouse_up(), None);
        assert_eq!(html_action("bypass -h npc_5_Chat 1"), Some(KeyAction::Bypass("npc_5_Chat 1".into())));
        assert_eq!(html_action("link merchant/7829.htm"), Some(KeyAction::LinkHtml("merchant/7829.htm".into())));
        assert_eq!(html_action("other"), None);
    }

    #[test]
    fn heine_links_and_table_buttons_answer_at_the_physical_position_of_the_ini_window() {
        let mut ui = game_ui((1558, 933));
        ui.open_npc_html(&npc_html(HEINE_PET, 0));
        let viewer = ui.npc_viewer.as_ref().unwrap();
        let buy_y = viewer.lines.iter().find(|l| l.spans[0].text.starts_with("Buy a pet")).unwrap().y;
        let quest_y = viewer.lines.iter().find(|l| l.spans[0].text == "Quest").unwrap().y;
        assert!(ui.mouse_down(987 + 7 + 20, 494 + 30 + buy_y + 8));
        assert_eq!(ui.mouse_up().as_deref(), Some("bypass -h npc_%objectId%_Buy 5710"));
        ui.open_npc_html(&npc_html(HEINE_PET, 0));
        assert!(ui.mouse_down(987 + 7 + 2, 494 + 30 + quest_y + 2));
        assert_eq!(ui.mouse_up().as_deref(), Some("bypass -h npc_%objectId%_Quest"));
        ui.open_npc_html(&npc_html(HEINE_PET, 0));
        assert!(ui.mouse_down(987 + 7 + 250, 494 + 30 + quest_y + 2));
        assert_eq!(ui.mouse_up(), None);
        ui.open_npc_html(&npc_html(AGIT_SALE, 0));
        let row_y =
            ui.npc_viewer.as_ref().unwrap().lines.iter().find(|l| l.widgets.iter().any(|w| w.width == 70)).unwrap().y;
        assert!(ui.mouse_down(987 + 7 + 200 + 30, 494 + 30 + row_y + 10));
        assert_eq!(ui.mouse_up().as_deref(), Some("bypass -h npc_%objectId%_sale2"));
        ui.open_npc_html(&npc_html(AGIT_SALE, 0));
        assert!(ui.mouse_down(987 + 7 + 130 + 30, 494 + 30 + row_y + 10));
        assert_eq!(ui.mouse_up().as_deref(), Some("bypass -h npc_%objectId%_selectedItems"));
        ui.open_npc_html(&npc_html(AGIT_SALE, 0));
        assert!(ui.mouse_down(987 + 7 + 60, 494 + 30 + row_y + 10));
        assert_eq!(ui.mouse_up(), None);
    }

    #[test]
    fn the_title_bar_drags_the_viewer_and_the_close_button_hides_it() {
        let mut ui = game_ui((1558, 933));
        ui.open_npc_html(&npc_html(MERCHANT, 0));
        assert!(ui.mouse_down(987 + 100, 494 + 10));
        assert_eq!(ui.desktop.dragging(), Some(WindowId::NPC_HTML));
        ui.mouse_move(987 + 90, 494 + 20);
        assert_eq!(ui.mouse_up(), None);
        assert_eq!(ui.desktop.window(WindowId::NPC_HTML).unwrap().rect, Rect::new(977, 504, 310, 401));
        assert!(ui.mouse_down(977 + 287 + 7, 504 + 3 + 7));
        assert_eq!(ui.mouse_up(), None);
        assert!(!ui.desktop.window(WindowId::NPC_HTML).unwrap().visible);
        assert!(ui.npc_viewer.is_none());
        assert!(!ui.mouse_down(977 + 100, 504 + 100));
    }

    #[test]
    fn the_wheel_scrolls_the_viewer_only_over_it() {
        let mut ui = game_ui((1558, 933));
        assert!(!ui.wheel_over(990, 500, -1.0, &[]));
        ui.open_npc_html(&npc_html(&format!("<html><body>{}</body></html>", "line<br>".repeat(40)), 0));
        assert!(!ui.wheel_over(500, 500, -1.0, &[]));
        assert!(ui.wheel_over(990, 600, -1.0, &[]));
        assert_eq!(ui.npc_viewer.as_ref().unwrap().scroll, 39);
        assert!(ui.wheel_over(990, 600, 3.0, &[]));
        assert_eq!(ui.npc_viewer.as_ref().unwrap().scroll, 0);
    }

    #[test]
    fn windows_start_at_the_ini_positions_rearranged_to_the_real_size() {
        let ui = game_ui((1558, 933));
        let status = ui.desktop.window(WindowId::STATUS).unwrap();
        assert_eq!(status.rect, Rect::new(0, 0, 172, 84));
        assert_eq!(status.grip, Rect::new(0, 0, 12, 84));
        let chat = ui.desktop.window(WindowId::CHAT).unwrap();
        assert_eq!(chat.rect, Rect::new(0, 746, CHAT_WIDTH, 187));
        assert_eq!(chat.grip, Rect::new(0, 0, CHAT_WIDTH, CHAT_HEADER));
        assert_eq!(ui.chat.height, 187);
        assert_eq!(ui.alpha, 1.0);
        let target = ui.desktop.window(WindowId::TARGET).unwrap();
        assert_eq!(target.rect, Rect::new(677, 0, 172, 46));
        assert_eq!(target.grip, Rect::new(0, 0, 12, 46));
        let shortcut = ui.desktop.window(WindowId::SHORTCUT).unwrap();
        assert_eq!(shortcut.rect, Rect::new(1478, 168, 46, 504));
        assert_eq!(shortcut.grip, Rect::new(0, 0, 46, 12));
        assert!(ui.shortcut.vertical);
        let ui = game_ui((1558, 1782));
        assert_eq!(ui.desktop.size, (1558, 1782));
        assert_eq!(ui.desktop.window(WindowId::STATUS).unwrap().rect.y, 0);
        assert_eq!(ui.desktop.window(WindowId::CHAT).unwrap().rect.y, 1595);
    }

    #[test]
    fn without_inis_the_status_sits_at_the_origin_and_the_chat_at_the_bottom() {
        let ui = GameUi::new(
            font("s"),
            font("l"),
            Options::default(),
            WindowsInfo::default(),
            ChatFilter::from_ini(&parse_ini(CHAT_FILTER)),
            (1280, 720),
        ).unwrap();
        assert_eq!(ui.desktop.window(WindowId::STATUS).unwrap().rect, Rect::new(0, 0, 172, 84));
        assert_eq!(ui.desktop.window(WindowId::CHAT).unwrap().rect, Rect::new(0, 720 - 187, CHAT_WIDTH, 187));
        assert_eq!(ui.desktop.window(WindowId::TARGET).unwrap().rect, Rect::new(640 - 86, 0, 172, 46));
        assert_eq!(ui.desktop.window(WindowId::SHORTCUT).unwrap().rect, Rect::new(1280 - 46, 360 - 252, 46, 504));
        assert_eq!(ui.alpha, 1.0);
    }

    #[test]
    fn frame_paints_the_status_grip_at_the_window_and_the_chat_header() {
        let ui = game_ui((1558, 933));
        let list = ui.frame(&world(), &[], &[], None, &WorldViews::default());
        let grip = with_texture(&list.quads, SMALLBAR_1);
        let status = grip.iter().find(|quad| quad.x == 0.0 && quad.y == 0.0).expect("grip do status");
        assert_eq!(status.color[3], 1.0);
        let level = with_texture(&list.quads, PS_LEVELBACK);
        assert_eq!((level[0].x, level[0].y), (16.0, 6.0));
        let hp = with_texture(&list.quads, PS_HPBAR);
        assert_eq!((hp[0].x, hp[0].y, hp[0].w), (16.0, 41.0, 75.0));
        let header = with_texture(&list.quads, CHATTING_BACK2);
        assert_eq!((header[0].x, header[0].y, header[0].w), (0.0, 746.0, CHAT_WIDTH as f32));
        let name: Vec<&Quad> = list.quads.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e").collect();
        assert!(name.iter().any(|q| q.color == TEXT.rgba() && q.y == 9.0 && q.x == 40.0));
    }

    #[test]
    fn target_window_paints_the_selected_entity_and_the_shortcut_bar_is_always_there() {
        let ui = game_ui((1558, 933));
        let mut world = world();
        let list = ui.frame(&world, &[], &[], None, &WorldViews::default());
        assert!(with_texture(&list.quads, HP_BAR_BACK).is_empty());
        let back = with_texture(&list.quads, SHORTCUT_BACKV);
        assert_eq!((back[0].x, back[0].y), (1478.0, 180.0));
        let mut gremlin = me();
        gremlin.object_id = 2;
        gremlin.kind = EntityKind::Npc { npc_id: 1, rhand: 0, chest: 0, lhand: 0, attackable: true };
        gremlin.name = "Gremlin".into();
        gremlin.hp = Some((25, 100));
        world.entities.insert(2, gremlin);
        assert!(world.select(2));
        let view = target_view(&world, world.targeted().unwrap(), "Gremlin".into(), None);
        assert_eq!((view.kind, view.name.as_str(), view.hp), (TargetKind::Monster, "Gremlin", Some((25, 100))));
        let list = ui.frame(&world, &[], &[], Some(&view), &WorldViews::default());
        let hp = with_texture(&list.quads, HP_BAR);
        assert_eq!((hp[0].x, hp[0].y, hp[0].w), (693.0, 26.0, 37.0));
        assert!(with_texture(&list.quads, MP_BAR_BACK).is_empty());
        assert!(list.quads.iter().any(|q| q.texture == "L2Font-e.SmallFont-e" && q.y == 9.0 && q.x >= 677.0 + 12.0));
        world.select(1);
        let view = target_view(&world, world.targeted().unwrap(), "Killer".into(), None);
        assert_eq!((view.kind, view.mp), (TargetKind::Me, Some((200, 300))));
        let mut other = me();
        other.object_id = 3;
        world.entities.insert(3, other);
        world.select(3);
        assert_eq!(target_view(&world, world.targeted().unwrap(), String::new(), None).kind, TargetKind::Player);
    }

    #[test]
    fn incoming_chat_goes_through_the_word_filter_with_sysstring_740() {
        let mut tables = DatTables { obscene: vec!["bad".into()], ..Default::default() };
        tables.sysstring.insert(BAD_WORD_SYSSTRING, "-_-".into());
        let message = l2_net::game::Say2::text(1, 0, "Ann", "a BAD word");
        assert_eq!(say_line(&tables, &message).unwrap().text, "a -_- word");
    }

    #[test]
    fn a_selected_door_paints_its_name_and_hp_bar_and_a_static_object_only_the_name() {
        let ui = game_ui((1558, 933));
        let mut world = World::default();
        world.apply_world(&killer_world::WorldEvent::DoorInfo(l2_net::game::DoorInfo { object_id: 100, door_id: 17220001 }));
        world.apply_world(&killer_world::WorldEvent::DoorStatusUpdate(l2_net::game::DoorStatusUpdate {
            object_id: 100, closed: true, damage: 0, show_hp: true, door_id: 17220001, max_hp: 200, hp: 100,
        }));
        world.apply_world(&killer_world::WorldEvent::StaticObject(l2_net::game::StaticObject { static_object_id: 24160024, object_id: 900 }));
        let mut tables = DatTables::default();
        tables.staticobject.insert(17220001, "Clan Hall Door".into());
        tables.staticobject.insert(24160024, "Throne".into());
        assert!(static_target_view(&world, Some(&tables)).is_none());
        assert!(world.select(100));
        let door = static_target_view(&world, Some(&tables)).unwrap();
        assert_eq!((door.kind, door.name.as_str(), door.hp), (TargetKind::Door, "Clan Hall Door", Some((100, 200))));
        let list = ui.frame(&world, &[], &[], Some(&door), &WorldViews::default());
        assert_eq!(with_texture(&list.quads, HP_BAR).len(), 1);
        assert!(with_texture(&list.quads, MP_BAR_BACK).is_empty());
        assert!(world.select(900));
        let object = static_target_view(&world, Some(&tables)).unwrap();
        assert_eq!((object.kind, object.name.as_str(), object.hp), (TargetKind::Npc, "Throne", None));
        let list = ui.frame(&world, &[], &[], Some(&object), &WorldViews::default());
        assert!(with_texture(&list.quads, HP_BAR_BACK).is_empty());
    }

    #[test]
    fn frame_without_me_or_with_the_status_hidden_paints_no_status() {
        let mut ui = game_ui((1558, 933));
        let list = ui.frame(&World::default(), &[], &[], None, &WorldViews::default());
        assert!(with_texture(&list.quads, PS_LEVELBACK).is_empty());
        assert!(!with_texture(&list.quads, CHATTING_BACK2).is_empty());
        ui.status_visible = false;
        let list = ui.frame(&world(), &[], &[], None, &WorldViews::default());
        assert!(with_texture(&list.quads, PS_LEVELBACK).is_empty());
    }

    #[test]
    fn status_view_takes_the_vitals_of_me_and_zeroes_what_is_missing() {
        let view = status_view(&me());
        assert_eq!(view.name, "Killer");
        assert_eq!((view.level, view.hp, view.mp, view.cp), (52, (450, 900), (200, 300), (100, 100)));
        assert_eq!(format!("{:.2}", view.exp_percent()), "16.34");
        let mut bare = me();
        bare.hp = None;
        bare.cp = None;
        assert_eq!((status_view(&bare).hp, status_view(&bare).cp), ((0, 0), (0, 0)));
    }

    #[test]
    fn labels_are_centered_with_the_name_15_px_above_the_anchor_and_the_title_two_lines_above() {
        let ui = game_ui((1558, 933));
        let label = Label {
            object_id: 1,
            name: "!!".into(),
            title: "!!!!".into(),
            store_sign: None, speech: None, modern_text_size: None, name_visible: true,
            name_color: [255, 0, 255],
            title_color: [0x3f, 0x8b, 0xfe],
            screen: [400.0, 300.0], depth: 0.5,
            clan_crest: None,
            ally_crest: None,
            mark: None,
            relation_icons: Default::default(),
            broadcast_icon: None,
        };
        let list = ui.frame(&World::default(), &[label], &[], None, &WorldViews::default());
        let text: Vec<&Quad> =
            list.quads.iter().filter(|q| q.texture == "L2Font-e.LargeFont-e" && q.y < 700.0).collect();
        assert_eq!(text.len(), 6);
        assert_eq!((text[0].x, text[0].y), (394.0, 285.0));
        assert_eq!(text[0].color, Argb(0xFFFF00FF).rgba());
        assert_eq!((text[2].x, text[2].y), (388.0, 274.0));
        assert_eq!(text[2].color, Argb(0xFF3F8BFE).rgba());
        let untitled = Label { title: String::new(), ..label_at(10.0, 10.0) };
        let list = ui.frame(&World::default(), &[untitled], &[], None, &WorldViews::default());
        assert_eq!(list.quads.iter().filter(|q| q.texture == "L2Font-e.LargeFont-e" && q.y < 700.0).count(), 2);
        let crested = Label { clan_crest: Some("crest.crest_5".into()), ally_crest: Some("crest.ally_6".into()), ..label_at(400.0, 300.0) };
        let list = ui.frame(&World::default(), std::slice::from_ref(&crested), &[], None, &WorldViews::default());
        let ally = with_texture(&list.quads, "crest.ally_6");
        assert_eq!((ally[0].x, ally[0].y, ally[0].w, ally[0].h, ally[0].v), (382.0, 285.0, 8.0, 12.0, 4.0));
        let crest = with_texture(&list.quads, "crest.crest_5");
        assert_eq!((crest[0].x, crest[0].y, crest[0].w, crest[0].h), (390.0, 285.0, 16.0, 12.0));
        let text: Vec<&Quad> = list.quads.iter().filter(|q| q.texture == "L2Font-e.LargeFont-e" && q.y == 285.0).collect();
        assert_eq!(text[0].x, 406.0);
        assert_eq!(crate::characters::label_rects(&crested, &ui.large)[0], (382, 285, 36, ui.large.line_height as i32));
        let marked = Label { mark: Some(crate::characters::TargetMark::Target), ..crested.clone() };
        let list = ui.frame(&World::default(), std::slice::from_ref(&marked), &[], None, &WorldViews::default());
        let marks = with_texture(&list.quads, "L2ui.NWindow.target");
        let name_x = 406.0;
        let name_w = ui.large.measure(&marked.name) as f32;
        assert_eq!(marks.len(), 2);
        assert_eq!((marks[0].x, marks[0].y, marks[0].w, marks[0].h), (name_x - 19.0 - 26.0, 284.0, 19.0, 16.0));
        assert_eq!((marks[1].x, marks[1].y), (name_x + name_w, 284.0));
        assert_eq!((marks[0].u, marks[0].v, marks[0].uw, marks[0].vh), (0.0, 0.0, 19.0, 16.0));
        let hidden = Label { name_visible: false, mark: Some(crate::characters::TargetMark::Normal), ..label_at(400.0, 300.0) };
        let list = ui.frame(&World::default(), &[hidden], &[], None, &WorldViews::default());
        assert_eq!(with_texture(&list.quads, "L2ui.NWindow.normal").len(), 2);
        assert!(!list.quads.iter().any(|q| q.texture == "L2Font-e.LargeFont-e" && q.y < 700.0));
    }

    fn label_at(x: f32, y: f32) -> Label {
        Label { object_id: 1, name: "!!".into(), title: "!".into(), store_sign: None, speech: None, modern_text_size: None, name_visible: true, name_color: [255; 3], title_color: [255; 3], screen: [x, y], depth: 0.5, clan_crest: None, ally_crest: None, mark: None, relation_icons: Default::default(), broadcast_icon: None }
    }

    #[test]
    fn a_printable_key_focuses_the_chat_line_unless_enter_chatting_is_on_or_the_key_belongs_to_the_camera() {
        let mut ui = game_ui((1024, 768));
        ui.keys.enter_chatting = false;
        assert_eq!(ui.key(InputKey::Char('w')), KeyAction::Ignored);
        assert!(!ui.chat.focused);
        assert_eq!(ui.key(InputKey::Char('o')), KeyAction::Consumed);
        assert!(ui.chat.focused);
        assert_eq!(ui.chat.input.text, "o");
        ui.chat.focused = false;
        ui.chat.input.clear();
        ui.keys.enter_chatting = true;
        assert_eq!(ui.key(InputKey::Char('o')), KeyAction::Ignored);
        assert!(!ui.chat.focused);
    }

    #[test]
    fn enter_sends_the_line_keeps_the_focus_without_enter_chatting_and_drops_it_with_enter_chatting() {
        let mut ui = game_ui((1024, 768));
        ui.chat.focused = true;
        ui.chat.input.text = "oi".into();
        ui.chat.input.cursor = 2;
        ui.keys.enter_chatting = false;
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Say("oi".into()));
        assert!(ui.chat.focused);
        ui.chat.input.text = "oi".into();
        ui.keys.enter_chatting = true;
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Say("oi".into()));
        assert!(!ui.chat.focused);
    }

    #[test]
    fn slash_lines_run_the_text_command_table_and_chat_lines_follow_the_tab_and_the_flood_control() {
        use killer_ui::chat_line::LineEffect;
        let mut ui = game_ui((1024, 768));
        ui.keys.commandname.insert("sit".into(), 4);
        ui.keys.commandname.insert("stand".into(), 5);
        let world = World::default();
        let effects = ui.chat_line("\u{ff0f}sit", &world);
        assert_eq!(effects, vec![LineEffect::Command("\u{ff0f}sit".into())]);
        ui.keys.chat_command(&world, "\u{ff0f}sit");
        ui.keys.chat_command(&world, "/stand");
        let commands = ui.take_commands();
        assert!(commands.contains(&UiCommand::Net(killer_world::NetCommand::ChangeWaitType { standing: false })));
        assert!(commands.contains(&UiCommand::Net(killer_world::NetCommand::ChangeWaitType { standing: true })));
        ui.chat.old_chat = true;
        ui.chat.select_tab(2);
        let effects = ui.chat_line(&ui.chat.input.text.clone(), &world);
        assert!(effects.is_empty());
        let effects = ui.chat_line("#oi", &world);
        assert_eq!(effects, vec![LineEffect::Chat { kind: 3, text: "oi".into(), target: String::new() }]);
        for _ in 0..10 {
            ui.chat_line(&"x".repeat(150), &world);
        }
        assert!(ui.chat.limiter.limited);
        let blocked = ui.chat_line("hello", &world);
        assert_eq!(blocked, vec![LineEffect::System(killer_ui::chat_line::FLOOD_MESSAGE)]);
        let start = killer_platform::time::Instant::now();
        for second in 0..2000u64 {
            ui.tick_chat_limiter(start + std::time::Duration::from_secs(second));
        }
        assert!(!ui.chat.limiter.limited);
        let released = ui.take_commands();
        assert!(released.iter().any(|command| matches!(command, UiCommand::ColoredSystemMessage { id: 965, .. })));
    }

    #[test]
    fn the_builder_level_from_user_info_gates_the_double_slash_commands() {
        use killer_ui::chat_line::LineEffect;
        let mut ui = game_ui((1024, 768));
        let mut world = World::default();
        assert!(ui.chat_line("//admin_help", &world).is_empty());
        world.builder_level = 1;
        assert_eq!(ui.chat_line("//admin_help", &world), vec![LineEffect::Builder("admin_help".into())]);
    }

    #[test]
    fn text_capture_adds_its_start_and_end_lines_to_the_current_tab_and_writes_what_the_tab_shows() {
        use killer_ui::text_capture::Stamp;
        let stamp = Stamp { month: 1, day: 2, hour: 3, minute: 4, second: 5 };
        let mut ui = game_ui((1024, 768));
        assert!(ui.chat.capture.exec("TextCapture ON"));
        let (name, _) = ui.tick_text_capture(stamp).unwrap();
        assert_eq!(name, "L2_01_02_03_04.log");
        assert_eq!(ui.chat.active_list().last().unwrap().text, "[ Text Capture Start ][ FileName : L2_01_02_03_04.log ]");
        let mut log = killer_world::chat::ChatLog::default();
        log.push(ChatLine::new(CHAT_ALL, "a", "oi"));
        ui.ingest_chat(&log);
        let (_, bytes) = ui.tick_text_capture(stamp).unwrap();
        assert!(bytes.len() > 20);
        assert!(ui.chat.capture.exec("TextCapture OFF"));
        ui.tick_text_capture(stamp);
        assert_eq!(ui.chat.active_list().last().unwrap().text, killer_ui::text_capture::END_LINE);
    }

    #[test]
    fn dragging_a_chat_tab_off_the_bar_floats_its_list_and_dropping_it_on_the_bar_merges_it_back() {
        let mut ui = game_ui((1558, 933));
        let world = World::default();
        let chat = ui.desktop.window(WindowId::CHAT).unwrap().rect;
        let (tx, ty) = (chat.x + killer_ui::chat::TAB_X[2] + 10, chat.y + chat.h - killer_ui::chat::CHAT_FOOTER + 5);
        let id = WindowId(killer_ui::chat::ChatView::split_window_id(2));
        assert!(ui.mouse_down_at(tx, ty, Instant::now(), &world));
        assert!(ui.desktop.window(id).is_none());
        ui.mouse_move(tx + 5, ty - 100);
        assert!(ui.chat.detached[2]);
        let floating = ui.desktop.window(id).unwrap();
        assert!(floating.visible);
        assert_eq!((floating.rect.x, floating.rect.y), (chat.x + 5, chat.y - 100));
        ui.mouse_move(tx, ty - 300);
        assert_eq!(ui.desktop.window(id).unwrap().rect.y, chat.y - 300);
        ui.mouse_up_in(&world);
        assert!(ui.chat.detached[2]);
        assert!(ui.desktop.window(id).unwrap().visible);
        let list = ui.frame(&world, &[], &[], None, &WorldViews::default());
        assert!(!with_texture(&list.quads, killer_ui::chat::CHATTING_TAB_SPLIT).is_empty());
        assert!(ui.mouse_down_at(tx, ty - 300, Instant::now(), &world));
        ui.mouse_move(tx + 2, ty + 2);
        ui.mouse_up_in(&world);
        assert!(!ui.chat.detached[2]);
        assert!(!ui.desktop.window(id).unwrap().visible);
        assert_eq!(ui.chat.active_tab, 2);
    }

    #[test]
    fn a_link_with_msg_asks_before_it_runs_and_the_answer_decides() {
        let mut ui = game_ui((1024, 768));
        ui.html_confirm(None, "1983;Pet\u{1}bypass -h npc_1_Buy 5", Instant::now());
        assert!(ui.dialog.is_some());
        assert_eq!(ui.dialog.as_ref().unwrap().answer, DialogAnswer::HtmlLink { action: "bypass -h npc_1_Buy 5".into(), source: HtmlSource::Npc });
        assert_eq!(ui.finish_dialog(false), KeyAction::Consumed);
        assert!(ui.dialog.is_none());
        ui.html_confirm(None, "1983;Pet\u{1}bypass -h npc_1_Buy 5", Instant::now());
        assert_eq!(ui.finish_dialog(true), KeyAction::Bypass("npc_1_Buy 5".into()));
        assert_eq!(html_action_result("bypass -h npc_1_Buy 5", true), 2);
        assert_eq!(html_action_result("bypass npc_1_Buy 5", true), 1);
        assert_eq!(html_action_result("bypass", true), 0);
        assert_eq!(html_action_result("link a.htm", true), 1);
        assert_eq!(html_action_result("1983;Pet\u{1}bypass -h npc_1_Buy 5", true), 2);
    }

    #[test]
    fn chat_lines_are_painted_with_their_channel_colors() {
        let mut ui = game_ui((1558, 933));
        let lines = [chat_entry(&ChatLine::new(CHAT_ALL, "a", "!")), chat_entry(&ChatLine::new(CHAT_SHOUT, "b", "!"))];
        assert_eq!(lines[0], ChatEntry::new(CHAT_ALL, "a", "!"));
        lines.iter().for_each(|line| ui.chat.ingest(line));
        let list = ui.frame(&World::default(), &[], &lines, None, &WorldViews::default());
        let text: Vec<&Quad> = list.quads.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e").collect();
        let shout: Vec<&&Quad> = text.iter().filter(|q| q.color == chat_color(CHAT_SHOUT).rgba()).collect();
        assert_eq!(shout.len(), 3);
        assert_eq!((shout[0].x, shout[0].y), (killer_ui::chat::LINE_TEXT_X as f32, 746.0 + killer_ui::chat::LINE_TEXT_Y as f32 + 15.0));
    }

    #[test]
    fn mouse_down_on_the_grip_drags_and_outside_the_windows_is_ignored() {
        let mut ui = game_ui((1558, 933));
        assert!(!ui.mouse_down(600, 600));
        assert!(!ui.mouse_move(610, 610));
        assert!(ui.mouse_down(5, 40));
        assert!(ui.mouse_move(105, 90));
        assert_eq!(ui.desktop.window(WindowId::STATUS).unwrap().rect, Rect::new(100, 50, 172, 84));
        ui.mouse_up();
        assert!(!ui.mouse_move(500, 500));
        assert_eq!(ui.desktop.window(WindowId::STATUS).unwrap().rect.x, 100);
        assert!(ui.mouse_down(150, 90));
        assert!(!ui.mouse_move(160, 100));
        assert_eq!(ui.desktop.window(WindowId::STATUS).unwrap().rect.x, 100);
        ui.mouse_up();
    }

    #[test]
    fn dragging_the_thin_bar_of_the_shortcut_window_moves_it_and_the_body_does_not() {
        let mut ui = game_ui((1558, 933));
        assert_eq!(ui.desktop.window(WindowId::SHORTCUT).unwrap().grip_rect(), Rect::new(1478, 168, 46, 12));
        assert!(ui.mouse_down(1500, 173));
        assert_eq!(ui.desktop.dragging(), Some(WindowId::SHORTCUT));
        assert!(ui.mouse_move(1400, 273));
        assert_eq!(ui.desktop.window(WindowId::SHORTCUT).unwrap().rect, Rect::new(1378, 268, 46, 504));
        ui.mouse_up();
        assert!(!ui.mouse_move(1300, 300));
        assert_eq!(ui.desktop.window(WindowId::SHORTCUT).unwrap().rect.x, 1378);
        assert!(ui.mouse_down(1400, 400));
        assert_eq!(ui.desktop.dragging(), None);
        assert!(!ui.mouse_move(1300, 300));
        ui.mouse_up();
        assert_eq!(ui.desktop.window(WindowId::SHORTCUT).unwrap().rect, Rect::new(1378, 268, 46, 504));
        assert!(ui.shortcut.vertical);
    }

    #[test]
    fn a_double_click_on_the_inventory_grid_uses_the_item_and_disarms_the_next_drag() {
        use killer_ui::inventory::{GRID_RECT, INVENTORY_SIZE};
        let mut ui = game_ui((1558, 933));
        let mut world = world();
        crate::ui_keys::demo_fill(&mut world);
        ui.sync_windows(&world, None, killer_platform::time::Instant::now());
        ui.world_event(&killer_world::WorldEvent::ItemList(l2_net::game::ItemList { show_window: true, items: vec![] }), &World::default());
        let rect = ui.desktop.window(WindowId::INVENTORY).unwrap().rect;
        assert_eq!((rect.w, rect.h), INVENTORY_SIZE);
        let (x, y) = (rect.x + GRID_RECT.x + 10 + 37, rect.y + GRID_RECT.y + 10);
        let t0 = killer_platform::time::Instant::now();
        let bar = ui.desktop.window(WindowId::SHORTCUT).unwrap().rect;
        let (bx, by) = (bar.x + 30, bar.y + 70);
        let reg = UiCommand::Net(killer_world::NetCommand::ShortcutReg { kind: 1, slot: 1, id: 107, character_type: 1 });
        let drop_on_bar = |ui: &mut GameUi| {
            ui.mouse_move(bx, by);
            ui.mouse_up_in(&world);
            crate::ui_keys::silent(ui.take_commands())
        };
        assert!(ui.mouse_down_at(x, y, t0, &world));
        assert_eq!(drop_on_bar(&mut ui), vec![reg.clone()]);
        assert!(ui.mouse_down_at(x, y, t0 + Duration::from_millis(1000), &world));
        assert_eq!(drop_on_bar(&mut ui), vec![reg.clone()]);
        assert!(ui.mouse_down_at(x + 1, y - 1, t0 + Duration::from_millis(1000 + DOUBLE_CLICK_TIME_MS as u64), &world));
        assert_eq!(crate::ui_keys::silent(ui.take_commands()), vec![UiCommand::Net(killer_world::NetCommand::UseItem { object_id: 107, ctrl: false })]);
        assert!(drop_on_bar(&mut ui).is_empty());
        assert!(ui.mouse_down_at(x, y, t0 + Duration::from_millis(2600), &world));
        assert!(drop_on_bar(&mut ui).is_empty());
        assert!(ui.mouse_down_at(x, y, t0 + Duration::from_millis(3300), &world));
        assert_eq!(drop_on_bar(&mut ui), vec![reg]);
        assert!(ui.mouse_down_at(x + 3, y, t0 + Duration::from_millis(3400), &world));
        assert!(crate::ui_keys::silent(ui.take_commands()).is_empty());
        ui.mouse_move(x + 3, y);
        ui.mouse_up_in(&world);
        let (px, py) = (rect.x + 8 + 7 + 5, rect.y + 31 + 45 + 5);
        assert!(ui.mouse_down_at(px, py, t0 + Duration::from_millis(5000), &world));
        ui.mouse_move(px, py);
        ui.mouse_up_in(&world);
        assert!(crate::ui_keys::silent(ui.take_commands()).is_empty());
        assert!(ui.mouse_down_at(px, py, t0 + Duration::from_millis(5100), &world));
        assert_eq!(crate::ui_keys::silent(ui.take_commands()), vec![UiCommand::Net(killer_world::NetCommand::Unequip { body_part: killer_ui::inventory::BODY_PART_R_HAND })]);
    }

    #[test]
    fn a_press_on_the_player_status_body_selects_the_player_itself() {
        let mut ui = game_ui((1558, 933));
        let world = world();
        let rect = ui.desktop.window(WindowId::STATUS).unwrap().rect;
        let grip = ui.desktop.window(WindowId::STATUS).unwrap().grip_rect();
        let now = killer_platform::time::Instant::now();
        assert!(ui.mouse_down_at(grip.x + 2, grip.y + 2, now, &world));
        assert_eq!(ui.take_status_click(&world), None);
        ui.mouse_up_in(&world);
        assert!(ui.mouse_down_at(rect.x + 80, rect.y + rect.h - 6, now, &world));
        assert_eq!(ui.take_status_click(&world), Some(KeyAction::Consumed));
        let me = world.me.as_ref().unwrap().object_id;
        assert!(ui.keys.take_commands().iter().any(|c| matches!(c, UiCommand::Net(NetCommand::Action { object_id, .. }) if *object_id == me)));
        assert_eq!(ui.take_status_click(&world), None);
    }

    #[test]
    fn a_press_on_the_target_window_body_reports_the_click_but_not_on_its_grip_or_buttons() {
        let mut ui = game_ui((1558, 933));
        let world = world();
        let rect = ui.desktop.window(WindowId::TARGET).unwrap().rect;
        let now = killer_platform::time::Instant::now();
        assert!(ui.mouse_down_at(rect.x + 80, rect.y + 30, now, &world));
        assert!(std::mem::take(&mut ui.target_clicked));
        ui.mouse_up_in(&world);
        assert!(ui.mouse_down_at(rect.x + 3, rect.y + 30, now, &world));
        assert!(!ui.target_clicked);
        ui.mouse_up_in(&world);
        assert!(ui.mouse_down_at(rect.x + rect.w - 10, rect.y + 10, now, &world));
        assert!(!ui.target_clicked);
        ui.mouse_up_in(&world);
        assert!(ui.mouse_down_at(rect.x + 5, rect.y + 10, now, &world));
        assert!(!ui.target_clicked);
    }

    #[test]
    fn hovering_an_inventory_slot_or_a_bar_slot_shows_the_item_tooltip_above_the_cursor_at_once() {
        use killer_ui::inventory::GRID_RECT;
        use killer_ui::tooltip::FRAME_TEXTURES;
        let mut ui = game_ui((1558, 933));
        let mut world = world();
        crate::ui_keys::demo_fill(&mut world);
        ui.sync_windows(&world, None, killer_platform::time::Instant::now());
        ui.world_event(&killer_world::WorldEvent::ItemList(l2_net::game::ItemList { show_window: true, items: vec![] }), &World::default());
        let rect = ui.desktop.window(WindowId::INVENTORY).unwrap().rect;
        let (x, y) = (rect.x + GRID_RECT.x + 10 + 37, rect.y + GRID_RECT.y + 10);
        ui.mouse_move(x, y);
        let tip = ui.keys.tooltip.clone().unwrap();
        assert_eq!((tip.enchant, tip.count), (0, Some(15)));
        assert_eq!(tip.lines[0].label, "sysstring 52");
        let list = ui.frame(&world, &[], &[], None, &WorldViews::default());
        let corner = with_texture(&list.quads, FRAME_TEXTURES[0]);
        let (w, h) = tip.size(&ui.small);
        assert_eq!((corner[0].x, corner[0].y), (x as f32, (y - h) as f32));
        assert!(w > 0);
        ui.mouse_move(rect.x + 100, rect.y + 5);
        assert!(ui.keys.tooltip.is_none());
        let bar = ui.desktop.window(WindowId::SHORTCUT).unwrap().rect;
        ui.mouse_move(bar.x + 30, bar.y + 40);
        assert_eq!(ui.keys.tooltip.as_ref().map(|t| t.count), Some(Some(15)));
        ui.mouse_move(10, 500);
        assert!(ui.keys.tooltip.is_none());
    }

    #[test]
    fn the_target_info_button_toggles_the_76_px_mode_and_the_view_takes_the_pledge_names() {
        let mut ui = game_ui((1558, 933));
        let mut world = world();
        let rect = ui.desktop.window(WindowId::TARGET).unwrap().rect;
        let now = killer_platform::time::Instant::now();
        assert_eq!(rect.h, 46);
        assert!(ui.mouse_down_at(rect.x + 5, rect.y + 10, now, &world));
        assert!(ui.target_expanded);
        assert_eq!(ui.desktop.window(WindowId::TARGET).unwrap().rect.h, 76);
        ui.mouse_up_in(&world);
        let mut gremlin = me();
        gremlin.object_id = 2;
        gremlin.clan_id = 33;
        gremlin.clan_crest_id = 5;
        gremlin.ally_crest_id = 6;
        world.entities.insert(2, gremlin);
        world.target = Some(2);
        world.pledges.insert(33, killer_world::PledgeRecord { pledge_id: 33, name: "Canastra".into(), ally_name: "Baralho".into(), crest_id: 5, ally_id: 44, ally_crest_id: 6, large_crest_id: 0, ..Default::default() });
        let view = target_view(&world, world.targeted().unwrap(), "Gremlin".into(), None);
        assert_eq!((view.clan.as_deref(), view.ally.as_deref(), view.clan_crest.as_deref()), (Some("Canastra"), Some("Baralho"), None));
        world.pledge_window.crests.insert(5, vec![1]);
        let view = target_view(&world, world.targeted().unwrap(), "Gremlin".into(), None);
        assert_eq!(view.clan_crest.as_deref(), Some("crest.crest_5"));
        let list = ui.frame(&world, &[], &[], Some(&view), &WorldViews::default());
        assert_eq!(with_texture(&list.quads, "crest.crest_5").len(), 1);
        assert!(ui.mouse_down_at(rect.x + 5, rect.y + 10, now, &world));
        assert!(!ui.target_expanded);
        assert_eq!(ui.desktop.window(WindowId::TARGET).unwrap().rect.h, 46);
    }

    #[test]
    fn the_use_item_popup_opens_centered_as_type_0x45_and_ok_answers_with_the_object_id() {
        let mut ui = game_ui((1558, 933));
        let mut world = world();
        let now = killer_platform::time::Instant::now();
        ui.keys.dialogs.push(killer_world::Dialog::UseItemPopup { object_id: 108, message_id: 1080, item_name: "Facelifting Potion - A".into() });
        assert!(ui.tick_dialog(&mut world, None, now).is_none());
        let dialog = ui.dialog.as_ref().unwrap();
        assert_eq!(dialog.view.dialog_type, DIALOG_TYPE_USE_ITEM_POPUP);
        assert_eq!(dialog.window, WindowId::DIALOG);
        assert_eq!(dialog.view.message, "systemmsg 1080");
        assert!(ui.desktop.window(WindowId::DIALOG).unwrap().visible);
        let rect = ui.desktop.window(WindowId::DIALOG).unwrap().rect;
        let (bx, by) = (rect.x + dialog::FIRST_BUTTON_X + 5, rect.y + dialog::BUTTON_Y + 5);
        assert!(ui.mouse_down_at(bx, by, now, &world));
        ui.mouse_move(bx, by);
        assert_eq!(ui.dialog_released(), Some(KeyAction::Dialog(DialogAnswer::UseItem { object_id: 108 }, true)));
        assert!(ui.dialog.is_none());
        ui.keys.dialogs.push(killer_world::Dialog::RecipeRegister { object_id: 109, item_name: "Recipe: Wooden Arrow".into() });
        assert!(ui.tick_dialog(&mut world, None, now).is_none());
        assert_eq!(ui.dialog.as_ref().unwrap().view.dialog_type, DIALOG_TYPE_RECIPE_REGISTER);
        assert_eq!(ui.dialog.as_ref().unwrap().view.message, format!("systemmsg {RECIPE_REGISTER_SYSMSG}"));
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Dialog(DialogAnswer::UseItem { object_id: 109 }, false));
    }

    #[test]
    fn the_bar_changes_page_by_alt_f_keys_and_by_the_prev_next_buttons_and_paints_that_page() {
        use killer_ui::draw::DIGIT_TEXTURE;
        use killer_ui::keys::{VK_F1, WM_SYSKEYDOWN, WM_SYSKEYUP};
        let mut ui = game_ui((1558, 933));
        let mut world = world();
        crate::ui_keys::demo_fill(&mut world);
        world.shortcuts.push(l2_net::game::Shortcut {
            slot: 26,
            kind: l2_net::game::ShortcutKind::Item { object_id: 107 },
            character_type: 1,
        });
        ui.sync_windows(&world, None, killer_platform::time::Instant::now());
        let bar_x = 1478;
        let bar_y = 168;
        let digits = |list: &DrawList| {
            with_texture(&list.quads, DIGIT_TEXTURE)
                .iter()
                .filter(|q| q.x >= bar_x as f32 && q.x < (bar_x + 46) as f32 && q.y == (bar_y + 16) as f32)
                .map(|q| q.x)
                .collect::<Vec<_>>()
        };
        let list = ui.frame(&world, &[], &[], None, &WorldViews::default());
        assert_eq!(with_texture(&list.quads, killer_ui::shortcut::SLOT_OUTLINE).len(), 4);
        assert_eq!(digits(&list), [(bar_x + 22 - ui.small.measure("1") as i32 / 2) as f32]);
        assert!(ui.hook_key(WM_SYSKEYDOWN, VK_F1 + 2, 0x2000 << 16, &world));
        assert!(!ui.hook_key(WM_SYSKEYUP, VK_F1 + 2, 0xc000 << 16, &world));
        assert_eq!(ui.shortcut.page, 2);
        let list = ui.frame(&world, &[], &[], None, &WorldViews::default());
        let outlines = with_texture(&list.quads, killer_ui::shortcut::SLOT_OUTLINE);
        assert_eq!(outlines.len(), 1);
        assert_eq!((outlines[0].x, outlines[0].y), ((bar_x + 6) as f32, (bar_y + 107 - 1) as f32));
        assert_eq!(digits(&list).len(), 1);
        assert!(ui.mouse_down(bar_x + 38, bar_y + 20));
        assert_eq!(ui.desktop.dragging(), None);
        ui.mouse_up();
        assert_eq!(ui.shortcut.page, 3);
        assert!(ui.mouse_down(bar_x + 8, bar_y + 20));
        ui.mouse_move(bar_x + 20, bar_y + 60);
        ui.mouse_up();
        assert_eq!(ui.shortcut.page, 3);
        for _ in 0..4 {
            assert!(ui.mouse_down(bar_x + 8, bar_y + 20));
            ui.mouse_up();
        }
        assert_eq!(ui.shortcut.page, 9);
        let list = ui.frame(&world, &[], &[], None, &WorldViews::default());
        assert_eq!(digits(&list), [(bar_x + 17) as f32, (bar_x + 21) as f32]);
        assert!(ui.mouse_down(bar_x + 38, bar_y + 20));
        ui.mouse_up();
        assert_eq!(ui.shortcut.page, 0);
        assert!(ui.hook_key(WM_SYSKEYDOWN, VK_F1 + 9, 0x2000 << 16, &world));
        assert_eq!(ui.shortcut.page, 9);
        ui.rotate_shortcut();
        let window = ui.desktop.window(WindowId::SHORTCUT).unwrap().rect;
        assert!(ui.mouse_down(window.x + 20, window.y + 8));
        ui.mouse_up();
        assert_eq!(ui.shortcut.page, 0);
        assert!(ui.mouse_down(window.x + 20, window.y + 38));
        ui.mouse_up();
        assert_eq!(ui.shortcut.page, 9);
    }

    #[test]
    fn the_extend_commands_of_the_joypad_hold_the_left_or_right_half_of_the_pad() {
        let mut ui = game_ui((1558, 933));
        ui.shortcut.set_pad_hold_left(true);
        assert_eq!(ui.shortcut.pad_hold, 1);
        ui.shortcut.set_pad_hold_right(true);
        assert_eq!(ui.shortcut.pad_hold, 2);
        ui.shortcut.set_pad_hold_right(false);
        assert_eq!(ui.shortcut.pad_hold, 0);
    }

    #[test]
    fn releasing_on_the_rotate_button_turns_the_bar_around_its_bottom_right_corner() {
        let mut ui = game_ui((1558, 933));
        assert!(ui.mouse_down(1478 + 20, 168 + 495));
        assert_eq!(ui.desktop.dragging(), None);
        ui.mouse_move(1478 + 22, 168 + 496);
        ui.mouse_up();
        assert!(!ui.shortcut.vertical);
        let window = ui.desktop.window(WindowId::SHORTCUT).unwrap();
        assert_eq!(window.rect, Rect::new(1020, 626, 504, 46));
        assert_eq!(window.grip, Rect::new(0, 0, 12, 46));
        let list = ui.frame(&world(), &[], &[], None, &WorldViews::default());
        let back = with_texture(&list.quads, killer_ui::shortcut::SHORTCUT_BACK);
        assert_eq!((back[0].x, back[0].y, back[0].w, back[0].h), (1032.0, 626.0, 492.0, 46.0));
        assert!(with_texture(&list.quads, SHORTCUT_BACKV).is_empty());
        assert!(ui.mouse_down(1020 + 495, 626 + 20));
        ui.mouse_move(200, 200);
        ui.mouse_up();
        assert!(!ui.shortcut.vertical);
        assert!(ui.mouse_down(1020 + 495, 626 + 20));
        ui.mouse_up();
        assert!(ui.shortcut.vertical);
        assert_eq!(ui.desktop.window(WindowId::SHORTCUT).unwrap().rect, Rect::new(1478, 168, 46, 504));
        assert_eq!(ui.desktop.window(WindowId::SHORTCUT).unwrap().grip, Rect::new(0, 0, 46, 12));
        assert!(ui.mouse_down(1478 + 5, 168 + 5));
        ui.mouse_move(1478 + 5 - 1300, 168 + 5);
        ui.mouse_up();
        assert_eq!(ui.desktop.window(WindowId::SHORTCUT).unwrap().rect, Rect::new(178, 168, 46, 504));
        ui.rotate_shortcut();
        assert_eq!(ui.desktop.window(WindowId::SHORTCUT).unwrap().rect, Rect::new(0, 626, 504, 46));
    }

    #[test]
    fn a_held_button_shows_its_pushed_face_only_while_the_pointer_stays_on_it() {
        let mut ui = game_ui((1558, 933));
        let world = world();
        ui.frame(&world, &[], &[], None, &WorldViews::default());
        ui.mouse_move(1478 + 20, 168 + 495);
        assert!(ui.mouse_down(1478 + 20, 168 + 495));
        let held = ui.frame(&world, &[], &[], None, &WorldViews::default());
        assert_eq!(with_texture(&held.quads, "L2UI_CH3.ShortcutWnd.shortcut_rotate_down").len(), 1);
        assert!(with_texture(&held.quads, killer_ui::shortcut::SHORTCUT_ROTATE).is_empty());
        ui.mouse_move(1478 + 20, 168 + 300);
        let away = ui.frame(&world, &[], &[], None, &WorldViews::default());
        assert_eq!(with_texture(&away.quads, killer_ui::shortcut::SHORTCUT_ROTATE).len(), 1);
        ui.mouse_up();
        let released = ui.frame(&world, &[], &[], None, &WorldViews::default());
        assert!(with_texture(&released.quads, "L2UI_CH3.ShortcutWnd.shortcut_rotate_down").is_empty());
    }

    #[test]
    fn a_hovered_button_shows_its_over_face_and_goes_back_to_the_normal_one_when_the_pointer_leaves() {
        let mut ui = game_ui((1558, 933));
        let world = world();
        ui.frame(&world, &[], &[], None, &WorldViews::default());
        ui.mouse_move(1478 + 20, 168 + 495);
        let over = ui.frame(&world, &[], &[], None, &WorldViews::default());
        assert_eq!(with_texture(&over.quads, "L2UI_CH3.ShortcutWnd.shortcut_rotate_over").len(), 1);
        assert!(with_texture(&over.quads, killer_ui::shortcut::SHORTCUT_ROTATE).is_empty());
        ui.mouse_move(1478 + 20, 168 + 300);
        let away = ui.frame(&world, &[], &[], None, &WorldViews::default());
        assert_eq!(with_texture(&away.quads, killer_ui::shortcut::SHORTCUT_ROTATE).len(), 1);
    }

    #[test]
    fn clicking_the_chat_focuses_it_and_clicking_the_ground_releases_it() {
        let mut ui = game_ui((1558, 933));
        assert!(!ui.chat_focused());
        assert!(ui.mouse_down(100, 900));
        assert!(ui.chat_focused());
        ui.mouse_up();
        assert!(ui.mouse_down(5, 40));
        assert!(!ui.chat_focused());
        ui.mouse_up();
        assert!(ui.mouse_down(100, 900));
        assert!(ui.chat_focused());
        ui.mouse_up();
        assert!(!ui.mouse_down(800, 400));
        assert!(!ui.chat_focused());
        assert!(ui.mouse_down(151 + 10, 746 + 141 + 5));
        assert_eq!(ui.chat.active_tab, 2);
        ui.mouse_up();
        assert!(ui.mouse_down(30, 746 + 141 + 5));
        assert_eq!(ui.chat.active_tab, 0);
    }

    #[test]
    fn resizing_rearranges_the_windows() {
        let mut ui = game_ui((1558, 933));
        ui.resized((3116, 933));
        assert_eq!(ui.desktop.size, (3116, 933));
        assert_eq!(ui.desktop.window(WindowId::CHAT).unwrap().rect, Rect::new(0, 746, CHAT_WIDTH, 187));
        ui.mouse_down(5, 40);
        ui.mouse_move(705, 40);
        ui.mouse_up();
        ui.resized((1558, 933));
        assert_eq!(ui.desktop.window(WindowId::STATUS).unwrap().rect.x, 330);
    }

    #[test]
    fn enter_focuses_the_chat_types_and_sends() {
        let mut ui = game_ui((1558, 933));
        assert_eq!(ui.key(InputKey::Char('w')), KeyAction::Ignored);
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Consumed);
        assert!(ui.chat_focused());
        assert_eq!(ui.key(InputKey::Char('o')), KeyAction::Consumed);
        assert_eq!(ui.key(InputKey::Char('i')), KeyAction::Consumed);
        assert_eq!(ui.chat.input.text, "oi");
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Say("oi".into()));
        assert!(ui.chat_focused());
        assert_eq!(ui.chat.input.text, "");
        assert_eq!(ui.key(InputKey::Enter), KeyAction::Consumed);
        assert!(!ui.chat_focused());
        ui.key(InputKey::Enter);
        ui.key(InputKey::Char('x'));
        assert_eq!(ui.key(InputKey::Escape), KeyAction::Consumed);
        assert!(!ui.chat_focused());
        assert_eq!(ui.chat.input.text, "");
    }

    #[test]
    fn winit_keys_map_to_input_keys() {
        assert_eq!(input_keys(&Key::Named(NamedKey::Enter)), [InputKey::Enter]);
        assert_eq!(input_keys(&Key::Named(NamedKey::Backspace)), [InputKey::Backspace]);
        assert_eq!(input_keys(&Key::Named(NamedKey::ArrowLeft)), [InputKey::Left]);
        assert_eq!(input_keys(&Key::Named(NamedKey::ArrowRight)), [InputKey::Right]);
        assert_eq!(input_keys(&Key::Named(NamedKey::Escape)), [InputKey::Escape]);
        assert_eq!(input_keys(&Key::Named(NamedKey::Space)), [InputKey::Char(' ')]);
        assert_eq!(input_keys(&Key::Character("é".into())), [InputKey::Char('é')]);
        assert_eq!(input_keys(&Key::Character("\n".into())), [InputKey::Enter]);
        assert!(input_keys(&Key::Named(NamedKey::F1)).is_empty());
    }

    #[test]
    fn say_preserves_plain_text_and_rejects_missing_table_rows() {
        let tables=tables();
        let plain=l2_net::game::Say2::text(1,0,"Wender","Olá, clã!");
        assert_eq!(say_line(&tables,&plain),Some(ChatLine::new(0,"Wender","Olá, clã!")));
        let missing=l2_net::game::Say2 {object_id:1,kind:11,body:l2_net::game::Say2Body::Tables {sys_string_id:-1,sys_msg_id:1174}};
        assert!(say_line(&tables,&missing).is_none());
    }

    #[test]
    #[ignore]
    fn say_resolves_every_original_string_and_system_message() {
        let root=std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client=killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let tables=client.tables().unwrap();
        let resolve=|sender:u32,message:u32|say_line(&tables,&l2_net::game::Say2 {object_id:1,kind:11,body:l2_net::game::Say2Body::Tables {sys_string_id:sender as i32,sys_msg_id:message as i32}}).unwrap();
        assert_eq!(resolve(801,1174),ChatLine::new(11,tables.sysstring[&801].clone(),tables.systemmsg[&1174].text.clone()));
        for (&id,text) in &tables.sysstring {assert_eq!(resolve(id,1174).sender,*text);}
        for (&id,message) in &tables.systemmsg {assert_eq!(resolve(801,id),ChatLine::new(11,tables.sysstring[&801].clone(),message.text.clone()));}
    }

    #[test]
    fn system_messages_request_the_second_sound_as_voice_unless_it_is_empty_or_none() {
        let mut tables = tables();
        for (id, voice) in [(1, "SystemMsg_e.1063"), (2, ""), (3, "None")] {
            tables.systemmsg.insert(id, SystemMessageRecord { id, present: 1, text: "x".into(), group: 0, color: [0; 3], alpha: 0xFF, sound: String::new(), second_sound: voice.into() });
        }
        let message = |id| SystemMessage { id, args: Vec::new() };
        assert_eq!(system_voice(&tables, &message(1)).as_deref(), Some("SystemMsg_e.1063"));
        assert_eq!(system_voice(&tables, &message(2)), None);
        assert_eq!(system_voice(&tables, &message(3)), None);
        assert_eq!(system_voice(&tables, &message(99)), None);
        assert_eq!(system_voice(&tables, &message(-1)), None);
    }

    #[test]
    fn etc_status_values_become_the_fixed_skill_icons_in_the_engine_order() {
        assert!(etc_icons([0; 5]).is_empty());
        assert_eq!(etc_icons([3, 0, 0, 1, 0]), [(4271, 3), (4268, 1)]);
        assert_eq!(etc_icons([2, 5, 7, 9, 11]), [(4271, 2), (4270, 5), (4269, 1), (4268, 1), (4267, 1)]);
        assert_eq!(etc_icons([-1, 4, 0, 0, -3]), [(4270, 4)]);
    }

    #[test]
    fn system_messages_play_the_table_sound_unless_it_is_empty_or_none() {
        let mut tables = tables();
        for (id, sound) in [(1, "ItemSound3.sys_impossible"), (2, ""), (3, "None")] {
            tables.systemmsg.insert(id, SystemMessageRecord { id, present: 1, text: "x".into(), group: 0, color: [0; 3], alpha: 0xFF, sound: sound.into(), second_sound: String::new() });
        }
        let message = |id| SystemMessage { id, args: Vec::new() };
        assert_eq!(system_sound(&tables, &message(1)).as_deref(), Some("ItemSound3.sys_impossible"));
        assert_eq!(system_sound(&tables, &message(2)), None);
        assert_eq!(system_sound(&tables, &message(3)), None);
        assert_eq!(system_sound(&tables, &message(99)), None);
        assert_eq!(system_sound(&tables, &message(-1)), None);
    }

    #[test]
    fn system_messages_take_the_dat_text_with_the_arguments_in_order() {
        let mut tables = tables();
        tables.systemmsg.insert(
            30,
            SystemMessageRecord {
                id: 30,
                present: 1,
                text: "$s1 has obtained $s2 $s3 using $s5 from $s4.".into(),
                group: 0,
                color: [0; 3],
                alpha: 0xFF,
                sound: String::new(),
                second_sound: String::new(),
            },
        );
        tables.systemmsg.insert(
            7,
            SystemMessageRecord {
                id: 7,
                present: 1,
                text: "Welcome.".into(),
                group: 2,
                color: [0x79, 0x9b, 0xb0],
                alpha: 0xFF,
                sound: String::new(),
                second_sound: String::new(),
            },
        );
        tables.npcname.insert(1, NpcName { id: 1, name: "Gremlin".into(), title: String::new(), color: [0; 3] });
        tables.itemname.insert(57, l2_dat::itemname::ItemName {
            id: 57,
            name: "Adena".into(),
            add_name: String::new(),
            description: String::new(),
            popup: -1,
        });
        tables.skillname.insert((3, 2), l2_dat::skillname::SkillName {
            id: 3,
            level: 2,
            name: "Power Strike".into(),
            description: String::new(),
            desc_add1: String::new(),
            desc_add2: String::new(),
        });
        let message = SystemMessage {
            id: 30,
            args: vec![
                SystemMessageArg::Text("Luzal".into()),
                SystemMessageArg::Number(42),
                SystemMessageArg::ItemName(57),
                SystemMessageArg::NpcName(NPC_NAME_BASE + 1),
                SystemMessageArg::SkillName { id: 3, level: 2 },
            ],
        };
        let line = system_line(&tables, &message).unwrap();
        assert_eq!(line.kind, killer_world::chat::CHAT_SYSTEM);
        assert_eq!(line.sender, "");
        assert_eq!((line.group, line.color), (0, Some(0xFF000000)));
        assert_eq!(line.text, "Luzal has obtained 42 Adena using Power Strike from Gremlin.");
        let welcome = system_line(&tables, &SystemMessage { id: 7, args: vec![] }).unwrap();
        assert_eq!((welcome.text.as_str(), welcome.group, welcome.color), ("Welcome.", 2, Some(0xFFB09B79)));
        let unknown = system_line(&tables, &SystemMessage { id: 99, args: vec![] }).unwrap();
        assert_eq!(unknown.text, "\u{C54C} \u{C218} \u{C5C6}\u{B294} System Message format : 99");
        assert_eq!(unknown.color, Some(0xFFDCDCDC));
        assert!(system_line(&tables, &SystemMessage { id: 3000, args: vec![] }).is_some_and(|line| line.text.ends_with("3000")));
    }

    #[test]
    fn npc_say_follows_npcsay_0x10064cd0_with_the_npc_name_the_channel_colors_and_the_announcement_name() {
        let mut tables = tables();
        tables.npcname.insert(7, NpcName { id: 7, name: "Gremlin".into(), title: String::new(), color: [0; 3] });
        tables.sysstring.insert(ANNOUNCEMENT_SYSSTRING, "Announcements".into());
        let say = |kind: i32, text: &str| l2_net::game::NpcSay { object_id: 1, kind, npc_id: 7, text: text.into() };
        let line = npc_say_line(&tables, &say(1, "oi")).unwrap();
        assert_eq!((line.kind, line.sender.as_str(), line.color), (1, "Gremlin", Some(0xFFFF7200)));
        let line = npc_say_line(&tables, &say(10, "aviso")).unwrap();
        assert_eq!((line.kind, line.sender.as_str(), line.color), (7, "Announcements", Some(0xFF80FFFF)));
        assert_eq!(npc_say_line(&tables, &say(18, "x")).unwrap().color, Some(0xFF7B7DF2));
        let line = npc_say_line(&tables, &say(11, "x")).unwrap();
        assert_eq!((line.kind, line.color), (0, Some(0xFFDCDCDC)));
        assert!(npc_say_line(&tables, &say(1, "")).is_none());
    }

    #[test]
    fn say2_announcement_takes_its_name_from_sysstring_328() {
        let mut tables = tables();
        tables.sysstring.insert(ANNOUNCEMENT_SYSSTRING, "Announcements".into());
        let said = l2_net::game::Say2::text(0, 10, "", "texto");
        assert_eq!(say_line(&tables, &said).unwrap().sender, "Announcements");
    }

    #[test]
    fn system_message_parameters_follow_end_system_message_param() {
        let params = |list: &[Option<&str>]| list.iter().map(|p| p.map(str::to_string)).collect::<Vec<_>>();
        assert_eq!(format_system_message("$c1 hits $c2 for $s3", &params(&[Some("A"), Some("B"), Some("5")])), "A hits B for 5");
        assert_eq!(format_system_message("x $s2 y $s1 z", &params(&[Some("$s2"), Some("b")])), "x b y $s2 z");
        assert_eq!(format_system_message("a $s3 b", &params(&[Some("1")])), "a  b");
        assert_eq!(format_system_message("a $s1 b", &params(&[None])), "a  b");
        assert_eq!(format_system_message("100% sure", &[]), "100% sure");
        assert_eq!(format_system_message("$n1!", &params(&[Some("q")])), "q!");
        let eun = |name: &str| format_system_message("$s1%\u{C740}", &params(&[Some(name)]));
        assert_eq!(eun("\u{AC00}"), "\u{AC00}\u{B294}");
        assert_eq!(eun("\u{AC01}"), "\u{AC01}\u{C740}");
        assert_eq!(eun("Bob"), "Bob\u{B294}");
        assert_eq!(format_system_message("\u{C774}\u{C744}", &[]), "\u{C774}\u{C744}");
        assert_eq!(format_system_message("%\u{C774}", &[]), "\u{C774}");
        assert_eq!(format_system_message("$s1%\u{C744}", &params(&[None])), "\u{C744}");
    }

    #[test]
    fn system_message_object_parameters_are_blank_when_the_table_has_no_row() {
        let mut tables = tables();
        tables.castlename.insert(3, l2_dat::castlename::Castle { index: 2, present: 1, id: 3, name: "Aden".into(), location: String::new(), description: String::new() });
        tables.systemmsg.insert(500, SystemMessageRecord { id: 500, present: 1, text: "[$s1|$s2|$s3|$s4]".into(), group: 0, color: [0; 3], alpha: 255, sound: String::new(), second_sound: String::new() });
        let message = SystemMessage {
            id: 500,
            args: vec![
                SystemMessageArg::NpcName(7),
                SystemMessageArg::ItemName(9_999_999),
                SystemMessageArg::CastleName(3),
                SystemMessageArg::Unknown { kind: 77 },
            ],
        };
        assert_eq!(system_line(&tables, &message).unwrap().text, "[||Aden|]");
    }

    #[test]
    fn clicking_the_quest_mark_button_through_the_real_mouse_path_opens_the_quest_tab() {
        let mut ui = game_ui((1558, 933));
        let world = World::default();
        ui.world_event(&killer_world::WorldEvent::ShowQuestMark(l2_net::game::ShowQuestMark { value: 348 }), &world);
        let rect = ui.desktop.window(WindowId::QUEST_BUTTON).unwrap().rect;
        assert!(ui.desktop.window(WindowId::QUEST_BUTTON).unwrap().visible);
        let (x, y) = (rect.x + rect.w / 2, rect.y + rect.h / 2);
        assert_eq!(ui.window_under(x, y).map(|(id, _, _)| id), Some(WindowId::QUEST_BUTTON));
        ui.mouse_move(x, y);
        assert!(ui.mouse_down_at(x, y, Instant::now(), &world));
        ui.mouse_up_in(&world);
        assert!(ui.desktop.window(WindowId::MAIN).unwrap().visible);
        assert_eq!(ui.keys.main.tab, killer_ui::keys::MainTab::Quests);
        assert_eq!(ui.keys.quest.marked, Some(348));
    }

    #[test]
    fn mail_arrived_1006a250_shows_the_mail_button_and_its_click_opens_the_mail_tab_of_the_board() {
        let mut ui = game_ui((1558, 933));
        let world = World::default();
        ui.world_event(&killer_world::WorldEvent::MailArrived, &world);
        let rect = ui.desktop.window(WindowId::MAIL_BUTTON).unwrap().rect;
        assert!(ui.desktop.window(WindowId::MAIL_BUTTON).unwrap().visible);
        assert!(ui.tutorial.mail_button.effect);
        assert_eq!(rect.x, killer_ui::tutorial::MAIL_BUTTON_X);
        let (x, y) = (rect.x + rect.w / 2, rect.y + rect.h / 2);
        assert_eq!(ui.window_under(x, y).map(|(id, _, _)| id), Some(WindowId::MAIL_BUTTON));
        ui.mouse_move(x, y);
        assert!(ui.mouse_down_at(x, y, Instant::now(), &world));
        ui.mouse_up_in(&world);
        assert!(!ui.desktop.window(WindowId::MAIL_BUTTON).unwrap().visible);
        assert!(ui.desktop.window(WindowId::BOARD).unwrap().visible);
        assert_eq!(ui.take_board_command(), Some(KeyAction::Bypass("_maillist_0_1_0_".into())));
    }

    #[test]
    fn clicking_a_quest_root_through_the_real_mouse_path_expands_it() {
        use l2_dat::QuestName;
        let mut ui = game_ui((1558, 933));
        let mut world = World::default();
        let mut tables = DatTables::default();
        for state in 1..=3 {
            tables.questname.insert((348, state), QuestName {
                index: 0,
                valid: 1, id: 348, state, title: "An Arrogant Search".into(), sub_name: format!("step {state}"), description: "d".into(),
                item_ids: vec![], item_nums: vec![], target_loc: [0.0; 3], level: 60, quest_level: 75, quest_type: 3,
                target_name: String::new(), minimap_only: 0, journal_disp: 1, unknown: 1, start_npc_id: 0, start_npc_loc: [0.0; 3],
                requirement: String::new(), intro: String::new(),
            });
        }
        world.windows.quests = vec![l2_net::game::QuestState { quest_id: 348, cond: 2 }];
        ui.keys.open_marked_quest(0, &mut ui.desktop);
        ui.world_event(&killer_world::WorldEvent::QuestList(l2_net::game::QuestList { quests: world.windows.quests.clone(), extras: Vec::new() }), &world);
        ui.sync_windows(&world, Some(&tables), Instant::now());
        assert_eq!(ui.keys.quest.quests.len(), 1);
        let rect = ui.desktop.window(WindowId::MAIN).unwrap().rect;
        let (x, y) = (rect.x + killer_ui::mainwnd::PAGE_RECT.x + killer_ui::quest::TREE_RECT.x + 3 + 7, rect.y + killer_ui::mainwnd::PAGE_RECT.y + killer_ui::quest::TREE_RECT.y + 5 + 7);
        ui.mouse_move(x, y);
        assert!(ui.mouse_down_at(x, y, Instant::now(), &world));
        ui.mouse_up_in(&world);
        ui.sync_windows(&world, Some(&tables), Instant::now());
        assert!(ui.keys.quest.quests[0].expanded);
    }
}
