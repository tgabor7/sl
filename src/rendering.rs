use smithay_client_toolkit::{compositor::CompositorHandler, shm::raw::RawPool};
use wayland_client::{Connection, QueueHandle, protocol::{wl_surface, wl_output, wl_shm}};

use crate::AppData;

fn draw_circle(
    pool: &mut RawPool,
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    radius: i32,
    ) {
    
    let canvas = pool.mmap();

    canvas.chunks_exact_mut(4).enumerate().for_each(|(index, chunk)| {
        let x_p = (index % width as usize) as i32;
        let y_p = (index / width as usize) as i32;
        let color = 0xFFFFFFFF as u32;
    
        if (x_p - x).pow(2) + (y_p - y).pow(2) < radius.pow(2) {
            let array: &mut [u8; 4] = chunk.try_into().unwrap();
            *array = color.to_le_bytes();
        }
    });
}

impl CompositorHandler for AppData {
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
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        let (width, height) = (self.width, self.height);

        let mut pool = RawPool::new(width as usize * height as usize * 4, &self.shm).unwrap();
        let canvas = pool.mmap();
        canvas.chunks_exact_mut(4).enumerate().for_each(|(index, chunk)| {
            let x = (index % width as usize) as u32;
            let y = (index / width as usize) as u32;
            let color = 0xFF000000 as u32;

            let array: &mut [u8; 4] = chunk.try_into().unwrap();
            *array = color.to_le_bytes();
        });

        for i in 0..self.password.len() {
            let x = (width as i32 / 2) + (i as i32 * 20) - (self.password.len() as i32 * 10);
            let y = height as i32 / 2;
            let radius = 8;
            draw_circle(&mut pool, width, height, x, y, radius);
        }

        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            width as i32 * 4,
            wl_shm::Format::Argb8888,
            (),
            _qh,
        );

        _surface.attach(Some(&buffer), 0, 0);

        _surface.damage_buffer(0, 0, width as i32, height as i32);
        _surface.frame(_qh, _surface.clone());

        _surface.commit();

        buffer.destroy();
    }
}
