use std::{
    collections::VecDeque,
    fs,
    io::Cursor,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering::SeqCst},
    },
    thread,
    time::Duration,
};

use chrono::{DateTime, Local};
use dbus::{
    MessageType,
    arg::{PropMap, RefArg, cast, prop_cast},
    blocking::stdintf::org_freedesktop_dbus::RequestNameReply,
    channel::{MatchingReceiver, Sender},
    message::{MatchRule, SignalArgs},
};
use dbus_crossroads::Crossroads;
use image::{DynamicImage, ImageBuffer, ImageFormat};
use ori_native::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum CloseReason {
    Expired,
    Dismissed,
    Notified,
    Undefined,
}

#[derive(Debug)]
pub struct Notification {
    pub id: u32,
    pub generation: u32,
    pub app_name: String,
    pub app_icon: Option<Vec<u8>>,
    pub hint_image: Option<Vec<u8>>,
    pub summary: String,
    pub body: String,
    pub actions: Vec<[String; 2]>,
    pub urgency: Urgency,
    pub time: DateTime<Local>,
}

pub enum NotifyMessage {
    Connected(Connection),
    Notification(Box<Notification>),
    Close(u32, Option<u32>, CloseReason),
}

type DBusStruct = VecDeque<Box<dyn RefArg>>;

fn load_image_from_data(data: &DBusStruct) -> Option<Vec<u8>> {
    let mut it = data.iter();

    let width = *cast::<i32>(it.next()?)?;
    let height = *cast::<i32>(it.next()?)?;
    let rowstride = *cast::<i32>(it.next()?)?;
    let _one_point_two_bit_alpha = *cast::<bool>(it.next()?)?;
    let bits_per_sample = *cast::<i32>(it.next()?)?;
    let channels = *cast::<i32>(it.next()?)?;
    let bytes = cast::<Vec<u8>>(it.next()?)?.clone();

    let pixelstride = (channels * bits_per_sample + 7) / 8;
    let len_expected = (height - 1) * rowstride + width * pixelstride;
    if bytes.len() != len_expected as usize {
        return None;
    }

    let buffer = match channels {
        3 => ImageBuffer::from_raw(width as u32, height as u32, bytes)
            .map(|buf| DynamicImage::ImageRgb8(buf).into_rgba8())?,

        4 => ImageBuffer::from_raw(width as u32, height as u32, bytes)
            .map(|buf| DynamicImage::ImageRgba8(buf).into_rgba8())?,

        _ => return None,
    };

    let mut buf = Vec::new();
    buffer
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Bmp)
        .unwrap();

    Some(buf)
}

impl super::bus::OrgFreedesktopNotifications for Arc<dyn Proxy> {
    fn get_capabilities(&mut self) -> Result<Vec<String>, dbus::MethodErr> {
        Ok(vec![
            String::from("actions"),
            String::from("body"),
            String::from("icon-static"),
        ])
    }

    fn notify(
        &mut self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: PropMap,
        expire_timeout: i32,
    ) -> Result<u32, dbus::MethodErr> {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        static NEXT_GEN: AtomicU32 = AtomicU32::new(0);

        let gn = NEXT_GEN.fetch_add(1, SeqCst);

        let id = match replaces_id == 0 {
            true => NEXT_ID.fetch_add(1, SeqCst),
            false => replaces_id,
        };

        let app_icon = match app_icon.as_str() {
            "" => None,
            _ => Some(fs::read(app_icon).unwrap()),
        };

        let hint_image = if let Some(data) = prop_cast::<DBusStruct>(&hints, "image-data") {
            load_image_from_data(data)
        } else if let Some(data) = prop_cast::<DBusStruct>(&hints, "image_data") {
            load_image_from_data(data)
        } else if let Some(path) = hints.get("image-path") {
            Some(fs::read(path.as_str().unwrap()).unwrap())
        } else {
            hints
                .get("image_path")
                .map(|path| fs::read(path.as_str().unwrap()).unwrap())
        };

        let urgency = match prop_cast::<u8>(&hints, "urgency") {
            Some(0) => Urgency::Low,
            Some(1) => Urgency::Normal,
            Some(2) => Urgency::Critical,
            _ => Urgency::Normal,
        };

        let expire_duration = match expire_timeout > 0 {
            true => Duration::from_millis(expire_timeout as u64),
            false => match actions.is_empty() {
                true => Duration::from_secs(5),
                false => Duration::from_secs(15),
            },
        };

        self.spawn({
            let proxy = self.clone();

            async move {
                tokio::time::sleep(expire_duration).await;
                let message = NotifyMessage::Close(id, Some(gn), CloseReason::Expired);
                proxy.message(Message::new(message, None));
            }
        });

        let notification = Notification {
            id,
            generation: gn,
            app_name,
            app_icon,
            hint_image,
            summary,
            body,
            actions: actions.as_chunks().0.to_vec(),
            urgency,
            time: Local::now(),
        };

        self.message(Message::new(
            NotifyMessage::Notification(Box::new(notification)),
            None,
        ));

        Ok(id)
    }

    fn close_notification(&mut self, id: u32) -> Result<(), dbus::MethodErr> {
        self.message(Message::new(
            NotifyMessage::Close(id, None, CloseReason::Notified),
            None,
        ));

        Ok(())
    }

    fn get_server_information(
        &mut self,
    ) -> Result<(String, String, String, String), dbus::MethodErr> {
        Ok((
            env!("CARGO_PKG_NAME").into(),
            env!("CARGO_PKG_AUTHORS").into(),
            env!("CARGO_PKG_VERSION").into(),
            String::from("1.2"),
        ))
    }
}

pub async fn task(proxy: impl Proxy) {
    let conn = dbus::blocking::SyncConnection::new_session().unwrap();

    let reply = conn
        .request_name("org.freedesktop.Notifications", false, true, false)
        .unwrap();

    if let RequestNameReply::InQueue = reply {
        println!("In queue");
    }

    let match_rule = MatchRule::new()
        .with_type(MessageType::Signal)
        .with_interface("org.freedesktop.DBus")
        .with_member("NameAcquired");

    conn.add_match(match_rule, |_: (), _, msg| {
        msg.get1::<&str>() != Some("org.freedesktop.Notifications")
    })
    .unwrap();

    let proxy: Arc<dyn Proxy> = Arc::new(proxy);
    let mut crossroads = Crossroads::new();
    let token = super::bus::register_org_freedesktop_notifications(&mut crossroads);
    crossroads.insert("/org/freedesktop/Notifications", &[token], proxy.clone());

    conn.start_receive(
        MatchRule::new_method_call(),
        Box::new({
            let crossroads = Mutex::new(crossroads);

            move |msg, conn| {
                crossroads
                    .lock()
                    .unwrap()
                    .handle_message(msg, conn)
                    .unwrap();
                true
            }
        }),
    );

    let conn = Arc::new(conn);
    let connection = Connection { dbus: conn.clone() };
    proxy.message(Message::new(NotifyMessage::Connected(connection), None));

    thread::spawn(move || {
        loop {
            if let Err(err) = conn.process(Duration::from_millis(20)) {
                println!("DBus error: {err}");
            }
        }
    });
}

pub struct Connection {
    dbus: Arc<dbus::blocking::SyncConnection>,
}

impl Connection {
    pub fn notification_closed(&self, id: u32, reason: CloseReason) {
        let reason = match reason {
            CloseReason::Expired => 1,
            CloseReason::Dismissed => 2,
            CloseReason::Notified => 3,
            CloseReason::Undefined => 4,
        };

        let message = super::bus::OrgFreedesktopNotificationsNotificationClosed { id, reason };

        let path = "/org/freedesktop/Notifications".into();
        let _ = self.dbus.send(message.to_emit_message(&path));
    }

    pub fn action_invoked(&self, id: u32, action_key: String) {
        let message = super::bus::OrgFreedesktopNotificationsActionInvoked { id, action_key };

        let path = "/org/freedesktop/Notifications".into();
        let _ = self.dbus.channel().send(message.to_emit_message(&path));
    }
}
