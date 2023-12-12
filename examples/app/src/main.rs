use std::collections::HashMap;

use pyrite::app::{
    resource::ResourceBank,
    task::{Task, TaskFunction},
    AppBuilder,
};

const MAX_COUNT: u32 = 10;

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
    let app_builder = AppBuilder::new();
    app_builder.add_resource(Counter { count: 0 });

    app_builder.set_schedule(|schedule| {
        let [increment_counter, should_increment_counter] = schedule.add_tasks([
            // Runtime tasks will run whenever possible.
            RuntimeTask::new(increment_counter),
            // Conditional tasks are used in control flow decisions, allowing for async queries.
            ConditionalTask::new(should_increment_counter),
        ]);

        // Sets up the dependency graph for the tasks.
        increment_counter.depends_on(cond(should_increment_counter, true));
    });

    app_builder.set_entry_point(|app| {
        app.run_schedule();
    });

    app_builder.run();

    // Should increment count from 0 to 10
}

// should_increment_counter ->
