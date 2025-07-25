use crate::{AutoFocusable, Disableable};
use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, InteractiveElement, Interactivity, IntoElement,
    Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement, Point, RenderOnce,
    Stateful, StatefulInteractiveElement, StyleRefinement, Styled, Window, div,
    prelude::FluentBuilder,
};
use smallvec::SmallVec;
use std::rc::Rc;

pub fn button(id: impl Into<ElementId>) -> Button {
    let id = id.into();
    Button {
        id: id.clone(),
        base: div().id(id),
        disabled: false,
        children: SmallVec::new(),
        on_click: None,
        auto_focus: false,
    }
}

#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    base: Stateful<Div>,
    disabled: bool,
    children: SmallVec<[AnyElement; 2]>,
    on_click: Option<Rc<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>>,
    auto_focus: bool,
}

impl Button {
    pub fn on_click(
        mut self,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(Box::new(on_click)));
        self
    }
}

impl Disableable for Button {
    fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl AutoFocusable for Button {
    fn auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Button {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Button {}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, app: &mut App) -> impl IntoElement {
        let focus_handle = window.use_keyed_state(self.id, app, |window, app| {
            let focus_handle = app.focus_handle().tab_stop(true);
            if self.auto_focus {
                focus_handle.focus(window);
            }
            focus_handle
        });

        self.base
            .when_some(
                self.on_click.filter(|_| !self.disabled),
                |this, on_click| {
                    let on_click_clone = on_click.clone();
                    this.track_focus(focus_handle.read(app))
                        .on_mouse_down(MouseButton::Left, move |_, window, app| {
                            window.prevent_default();
                            focus_handle.update(app, |focus_handle, _| {
                                if !focus_handle.is_focused(window) {
                                    focus_handle.focus(window);
                                }
                            });
                        })
                        .on_click(move |event, window, app| (on_click)(event, window, app))
                        .on_key_up(move |event, window, app| {
                            if event.keystroke.key == "space" || event.keystroke.key == "enter" {
                                (on_click_clone)(
                                    &ClickEvent {
                                        down: MouseDownEvent {
                                            button: MouseButton::Left,
                                            position: Point::default(),
                                            modifiers: Modifiers::none(),
                                            click_count: 0,
                                            first_mouse: false,
                                        },
                                        up: MouseUpEvent {
                                            button: MouseButton::Left,
                                            position: Point::default(),
                                            modifiers: event.keystroke.modifiers,
                                            click_count: 0,
                                        },
                                    },
                                    window,
                                    app,
                                );
                            }
                        })
                },
            )
            .children(self.children)
    }
}
