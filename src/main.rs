use pam::{Authenticator, PasswordConv};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{
            EventLoop, LoopHandle,
        },
        calloop_wayland_source::WaylandSource,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    session_lock::{
        SessionLock, SessionLockHandler, SessionLockState, SessionLockSurface,
        SessionLockSurfaceConfigure,
    },
    shm::{raw::RawPool, Shm, ShmHandler}, seat::SeatState,
};
use std::time::Duration;
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_buffer, wl_output, wl_shm, wl_surface, wl_keyboard},
    Connection, QueueHandle,
};

mod input;
mod rendering;

struct AppData {
    loop_handle: LoopHandle<'static, Self>,
    conn: Connection,
    compositor_state: CompositorState,
    output_state: OutputState,
    registry_state: RegistryState,
    shm: Shm,
    session_lock_state: SessionLockState,
    session_lock: Option<SessionLock>,
    lock_surfaces: Vec<SessionLockSurface>,
    exit: bool,
    seat_state: SeatState,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    password: String,
    width: u32,
    height: u32,
    failed: bool,
    authenticator: Authenticator<'static, PasswordConv>,
}

fn main() {
    env_logger::init();

    let conn = Connection::connect_to_env().unwrap();

    let (globals, event_queue) = registry_queue_init(&conn).unwrap();
    let qh: QueueHandle<AppData> = event_queue.handle();
    let mut event_loop: EventLoop<AppData> =
        EventLoop::try_new().expect("Failed to initialize the event loop!");

    let mut app_data = AppData {
        loop_handle: event_loop.handle(),
        conn: conn.clone(),
        compositor_state: CompositorState::bind(&globals, &qh).unwrap(),
        output_state: OutputState::new(&globals, &qh),
        registry_state: RegistryState::new(&globals),
        shm: Shm::bind(&globals, &qh).unwrap(),
        session_lock_state: SessionLockState::new(&globals, &qh),
        session_lock: None,
        lock_surfaces: Vec::new(),
        exit: false,
        seat_state: SeatState::new(&globals, &qh),
        keyboard: None,
        password: String::new(),
        width: 0,
        height: 0,
        failed: false,
        authenticator: Authenticator::with_password("system-auth").unwrap(),
    };

    app_data.session_lock =
        Some(app_data.session_lock_state.lock(&qh).expect("ext-session-lock not supported"));

    WaylandSource::new(conn.clone(), event_queue).insert(event_loop.handle()).unwrap();

    loop {
        event_loop.dispatch(Duration::from_millis(16), &mut app_data).unwrap();

        if app_data.exit {
            break;
        }
    }
}

impl SessionLockHandler for AppData {
    fn locked(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, session_lock: SessionLock) {
        println!("Locked");

        for output in self.output_state.outputs() {
            let surface = self.compositor_state.create_surface(&qh);
            let lock_surface = session_lock.create_lock_surface(surface, &output, qh);
            self.lock_surfaces.push(lock_surface);
        }
    }

    fn finished(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _session_lock: SessionLock,
    ) {
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        session_lock_surface: SessionLockSurface,
        configure: SessionLockSurfaceConfigure,
        _serial: u32,
    ) {
        let (width, height) = configure.new_size;
        self.width = width;
        self.height = height;
        let mut pool = RawPool::new(width as usize * height as usize * 4, &self.shm).unwrap();
        let canvas = pool.mmap();
        canvas.chunks_exact_mut(4).enumerate().for_each(|(index, chunk)| {

            let color = 0xFF000000 as u32;

            let array: &mut [u8; 4] = chunk.try_into().unwrap();
            *array = color.to_le_bytes();
        });
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            width as i32 * 4,
            wl_shm::Format::Argb8888,
            (),
            qh,
        );

        session_lock_surface.wl_surface().attach(Some(&buffer), 0, 0);

        session_lock_surface.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        session_lock_surface.wl_surface().frame(qh, session_lock_surface.wl_surface().clone());

        session_lock_surface.wl_surface().commit();

        buffer.destroy();
    }
}

impl OutputHandler for AppData {
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

impl ProvidesRegistryState for AppData {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState,];
}

impl ShmHandler for AppData {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

smithay_client_toolkit::delegate_compositor!(AppData);
smithay_client_toolkit::delegate_output!(AppData);
smithay_client_toolkit::delegate_session_lock!(AppData);
smithay_client_toolkit::delegate_seat!(AppData);
smithay_client_toolkit::delegate_keyboard!(AppData);
smithay_client_toolkit::delegate_shm!(AppData);
smithay_client_toolkit::delegate_registry!(AppData);
wayland_client::delegate_noop!(AppData: ignore wl_buffer::WlBuffer);
