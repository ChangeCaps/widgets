use ori_native::prelude::*;

use crate::{Data, bluetooth, feed, power};

pub fn button(monitor_index: usize) -> impl View<Data> + use<> {
    pressable(move |data: &Data, state| {
        let mut color = match data.open.contains(&monitor_index) {
            true => theme::ACCENT,
            false => theme::SURFACE,
        };

        if state.pressed {
            color = color.fade(0.6);
        } else if state.hovered {
            color = color.fade(0.8);
        }

        transition(color, Ease(0.1), |_, color| {
            image(include_bytes!("icon/menu.svg"))
                .tint(color)
                .size(28.0, 28.0)
        })
    })
    .on_press(move |data| {
        if !data.open.insert(monitor_index) {
            data.open.remove(&monitor_index);
        }
    })
}

pub fn contents(data: &Data, monitor_index: usize) -> impl View<Data> + use<> {
    const WIDTH: f32 = 500.0;

    let width = match data.open.contains(&monitor_index) {
        true => WIDTH,
        false => 0.0,
    };

    transition(width, Ease(0.4), |data: &Data, width| {
        let contents = column((
            power::menu(),
            map(feed::menu(&data.feed), |data: &mut Data, map| {
                map(&mut data.feed)
            }),
            map(bluetooth::menu(&data.bluetooth), |data: &mut Data, map| {
                map(&mut data.bluetooth)
            }),
        ))
        .width(WIDTH)
        .justify_content(Justify::Start)
        .padding(20.0)
        .gap(32.0)
        .flex(0.0);

        row(vscroll(contents).height(Fract(1.0)))
            .width(width)
            .justify_content(Justify::Start)
            .align_items(Align::Start)
            .overflow(Overflow::Hidden)
    })
}
