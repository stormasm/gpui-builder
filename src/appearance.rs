pub mod colors {
    use gpui::{rgb, rgba, Rgba};
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref BG: Rgba = rgb(0x181818);
        pub static ref BORDER: Rgba = rgb(0x505050);
        pub static ref SIDEBAR_BG: Rgba = rgb(0x181818);
        pub static ref TEXT: Rgba = rgb(0xb0b0b0);
        pub static ref LIST_ITEM_HOVER: Rgba = rgba(0xffffff04);
        pub static ref ACTIVE_ELEMENT_INDICATOR_MAX: Rgba = rgba(0xff000080);
    }
}

pub mod sizes {
    use gpui::{px, AbsoluteLength};
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref TEXT_SIZE: AbsoluteLength = px(14.0).into();
        pub static ref SIDEBAR_WIDTH: AbsoluteLength = px(250.0).into();
        pub static ref TREEVIEW_INDENT: AbsoluteLength = px(32.0).into();
    }
}
