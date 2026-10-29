use cosmic_applet_homeassistant::applet::popup_rows;
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
fn rows_in_config_order_name_left_value_right() {
    let v = values(vec![
        ("sensor.humidity", Ok(EntityState { state: "62".into(), unit: Some("%".into()), friendly_name: None })),
        ("sensor.temp", Ok(EntityState { state: "21.5".into(), unit: Some("°C".into()), friendly_name: None })),
    ]);
    // Reihenfolge folgt der Konfiguration, nicht der Fetch-Reihenfolge.
    assert_eq!(
        popup_rows(&v, &entities()),
        vec![
            ("Temperatur".to_string(), "21.5 °C".to_string()),
            ("Luftfeuchte".to_string(), "62 %".to_string()),
        ]
    );
}

#[test]
fn failed_entity_shows_dash() {
    let e = vec![EntityConfig { entity_id: "sensor.x".into(), name: "X".into() }];
    let v = values(vec![("sensor.x", Err(()))]);
    assert_eq!(popup_rows(&v, &e), vec![("X".to_string(), "—".to_string())]);
}

#[test]
fn missing_entity_shows_dash() {
    let e = vec![EntityConfig { entity_id: "sensor.x".into(), name: "X".into() }];
    assert_eq!(popup_rows(&HashMap::new(), &e), vec![("X".to_string(), "—".to_string())]);
}

#[test]
fn empty_config_gives_no_rows() {
    assert!(popup_rows(&HashMap::new(), &[]).is_empty());
}
