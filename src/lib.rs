#![deny(clippy::all)]

use std::io::Cursor;

use image::GenericImageView;
use napi::bindgen_prelude::{AsyncTask, Buffer, Task};
use napi::{Env, Error, Result, Status};
use napi_derive::napi;
use oxipng::{Options, StripChunks};

#[napi(object)]
pub struct CompressOptions {
  pub width: Option<u32>,
  pub level: Option<u8>,
  pub strip: Option<String>,
}

pub struct CompressTask {
  data: Vec<u8>,
  width: Option<u32>,
  level: Option<u8>,
  strip: Option<StripChunks>,
}

impl CompressTask {
  fn new(buffer: Buffer, options: Option<CompressOptions>) -> Result<Self> {
    let data: Vec<u8> = buffer.into();
    let (width, level, strip) = match options {
      Some(opts) => (opts.width, opts.level, opts.strip),
      None => (None, None, None),
    };

    if let Some(width) = width {
      if width == 0 {
        return Err(Error::new(
          Status::InvalidArg,
          "width must be greater than 0".to_string(),
        ));
      }
    }

    if let Some(level) = level {
      if level > 6 {
        return Err(Error::new(
          Status::InvalidArg,
          "level must be between 0 and 6".to_string(),
        ));
      }
    }

    let strip = match strip {
      Some(value) => Some(parse_strip(&value)?),
      None => None,
    };

    Ok(Self {
      data,
      width,
      level,
      strip,
    })
  }
}

impl Task for CompressTask {
  type Output = Vec<u8>;
  type JsValue = Buffer;

  fn compute(&mut self) -> Result<Self::Output> {
    let mut data = std::mem::take(&mut self.data);

    if let Some(width) = self.width {
      let img = image::load_from_memory(&data)
        .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid PNG buffer: {e}")))?;

      let (orig_w, orig_h) = img.dimensions();
      if width < orig_w {
        let ratio = width as f64 / orig_w as f64;
        let mut new_h = (orig_h as f64 * ratio).round() as u32;
        if new_h == 0 {
          new_h = 1;
        }

        let resized = img.resize_exact(width, new_h, image::imageops::FilterType::Lanczos3);
        let mut resized_bytes = Vec::new();
        resized
          .write_to(
            &mut Cursor::new(&mut resized_bytes),
            image::ImageFormat::Png,
          )
          .map_err(|e| {
            Error::new(
              Status::GenericFailure,
              format!("Failed to encode resized PNG: {e}"),
            )
          })?;
        data = resized_bytes;
      }
    }

    let mut opts = match self.level {
      Some(level) => Options::from_preset(level),
      None => Options::default(),
    };

    if let Some(ref strip) = self.strip {
      opts.strip = strip.clone();
    }

    let optimized = oxipng::optimize_from_memory(&data, &opts)
      .map_err(|e| Error::new(Status::GenericFailure, format!("oxipng failed: {e}")))?;

    Ok(optimized)
  }

  fn resolve(&mut self, _env: Env, output: Vec<u8>) -> Result<Self::JsValue> {
    Ok(Buffer::from(output))
  }
}

fn parse_strip(value: &str) -> Result<StripChunks> {
  match value.to_ascii_lowercase().as_str() {
    "none" => Ok(StripChunks::None),
    "safe" => Ok(StripChunks::Safe),
    "all" => Ok(StripChunks::All),
    _ => Err(Error::new(
      Status::InvalidArg,
      "strip must be one of: 'none', 'safe', 'all'".to_string(),
    )),
  }
}

#[napi]
pub fn compress_png(
  buffer: Buffer,
  options: Option<CompressOptions>,
) -> Result<AsyncTask<CompressTask>> {
  Ok(AsyncTask::new(CompressTask::new(buffer, options)?))
}
