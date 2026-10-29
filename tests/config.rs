use cosmic_applet_homeassistant::config::{self, AppConfig, EntityConfig};

/// Alle env-basierten Tests in EINEM Test: `std::env` ist prozessweit —
/// parallel laufende Tests würden sich gegenseitig die Variablen wegnehmen.
#[test]
fn env_fallbacks_sequential()
{
    unsafe {
        std::env::set_var("HOME_ASSISTANT_URL", " http://home.local:8123/ ");
        std::env::remove_var("HA_URL");
    }
    assert_eq!(
        config::instance_url().as_deref(),
        Some("http://home.local:8123")
    );

    unsafe {
        std::env::remove_var("HOME_ASSISTANT_URL");
        std::env::set_var("HA_URL", "   ");
    }
    assert_eq!(config::instance_url(), None);
    unsafe { std::env::set_var("HA_URL", "https://ha.example.org") };
    assert_eq!(
        config::instance_url().as_deref(),
        Some("https://ha.example.org")
    );

    unsafe {
        std::env::remove_var("HOME_ASSISTANT_TOKEN");
        std::env::set_var("HA_TOKEN", " eyJhb.d.123 ");
    }
    assert_eq!(config::access_token().as_deref(), Some("eyJhb.d.123"));
    unsafe {
        std::env::remove_var("HA_TOKEN");
        std::env::set_var("HOME_ASSISTANT_TOKEN", "");
    }
    assert_eq!(config::access_token(), None);
}
#[test]
fn app_config_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let cfg = AppConfig {
        instance_url: "http://homeassistant.local:8123".into(),
        access_token: "abc.def.ghi".into(),
        panel_count: Some(1),
        refresh_minutes: 1,
        entities: vec![EntityConfig {
            entity_id: "sensor.temperatur".into(),
            name: "Temperatur".into(),
        }],
    };

    config::save_to(&path, &cfg).unwrap();

    let loaded = config::load_from(&path);
    assert_eq!(loaded, cfg);
}

#[test]
fn load_invalid_toml_falls_back_to_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "not valid toml [[[").unwrap();

    assert_eq!(config::load_from(&path), AppConfig::default());
}

#[test]
fn load_missing_file_falls_back_to_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing.toml");

    assert_eq!(config::load_from(&path), AppConfig::default());
}


#[test]
fn refresh_interval_defaults_to_one_minute() {
    let cfg = AppConfig::default();
    assert_eq!(config::refresh_interval(&cfg), std::time::Duration::from_secs(60));
}

#[test]
fn refresh_interval_minimum_is_one_minute() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "refresh_minutes = 0\n").unwrap();
    let cfg = config::load_from(&path);
    assert_eq!(cfg.refresh_minutes, 1);

    std::fs::write(&path, "refresh_minutes = 5\n").unwrap();
    let cfg = config::load_from(&path);
    assert_eq!(config::refresh_interval(&cfg), std::time::Duration::from_secs(300));
}

#[test]
fn move_entity_reorders_within_bounds() {
    let mut es = vec![
        EntityConfig { entity_id: "a".into(), name: "A".into() },
        EntityConfig { entity_id: "b".into(), name: "B".into() },
        EntityConfig { entity_id: "c".into(), name: "C".into() },
    ];
    config::move_entity(&mut es, 0, 2);
    assert_eq!(es.iter().map(|e| e.entity_id.as_str()).collect::<Vec<_>>(), ["b", "c", "a"]);
    config::move_entity(&mut es, 2, 0);
    assert_eq!(es.iter().map(|e| e.entity_id.as_str()).collect::<Vec<_>>(), ["a", "b", "c"]);
}

#[test]
fn move_entity_ignores_out_of_bounds() {
    let mut es = vec![EntityConfig { entity_id: "a".into(), name: "A".into() }];
    config::move_entity(&mut es, 5, 0);  // from OOB
    config::move_entity(&mut es, 0, 5);  // to OOB
    assert_eq!(es.len(), 1);
    assert_eq!(es[0].entity_id, "a");
}

#[test]
fn panel_entities_none_means_all() {
    let cfg = AppConfig {
        entities: vec![
            EntityConfig { entity_id: "a".into(), name: "A".into() },
            EntityConfig { entity_id: "b".into(), name: "B".into() },
        ],
        panel_count: None,
        ..AppConfig::default()
    };
    assert_eq!(cfg.panel_entities().len(), 2);
}

#[test]
fn panel_entities_some_means_first_n_clamped() {
    let cfg = AppConfig {
        entities: vec![
            EntityConfig { entity_id: "a".into(), name: "A".into() },
            EntityConfig { entity_id: "b".into(), name: "B".into() },
        ],
        panel_count: Some(1),
        ..AppConfig::default()
    };
    assert_eq!(cfg.panel_entities().len(), 1);
    assert_eq!(cfg.panel_entities()[0].entity_id, "a");

    // Some(99) wird auf Länge geklemmt
    let cfg = AppConfig { panel_count: Some(99), ..cfg };
    assert_eq!(cfg.panel_entities().len(), 2);
}

#[test]
fn remove_row_above_boundary_shifts_boundary() {
    let mut rows = vec![
        EntityConfig { entity_id: "a".into(), name: "A".into() },
        EntityConfig { entity_id: "b".into(), name: "B".into() },
        EntityConfig { entity_id: "c".into(), name: "C".into() },
    ];
    let mut boundary = 2; // a,b im Panel, C im Dropdown

    // Entfernen ÜBER der Linie: Linie muss mitrutschen, sonst rutscht
    // ein Dropdown-Entity ins Panel.
    config::remove_row(&mut rows, &mut boundary, 0);
    assert_eq!(rows.len(), 2);
    assert_eq!(boundary, 1);
    assert_eq!(rows[0].entity_id, "b"); // weiter im Panel

    // Entfernen UNTER der Linie: Linie bleibt.
    config::remove_row(&mut rows, &mut boundary, 1);
    assert_eq!(rows.len(), 1);
    assert_eq!(boundary, 1);
}

#[test]
fn remove_row_clamps_boundary_to_length() {
    let mut rows = vec![EntityConfig { entity_id: "a".into(), name: "A".into() }];
    let mut boundary = 1;
    config::remove_row(&mut rows, &mut boundary, 0);
    assert!(rows.is_empty());
    assert_eq!(boundary, 0);
    config::remove_row(&mut rows, &mut boundary, 0); // OOB ignorieren
}

#[test]
fn move_entity_across_keeps_other_members() {
    let mk = |id: &str| EntityConfig { entity_id: id.into(), name: id.into() };
    let mut es = vec![mk("a"), mk("b"), mk("c"), mk("d"), mk("e")];
    let mut boundary = 3; // a,b,c im Panel

    // Entity ÜBER die Linie nach oben (d von idx 3 nach 2): d kommt ins
    // Panel, die bisherigen Panel-Mitglieder a,b,c bleiben drin.
    config::move_entity_across(&mut es, &mut boundary, 3, 2);
    assert_eq!(boundary, 4);
    let ids: Vec<&str> = es.iter().map(|e| e.entity_id.as_str()).collect();
    assert_eq!(ids, ["a", "b", "d", "c", "e"]);

    // Entity UNTER die Linie nach unten (b von idx 1 nach 3): b verlässt
    // das Panel, alle anderen bleiben in ihren Gruppen.
    config::move_entity_across(&mut es, &mut boundary, 1, 3);
    // a, d, c bleiben im Panel; die Linie folgt dem zurückkehrenden b.
    assert_eq!(boundary, 3);
    let ids: Vec<&str> = es.iter().map(|e| e.entity_id.as_str()).collect();
    assert_eq!(ids, ["a", "d", "c", "b", "e"]);
}

#[test]
fn move_entity_across_internal_moves_follow_line() {
    let mk = |id: &str| EntityConfig { entity_id: id.into(), name: id.into() };
    let mut es = vec![mk("a"), mk("b"), mk("c"), mk("d"), mk("e")];
    let mut boundary = 2;

    // Internes Aufwärtsschieben im Panel (b von 1 nach 0): gleiche
    // Mitglieder, Linie bleibt zwischen denselben Entities.
    config::move_entity_across(&mut es, &mut boundary, 1, 0);
    assert_eq!(boundary, 2);
    let ids: Vec<&str> = es.iter().map(|e| e.entity_id.as_str()).collect();
    assert_eq!(ids, ["b", "a", "c", "d", "e"]);

    // Internes Verschieben im Dropdown (d von 3 nach 4): Linie unverändert.
    config::move_entity_across(&mut es, &mut boundary, 3, 4);
    assert_eq!(boundary, 2);
}

#[test]
fn move_entity_across_ignores_out_of_bounds() {
    let mk = |id: &str| EntityConfig { entity_id: id.into(), name: id.into() };
    let mut es = vec![mk("a")];
    let mut boundary = 1;
    config::move_entity_across(&mut es, &mut boundary, 5, 0);
    config::move_entity_across(&mut es, &mut boundary, 0, 5);
    assert_eq!(es.len(), 1);
    assert_eq!(boundary, 1);
}
