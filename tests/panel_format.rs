use cosmic_applet_homeassistant::applet::format_panel_line;
use cosmic_applet_homeassistant::config::EntityConfig;
use cosmic_applet_homeassistant::ha::EntityState;
use std::collections::HashMap;

fn entities() -> Vec<EntityConfig> {
    vec![
        EntityConfig {
            entity_id: "sensor.temp".into(),
            name: "Temperatur".into(),
        },
        EntityConfig {
            entity_id: "sensor.humidity".into(),
            name: "Luftfeuchte".into(),
        },
    ]
}

fn values(
    entries: Vec<(&str, Result<EntityState, ()>)>,
) -> HashMap<String, Result<EntityState, ()>> {
    entries
        .into_iter()
        .map(|(id, r)| (id.to_string(), r))
        .collect()
}

#[test]
fn empty_entity_list_renders_empty_line() {
    let line = format_panel_line(&HashMap::new(), &[]);
    assert_eq!(line, "");
}

#[test]
fn values_in_config_order_with_unit() {
    let v = values(vec![
        ("sensor.temp", Ok(EntityState { state: "21.5".into(), unit: Some("°C".into()), friendly_name: None })),
        ("sensor.humidity", Ok(EntityState { state: "62".into(), unit: Some("%".into()), friendly_name: None })),
    ]);
    assert_eq!(format_panel_line(&v, &entities()), "21.5 °C · 62 %");
}

#[test]
fn failed_entity_renders_dash() {
    let v = values(vec![
        ("sensor.temp", Err(())),
        ("sensor.humidity", Ok(EntityState { state: "62".into(), unit: Some("%".into()), friendly_name: None })),
    ]);
    assert_eq!(format_panel_line(&v, &entities()), "— · 62 %");
}

#[test]
fn value_without_unit_has_no_unit_suffix() {
    let e = vec![EntityConfig { entity_id: "b.tuer".into(), name: "Tür".into() }];
    let v = values(vec![(
        "b.tuer",
        Ok(EntityState { state: "on".into(), unit: None, friendly_name: None }),
    )]);
    assert_eq!(format_panel_line(&v, &e), "on");
}

#[test]
fn fmt_updated_has_timestamp_structure() {
    use cosmic_applet_homeassistant::applet::fmt_updated;
    // Nur Struktur prüfen (lokale Zeitzone): HH:MM:SS mit Doppelpunkten.
    let s = fmt_updated(1_788_278_419);
    assert!(s.contains(':'), "erwartete HH:MM:SS-Struktur, got {s}");
}
