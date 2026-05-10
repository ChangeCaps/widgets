use ori_native::prelude::*;

pub fn tooltip<T, V>(
    contents: impl View<T> + 'static,
    mut tooltip: impl FnMut(&T) -> V + 'static,
) -> impl View<T>
where
    V: ViewSeq<T> + 'static,
{
    let mut contents = Some(contents);

    pressable(move |data, state| {
        popup(
            maybe(contents.take()),
            state.hovered.then(|| tooltip_body(tooltip(data))),
        )
        .side(Side::Right)
    })
}

pub fn tooltip_body<T>(contents: impl ViewSeq<T> + 'static) -> impl View<T> + 'static {
    column(contents)
        .background(theme::BACKGROUND)
        .border(1.0, Color::BLACK.fade(0.1))
        .corner(8.0)
        .padding(10.0)
        .shadow_color(Color::BLACK.fade(0.4))
        .shadow_radius(8.0)
        .shadow_offset(2.0, 3.0)
        .margin(12.0)
}
