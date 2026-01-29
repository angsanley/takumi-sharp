use std::{ffi::CStr, sync::LazyLock};

use takumi::{GlobalContext, layout::node::NodeKind, rendering::RenderOptionsBuilder};

static mut GLOBAL_CONTEXT: LazyLock<GlobalContext> = LazyLock::new(GlobalContext::default);
static mut GLOBAL_LAST_ERROR: String = String::new();

/// The viewport for the image renderer.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    /// The width of the viewport in pixels, or -1 for none/auto.
    pub width: i32,
    /// The height of the viewport in pixels, or -1 for none/auto.
    pub height: i32,
    /// The font size in pixels, used for em and rem units.
    pub font_size: f32,
    /// The device pixel ratio
    pub device_pixel_ratio: f32,
}

impl From<Viewport> for takumi::layout::Viewport {
    fn from(value: Viewport) -> Self {
        Self {
            width: if value.width < 0 { None } else { Some(value.width as u32) },
            height: if value.height < 0 { None } else { Some(value.height as u32) },
            font_size: value.font_size,
            device_pixel_ratio: value.device_pixel_ratio,
        }
    }
}

#[repr(u8)]
pub enum ImageFormat {
    /// An Image in PNG Format
    Png,

    /// An Image in JPEG Format
    Jpeg,

    /// An Image in GIF Format
    Gif,

    /// An Image in WEBP Format
    WebP,

    /// An Image in general PNM Format
    Pnm,

    /// An Image in TIFF Format
    Tiff,

    /// An Image in TGA Format
    Tga,

    /// An Image in DDS Format
    Dds,

    /// An Image in BMP Format
    Bmp,

    /// An Image in ICO Format
    Ico,

    /// An Image in Radiance HDR Format
    Hdr,

    /// An Image in OpenEXR Format
    OpenExr,

    /// An Image in farbfeld Format
    Farbfeld,

    /// An Image in AVIF Format
    Avif,

    /// An Image in QOI Format
    Qoi,
}

impl From<ImageFormat> for takumi::image::ImageFormat {
    fn from(value: ImageFormat) -> Self {
        match value {
            ImageFormat::Png => Self::Png,
            ImageFormat::Jpeg => Self::Jpeg,
            ImageFormat::Gif => Self::Gif,
            ImageFormat::WebP => Self::WebP,
            ImageFormat::Pnm => Self::Pnm,
            ImageFormat::Tiff => Self::Tiff,
            ImageFormat::Tga => Self::Tga,
            ImageFormat::Dds => Self::Dds,
            ImageFormat::Bmp => Self::Bmp,
            ImageFormat::Ico => Self::Ico,
            ImageFormat::Hdr => Self::Hdr,
            ImageFormat::OpenExr => Self::OpenExr,
            ImageFormat::Farbfeld => Self::Farbfeld,
            ImageFormat::Avif => Self::Avif,
            ImageFormat::Qoi => Self::Qoi,
        }
    }
}


/// Loads and stores font data into the global font context.
///
/// # Safety
///
/// - `data` must be a valid pointer to a byte array of at least `len` bytes.
/// - The data must remain valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn global_font_context_load_and_store(
    data: *const u8,
    len: usize,
) -> bool {
    let data_arr = unsafe { std::slice::from_raw_parts(data, len) };
    if let Err(e) = unsafe { &mut *std::ptr::addr_of_mut!(GLOBAL_CONTEXT) }
        .font_context
        .load_and_store(data_arr, None, None)
    {
        unsafe { *std::ptr::addr_of_mut!(GLOBAL_LAST_ERROR) = e.to_string() };
        return false;
    }
    true
}

/// Calculates the required buffer size for rendering a node to an image.
///
/// # Safety
///
/// - `node_str` must be a valid null-terminated C string pointer.
#[no_mangle]
pub unsafe extern "C" fn render_calculate_buffer_size_with_format(
    node_str: *const std::ffi::c_char,
    viewport: Viewport,
    format: ImageFormat,
) -> u64 {
    if node_str.is_null() {
        return 0;
    }

    let node_str = match CStr::from_ptr(node_str).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let node: NodeKind = match serde_json::from_str(node_str) {
        Ok(n) => n,
        Err(_) => return 0,
    };

    let opt = match RenderOptionsBuilder::default()
        .viewport(viewport.into())
        .node(node)
        .global(unsafe { &*std::ptr::addr_of!(GLOBAL_CONTEXT) })
        .build()
    {
        Ok(o) => o,
        Err(e) => {
            unsafe { *std::ptr::addr_of_mut!(GLOBAL_LAST_ERROR) = e.to_string() };
            return 0;
        }
    };

    let img_format: takumi::image::ImageFormat = format.into();

    let image = match takumi::rendering::render(opt) {
        Ok(img) => img,
        Err(e) => {
            unsafe { *std::ptr::addr_of_mut!(GLOBAL_LAST_ERROR) = e.to_string() };
            return 0;
        }
    };

    let mut cursor = std::io::Cursor::new(Vec::new());
    if let Err(e) = image.write_to(&mut cursor, img_format) {
        unsafe { *std::ptr::addr_of_mut!(GLOBAL_LAST_ERROR) = e.to_string() };
        return 0;
    }

    cursor.into_inner().len() as u64
}

/// Renders a node to an image and writes it to a buffer.
///
/// # Safety
///
/// - `node_str` must be a valid null-terminated C string pointer.
/// - `out_buffer` must be a valid pointer to a buffer of at least `buffer_len` bytes.
/// - The buffer must remain valid for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn render_to_buffer_with_format(
    node_str: *const std::ffi::c_char,
    viewport: Viewport,
    format: ImageFormat,
    out_buffer: *mut u8,
    buffer_len: u64,
) -> bool {
    if node_str.is_null() || out_buffer.is_null() || buffer_len == 0 {
        return false;
    }

    let node_str = match CStr::from_ptr(node_str).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let node: NodeKind = match serde_json::from_str(node_str) {
        Ok(n) => n,
        Err(_) => return false,
    };

    let opt = match RenderOptionsBuilder::default()
        .viewport(viewport.into())
        .node(node)
        .global(unsafe { &*std::ptr::addr_of!(GLOBAL_CONTEXT) })
        .build()
    {
        Ok(o) => o,
        Err(e) => {
            unsafe { *std::ptr::addr_of_mut!(GLOBAL_LAST_ERROR) = e.to_string() };
            return false;
        }
    };

    let img_format: takumi::image::ImageFormat = format.into();

    let image = match takumi::rendering::render(opt) {
        Ok(img) => img,
        Err(e) => {
            unsafe { *std::ptr::addr_of_mut!(GLOBAL_LAST_ERROR) = e.to_string() };
            return false;
        }
    };

    let mut cursor = std::io::Cursor::new(Vec::new());
    if let Err(e) = image.write_to(&mut cursor, img_format) {
        unsafe { *std::ptr::addr_of_mut!(GLOBAL_LAST_ERROR) = e.to_string() };
        return false;
    }

    let bytes = cursor.into_inner();
    if bytes.len() > buffer_len as usize {
        unsafe { *std::ptr::addr_of_mut!(GLOBAL_LAST_ERROR) = "Buffer too small".to_string() };
        return false;
    }

    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buffer, bytes.len());
    true
}