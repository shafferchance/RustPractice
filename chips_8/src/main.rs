use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoopBuilder};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

use chips_8::gfx::core::{load_gl, chip_8_texture_to_opengl};
use chips_8::scenes::textured::create_scene_with_chips_8_text;
use chips_8::core::ops::MyChips8;

fn main() {
    let mut chips_8_state = MyChips8::new();

    let el = EventLoopBuilder::new().build();
    let wb = WindowBuilder::new()
                                            .with_title("Chips 8")
                                            .with_inner_size(
                                                glutin::dpi::LogicalSize::new(512, 512));

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    println!("Pixel format of the window's GL context: {:?}", windowed_context.get_pixel_format());
    let gl = load_gl(&windowed_context.context());

    let mut scene = create_scene_with_chips_8_text(&gl, &chips_8_state.gfx);
    let rom = include_bytes!("IBM_Logo.ch8");
    chips_8_state.load_rom(rom);

    let mut wait_next_loop = false;

    el.run(move |event, _, control_flow| {
        if wait_next_loop {
            *control_flow = ControlFlow::Wait;
        }

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                gl.draw_frame([0.2, 0.3, 0.3, 1.0], &scene);
                windowed_context.swap_buffers().unwrap();
            },
            Event::Resumed => {
                println!("Resumed");
                wait_next_loop = false;
                chips_8_state.wait = false;
            },
            Event::MainEventsCleared => {
                chips_8_state.enumlate_cycle();

                if chips_8_state.wait {
                    wait_next_loop = true;
                }

                if chips_8_state.draw {
                    chips_8_state.draw = false;
                    if let Some(texture) = &mut scene.objects[0].texture {
                        chip_8_texture_to_opengl(texture, &chips_8_state.gfx);
                    }
                    scene.render_scene_objects(&gl);
                    windowed_context.window().request_redraw();
                }
            },
            _ => (),
        }
    });
}
