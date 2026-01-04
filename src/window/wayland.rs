use std::{convert::TryInto, num::NonZeroU32, time::Instant};

use mlua::Lua;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::calloop_wayland_source::WaylandSource,
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        Capability, SeatHandler, SeatState,
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
    },
    shell::{
        WaylandSurface,
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
    },
    shm::{Shm, ShmHandler, slot::SlotPool},
};
use wayland_client::{
    Connection, QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
};

use crate::window::{Event, Margin, Opts};

#[allow(dead_code)]
pub struct SimpleLayer {
    state: LayerState,
    layer: Layer,
    anchor: Option<Anchor>,
    margin: Margin,
    event_loop: calloop::EventLoop<'static, LayerState>,
}

impl From<super::Layer> for Layer {
    fn from(val: super::Layer) -> Self {
        match val {
            super::Layer::Background => Layer::Background,
            super::Layer::Bottom => Layer::Bottom,
            super::Layer::Overlay => Layer::Overlay,
            super::Layer::Top => Layer::Top,
        }
    }
}

#[allow(dead_code)]
struct LayerState {
    should_exit: bool,
    first_configure: bool,
    width: u32,
    height: u32,
    exclusive_zone: i32,
    shm: Shm,
    pool: SlotPool,
    layer: LayerSurface,
    pointer: Option<wl_pointer::WlPointer>,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    keyboard_focus: bool,
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    events: Vec<Event>,
    dispatched_events: bool,
    modifiers: crate::window::Modifiers,
    last_frame: Instant,
    lua: Lua,
}

impl crate::Window for SimpleLayer {
    fn new(opts: Opts, lua: Lua) -> Self {
        env_logger::init();

        let conn = Connection::connect_to_env().unwrap();

        let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
        let qh: QueueHandle<LayerState> = event_queue.handle();

        let compositor =
            CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
        let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");

        // Since we are not using the GPU in this example, we use wl_shm to allow software rendering to a buffer
        // we share with the compositor process.
        let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

        let surface = compositor.create_surface(&qh);

        let layer = layer_shell.create_layer_surface(
            &qh,
            surface,
            opts.layer.into(),
            opts.namespace.clone(),
            None,
        );

        if let Some(a) = opts.anchor {
            layer.set_anchor(a);
        }
        let margin = opts.margin;

        layer.set_margin(margin.top, margin.right, margin.bottom, margin.left);
        layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        layer.set_size(opts.width, opts.height);
        layer.set_exclusive_zone(opts.exclusive_zone);
        layer.commit();

        let pool = SlotPool::new((opts.width * opts.height * 4) as usize, &shm)
            .expect("failed to create pool");

        let event_loop = calloop::EventLoop::<LayerState>::try_new().unwrap();

        let mut layer_state = LayerState {
            // Seats and outputs may be hotplugged at runtime, therefore we need to setup a registry state to
            // listen for seats and outputs.
            registry_state: RegistryState::new(&globals),
            seat_state: SeatState::new(&globals, &qh),
            output_state: OutputState::new(&globals, &qh),

            should_exit: false,
            first_configure: true,
            width: opts.width,
            height: opts.height,
            exclusive_zone: opts.exclusive_zone,
            layer,
            events: Vec::new(),

            pool,
            shm,
            modifiers: crate::window::Modifiers::default(),

            keyboard: None,
            keyboard_focus: false,

            pointer: None,
            dispatched_events: false,

            last_frame: Instant::now(),
            lua,
        };

        event_queue.roundtrip(&mut layer_state).unwrap();
        let wayland_source = WaylandSource::new(conn, event_queue);

        event_loop
            .handle()
            .insert_source(wayland_source, |_, queue, state: &mut LayerState| {
                let result = queue.dispatch_pending(state);

                if result.is_ok() {
                    state.dispatched_events = true;
                }

                result
            })
            .unwrap();

        SimpleLayer {
            state: layer_state,
            layer: opts.layer.into(),
            anchor: opts.anchor,
            margin: opts.margin,
            event_loop,
        }
    }

    fn run(&mut self) {
        loop {
            self.event_loop
                .dispatch(Some(std::time::Duration::from_millis(16)), &mut self.state)
                .unwrap();

            let mut tmp_events = Vec::new();
            std::mem::swap(&mut tmp_events, &mut self.state.events);
            for event in tmp_events.drain(..) {
                SimpleLayer::handle_event(self, event);
                if self.state.should_exit {
                    return;
                }
            }
        }
    }

    fn exit(&mut self) {
        self.state.events.push(Event::Exit);
    }
}

impl SimpleLayer {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Resized { width, height } => {
                println!("Resized w: {} h: {}", width, height);
            }
            Event::PointerButtonPressed { button, modifiers } => {
                println!("Button {:?} pressed with {:?}", button, modifiers);
            }
            Event::PointerButtonReleased { button, modifiers } => {
                println!("Button {:?} released with {:?}", button, modifiers);
            }
            Event::PointerMoved { x, y } => {
                println!("Mouse moved at {}, {}", x, y);
            }
            Event::KeyboardKeyPressed { key, modifiers } => {
                println!("Key pressed: {:?} with {:?}", key, modifiers);
            }
            Event::KeyboardKeyReleased { key, modifiers } => {
                println!("Key {:?}: {:?}", key, modifiers);
            }
            Event::Exit => {
                println!("Exiting");
                self.state.should_exit = true;
            }
            _ => {}
        }
    }
}

impl CompositorHandler for LayerState {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(qh);
    }
}

impl OutputHandler for LayerState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for LayerState {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.events.push(Event::Exit);
        self.should_exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.width = NonZeroU32::new(configure.new_size.0).map_or(256, NonZeroU32::get);
        self.height = NonZeroU32::new(configure.new_size.1).map_or(256, NonZeroU32::get);

        // Initiate the first draw.
        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }
}

impl SeatHandler for LayerState {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            println!("Set keyboard capability");
            let keyboard = self
                .seat_state
                .get_keyboard(qh, &seat, None)
                .expect("failed to create keyboard");
            self.keyboard = Some(keyboard);
        }

        if capability == Capability::Pointer && self.pointer.is_none() {
            println!("Set pointer capability");
            let pointer = self
                .seat_state
                .get_pointer(qh, &seat)
                .expect("failed to create pointer");
            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            println!("Unset keyboard capability");
            self.keyboard.take().unwrap().release();
        }

        if capability == Capability::Pointer && self.pointer.is_some() {
            println!("Unset pointer capability");
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for LayerState {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        keysyms: &[Keysym],
    ) {
        if self.layer.wl_surface() == surface {
            println!("Keyboard focus on window with pressed syms: {keysyms:?}");
            self.keyboard_focus = true;
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if self.layer.wl_surface() == surface {
            println!("Release keyboard focus on window");
            self.keyboard_focus = false;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        self.events.push(Event::KeyboardKeyPressed {
            key: event,
            modifiers: crate::window::Modifiers::default(),
        });
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        self.events.push(Event::KeyboardKeyReleased {
            key: event,
            modifiers: crate::window::Modifiers::default(),
        });
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        modifiers: Modifiers,
    ) {
        println!("Update modifiers: {modifiers:?}");
    }
}

impl PointerHandler for LayerState {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        use PointerEventKind::*;
        for event in events {
            // Ignore events for other surfaces
            if &event.surface != self.layer.wl_surface() {
                continue;
            }
            match event.kind {
                Enter { .. } => self.events.push(Event::PointerEntered {
                    x: event.position.0,
                    y: event.position.1,
                }),
                Leave { .. } => {
                    self.events.push(Event::PointerLeft);
                }
                Motion { .. } => {
                    self.events.push(Event::PointerMoved {
                        x: event.position.0,
                        y: event.position.1,
                    });
                }
                Press { .. } => {
                    self.events.push(Event::PointerButtonPressed {
                        button: event.clone(),
                        modifiers: self.modifiers.clone(),
                    });
                }
                Release { .. } => {
                    self.events.push(Event::PointerButtonReleased {
                        button: event.clone(),
                        modifiers: self.modifiers.clone(),
                    });
                }
                Axis { .. } => {}
            }
        }
    }
}

impl ShmHandler for LayerState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl LayerState {
    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let now = Instant::now();
        let frametime = now.duration_since(self.last_frame);
        self.last_frame = Instant::now();
        let fps = 1.0 / frametime.as_secs_f32();

        let (buffer, canvas_data) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");

        // Draw to the window:
        {
            let mut canvas = crate::renderer::skia_cpu::Canvas::new(
                width.try_into().unwrap(),
                height.try_into().unwrap(),
                canvas_data,
            );

            canvas.clear(0xFF707070);
            canvas.draw_fps(fps as u32);

            let g = self.lua.globals();

            let d: mlua::Function = g.get("draw").unwrap();
            self.lua
                .scope(|scope| {
                    let canvas = scope.create_userdata(canvas)?;
                    d.call::<()>(canvas)
                })
                .unwrap();
        }

        // Damage the entire window
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        // Request our next frame
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());

        // Attach and commit to present.
        buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach");
        self.layer.commit();

        // TODO save and reuse buffer when the window size is unchanged.  This is especially
        // useful if you do damage tracking, since you don't need to redraw the undamaged parts
        // of the canvas.
    }
}

delegate_compositor!(LayerState);
delegate_output!(LayerState);
delegate_shm!(LayerState);

delegate_seat!(LayerState);
delegate_keyboard!(LayerState);
delegate_pointer!(LayerState);

delegate_layer!(LayerState);

delegate_registry!(LayerState);

impl ProvidesRegistryState for LayerState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}
