use killer_assets::client::Client;
use std::path::PathBuf;

macro_rules! probe {
    ($client:expr, $report:expr, $name:expr, $read:expr) => {{
        let outcome = match $client.read_system_file($name) {
            Ok(body) => match $read(&body) {
                Ok(_) => "ok".to_string(),
                Err(error) => format!("FALHA: {error}"),
            },
            Err(error) => format!("ausente: {error}"),
        };
        $report.push(format!("{:28} {outcome}", $name));
    }};
}

#[test]
#[ignore]
fn every_dat_table_reads_on_the_client_in_client_dir() {
    let root = PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR"));
    let mut client = Client::open(&root).unwrap();
    let chronicle = client.chronicle();
    eprintln!("crônica: {}", chronicle.name());
    let mut report = Vec::new();
    probe!(client, report, "weapongrp.dat", |body: &[u8]| l2_dat::weapongrp::read_as(body, chronicle));
    probe!(client, report, "armorgrp.dat", l2_dat::armorgrp::read);
    probe!(client, report, "etcitemgrp.dat", l2_dat::etcitemgrp::read);
    probe!(client, report, "npcgrp.dat", |body: &[u8]| l2_dat::npcgrp::read_as(body, chronicle));
    probe!(client, report, "itemname-e.dat", |body: &[u8]| l2_dat::itemname::read_as(body, chronicle));
    probe!(client, report, "skillname-e.dat", l2_dat::skillname::read);
    probe!(client, report, "npcname-e.dat", l2_dat::npcname::read);
    probe!(client, report, "chargrp.dat", l2_dat::chargrp::read);
    probe!(client, report, "hairgrp.dat", l2_dat::hairgrp::read);
    probe!(client, report, "helmetgrp.dat", l2_dat::helmetgrp::read);
    probe!(client, report, "hairaccessarygrp.dat", l2_dat::helmetgrp::read);
    probe!(client, report, "hairaccessorylocgrp.dat", l2_dat::hairaccessorylocgrp::read);
    probe!(client, report, "skillsoundgrp.dat", l2_dat::skillsoundgrp::read);
    probe!(client, report, "eula-e.dat", |body: &[u8]| l2_dat::eula::read_as(body, chronicle));
    probe!(client, report, "creditgrp-e.dat", l2_dat::creditgrp::read);
    probe!(client, report, "questname-e.dat", |body: &[u8]| l2_dat::questname::read_as(body, chronicle));
    probe!(client, report, "systemmsg-e.dat", |body: &[u8]| l2_dat::systemmsg::read_as(body, chronicle));
    probe!(client, report, "sysstring-e.dat", l2_dat::sysstring::read);
    probe!(client, report, "symbolname-e.dat", l2_dat::symbolname::read);
    probe!(client, report, "hennagrp-e.dat", l2_dat::hennagrp::read);
    probe!(client, report, "skillgrp.dat", l2_dat::skillgrp::read);
    probe!(client, report, "zonename-e.dat", |body: &[u8]| l2_dat::zonename::read_as(body, chronicle));
    probe!(client, report, "castlename-e.dat", l2_dat::castlename::read);
    probe!(client, report, "staticobject-e.dat", l2_dat::staticobject::read);
    probe!(client, report, "mobskillanimgrp.dat", l2_dat::mobskillanimgrp::read);
    probe!(client, report, "actionname-e.dat", l2_dat::actionname::read);
    probe!(client, report, "commandname-e.dat", l2_dat::commandname::read);
    probe!(client, report, "obscene-e.dat", |body: &[u8]| l2_dat::obscene::read_as(body, chronicle));
    probe!(client, report, "musicinfo.dat", l2_dat::musicinfo::read);
    probe!(client, report, "entereventgrp.dat", l2_dat::entereventgrp::read);
    probe!(client, report, "classinfo-e.dat", l2_dat::classinfo::read);
    probe!(client, report, "servername-e.dat", l2_dat::servername::read);
    probe!(client, report, "logongrp.dat", l2_dat::logongrp::read);
    probe!(client, report, "Recipe-c.dat", l2_dat::recipe::read);
    eprintln!("{}", report.join("\n"));
}
