# AGENTS.md

Guidelines and context for AI agents working on this repository.

## Project

**cosmic-applet-homeassistant** — A native COSMIC panel applet for Pop!_OS that
displays a **configurable list of Home Assistant entities** in the panel
(state values with units, e.g. temperature sensors, binary states).

**Status: skeleton** (forked from the author's `cosmic_opencode_tray` /
cosmic-applet-opencode-quota with all OpenCode specifics removed). The libcosmic
applet shell, popup mechanics, i18n and config layout are in place; the Home
Assistant integration and the entity-list configuration still need to be
designed and built. OpenCode-specific modules (quota parsing, OpenCode API,
cache format, OpenCode logo, quota panel formatting) were deliberately **not**
carried over.

## Architecture decisions (inherited from the source project)

- **Native COSMIC applet** (`cosmic::applet::run`), NOT an SNI/ksni tray item:
  cosmic-applet-status-area forces every SNI icon into a fixed square
  (`Length::Fixed` on both axes) — readable wide text is physically impossible
  there. A native applet renders arbitrary text at panel height via
  `core.applet.text(...)` (automatic theming: white/black depending on
  dark/light).
- **Panel widget:** `button::custom(panel_element)` with
  `theme::Button::AppletIcon`, wrapped in `autosize::autosize(btn, AUTOSIZE_MAIN_ID)`
  so the panel width follows the text length.
- **Popup:** the working pattern (verified in the source project) is
  `on_press_with_rectangle` on the panel button → `Message::TogglePopup(bounds)`
  → `window::Id::unique()` + `core.applet.get_popup_settings(...)` with
  `positioner.size = None` and `anchor_rect` = full widget bounds →
  `get_popup(popup_settings)`; close via `destroy_popup` +
  `on_close_requested` → `Message::PopupClosed`. Popup content:
  `core.applet.popup_container(popup_element)` in `view_window`.
- **Deprecation note:** if the popup ever stops showing, the official reference
  is `examples/applet` in the libcosmic checkout (`~/.cargo/git/checkouts/…`),
  pattern `on_press_with_rectangle` + `Message::Surface(surface::Action)` +
  `cosmic::task::message(Action::Cosmic(app::Action::Surface(a)))` + boxed view.
  The case-sensitive selector API changes between libcosmic revisions — check
  the checked-out revision first.

## libcosmic applet patterns (important)

- Dependency: `libcosmic` from Git with `default-features = false` and explicit
  features, including `"applet"`, `"applet-token"`, `"multi-window"`, `"tokio"`,
  `"wayland"`, `"winit"`. (Without the `applet` feature, `cosmic::applet` does
  not exist.)
- `cosmic::app::Task<M>` is an alias for `iced::Task<cosmic::Action<M>>` — the
  Application trait methods return `Task<Message>`. `cosmic::Task` (root) is the
  bare `iced::Task` — do not confuse the two.
- **Text colors:** do not use iced's `Text::color_maybe` (conflicts with the
  cosmic theme class system). Use `.class(color)` on the text widget — cosmic's
  `Text` catalog supports `Text::Color(Color)` (see
  `cosmic::theme::style::iced::Text`). Theme colors:
  `cosmic::theme::active().cosmic().warning.base` (yellow) /
  `.destructive.base` (red); bind `theme::active()` to a variable first
  (`let binding = cosmic::theme::active(); let theme = binding.cosmic();`) to
  avoid a temporary-value lifetime error.
- **Refresh via a self-rescheduling task chain** (pattern from the source
  project, currently not yet wired in): `init` → `Task::perform(fetch, …)` →
  `Message::Fetched` → `Task::perform(tokio::time::sleep(60s), …)` →
  `Message::Tick` → fetch …, instead of an active subscription.
- Spacing/padding inside the popup: destructure
  `cosmic::theme::active().cosmic().spacing` (`space_xxs`, `space_s`, `space_m`).

## Home Assistant data source (to be verified when building)

Candidates for the integration (verify against the current HA docs first):

- **REST:** `GET {instance_url}/api/states/<entity_id>` with
  `Authorization: Bearer <long-lived access token>` →
  `{"state": "21.5", "attributes": {"unit_of_measurement": "°C", …}, …}`.
  Polling model, simple; N requests for N entities (or `/api/states` once and
  filter).
- **WebSocket:** `{instance_url}/api/websocket` — `subscribe_entities` for
  push updates (no polling, but stateful connection + reconnect logic).
- Long-lived access token: Home Assistant → user profile → security.
- Instance URL / token currently via env placeholders (`HOME_ASSISTANT_URL` /
  `HA_URL`, `HOME_ASSISTANT_TOKEN` / `HA_TOKEN`, see `src/config.rs`).
  The entity list and where its configuration lives (cosmic-config DBus config,
  TOML/JSON file, env) is an open design decision for the next session.

## Module structure

- `src/config.rs` — instance URL/token resolution (env placeholders),
  cache path; entity-list configuration TBD.
- `src/i18n.rs` — Fluent loader (`fl!` macro, catalogs in
  `i18n/<lang>/cosmic_applet_homeassistant.ftl`, language chosen from the
  desktop locale, `I18N_LANGUAGE` override for tests).
- `src/applet.rs` — the libcosmic applet (panel view, popup, quit).
- Future modules (suggestion): `ha.rs` (HTTP client, pure parser),
  `cache.rs` (last known values), modeled after the source project's
  `api.rs`/`cache.rs` split (pure data transform isolated for testing).

## References to COSMIC applet internals (cosmic-applets repo)

For panel/applet behavior see the source of `pop-os/cosmic-applets`
(status-notifier watcher, size/anchor handling) and `pop-os/libcosmic`
(`src/applet/mod.rs`, `src/widget/autosize.rs`, `src/widget/rectangle_tracker/`).

## Testing & building

- Test suite: `cargo test` — integration tests in `tests/` (config, i18n_de).
- Translations: always add new strings via `fl!("message-id")` in
  `src/applet.rs`; create message IDs in both catalogs (`i18n/en/…`,
  `i18n/de/…`) — the `fl!` macro validates against the EN catalog at compile
  time.
- Lint: `cargo clippy --all-targets -- -D warnings`.
- Release: `cargo build --release` (10–15 min, iced/libcosmic are heavy).
- Install (user-local): `./install.sh`
  - Binary → `~/.local/bin/cosmic-applet-homeassistant`
  - Applet discovery →
    `~/.local/share/applications/net.d11n.CosmicHomeassistant.desktop`
    (desktop entry with `X-CosmicApplet=true`, `Exec=<absolute path>`,
    `NoDisplay=true`).
- Panel configuration:
  `~/.config/cosmic/com.system76.CosmicPanel.Panel/v1/plugins_wings`
  (RON). `./restart-applet.sh` toggles the panel entry for a hot reload during
  development.
