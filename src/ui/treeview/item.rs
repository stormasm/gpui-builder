use gpui::prelude::*;
use gpui::*;
use uuid::Uuid;

use crate::appearance::{colors, sizes};
use crate::component::element::div::DivElement;
use crate::component::element::text::TextElement;
use crate::component::element::ComponentElement;
use crate::component::Component;
use crate::ui::context_menu::{ContextMenuAction, ContextMenuGlobal};

pub struct TreeviewItem {
    component: Model<Component>,
    element: ComponentElement,
    active_element: Model<Option<Uuid>>,

    indent: u32,
    cached_text: String,
    child_views: Vec<View<TreeviewItem>>,

    hovered: bool,
    active: bool,
}

impl TreeviewItem {
    pub fn new<V: 'static>(
        indent: u32,
        component: &Model<Component>,
        element: ComponentElement,
        active_element: Model<Option<Uuid>>,
        cx: &mut ViewContext<V>,
    ) -> View<Self> {
        cx.new_view(|cx| {
            let cached_text = get_element_text(&element, cx);
            let child_views =
                Self::make_child_views(indent + 1, component, &element, &active_element, cx);
            let active = *active_element.read(cx) == Some(element.id());

            Self::observe_text(&element, cx);
            Self::observe_children(&element, cx);
            Self::observe_active_element(&active_element, cx);

            Self {
                component: component.clone(),
                element,
                active_element,

                indent,
                cached_text,
                child_views,

                hovered: false,
                active,
            }
        })
    }

    fn make_child_views(
        indent: u32,
        component: &Model<Component>,
        element: &ComponentElement,
        active_element: &Model<Option<Uuid>>,
        cx: &mut ViewContext<Self>,
    ) -> Vec<View<TreeviewItem>> {
        match element {
            ComponentElement::Div(div_element) => {
                let children = div_element.children.read(cx).clone();
                children
                    .iter()
                    .map(|child| {
                        TreeviewItem::new(
                            indent,
                            component,
                            child.clone(),
                            active_element.clone(),
                            cx,
                        )
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    fn observe_text(element: &ComponentElement, cx: &mut ViewContext<Self>) {
        if let ComponentElement::Text(element) = element {
            let text_property = element.properties.get("text").unwrap();
            cx.observe(text_property, |this, _, cx| {
                this.cached_text = get_element_text(&this.element, cx);
                cx.notify();
            })
            .detach();
        }
    }

    fn observe_children(element: &ComponentElement, cx: &mut ViewContext<Self>) {
        if let ComponentElement::Div(element) = element {
            cx.observe(&element.children, |this, _, cx| {
                this.child_views = Self::make_child_views(
                    this.indent + 1,
                    &this.component,
                    &this.element,
                    &this.active_element,
                    cx,
                );
                cx.notify();
            })
            .detach();
        }
    }

    fn observe_active_element(active_element: &Model<Option<Uuid>>, cx: &mut ViewContext<Self>) {
        cx.observe(active_element, |this, active_element, cx| {
            this.active = Some(this.element.id()) == *active_element.read(cx);
            cx.notify();
        })
        .detach();
    }

    fn context_menu_actions(&self, cx: &mut ViewContext<Self>) -> Vec<Vec<ContextMenuAction>> {
        let mut actions = match self.element {
            ComponentElement::Div(_) => {
                vec![vec![
                    ContextMenuAction::new(
                        "New `div` child".to_string(),
                        cx.listener(|this, _, cx| {
                            if let ComponentElement::Div(element) = &this.element {
                                let new_element = DivElement::new(cx);
                                let id = new_element.id;
                                cx.update_model(&element.children, |children, cx| {
                                    children.push(new_element.into());
                                    cx.notify();
                                });
                                cx.update_model(&this.active_element, |active_element, cx| {
                                    *active_element = Some(id);
                                    cx.notify();
                                });
                            } else {
                                unreachable!();
                            }
                        }),
                    ),
                    ContextMenuAction::new(
                        "New `text` child".to_string(),
                        cx.listener(|this, _, cx| {
                            if let ComponentElement::Div(element) = &this.element {
                                let new_element = TextElement::new(cx);
                                let id = new_element.id;
                                cx.update_model(&element.children, |children, cx| {
                                    children.push(new_element.into());
                                    cx.notify();
                                });
                                cx.update_model(&this.active_element, |active_element, cx| {
                                    *active_element = Some(id);
                                    cx.notify();
                                });
                            } else {
                                unreachable!();
                            }
                        }),
                    ),
                ]]
            }
            ComponentElement::Text(_) => Vec::new(),
        };
        if self.component.read(cx).root.as_ref().unwrap().id() != self.element.id() {
            actions.push(vec![
                ContextMenuAction::new(
                    "Move up".to_string(),
                    cx.listener(|this, _, cx| {
                        let root = this.component.read(cx).root.clone().unwrap();
                        let parent: DivElement = root
                            .find_parent_recursive(this.element.id(), cx)
                            .unwrap()
                            .into();

                        let child_id = this.element.id();
                        this.active_element.update(cx, |active_element, cx| {
                            *active_element = Some(child_id);
                            cx.notify();
                        });

                        let children = parent.children.read(cx).clone();
                        if let Some(pos) = children.iter().position(|child| child.id() == child_id)
                        {
                            if pos > 0 {
                                parent.children.update(cx, |children, cx| {
                                    children.swap(pos, pos - 1);
                                    cx.notify();
                                });
                            } else if let Some(grandparent) =
                                root.find_parent_recursive(parent.id, cx)
                            {
                                let grandparent: DivElement = grandparent.into();

                                parent.children.update(cx, |children, cx| {
                                    let child = children.remove(pos);
                                    cx.notify();

                                    grandparent.children.update(cx, |grandchildren, cx| {
                                        let parent_pos = grandchildren
                                            .iter()
                                            .position(|p| p.id() == parent.id)
                                            .unwrap();
                                        grandchildren.insert(parent_pos, child);
                                        cx.notify();
                                    });
                                });
                            }
                        }
                    }),
                ),
                ContextMenuAction::new(
                    "Move down".to_string(),
                    cx.listener(|this, _, cx| {
                        let root = this.component.read(cx).root.clone().unwrap();
                        let parent: DivElement = root
                            .find_parent_recursive(this.element.id(), cx)
                            .unwrap()
                            .into();

                        let child_id = this.element.id();
                        this.active_element.update(cx, |active_element, cx| {
                            *active_element = Some(child_id);
                            cx.notify();
                        });

                        let children = parent.children.read(cx).clone();
                        if let Some(pos) = children.iter().position(|child| child.id() == child_id)
                        {
                            if pos < children.len() - 1 {
                                parent.children.update(cx, |children, cx| {
                                    children.swap(pos, pos + 1);
                                    cx.notify();
                                });
                            } else if let Some(grandparent) =
                                root.find_parent_recursive(parent.id, cx)
                            {
                                let grandparent: DivElement = grandparent.into();

                                parent.children.update(cx, |children, cx| {
                                    let child = children.remove(pos);
                                    cx.notify();

                                    grandparent.children.update(cx, |grandchildren, cx| {
                                        let parent_pos = grandchildren
                                            .iter()
                                            .position(|p| p.id() == parent.id)
                                            .unwrap();
                                        grandchildren.insert(parent_pos + 1, child);
                                        cx.notify();
                                    });
                                });
                            }
                        }
                    }),
                ),
            ]);
            actions.push(vec![ContextMenuAction::new(
                "Delete".to_string(),
                cx.listener(|this, _, cx| {
                    let root = this.component.read(cx).root.clone().unwrap();
                    let parent: DivElement = root
                        .find_parent_recursive(this.element.id(), cx)
                        .unwrap()
                        .into();
                    parent.children.update(cx, |children, cx| {
                        *children = children
                            .iter()
                            .filter(|child| child.id() != this.element.id())
                            .cloned()
                            .collect();
                        cx.notify();
                    });
                    this.active_element.update(cx, |active_element, cx| {
                        // TODO: Only switch to parent if the active element is a child
                        *active_element = Some(parent.id);
                        cx.notify();
                    });
                }),
            )]);
        }
        actions
    }
}

impl Render for TreeviewItem {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .px_2()
                    .when(self.hovered, |this| this.bg(*colors::LIST_ITEM_HOVER))
                    .when(self.active, |this| this.bg(*colors::LIST_ITEM_ACTIVE))
                    .children((0..self.indent).map(|_| {
                        div()
                            .min_w(*sizes::TREEVIEW_INDENT)
                            .border_l_1()
                            .border_color(*colors::BORDER)
                    }))
                    .child(self.cached_text.clone())
                    .id("treeview-item")
                    .on_mouse_up(
                        MouseButton::Right,
                        cx.listener(|this, event: &MouseUpEvent, cx| {
                            ContextMenuGlobal::activate(
                                event.position,
                                this.context_menu_actions(cx),
                                cx,
                            )
                        }),
                    )
                    .on_click(cx.listener(|this, event: &ClickEvent, cx| {
                        if event.down.button == MouseButton::Left {
                            cx.update_model(&this.active_element, |active_element, cx| {
                                *active_element = Some(this.element.id());
                                cx.notify();
                            })
                        }
                    }))
                    .on_hover(cx.listener(|this, hover, cx| {
                        this.hovered = *hover;
                        cx.notify();
                    })),
            )
            .children(self.child_views.clone())
    }
}

fn get_element_text(element: &ComponentElement, cx: &AppContext) -> String {
    match element {
        ComponentElement::Div(_) => "div:".to_string(),
        ComponentElement::Text(element) => {
            let text_property = element.properties.get("text").unwrap().read(cx).1.clone();
            let text_property: String = text_property.into();
            format!("\"{text_property}\"")
        }
    }
}
