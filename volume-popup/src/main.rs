use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use libpulse_binding::{
    callbacks::ListResult,
    context::{
        Context, FlagSet, State,
        subscribe::{Facility, InterestMaskSet},
    },
    mainloop::standard::Mainloop,
    volume::Volume,
};
use ori_native::prelude::*;

fn main() {
    let mut data = Data {
        volume: 0.0,
        show: false,
        changed: Instant::now(),
    };

    App::new().run(&mut data, ui);
}

struct Data {
    volume: f32,
    show: bool,
    changed: Instant,
}

static TIMEOUT: Duration = Duration::from_secs(1);

mod theme {
    use ori_native::prelude::*;

    pub static BACKGROUND: Color = Color::hex("#353535");
    pub static OUTLINE: Color = Color::hex("#ffffff").fade(0.1);
    pub static PRIMARY: Color = Color::hex("#a6d189");
    pub static ACCENT: Color = Color::hex("#b5bfe2");
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    effects((listen(), data.show.then(|| volume_window(data))))
}

fn volume_window(data: &Data) -> impl Effect<Data> + use<> {
    layer_shell(
        row((
            volume_bar(data.volume),
            image(include_bytes!("icon/sound-high.svg"))
                .size(32.0, 32.0)
                .tint(theme::ACCENT),
        ))
        .gap(20.0)
        .padding(24.0)
        .padding_left(40.0)
        .padding_right(40.0)
        .justify_contents(Justify::Center)
        .align_items(Align::Center)
        .flex(1.0)
        .background_color(theme::BACKGROUND)
        .border_color(theme::OUTLINE)
        .border(1.0)
        .corner(8.0),
    )
    .sizing(Sizing::Content)
    .exclusive_zone(ExclusiveZone::Fixed(0))
    .anchor_top(true)
    .margin_top(40)
}

fn volume_bar(fraction: f32) -> impl View<Data> + use<> {
    transition(fraction, Ease(0.05), |fraction, _| {
        row(row(())
            .background_color(theme::PRIMARY)
            .width(Fraction(fraction)))
        .background_color(Color::BLACK.fade(0.4))
        .size(200.0, 8.0)
        .corner(4.0)
    })
}

fn listen() -> impl Effect<Data> {
    enum Message {
        Volume(f32),
        Timeout,
    }

    task(
        |_data, sink| async {
            listen_volume_changes(move |fraction| {
                sink.send(Message::Volume(fraction));
            });
        },
        |data: &mut Data, sink, message| match message {
            Message::Volume(volume) => {
                if data.volume != volume {
                    data.show = true;
                    data.volume = volume;
                    data.changed = Instant::now();

                    Action::spawn(async move {
                        tokio::time::sleep(TIMEOUT).await;
                        sink.send(Message::Timeout);
                    })
                    .with_rebuild(true)
                } else {
                    Action::new()
                }
            }

            Message::Timeout => {
                if data.changed.elapsed() > TIMEOUT {
                    data.show = false;

                    Action::rebuild()
                } else {
                    Action::new()
                }
            }
        },
    )
}

fn listen_volume_changes(f: impl Fn(f32) + Send + 'static) {
    thread::spawn(move || {
        let mut mainloop = Mainloop::new().unwrap();
        let mut context = Context::new(&mainloop, "volume-popup").unwrap();

        context.connect(None, FlagSet::NOFLAGS, None).unwrap();

        loop {
            mainloop.iterate(false);
            match context.get_state() {
                State::Ready => break,
                State::Failed | State::Terminated => return,
                _ => {}
            }
        }

        let default_sink = Arc::new(Mutex::new(String::new()));

        context.introspect().get_server_info({
            let default_sink = default_sink.clone();

            move |info| {
                *default_sink.lock().unwrap() = info
                    .default_sink_name
                    .clone()
                    .unwrap_or_default()
                    .to_string();
            }
        });

        let introspector = context.introspect();
        let f = Arc::new(f);

        context.set_subscribe_callback(Some(Box::new(move |facility, _, index| match facility {
            Some(Facility::Sink) => {
                introspector.get_sink_info_by_index(index, {
                    let default_sink = default_sink.clone();
                    let f = f.clone();

                    move |res| {
                        if let ListResult::Item(info) = res
                            && let Ok(default_sink) = default_sink.lock()
                            && info.name.as_deref() == Some(default_sink.as_ref())
                        {
                            let avg = info.volume.avg();
                            let fraction = avg.0 as f32 / Volume::NORMAL.0 as f32;

                            f(fraction);
                        }
                    }
                });
            }

            Some(Facility::Server) => {
                introspector.get_server_info({
                    let default_sink = default_sink.clone();

                    move |info| {
                        *default_sink.lock().unwrap() = info
                            .default_sink_name
                            .clone()
                            .unwrap_or_default()
                            .to_string();
                    }
                });
            }

            _ => {}
        })));

        context.subscribe(InterestMaskSet::SINK | InterestMaskSet::SERVER, |_| {});

        let _ = mainloop.run();
    });
}
