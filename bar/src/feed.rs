use std::{process, time::Duration};

use chrono::{DateTime, Datelike, Days, Local, Months};
use ori_native::prelude::*;
use serde::Deserialize;

const INTERVAL: Duration = Duration::from_mins(20);

pub struct Data {
    config: Config,
    items: Vec<Item>,
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
    #[serde(default = "default_feed_color")]
    color: String,
}

fn default_feed_color() -> String {
    String::from("#9e993c")
}

struct Item {
    channel: String,
    title: String,
    description: String,
    link: Option<String>,
    time: DateTime<Local>,
    color: Color,
}

impl Data {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            items: Vec::new(),
            version: 0,
        }
    }

    fn compose_feed(&mut self) {
        self.items.sort_by_key(|item| item.time);
        self.items.reverse();
    }
}

pub fn menu(data: &Data) -> impl View<Data> + use<> {
    memo(data.version, |data: &Data| {
        enum Entry {
            Section(&'static str),
            Item(usize),
        }

        let mut items = Vec::new();

        let mut today = false;
        let mut yesterday = false;
        let mut this_week = false;
        let mut last_week = false;
        let mut this_month = false;
        let mut last_month = false;
        let mut year = false;

        for (i, item) in data.items.iter().enumerate() {
            if item.time.date_naive() == Local::now().date_naive() {
                if !today {
                    today = true;
                    items.push(Entry::Section("Today"));
                }
            } else if Some(item.time.date_naive()) == Local::now().date_naive().pred_opt() {
                if !yesterday {
                    yesterday = true;
                    items.push(Entry::Section("Yesterday"));
                }
            } else if item.time.iso_week() == Local::now().iso_week() {
                if !this_week {
                    this_week = true;
                    items.push(Entry::Section("This week"));
                }
            } else if item.time.iso_week() == one_week_ago().iso_week() {
                if !last_week {
                    last_week = true;
                    items.push(Entry::Section("Last week"));
                }
            } else if item.time.month() == Local::now().month()
                && item.time.year() == Local::now().year()
            {
                if !this_month {
                    this_month = true;
                    items.push(Entry::Section("This month"));
                }
            } else if item.time.month() == one_month_ago().month()
                && item.time.year() == one_month_ago().year()
            {
                if !last_month {
                    last_month = true;
                    items.push(Entry::Section("Last month"));
                }
            } else if !year {
                year = true;
                items.push(Entry::Section("This year"));
            }

            items.push(Entry::Item(i));
        }

        column(
            list(items.len(), move |_, index| match items[index] {
                Entry::Section(title) => any(section(title)),
                Entry::Item(index) => any(item(index)),
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

fn item(index: usize) -> impl View<Data> + use<> {
    pressable(move |data: &Data, state| {
        let item = &data.items[index];
        gtk4::popover(item_list(item), item_popover(item))
            .position(gtk4::Position::Right)
            .is_open(state.hovered)
    })
    .on_press(move |data| {
        if let Some(ref link) = data.items[index].link {
            let _ = process::Command::new("open").arg(link).spawn();
        }
    })
}

fn item_header<T>(item: &Item, wrap_title: bool) -> impl View<T> + use<T> {
    let channel = text(&item.channel)
        .color(theme::feed::TEXT)
        .family("Inter")
        .weight(Weight::BOLD)
        .size(12.0);

    let date = text(item.time.format("%a %d/%m/%y %H:%M").to_string())
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

fn item_list<T>(item: &Item) -> impl View<T> + use<T> {
    column(item_header(item, false))
        .background(item.color)
        .padding(8.0)
        .corner(4.0)
        .shadow_color(Color::BLACK.fade(0.6))
        .shadow_radius(8.0)
        .shadow_offset(2.0, 2.0)
}

fn item_popover<T>(item: &Item) -> impl View<T> + use<T> {
    let header = column(item_header(item, true))
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

    fn rss_item(config: &Feed, channel: &rss::Channel, item: &rss::Item) -> Option<Item> {
        let time = DateTime::parse_from_rfc2822(item.pub_date()?).ok()?;

        Some(Item {
            channel: channel.title.clone(),
            title: item.title.clone()?,
            description: item.description.clone()?,
            link: item.link.clone(),
            time: time.with_timezone(&Local),
            color: Color::hex(&config.color),
        })
    }

    fn atom_item(
        config: &Feed,
        feed: &atom_syndication::Feed,
        entry: &atom_syndication::Entry,
    ) -> Option<Item> {
        let media = entry.extensions().get("media")?.get("group")?.first()?;
        let description = media
            .children()
            .get("description")?
            .first()?
            .value
            .clone()?;

        Some(Item {
            channel: feed.title.to_string(),
            title: entry.title.to_string(),
            description,
            link: entry.links().first().map(|link| link.href.clone()),
            time: entry.published()?.with_timezone(&Local),
            color: Color::hex(&config.color),
        })
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
            data.items.clear();

            for (index, channel) in message.channels {
                for item in channel.items.iter() {
                    if let Some(item) = rss_item(&data.config.feeds[index], &channel, item) {
                        data.items.push(item);
                    }
                }
            }

            for (index, feed) in message.feeds {
                for entry in feed.entries.iter() {
                    if let Some(item) = atom_item(&data.config.feeds[index], &feed, entry) {
                        data.items.push(item);
                    }
                }
            }

            data.compose_feed();
            data.version += 1;
        },
    )
}
