mod hyprland;
mod menu;
mod time;

use gdk4::{glib::object::Cast, prelude::DisplayExt};
use ori_native::prelude::*;

use crate::{hyprland::Hyprland, menu::Menu, time::Time};

fn main() {
    let mut data = Data {
        hyprland: Hyprland::new(),
        time: Time::new(),
        menu: Menu::new(),
    };

    App::new().run(&mut data, ui).unwrap();
}

struct Data {
    hyprland: Hyprland,
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
        map(time::listen_task(), |data: &mut Data, lens| {
            lens(&mut data.time)
        }),
    ))
}

fn bar(data: &Data, monitor_index: usize) -> impl View<Data> + use<> {
    let bar = column((
        map(menu::button(monitor_index), |data: &mut Data, map| {
            map(&mut data.menu)
        }),
        map(
            hyprland::workspaces(&data.hyprland, monitor_index),
            |data: &mut Data, map| map(&mut data.hyprland),
        ),
        map(time::time(&data.time), |data: &mut Data, map| {
            map(&mut data.time)
        }),
    ))
    .padding(12.0)
    .padding_top(20.0)
    .padding_bottom(20.0)
    .background_color(theme::BACKGROUND)
    .justify_contents(Justify::SpaceBetween)
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
    .background_color(Color::hex("#282828"))
}
