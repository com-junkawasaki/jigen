//! WASM module for web deployment
//!
//! Provides JavaScript bindings and web-specific functionality.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::{console, window};

#[cfg(target_arch = "wasm32")]
mod bindings;

#[cfg(target_arch = "wasm32")]
pub use bindings::*;

/// WASM-specific utilities
pub struct WasmUtils;

impl WasmUtils {
    /// Log to browser console
    #[cfg(target_arch = "wasm32")]
    pub fn console_log(message: &str) {
        console::log_1(&JsValue::from_str(message));
    }

    /// Log to browser console (no-op on native)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn console_log(_message: &str) {
        // No-op on native
    }

    /// Get current time in milliseconds
    #[cfg(target_arch = "wasm32")]
    pub fn now() -> f64 {
        window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0)
    }

    /// Get current time in milliseconds (fallback on native)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn now() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64
    }

    /// Request animation frame callback
    #[cfg(target_arch = "wasm32")]
    pub fn request_animation_frame(callback: js_sys::Function) -> Result<i32, JsValue> {
        window()
            .ok_or("No window")?
            .request_animation_frame(callback.as_ref())
    }

    /// Request animation frame callback (no-op on native)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn request_animation_frame(_callback: ()) -> Result<i32, ()> {
        Ok(0) // No-op on native
    }
}

/// Performance monitoring
pub struct PerformanceMonitor {
    frame_count: u64,
    last_time: f64,
    fps: f32,
    frame_time: f32,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            last_time: WasmUtils::now(),
            fps: 0.0,
            frame_time: 0.0,
        }
    }

    pub fn update(&mut self) {
        self.frame_count += 1;
        let current_time = WasmUtils::now();
        let delta_time = current_time - self.last_time;

        if delta_time >= 1000.0 { // Update every second
            self.fps = (self.frame_count as f32) / ((delta_time / 1000.0) as f32);
            self.frame_time = (delta_time as f32) / (self.frame_count as f32);

            // Log performance info
            WasmUtils::console_log(&format!("FPS: {:.1}, Frame Time: {:.2}ms", self.fps, self.frame_time));

            self.frame_count = 0;
            self.last_time = current_time;
        }
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn frame_time(&self) -> f32 {
        self.frame_time
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage monitoring (WASM specific)
#[cfg(target_arch = "wasm32")]
pub struct MemoryMonitor {
    initial_heap: f64,
}

#[cfg(target_arch = "wasm32")]
impl MemoryMonitor {
    pub fn new() -> Self {
        Self {
            initial_heap: Self::current_heap_size(),
        }
    }

    pub fn current_heap_size() -> f64 {
        web_sys::js_sys::global()
            .dyn_into::<web_sys::js_sys::Object>()
            .ok()
            .and_then(|global| js_sys::Reflect::get(&global, &"performance".into()).ok())
            .and_then(|perf| js_sys::Reflect::get(&perf, &"memory".into()).ok())
            .and_then(|mem| js_sys::Reflect::get(&mem, &"usedJSHeapSize".into()).ok())
            .and_then(|size| size.as_f64())
            .unwrap_or(0.0)
    }

    pub fn used_heap_size(&self) -> f64 {
        Self::current_heap_size() - self.initial_heap
    }

    pub fn log_memory_usage(&self) {
        let used = self.used_heap_size() / (1024.0 * 1024.0); // MB
        WasmUtils::console_log(&format!("Memory Used: {:.2} MB", used));
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct MemoryMonitor;

#[cfg(not(target_arch = "wasm32"))]
impl MemoryMonitor {
    pub fn new() -> Self {
        Self
    }

    pub fn current_heap_size() -> f64 {
        0.0
    }

    pub fn used_heap_size(&self) -> f64 {
        0.0
    }

    pub fn log_memory_usage(&self) {
        // No-op on native
    }
}

/// Web event handling
#[cfg(target_arch = "wasm32")]
pub struct WebEventHandler {
    event_listeners: std::collections::HashMap<String, js_sys::Function>,
}

#[cfg(target_arch = "wasm32")]
impl WebEventHandler {
    pub fn new() -> Self {
        Self {
            event_listeners: std::collections::HashMap::new(),
        }
    }

    pub fn add_event_listener(&mut self, event_type: &str, callback: js_sys::Function) -> Result<(), JsValue> {
        if let Some(window) = window() {
            window.add_event_listener_with_callback(event_type, callback.as_ref())?;
            self.event_listeners.insert(event_type.to_string(), callback);
            Ok(())
        } else {
            Err(JsValue::from_str("No window object available"))
        }
    }

    pub fn remove_event_listener(&mut self, event_type: &str) -> Result<(), JsValue> {
        if let Some(callback) = self.event_listeners.remove(event_type) {
            if let Some(window) = window() {
                window.remove_event_listener_with_callback(event_type, callback.as_ref())?;
            }
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct WebEventHandler;

#[cfg(not(target_arch = "wasm32"))]
impl WebEventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn add_event_listener(&mut self, _event_type: &str, _callback: ()) -> Result<(), ()> {
        Ok(())
    }

    pub fn remove_event_listener(&mut self, _event_type: &str) -> Result<(), ()> {
        Ok(())
    }
}

/// Asset loading for web
#[cfg(target_arch = "wasm32")]
pub struct WebAssetLoader {
    loaded_assets: std::collections::HashMap<String, JsValue>,
}

#[cfg(target_arch = "wasm32")]
impl WebAssetLoader {
    pub fn new() -> Self {
        Self {
            loaded_assets: std::collections::HashMap::new(),
        }
    }

    pub async fn load_image(&mut self, url: &str) -> Result<web_sys::HtmlImageElement, JsValue> {
        let image = web_sys::HtmlImageElement::new()?;
        image.set_src(url);

        let (tx, rx) = futures::channel::oneshot::channel();

        let onload = Closure::wrap(Box::new(move || {
            let _ = tx.send(Ok(()));
        }) as Box<dyn FnMut()>);

        let onerror = Closure::wrap(Box::new(move || {
            let _ = tx.send(Err(JsValue::from_str("Failed to load image")));
        }) as Box<dyn FnMut()>);

        image.set_onload(Some(onload.as_ref().unchecked_ref()));
        image.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        onload.forget();
        onerror.forget();

        rx.await??;

        self.loaded_assets.insert(url.to_string(), image.clone().into());
        Ok(image)
    }

    pub async fn load_json(&mut self, url: &str) -> Result<JsValue, JsValue> {
        let window = window().ok_or("No window")?;
        let resp = window.fetch_with_str(url);
        let resp = wasm_bindgen_futures::JsFuture::from(resp).await?;
        let resp: web_sys::Response = resp.dyn_into()?;
        let json = wasm_bindgen_futures::JsFuture::from(resp.json()?).await?;

        self.loaded_assets.insert(url.to_string(), json.clone());
        Ok(json)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct WebAssetLoader;

#[cfg(not(target_arch = "wasm32"))]
impl WebAssetLoader {
    pub fn new() -> Self {
        Self
    }

    pub async fn load_image(&mut self, _url: &str) -> Result<(), ()> {
        Ok(())
    }

    pub async fn load_json(&mut self, _url: &str) -> Result<(), ()> {
        Ok(())
    }
}
