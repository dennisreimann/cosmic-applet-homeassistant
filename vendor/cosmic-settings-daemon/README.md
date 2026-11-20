# Vendored: cosmic-settings-daemon (0.1.0)

Snapshot von [pop-os/dbus-settings-bindings][upstream], Revision
`eed01dd3609e90e3c8cd043656734c500956c793` (MPL-2.0, siehe `LICENSE.md`).

## Warum liegt dieses Crate im Repo?

Der gepinnte libcosmic-Rev (`95236a9…`, siehe `Cargo.toml`) hängt
zwingend an `cosmic-settings-daemon` aus dem Repo
`pop-os/dbus-settings-bindings`. Upstream hat das Crate auf `main` zu
`cosmic-settings-daemon-zbus` (zbus 6) umbenannt — jede Auflösung gegen
`main` scheitert daher mit „no matching package named
`cosmic-settings-daemon`“, und `cargo update` bricht ab.

Der `[patch.'https://github.com/pop-os/dbus-settings-bindings']`-Eintrag
in der Workspace-`Cargo.toml` bindet dieses Crate über den Pfad ein.
Damit läuft `cargo update` wieder vollständig; die übrigen Bindings des
Upstream-Repos (networkmanager, upower, …) aktualisieren sich normal.

Ein Git-Rev-Patch als Alternative scheitert an Cargos
Same-Source-Regel („patches must point to different sources“) — der
Pfad-Patch ist der einzige self-contained Ausweg.

## Wann kann das Vendor weg?

Sobald der libcosmic-Pin auf eine Revision gehoben wird, die die
zbus-6-Bindings unter dem neuen Namen nutzt, kann der Patch-Eintrag und
dieses Verzeichnis entfernt werden.

[upstream]: https://github.com/pop-os/dbus-settings-bindings
