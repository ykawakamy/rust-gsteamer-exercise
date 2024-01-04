use gst::glib;
use gst::prelude::*;

mod imp;

glib::wrapper! {
    pub struct RoundedCorners(ObjectSubclass<imp::RoundedCorners>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        "roundedcorners",
        gst::Rank::NONE,
        RoundedCorners::static_type(),
    )
}

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    register(plugin)?;
    Ok(())
}

gst::plugin_define!(
    roundedcorners,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION")),
    "Private",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY")
);
