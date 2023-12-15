use std::collections::HashMap;

use pyrite::app::{
    resource::{Resource, ResourceBank},
    AppBuilder,
};

const MAX_COUNT: u32 = 10;

#[derive(Resource)]
struct Counter {
    count: u32,
}

fn should_increment_counter(counter: &Counter) -> bool {
    counter.count < MAX_COUNT
}

fn increment_counter(counter: &mut Counter) {
    println!(
        "Incrementing counter from {} to {}",
        counter.count,
        counter.count + 1
    );
    counter.count += 1;
}

fn main() {
    let mut app_builder = AppBuilder::new();
    app_builder.add_resource(Counter { count: 0 });

    app_builder.run();

    // Should increment count from 0 to 10
}

// should_increment_counter ->
