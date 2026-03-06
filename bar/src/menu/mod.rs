use std::collections::HashSet;

use ori_native::prelude::*;

pub struct Menu {
    open: HashSet<usize>,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            open: HashSet::new(),
        }
    }

    pub fn is_open(&self, monitor_index: usize) -> bool {
        self.open.contains(&monitor_index)
    }
}

pub fn button(monitor_index: usize) -> impl View<Menu> + use<> {
    pressable(move |menu: &Menu, state| {
        let mut color = match menu.open.contains(&monitor_index) {
            true => theme::ACCENT,
            false => theme::SURFACE,
        };

        if state.pressed {
            color = color.fade(0.6);
        } else if state.hovered {
            color = color.fade(0.8);
        }

        transition(color, Ease(0.1), |color, _| {
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

pub fn contents(menu: &Menu, monitor_index: usize) -> impl View<Menu> + use<> {
    const WIDTH: f32 = 500.0;

    let width = match menu.is_open(monitor_index) {
        true => WIDTH,
        false => 0.0,
    };

    transition(width, Ease(0.4), |width, _| {
        let contents = column(())
            .width(WIDTH)
            .justify_contents(Justify::Center)
            .align_items(Align::Center)
            .flex(0.0);

        row(contents).width(width).overflow(Overflow::Hidden)
    })
}
