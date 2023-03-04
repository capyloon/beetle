use super::*;

#[test]
fn test_new_api() -> Result<()> {
    let mut s = SettingEngine::default();
    s.detach_data_channels();
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;

    let api = APIBuilder::new()
        .with_setting_engine(s)
        .with_media_engine(m)
        .build();

    assert_eq!(
        api.setting_engine.detach.data_channels, true,
        "Failed to set settings engine"
    );
    assert_eq!(
        api.media_engine.audio_codecs.is_empty(),
        false,
        "Failed to set media engine"
    );

    Ok(())
}
