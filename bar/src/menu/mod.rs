use std::collections::HashSet;

use ori_native::prelude::*;
use serde::Deserialize;

mod feed;
mod power;

pub struct Data {
    open: HashSet<usize>,
    feed: feed::Data,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(flatten)]
    feed: feed::Config,
}

impl Data {
    pub fn new(config: Config) -> Self {
        Self {
            open: HashSet::new(),
            feed: feed::Data::new(config.feed),
        }
    }

    pub fn is_open(&self, monitor_index: usize) -> bool {
        self.open.contains(&monitor_index)
    }
}

pub fn button(monitor_index: usize) -> impl View<Data> + use<> {
    pressable(move |menu: &Data, state| {
        let mut color = match menu.open.contains(&monitor_index) {
            true => theme::ACCENT,
            false => theme::SURFACE,
        };

        if state.pressed {
            color = color.fade(0.6);
        } else if state.hovered {
            color = color.fade(0.8);
        }

        transition(color, Ease(0.1), |_, color| {
            image(include_bytes!("../icon/menu.svg"))
                .tint(color)
                .size(28.0, 28.0)
        })
    })
    .on_press(move |menu| {
        if !menu.open.insert(monitor_index) {
            menu.open.remove(&monitor_index);
        }
    })
}

pub fn contents(menu: &Data, monitor_index: usize) -> impl View<Data> + use<> {
    const WIDTH: f32 = 500.0;

    let width = match menu.is_open(monitor_index) {
        true => WIDTH,
        false => 0.0,
    };

    transition(width, Ease(0.4), |menu: &Data, width| {
        let contents = column((
            power::power(),
            map(feed::feed(&menu.feed), |menu: &mut Data, map| {
                map(&mut menu.feed)
            }),
        ))
        .width(WIDTH)
        .justify_content(Justify::Start)
        .padding(20.0)
        .gap(32.0)
        .flex(0.0);

        row(vscroll(contents))
            .width(width)
            .justify_content(Justify::Start)
            .align_items(Align::Start)
            .overflow(Overflow::Hidden)
    })
}

pub fn job() -> impl Effect<Data> {
    effects(map(feed::job(), |data: &mut Data, map| map(&mut data.feed)))
}
