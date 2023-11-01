use smithay_client_toolkit::seat::{SeatHandler, SeatState, Capability, keyboard::{KeyboardHandler, Keysym, KeyEvent, Modifiers}};
use wayland_client::{Connection, QueueHandle, protocol::{wl_seat::WlSeat, wl_keyboard::WlKeyboard, wl_surface::WlSurface}};

use crate::AppData;

impl KeyboardHandler for AppData {
    fn enter(
        &mut self,
        _: &Connection,
        _qh: &QueueHandle<Self>,
        _: &WlKeyboard,
        surface: &WlSurface,
        _: u32,
        _: &[u32],
        keysyms: &[Keysym],
    ) {
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        surface: &WlSurface,
        _: u32,
    ) {
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        println!("Key press: {event:?}");
        if event.keysym == Keysym::Return {
            let user = std::env::var("USER").unwrap();

            self.authenticator
                .get_handler()
                .set_credentials(user, self.password.clone());

            if self.authenticator.authenticate().is_ok() {
                println!("Authenticated");
                self.session_lock.take();
                _conn.roundtrip().unwrap();
                self.exit = true;
            } else {
                println!("Failed to authenticate");
                self.failed = true;
            }
        } else if event.keysym == Keysym::BackSpace {
            self.password.pop();
            println!("Password: {}", self.password);
        } else {
            let key = event.keysym.key_char().unwrap();
            self.password.push_str(&key.to_string());
            println!("Password: {}", self.password);
        }
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        println!("Key release: {event:?}");
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        _serial: u32,
        modifiers: Modifiers,
    ) {
        println!("Update modifiers: {modifiers:?}");
    }
}
impl SeatHandler for AppData {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            println!("Set keyboard capability");
            let keyboard =
                self.seat_state.get_keyboard(qh, &seat, None).expect("Failed to create keyboard");
            self.keyboard = Some(keyboard);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            println!("Unset keyboard capability");
            self.keyboard.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}
}
