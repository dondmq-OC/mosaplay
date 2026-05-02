//! Raw FFI bindings to libmpv render API.
//! Only the subset needed for embedded OpenGL rendering.

#![allow(non_camel_case_types, dead_code)]

use std::ffi::{c_char, c_int, c_void};
use std::os::raw::c_ulong;

// ── Opaque types ────────────────────────────────────────────
pub type MpvHandle = c_void;
pub type MpvRenderContext = c_void;
pub type MpvEventHandle = c_void;

// ── Enums & constants ───────────────────────────────────────
pub const MPV_ERROR_SUCCESS: c_int = 0;

// mpv_render_param_type
pub const MPV_RENDER_PARAM_API_TYPE: c_int = 1;
pub const MPV_RENDER_PARAM_OPENGL_INIT_PARAMS: c_int = 2;
pub const MPV_RENDER_PARAM_OPENGL_FBO: c_int = 3;
pub const MPV_RENDER_PARAM_FLIP_Y: c_int = 4;
pub const MPV_RENDER_PARAM_DEPTH: c_int = 5;
pub const MPV_RENDER_PARAM_ICC_PROFILE: c_int = 6;
pub const MPV_RENDER_PARAM_ADVANCED_CONTROL: c_int = 10;
pub const MPV_RENDER_PARAM_BLOCK_FOR_TARGET_TIME: c_int = 11;
pub const MPV_RENDER_PARAM_SKIP_RENDERING: c_int = 12;

// mpv_format
pub const MPV_FORMAT_STRING: c_int = 1;
pub const MPV_FORMAT_FLAG: c_int = 3;
pub const MPV_FORMAT_INT64: c_int = 4;
pub const MPV_FORMAT_DOUBLE: c_int = 5;
pub const MPV_FORMAT_NODE: c_int = 6;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MpvNode {
    pub format: c_int,
    pub u: MpvNodeUnion,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union MpvNodeUnion {
    pub flag: c_int,
    pub int64: i64,
    pub double: f64,
    pub string: *mut c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MpvNodeList {
    pub num: c_int,
    pub values: *mut MpvNode,
    pub keys: *mut *mut c_char,
}

// mpv_event_id
pub const MPV_EVENT_NONE: c_ulong = 0;
pub const MPV_EVENT_SHUTDOWN: c_ulong = 1;
pub const MPV_EVENT_LOG_MESSAGE: c_ulong = 2;
pub const MPV_EVENT_FILE_LOADED: c_ulong = 8;
pub const MPV_EVENT_END_FILE: c_ulong = 9;
pub const MPV_EVENT_PROPERTY_CHANGE: c_ulong = 22;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MpvEventProperty {
    pub name: *mut c_char,
    pub format: c_int,
    pub data: *mut c_void,
}

#[repr(C)]
pub union MpvEventData {
    pub property: *mut MpvEventProperty,
}

#[repr(C)]
pub struct MpvEvent {
    pub event_id: c_ulong,
    pub error: c_int,
    pub reply_userdata: c_ulong,
    pub data: *mut c_void,
}

// ── Render params ──────────────────────────────────────────
#[repr(C)]
pub struct MpvRenderParam {
    pub type_: c_int,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct MpvOpenGLInitParams {
    pub get_proc_address: extern "C" fn(*mut c_void, *const c_char) -> *mut c_void,
    pub get_proc_address_ctx: *mut c_void,
}

#[repr(C)]
pub struct MpvOpenGLFbo {
    pub fbo: c_int,
    pub w: c_int,
    pub h: c_int,
    pub internal_format: c_int,
}

// ── Functions ───────────────────────────────────────────────
extern "C" {
    pub fn mpv_create() -> *mut MpvHandle;
    pub fn mpv_initialize(handle: *mut MpvHandle) -> c_int;
    pub fn mpv_destroy(handle: *mut MpvHandle);
    pub fn mpv_terminate_destroy(handle: *mut MpvHandle);
    pub fn mpv_set_option_string(
        handle: *mut MpvHandle,
        name: *const c_char,
        data: *const c_char,
    ) -> c_int;
    pub fn mpv_set_property_string(
        handle: *mut MpvHandle,
        name: *const c_char,
        data: *const c_char,
    ) -> c_int;
    pub fn mpv_set_property_async(
        handle: *mut MpvHandle,
        reply_userdata: c_ulong,
        name: *const c_char,
        format: c_int,
        data: *mut c_void,
    ) -> c_int;
    pub fn mpv_get_property(
        handle: *mut MpvHandle,
        name: *const c_char,
        format: c_int,
        data: *mut c_void,
    ) -> c_int;
    pub fn mpv_command_string(handle: *mut MpvHandle, args: *const c_char) -> c_int;
    pub fn mpv_command_async(
        handle: *mut MpvHandle,
        reply_userdata: c_ulong,
        args: *const *const c_char,
    ) -> c_int;
    pub fn mpv_observe_property(
        handle: *mut MpvHandle,
        reply_userdata: c_ulong,
        name: *const c_char,
        format: c_int,
    ) -> c_int;
    pub fn mpv_render_context_create(
        res: *mut *mut MpvRenderContext,
        mpv: *mut MpvHandle,
        params: *mut MpvRenderParam,
    ) -> c_int;
    pub fn mpv_render_context_render(
        ctx: *mut MpvRenderContext,
        params: *mut MpvRenderParam,
    ) -> c_int;
    pub fn mpv_render_context_set_parameter(
        ctx: *mut MpvRenderContext,
        param: MpvRenderParam,
    ) -> c_int;
    pub fn mpv_render_context_free(ctx: *mut MpvRenderContext);
    pub fn mpv_render_context_update(ctx: *mut MpvRenderContext) -> c_ulong;
    pub fn mpv_render_context_report_swap(ctx: *mut MpvRenderContext);
    pub fn mpv_wait_event(handle: *mut MpvHandle, timeout: f64) -> *mut MpvEvent;
    pub fn mpv_event_name(event: c_ulong) -> *const c_char;
    pub fn mpv_free(data: *mut c_void);
}
