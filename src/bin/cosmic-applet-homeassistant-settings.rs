//! Eigenständiges Einstellungsfenster für das Applet.
//!
//! Das Applet selbst kann aus dem Panel-Applet-Kontext keine normalen
//! Fenster öffnen (privilegierter Panel-Wayland-Socket, nur
//! Layer-Surfaces). Das Applet spawnt daher dieses Binary
//! (`Message::OpenSettings` → `settings_binary_path()`).

use cosmic::app::{Core, Task};
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::{Alignment, Length, window};
use cosmic::widget::{button, column, container, divider, row, scrollable, text, text_input};
use cosmic::{Application, Element};
use cosmic_applet_homeassistant::config::{self, AppConfig, EntityConfig};
use cosmic_applet_homeassistant::fl;

fn main() -> cosmic::iced::Result {
    cosmic::app::run::<SettingsApp>(
        cosmic::app::Settings::default()
            .size(cosmic::iced::Size::new(550.0, 640.0))
            // Breite fix (nur in der Höhe skalierbar). Height-Grenzen
            // explizit endlich — max-Höhe INFINITY erzeugt beim xdg_toplevel
            // einen negativen Wert und killt das Fenster.
            .size_limits(
                cosmic::iced::Limits::NONE
                    .min_width(550.0)
                    .min_height(400.0)
                    .max_width(550.0)
                    .max_height(1200.0),
            ),
        (),
    )
}

/// Feste Breite aller Input-Felder im Settings-Fenster.
const INPUT_WIDTH: f32 = 170.0;

#[derive(Debug, Clone)]
enum Message {
    UrlChanged(String),
    TokenChanged(String),
    RefreshMinutesChanged(String),
    EntityIdChanged(usize, String),
    NameChanged(usize, String),
    MoveRowUp(usize),
    MoveRowDown(usize),
    BoundaryUp,
    BoundaryDown,
    AddRow,
    RemoveRow(usize),
    Save,
}

#[derive(Default)]
struct SettingsApp {
    core: Core,
    url: String,
    token: String,
    refresh_minutes: String,
    /// Grenzindex: rows[..boundary] erscheint im Panel. rows.len() = alle.
    boundary: usize,
    rows: Vec<EntityConfig>,
}

impl Application for SettingsApp {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "net.d11n.CosmicHomeassistantSettings";

    fn init(core: Core, _flags: ()) -> (Self, Task<Message>) {
        let cfg: AppConfig = cosmic_applet_homeassistant::config::load();
        let main_id = core.main_window_id().unwrap();
        let mut app = Self {
            core,
            url: cfg.instance_url,
            token: cfg.access_token,
            refresh_minutes: cfg.refresh_minutes.to_string(),
            boundary: match cfg.panel_count {
                Some(n) => n.min(cfg.entities.len()),
                None => cfg.entities.len(),
            },
            rows: cfg.entities.clone(),
        };
        app.core.set_header_title(fl!("settings-title"));
        let title_task = cosmic::ApplicationExt::set_window_title(
            &mut app,
            fl!("settings-title"),
            main_id,
        );
        (app, title_task)
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UrlChanged(v) => self.url = v,
            Message::TokenChanged(v) => self.token = v,
            Message::RefreshMinutesChanged(v) => self.refresh_minutes = v,
            Message::EntityIdChanged(i, v) => {
                if let Some(r) = self.rows.get_mut(i) {
                    r.entity_id = v;
                }
            }
            Message::NameChanged(i, v) => {
                if let Some(r) = self.rows.get_mut(i) {
                    r.name = v;
                }
            }
            Message::MoveRowUp(i) => {
                if i > 0 {
                    config::move_entity_across(&mut self.rows, &mut self.boundary, i, i - 1);
                }
            }
            Message::MoveRowDown(i) => {
                if i + 1 < self.rows.len() {
                    config::move_entity_across(&mut self.rows, &mut self.boundary, i, i + 1);
                }
            }
            Message::BoundaryUp => {
                if self.boundary > 0 {
                    self.boundary -= 1;
                }
            }
            Message::BoundaryDown => {
                if self.boundary < self.rows.len() {
                    self.boundary += 1;
                }
            }
            Message::AddRow => self.rows.push(EntityConfig {
                entity_id: String::new(),
                name: String::new(),
            }),
            Message::RemoveRow(i) => config::remove_row(&mut self.rows, &mut self.boundary, i),
            Message::Save => {
                let refresh_minutes: u32 =
                    self.refresh_minutes.trim().parse().unwrap_or(1);
                let cfg = AppConfig {
                    instance_url: self.url.trim().trim_end_matches('/').to_string(),
                    access_token: self.token.trim().to_string(),
                    panel_count: (self.boundary != self.rows.len()).then_some(self.boundary),
                    entities: self
                        .rows
                        .iter()
                        .filter(|e| !e.entity_id.trim().is_empty())
                        .map(|e| EntityConfig {
                            entity_id: e.entity_id.trim().to_string(),
                            name: if e.name.trim().is_empty() {
                                e.entity_id.trim().to_string()
                            } else {
                                e.name.trim().to_string()
                            },
                        })
                        .collect(),
                    refresh_minutes: refresh_minutes.clamp(1, 1440),
                };
                if let Err(e) = cosmic_applet_homeassistant::config::save(&cfg) {
                    eprintln!("Config speichern fehlgeschlagen: {e}");
                }
                return window::close(self.core.main_window_id().unwrap());
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        self.view_window(self.core.main_window_id().unwrap())
    }

    fn view_window(&self, _id: window::Id) -> Element<'_, Message> {
        let binding = cosmic::theme::active();
        let Spacing {
            space_xxxs,
            space_xxs,
            space_s,
            space_m,
            ..
        } = binding.cosmic().spacing;

        let mut col = column![].padding(space_m).spacing(space_s).width(Length::Fill);

        // Label + Input als Gruppe: Abstand halbiert (space_xxxs statt
        // space_s), damit das Label optisch am Input hängt.
        fn labeled<'a>(
            label: String,
            input: cosmic::widget::TextInput<'a, Message>,
            spacing: f32,
        ) -> cosmic::widget::Column<'a, Message, cosmic::Theme> {
            cosmic::widget::column![text::body(label), input]
                .spacing(spacing)
                .width(Length::Fill)
        }

        col = col.push(labeled(
            fl!("instance-url"),
            text_input::text_input(fl!("instance-url"), &self.url)
                .on_input(Message::UrlChanged),
            space_xxxs as f32,
        ));

        col = col.push(labeled(
            fl!("access-token"),
            text_input::secure_input(fl!("access-token"), &self.token, None, true)
                .on_input(Message::TokenChanged),
            space_xxxs as f32,
        ));

        // Feste Breite für ALLE Inputs: Intervall-Feld endet exakt wie die
        // Entity-Felder — "Minuten" beginnt exakt auf Höhe des Namensfelds.
        col = col.push(
            column![
                text::body(fl!("refresh-interval")),
                row![
                    text_input::text_input(fl!("refresh-interval"), &self.refresh_minutes)
                        .on_input(Message::RefreshMinutesChanged)
                        .width(Length::Fixed(INPUT_WIDTH)),
                    text::body(fl!("minutes")),
                ]
                .align_y(Alignment::Center)
                .spacing(space_xxs),
            ]
            .spacing(space_xxxs),
        );

        col = col.push(
            container(text::heading(fl!("entities-header")))
                .padding([space_xxs, 0, 0, 0]),
        );

        // Entfernen-Kreuz (X) in kräftigem Rot, quadratisch in Input-Höhe,
        // runde Ecken via Button-Style.
        let trash_button = |i: usize| {
            let trash_icon = cosmic::widget::icon::icon(
                cosmic::widget::icon::Handle::from(
                    cosmic::widget::icon::from_name("window-close-symbolic"),
                ),
            )
            .size(16)
            .class(cosmic::theme::Svg::custom(move |_| {
                cosmic::iced::widget::svg::Style {
                    color: Some(cosmic::iced::Color::from_rgb(0.83, 0.11, 0.11)),
                }
            }));
            button::custom(trash_icon)
                .padding(f32::from(space_xxs) + 2.0)
                .on_press(Message::RemoveRow(i))
        };
        let arrow = |name: &str, msg: Message| {
            button::icon(cosmic::widget::icon::Handle::from(
                cosmic::widget::icon::from_name(name),
            ))
            .on_press(msg)
        };

        // Zeile mit der Trennlinie: Divider füllt bis zum Button-Cluster;
        // Pfeile + Platzhalter (≈ Trash-Breite) spiegeln das Muster der
        // Entity-Zeilen, damit die Pfeile exakt übereinanderstehen.
        let boundary_row = || {
            row![
                divider::horizontal::default().width(Length::Fill),
                arrow("go-up-symbolic", Message::BoundaryUp),
                arrow("go-down-symbolic", Message::BoundaryDown),
                container(row![]).width(Length::Fixed(36.0)),
            ]
            .spacing(space_xxs)
            .align_y(Alignment::Center)
            .width(Length::Fill)
        };

        for (i, row_cfg) in self.rows.iter().enumerate() {
            if i == self.boundary {
                // Trennlinie: darüber steht, was im Panel erscheint. Immer
                // sichtbar — auch an erster/letzter Position, sonst lässt
                // sie sich nicht zurückholen.
                col = col.push(boundary_row());
            }
            col = col.push(
                row![
                    text_input::text_input(fl!("entity-id"), &row_cfg.entity_id)
                        .on_input(move |v| Message::EntityIdChanged(i, v))
                        .width(Length::Fixed(INPUT_WIDTH)),
                    text_input::text_input(fl!("entity-name"), &row_cfg.name)
                        .on_input(move |v| Message::NameChanged(i, v))
                        .width(Length::Fixed(INPUT_WIDTH)),
                    // Button-Cluster rechtsbündig, damit er bei allen
                    // Zeilen auf derselben x-Position endet.
                    container(row![]).width(Length::Fill),
                    arrow("go-up-symbolic", Message::MoveRowUp(i)),
                    arrow("go-down-symbolic", Message::MoveRowDown(i)),
                    trash_button(i),
                ]
                .spacing(space_xxs)
                .align_y(Alignment::Center)
                .width(Length::Fill),
            );
        }
        if !self.rows.is_empty() && self.boundary == self.rows.len() {
            // Linie an letzter Position: hinter allen Zeilen ausgeben.
            col = col.push(boundary_row());
        }
        col = col.push(
            row![
                button::standard(fl!("add-entity")).on_press(Message::AddRow),
                // Speichern rechtsbündig
                container(row![]).width(Length::Fill),
                button::suggested(fl!("save")).on_press(Message::Save),
            ]
            .spacing(space_s)
            .width(Length::Fill),
        );

        container(scrollable(col))
            .width(Length::Fill)
            .into()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        None
    }
}
