use cosmic_applet_homeassistant::{fl, i18n};
use i18n_embed::LanguageLoader;
use std::sync::Once;

static DE_LOCALE: Once = Once::new();

/// Deterministische Sprache für diesen Testprozess: DE.
fn force_de() {
    DE_LOCALE.call_once(|| std::env::set_var("I18N_LANGUAGE", "de"));
}

#[test]
fn german_catalog_covers_all_messages() {
    force_de();
    assert_eq!(fl!("popup-title"), "Home Assistant");
    assert_eq!(fl!("quit"), "Beenden");
    assert_eq!(fl!("settings"), "Einstellungen");
    assert_eq!(fl!("no-entities"), "Keine Entities konfiguriert");
    assert_eq!(fl!("instance-url"), "Instanz-URL");
    assert_eq!(fl!("save"), "Speichern");
}

#[test]
fn loader_respects_i18n_language_override() {
    force_de();
    // Nach dem Override und localize() muss der Loader die DE-Sprache
    // berücksichtigen (aktuelle Sprache beginnt mit "de").
    i18n::localize();
    let current = (*i18n::LANGUAGE_LOADER).current_language().to_string();
    assert!(current.starts_with("de"), "aktuelle Sprache: {current}");
}
