use hyprland::{
    data::{Monitor, Monitors, Workspace, Workspaces},
    dispatch::{Dispatch, DispatchType},
    event_listener::EventListener,
    shared::HyprData,
};
use ori_native::prelude::*;

use crate::theme;

pub struct Hyprland {
    workspaces: Vec<Option<Workspace>>,
    monitors: Vec<Monitor>,
}

impl Hyprland {
    pub fn new() -> Self {
        Self {
            workspaces: Self::fetch_workspaces(),
            monitors: Self::fetch_monitors(),
        }
    }

    fn fetch_workspaces() -> Vec<Option<Workspace>> {
        let mut workspaces = Workspaces::get().unwrap().into_iter().collect::<Vec<_>>();

        (1..=10)
            .map(|i| {
                workspaces
                    .iter()
                    .position(|w| w.id == i)
                    .map(|i| workspaces.remove(i))
            })
            .collect()
    }

    fn fetch_monitors() -> Vec<Monitor> {
        Monitors::get().unwrap().into_iter().collect()
    }
}

pub fn workspaces(data: &Hyprland, monitor_index: usize) -> impl View<Hyprland> + use<> {
    let workspaces = data
        .workspaces
        .iter()
        .enumerate()
        .map(|(index, workspace)| {
            self::workspace(
                data,
                &data.monitors[monitor_index],
                workspace.as_ref(),
                index,
            )
        })
        .collect::<Vec<_>>();

    column(workspaces).gap(8.0).align_items(Align::Center)
}

fn workspace(
    data: &Hyprland,
    monitor: &Monitor,
    workspace: Option<&Workspace>,
    index: usize,
) -> impl View<Hyprland> + use<> {
    #[derive(Clone, Copy)]
    enum Kind {
        Active,
        Visible,
        Used,
        Empty,
    }

    let kind = if let Some(workspace) = workspace {
        if monitor.active_workspace.id == workspace.id {
            Kind::Active
        } else if data
            .monitors
            .iter()
            .any(|m| m.active_workspace.id == workspace.id)
        {
            Kind::Visible
        } else if workspace.windows > 0 {
            Kind::Used
        } else {
            Kind::Empty
        }
    } else {
        Kind::Empty
    };

    let height = match kind {
        Kind::Active => 32.0,
        Kind::Visible => 28.0,
        Kind::Used => 24.0,
        Kind::Empty => 20.0,
    };

    transition(height, Ease(0.2), move |height, _| {
        pressable(move |_, _state| {
            let view = column(())
                .size(8.0, height)
                .background_color(theme::OUTLINE)
                .corner(4.0);

            match kind {
                Kind::Active => view.background_color(theme::PRIMARY),
                Kind::Visible => view.background_color(theme::ACCENT),
                Kind::Used => view.background_color(theme::SURFACE),
                Kind::Empty => view,
            }
        })
        .on_press(move |_| {
            Dispatch::call(DispatchType::Custom(
                "focusworkspaceoncurrentmonitor",
                &(index + 1).to_string(),
            ))
            .unwrap();

            Action::new()
        })
    })
}

pub fn listen_task() -> impl Effect<Hyprland> {
    enum Changed {
        Workspaces,
        Monitors,
    }

    task(
        |_, sink| async move {
            tokio::spawn(async move {
                let mut listener = EventListener::new();

                listener.add_workspace_changed_handler({
                    let sink = sink.clone();

                    move |_| {
                        sink.send(Changed::Monitors);
                        sink.send(Changed::Workspaces);
                    }
                });

                listener.add_monitor_added_handler({
                    let sink = sink.clone();
                    move |_| sink.send(Changed::Monitors)
                });

                listener.add_monitor_removed_handler({
                    let sink = sink.clone();
                    move |_| sink.send(Changed::Monitors)
                });

                let _ = listener.start_listener();
            })
            .await
            .unwrap();
        },
        |data: &mut Hyprland, _, message| match message {
            Changed::Workspaces => {
                data.workspaces = Hyprland::fetch_workspaces();
            }

            Changed::Monitors => {
                data.monitors = Hyprland::fetch_monitors();
            }
        },
    )
}
