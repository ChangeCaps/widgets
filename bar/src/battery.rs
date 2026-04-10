use std::time::Duration;

use ori_native::prelude::*;

use crate::views::tooltip;

pub struct Data {
    version: u64,
    manager: battery::Manager,
    batteries: Vec<battery::Battery>,
}

impl Data {
    pub fn new() -> battery::Result<Self> {
        Ok(Self {
            version: 0,
            manager: battery::Manager::new()?,
            batteries: Vec::new(),
        })
    }

    pub fn show(&self) -> bool {
        !self.batteries.is_empty()
    }
}

pub const INTERVAL: Duration = Duration::from_secs(2);

pub fn icon(data: &Data) -> impl View<Data> + use<> {
    memo(data.version, |data: &Data| {
        let Some(battery) = data.batteries.first() else {
            return any(column(()));
        };

        let charge = battery.state_of_charge().value;
        let icon: &[u8] = if battery.state() == battery::State::Charging {
            include_bytes!("icon/battery-charging.svg")
        } else if charge < 0.25 {
            include_bytes!("icon/battery25.svg")
        } else if charge < 0.50 {
            include_bytes!("icon/battery50.svg")
        } else if charge < 0.75 {
            include_bytes!("icon/battery75.svg")
        } else {
            include_bytes!("icon/battery.svg")
        };

        let icon = image(icon)
            .size(24.0, 24.0)
            .margin(4.0)
            .tint(theme::YELLOW.fade(0.8));

        let charge = pair("Charge", format!("{:02.0}", charge * 100.0));

        let time_to_empty = battery
            .time_to_empty()
            .map(|time| pair("Empty", format_time(time.value)));

        let time_to_full = battery
            .time_to_full()
            .map(|time| pair("Full", format_time(time.value)));

        let health = battery.state_of_health().value;
        let health = if health < 0.50 {
            "bad"
        } else if health < 0.75 {
            "okay"
        } else if health < 0.90 {
            "fine"
        } else if health < 0.95 {
            "good"
        } else {
            "great"
        };

        let health = pair("Health", health);

        any(tooltip(
            icon,
            column((charge, health, time_to_empty, time_to_full)).gap(8.0),
        ))
    })
}

fn pair<T>(left: impl Into<String>, right: impl Into<String>) -> impl View<T> {
    row((
        text(left)
            .color(theme::SURFACE)
            .size(12.0)
            .family("Ubuntu Light"),
        text(right)
            .color(theme::SURFACE)
            .size(12.0)
            .family("Ubuntu Light"),
    ))
    .justify_content(Justify::SpaceBetween)
    .gap(12.0)
}

fn format_time(seconds: f32) -> String {
    let seconds = seconds.round().abs() as u64;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    format!("{:02}:{:02}:{:02}", hours, minutes % 60, seconds % 60)
}

pub fn job() -> impl Effect<Data> {
    task(
        |_, sink| async move {
            loop {
                sink.send(());
                tokio::time::sleep(INTERVAL).await;
            }
        },
        |data: &mut Data, _, _| {
            data.version += 1;
            data.batteries = data
                .manager
                .batteries()
                .into_iter()
                .flatten()
                .flatten()
                .collect();
        },
    )
}
