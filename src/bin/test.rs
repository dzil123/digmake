use uuid::Uuid;

pub fn offline_player_uuid(text: &str) -> Uuid {
    let bytes = md5::compute(text).into();

    uuid::Builder::from_bytes(bytes)
        .set_variant(uuid::Variant::RFC4122)
        .set_version(uuid::Version::Md5)
        .build()
}

fn dbg(x: &Uuid) {
    dbg!(&x, x.get_variant(), x.get_version_num(), x.get_version());
}

fn main() {
    let x = Uuid::parse_str("b4bcabdd-6041-360c-84de-bb50c9a8b0b6").unwrap();
    dbg(&x);

    let y = offline_player_uuid("OfflinePlayer:dzil1234");
    dbg(&y);

    assert_eq!(x, y);
}
