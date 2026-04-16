use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process,
    time::Duration,
};

use chrono::{DateTime, Datelike, Days, Local, Months};
use ori_native::prelude::*;
use serde::{Deserialize, Serialize};

const INTERVAL: Duration = Duration::from_mins(20);

pub struct Data {
    config: Config,
    items: BTreeMap<Key, Item>,
    version: u64,
}

#[derive(Default, Deserialize)]
pub struct Config {
    #[serde(rename = "feed")]
    feeds: Vec<Feed>,
}

#[derive(Clone, Deserialize)]
pub struct Feed {
    url: String,
    #[serde(default = "color::default")]
    color: String,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct Key {
    time: DateTime<Local>,
    guid: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Item {
    channel: String,
    title: String,
    description: String,
    link: Option<String>,
    #[serde(with = "color")]
    color: Color,
}

mod color {
    use ori_native::prelude::*;
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn default() -> String {
        String::from("#9e993c")
    }

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&color.to_hex())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex = <&str>::deserialize(deserializer)?;
        Color::try_hex(hex).ok_or_else(|| D::Error::custom("invalid hex color"))
    }
}

impl Data {
    pub fn new(config: Config) -> eyre::Result<Self> {
        Ok(Self {
            config,
            items: Self::load_items()?,
            version: 0,
        })
    }

    fn path() -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| String::from("~"));

        Path::new(&home)
            .join(".local")
            .join("share")
            .join("widgets")
            .join("feed.json")
    }

    fn store_items(&self) -> eyre::Result<()> {
        let home = env::var("HOME").unwrap_or_else(|_| String::from("~"));

        let path = Path::new(&home)
            .join(".local")
            .join("share")
            .join("widgets");

        fs::create_dir_all(path)?;

        let json = ron::ser::to_string_pretty(&self.items, Default::default())?;
        fs::write(Self::path(), json)?;

        Ok(())
    }

    fn load_items() -> eyre::Result<BTreeMap<Key, Item>> {
        let Ok(json) = fs::read_to_string(Self::path()) else {
            return Ok(Default::default());
        };

        let items = ron::from_str(&json)?;

        Ok(items)
    }
}

pub fn menu(data: &Data) -> impl View<Data> + use<> {
    memo(data.version, |data: &Data| {
        enum Entry {
            Section(&'static str),
            Item(Key),
        }

        let mut items = Vec::new();

        let mut today = false;
        let mut yesterday = false;
        let mut this_week = false;
        let mut last_week = false;
        let mut this_month = false;
        let mut last_month = false;
        let mut year = false;

        for key in data.items.keys().rev() {
            if key.time.date_naive() == Local::now().date_naive() {
                if !today {
                    today = true;
                    items.push(Entry::Section("Today"));
                }
            } else if Some(key.time.date_naive()) == Local::now().date_naive().pred_opt() {
                if !yesterday {
                    yesterday = true;
                    items.push(Entry::Section("Yesterday"));
                }
            } else if key.time.iso_week() == Local::now().iso_week() {
                if !this_week {
                    this_week = true;
                    items.push(Entry::Section("This week"));
                }
            } else if key.time.iso_week() == one_week_ago().iso_week() {
                if !last_week {
                    last_week = true;
                    items.push(Entry::Section("Last week"));
                }
            } else if key.time.month() == Local::now().month()
                && key.time.year() == Local::now().year()
            {
                if !this_month {
                    this_month = true;
                    items.push(Entry::Section("This month"));
                }
            } else if key.time.month() == one_month_ago().month()
                && key.time.year() == one_month_ago().year()
            {
                if !last_month {
                    last_month = true;
                    items.push(Entry::Section("Last month"));
                }
            } else if !year {
                year = true;
                items.push(Entry::Section("This year"));
            }

            items.push(Entry::Item(key.clone()));
        }

        column(
            list(items.len(), move |_, index| match items[index] {
                Entry::Section(title) => any(section(title)),
                Entry::Item(ref key) => any(item(key)),
            })
            .padding(10.0)
            .gap(16.0)
            .max_height(800.0),
        )
        .background(Color::BLACK.fade(0.2))
        .corner(20.0)
        .overflow(Overflow::Hidden)
    })
}

fn one_week_ago() -> DateTime<Local> {
    Local::now().checked_sub_days(Days::new(7)).unwrap()
}

fn one_month_ago() -> DateTime<Local> {
    Local::now().checked_sub_months(Months::new(1)).unwrap()
}

fn section(name: &str) -> impl View<Data> + use<> {
    text(name)
        .margin_top(12.0)
        .color(theme::SURFACE)
        .family("Inter")
        .size(16.0)
        .weight(Weight::BOLD)
}

fn item(key: &Key) -> impl View<Data> + use<> {
    pressable({
        let key = key.clone();

        move |data: &Data, state| {
            let item = &data.items[&key];
            gtk4::popover(item_list(&key, item), item_popover(&key, item))
                .position(gtk4::Position::Right)
                .is_open(state.hovered)
        }
    })
    .on_press({
        let key = key.clone();

        move |data| {
            if let Some(ref link) = data.items[&key].link {
                let _ = process::Command::new("open").arg(link).spawn();
            }
        }
    })
}

fn item_header<T>(key: &Key, item: &Item, wrap_title: bool) -> impl View<T> + use<T> {
    let channel = text(&item.channel)
        .color(theme::feed::TEXT)
        .family("Inter")
        .weight(Weight::BOLD)
        .size(12.0);

    let date = text(key.time.format("%a %d/%m/%y %H:%M").to_string())
        .color(theme::feed::TEXT)
        .family("Inter")
        .weight(Weight::BOLD)
        .size(12.0);

    let title = text(&item.title)
        .color(theme::feed::TEXT.lighten(0.08))
        .family("Inter")
        .weight(Weight::BOLD)
        .size(10.0)
        .wrap(if wrap_title { Wrap::Word } else { Wrap::None });

    column((
        row((channel, date)).justify_content(Justify::SpaceBetween),
        title,
    ))
    .gap(4.0)
    .overflow(if wrap_title {
        Overflow::Visible
    } else {
        Overflow::Hidden
    })
}

fn item_list<T>(key: &Key, item: &Item) -> impl View<T> + use<T> {
    column(item_header(key, item, false))
        .background(item.color)
        .padding(8.0)
        .corner(4.0)
        .shadow_color(Color::BLACK.fade(0.6))
        .shadow_radius(8.0)
        .shadow_offset(2.0, 2.0)
}

fn item_popover<T>(key: &Key, item: &Item) -> impl View<T> + use<T> {
    let header = column(item_header(key, item, true))
        .border_bottom(4.0, theme::feed::TEXT)
        .padding(8.0);

    let description = text(&item.description)
        .color(theme::feed::TEXT.lighten(0.12))
        .family("Inter")
        .weight(Weight::BOLD)
        .wrap(Wrap::Word)
        .size(10.0);

    column((header, column(description).padding(8.0)))
        .background(item.color)
        .corner(4.0)
        .max_width(600.0)
        .shadow_color(Color::BLACK.fade(0.4))
        .shadow_radius(8.0)
        .shadow_offset(3.0, 2.0)
        .margin(12.0)
}

pub fn job() -> impl Effect<Data> {
    struct Message {
        channels: Vec<(usize, rss::Channel)>,
        feeds: Vec<(usize, atom_syndication::Feed)>,
    }

    async fn fetch_channels(sink: &Sink<Message>, feeds: &[Feed]) {
        let mut message = Message {
            channels: Vec::new(),
            feeds: Vec::new(),
        };

        for (index, feed) in feeds.iter().enumerate() {
            if let Err(error) = fetch_channel(&mut message, index, &feed.url).await {
                error!("failed fetching channel: {error}");
            }
        }

        sink.send(message);
    }

    async fn fetch_channel(message: &mut Message, index: usize, url: &str) -> eyre::Result<()> {
        let content = reqwest::get(url).await?.bytes().await?;

        if let Ok(channel) = rss::Channel::read_from(&content[..]) {
            message.channels.push((index, channel));
            Ok(())
        } else if let Ok(feed) = atom_syndication::Feed::read_from(&content[..]) {
            message.feeds.push((index, feed));
            Ok(())
        } else {
            Err(eyre::eyre!("failed loading feed"))
        }
    }

    fn rss_item(config: &Feed, channel: &rss::Channel, item: &rss::Item) -> Option<(Key, Item)> {
        let time = DateTime::parse_from_rfc2822(item.pub_date()?).ok()?;

        let key = Key {
            time: time.with_timezone(&Local),
            guid: item.guid()?.value.clone(),
        };

        let item = Item {
            channel: channel.title.clone(),
            title: item.title.clone()?,
            description: item.description.clone()?,
            link: item.link.clone(),
            color: Color::hex(&config.color),
        };

        Some((key, item))
    }

    fn atom_item(
        config: &Feed,
        feed: &atom_syndication::Feed,
        entry: &atom_syndication::Entry,
    ) -> Option<(Key, Item)> {
        let media = entry.extensions().get("media")?.get("group")?.first()?;
        let description = media
            .children()
            .get("description")?
            .first()?
            .value
            .clone()?;

        let key = Key {
            time: entry.published()?.with_timezone(&Local),
            guid: entry.id.clone(),
        };

        let item = Item {
            channel: feed.title.to_string(),
            title: entry.title.to_string(),
            description,
            link: entry.links().first().map(|link| link.href.clone()),
            color: Color::hex(&config.color),
        };

        Some((key, item))
    }

    task(
        |data: &mut Data, sink| {
            let feeds = data.config.feeds.clone();

            async move {
                loop {
                    fetch_channels(&sink, &feeds).await;
                    tokio::time::sleep(INTERVAL).await;
                }
            }
        },
        |data, _, message| {
            for (index, channel) in message.channels {
                for item in channel.items.iter() {
                    if let Some((key, item)) = rss_item(&data.config.feeds[index], &channel, item) {
                        data.items.insert(key, item);
                    }
                }
            }

            for (index, feed) in message.feeds {
                for entry in feed.entries.iter() {
                    if let Some((key, item)) = atom_item(&data.config.feeds[index], &feed, entry) {
                        data.items.insert(key, item);
                    }
                }
            }

            if let Err(err) = data.store_items() {
                error!("failed storing feed: {err}");
            }

            data.version += 1;
        },
    )
}
