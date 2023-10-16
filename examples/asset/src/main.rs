use pyrite::app::stage::DEFAULT_STAGE;
use pyrite::asset::loaders::txt::TxtLoader;
use pyrite::asset::Assets;
use pyrite::prelude::*;

#[derive(Resource)]
struct AssetBank {
    handle: Handle<String>,
}

impl AssetBank {
    fn wait_for_assets(&self) {
        // TODO: Make the asset loading process asynchronous.
        // self.handle.wait();
    }
}

fn main() {
    let mut app = AppBuilder::new();

    setup(&mut app);

    app.set_entry_point(entry_point);
    app.run();
}

fn setup(app: &mut AppBuilder) {
    let mut assets = Assets::new();
    assets.add_loader(TxtLoader {});
    let handle = assets.load::<String>("assets/test.txt");

    app.add_resource(assets);
    app.add_resource(AssetBank { handle });

    app.add_system(|bank: Res<AssetBank>| {
        bank.wait_for_assets();
        println!("{}", *bank.handle);
    });
}

fn entry_point(mut application: Application) {
    application.execute_stage(DEFAULT_STAGE);
}
