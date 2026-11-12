use crate::{DatReader, Error};

pub const CATEGORY_SPECIAL: u32 = 0;
pub const CATEGORY_BASIC: u32 = 1;
pub const CATEGORY_PARTY: u32 = 2;
pub const CATEGORY_SOCIAL: u32 = 3;
pub const CATEGORY_PET: u32 = 4;
pub const CATEGORY_SUMMON: u32 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub id: u32,
    pub kind: i32,
    pub category: u32,
    pub category2: Vec<i32>,
    pub cmd: String,
    pub icon: String,
    pub name: String,
    pub desc: String,
}

pub fn read(body: &[u8]) -> Result<Vec<Action>, Error> {
    let mut reader = DatReader::new(body);
    let count = reader.u32()?;
    (0..count).map(|_| record(&mut reader)).collect()
}

fn record(r: &mut DatReader<'_>) -> Result<Action, Error> {
    r.u32()?;
    let id = r.u32()?;
    let kind = r.i32()?;
    let category = r.u32()?;
    let count = r.count()?;
    let category2 = (0..count).map(|_| r.i32()).collect::<Result<Vec<_>, _>>()?;
    let name = r.ascf()?;
    let icon = r.ascf()?;
    let desc = r.ascf()?;
    let cmd = r.unicode()?;
    Ok(Action { id, kind, category, category2, cmd, icon, name, desc })
}

pub const COMMAND_TOKENS: [(&str, u32); 110] = [
    ("loc", 0x0),
    ("petition", 0x1),
    ("petitioncancel", 0x2),
    ("gmlist", 0x3),
    ("sit", 0x4),
    ("stand", 0x5),
    ("walk", 0x6),
    ("run", 0x7),
    ("attack", 0x8),
    ("attackforce", 0x9),
    ("attackstand", 0xa),
    ("trade", 0xb),
    ("target", 0xc),
    ("targetnext", 0xd),
    ("pickup", 0xe),
    ("assist", 0xf),
    ("partyinvite", 0x10),
    ("partyleave", 0x11),
    ("partydismiss", 0x12),
    ("vendor", 0x13),
    ("partymatching", 0x14),
    ("socialhello", 0x15),
    ("socialvictory", 0x16),
    ("socialcharge", 0x17),
    ("useskill", 0x18),
    ("useskillforce", 0x19),
    ("useskillstand", 0x1a),
    ("useshortcut", 0x1b),
    ("useshortcutforce", 0x1c),
    ("useshortcutstand", 0x1d),
    ("allyinvite", 0x1e),
    ("allyleave", 0x1f),
    ("allyoust", 0x20),
    ("allydismiss", 0x21),
    ("allycrest", 0x22),
    ("allyInfo", 0x23),
    ("friendinvite", 0x24),
    ("friendlist", 0x25),
    ("frienddel", 0x26),
    ("buy", 0x27),
    ("allywarstart", 0x28),
    ("allywarstop", 0x29),
    ("allywarsurrender", 0x2a),
    ("blocklist", 0x2b),
    ("block", 0x2c),
    ("unblock", 0x2d),
    ("socialno", 0x2e),
    ("socialyes", 0x2f),
    ("socialbow", 0x30),
    ("socialunaware", 0x31),
    ("socialwaitinga", 0x32),
    ("sociallaugh", 0x33),
    ("escape", 0x34),
    ("remaintime", 0x35),
    ("socialapplause", 0x36),
    ("socialdance", 0x37),
    ("socialsad", 0x38),
    ("sitstand", 0x39),
    ("walkrun", 0x3a),
    ("manufacture", 0x3b),
    ("mountdismount", 0x3c),
    ("mount", 0x3d),
    ("dismount", 0x3e),
    ("pethold", 0x3f),
    ("petattack", 0x40),
    ("petstop", 0x41),
    ("petpickup", 0x42),
    ("petrevert", 0x43),
    ("petspecial", 0x44),
    ("summonhold", 0x45),
    ("summonattack", 0x46),
    ("summonstop", 0x47),
    ("summonsiege", 0x48),
    ("skill4036", 0x49),
    ("skill4138", 0x4a),
    ("commend", 0x4b),
    ("wait", 0x4c),
    ("time", 0x4d),
    ("allblock", 0x4e),
    ("allunblock", 0x4f),
    ("equip", 0x50),
    ("partyinfo", 0x51),
    ("pledgewarinfo", 0x52),
    ("allywarinfo", 0x53),
    ("storefind", 0x54),
    ("leaderchange", 0x55),
    ("warattack", 0x56),
    ("warstop", 0x57),
    ("attacklist", 0x58),
    ("underattacklist", 0x59),
    ("warlist", 0x5a),
    ("allycrestdel", 0x5b),
    ("openchannel", 0x5c),
    ("closechannel", 0x5d),
    ("invitechannel", 0x5e),
    ("banchannel", 0x5f),
    ("outchannel", 0x60),
    ("channelinfo", 0x61),
    ("selfnickname", 0x62),
    ("warstatus", 0x63),
    ("pledgepenalty", 0x64),
    ("manufacture2", 0x65),
    ("chinatime", 0x66),
    ("recstart", 0x67),
    ("recstop", 0x68),
    ("recstartstop", 0x69),
    ("petmove", 0x6a),
    ("summonmove", 0x6b),
    ("unsummon", 0x6c),
    ("olympiadstat", 0x6d),
];

pub fn command_token(cmd: &str) -> Option<u32> {
    COMMAND_TOKENS.iter().find(|(name, _)| name.eq_ignore_ascii_case(cmd)).map(|(_, token)| *token)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unicode(s: &str) -> Vec<u8> {
        let mut out = (s.len() as u32 * 2).to_le_bytes().to_vec();
        out.extend(s.encode_utf16().flat_map(u16::to_le_bytes));
        out
    }

    #[test]
    fn sit_stand_is_action_zero_of_the_basic_category() {
        let mut body = 1u32.to_le_bytes().to_vec();
        for v in [1u32, 0] {
            body.extend(v.to_le_bytes());
        }
        body.extend((-1i32).to_le_bytes());
        body.extend(1u32.to_le_bytes());
        body.push(1);
        body.extend((-2i32).to_le_bytes());
        body.extend(b"\x0aSit/Stand\x00");
        body.extend(b"\x0ficon.action001\x00");
        body.extend(b"\x17Toggle Sit/Stand mode.\x00");
        body.extend(unicode("sitstand"));
        let actions = read(&body).unwrap();
        assert_eq!(actions.len(), 1);
        let sit = &actions[0];
        assert_eq!((sit.id, sit.kind, sit.category), (0, -1, CATEGORY_BASIC));
        assert_eq!(sit.category2, [-2]);
        assert_eq!(
            (sit.cmd.as_str(), sit.icon.as_str(), sit.name.as_str()),
            ("sitstand", "icon.action001", "Sit/Stand")
        );
        assert_eq!(sit.desc, "Toggle Sit/Stand mode.");
        assert!(read(&body[..body.len() - 1]).is_err());
    }
}
