//! Konfiguration des Applets.
//!
//! Persistente Konfiguration liegt als TOML-Datei unter
//! `~/.config/cosmic-applet-homeassistant/config.toml` (URL, Token,
//! Entity-Liste). Fehlt das File (oder einzelne Felder), werden die
//! Umgebungsvariablen `HOME_ASSISTANT_URL`/`HA_URL` bzw.
//! `HOME_ASSISTANT_TOKEN`/`HA_TOKEN` als Fallback herangezogen.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Eine konfigurierte Entity: HA-Entity-ID plus Anzeigename.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityConfig {
    pub entity_id: String,
    pub name: String,
}

/// Gesamte Applet-Konfiguration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub instance_url: String,
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub entities: Vec<EntityConfig>,
    /// Wie viele der Entities (in Reihenfolge) im Panel neben dem Logo
    /// erscheinen. `None` = alle (inkl. Popup-Liste).
    #[serde(default)]
    pub panel_count: Option<usize>,
    /// Aktualisierungsintervall in Minuten (Minimum 1).
    #[serde(default = "default_refresh_minutes")]
    pub refresh_minutes: u32,
}

fn default_refresh_minutes() -> u32 {
    1
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            instance_url: String::new(),
            access_token: String::new(),
            entities: Vec::new(),
            panel_count: None,
            refresh_minutes: default_refresh_minutes(),
        }
    }
}

impl AppConfig {
    /// Die Entities, die im Panel neben dem Logo erscheinen: die ersten
    /// `panel_count` (geklemmt auf die Länge); `None` = alle.
    #[must_use]
    pub fn panel_entities(&self) -> &[EntityConfig] {
        let n = match self.panel_count {
            Some(n) => n.min(self.entities.len()),
            None => self.entities.len(),
        };
        &self.entities[..n]
    }
}

/// Verschiebt eine Entity von Index `from` nach Index `to`
/// (Out-of-Bounds wird ignoriert — UI-Buttons können nicht "aus dem
/// Listenrand ziehen").
pub fn move_entity(entities: &mut Vec<EntityConfig>, from: usize, to: usize) {
    if from >= entities.len() || to >= entities.len() || from == to {
        return;
    }
    let moved = entities.remove(from);
    entities.insert(to, moved);
}

/// Entfernt die Entity an `i` und rückt die Panel-Grenze mit: Wird eine
/// Entity über der Grenze entfernt, verschiebt sich die Grenze nach unten,
/// damit die restlichen Entities panel/zugehörig bleiben. Die Grenze wird
/// immer auf die Listenlänge geklemmt.
pub fn remove_row(rows: &mut Vec<EntityConfig>, boundary: &mut usize, i: usize) {
    if i >= rows.len() {
        return;
    }
    rows.remove(i);
    if i < *boundary {
        *boundary -= 1;
    }
    *boundary = (*boundary).min(rows.len());
}

/// Verschiebt eine Entity unter Beachtung der Panel-Grenze: Kreuzt der
/// Weg die Linie, wechselt GENAU diese Entity die Gruppe — die Linie folgt,
/// sodass alle übrigen Entities in ihren Gruppen bleiben (internes
/// Verschieben lässt die Linie zwischen denselben Entities stehen).
pub fn move_entity_across(
    entities: &mut Vec<EntityConfig>,
    boundary: &mut usize,
    from: usize,
    to: usize,
) {
    if from >= entities.len() || to >= entities.len() || from == to {
        return;
    }
    let was_in_panel = from < *boundary;
    let moved = entities.remove(from);
    if was_in_panel {
        *boundary = boundary.saturating_sub(1);
    }
    entities.insert(to, moved);
    if to < *boundary {
        *boundary += 1;
    }
}

/// Aktualisierungsintervall als Duration; immer mindestens 1 Minute.
#[must_use]
pub fn refresh_interval(cfg: &AppConfig) -> std::time::Duration {
    std::time::Duration::from_secs(u64::from(cfg.refresh_minutes.max(1)) * 60)
}

/// Pfad der persistenten Konfigurationsdatei.
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("cosmic-applet-homeassistant").join("config.toml"))
}

/// Lädt die Konfiguration vom Standardpfad; fehlende Felder werden
/// über die env-Fallbacks gefüllt.
pub fn load() -> AppConfig {
    match config_path() {
        Some(path) => load_from(&path),
        None => AppConfig::default(),
    }
    .with_env_fallback()
}

/// Lädt die Konfiguration aus einer Datei. Fehlt die Datei oder ist sie
/// ungültig, wird der Default zurückgegeben (keine Panik). Ohne env-Fallback.
pub fn load_from(path: &Path) -> AppConfig {
    let mut cfg = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str::<AppConfig>(&s).ok())
        .unwrap_or_default();
    if cfg.refresh_minutes == 0 {
        cfg.refresh_minutes = default_refresh_minutes();
    }
    cfg
}

impl AppConfig {
    /// Füllt leere Felder aus den env-Fallbacks (`HOME_ASSISTANT_URL`/
    /// `HA_URL`, `HOME_ASSISTANT_TOKEN`/`HA_TOKEN`).
    fn with_env_fallback(mut self) -> Self {
        if self.instance_url.is_empty() {
            self.instance_url = instance_url().unwrap_or_default();
        }
        if self.access_token.is_empty() {
            self.access_token = access_token().unwrap_or_default();
        }
        self
    }
}

/// Speichert die Konfiguration am Standardpfad (Rechte `0600` unter Unix).
pub fn save(cfg: &AppConfig) -> std::io::Result<()> {
    let path = config_path().ok_or_else(|| {
        std::io::Error::other("kein Config-Verzeichnis ermittelbar")
    })?;
    save_to(&path, cfg)
}

/// Testbare Variante: speichert an einen expliziten Pfad.
pub fn save_to(path: &Path, cfg: &AppConfig) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = toml::to_string(cfg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, body)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Basis-URL der Home-Assistant-Instanz (z. B. `http://homeassistant.local:8123`).
/// Ohne trailing slash.
pub fn instance_url() -> Option<String> {
    clean(
        std::env::var("HOME_ASSISTANT_URL")
            .or_else(|_| std::env::var("HA_URL"))
            .ok(),
    )
    .map(|url| url.trim_end_matches('/').to_string())
}

/// Long-Lived Access Token (Home Assistant → Profil → Sicherheit).
pub fn access_token() -> Option<String> {
    clean(
        std::env::var("HOME_ASSISTANT_TOKEN")
            .or_else(|_| std::env::var("HA_TOKEN"))
            .ok(),
    )
}

/// Cache-Pfad für zuletzt bekannte Entity-Werte (Startup ohne Wartezeit).
pub fn default_cache_path() -> Option<std::path::PathBuf> {
    dirs::cache_dir().map(|d| d.join("cosmic-applet-homeassistant").join("cache.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_to_sets_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("config.toml");
        save_to(&path, &AppConfig::default()).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
}
