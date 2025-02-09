// Window management and event handling depencies using the "winit" crate
// Winit provides cross-platform window creation and event handling
use winit::{

    // Import types for handling system and window events
    event::{

        Event,
        // - Is the enum that represents all possible events that can occur...
        // (window events, device events, etc.)
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
        // - "unwrap()" crashes the application if operation fails; if operation...
        // succeeds no effect

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
        // - Use first suppported transparency mode supported by the GPU for handling transparency

        view_formats: vec![],
        // - No additional format views needed

        desired_maximum_frame_latency: 2,
        // - Number of frames GPU can prepare ahead
    };

    // Configure the surface using our previosly established configuration settings
    surface.configure(&device, &config);

    // Create a reference to our window, that we can use inside the event loop
    let window_reference = &window;

    // Run the event loop, and handle different events...
    // also uses "move" to take ownership of any outside variables used
    event_loop.run(move |event, target| {

        // Handle different events
        match event {

            // Handle window close request,
            // ("..": ignore any other fields in WindowEvent)
            Event::WindowEvent {event: WindowEvent::CloseRequested, ..} => {

                // Terminate the loop to cleanly close our application
                target.exit();
            }

            // Handle window resizing, get new window dimensions to "window_new_size" 
            // ("..": ignore any other fields in WindowEvent)
            Event::WindowEvent {event: WindowEvent::Resized(window_new_size), .. } => {

                //Update surface configuration with new window dimensions
                config.width = window_new_size.width;
                config.height = window_new_size.height;

                // Tell GPU to create a new surface to match the new surface dimensions
                surface.configure(&device, &config);
            }

            // Handle event in which the event loop is about to enter a waiting state...
            // after all pending events have been processed
            Event::AboutToWait => {

                // Request a redraw of the window, triggering the RedrawRequested event
                window_reference.request_redraw();
            }

            // Handle window redraw requests
            // ("..": ignore any other fields in WindowEvent)
            Event::WindowEvent {event: WindowEvent::RedrawRequested, ..} => {

                // Get the next frame to render to, from the surface
                let frame = surface.get_current_texture().unwrap();
                // - "unwrap()" crashes the application if operation fails; if operation succeeds no effect
                
                // Set up how we want to use this frame's texture for rendering,
                // Creates a default view that lets us render to the entire texture
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                // Create a new command recorder to store all the GPU commands until...
                // we want to send them to the GPU all at once, to be executed
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {label: Some("Render Encoder")});
                // - "label" name for debugging

                // Create a new scope - this makes sure we finish setting up the render...
                // pass before we try to submit our commands
                {

                    // Begin a render pass that describes how to render our frame
                    let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {

                        label: Some("Render Pass"),
                        // - Name for debugging

                        // Configure how we want to handle color data in this render pass
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {

                            // Reference the texture we're rendering to
                            view: &view,

                            resolve_target: None,
                            // - We're not using anti-aliasing (edge smoothing),
                            // so we don't need a seperate texture for the final image

                            // Define operations for this color attachment
                            ops: wgpu::Operations {

                                // Clear the screen and set to a color
                                load: wgpu::LoadOp::Clear(wgpu::Color {

                                    r: 0.4, // Red
                                    g: 0.1, // Green
                                    b: 0.4, // Blue
                                    a: 1.0, // Alpha/'opacity'
                                    // - Set to a purple background
                                }),

                                // Store the rendering results
                                store: wgpu::StoreOp::Store,

                            },
                        })],

                        depth_stencil_attachment: None,
                        // - Don't track which objects are in front of others

                        timestamp_writes: None,
                        // - Don't measure how long rendering takes

                        occlusion_query_set: None,
                        // - Don't check if objects are hidden behind other objects
                    });
                }

                // Submit the encoded commands to the GPU's command queue
                queue.submit(std::iter::once(encoder.finish()));

                // Present the rendered frame to the screen
                frame.present();
                
            // End of RedrawRequested event handler
            }
            
            //Catch-all to ignore any event types we haven't explicitly handled above,
            // With our event handlers
            _ => (),
        
        }
        // - End of match statement for event types

    }).unwrap();
    // - End of event loop
    // - "unwrap()" crashes the application if operation fails; if operation succeeds no effect

}
// - End of async run() function

// Program entry point - this is where Rust begins executing the program
fn main() {

    // Start running our async program code in a synchronous context
    // This is needed because main() can't be async, but run() is
    pollster::block_on(run());
    
}