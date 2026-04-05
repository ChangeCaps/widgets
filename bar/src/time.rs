use std::time::Duration;

use chrono::{DateTime, Local};
use ori_native::{Weight, prelude::*};

pub struct Data {
    time: DateTime<Local>,
}

impl Data {
    pub fn new() -> Self {
        Self { time: Local::now() }
    }
}

pub fn time(data: &Data) -> impl View<Data> + use<> {
    column((
        text(data.time.format("%H").to_string())
            .color(theme::SURFACE)
            .size(14.0)
            .weight(Weight::BOLD)
            .family("Ubuntu Light"),
        text(data.time.format("%M").to_string())
            .color(theme::SURFACE)
            .size(14.0)
            .weight(Weight::BOLD)
            .family("Ubuntu Light"),
    ))
    .align_items(Align::Center)
}

pub fn job() -> impl Effect<Data> {
    task(
        |_, sink| async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                sink.send(());
            }
        },
        |data: &mut Data, _, _| {
            data.time = Local::now();
        },
    )
}
