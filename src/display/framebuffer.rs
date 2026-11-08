//! Framebuffer text renderer.
//!
//! Provides a [`FrameBuffer`] type that owns a pointer to the linear
//! framebuffer, tracks cursor state, and implements [`core::fmt::Write`]
//! for formatted text output with automatic line wrapping and scrolling.

use alloc::vec::Vec;
use core::fmt::{Result, Write};

use crate::display::{cell::Cell, font};

pub struct FrameBuffer {
    address: *mut u32,
    width: usize,
    height: usize,
    stride: usize,
    cursor_x: usize,
    cursor_y: usize,
    cells: Vec<Cell>,
}

// SAFETY: FrameBuffer is only accessed through a Mutex<FrameBuffer> (FRAMEBUFFER
// static in boot::info), which serializes all reads and writes to the raw pointer.
unsafe impl Sync for FrameBuffer {}
unsafe impl Send for FrameBuffer {}

impl FrameBuffer {
    pub fn new(address: *mut u32, width: usize, height: usize, stride: usize) -> Self {
        FrameBuffer {
            address,
            width,
            height,
            stride,
            cursor_x: 0,
            cursor_y: 0,
            cells: Vec::new(),
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    fn newline(&mut self) {
        self.cells.clear();

        self.cursor_x = 0;

        let raster_h = font::SIZE.val();

        if self.cursor_y + raster_h >= self.height {
            self.scroll_up();
            return;
        }

        self.cursor_y += raster_h;
    }

    fn backspace(&mut self) {
        let Some(cell) = self.cells.pop() else {
            return;
        };

        self.cursor_x = cell.start_x();
        self.cursor_y = cell.start_y();

        for y in cell.start_y()..cell.start_y() + font::SIZE.val() {
            for x in cell.start_x()..cell.start_x() + cell.width() {
                if x < self.width && y < self.height {
                    self.put_pixel(x, y, 0);
                }
            }
        }
    }

    fn scroll_up(&mut self) {
        let raster_h = font::SIZE.val();

        let scroll_pixels = raster_h * self.stride;
        let total_pixels = self.stride * self.height;

        // SAFETY: The framebuffer address and stride are provided by the
        // bootloader and remain valid for the kernel's lifetime. The copy
        // region is within bounds (total_pixels covers the entire buffer).
        unsafe {
            core::ptr::copy(
                self.address.add(scroll_pixels),
                self.address,
                total_pixels - scroll_pixels,
            );

            core::ptr::write_bytes(
                self.address.add(total_pixels - scroll_pixels),
                0,
                scroll_pixels,
            );
        }
    }

    fn put_pixel(&self, x: usize, y: usize, color: u32) {
        // SAFETY: Callers bounds-check x < width and y < height before
        // invoking put_pixel, so the computed offset stays within the
        // framebuffer memory region.
        unsafe { *self.address.add(y * self.stride + x) = color }
    }
}

impl Write for FrameBuffer {
    fn write_str(&mut self, s: &str) -> Result {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result {
        if c == '\n' {
            self.newline();
            return Ok(());
        }

        if c == '\u{8}' {
            self.backspace();
            return Ok(());
        }

        let Some(glyph) = noto_sans_mono_bitmap::get_raster(c, font::WEIGHT, font::SIZE) else {
            return Ok(());
        };

        let start_x = self.cursor_x;
        let start_y = self.cursor_y;

        self.cells.push(Cell::new(start_x, start_y, glyph.width()));

        let rows = glyph.rows();

        for (row, columns) in rows.enumerate() {
            for (column, intensity) in columns.iter().enumerate() {
                if *intensity == 0 {
                    continue;
                }

                let x = self.cursor_x + column;
                let y = self.cursor_y + row;

                if x >= self.width || y >= self.height {
                    continue;
                }

                let value = *intensity as u32;
                let rgba = (value << 16) | (value << 8) | value;

                self.put_pixel(x, y, rgba);
            }
        }

        self.cursor_x += glyph.width();

        if self.cursor_x + glyph.width() >= self.width {
            self.newline();
        }

        Ok(())
    }
}
