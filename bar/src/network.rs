use futures::stream::StreamExt;
use ori_native::prelude::*;

pub struct Data {
    version: u64,
    id: Option<String>,
    connectivity: Connectivity,
    device_kind: Option<DeviceKind>,
}

#[derive(Clone, Debug)]
enum Connectivity {
    None,
    Full,
}

#[derive(Clone, Debug)]
enum DeviceKind {
    Wifi,
    Ethernet,
}

impl Data {
    pub fn new() -> Self {
        Self {
            version: 0,
            id: None,
            connectivity: Connectivity::None,
            device_kind: None,
        }
    }
}

pub fn icon(data: &Data) -> impl View<Data> + use<> {
    memo(data.version, |_| {
        pressable(|data: &Data, state| {
            let icon: &[u8] = match data.device_kind {
                Some(DeviceKind::Ethernet) => match data.connectivity {
                    Connectivity::None => include_bytes!("icon/network-off.svg"),
                    Connectivity::Full => include_bytes!("icon/network.svg"),
                },

                Some(DeviceKind::Wifi) | None => match data.connectivity {
                    Connectivity::None => include_bytes!("icon/wifi-off.svg"),
                    Connectivity::Full => include_bytes!("icon/wifi.svg"),
                },
            };

            let id = data.id.as_ref().map(|id| {
                text(id)
                    .color(theme::SURFACE)
                    .size(12.0)
                    .family("Ubuntu Light")
            });

            gtk4::popover(
                image(icon)
                    .size(24.0, 24.0)
                    .margin(4.0)
                    .tint(theme::ROSE.fade(0.8)),
                column(id)
                    .gap(4.0)
                    .background(theme::BACKGROUND)
                    .border(1.0, Color::BLACK.fade(0.1))
                    .corner(8.0)
                    .padding(10.0)
                    .shadow_color(Color::BLACK.fade(0.4))
                    .shadow_radius(8.0)
                    .shadow_offset(2.0, 3.0)
                    .margin(12.0),
            )
            .position(gtk4::Position::Right)
            .is_open(state.hovered)
        })
    })
}

pub fn job() -> impl Effect<Data> {
    enum Message {
        Id(String),
        Connectivity(Connectivity),
        DeviceKind(DeviceKind),
    }

    async fn job(sink: Sink<Message>) -> zbus::Result<()> {
        let connection = zbus::Connection::system().await?;

        let proxy = zbus::Proxy::new(
            &connection,
            "org.freedesktop.NetworkManager",
            "/org/freedesktop/NetworkManager",
            "org.freedesktop.NetworkManager",
        )
        .await?;

        let mut primary_connection = proxy
            .receive_property_changed::<zbus::zvariant::OwnedObjectPath>("PrimaryConnection")
            .await;

        let mut connectivity = proxy.receive_property_changed::<u32>("Connectivity").await;

        loop {
            tokio::select! {
                Some(path) = primary_connection.next() => {
                    let path = path.get().await?;
                    get_device_kind(&sink, &connection, path).await?;
                }

                Some(connectivity) = connectivity.next() => {
                    let connectivity = connectivity.get().await?;

                    let connectivity = match connectivity {
                        4 => Connectivity::Full,
                        _ => Connectivity::None,
                    };

                    sink.send(Message::Connectivity(connectivity));
                }
            }
        }
    }

    async fn get_device_kind(
        sink: &Sink<Message>,
        connection: &zbus::Connection,
        path: zbus::zvariant::OwnedObjectPath,
    ) -> zbus::Result<()> {
        let proxy = zbus::Proxy::new(
            connection,
            "org.freedesktop.NetworkManager",
            path,
            "org.freedesktop.NetworkManager.Connection.Active",
        )
        .await?;

        let id = proxy.get_property::<String>("Id").await?;
        sink.send(Message::Id(id));

        let devices = proxy
            .get_property::<Vec<zbus::zvariant::OwnedObjectPath>>("Devices")
            .await?;

        let device = devices.first().unwrap();

        let proxy = zbus::Proxy::new(
            connection,
            "org.freedesktop.NetworkManager",
            device,
            "org.freedesktop.NetworkManager.Device",
        )
        .await?;

        let device_type = proxy.get_property::<u32>("DeviceType").await?;

        let device_kind = match device_type {
            1 => DeviceKind::Ethernet,
            _ => DeviceKind::Wifi,
        };

        sink.send(Message::DeviceKind(device_kind));

        Ok(())
    }

    task(
        |_, sink| async move {
            if let Err(err) = job(sink).await {
                error!("{err}");
            }
        },
        |data: &mut Data, _, message| match message {
            Message::Id(id) => {
                data.version += 1;
                data.id = Some(id);
            }

            Message::Connectivity(connectivity) => {
                data.version += 1;
                data.connectivity = connectivity;
            }

            Message::DeviceKind(device_kind) => {
                data.version += 1;
                data.device_kind = Some(device_kind);
            }
        },
    )
}
