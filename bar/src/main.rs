mod battery;
mod bluetooth;
mod hyprland;
mod menu;
mod network;
mod time;

use std::{env, fs, path::Path};

use gdk4::{glib::object::Cast, prelude::DisplayExt};
use ori_native::prelude::*;
use serde::Deserialize;

#[derive(Default, Deserialize)]
struct Config {
    #[serde(flatten)]
    menu: menu::Config,
}

fn main() -> eyre::Result<()> {
    App::init_log();

    let config = read_config()?;

    let mut data = Data {
        hyprland: hyprland::Data::new(),
        bluetooth: bluetooth::Data::new(),
        battery: battery::Data::new()?,
        network: network::Data::new(),
        time: time::Data::new(),
        menu: menu::Data::new(config.menu),
    };

    App::new().run(&mut data, ui)?;

    Ok(())
}

fn read_config() -> eyre::Result<Config> {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("~"));
    let path = Path::new(&home).join(".config/widgets/widgets.toml");

    let Ok(config) = fs::read(path) else {
        warn!("failed to load config");
        return Ok(Default::default());
    };

    Ok(toml::de::from_slice(&config)?)
}

struct Data {
    hyprland: hyprland::Data,
    bluetooth: bluetooth::Data,
    battery: battery::Data,
    network: network::Data,
    time: time::Data,
    menu: menu::Data,
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
        map(hyprland::job(), |data: &mut Data, map| {
            map(&mut data.hyprland)
        }),
        map(bluetooth::job(), |data: &mut Data, map| {
            map(&mut data.bluetooth)
        }),
        map(
            battery::job(),
            |data: &mut Data, map| map(&mut data.battery),
        ),
        map(
            network::job(),
            |data: &mut Data, map| map(&mut data.network),
        ),
        map(time::job(), |data: &mut Data, map| map(&mut data.time)),
        map(menu::job(), |data: &mut Data, map| map(&mut data.menu)),
    ))
}

fn bar(data: &Data, monitor_index: usize) -> impl View<Data> + use<> {
    let bar = column((
        // menu button
        column((
            map(menu::button(monitor_index), |data: &mut Data, map| {
                map(&mut data.menu)
            }),
            column(())
                .background(theme::MANTLE)
                .justify_content(Justify::Start)
                .align_items(Align::Center)
                .padding_top(8.0)
                .padding_bottom(8.0)
                .corner(8.0)
                .width(32.0)
                .flex(1.0),
        ))
        .justify_content(Justify::Start)
        .align_items(Align::Center)
        .flex(1.0)
        .flex_basis(0.0)
        .gap(32.0),
        // hyprland workspaces
        map(
            hyprland::workspaces(&data.hyprland, monitor_index),
            |data: &mut Data, map| map(&mut data.hyprland),
        ),
        // bottom column
        column((
            column((
                map(bluetooth::icon(&data.bluetooth), |data: &mut Data, map| {
                    map(&mut data.bluetooth)
                }),
                map(network::icon(&data.network), |data: &mut Data, map| {
                    map(&mut data.network)
                }),
                data.battery.show().then(|| {
                    map(battery::icon(&data.battery), |data: &mut Data, map| {
                        map(&mut data.battery)
                    })
                }),
            ))
            .background(theme::MANTLE)
            .justify_content(Justify::End)
            .align_items(Align::Center)
            .padding_top(8.0)
            .padding_bottom(8.0)
            .corner(8.0)
            .width(32.0)
            .gap(4.0)
            .flex(1.0),
            map(time::time(&data.time), |data: &mut Data, map| {
                map(&mut data.time)
            }),
        ))
        .justify_content(Justify::End)
        .align_items(Align::Center)
        .flex(1.0)
        .flex_basis(0.0)
        .gap(32.0),
    ))
    .background(theme::BACKGROUND)
    .justify_content(Justify::Center)
    .align_items(Align::Center)
    .shadow_color(Color::BLACK.fade(0.4))
    .shadow_radius(8.0)
    .padding_top(20.0)
    .padding_bottom(20.0)
    .width(52.0)
    .gap(48.0);

    row((
        map(
            menu::contents(&data.menu, monitor_index),
            |data: &mut Data, map| map(&mut data.menu),
        ),
        bar,
    ))
    .background(Color::hex("#282828"))
}
