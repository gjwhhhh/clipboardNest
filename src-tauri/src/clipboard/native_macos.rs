#![allow(unexpected_cfgs)]

#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::NSAutoreleasePool;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
use std::ffi::{c_void, CStr};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NativeClipboardImage {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[cfg(target_os = "macos")]
fn decode_image_bytes(data: &[u8]) -> Result<NativeClipboardImage, Box<dyn std::error::Error>> {
    let image = image::load_from_memory(data)?.to_rgba8();
    let (width, height) = image.dimensions();
    Ok(NativeClipboardImage {
        rgba: image.into_raw(),
        width,
        height,
    })
}

#[cfg(target_os = "macos")]
unsafe fn nsstring(text: &str) -> id {
    let string: id = msg_send![class!(NSString), alloc];
    msg_send![string, initWithBytes:text.as_ptr() length:text.len() encoding:4usize]
}

#[cfg(target_os = "macos")]
unsafe fn nsstring_to_string(value: id) -> Option<String> {
    if value == nil {
        return None;
    }

    let utf8_ptr: *const std::os::raw::c_char = msg_send![value, UTF8String];
    if utf8_ptr.is_null() {
        return None;
    }

    Some(CStr::from_ptr(utf8_ptr).to_string_lossy().into_owned())
}

#[cfg(target_os = "macos")]
unsafe fn pasteboard_type_names(pasteboard: id) -> Vec<String> {
    let types: id = msg_send![pasteboard, types];
    if types == nil {
        return Vec::new();
    }

    let count: usize = msg_send![types, count];
    let mut type_names = Vec::with_capacity(count);
    for index in 0..count {
        let value: id = msg_send![types, objectAtIndex:index];
        if let Some(type_name) = nsstring_to_string(value) {
            type_names.push(type_name);
        }
    }

    type_names
}

#[cfg(target_os = "macos")]
unsafe fn pasteboard_bytes_for_type(pasteboard: id, type_name: &str) -> Option<Vec<u8>> {
    let type_ns = nsstring(type_name);
    let data: id = msg_send![pasteboard, dataForType:type_ns];
    if data == nil {
        return None;
    }

    let length: usize = msg_send![data, length];
    if length == 0 {
        return None;
    }

    let bytes: *const u8 = msg_send![data, bytes];
    if bytes.is_null() {
        return None;
    }

    Some(std::slice::from_raw_parts(bytes, length).to_vec())
}

#[cfg(target_os = "macos")]
unsafe fn nsimage_bytes_from_pasteboard(pasteboard: id) -> Option<Vec<u8>> {
    let image: id = msg_send![class!(NSImage), alloc];
    let image: id = msg_send![image, initWithPasteboard:pasteboard];
    if image == nil {
        return None;
    }

    let tiff_data: id = msg_send![image, TIFFRepresentation];
    if tiff_data == nil {
        return None;
    }

    let length: usize = msg_send![tiff_data, length];
    if length == 0 {
        return None;
    }

    let bytes: *const u8 = msg_send![tiff_data, bytes];
    if bytes.is_null() {
        return None;
    }

    Some(std::slice::from_raw_parts(bytes, length).to_vec())
}

#[cfg(target_os = "macos")]
unsafe fn nsurl_file_path(url: id) -> Option<PathBuf> {
    if url == nil {
        return None;
    }

    let is_file_url: bool = msg_send![url, isFileURL];
    if !is_file_url {
        return None;
    }

    let path: id = msg_send![url, path];
    nsstring_to_string(path).map(PathBuf::from)
}

#[cfg(target_os = "macos")]
unsafe fn first_file_url_path_from_object_array(objects: id) -> Option<PathBuf> {
    if objects == nil {
        return None;
    }

    let count: usize = msg_send![objects, count];
    for index in 0..count {
        let value: id = msg_send![objects, objectAtIndex:index];
        if let Some(path) = nsurl_file_path(value) {
            return Some(path);
        }
    }

    None
}

#[cfg(target_os = "macos")]
pub fn read_file_url_from_clipboard() -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        let classes: id = msg_send![class!(NSArray), arrayWithObject:class!(NSURL)];
        let objects: id = msg_send![pasteboard, readObjectsForClasses:classes options:nil];
        let path = first_file_url_path_from_object_array(objects).or_else(|| {
            let url: id = msg_send![class!(NSURL), URLFromPasteboard:pasteboard];
            nsurl_file_path(url)
        });
        let _: () = msg_send![pool, drain];
        Ok(path)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn read_file_url_from_clipboard() -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    Ok(None)
}

#[cfg(target_os = "macos")]
pub fn clipboard_change_count() -> Result<Option<i64>, Box<dyn std::error::Error>> {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        let count: isize = msg_send![pasteboard, changeCount];
        let _: () = msg_send![pool, drain];
        Ok(Some(count as i64))
    }
}

#[cfg(not(target_os = "macos"))]
pub fn clipboard_change_count() -> Result<Option<i64>, Box<dyn std::error::Error>> {
    Ok(None)
}

#[cfg(target_os = "macos")]
pub fn read_image_from_clipboard(
) -> Result<Option<NativeClipboardImage>, Box<dyn std::error::Error>> {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        let mut best_image: Option<(u64, NativeClipboardImage)> = None;

        if let Some(bytes) = nsimage_bytes_from_pasteboard(pasteboard) {
            if let Ok(image) = decode_image_bytes(&bytes) {
                let area = image.width as u64 * image.height as u64;
                best_image = Some((area, image));
            }
        }

        for type_name in pasteboard_type_names(pasteboard) {
            let Some(bytes) = pasteboard_bytes_for_type(pasteboard, &type_name) else {
                continue;
            };

            let Ok(image) = decode_image_bytes(&bytes) else {
                continue;
            };

            let area = image.width as u64 * image.height as u64;
            let replace = best_image
                .as_ref()
                .map(|(best_area, _)| area > *best_area)
                .unwrap_or(true);

            if replace {
                best_image = Some((area, image));
            }
        }

        let _: () = msg_send![pool, drain];
        Ok(best_image.map(|(_, image)| image))
    }
}

#[cfg(not(target_os = "macos"))]
pub fn read_image_from_clipboard(
) -> Result<Option<NativeClipboardImage>, Box<dyn std::error::Error>> {
    Ok(None)
}

#[cfg(target_os = "macos")]
pub fn write_png_to_clipboard(png_bytes: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        let _: isize = msg_send![pasteboard, clearContents];

        let data: id = msg_send![
            class!(NSData),
            dataWithBytes: png_bytes.as_ptr() as *const c_void
            length: png_bytes.len()
        ];
        let type_name = nsstring("public.png");
        let success: bool = msg_send![pasteboard, setData:data forType:type_name];
        let _: () = msg_send![pool, drain];
        Ok(success)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn write_png_to_clipboard(_png_bytes: &[u8]) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(false)
}

/// 将文件写回系统剪贴板（使用 NSPasteboard 的文件 URL 方式）
#[cfg(target_os = "macos")]
pub fn write_file_to_clipboard(file_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    unsafe {
        let pool = NSAutoreleasePool::new(nil);
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        let _: isize = msg_send![pasteboard, clearContents];

        // 创建 NSURL 从文件路径
        let ns_path = nsstring(file_path);
        let url: id = msg_send![class!(NSURL), fileURLWithPath:ns_path];

        // 写入 NSURL 数组到剪贴板
        let classes: id = msg_send![class!(NSArray), arrayWithObject:class!(NSURL)];
        let objects: id = msg_send![class!(NSArray), arrayWithObject:url];

        let success: bool = msg_send![pasteboard, writeObjects:objects];

        let _: () = msg_send![pool, drain];
        Ok(success)
    }
}

/// 非 macOS 平台的桩函数
#[cfg(not(target_os = "macos"))]
pub fn write_file_to_clipboard(_file_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(false)
}

/// 获取当前最前面的应用名称
#[cfg(target_os = "macos")]
pub fn get_frontmost_app_name() -> Option<String> {
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let app: id = msg_send![workspace, frontmostApplication];
        if app == nil {
            return None;
        }
        let name: id = msg_send![app, localizedName];
        nsstring_to_string(name)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn get_frontmost_app_name() -> Option<String> {
    None
}
