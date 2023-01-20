use cac_core::math::URect;

use cac_context::Context;

use crate::{runner::TestCase, TestResult};

pub fn tests() -> Vec<TestCase> {
    vec![TEST!(viewport_is_window_size)]
}

fn viewport_is_window_size(ctx: &impl Context) -> TestResult {
    let view_port = ctx.viewport();

    check!(view_port == URect::new(0, 0, crate::CONTEXT_WIDTH, crate::CONTEXT_HEIGHT));

    Ok(())
}
