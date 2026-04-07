use futures::StreamExt;
use ori_native::prelude::*;

pub struct Data {
    version: u64,
    enabled: bool,
    connection: Option<zbus::Connection>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            version: 0,
            enabled: false,
            connection: None,
        }
    }
}

pub fn icon(data: &Data) -> impl View<Data> + use<> {
    async fn enable_bluetooth(connection: zbus::Connection, enabled: bool) -> zbus::Result<()> {
        let proxy = zbus::Proxy::new(
            &connection,
            "org.bluez",
            "/org/bluez/hci0",
            "org.bluez.Adapter1",
        )
        .await?;

        proxy.set_property("Powered", enabled).await?;

        Ok(())
    }

    memo(data.version, |_| {
        pressable(|data: &Data, state| {
            let mut color = match data.enabled {
                true => theme::ACCENT.fade(0.8),
                false => theme::SURFACE.fade(0.8),
            };

            if state.pressed {
                color = color.fade(0.8);
            } else if state.hovered {
                color = color.fade(0.9);
            }

            transition(color, Ease(0.1), |_, color| {
                image(include_bytes!("icon/bluetooth.svg"))
                    .size(24.0, 24.0)
                    .margin(4.0)
                    .tint(color)
            })
        })
        .on_press(|data: &mut Data| {
            let connection = data.connection.clone();
            let enabled = !data.enabled;

            Action::spawn(async move {
                if let Some(connection) = connection
                    && let Err(err) = enable_bluetooth(connection, enabled).await
                {
                    error!("{err}");
                }
            })
        })
    })
}

pub fn job() -> impl Effect<Data> {
    enum Message {
        Connection(zbus::Connection),
        Enabled(bool),
    }

    async fn job(sink: Sink<Message>) -> zbus::Result<()> {
        let connection = zbus::Connection::system().await?;
        sink.send(Message::Connection(connection.clone()));

        let proxy = zbus::Proxy::new(
            &connection,
            "org.bluez",
            "/org/bluez/hci0",
            "org.bluez.Adapter1",
        )
        .await?;

        let mut powered = proxy.receive_property_changed::<bool>("Powered").await;

        while let Some(powered) = powered.next().await {
            let powered = powered.get().await?;

            sink.send(Message::Enabled(powered))
        }

        Ok(())
    }

    task(
        |_, sink| async move {
            if let Err(err) = job(sink).await {
                error!("{err}");
            }
        },
        |data: &mut Data, _, message| match message {
            Message::Connection(connection) => {
                data.connection = Some(connection);
            }

            Message::Enabled(enabled) => {
                data.version += 1;
                data.enabled = enabled;
            }
        },
    )
}
