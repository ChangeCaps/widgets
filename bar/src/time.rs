use std::time::Duration;

use chrono::{DateTime, Local};
use ori_native::{Weight, prelude::*};

use crate::theme;

pub struct Time {
    time: DateTime<Local>,
}

impl Time {
    pub fn new() -> Self {
        Self { time: Local::now() }
    }
}

pub fn time(data: &Time) -> impl View<Time> + use<> {
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
}

pub fn listen_task() -> impl Effect<Time> {
    task(
        |_, sink| async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                sink.send(());
            }
        },
        |data: &mut Time, _, _| {
            data.time = Local::now();
        },
    )
}
