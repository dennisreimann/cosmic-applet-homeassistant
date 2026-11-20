# Vendored: switcheroo-control (0.1.0)

Wie `vendor/cosmic-settings-daemon`: Snapshot von
`pop-os/dbus-settings-bindings @ eed01dd…` (MPL-2.0). Upstream main hat
das Crate zu `switcheroo-control-zbus` umbenannt; der gepinnte
libcosmic-Rev braucht über sein Workspace-Crate `cosmic-launch`
weiterhin den alten Namen. Der `[patch]`-Eintrag in der
Workspace-`Cargo.toml` hält `cargo update` funktionsfähig.
