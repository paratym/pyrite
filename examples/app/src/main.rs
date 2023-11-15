use std::thread;
use std::time::Duration;

use pyrite::app::resource::ResourceBank;
use pyrite::app::stage::DEFAULT_STAGE;
use pyrite::app::system::SystemParameter;
use pyrite::desktop;
use pyrite::prelude::{Time, *};

#[derive(Resource)]
struct TestResource {
    value: i32,
}

#[derive(Resource)]
struct OtherResource {
    value: bool,
}

fn test_system() {
    thread::sleep(Duration::from_secs(1));
    println!("Hello, world!");
}

fn with_resource_system(resource: ResMut<TestResource>) {
    thread::sleep(Duration::from_secs_f32(0.5));

    println!("Resource value: {:?}", resource.value);
}

fn with_resource_mut_system(mut resource: ResMut<TestResource>) {
    thread::sleep(Duration::from_secs_f32(0.75));
    resource.value += 1;
    println!("Mutated Resource value: {:?}", resource.value);
}

fn main() {
    let mut app = AppBuilder::new();

    app.add_resource(TestResource { value: 0 });
    app.add_resource(OtherResource { value: true });

    app.add_system(test_system);
    app.add_system(with_resource_system);
    app.add_system(with_resource_mut_system);

    app.set_entry_point(|mut application| {
        application.execute_stage(DEFAULT_STAGE);
    });

    app.run();
}
