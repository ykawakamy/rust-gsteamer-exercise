use std::{sync::Mutex, cell::RefCell};

use glib;
use gst;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::BaseTransformImpl;
use gst_video::{subclass::prelude::VideoFilterImpl, VideoFrameExt};
use once_cell::sync::Lazy;

use super::{CLASS_NAME, ELEMENT_NAME};

// GStreamerのログで識別、フィルタリングするためのログカテゴリ情報
static CAT: Lazy<gst::DebugCategory> = Lazy::new(|| {
    gst::DebugCategory::new(
        ELEMENT_NAME,
        gst::DebugColorFlags::empty(),
        Some("SimpleTrans Element"),
    )
});

#[derive(Default)] 
pub struct SimpleTrans {
    prev_data: Mutex<RefCell<Vec<u8>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for SimpleTrans {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::SimpleTrans;
    type ParentType = gst_video::VideoFilter;
}

impl ObjectImpl for SimpleTrans {}

impl GstObjectImpl for SimpleTrans {}

impl ElementImpl for SimpleTrans {
    fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
        static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
            gst::subclass::ElementMetadata::new(
                "RGB-GRAY Converter",
                "Filter/Effect/Converter/Video",
                "Converts RGB to GRAY or grayscale RGB",
                "Sebastian Dröge <sebastian@centricular.com>",
            )
        });

        Some(&*ELEMENT_METADATA)
    }

    fn pad_templates() -> &'static [gst::PadTemplate] {
        static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
            let caps = gst_video::VideoCapsBuilder::new()
                .format_list([gst_video::VideoFormat::Bgrx, gst_video::VideoFormat::Gray8])
                .build();
            let src_pad_template = gst::PadTemplate::new(
                "src",
                gst::PadDirection::Src,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            let caps = gst_video::VideoCapsBuilder::new()
                .format(gst_video::VideoFormat::Bgrx)
                .build();
            let sink_pad_template = gst::PadTemplate::new(
                "sink",
                gst::PadDirection::Sink,
                gst::PadPresence::Always,
                &caps,
            )
            .unwrap();

            vec![src_pad_template, sink_pad_template]
        });

        PAD_TEMPLATES.as_ref()
    }
}

impl BaseTransformImpl for SimpleTrans {
    const MODE: gst_base::subclass::BaseTransformMode =
        gst_base::subclass::BaseTransformMode::NeverInPlace;
    const PASSTHROUGH_ON_SAME_CAPS: bool = false;
    const TRANSFORM_IP_ON_PASSTHROUGH: bool = false;

    fn transform_caps(
        &self,
        direction: gst::PadDirection,
        caps: &gst::Caps,
        filter: Option<&gst::Caps>,
    ) -> Option<gst::Caps> {
        let other_caps = if direction == gst::PadDirection::Src {
            let mut caps = caps.clone();

            for s in caps.make_mut().iter_mut() {
                s.set("format", gst_video::VideoFormat::Bgrx.to_str());
            }

            caps
        } else {
            let mut gray_caps = gst::Caps::new_empty();

            {
                let gray_caps = gray_caps.get_mut().unwrap();

                for s in caps.iter() {
                    let mut s_gray = s.to_owned();
                    s_gray.set("format", gst_video::VideoFormat::Gray8.to_str());
                    gray_caps.append_structure(s_gray);
                }
                gray_caps.append(caps.clone());
            }

            gray_caps
        };

        gst::debug!(
            CAT,
            imp: self,
            "Transformed caps from {} to {} in direction {:?}",
            caps,
            other_caps,
            direction
        );

        if let Some(filter) = filter {
            Some(filter.intersect_with_mode(&other_caps, gst::CapsIntersectMode::First))
        } else {
            Some(other_caps)
        }
    }
}

impl VideoFilterImpl for SimpleTrans {
    fn transform_frame(
        &self,
        in_frame: &gst_video::VideoFrameRef<&gst::BufferRef>,
        out_frame: &mut gst_video::VideoFrameRef<&mut gst::BufferRef>,
    ) -> Result<gst::FlowSuccess, gst::FlowError> {
        let width = in_frame.width() as usize;
        let in_stride = in_frame.plane_stride()[0] as usize;
        let in_data = in_frame.plane_data(0).unwrap();
        let out_stride = out_frame.plane_stride()[0] as usize;
        let out_format = out_frame.format();
        let out_data = out_frame.plane_data_mut(0).unwrap();
        let prev_data_ref = self.prev_data.lock().unwrap().try_into();
        let prev_data = match prev_data_ref {
            Ok => prev_data_ref,
            _ => in_data,
        };

        if out_format == gst_video::VideoFormat::Bgrx {
            assert_eq!(in_data.len() % 4, 0);
            assert_eq!(out_data.len() % 4, 0);
            assert_eq!(out_data.len() / out_stride, in_data.len() / in_stride);

            let in_line_bytes = width * 4;
            let out_line_bytes = width * 4;

            assert!(in_line_bytes <= in_stride);
            assert!(out_line_bytes <= out_stride);

            for (in_line, out_line) in in_data
                .chunks_exact(in_stride)
                .zip(out_data.chunks_exact_mut(out_stride))
            {
                for (in_p, out_p) in in_line[..in_line_bytes]
                    .chunks_exact(4)
                    .zip(out_line[..out_line_bytes].chunks_exact_mut(4))
                {
                    assert_eq!(out_p.len(), 4);

                    let gray = SimpleTrans::bgrx_to_gray(in_p);
                    out_p[0] = gray;
                    out_p[1] = gray;
                    out_p[2] = gray;
                }
            }
        } else if out_format == gst_video::VideoFormat::Gray8 {
            assert_eq!(in_data.len() % 4, 0);
            assert_eq!(out_data.len() / out_stride, in_data.len() / in_stride);

            let in_line_bytes = width * 4;
            let out_line_bytes = width;

            assert!(in_line_bytes <= in_stride);
            assert!(out_line_bytes <= out_stride);

            for (in_line, out_line) in in_data
                .chunks_exact(in_stride)
                .zip(out_data.chunks_exact_mut(out_stride))
            {
                for (in_p, out_p) in in_line[..in_line_bytes]
                    .chunks_exact(4)
                    .zip(out_line[..out_line_bytes].iter_mut())
                {
                    let gray = SimpleTrans::bgrx_to_gray(in_p);
                    *out_p = gray;
                }
            }
        } else {
            unimplemented!();
        }

        prev_data_ref.replace(in_data.to_vec());

        Ok(gst::FlowSuccess::Ok)
    }
}

impl SimpleTrans {
    #[inline]
    fn bgrx_to_gray(in_p: &[u8]) -> u8 {
        // See https://en.wikipedia.org/wiki/YUV#SDTV_with_BT.601
        const R_Y: u32 = 19595; // 0.299 * 65536
        const G_Y: u32 = 38470; // 0.587 * 65536
        const B_Y: u32 = 7471; // 0.114 * 65536

        assert_eq!(in_p.len(), 4);

        let b = u32::from(in_p[0]);
        let g = u32::from(in_p[1]);
        let r = u32::from(in_p[2]);

        let gray = ((r * R_Y) + (g * G_Y) + (b * B_Y)) / 65536;

        gray as u8
    }
}
