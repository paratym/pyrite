use pyrite::app::stage::DEFAULT_STAGE;
use pyrite::asset::loaders::txt::TxtLoader;
use pyrite::asset::Assets;
use pyrite::prelude::*;

#[derive(Resource)]
struct ShouldExit(bool);

#[derive(Resource)]
struct AssetBank {
    handle: Handle<String>,
}

fn setup(app: &mut AppBuilder) {
    // Assets Resource Setup
    let mut assets = Assets::new();
    assets.add_loader(TxtLoader {});

    let handle = assets.load::<String>("assets/test.txt");

    app.add_resource(assets);
    app.add_resource(AssetBank { handle });
    app.add_resource(ShouldExit(false));

    app.add_system(|mut assets: ResMut<Assets>| {
        assets.update();
    });
    app.add_system(
        |bank: Res<AssetBank>, mut should_exit: ResMut<ShouldExit>| {
            if let Some(asset) = bank.handle.get() {
                println!("Asset loaded: {}", asset);
                should_exit.0 = true;
            }
        },
    );
}

fn main() {
    let mut app = AppBuilder::new();

    setup(&mut app);

    app.set_entry_point(entry_point);
    app.run();
}

fn entry_point(mut application: Application) {
    while !application.get_resource::<ShouldExit>().0 {
        application.execute_stage(DEFAULT_STAGE);
    }
}
