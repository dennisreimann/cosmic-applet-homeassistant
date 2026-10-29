use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use i18n_embed::{DefaultLocalizer, LanguageLoader, Localizer};
use rust_embed::RustEmbed;
use std::sync::{LazyLock, OnceLock};

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();
    loader
        .load_fallback_language(&Localizations)
        .expect("error while loading fallback language (en)");
    loader
});

static LOCALIZATION_INITIALIZED: OnceLock<()> = OnceLock::new();

/// Übersetzter String nach `message_id` (Compile-Zeit-Validierung gegen den
/// EN-Katalog), optional mit `name = value`-Argumenten.
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        $crate::i18n::localize();
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id)
    }};
    ($message_id:literal, $($args:expr),*) => {{
        $crate::i18n::localize();
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

/// Sprachwahl: Override via `I18N_LANGUAGE` (kommagetrennte BCP-47-Tags,
/// z. B. `en` oder `de` — für deterministische Tests), sonst die
/// Desktop-Locale.
fn requested_languages() -> Vec<i18n_embed::unic_langid::LanguageIdentifier> {
    if let Ok(spec) = std::env::var("I18N_LANGUAGE") {
        let tags: Vec<_> = spec
            .split(',')
            .filter_map(|tag| tag.trim().parse().ok())
            .collect();
        if !tags.is_empty() {
            return tags;
        }
    }
    i18n_embed::DesktopLanguageRequester::requested_languages()
}

/// Lädt die gewählte Sprache einmalig in den Loader (idempotent; wird vom
/// `fl!`-Makro vor jedem Zugriff aufgerufen).
pub fn localize() {
    LOCALIZATION_INITIALIZED.get_or_init(|| {
        let localizer = DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations);
        if let Err(error) = localizer.select(&requested_languages()) {
            eprintln!("error while loading language: {error}");
        }
        // Unicode-Isolationszeichen um Platzhalter deaktivieren. Wichtig NACH
        // dem select(): set_use_isolating wirkt nur auf bereits geladene
        // Bundles (Default ist an).
        LANGUAGE_LOADER.set_use_isolating(false);
    });
}
