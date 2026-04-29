use std::collections::HashMap;

use futures::StreamExt;
use notify_rust::Notification;
use ori_native::prelude::*;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

use crate::views::tooltip_body;

pub struct Data {
    version: u64,
    enabled: bool,
    discovering: bool,
    connection: Option<zbus::Connection>,
    devices: HashMap<OwnedObjectPath, Device>,
}

struct Device {
    addr: String,
    name: String,
    connected: bool,
    trusted: bool,
    paired: bool,
}

impl Data {
    pub fn new() -> Self {
        Self {
            version: 0,
            enabled: false,
            discovering: false,
            connection: None,
            devices: HashMap::new(),
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
                true => theme::PRIMARY.fade(0.8),
                false => theme::SURFACE.fade(0.8),
            };

            if state.pressed {
                color = color.fade(0.8);
            } else if state.hovered {
                color = color.fade(0.9);
            }

            let mut devices = data
                .devices
                .values()
                .filter(|device| device.connected)
                .map(|device| {
                    text(&device.name)
                        .color(theme::SURFACE)
                        .size(12.0)
                        .family("Ubuntu Light")
                })
                .collect::<Vec<_>>();

            if devices.is_empty() {
                devices.push(
                    text("No devices connected")
                        .color(theme::SURFACE)
                        .size(12.0)
                        .family("Ubuntu Light"),
                );
            }

            gtk4::popover(
                transition(color, Ease(0.1), move |data: &Data, color| {
                    let icon: &[u8] = match data.devices.values().any(|device| device.connected) {
                        true => include_bytes!("icon/bluetooth-connected.svg"),
                        false => include_bytes!("icon/bluetooth.svg"),
                    };

                    image(icon).size(24.0, 24.0).margin(4.0).tint(color)
                }),
                tooltip_body(column(devices).gap(8.0)),
            )
            .is_open(state.hovered)
            .position(gtk4::Position::Right)
        })
        .on_press(|data: &mut Data| {
            let connection = data.connection.clone();
            let enabled = !data.enabled;

            Action::spawn(async move {
                if let Some(connection) = connection
                    && let Err(err) = enable_bluetooth(connection, enabled).await
                {
                    notify_error(&err).await;
                }
            })
        })
    })
}

pub fn menu(data: &Data) -> impl View<Data> + use<> {
    async fn enable_discovery(connection: zbus::Connection, enabled: bool) -> zbus::Result<()> {
        let proxy = zbus::Proxy::new(
            &connection,
            "org.bluez",
            "/org/bluez/hci0",
            "org.bluez.Adapter1",
        )
        .await?;

        if enabled {
            proxy.call_method("StartDiscovery", &()).await?;
        } else {
            proxy.call_method("StopDiscovery", &()).await?;
        }

        Ok(())
    }

    memo(data.version, |data: &Data| {
        let discover = pressable(|data: &Data, state| {
            let mut color = match data.discovering {
                true => theme::GREEN,
                false => theme::SURFACE,
            };

            if state.pressed {
                color = color.fade(0.8);
            } else if state.hovered {
                color = color.fade(0.9);
            }

            gtk4::popover(
                image(include_bytes!("icon/search.svg"))
                    .size(28.0, 28.0)
                    .tint(color),
                tooltip_body(
                    text(match data.discovering {
                        true => "Stop device discovery",
                        false => "Start device discovery",
                    })
                    .color(theme::SURFACE)
                    .size(12.0)
                    .family("Ubuntu Light"),
                ),
            )
            .is_open(state.hovered)
        })
        .on_press(|data: &mut Data| {
            let connection = data.connection.clone();
            let enabled = !data.discovering;

            Action::spawn(async move {
                if let Some(connection) = connection
                    && let Err(err) = enable_discovery(connection, enabled).await
                {
                    notify_error(&err).await;
                }
            })
        });

        let mut devices = data.devices.iter().collect::<Vec<_>>();

        devices.sort_by(|(_, a), (_, b)| {
            let paired = b.paired.cmp(&a.paired);
            let name = a.name.cmp(&b.name);

            paired.then(name)
        });

        column((
            row((
                text("Bluetooth devices")
                    .size(18.0)
                    .family("Inter")
                    .weight(Weight::BOLD)
                    .color(theme::SURFACE),
                discover,
            ))
            .justify_content(Justify::SpaceBetween)
            .align_items(Align::Center),
            devices
                .into_iter()
                .map(|(path, device)| (path.clone(), self::device(path, device)))
                .collect::<Keyed<_, _>>(),
        ))
        .gap(12.0)
    })
}

fn device(path: &OwnedObjectPath, device: &Device) -> impl View<Data> + use<> {
    row((
        row(text(&device.name)
            .size(12.0)
            .color(Color::BLACK.fade(0.8))
            .family("Inter")
            .weight(Weight::SEMI_BOLD))
        .overflow(Overflow::Hidden),
        row((
            connect_button(path, device),
            pair_button(path, device),
            trust_button(path, device),
        ))
        .gap(12.0),
    ))
    .justify_content(Justify::SpaceBetween)
    .align_items(Align::Center)
    .background(theme::ROSE)
    .corner(8.0)
    .padding(8.0)
    .padding_right(16.0)
    .padding_left(16.0)
    .gap(8.0)
    .shadow_color(Color::BLACK.fade(0.4))
    .shadow_radius(8.0)
    .shadow_offset(2.0, 2.0)
}

fn connect_button(path: &OwnedObjectPath, device: &Device) -> impl View<Data> + use<> {
    async fn set_connected(
        connection: zbus::Connection,
        path: OwnedObjectPath,
        connected: bool,
    ) -> zbus::Result<()> {
        let proxy = zbus::Proxy::new(&connection, "org.bluez", path, "org.bluez.Device1").await?;

        if connected {
            proxy.call_method("Connect", &()).await?;
        } else {
            proxy.call_method("Disconnect", &()).await?;
        }

        Ok(())
    }

    button(
        match device.connected {
            true => include_bytes!("icon/bluetooth-connected.svg"),
            false => include_bytes!("icon/bluetooth.svg"),
        },
        match device.connected {
            true => theme::PRIMARY,
            false => Color::BLACK.fade(0.3),
        },
        match device.connected {
            true => "Disconnect",
            false => "Connect",
        },
        {
            let path = path.clone();
            let connected = !device.connected;

            move |data: &mut Data| {
                let path = path.clone();
                let connection = data.connection.clone();

                Action::spawn(async move {
                    if let Some(connection) = connection
                        && let Err(err) = set_connected(connection, path, connected).await
                    {
                        notify_error(&err).await;
                    }
                })
            }
        },
    )
}

fn pair_button(path: &OwnedObjectPath, device: &Device) -> impl View<Data> + use<> {
    async fn set_paired(
        connection: zbus::Connection,
        path: OwnedObjectPath,
        paired: bool,
    ) -> zbus::Result<()> {
        if paired {
            let proxy = zbus::Proxy::new(
                &connection,
                "org.bluez",
                path, // device path
                "org.bluez.Device1",
            )
            .await?;

            proxy.call_method("Pair", &()).await?;
        } else {
            let proxy = zbus::Proxy::new(
                &connection,
                "org.bluez",
                "/org/bluez/hci0",
                "org.bluez.Adapter1",
            )
            .await?;

            proxy.call_method("RemoveDevice", &(path)).await?;
        }

        Ok(())
    }

    button(
        match device.paired {
            true => include_bytes!("icon/link.svg"),
            false => include_bytes!("icon/link-break.svg"),
        },
        match device.paired {
            true => Color::BLACK.fade(0.8),
            false => Color::BLACK.fade(0.3),
        },
        match device.paired {
            true => "Unpair",
            false => "Pair",
        },
        {
            let path = path.clone();
            let paired = !device.paired;

            move |data: &mut Data| {
                let path = path.clone();
                let connection = data.connection.clone();

                Action::spawn(async move {
                    if let Some(connection) = connection
                        && let Err(err) = set_paired(connection, path, paired).await
                    {
                        notify_error(&err).await;
                    }
                })
            }
        },
    )
}

fn trust_button(path: &OwnedObjectPath, device: &Device) -> impl View<Data> + use<> {
    async fn set_trusted(
        connection: zbus::Connection,
        path: OwnedObjectPath,
        trusted: bool,
    ) -> zbus::Result<()> {
        let proxy = zbus::Proxy::new(&connection, "org.bluez", path, "org.bluez.Device1").await?;
        proxy.set_property("Trusted", trusted).await?;

        Ok(())
    }

    button(
        match device.trusted {
            true => include_bytes!("icon/heart.svg"),
            false => include_bytes!("icon/heart-break.svg"),
        },
        match device.trusted {
            true => Color::BLACK.fade(0.8),
            false => Color::BLACK.fade(0.3),
        },
        match device.trusted {
            true => "Untrust",
            false => "Trust",
        },
        {
            let path = path.clone();
            let trusted = !device.trusted;

            move |data: &mut Data| {
                let path = path.clone();
                let connection = data.connection.clone();

                Action::spawn(async move {
                    if let Some(connection) = connection
                        && let Err(err) = set_trusted(connection, path, trusted).await
                    {
                        notify_error(&err).await;
                    }
                })
            }
        },
    )
}

fn button(
    icon: &'static [u8],
    color: Color,
    tooltip: &'static str,
    on_press: impl FnMut(&mut Data) -> Action + 'static,
) -> impl View<Data> {
    pressable({
        move |_, state| {
            let mut color = color;

            if state.pressed {
                color = color.fade(0.6);
            } else if state.hovered {
                color = color.fade(0.8);
            }

            gtk4::popover(
                transition(color, Ease(0.1), move |_, color| {
                    image(icon).size(24.0, 24.0).tint(color)
                }),
                column(
                    text(tooltip)
                        .size(12.0)
                        .color(Color::BLACK.fade(0.8))
                        .family("Inter")
                        .weight(Weight::SEMI_BOLD),
                )
                .background(theme::ROSE)
                .border(1.0, Color::BLACK.fade(0.1))
                .corner(8.0)
                .padding(10.0)
                .shadow_color(Color::BLACK.fade(0.4))
                .shadow_radius(8.0)
                .shadow_offset(2.0, 3.0)
                .margin(12.0),
            )
            .is_open(state.hovered)
        }
    })
    .on_press(on_press)
}

async fn notify_error(error: &zbus::Error) {
    let mut message = error.to_string();

    if let Some(description) = error.description() {
        message = description.to_string();
    }

    if message.contains("AuthenticationFailed") {
        message = String::from("Pairing failed.");
    } else if message.contains("ConnectionAttemptFailed")
        || message.contains("br-connection-create-socket")
    {
        message = String::from("Could not connect to device.");
    } else if message.contains("br-connection-refused") {
        message = String::from("Connection refused.");
    } else if message.contains("AlreadyExists") {
        message = String::from("Device is already paired.");
    }

    let _ = Notification::new()
        .appname("Bluetooth manager")
        .summary("Bluetooth Error")
        .body(&message)
        .show_async()
        .await;
}

pub fn job() -> impl Effect<Data> {
    enum Message {
        Connection(zbus::Connection),
        Enabled(bool),
        Discovering(bool),

        Added {
            path: OwnedObjectPath,
            addr: String,
            name: String,
        },

        Removed {
            path: OwnedObjectPath,
        },

        Connected {
            path: OwnedObjectPath,
            connected: bool,
        },

        Trusted {
            path: OwnedObjectPath,
            trusted: bool,
        },

        Paired {
            path: OwnedObjectPath,
            paired: bool,
        },
    }

    struct Agent;

    #[zbus::interface(name = "org.bluez.Agent1")]
    impl Agent {}

    async fn job(sink: Sink<Message>) -> zbus::Result<()> {
        let connection = zbus::Connection::system().await?;
        sink.send(Message::Connection(connection.clone()));

        let agent_path = ObjectPath::from_static_str("/org/hjalte/bluez/agent")?;

        connection.object_server().at(&agent_path, Agent).await?;

        let proxy = zbus::Proxy::new(
            &connection,
            "org.bluez",
            "/org/bluez",
            "org.bluez.AgentManager1",
        )
        .await?;

        proxy
            .call_method("RegisterAgent", &(&agent_path, "NoInputNoOutput"))
            .await?;

        proxy
            .call_method("RequestDefaultAgent", &(&agent_path))
            .await?;

        let proxy = zbus::Proxy::new(
            &connection,
            "org.bluez",
            "/org/bluez/hci0",
            "org.bluez.Adapter1",
        )
        .await?;

        let mut powered = proxy.receive_property_changed::<bool>("Powered").await;
        let mut discovering = proxy.receive_property_changed::<bool>("Discovering").await;

        let manager = zbus::fdo::ObjectManagerProxy::new(&connection, "org.bluez", "/").await?;

        let mut devices = HashMap::new();

        for (path, interfaces) in manager.get_managed_objects().await? {
            if interfaces.contains_key("org.bluez.Device1") {
                let handle = spawn_device_handler(sink.clone(), connection.clone(), path.clone());
                devices.insert(path, handle);
            }
        }

        let mut added = manager.receive_interfaces_added().await?;
        let mut removed = manager.receive_interfaces_removed().await?;

        loop {
            tokio::select! {
                Some(powered) = powered.next() => {
                    let powered = powered.get().await?;
                    sink.send(Message::Enabled(powered));
                }

                Some(discovering) = discovering.next() => {
                    let discovering = discovering.get().await?;
                    sink.send(Message::Discovering(discovering));
                }

                Some(added) = added.next() => {
                    let added = added.args()?;

                    if added.interfaces_and_properties().contains_key("org.bluez.Device1") {
                        let handle = spawn_device_handler(
                            sink.clone(),
                            connection.clone(),
                            added.object_path.clone().into(),
                        );

                        devices.insert(added.object_path.into(), handle);
                    }
                }

                Some(removed) = removed.next() => {
                    let removed = removed.args()?;

                    if removed.interfaces().iter().any(|x| x == "org.bluez.Device1")
                        && let Some(handle) = devices.remove(removed.object_path())
                    {
                        handle.abort();

                        sink.send(Message::Removed {
                            path: removed.object_path.into(),
                        });
                    }
                }
            }
        }
    }

    fn spawn_device_handler(
        sink: Sink<Message>,
        connection: zbus::Connection,
        path: zbus::zvariant::OwnedObjectPath,
    ) -> tokio::task::JoinHandle<()> {
        let future = handle_device(sink, connection, path);

        tokio::spawn(async move {
            if let Err(err) = future.await {
                error!("{err}");
            }
        })
    }

    async fn handle_device(
        sink: Sink<Message>,
        connection: zbus::Connection,
        path: zbus::zvariant::OwnedObjectPath,
    ) -> zbus::Result<()> {
        let proxy = zbus::Proxy::new(
            &connection,
            "org.bluez",
            path.clone(), // use the path
            "org.bluez.Device1",
        )
        .await?;

        let addr = proxy.get_property::<String>("Address").await?;

        let Ok(name) = proxy.get_property::<String>("Name").await else {
            return Ok(());
        };

        sink.send(Message::Added {
            path: path.clone(),
            addr,
            name,
        });

        let mut connected = proxy.receive_property_changed::<bool>("Connected").await;
        let mut trusted = proxy.receive_property_changed::<bool>("Trusted").await;
        let mut paired = proxy.receive_property_changed::<bool>("Paired").await;

        loop {
            tokio::select! {
                Some(connected) = connected.next() => {
                    let connected = connected.get().await?;

                    sink.send(Message::Connected {
                        path: path.clone(),
                        connected,
                    });
                }

                Some(trusted) = trusted.next() => {
                    let trusted = trusted.get().await?;

                    sink.send(Message::Trusted {
                        path: path.clone(),
                        trusted,
                    });
                }

                Some(paired) = paired.next() => {
                    let paired = paired.get().await?;

                    sink.send(Message::Paired {
                        path: path.clone(),
                        paired,
                    });
                }
            }
        }
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

            Message::Discovering(discovering) => {
                data.version += 1;
                data.discovering = discovering;
            }

            Message::Added { path, addr, name } => {
                let device = Device {
                    addr,
                    name,
                    connected: false,
                    trusted: false,
                    paired: false,
                };

                data.version += 1;
                data.devices.insert(path, device);
            }

            Message::Removed { path } => {
                data.version += 1;
                data.devices.remove(&path);
            }

            Message::Connected { path, connected } => {
                data.version += 1;

                if let Some(device) = data.devices.get_mut(&path) {
                    device.connected = connected;
                }
            }

            Message::Trusted { path, trusted } => {
                data.version += 1;

                if let Some(device) = data.devices.get_mut(&path) {
                    device.trusted = trusted;
                }
            }

            Message::Paired { path, paired } => {
                data.version += 1;

                if let Some(device) = data.devices.get_mut(&path) {
                    device.paired = paired;
                }
            }
        },
    )
}
