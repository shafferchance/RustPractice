// use chips_8::scenes::rectangle::create_rectangle_scene;
use chips_8::scenes::textured::create_textured_scene;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

use chips_8::gfx::init::load_gl;

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Chips 8").with_inner_size(glutin::dpi::LogicalSize::new(1024, 512));

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    println!("Pixel format of the window's GL context: {:?}", windowed_context.get_pixel_format());
    let gl = load_gl(&windowed_context.context());

    let scene = create_textured_scene(&gl);

    el.run(move |event, _, control_flow| {
        // println!("{:?}", event);
        *control_flow = ControlFlow::Wait;

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
            _ => (),
        }
    });
}
