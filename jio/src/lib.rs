use bespoke::Bespoke;
use futures::TryStreamExt;
use ori_native::prelude::*;

mod swipeable;

use serde::{Deserialize, Serialize};
use swipeable::*;
use uuid::Uuid;

#[ori_native::main]
pub fn main() {
    let mut data = Data {
        db: Bespoke::new("http://91.98.131.126:3000/").unwrap(),
        input: String::new(),
        items: Vec::new(),
    };

    App::new().run(&mut data, ui).unwrap();
}

struct Data {
    db: Bespoke,
    input: String,
    items: Vec<Uuid>,
}

#[derive(Clone, Debug, Default, PartialEq, Hash, Serialize, Deserialize)]
#[serde(default)]
struct Item {
    name: String,
}

impl Item {
    fn new(name: String) -> Self {
        Self { name }
    }
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    effect(
        window(
            row(safe_area(column((input(data), items(data))).flex(1.0)).flex(1.0))
                .background(theme::BACKGROUND)
                .flex(1.0),
        ),
        task(
            |data: &mut Data, sink| {
                let connection = data.db.clone();

                async move {
                    while let Err(error) = listen(&connection, &sink).await {
                        error!("{error}");
                    }
                }
            },
            |data, _sink, items| {
                data.items = items;
            },
        ),
    )
}

fn input(data: &Data) -> impl View<Data> + use<> {
    row(textinput()
        .text(&data.input)
        .placeholder("What needs doing?")
        .color(theme::ON_SURFACE)
        .newline(Newline::Never)
        .submit(Submit::Nothing)
        .accept_tab(false)
        .on_change(|data: &mut Data, name| data.input = name)
        .on_submit(|data: &mut Data, name| {
            data.input.clear();

            let id = Uuid::new_v4();
            data.items.insert(0, id);

            let item = Item::new(name);

            let db = data.db.clone();
            let order = data.items.clone();

            Action::rebuild().with_spawn(async move {
                db.set(&id, &item).await.unwrap();
                db.set("items", &order).await.unwrap();
            })
        })
        .size(20.0)
        .flex(1.0))
    .padding(12.0)
}

fn items(data: &Data) -> impl View<Data> + use<> {
    let items = data.items.iter().map(|id| (*id, item(*id)));
    vscroll(column(keyed(items))).flex(1.0)
}

fn item(id: Uuid) -> impl View<Data> + use<> {
    const HEIGHT: f32 = 80.0;

    with(
        |_| Item::default(),
        move |state, _| {
            effect(
                memo_hashed(&state, move |(state, _): &(Item, _)| {
                    without(
                        swipeable(text(&state.name).size(18.0))
                            .left_fill(true)
                            .left_view(text("left view"))
                            .left_color(theme::BLOOD)
                            .right_view(remove(id))
                            .height(HEIGHT)
                            .align_items(Align::Center)
                            .padding(12.0)
                            .flex(1.0),
                    )
                }),
                task(
                    move |(_, data): &mut (_, Data), sink| {
                        let db = data.db.clone();

                        async move {
                            let mut stream = db.subscribe(&id).await.unwrap();

                            while let Ok(Some(item)) = stream.try_next().await {
                                sink.send(item);
                            }
                        }
                    },
                    |(state, _), _, item| {
                        *state = item;
                    },
                ),
            )
        },
    )
}

fn remove(id: Uuid) -> impl View<Data> + use<> {
    button(
        image(include_bytes!("icon/trash.svg")),
        move |data: &mut Data| {
            data.items.retain(|item| *item != id);

            let db = data.db.clone();
            let items = data.items.clone();

            Action::rebuild().with_spawn(async move {
                let _ = db.remove(&id).await;
                let _ = db.set("items", &items).await;
            })
        },
    )
    .justify_content(Justify::Center)
    .aspect_ratio(1.0)
}

async fn listen(db: &Bespoke, sink: &Sink<Vec<Uuid>>) -> eyre::Result<()> {
    let mut stream = db.subscribe("items").await?;

    while let Some(items) = stream.try_next().await? {
        sink.send(items);
    }

    Ok(())
}
