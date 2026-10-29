use cosmic_applet_homeassistant::applet::settings_binary_path;

#[test]
fn settings_binary_is_sibling_of_running_binary() {
    // relativer Pfad: exe-Verzeichnis + Name des Settings-Binaries
    let path = settings_binary_path();
    assert!(path.is_some(), "current_exe muss auflösbar sein");
    let path = path.unwrap();
    assert!(path.ends_with("cosmic-applet-homeassistant-settings"));
}

#[test]
fn sibling_resolution_works() {
    // Der Helper selbst: exe `/x/y/foo` → `/x/y/bar`
    let s = cosmic_applet_homeassistant::applet::sibling_binary("/x/y/foo", "bar");
    assert_eq!(s, "/x/y/bar");
}
