use hyprland::{
    data::{Monitor, Monitors, Workspace, Workspaces},
    dispatch::{Dispatch, DispatchType},
    event_listener::EventListener,
    shared::HyprData,
};
use ori_native::prelude::*;

pub struct Data {
    workspaces: Vec<Option<Workspace>>,
    monitors: Vec<Monitor>,
}

impl Data {
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

pub fn workspaces(data: &Data, monitor_index: usize) -> impl View<Data> + use<> {
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
    data: &Data,
    monitor: &Monitor,
    workspace: Option<&Workspace>,
    index: usize,
) -> impl View<Data> + use<> {
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

    let color = match kind {
        Kind::Active => theme::GREEN,
        Kind::Visible => theme::PRIMARY,
        Kind::Used => theme::SURFACE,
        Kind::Empty => theme::OUTLINE,
    };

    transition(height, Ease(0.2), move |_, height| {
        pressable(move |_, _| column(()).size(8.0, height).corner(4.0).background(color)).on_press(
            move |_| {
                Dispatch::call(DispatchType::Custom(
                    "focusworkspaceoncurrentmonitor",
                    &(index + 1).to_string(),
                ))
                .unwrap();

                Action::new()
            },
        )
    })
}

pub fn job() -> impl Effect<Data> {
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
        |data: &mut Data, _, message| match message {
            Changed::Workspaces => {
                data.workspaces = Data::fetch_workspaces();
            }

            Changed::Monitors => {
                data.monitors = Data::fetch_monitors();
            }
        },
    )
}
