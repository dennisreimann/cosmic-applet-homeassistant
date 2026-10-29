use cosmic_applet_homeassistant::ha;

#[test]
fn parse_state_with_unit() {
    let body: &str = r#"{"entity_id":"sensor.temperatur","state":"21.5","attributes":{"unit_of_measurement":"°C","friendly_name":"Temperatur"}}"#;
    let s = ha::parse_state_response(body.as_bytes()).unwrap();
    assert_eq!(s.state, "21.5");
    assert_eq!(s.unit.as_deref(), Some("°C"));
    assert_eq!(s.friendly_name.as_deref(), Some("Temperatur"));
}

#[test]
fn parse_state_without_unit() {
    let body: &str = r#"{"entity_id":"binary_sensor.tuer","state":"on","attributes":{"friendly_name":"Tür"}}"#;
    let s = ha::parse_state_response(body.as_bytes()).unwrap();
    assert_eq!(s.state, "on");
    assert_eq!(s.unit, None);
    assert_eq!(s.friendly_name.as_deref(), Some("Tür"));
}

#[test]
fn parse_missing_state_is_err() {
    let body: &str = r#"{"attributes":{}}"#;
    assert!(ha::parse_state_response(body.as_bytes()).is_err());
}

#[tokio::test]
async fn fetch_entity_via_httpmock() {
    let server = httpmock::MockServer::start();
    server.mock(|when, then| {
        when.method(httpmock::Method::GET)
            .path("/api/states/sensor.test")
            .header("Authorization", "Bearer tok");
        then.status(200).body(
            r#"{"entity_id":"sensor.test","state":"42","attributes":{"unit_of_measurement":"%"}}"#,
        );
    });

    let s = ha::fetch_entity(&server.base_url(), "tok", "sensor.test")
        .await
        .unwrap();
    assert_eq!(s.state, "42");
    assert_eq!(s.unit.as_deref(), Some("%"));
}

#[tokio::test]
async fn fetch_entity_unknown_entity_is_err() {
    let server = httpmock::MockServer::start();
    server.mock(|when, then| {
        when.method(httpmock::Method::GET)
            .path("/api/states/sensor.missing");
        then.status(404).body("Entity not found.");
    });

    let err = ha::fetch_entity(&server.base_url(), "tok", "sensor.missing")
        .await
        .unwrap_err();
    assert!(err.contains("404"), "Fehlermeldung: {err}");
}
