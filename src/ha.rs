//! Home-Assistant-REST-Zugriff.
//!
//! HTTP-Fetch (async) plus reine Parser-Funktion, damit das Parsing
//! ohne Server getestet werden kann.

use serde_json::Value;

/// Der angezeigbare Zustand einer Entity.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityState {
    pub state: String,
    pub unit: Option<String>,
    pub friendly_name: Option<String>,
}

const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Parst die Antwort von `GET /api/states/<entity_id>`.
pub fn parse_state_response(body: &[u8]) -> Result<EntityState, String> {
    let v: Value =
        serde_json::from_slice(body).map_err(|e| format!("ungültiges JSON: {e}"))?;
    let state = v
        .get("state")
        .and_then(|s| s.as_str())
        .ok_or_else(|| "Feld \"state\" fehlt".to_string())?
        .to_string();
    let attrs = v.get("attributes");
    Ok(EntityState {
        state,
        unit: attrs
            .and_then(|a| a.get("unit_of_measurement"))
            .and_then(|u| u.as_str())
            .map(str::to_string),
        friendly_name: attrs
            .and_then(|a| a.get("friendly_name"))
            .and_then(|f| f.as_str())
            .map(str::to_string),
    })
}

/// Holt eine Entity per `GET {base_url}/api/states/{entity_id}`.
pub async fn fetch_entity(
    base_url: &str,
    token: &str,
    entity_id: &str,
) -> Result<EntityState, String> {
    let url = format!(
        "{}/api/states/{}",
        base_url.trim_end_matches('/'),
        entity_id
    );
    let client = reqwest::Client::builder()
        .timeout(TIMEOUT)
        .build()
        .map_err(|e| format!("HTTP-Client: {e}"))?;
    let resp = client
        .get(&url)
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|e| format!("{url}: {e}"))?;

    let status = resp.status();
    let body = resp
        .bytes()
        .await
        .map_err(|e| format!("{url}: Body lesen: {e}"))?;
    if !status.is_success() {
        return Err(format!("{url}: Status {}", status.as_u16()));
    }
    parse_state_response(&body).map_err(|e| format!("{url}: {e}"))
}
