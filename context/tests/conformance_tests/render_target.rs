use cac_context::{render_target::Native, Context, RenderTarget};
use cac_core::{
    image::{self, Format, Image},
    math::URect,
    Color32,
};

use crate::{runner::TestCase, TestResult};

const FOUR_COLOR_VP: &[u8] = include_bytes!("res/screen_viewport.png");

fn sample(img1: &Image, img2: &Image, x: u32, y: u32) -> bool {
    let pixel1 = img1.sample(x, y).unwrap();
    let pixel2 = img2.sample(x, y).unwrap();

    println!("{pixel1:?} == {pixel2:?}");

    pixel1 == pixel2
}

pub fn tests() -> Vec<TestCase> {
    vec![TEST!(clear_color), TEST!(screen_viewport)]
}

fn clear_color(ctx: &mut impl Context) -> TestResult {
    const COLOR: Color32 = Color32::RED;
    let handle = ctx.create_render_target(RenderTarget {
        clear_color: Some(COLOR),
        viewport: ctx.viewport(),
    })?;

    if let Some(rt) = ctx.render_target_mut(handle) {
        rt.clear();
    } else {
        error!("render target not found")
    }

    ctx.update();

    if let Some(rt) = ctx.render_target(handle) {
        let screenshot = rt.read_pixels(Format::RgbF32, ctx.viewport())?;
        let comparison = image::Image::with_color32(8, 8, COLOR, Format::RgbF32)?;

        let hash1 = screenshot.hash()?;
        let hash2 = comparison.hash()?;

        let pixel1 = screenshot.sample(4, 3);
        let pixel2 = comparison.sample(4, 3);

        check!((hash1 ^ hash2).count_ones() <= 1);
        check!(pixel1 == pixel2);
    } else {
        error!("render target not found")
    }

    Ok(())
}

fn screen_viewport(ctx: &mut impl Context) -> TestResult {
    let mut viewport = URect {
        x: 0,
        y: 0,
        width: crate::CONTEXT_WIDTH,
        height: crate::CONTEXT_HEIGHT,
    };

    let screen = ctx.create_render_target(cac_context::RenderTarget {
        clear_color: None,
        viewport,
    })?;

    //half screen
    viewport.width = crate::CONTEXT_WIDTH / 2;
    viewport.height = crate::CONTEXT_HEIGHT / 2;

    if let Some(screen) = ctx.render_target_mut(screen) {
        //TL
        screen.set_clear_color(Color32::GAINSBORO.into());
        screen.set_viewport(viewport);
        screen.clear();
        //TR
        viewport.x = 400;
        screen.set_clear_color(Color32::PERSIAN_INDIGO.into());
        screen.set_viewport(viewport);
        screen.clear();
        //BL
        viewport.x = 0;
        viewport.y = 300;
        screen.set_clear_color(Color32::UNITY_YELLOW.into());
        screen.set_viewport(viewport);
        screen.clear();
        //BR
        viewport.x = 400;
        viewport.y = 300;
        screen.set_clear_color(Color32::DARK_JUNGLE_GREEN.into());
        screen.set_viewport(viewport);
        screen.clear();
    }

    ctx.update();

    if let Some(screen) = ctx.render_target(screen) {
        let img = screen.read_pixels(Format::RgbU8, ctx.viewport())?;
        let hash = img.hash()?;

        let golden_screenshot = Image::load_from_memory(Format::RgbU8, FOUR_COLOR_VP)?;

        let hash2 = golden_screenshot.hash()?;

        check!((hash ^ hash2).count_ones() <= 1);
        check!(sample(&img, &golden_screenshot, 0, 0));
        check!(sample(
            &img,
            &golden_screenshot,
            img.width - 1,
            img.height - 1
        ));
        check!(sample(&img, &golden_screenshot, img.width - 1, 0));
        check!(sample(&img, &golden_screenshot, 0, img.height - 1));
    }

    Ok(())
}
