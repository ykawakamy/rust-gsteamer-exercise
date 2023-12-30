use glib;
use gst;
use gst::subclass::prelude::*;
use gst_base::subclass::prelude::BaseTransformImpl;
use gst_video::prelude::*;
use gst_video::subclass::prelude::*;
use gst_video::VideoFormat;
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
pub struct SimpleTrans {}

#[glib::object_subclass]
impl ObjectSubclass for SimpleTrans {
    const NAME: &'static str = CLASS_NAME;
    type Type = super::SimpleTrans;
    // どのクラスを継承するか。
    // 時前で全て実装する場合はgst::Elementを使うが、パターンが明らかなら
    // gst_baseのBaseSrc,BaseSink,BaseTransform,GstAggregatorなどを選ぶのがよい
    type ParentType = gst_base::BaseTransform;
}

impl ObjectImpl for SimpleTrans {}

impl GstObjectImpl for SimpleTrans {}

impl ElementImpl for SimpleTrans {
  // gst-inspectから見れるエレメントの基本情報
  fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
      static ELEMENT_METADATA: Lazy<gst::subclass::ElementMetadata> = Lazy::new(|| {
          gst::subclass::ElementMetadata::new(
              CLASS_NAME,
              "Generic",
              "simple transform",
              "FUJINAKA Fumiya <uzuna.kf@gmail.com>",
          )
      });

      Some(&*ELEMENT_METADATA)
  }

  // このエレメントがどのようなPadを持つのかを宣言する
  fn pad_templates() -> &'static [gst::PadTemplate] {
      static PAD_TEMPLATES: Lazy<Vec<gst::PadTemplate>> = Lazy::new(|| {
          let caps = gst::Caps::new_any();
          let src_pad_template = gst::PadTemplate::new(
              "src",
              gst::PadDirection::Src,
              gst::PadPresence::Always,
              &caps,
          )
          .unwrap();

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
  // AlwaysInPlaceなので入力されたGstBufferを使いそのままsinkに流す
  const MODE: gst_base::subclass::BaseTransformMode =
      gst_base::subclass::BaseTransformMode::AlwaysInPlace;
  // 文字通りPassThroughするかどうか
  const PASSTHROUGH_ON_SAME_CAPS: bool = true;
  const TRANSFORM_IP_ON_PASSTHROUGH: bool = true;

  // AlwaysInPlace以外の場合に実装が必要
  // inbufの内容を加工してoutbufに書き込みOkを返すとsinkに送られる
  fn transform(
      &self,
      _inbuf: &gst::Buffer,
      _outbuf: &mut gst::BufferRef,
  ) -> Result<gst::FlowSuccess, gst::FlowError> {
      gst::info!(CAT, "transform");
      Ok(gst::FlowSuccess::Ok)
  }

  // transformの入力されたGstBufferに手を加える、あるいは何もせずにsinkに流す
  // 例えばカラー変更などバッファのサイズが変わらないなどの場合に使う
  fn transform_ip(&self, _buf: &mut gst::BufferRef) -> Result<gst::FlowSuccess, gst::FlowError> {
      gst::info!(CAT, "transform_ip");
      Ok(gst::FlowSuccess::Ok)
  }

  // PassThroughは基本的にはGstBufferを無駄に処理せずにただsinkに流すためにある
  // 例えばカラー変更でsrcとsinkのCapabilityが同じ場合は処理の必要がない
  // このような処理を分けたい場合にPASSTHROUGH_ON_SAME_CAPS=trueにして利用する
  fn transform_ip_passthrough(
      &self,
      _buf: &gst::Buffer,
  ) -> Result<gst::FlowSuccess, gst::FlowError> {
      gst::info!(CAT, "transform_ip_passthrough");
      Ok(gst::FlowSuccess::Ok)
  }
}