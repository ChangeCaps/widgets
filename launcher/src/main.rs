use freedesktop_desktop_entry::{DesktopEntry, desktop_entries};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use ori_native::prelude::*;

fn main() {
    let entries = desktop_entries(&[]);
    let sorted = entries
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, _)| (i, 0))
        .rev()
        .collect();

    let mut data = Data {
        entries,
        sorted,
        select: 0,
        search: String::new(),
    };

    App::new().run(&mut data, ui).unwrap();
}

struct Data {
    entries: Vec<DesktopEntry>,
    sorted: Vec<(usize, u16)>,
    select: usize,
    search: String,
}

impl Data {
    fn next(&mut self) {
        self.select += 1;
        self.select = self.select.min(self.sorted.len() - 1);
    }

    fn prev(&mut self) {
        self.select = self.select.saturating_sub(1);
    }

    fn search(&mut self, text: String) {
        self.search = text;

        let mut search_buf = Vec::new();
        let mut name_buf = Vec::new();

        let search = Utf32Str::new(&self.search, &mut search_buf);

        let mut config = Config::DEFAULT;
        config.ignore_case = true;

        let mut matcher = Matcher::new(config);

        self.sorted = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(i, e)| {
                let name = e.name::<&str>(&[])?;
                let name = Utf32Str::new(&name, &mut name_buf);

                let mut scores = Vec::new();
                scores.push(matcher.fuzzy_match(name, search));

                for keyword in e.keywords::<&str>(&[]).into_iter().flatten() {
                    let keyword = Utf32Str::new(&keyword, &mut name_buf);
                    scores.push(matcher.fuzzy_match(keyword, search));
                }

                for category in e.categories().into_iter().flatten() {
                    let category = Utf32Str::new(category, &mut name_buf);
                    scores.push(matcher.fuzzy_match(category, search));
                }

                scores.into_iter().flatten().max().map(|score| (i, score))
            })
            .collect();

        self.sorted.sort_by_key(|(_, score)| *score);
        self.sorted.reverse();
        self.select = self.select.min(self.sorted.len().saturating_sub(1));
    }

    fn launch(&mut self, _text: String) {
        let (index, _) = self.sorted[self.select];
        let entry = &self.entries[index];

        if let Ok(exec) = entry.parse_exec()
            && let Some(command) = exec.first()
        {
            #[allow(clippy::zombie_processes)]
            std::process::Command::new(command)
                .args(exec.iter().skip(1))
                .spawn()
                .unwrap();

            std::process::exit(0);
        }
    }
}

fn ui(data: &Data) -> impl Effect<Data> + use<> {
    let entries = data
        .sorted
        .iter()
        .enumerate()
        .filter_map(|(i, (j, _))| entry(&data.entries[*j], i, i == data.select))
        .take(32)
        .collect::<Vec<_>>();

    let view = column((
        row(textinput()
            .text(&data.search)
            .size(16.0)
            .flex(1.0)
            .family("Ubuntu Light")
            .color(Color::WHITE)
            .newline(Newline::None)
            .on_change(Data::search)
            .on_submit(Data::launch))
        .padding(8.0)
        .border_bottom_width(2.0)
        .border_color(theme::OUTLINE),
        vscroll(column(entries)),
    ))
    .background(theme::BACKGROUND)
    .border(1.0, theme::OUTLINE)
    .corner(8.0)
    .padding(12.0)
    .gap(8.0)
    .shadow_color(Color::BLACK.fade(0.4))
    .shadow_radius(8.0)
    .shadow_offset(2.0, 3.0)
    .margin(12.0)
    .size(600.0, 400.0);

    let shell = layer_shell(view)
        .sizing(Sizing::Content)
        .keyboard(KeyboardInput::OnDemand)
        .on_key(NamedKey::Escape, Modifiers::empty(), |_| -> () {
            std::process::exit(0);
        })
        .on_key('n', Modifiers::CONTROL, Data::next)
        .on_key('p', Modifiers::CONTROL, Data::prev)
        .on_key('j', Modifiers::CONTROL, Data::next)
        .on_key('k', Modifiers::CONTROL, Data::prev)
        .on_key(NamedKey::ArrowDown, Modifiers::empty(), Data::next)
        .on_key(NamedKey::ArrowUp, Modifiers::empty(), Data::prev);

    effects(shell)
}

fn entry(entry: &DesktopEntry, index: usize, selected: bool) -> Option<impl View<Data> + use<>> {
    let name = entry.name::<&str>(&[])?.to_string();

    Some(
        pressable(move |_, state| {
            let color = match selected {
                true => Color::BLACK.fade(0.2),
                false => match state.hovered {
                    true => Color::WHITE.fade(0.1),
                    false => Color::TRANSPARENT,
                },
            };

            let padding = match selected {
                true => 8.0,
                false => 0.0,
            };

            let name = name.clone();
            any(transition(
                (color, padding),
                Ease(0.1),
                move |_, (color, padding)| {
                    row(text(&name)
                        .color(Color::WHITE.fade(0.5))
                        .family("Ubuntu Light"))
                    .background(color)
                    .padding(8.0)
                    .padding_left(8.0 + padding)
                    .corner(8.0)
                },
            ))
        })
        .on_press(move |data: &mut Data| {
            data.select = index;
        }),
    )
}
