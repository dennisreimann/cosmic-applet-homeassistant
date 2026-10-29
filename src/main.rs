fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,cosmic_applet_homeassistant=debug".into()),
        )
        .init();
    let _ = tracing_log::LogTracer::init();
    cosmic_applet_homeassistant::applet::run()
}
