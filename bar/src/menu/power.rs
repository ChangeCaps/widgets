use std::process;

use ori_native::prelude::*;

pub fn power<T>() -> impl View<T> {
    freeze(|| {
        let shutdown = button(include_bytes!("../icon/system-off.svg"), || {
            let _ = process::Command::new("shutdown").arg("now").spawn();
        });

        let reboot = button(include_bytes!("../icon/system-restart.svg"), || {
            let _ = process::Command::new("reboot").spawn();
        });

        let logout = button(include_bytes!("../icon/system-logout.svg"), || {
            let _ = process::Command::new("hyprctl")
                .arg("dispatch")
                .arg("exit")
                .spawn();
        });

        row((shutdown, reboot, logout))
            .justify_content(Justify::Center)
            .background(theme::MANTLE)
            .corner(20.0)
    })
}

fn button<T>(icon: &'static [u8], mut on_press: impl FnMut() + 'static) -> impl View<T> {
    pressable(move |_, state| {
        let color = if state.pressed {
            theme::ACCENT.darken(0.2)
        } else if state.hovered {
            theme::ACCENT.darken(0.1)
        } else {
            theme::ACCENT
        };

        transition(color, Ease(0.1), move |_, color| {
            row(image(icon).size(48.0, 48.0).tint(color)).padding(16.0)
        })
    })
    .on_press(move |_| {
        on_press();
        Action::new()
    })
}
