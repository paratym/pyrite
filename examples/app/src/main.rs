use pyrite::prelude::*;

#[derive(Resource)]
struct TestResource {
    value: i32,
}

fn test_system() {
    println!("Hello, world!");
}

fn with_resource_system(resource: Res<TestResource>) {
    println!("Resource value: {}", resource.value);
}

fn with_resource_mut_system(mut resource: ResMut<TestResource>) {
    resource.value += 1;
    println!("Mutated Resource value: {}", resource.value);
}

struct Entry {}

impl EntryPoint for Entry {
    fn run(mut application: Application) {
        application.execute_systems();
    }
}

fn main() {
    let mut app = AppBuilder::new();

    app.add_resource(TestResource { value: 0 });

    app.add_system(test_system);
    app.add_system(with_resource_system);
    app.add_system(with_resource_mut_system);

    app.run::<Entry>();
}
