//! Module that rasterizes text using `swash`.

use anyhow::{Result, anyhow};
use cosmic_text::{CacheKey, CacheKeyFlags};
use pathfinder_geometry::rect::RectI;
use pathfinder_geometry::vector::{Vector2F, Vector2I, vec2i};

use crate::fonts::canvas::{Canvas, RasterFormat};
use crate::fonts::{FontId, GlyphId, RasterizedGlyph, SubpixelAlignment};
use crate::platform::FontDB as _;
use crate::rendering::GlyphConfig;
use crate::windowing::winit::fonts::FontDB;

impl FontDB {
    pub(super) fn glyph_raster_bounds(
        &self,
        font_id: FontId,
        size: f32,
        glyph_id: GlyphId,
        scale: Vector2F,
        _glyph_config: &GlyphConfig,
    ) -> Result<RectI> {
        let id = *self
            .text_layout_system
            .font_id_map
            .read()
            .get_by_left(&font_id)
            .unwrap();
        let image = self
            .swash_cache
            .write()
            .get_image_uncached(
                &mut self.text_layout_system.font_store.write(),
                CacheKey::new(
                    id,
                    glyph_id as u16,
                    size * scale.x(),
                    (0., 0.),
                    CacheKeyFlags::empty(),
                )
                .0,
            )
            .clone()
            .ok_or_else(|| anyhow!("Failed to get raster image"))?;

        if image.placement.width == 0 || image.placement.height == 0 {
            // Nothing to rasterize. This happens for glyphs a font maps but does
            // not draw (space, and color glyphs whose outlines are empty).
            return Ok(RectI::new(Vector2I::zero(), Vector2I::zero()));
        }

        // Take the bounds from the rasterized image rather than from the
        // typographic outline: color glyphs (COLR/CPAL layers, CBDT/CBLC
        // bitmaps) typically have no outline of their own, so their typographic
        // bounds are empty even though they draw perfectly well.
        let origin = vec2i(image.placement.left, -image.placement.top);
        let size = vec2i(image.placement.width as i32, image.placement.height as i32);
        Ok(RectI::new(origin, size))
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn rasterize_glyph(
        &self,
        font_id: FontId,
        size: f32,
        glyph_id: GlyphId,
        scale: Vector2F,
        subpixel_alignment: SubpixelAlignment,
        glyph_config: &GlyphConfig,
        requested_format: RasterFormat,
    ) -> Result<RasterizedGlyph> {
        let raster_bounds =
            self.glyph_raster_bounds(font_id, size, glyph_id, scale, glyph_config)?;

        let id = *self
            .text_layout_system
            .font_id_map
            .read()
            .get_by_left(&font_id)
            .unwrap();
        // Get the raster image without caching--the parent FontDB handles all caching for us.
        let image = self
            .swash_cache
            .write()
            .get_image_uncached(
                &mut self.text_layout_system.font_store.write(),
                CacheKey::new(
                    id,
                    glyph_id as u16,
                    size * scale.x(),
                    (subpixel_alignment.to_offset().x(), 0.),
                    CacheKeyFlags::empty(),
                )
                .0,
            )
            .clone()
            .unwrap();

        let (original_format, is_color) = match image.content {
            cosmic_text::SwashContent::Mask => (RasterFormat::A8, false),
            cosmic_text::SwashContent::SubpixelMask => (RasterFormat::Rgba32, false),
            cosmic_text::SwashContent::Color => (RasterFormat::Rgba32, true),
        };

        // Ensure the pixmap is in the correct requested format (in practice this converts A8 to
        // RGBA32).
        // TODO(alokedesai): Ensure our font rasterization code is robust to returned formats that
        // are different than incoming formats. Right now, we create text bounds based on the
        // _incoming_ format.
        let pixmap = if original_format == RasterFormat::A8 {
            let bytes_per_pixel = requested_format.bytes_per_pixel() as usize;
            let mut pixmap = Vec::with_capacity(image.data.len() * bytes_per_pixel);
            for byte in image.data {
                for _ in 0..bytes_per_pixel {
                    pixmap.push(byte);
                }
            }
            pixmap
        } else {
            image.data
        };

        let canvas = Canvas {
            pixels: pixmap,
            size: raster_bounds.size(),
            row_stride: image.placement.width as usize * original_format.bytes_per_pixel() as usize,
            format: RasterFormat::Rgba32,
        };

        anyhow::Ok(RasterizedGlyph {
            canvas,
            is_emoji: is_color,
        })
    }
}
