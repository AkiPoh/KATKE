// Window management and event handling depencies using the "winit" crate
// Winit provides cross-platform window creation and event handling
use winit::{

    // Import types for handling system and window events
    event::{

        Event,
        // - Is the enum that represents all possible events that can occur (window events, device events, etc.)
        WindowEvent
        // - Specifically handles window-related events like resizing, closing, keyboard/mouse input, etc.
    },

    // Import core event handling system
    event_loop::EventLoop,

    // Import builder pattern for window creation and configuration
    window::WindowBuilder,
};

// Main async function that sets up and runs the application
async fn run() {

    // Create a new event loop to handle window events
    let event_loop = EventLoop::new().unwrap();
        // - "unwrap()" crashes the application if operation fails; if operation succeeds no effect

    // Create and configure the application window
    let window = WindowBuilder::new()

        .with_title("KATKE")
        // - Titles window

        .build(&event_loop)
        // - Connect with a reference to the "event_loop" variable

        .unwrap()
        // - Crashes the application if operation fails; if operation succeeds no effect
        ;

    // Create variable and get window's physical size in pixels
    let size = window.inner_size();

    // Initialize a new wgpu instance with default settings,
    // this is our connection to the GPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

    // Create a surface for rendering,
    // a surface is part of the window we'll draw to
    let surface = instance.create_surface(&window).unwrap();

    // Request a GPU from computer, that meets the specified requirements
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {

        power_preference: wgpu::PowerPreference::default(),
        // - Request GPU with computer's default power profile

        force_fallback_adapter: false,
        // - Don't use software fallback if no GPU available

        compatible_surface: Some(&surface),
        // - GPU compatible with our surface
        })

        .await
        // - Waits here until operation completed

        .unwrap()
        // - Crashes the application if operation fails; if operation succeeds no effect
        ;

    // Request a connection to use the GPU ("device") and a way to send commands "queue"
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor{
            
            // # "device":

            label: None,
            // - No debug label

            required_features: wgpu::Features::empty(),
            // - No special features required

            required_limits: wgpu::Limits::default(),
            // - Use default GPU resource limits
        },
        
        // # "queue"

        None,
        // - No trace path for GPU command ("queue") debugging
        )

        .await
        // - Waits here until operation completed

        .unwrap()
        // - Crashes the application if operation fails; if operation succeeds no effect
        ;

    // Get the capabilities of our surface (what formats/settings it supports)
    let surface_capabilities = surface.get_capabilities(&adapter);

    // Find the supported surface color format, prefer sRGB if available
    let surface_color_format = surface_capabilities

        .formats
        // - Get list of all the color formats supported by the GPU

        .iter()
        // - Iterate over all color formats from the list...

        .copied()
        // - Make copies of each each color format as we iterate

        .find(|f| f.is_srgb())
        // - Look for the first color format that supports sRGB

        .unwrap_or(surface_capabilities.formats[0])
        // - If no sRGB format found use first available format instead
        ;

    // Configure the surface for rendering
    let mut config = wgpu::SurfaceConfiguration {

        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        // - Tells GPU this surface is for directly drawing to

        format: surface_color_format,
        // - Use the color format we selected 

        width: size.width,
        height: size.height,
        // - Tell GPU our window's size in pixels

        present_mode: wgpu::PresentMode::Fifo,
        // - Display rendered frames in order, synchronized with screen refresh rate...
        //   similar to VSync in games avoiding frame tearing

        alpha_mode: surface_capabilities.alpha_modes[0],
        // - Use first suppported transparency mode supported by the GPU, for handling transparency

        view_formats: vec![],
        // - No additional format views needed

        desired_maximum_frame_latency: 2,
        // - Number of frames GPU can prepare ahead
    };

    surface.configure(&device, &config);

    let window_ref = &window;
    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                target.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                config.width = new_size.width;
                config.height = new_size.height;
                surface.configure(&device, &config);
            }
            Event::AboutToWait => {
                window_ref.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let frame = surface.get_current_texture().unwrap();
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
                    label: Some("Render Encoder")
                });

                {
                    let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.4, // Slightly purple background
                                    g: 0.1,
                                    b: 0.4,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                }

                queue.submit(std::iter::once(encoder.finish()));
                frame.present();
            }
            _ => (),
        }
    }).unwrap();
}

fn main() {
    pollster::block_on(run());
}