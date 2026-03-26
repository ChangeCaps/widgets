use std::time::Duration;

use ori_native::prelude::*;

pub struct Battery {
    version: u64,
    manager: battery::Manager,
    batteries: Vec<battery::Battery>,
}

impl Battery {
    pub fn new() -> battery::Result<Self> {
        Ok(Self {
            version: 0,
            manager: battery::Manager::new()?,
            batteries: Vec::new(),
        })
    }
}

pub const INTERVAL: Duration = Duration::from_secs(2);

pub fn battery(battery: &Battery) -> impl View<Battery> + use<> {
    memo(battery.version, |_| {
        pressable(move |battery: &Battery, state| {
            let Some(battery) = battery.batteries.first() else {
                return any(column(()));
            };

            let charge = battery.state_of_charge().value;
            let icon = if battery.state() == battery::State::Charging {
                include_bytes!("icon/battery-charging.svg")
            } else if charge < 0.25 {
                include_bytes!("icon/battery25.svg")
            } else if charge < 0.50 {
                include_bytes!("icon/battery50.svg")
            } else if charge < 0.75 {
                include_bytes!("icon/battery75.svg")
            } else {
                include_bytes!("icon/battery.svg").as_slice()
            };

            let icon = image(icon).size(28.0, 28.0).tint(theme::ACCENT.fade(0.5));

            let charge = text(format!("charge: {:02.0}%", charge * 100.0,))
                .color(theme::SURFACE)
                .size(12.0)
                .family("Ubuntu Light");

            let time_to_empty = battery.time_to_empty().map(|time| {
                let seconds = time.value.round().abs() as u64;
                let minutes = seconds / 60;
                let hours = minutes / 60;

                text(format!(
                    "empty: {:02}:{:02}:{:02}",
                    hours,
                    minutes % 60,
                    seconds % 60,
                ))
                .color(theme::SURFACE)
                .size(12.0)
                .family("Ubuntu Light")
            });

            let time_to_full = battery.time_to_full().map(|time| {
                let seconds = time.value.round().abs() as u64;
                let minutes = seconds / 60;
                let hours = minutes / 60;

                text(format!(
                    "full: {:02}:{:02}:{:02}",
                    hours,
                    minutes % 60,
                    seconds % 60,
                ))
                .color(theme::SURFACE)
                .size(12.0)
                .family("Ubuntu Light")
            });

            let health = battery.state_of_health().value;
            let health = if health < 0.50 {
                "bad"
            } else if health < 0.75 {
                "okay"
            } else if health < 0.90 {
                "fine"
            } else {
                "good"
            };

            let health = text(format!("health: {health}"))
                .color(theme::SURFACE)
                .size(12.0)
                .family("Ubuntu Light");

            any(gtk4::popover(
                transform(icon).rotate(-90.0),
                column((charge, health, time_to_empty, time_to_full))
                    .gap(4.0)
                    .background(theme::BACKGROUND)
                    .border(1.0, Color::BLACK.fade(0.1))
                    .corner(8.0)
                    .padding(10.0),
            )
            .position(gtk4::Position::Right)
            .is_open(state.hovered))
        })
    })
}

pub fn listen_task() -> impl Effect<Battery> {
    task(
        |_, sink| async move {
            loop {
                sink.send(());
                tokio::time::sleep(INTERVAL).await;
            }
        },
        |battery: &mut Battery, _, _| {
            battery.version += 1;
            battery.batteries = battery
                .manager
                .batteries()
                .into_iter()
                .flatten()
                .flatten()
                .collect();
        },
    )
}
