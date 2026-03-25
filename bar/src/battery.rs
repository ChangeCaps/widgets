use std::time::Duration;

use ori_native::prelude::*;

pub struct Battery {
    manager: battery::Manager,
    batteries: Vec<battery::Battery>,
}

impl Battery {
    pub fn new() -> battery::Result<Self> {
        Ok(Self {
            manager: battery::Manager::new()?,
            batteries: Vec::new(),
        })
    }
}

pub const INTERVAL: Duration = Duration::from_secs(5);

pub fn battery(battery: &Battery) -> impl View<Battery> + use<> {
    let Some(battery) = battery.batteries.first() else {
        return any(column(()));
    };

    let icon = if battery.state() == battery::State::Charging {
        include_bytes!("icon/battery-charging.svg").as_slice()
    } else {
        include_bytes!("icon/battery.svg").as_slice()
    };

    any(column((
        image(icon).size(28.0, 28.0).tint(Color::BLACK.fade(0.2)),
        text(format!("{:2.0}", battery.state_of_charge().value * 100.0))
            .top(-4.0)
            .color(Color::BLACK.fade(0.2))
            .weight(Weight::BOLD)
            .size(12.0)
            .family("Ubuntu Light"),
    ))
    .align_items(Align::Center))
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
