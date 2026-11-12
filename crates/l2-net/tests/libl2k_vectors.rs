use l2_net::game::GameCrypt;
use l2_net::login::LoginCipher;

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

fn vectors(name: &str) -> Vec<(Vec<u8>, Vec<u8>)> {
    let text = std::fs::read_to_string(format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap();
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut parts = l.split_whitespace();
            (unhex(parts.next().unwrap()), unhex(parts.next().unwrap()))
        })
        .collect()
}

#[test]
fn login_seal_matches_libl2k() {
    let cipher = LoginCipher::c4();
    for (payload, sealed) in vectors("login_seal.hex") {
        assert_eq!(cipher.seal(&payload), sealed);
        let opened = cipher.open(&sealed).unwrap();
        assert_eq!(&opened[..payload.len()], &payload[..]);
        assert_eq!(opened.len(), sealed.len() - 8);
    }
}

#[test]
fn game_crypt_matches_libl2k_across_two_packets() {
    let text =
        std::fs::read_to_string(format!("{}/tests/fixtures/game_crypt.hex", env!("CARGO_MANIFEST_DIR"))).unwrap();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let mut parts = line.split_whitespace();
        let key_packet = unhex(parts.next().unwrap());
        let plain = unhex(parts.next().unwrap());
        let expected = unhex(parts.next().unwrap());
        let mut tx = GameCrypt::from_key_packet(&key_packet).unwrap();
        let mut buf = plain.clone();
        tx.encrypt(&mut buf[..261]);
        tx.encrypt(&mut buf[261..]);
        assert_eq!(buf, expected);
        let mut rx = GameCrypt::from_key_packet(&key_packet).unwrap();
        rx.decrypt(&mut buf[..261]);
        rx.decrypt(&mut buf[261..]);
        assert_eq!(buf, plain);
    }
}
