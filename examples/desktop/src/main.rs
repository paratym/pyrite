use pyrite::prelude::*;

fn main() {
    let mut app_builder = AppBuilder::new();

    setup_desktop_preset(
        &mut app_builder,
        DesktopConfig {
            window_config: WindowConfig::default(),
        },
    );

    app_builder.run::<DesktopEntryPoint>();
}
