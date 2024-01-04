use gst::glib;
use gst::prelude::*;

mod imp;

// The public Rust wrapper type for our element
glib::wrapper! {
    pub struct Rgb2Gray(ObjectSubclass<imp::Rgb2Gray>) @extends gst_base::BaseTransform, gst::Element, gst::Object;
}

// Registers the type for our element, and then registers in GStreamer under
// the name "rsrgb2gray" for being able to instantiate it via e.g.
// gst::ElementFactory::make().
pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    gst::Element::register(
        Some(plugin),
        "rsrgb2gray",
        gst::Rank::NONE,
        Rgb2Gray::static_type(),
    )
}

fn plugin_init(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
    register(plugin)?;
    Ok(())
}

gst::plugin_define!(
    rsrgb2gray,
    env!("CARGO_PKG_DESCRIPTION"),
    plugin_init,
    concat!(env!("CARGO_PKG_VERSION")),
    "Private",
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_NAME"),
    env!("CARGO_PKG_REPOSITORY")
);
