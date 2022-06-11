use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

use chips_8::gfx::init::load;

fn main() {
    // let el = EventLoop::new();
    // let wb = WindowBuilder::new().with_title("Chips 8");

    // let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    // let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    // println!("Pixel format of the window's GL context: {:?}", windowed_context.get_pixel_format());
    // let gl = load(&windowed_context.context());

    // el.run(move |event, _, control_flow| {
    //     // println!("{:?}", event);
    //     *control_flow = ControlFlow::Wait;

    //     match event {
    //         Event::LoopDestroyed => return,
    //         Event::WindowEvent { event, .. } => match event {
    //             WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
    //             WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
    //             _ => (),
    //         },
    //         Event::RedrawRequested(_) => {
    //             gl.draw_frame([0.2, 0.3, 0.3, 1.0]);
    //             windowed_context.swap_buffers().unwrap();
    //         },
    //         _ => (),
    //     }
    // });

    let thing = concat!(include_bytes!("./shaders/triangle.vert"), b"\0");
    println!(thing);
    
}
