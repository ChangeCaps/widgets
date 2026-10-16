use ori_native::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[builder]
pub fn swipeable<T>(
    contents: impl ViewSeq<T>,

    #[default = row(())] left_view: impl View<T>,
    #[default = row(())] right_view: impl View<T>,

    #[default] left_fill: bool,
    #[default] right_fill: bool,

    #[default = Color::RED] left_color: Color,
    #[default = Color::GREEN] right_color: Color,
    #[default = Color::TRANSPARENT] color: Color,

    /// Callback called when the left side is opened.
    #[default = |_| Action::new()]
    mut on_open_left: impl (FnMut(&mut T) -> impl Into<Action>) + 'static,
    /// Callback called when the right side is opened.
    #[default = |_| Action::new()]
    mut on_open_right: impl (FnMut(&mut T) -> impl Into<Action>) + 'static,
    /// Callback called when closed.
    #[default = |_| Action::new()]
    mut on_close: impl (FnMut(&mut T) -> impl Into<Action>) + 'static,

    #[default = BorderStyle {
        width: Sides::from((0.0, 0.0, 2.0, 0.0)),
        color: Color::TRANSPARENT,
    }]
    #[border]
    border: BorderStyle,

    #[layout] layout: LayoutStyle,
    #[corners] corners: Corners<f32>,
    #[padding] padding: Sides<Length>,
    #[shadow] shadow: Shadow,
    #[flex] flex: FlexStyle,
) -> impl View<T> {
    struct State<T> {
        left_width: f32,
        right_width: f32,

        left_fill: bool,
        right_fill: bool,

        width: f32,
        offset: f32,
        start: Option<f32>,

        changed: bool,
        side: Option<Side>,
        on_open_left: Option<Box<dyn FnMut(&mut T) -> Action>>,
        on_open_right: Option<Box<dyn FnMut(&mut T) -> Action>>,
        on_close: Option<Box<dyn FnMut(&mut T) -> Action>>,
    }

    impl<T> State<T> {
        fn side_changed(&mut self, data: &mut T) -> Action {
            self.changed = false;

            match self.side {
                Some(Side::Left) => match self.on_open_left {
                    Some(ref mut on_open) => on_open(data),
                    None => Action::new(),
                },

                Some(Side::Right) => match self.on_open_right {
                    Some(ref mut on_open) => on_open(data),
                    None => Action::new(),
                },

                None => match self.on_close {
                    Some(ref mut on_close) => on_close(data),
                    None => Action::new(),
                },
            }
        }

        fn left_width(&self) -> f32 {
            match self.left_fill {
                true => self.width,
                false => self.left_width,
            }
        }

        fn right_width(&self) -> f32 {
            match self.right_fill {
                true => self.width,
                false => self.right_width,
            }
        }

        fn left_threshold(&self) -> f32 {
            let left_threshold = self.width / 4.0;

            if left_threshold > self.left_width() {
                self.left_width()
            } else if self.side == Some(Side::Left) {
                self.left_width() - left_threshold
            } else {
                left_threshold
            }
        }

        fn right_threshold(&self) -> f32 {
            let right_threshold = self.width / 4.0;

            if right_threshold > self.right_width() {
                self.right_width()
            } else if self.side == Some(Side::Right) {
                self.right_width() - right_threshold
            } else {
                right_threshold
            }
        }

        fn set_side(&mut self, data: &mut T, side: Option<Side>) -> Action {
            if self.side != side {
                self.side = side;

                match side {
                    Some(Side::Left) => self.offset = self.left_width(),
                    Some(Side::Right) => self.offset = -self.right_width(),
                    None => self.offset = 0.0,
                }

                self.side_changed(data).with_rebuild(true)
            } else {
                Action::new()
            }
        }

        fn on_event(&mut self, data: &mut T, event: PressableEvent) -> Action {
            match event {
                PressableEvent::Pressed(event) => {
                    self.start = Some(event.position.x - self.offset);

                    Action::new()
                }

                PressableEvent::Moved(event) => {
                    if let Some(start) = self.start {
                        let offset = event.position.x - start;
                        self.offset = offset.clamp(-self.right_width(), self.left_width());

                        Action::rebuild()
                    } else {
                        Action::new()
                    }
                }

                PressableEvent::Released(..) | PressableEvent::Cancelled(..) => {
                    self.start = None;

                    let mut action = Action::rebuild();

                    if self.offset >= self.left_threshold() {
                        let changed = self.side != Some(Side::Left);
                        self.side = Some(Side::Left);

                        if self.offset == self.left_width() && changed {
                            action |= self.side_changed(data);
                        } else {
                            self.offset = self.left_width();
                            self.changed = changed;
                        }
                    } else if self.offset <= -self.right_threshold() {
                        let changed = self.side != Some(Side::Right);
                        self.side = Some(Side::Right);

                        if self.offset == -self.right_width() && changed {
                            action |= self.side_changed(data);
                        } else {
                            self.offset = -self.right_width();
                            self.changed = changed;
                        }
                    } else {
                        self.offset = 0.0;
                        self.changed = self.side.is_some();
                        self.side = None;
                    };

                    action
                }

                _ => Action::new(),
            }
        }
    }

    let mut contents = Some(contents);
    let mut left_view = Some(left_view);
    let mut right_view = Some(right_view);

    with(
        move |_| State {
            left_width: 0.0,
            right_width: 0.0,

            left_fill,
            right_fill,

            width: 0.0,
            offset: 0.0,
            start: None,

            changed: false,
            side: None,
            on_open_left: None,
            on_open_right: None,
            on_close: None,
        },
        move |_, _| {
            let mut contents = contents.take();
            let mut left_view = left_view.take();
            let mut right_view = right_view.take();

            let view = pressable(move |(state, _): &(State<T>, _), press| {
                let mut contents = contents.take();
                let mut left_view = left_view.take();
                let mut right_view = right_view.take();

                let border_color = if press.focused {
                    Color::hex("#0000ff")
                } else {
                    border.color
                };

                animate(
                    spring(state.offset, move |(state, _): &(State<T>, _), offset| {
                        let left = row(on_layout(
                            without(maybe(left_view.take())),
                            move |(state, _): &mut (State<T>, _), width, _| {
                                state.left_width = width;

                                if state.side == Some(Side::Left)
                                    && state.start.is_none()
                                    && !state.left_fill
                                {
                                    state.offset = width;
                                    Action::rebuild()
                                } else {
                                    Action::new()
                                }
                            },
                        ))
                        .top(0.0)
                        .bottom(0.0)
                        .left(0.0)
                        .width(offset.ceil())
                        .justify_content(Justify::Start)
                        .background(left_color)
                        .position(Position::Absolute)
                        .overflow(Overflow::Hidden);

                        let right = row(on_layout(
                            without(maybe(right_view.take())),
                            |(state, _): &mut (State<T>, _), width, _| {
                                state.right_width = width;

                                if state.side == Some(Side::Right)
                                    && state.start.is_none()
                                    && !state.right_fill
                                {
                                    state.offset = -width;
                                    Action::rebuild()
                                } else {
                                    Action::new()
                                }
                            },
                        ))
                        .top(0.0)
                        .bottom(0.0)
                        .right(0.0)
                        .width(-offset.floor())
                        .justify_content(Justify::End)
                        .background(right_color)
                        .position(Position::Absolute)
                        .overflow(Overflow::Hidden);

                        row((
                            detach(
                                offset <= 0.0
                                    && state.start.is_none()
                                    && state.side != Some(Side::Left),
                                left,
                            ),
                            detach(
                                offset >= 0.0
                                    && state.start.is_none()
                                    && state.side != Some(Side::Right),
                                right,
                            ),
                            without(
                                row(maybe_seq(contents.take()))
                                    .left(offset)
                                    .width(Fract(1.0))
                                    .background(color)
                                    .set_padding(padding)
                                    .set_flex(flex),
                            ),
                        ))
                        .overflow(Overflow::Hidden)
                        .border(border.width, border_color)
                        .set_corners(corners)
                        .set_layout(layout)
                        .set_shadow(shadow)
                    })
                    .duration_bounce(0.1, 0.0),
                )
                .on_end(|(state, data): &mut (State<T>, T)| match state.changed {
                    true => state.side_changed(data),
                    false => Action::new(),
                })
            })
            .focusable(true)
            .on_event(move |(state, data), event| state.on_event(data, event))
            .on_key('h', Modifiers::empty(), |(state, data)| match state.side {
                Some(Side::Left) => state.set_side(data, None),
                Some(Side::Right) => Action::new(),
                None => state.set_side(data, Some(Side::Right)),
            })
            .on_key('l', Modifiers::empty(), |(state, data)| match state.side {
                Some(Side::Left) => Action::new(),
                Some(Side::Right) => state.set_side(data, None),
                None => state.set_side(data, Some(Side::Left)),
            });

            on_layout(view, |(state, _): &mut (State<T>, _), width, _| {
                state.width = width;
                Action::new()
            })
        },
    )
    .update(move |state, _| {
        state.left_fill = left_fill;
        state.right_fill = right_fill;

        state.on_open_left = Some(Box::new(move |data| on_open_left(data).into()));
        state.on_open_right = Some(Box::new(move |data| on_open_right(data).into()));
        state.on_close = Some(Box::new(move |data| on_close(data).into()));
    })
}
