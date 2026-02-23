use std::{
    sync::{Arc, Mutex},
    thread,
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
        show: true,
    };

    App::new().run(&mut data, ui);
}

struct Data {
    volume: f32,
    show: bool,
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    effects((listen(), data.show.then(|| volume_window(data))))
}

fn volume_window(_data: &Data) -> impl Effect<Data> + use<> {
    window(row(()))
}

fn listen() -> impl Effect<Data> {
    enum Message {
        Volume(f32),
    }

    task(
        |_data, sink| async {
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

                context.set_subscribe_callback(Some(Box::new(move |facility, _, index| {
                    match facility {
                        Some(Facility::Sink) => {
                            introspector.get_sink_info_by_index(index, {
                                let sink = sink.clone();
                                let default_sink = default_sink.clone();

                                move |res| {
                                    if let ListResult::Item(info) = res
                                        && let Ok(default_sink) = default_sink.lock()
                                        && info.name.as_deref() == Some(default_sink.as_ref())
                                    {
                                        let avg = info.volume.avg();
                                        let fraction = avg.0 as f32 / Volume::NORMAL.0 as f32;

                                        sink.send(Message::Volume(fraction));
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
                    }
                })));

                context.subscribe(InterestMaskSet::SINK | InterestMaskSet::SERVER, |_| {});

                let _ = mainloop.run();
            });
        },
        |data: &mut Data, message| match message {
            Message::Volume(volume) => data.volume = volume,
        },
    )
}
