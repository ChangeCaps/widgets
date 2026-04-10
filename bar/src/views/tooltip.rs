use ori_native::prelude::*;

pub fn tooltip<T>(
    contents: impl View<T> + 'static,
    tooltip: impl ViewSeq<T> + 'static,
) -> impl View<T> {
    let mut contents = Some(contents);
    let mut tooltip = Some(
        column(tooltip)
            .background(theme::BACKGROUND)
            .border(1.0, Color::BLACK.fade(0.1))
            .corner(8.0)
            .padding(10.0)
            .shadow_color(Color::BLACK.fade(0.4))
            .shadow_radius(8.0)
            .shadow_offset(2.0, 3.0)
            .margin(12.0),
    );

    pressable(move |_, state| {
        gtk4::popover(maybe(contents.take()), maybe(tooltip.take()))
            .position(gtk4::Position::Right)
            .is_open(state.hovered)
    })
}
