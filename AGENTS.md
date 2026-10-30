# AGENTS.md

Guidelines and context for AI agents working on this repository.

## Project

**cosmic-applet-homeassistant** — A native COSMIC panel applet (v1.0.0) that
displays a **configurable list of Home Assistant entities** in the panel
(state values with units, e.g. temperature sensors) plus a popup with the
full list and a separate settings window.

**Status: released.** Everything below ships: REST polling, TOML
configuration with owner-only permissions, a settings window, and the
refresh loop. Do not redesign these mechanisms — extend them.

## architecture

- **Native COSMIC applet** (`cosmic::applet::run`), NOT an SNI/ksni tray
  item. Panel: `button::custom(panel_element)` with
  `theme::Button::AppletIcon`, wrapped in `autosize::autosize` so the panel
  width follows the text.
- **Popup:** `on_press_with_rectangle` → `Message::TogglePopup(bounds)` →
  `window::Id::unique()` + `core.applet.get_popup_settings(...)` with
  `positioner.size = None` and the full widget bounds as `anchor_rect`;
  close via `destroy_popup` + `on_close_requested` → `Message::PopupClosed`.
- **Settings window:** a second libcosmic window is impossible from the
  applet process (privileged panel wayland socket, only layer surfaces).
  The applet SPAWNS the helper binary `cosmic-applet-homeassistant-settings`
  (`src/bin/`); the path is resolved next to `current_exe`
  (`applet::settings_binary_path()`). The helper writes the config and
  exits.
- **Config:** TOML at
  `~/.config/cosmic-applet-homeassistant/config.toml`, written with `0600`
  (`config::save_to`): instance URL, access token, entity list
  (`entity_id` + display name), panel boundary (`panel_count`), refresh
  interval. Env vars (`HOME_ASSISTANT_URL`/`HA_URL`,
  `HOME_ASSISTANT_TOKEN`/`HA_TOKEN`) only fill values missing from the
  file.
- **Refresh:** self-rescheduling task chain — `init` → `fetch_all` →
  `Message::Fetched` → `tokio::time::sleep(interval)` → `Message::Tick` →
  `fetch_all` … (no subscription). A 1 s `stat()` watcher on the config
  file (`wait_for_config_change`) picks up settings saves within ~1 s —
  no restart needed.
- **REST:** `GET {instance_url}/api/states/<entity_id>` with
  `Authorization: Bearer <token>` (long-lived access token, HA → profile →
  security), parallel per entity (`src/ha.rs`: `fetch_entity` + pure
  parser `parse_state_response`).
- **Failed fetches keep state:** `merge_fetched` only applies `Ok`
  results; `Err` (network, auth, unknown entity) leaves the previously
  displayed value untouched and freezes the "Updated" clock. The `—`
  placeholder only appears for entities that have never been fetched
  successfully.
- **Panel line semantics:** the first `panel_count` entities (in list
  order) render next to the logo, separated by " · ". Moving an entity
  across the boundary line (`config::move_entity_across`) changes only
  that entity's membership; removing above the line shifts the line
  (`config::remove_row`).

## libcosmic patterns (verified at the pinned revision)

- libcosmic from Git with `default-features = false` and explicit features
  (`applet`, `applet-token`, `dbus-config`, `multi-window`, `tokio`,
  `wayland`, `winit`).
- `cosmic::app::Task<M>` is `iced::Task<cosmic::Action<M>>`; Application
  methods return `Task<Message>`. `cosmic::Task` (root) is the bare
  `iced::Task`.
- **Text colors:** do not use iced's `Text::color_maybe`; use `.class
  (color)` on the text widget. Bind `cosmic::theme::active()` to a
  variable before using `.cosmic()` spacing fields.
- Text inputs in windows: libcosmic auto-inserts a header bar for regular
  windows (`core.window.show_headerbar`) — do not add a manual
  `header_bar()` (double title bar). Set titles via
  `core.set_header_title` + `ApplicationExt::set_window_title`.
- `window::open` from the applet process does not work (see settings
  window above); the iced `size_limits` you pass must have finite max
  values on both axes, otherwise the xdg_toplevel gets a negative max
  size and the window crashes on open.

## Module structure

- `src/config.rs` — TOML `AppConfig` (load/save/`load_from`/`save_to`),
  env fallbacks, boundary helpers (`move_entity_across`, `remove_row`,
  `move_entity`).
- `src/ha.rs` — REST client + pure parser (httpmock-tested).
- `src/applet.rs` — panel, popup, fetch chain, config watcher, settings
  spawn; pure formatting helpers (`format_panel_line`, `popup_rows`)
  for tests.
- `src/bin/cosmic-applet-homeassistant-settings.rs` — settings window.
- `src/i18n.rs` — Fluent loader (`fl!` macro, catalogs in
  `i18n/<lang>/cosmic_applet_homeassistant.ftl`, language from desktop
  locale, `I18N_LANGUAGE` override for tests).

## Testing & building

- Test suite: `cargo test` (config roundtrip incl. invalid-TOML fallback,
  HA parser via httpmock, panel/popup formatting, i18n, settings-spawn
  path). Env-dependent tests live in ONE test function — `std::env` is
  process-wide and parallel tests race.
- Translations: add strings via `fl!("message-id")` in both catalogs
  (`i18n/en`, `i18n/de`) — `fl!` validates against EN at compile time.
- Lint: `cargo clippy --all-targets -- -D warnings`.
- Release: `cargo build --release` (10–15 min), then `./install.sh`
  (binaries → `~/.local/bin`, desktop entry →
  `~/.local/share/applications/net.d11n.CosmicHomeassistant.desktop`,
  icon → `~/.local/share/icons/hicolor/scalable/apps/` — without an icon
  the applet list in COSMIC settings shows nothing).
- Panel config: `~/.config/cosmic/com.system76.CosmicPanel.Panel/v1
  /plugins_wings` (RON). `./restart-applet.sh` toggles the panel entry for
  a hot reload.
