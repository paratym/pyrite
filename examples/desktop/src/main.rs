use pyrite::prelude::*;

fn main() {
    let mut app_builder = AppBuilder::new();

    setup_desktop_preset(
        &mut app_builder,
        DesktopConfig {
            application_name: "Desktop Example".to_string(),
            window_config: WindowConfig::default(),
            ..Default::default()
        },
    );

    app_builder.run();
}
