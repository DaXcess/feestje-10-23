use std::time::Duration;

use crate::AppState;
use anyhow::{anyhow, Result};
use image::RgbImage;
use libcamera::{
    camera::CameraConfigurationStatus,
    camera_manager::CameraManager,
    framebuffer::AsFrameBuffer,
    framebuffer_allocator::{FrameBuffer, FrameBufferAllocator},
    framebuffer_map::MemoryMappedFrameBuffer,
    geometry::Size,
    pixel_format::PixelFormat,
    stream::StreamRole,
};

const PIXEL_FORMAT_BGR: PixelFormat =
    PixelFormat::new(u32::from_le_bytes([b'B', b'G', b'2', b'4']), 0);

/// Capture an image using the first detected camera
///
/// Returns an [`RgbImage`] containing raw 8-bit RGB data
pub fn capture_image(state: &mut AppState, flash: bool) -> Result<RgbImage> {
    let mgr = CameraManager::new()?;
    let cameras = mgr.cameras();
    let camera = cameras.get(0).ok_or(anyhow!("Cannot find camera"))?;
    let mut cam = camera.acquire()?;

    let mut cfgs = cam
        .generate_configuration(&[StreamRole::ViewFinder])
        .ok_or(anyhow!("Cannot generate configuration for camera"))?;

    let mut our_cfg = cfgs
        .get_mut(0)
        .ok_or(anyhow!("Failed to generate configuration for camera"))?;
    our_cfg.set_pixel_format(PIXEL_FORMAT_BGR);
    our_cfg.set_size(Size {
        width: 3200,
        height: 2048,
    });

    if let CameraConfigurationStatus::Invalid = cfgs.validate() {
        return Err(anyhow!("Error validating camera configuration"));
    }

    cam.configure(&mut cfgs)?;

    let mut alloc = FrameBufferAllocator::new(&cam);

    let cfg = cfgs.get(0).expect("memory vanished");
    let stream = cfg.stream().expect("camera was reconfigured");
    let buffers = alloc.alloc(&stream)?;

    let buffers = buffers
        .into_iter()
        .map(|b| MemoryMappedFrameBuffer::new(b).expect("memory failure I guess"))
        .collect::<Vec<_>>();

    let mut reqs = buffers
        .into_iter()
        .map(|buf| {
            let mut req = cam.create_request(None).expect("failed to create request");
            req.add_buffer(&stream, buf)
                .expect("buffer attached more than once");
            req
        })
        .collect::<Vec<_>>();

    let (tx, rx) = std::sync::mpsc::channel();
    cam.on_request_completed(move |req| {
        tx.send(req).expect("I would be surprised if this fails");
    });

    // LEDS POWER UP HERE IF FLASH IS ENABLED

    if flash {
        state.matrix.set_brightness(100).ok();
        state.matrix.fill(255, 255, 255).ok();
    }

    cam.start(None)?;
    cam.queue_request(reqs.pop().ok_or(anyhow!("Capture request vanished"))?)?;

    let req = rx.recv_timeout(Duration::from_secs(2))?;

    // LEDS POWER DOWN HERE IF FLASH IS ENABLED

    if flash {
        state.matrix.clear().ok();
    }

    let framebuffer: &MemoryMappedFrameBuffer<FrameBuffer> =
        req.buffer(&stream).ok_or(anyhow!("missing frame buffer"))?;

    let planes = framebuffer.data();
    let rgb_data = planes.get(0).ok_or(anyhow!("missing RGB data"))?;

    let rgb_len = framebuffer
        .metadata()
        .ok_or(anyhow!("Frame is missing metadata"))?
        .planes()
        .get(0)
        .ok_or(anyhow!("Frame is missing plane"))?
        .bytes_used as usize;

    let size = cfg.get_size();

    Ok(
        RgbImage::from_raw(size.width, size.height, rgb_data[..rgb_len].to_vec())
            .expect("output image has invalid size"),
    )
}
