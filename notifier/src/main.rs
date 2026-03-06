mod bus;
mod dbus;

use ori_native::{Proxied, prelude::*};

use dbus::{Notification, NotifyMessage};

fn main() {
    App::new().run(&mut Data::new(), ui).unwrap();
}

struct Data {
    connection: Option<dbus::Connection>,
    notifications: Vec<Notification>,
}

impl Data {
    fn new() -> Self {
        Self {
            connection: None,
            notifications: Vec::new(),
        }
    }
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    effects((
        (!data.notifications.is_empty()).then(|| {
            let view = column(
                data.notifications
                    .iter()
                    .map(notification)
                    .collect::<Vec<_>>(),
            )
            .gap(10.0);

            layer_shell(view)
                .sizing(Sizing::Content)
                .layer(Layer::Overlay)
                .anchor_top(true)
                .anchor_right(true)
                .margin_top(12)
                .margin_right(12)
        }),
        freeze(|| {
            build_with_context(|cx: &mut Context, _| {
                cx.proxy().spawn(dbus::task(cx.proxy()));

                effects(())
            })
        }),
        receive(|data: &mut Data, message| match message {
            NotifyMessage::Connected(connection) => {
                data.connection = Some(connection);

                Action::new()
            }

            NotifyMessage::Notification(notification) => {
                if let Some(n) = data
                    .notifications
                    .iter_mut()
                    .find(|n| n.id == notification.id)
                {
                    *n = *notification;
                } else {
                    data.notifications.insert(0, *notification);
                }

                Action::rebuild()
            }

            NotifyMessage::Close(id, generation, reason) => {
                let index = data.notifications.iter().position(|n| {
                    let is_id = n.id == id;
                    let is_gen = generation.is_none_or(|g| g == n.generation);

                    is_id && is_gen
                });

                if let Some(index) = index {
                    data.notifications.remove(index);

                    if let Some(ref conn) = data.connection {
                        conn.notification_closed(id, reason);
                    }

                    Action::rebuild()
                } else {
                    Action::new()
                }
            }
        }),
    ))
}

fn notification(notification: &Notification) -> impl View<Data> + use<> {
    let hint_image = notification.hint_image.as_ref().map(|hint_image| {
        row(image(hint_image.clone()).size(75.0, 75.0))
            .corner(8.0)
            .flex(0.0)
            .overflow(Overflow::Hidden)
    });

    let header = row((hint_image, notification_header(notification))).gap(8.0);

    let mut actions = Vec::new();

    for action in &notification.actions {
        let id = notification.id;
        let key = action[0].clone();
        let label = action[1].clone();

        let action = pressable(move |_, state| {
            let color = if state.pressed {
                Color::BLACK.fade(0.05)
            } else if state.hovered {
                Color::WHITE.fade(0.1)
            } else {
                Color::WHITE.fade(0.075)
            };

            let label = label.clone();
            any(transition(color, Ease(0.2), move |color, _| {
                row(text(&label).size(12.0).color(Color::WHITE.fade(0.8)))
                    .justify_contents(Justify::Center)
                    .padding(12.0)
                    .background_color(color)
                    .corner(8.0)
                    .flex(1.0)
            }))
        })
        .on_press(move |data: &mut Data| {
            if let Some(ref conn) = data.connection {
                conn.action_invoked(id, key.clone());
            }
        });

        actions.push(action);
    }

    let actions = row(actions)
        .gap(10.0)
        .justify_contents(Justify::SpaceBetween);

    column((
        header,
        (!notification.actions.is_empty()).then_some(actions),
    ))
    .background_color(theme::BACKGROUND)
    .border_color(theme::OUTLINE)
    .shadow_color(Color::BLACK.fade(0.4))
    .shadow_radius(8.0)
    .shadow_offset(2.0, 3.0)
    .margin(12.0)
    .padding(16.0)
    .border(1.0)
    .corner(8.0)
    .width(400.0)
    .gap(16.0)
}

fn notification_header(notification: &Notification) -> impl View<Data> + use<> {
    let app_line = row((
        notification
            .app_icon
            .as_ref()
            .map(|app_icon| image(app_icon.clone()).size(10.0, 10.0)),
        text(&notification.app_name)
            .size(8.0)
            .color(Color::WHITE.fade(0.5)),
    ))
    .justify_contents(Justify::SpaceBetween)
    .flex(1.0);

    let summary = text(&notification.summary).size(12.0).color(Color::WHITE);

    let time = text(notification.time.format("%H:%M").to_string())
        .size(8.0)
        .color(Color::WHITE.fade(0.5));

    let header = column((row((app_line, time)), summary));

    let body = text(&notification.body)
        .size(10.0)
        .color(Color::WHITE.fade(0.5))
        .wrap(Wrap::Word);

    column((header, (!notification.body.is_empty()).then_some(body)))
        .flex(1.0)
        .overflow(Overflow::Hidden)
}
