mod battery;
mod hyprland;
mod menu;
mod time;

use gdk4::{glib::object::Cast, prelude::DisplayExt};
use ori_native::prelude::*;

use crate::{battery::Battery, hyprland::Hyprland, menu::Menu, time::Time};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut data = Data {
        hyprland: Hyprland::new(),
        battery: Battery::new()?,
        time: Time::new(),
        menu: Menu::new(),
    };

    App::new().run(&mut data, ui)?;

    Ok(())
}

struct Data {
    hyprland: Hyprland,
    battery: Battery,
    time: Time,
    menu: Menu,
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    let display = gdk4::Display::default().unwrap();

    let shells = display
        .monitors()
        .into_iter()
        .enumerate()
        .map(|(i, monitor)| {
            let monitor = monitor.unwrap().dynamic_cast().unwrap();

            layer_shell(bar(data, i))
                .monitor(Some(monitor))
                .anchor_left(true)
                .anchor_top(true)
                .anchor_bottom(true)
                .sizing(Sizing::Content)
                .namespace("bar-widget")
        })
        .collect::<Vec<_>>();

    effects((
        shells,
        map(hyprland::listen_task(), |data: &mut Data, lens| {
            lens(&mut data.hyprland)
        }),
        map(battery::listen_task(), |data: &mut Data, lens| {
            lens(&mut data.battery)
        }),
        map(time::listen_task(), |data: &mut Data, lens| {
            lens(&mut data.time)
        }),
    ))
}

fn bar(data: &Data, monitor_index: usize) -> impl View<Data> + use<> {
    let bar = column((
        // hyprland workspaces
        map(
            hyprland::workspaces(&data.hyprland, monitor_index),
            |data: &mut Data, map| map(&mut data.hyprland),
        ),
        // menu button
        column(map(menu::button(monitor_index), |data: &mut Data, map| {
            map(&mut data.menu)
        }))
        .position(Position::Absolute)
        .top(20.0),
        // bottom column
        column((
            map(battery::battery(&data.battery), |data: &mut Data, map| {
                map(&mut data.battery)
            }),
            map(time::time(&data.time), |data: &mut Data, map| {
                map(&mut data.time)
            }),
        ))
        .position(Position::Absolute)
        .align_items(Align::Center)
        .bottom(20.0)
        .gap(20.0),
    ))
    .width(52.0)
    .background(theme::BACKGROUND)
    .justify_content(Justify::Center)
    .align_items(Align::Center)
    .shadow_color(Color::BLACK.fade(0.4))
    .shadow_radius(8.0);

    row((
        map(
            menu::contents(&data.menu, monitor_index),
            |data: &mut Data, map| map(&mut data.menu),
        ),
        bar,
    ))
    .background(Color::hex("#282828"))
}
