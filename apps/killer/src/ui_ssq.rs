use killer_assets::client::DatTables;
use killer_ui::ssq::{Festival, FestivalGroup, Labels, Prediction, Seal, SsqView, Summary};
use killer_ui::window::{Desktop, WindowId};
use l2_net::game::packets::{SsqPage, SsqStatus};

pub fn labels(tables: &DatTables) -> Labels {
    let mut labels = Labels::from_sysstrings(|id| tables.sysstring.get(&id).cloned());
    labels.messages = tables.systemmsg.iter().map(|(&id, message)| (id as i32, message.text.clone())).collect();
    labels
}

pub fn receive(view: &mut SsqView, status: &SsqStatus, desktop: &mut Desktop) {
    match &status.body {
        SsqPage::Summary(summary) => {
            view.receive_summary(status.period, Summary {
                cycle: summary.cycle, period_message: summary.period_message, until_message: summary.until_message,
                player_cabal: summary.player_cabal, player_seal: summary.player_seal,
                stones: summary.stones, adena: summary.adena,
                dusk_stone_score: summary.dusk_stone_score, dusk_festival_score: summary.dusk_festival_score,
                dusk_total_score: summary.dusk_total_score, dusk_percent: summary.dusk_percent,
                dawn_stone_score: summary.dawn_stone_score, dawn_festival_score: summary.dawn_festival_score,
                dawn_total_score: summary.dawn_total_score, dawn_percent: summary.dawn_percent,
            });
            if let Some(window) = desktop.window_mut(WindowId::SSQ) { window.visible = true; }
            desktop.bring_to_front(WindowId::SSQ);
        }
        SsqPage::Predictions { winning_cabal, seals } => view.receive_predictions(status.period, *winning_cabal,
            seals.iter().map(|seal| Prediction { id: seal.id, owner: seal.owner, message: seal.message }).collect()),
        SsqPage::Festivals(groups) => view.receive_festivals(status.period, groups.iter().map(|group| FestivalGroup {
            flag: group.flag, festivals: group.festivals.iter().map(|festival| Festival {
                id: festival.id, level_score: festival.level_score, dusk_score: festival.dusk_score,
                dusk_members: festival.dusk_members.clone(), dawn_score: festival.dawn_score, dawn_members: festival.dawn_members.clone(),
            }).collect(),
        }).collect()),
        SsqPage::Seals { retain_limit, claim_limit, seals } => view.receive_seals(status.period, *retain_limit, *claim_limit,
            seals.iter().map(|seal| Seal { id: seal.id, owner: seal.owner, dawn_percent: seal.dawn_percent, dusk_percent: seal.dusk_percent }).collect()),
        SsqPage::Unknown => {}
    }
}
