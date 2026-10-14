use kitty_rc_commands::{
    Command,
    appearance::BackgroundLayout,
    targets::{Assignment, Opacity, WindowTarget},
    upload::{BackgroundPng, LogoPng, Run, UploadKey, UploadSubmission},
};
use kitty_rc_endpoint::Protection;
use kitty_rc_exchange::{PrepareError, prepare, prepare_upload};
use serde_json::{Value, json};
#[test]
fn pinned_constructor_metadata_chunk_bounds_and_envelopes() {
    let background = BackgroundPng::builder()
        .match_spec("title:🦀")
        .all(true)
        .configured(true)
        .layout(BackgroundLayout::CenteredScaled)
        .build();
    assert_eq!(
        background.request().payload,
        Some(json!({"match":"title:🦀","all":true,"configured":true,"layout":"cscaled"}))
    );
    let logo = LogoPng::builder()
        .target(WindowTarget::CallingWindow)
        .position("bottom-right")
        .alpha(Opacity::new(0.3).unwrap())
        .build();
    assert_eq!(
        logo.request().payload,
        Some(json!({"match":null,"self":true,"alpha":0.3,"position":"bottom-right"}))
    );
    let run = Run::builder()
        .cmdline(vec!["echo".into(), "a🦀\0binary".into()])
        .env(vec![Assignment::Set {
            name: "X".into(),
            value: "a=b".into(),
        }])
        .allow_remote_control(true)
        .remote_control_passwords(vec!["pw permit-*".into()])
        .build();
    assert_eq!(
        run.request().payload,
        Some(
            json!({"cmdline":["echo","a🦀\0binary"],"env":["X=a=b"],"allow_remote_control":true,"remote_control_password":["pw permit-*"]})
        )
    );
    for (key, profile) in [
        (
            UploadKey::BackgroundImage,
            prepare_upload(&background, &Protection::Plain, 16384).unwrap(),
        ),
        (
            UploadKey::WindowLogo,
            prepare_upload(&UploadSubmission(logo), &Protection::Plain, 16384).unwrap(),
        ),
        (
            UploadKey::Run,
            prepare_upload(&run, &Protection::Plain, 16384).unwrap(),
        ),
    ] {
        let raw = vec![255; key.raw_chunk_bytes()];
        let first = profile.frame(&raw, true, &Protection::Plain).unwrap();
        let first: Value = serde_json::from_slice(first.plaintext()).unwrap();
        assert_eq!(first["stream"], true);
        assert_eq!(
            first["payload"]["data"].as_str().unwrap().len(),
            raw.len().div_ceil(3) * 4
        );
        assert!(
            profile
                .frame(
                    &vec![0; key.raw_chunk_bytes() + 1],
                    false,
                    &Protection::Plain
                )
                .is_err()
        );
        let eof = profile.frame(&[], false, &Protection::Plain).unwrap();
        let eof: Value = serde_json::from_slice(eof.plaintext()).unwrap();
        assert_eq!(eof["payload"]["data"], "");
        assert!(eof.get("stream").is_none());
        assert_eq!(eof["stream_id"], first["stream_id"]);
        assert_eq!(eof["async"], first["async"]);
        assert!(profile.frame(&[], true, &Protection::Plain).is_err());
    }
    assert!(matches!(
        prepare(&run, &Protection::Plain, 16384),
        Err(PrepareError::UploadEntrance)
    ));
    assert!(prepare_upload(&background, &Protection::Plain, 100).is_err());
    let empty = Run::builder().cmdline(vec![]).build();
    assert!(matches!(
        prepare_upload(&empty, &Protection::Plain, 16384),
        Err(PrepareError::UploadProfile)
    ));
}

#[test]
fn each_upload_frame_has_fresh_single_protection_and_stable_conversation_identity() {
    let secret = x25519_dalek::StaticSecret::from([64; 32]);
    let key = kitty_rc_crypto::ServerPublicKey::parse(&format!(
        "1:{}",
        base85::encode(x25519_dalek::PublicKey::from(&secret).as_bytes())
    ))
    .unwrap();
    let protection = Protection::Encrypted {
        key,
        password: "exact 🦀\0password".into(),
        key_origin: "test".into(),
    };
    let run = Run::builder().cmdline(vec!["cat".into()]).build();
    let prepared = prepare_upload(&run, &protection, 16384).unwrap();
    let mut ivs = vec![];
    let mut keys = vec![];
    let mut identities = vec![];
    for (bytes, first) in [
        (&b"a"[..], true),
        (&b"\0\xff"[..], false),
        (&b""[..], false),
    ] {
        let frame = prepared
            .frame(bytes, first, &protection)
            .unwrap()
            .protect(&protection)
            .unwrap();
        let outer: Value = serde_json::from_slice(
            &frame[kitty_rc_frame::PREFIX.len()..frame.len() - kitty_rc_frame::SUFFIX.len()],
        )
        .unwrap();
        assert!(outer.get("cmd").is_none());
        assert!(outer.get("payload").is_none());
        ivs.push(outer["iv"].clone());
        keys.push(outer["pubkey"].clone());
        let decrypted =
            kitty_rc_crypto::decrypt_envelope(&serde_json::from_value(outer).unwrap(), &secret)
                .unwrap();
        assert_eq!(decrypted["cmd"], "run");
        assert_eq!(decrypted["password"], "exact 🦀\0password");
        assert!(decrypted.get("encrypted").is_none());
        identities.push((decrypted["stream_id"].clone(), decrypted["async"].clone()));
    }
    assert!(ivs[0] != ivs[1] && ivs[1] != ivs[2] && ivs[0] != ivs[2]);
    assert!(keys[0] != keys[1] && keys[1] != keys[2]);
    assert_eq!(identities[0], identities[1]);
    assert_eq!(identities[1], identities[2]);
    let raw = kitty_rc_commands::RawFinite {
        name: "run".into(),
        payload: json!({"cmdline":["cat"],"data":""}),
        class: kitty_rc_commands::Class::LongWait,
        submission_only: false,
    };
    assert!(matches!(
        prepare(&raw, &Protection::Plain, 16384),
        Err(PrepareError::UploadEntrance)
    ));
}
