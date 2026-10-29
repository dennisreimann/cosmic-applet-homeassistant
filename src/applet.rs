use crate::config::{AppConfig, EntityConfig};
use crate::fl;
use crate::ha::{self, EntityState};
use cosmic::app::{Core, Task};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{widget::svg, Alignment, Length, Rectangle, window};
use cosmic::widget::{autosize, button, column, container, divider, row, text, Id};
use cosmic::{applet, theme, Element};
use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::sync::LazyLock;

pub const APP_ID: &str = "net.d11n.CosmicHomeassistant";

const LOGO_SVG: &str = include_str!("../assets/ha.svg");

/// Wartet, bis sich die Config-Datei ändert (mtime), und liefert dann die
/// neue mtime. Ein Polling per stat() pro Sekunde ist negligible billig —
/// das erlaubt Auto-Refresh nach Speichern im Settings-Fenster ohne
/// Subscription-Mechanik.
async fn wait_for_config_change(baseline: Option<std::time::SystemTime>) -> Option<std::time::SystemTime> {
    let path = crate::config::config_path()?;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let mtime = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok();
        if let Some(t) = mtime {
            if baseline.is_none_or(|b| t != b) {
                return Some(t);
            }
        }
    }
}

/// Letzter bekannter Wert je Entity-ID; `Err(())` = Fetch fehlgeschlagen.
pub type EntityValues = HashMap<String, Result<EntityState, ()>>;

/// Unix-Sekunden als lokale `HH:MM:SS`-Zeit.
#[must_use]
pub fn fmt_updated(unix: i64) -> String {
    DateTime::from_timestamp(unix, 0)
        .map(|d| d.with_timezone(&Local).format("%H:%M:%S").to_string())
        .unwrap_or_else(|| unix.to_string())
}

/// Panel-Zeile: Werte aller konfigurierten Entities in Konfig-Reihenfolge.
/// Keine Entities → leerer String (dann nur das Logo).
pub fn format_panel_line(values: &EntityValues, entities: &[EntityConfig]) -> String {
    entities
        .iter()
        .filter_map(|e| {
            values.get(&e.entity_id).map(|r| match r {
                Ok(s) => match &s.unit {
                    Some(unit) => format!("{} {}", s.state, unit),
                    None => s.state.clone(),
                },
                Err(()) => "—".to_string(),
            })
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Holt alle konfigurierten Entities parallel.
async fn fetch_all(cfg: AppConfig) -> Vec<(String, Result<EntityState, ()>)> {
    let base_url = cfg.instance_url.clone();
    let token = cfg.access_token.clone();
    let configured = !base_url.is_empty() && !token.is_empty();
    let futures = cfg.entities.iter().map(|e| {
        let id = e.entity_id.clone();
        let base_url = base_url.clone();
        let token = token.clone();
        async move {
            let r = if !configured {
                Err(())
            } else {
                ha::fetch_entity(&base_url, &token, &id)
                    .await
                    .map_err(|_| ())
            };
            (id, r)
        }
    });
    futures_util::future::join_all(futures).await
}

// TODO: Refresh-Intervall siehe oben (konstant).

static AUTOSIZE_MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize-main"));

/// Resolves `name` as a sibling of `exe_path`.
pub fn sibling_binary(exe_path: &str, name: &str) -> String {
    match std::path::Path::new(exe_path).parent() {
        Some(dir) => dir.join(name).to_string_lossy().into_owned(),
        None => name.to_string(),
    }
}

/// Path of the settings binary next to the running applet binary.
pub fn settings_binary_path() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    Some(sibling_binary(&exe.to_string_lossy(), "cosmic-applet-homeassistant-settings"))
}

/// Startet das Applet (aufgerufen aus `main`).
pub fn run() -> cosmic::iced::Result {
    cosmic::applet::run::<HomeAssistantApplet>(())
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Button-Bounds (in Fenster-Koordinaten) für die Popup-Positionierung.
    TogglePopup(Rectangle),
    PopupClosed(window::Id),
    Fetched(Vec<(String, Result<EntityState, ()>)>),
    Tick,
    /// Config-Datei hat sich geändert (Settings-Fenster hat gespeichert).
    ConfigChanged(Option<std::time::SystemTime>),
    OpenSettings,
    Quit,
}

#[derive(Default)]
pub struct HomeAssistantApplet {
    core: Core,
    popup: Option<window::Id>,
    cfg: AppConfig,
    values: EntityValues,
    /// mtime der Config beim letzten Beobachten (Basis für den Watcher).
    config_mtime: Option<std::time::SystemTime>,
    /// Letzter abgeschlossener Fetch-Cycle (Unix-Sekunden).
    last_updated_unix: Option<i64>,
}

/// Popup-Zeilen: (Anzeigename, Wert) in Konfig-Reihenfolge.
pub fn popup_rows(values: &EntityValues, entities: &[EntityConfig]) -> Vec<(String, String)> {
    entities
        .iter()
        .map(|e| {
            let value = match values.get(&e.entity_id) {
                Some(Ok(s)) => match &s.unit {
                    Some(unit) => format!("{} {}", s.state, unit),
                    None => s.state.clone(),
                },
                _ => "—".to_string(),
            };
            (e.name.clone(), value)
        })
        .collect()
}

/// Panel-Beschriftung: formatierte Werte der konfigurierten Entities.
fn panel_element(state: &HomeAssistantApplet) -> Element<'_, Message> {
    let line = format_panel_line(&state.values, state.cfg.panel_entities());
    let h = state.core.applet.suggested_size(true).0 as f32;
    if state.core.applet.is_horizontal() {
        let mut r = row![]
            .align_y(Alignment::Center)
            .spacing(state.core.applet.suggested_padding(true).0 / 2);
        r = r.push(
            container(
                svg(svg::Handle::from_memory(LOGO_SVG.as_bytes().to_vec()))
                    .width(Length::Fixed(h))
                    .height(Length::Fixed(h)),
            )
            .align_y(Alignment::Center),
        );
        if !line.is_empty() {
            r = r.push(state.core.applet.text(line));
        }
        r.into()
    } else {
        column![state.core.applet.text(line)]
            .align_x(Alignment::Center)
            .into()
    }
}

fn popup_element(state: &HomeAssistantApplet) -> Element<'_, Message> {
    let Spacing { space_xxs, space_s, space_m, .. } = cosmic::theme::active().cosmic().spacing;

    let mut col = column![]
        .padding([space_s, 0, space_xxs, 0])
        .spacing(space_xxs);

    // Kopfzeile: Titel links, "Aktualisiert" rechts (Muster OpenCode-Widget).
    let head: Element<'_, Message> = match state.last_updated_unix {
        Some(unix) => row![
            text::heading(fl!("popup-title")).width(Length::Fill),
            text::body(fl!("updated-at", t = fmt_updated(unix))),
        ]
        .align_y(Alignment::Center)
        .padding([0, space_m])
        .into(),
        None => container(text::heading(fl!("popup-title")))
            .padding([0, space_m])
            .into(),
    };
    col = col.push(head);

    col = col.push(container(divider::horizontal::default()).padding([space_xxs, 0]));

    // Entity-Werte-Zeilen (Label links, Wert + Einheit rechts).
    let rows = popup_rows(&state.values, &state.cfg.entities);
    if rows.is_empty() {
        col = col.push(container(text::body(fl!("no-entities"))).padding([0, space_m]));
    } else {
        for (name, value) in rows {
            col = col.push(
                row![
                    text::body(name).width(Length::Fill),
                    text::body(value),
                ]
                .padding([0, space_m])
                .spacing(space_s),
            );
        }
    }

    // Trennlinie direkt über der Beenden-Zeile: kein unteres Padding,
    // damit der Abstand zur Menü-Zeile nicht zu groß wird.
    col = col.push(container(divider::horizontal::default()).padding([space_xxs, 0, 0, 0]));
    col = col.push(applet::menu_button(text::body(fl!("settings"))).on_press(Message::OpenSettings));
    col = col.push(applet::menu_button(text::body(fl!("quit"))).on_press(Message::Quit));
    container(col).into()
}

impl cosmic::Application for HomeAssistantApplet {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn init(core: Core, _flags: ()) -> (Self, Task<Message>) {
        let mut app = Self {
            core,
            popup: None,
            cfg: AppConfig::default(),
            values: EntityValues::new(),
            config_mtime: None,
            last_updated_unix: None,
        };
        app.cfg = crate::config::load();
        let cfg = app.cfg.clone();
        (
            app,
            Task::batch(vec![
                Task::perform(fetch_all(cfg), |r| cosmic::Action::App(Message::Fetched(r))),
                Task::perform(wait_for_config_change(None), |t| {
                    cosmic::Action::App(Message::ConfigChanged(t))
                }),
            ]),
        )
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TogglePopup(bounds) => {
                self.cfg = crate::config::load();
                if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = window::Id::unique();
                    self.popup = Some(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size = None;
                    // Anker = gesamtes Widget (nicht nur das Icon-Quadrat),
                    // damit das Popup zentriert darunter erscheint.
                    popup_settings.positioner.anchor_rect = Rectangle {
                        x: bounds.x as i32,
                        y: bounds.y as i32,
                        width: bounds.width.max(1.0) as i32,
                        height: bounds.height.max(1.0) as i32,
                    };
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
                Task::none()
            }
            Message::Fetched(results) => {
                for (id, r) in results {
                    self.values.insert(id, r);
                }
                self.last_updated_unix = Some(
                    chrono::Utc::now().timestamp(),
                );
                let interval = crate::config::refresh_interval(&self.cfg);
                Task::perform(tokio::time::sleep(interval), move |_| {
                    cosmic::Action::App(Message::Tick)
                })
            }
            Message::Tick => {
                let cfg = self.cfg.clone();
                Task::perform(fetch_all(cfg), |r| {
                    cosmic::Action::App(Message::Fetched(r))
                })
            }
            Message::ConfigChanged(mtime) => {
                self.config_mtime = mtime;
                self.cfg = crate::config::load();
                let cfg = self.cfg.clone();
                Task::batch(vec![
                    Task::perform(fetch_all(cfg), |r| {
                        cosmic::Action::App(Message::Fetched(r))
                    }),
                    Task::perform(wait_for_config_change(mtime), |t| {
                        cosmic::Action::App(Message::ConfigChanged(t))
                    }),
                ])
            }
            Message::OpenSettings => {
                match settings_binary_path() {
                    Some(path) => {
                        let child = std::process::Command::new(path).spawn();
                        if let Err(e) = child {
                            tracing::error!("Settings-Fenster starten: {e}");
                        }
                        Task::none()
                    }
                    None => {
                        tracing::error!("Settings-Binary nicht gefunden");
                        Task::none()
                    }
                }
            }
            Message::Quit => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let padding = self.core.applet.suggested_padding(true).0;
        let btn = button::custom(panel_element(self))
            .padding(if self.core.applet.is_horizontal() {
                [0, padding]
            } else {
                [padding, 0]
            })
            .class(theme::Button::AppletIcon)
            .on_press_with_rectangle(move |offset, bounds| {
                Message::TogglePopup(Rectangle {
                    x: bounds.x - offset.x,
                    y: bounds.y - offset.y,
                    width: bounds.width,
                    height: bounds.height,
                })
            });

        autosize::autosize(btn, AUTOSIZE_MAIN_ID.clone()).into()
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Message> {
        self.core
            .applet
            .popup_container(popup_element(self))
            .into()
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(applet::style())
    }
}
