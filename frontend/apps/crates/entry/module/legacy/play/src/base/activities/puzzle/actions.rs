use super::state::*;
use dominator::clone;
use std::rc::Rc;
use utils::{prelude::*, resize::ResizeInfo};
use components::traces::canvas::draw_single_shape;
use wasm_bindgen::prelude::*;

impl Puzzle {
    pub fn on_start(self: Rc<Self>) {
        let state = self;

        state.base.allow_stage_click();
    }

    pub fn create_pieces(&self) {

    }

    pub fn render_cutouts(&self) {

    }
}

impl PuzzleGame {
    pub fn on_start(self: Rc<Self>) {
        let state = self;

        state.base.allow_stage_click();
    }

    pub fn draw(&self, resize_info: &ResizeInfo) {
        let src_ctx = &self.effects.ctx;
        let dest_canvas = &self.cutouts_canvas;
        let dest_ctx = &self.cutouts_ctx;


        dest_canvas.set_width(resize_info.width as u32);
        dest_canvas.set_height(resize_info.height as u32);

        dest_ctx.draw_image_with_html_image_element_and_dw_and_dh(&self.effects.image_element, 0.0, 0.0, resize_info.width, resize_info.height).unwrap_ji();
        //draw the cutouts
        dest_ctx.set_fill_style(&JsValue::from_str("black"));
        for item in self.items.iter() {
            draw_single_shape(dest_ctx, resize_info, &item.raw.hotspot.shape, );
        }

        //TODO draw the items
        //need to somehow take the source image... maybe composite with masking?
        for item in self.items.iter() {
        }
    }
}
