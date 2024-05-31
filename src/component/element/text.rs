use gpui::{AppContext, Context};
use indexmap::IndexMap;
use uuid::Uuid;

use super::property::ElementProperty;
use super::ComponentElement;

#[derive(Clone)]
pub struct TextElement {
    pub id: Uuid,
    pub properties: IndexMap<String, ElementProperty>,
}

impl TextElement {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(text: &str, cx: &mut AppContext) -> ComponentElement {
        let mut properties = IndexMap::new();
        properties.insert("text".to_string(), text.to_string().into());

        ComponentElement::Text(cx.new_model(|_| {
            Self {
                id: Uuid::new_v4(),
                properties,
            }
        }))
    }
}
