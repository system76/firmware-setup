// SPDX-License-Identifier: GPL-3.0-only

use core::cell::Cell;
use orbclient::{Color, Mode, Renderer};
use std::prelude::*;
use std::proto::Protocol;
use std::uefi::graphics::{GraphicsBltOp, GraphicsBltPixel, GraphicsOutput};
use std::uefi::guid::GRAPHICS_OUTPUT_PROTOCOL_GUID;

pub struct Output(pub &'static mut GraphicsOutput);

impl Protocol<GraphicsOutput> for Output {
    fn guid() -> Guid {
        GRAPHICS_OUTPUT_PROTOCOL_GUID
    }

    fn new(inner: &'static mut GraphicsOutput) -> Self {
        Output(inner)
    }
}

pub struct Display {
    output: Output,
    w: u32,
    h: u32,
    data: Box<[Color]>,
    mode: Cell<Mode>,
}

impl Display {
    pub fn new(output: Output) -> Self {
        let w = output.0.Mode.Info.HorizontalResolution;
        let h = output.0.Mode.Info.VerticalResolution;
        Self {
            output,
            w,
            h,
            data: vec![Color::rgb(0, 0, 0); w as usize * h as usize].into_boxed_slice(),
            mode: Cell::new(Mode::Blend),
        }
    }

    pub fn blit(&mut self, x: i32, y: i32, w: u32, h: u32) -> bool {
        let status = (self.output.0.Blt)(
            self.output.0,
            self.data.as_mut_ptr() as *mut GraphicsBltPixel,
            GraphicsBltOp::BufferToVideo,
            x as usize,
            y as usize,
            x as usize,
            y as usize,
            w as usize,
            h as usize,
            0,
        );
        status.is_success()
    }
}

impl Renderer for Display {
    fn width(&self) -> u32 {
        self.w
    }

    fn height(&self) -> u32 {
        self.h
    }

    fn data(&self) -> &[Color] {
        &self.data
    }

    fn data_mut(&mut self) -> &mut [Color] {
        &mut self.data
    }

    fn sync(&mut self) -> bool {
        let w = self.width();
        let h = self.height();
        self.blit(0, 0, w, h)
    }

    fn mode(&self) -> &Cell<Mode> {
        &self.mode
    }
}
