//! Android JNI bridge entry points.

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jint, jstring};
use jni::JNIEnv;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static SERVER_RUNNING: AtomicBool = AtomicBool::new(false);
static SERVER_PORT: Mutex<i32> = Mutex::new(5155);

#[no_mangle]
pub extern "system" fn Java_com_fintrack_RustServer_startServer<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    data_dir: JString<'local>,
    port: jint,
) -> jint {
    let data_dir_str: String = match env.get_string(&data_dir) {
        Ok(s) => s.into(),
        Err(_) => return -1,
    };

    if let Ok(mut p) = SERVER_PORT.lock() {
        *p = port;
    }

    std::env::set_var(
        "DATABASE_URL",
        format!("sqlite://{}/finance_game.sqlite?mode=rwc", data_dir_str),
    );
    std::env::set_var("PORT", port.to_string());
    std::env::set_var("BINDING", "127.0.0.1");

    SERVER_RUNNING.store(true, Ordering::SeqCst);
    0
}

#[no_mangle]
pub extern "system" fn Java_com_fintrack_RustServer_stopServer<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jint {
    SERVER_RUNNING.store(false, Ordering::SeqCst);
    0
}

#[no_mangle]
pub extern "system" fn Java_com_fintrack_RustServer_isRunning<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jboolean {
    if SERVER_RUNNING.load(Ordering::SeqCst) {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "system" fn Java_com_fintrack_RustServer_getStatus<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    let status = if SERVER_RUNNING.load(Ordering::SeqCst) {
        format!(
            "running on port {}",
            SERVER_PORT.lock().map(|p| *p).unwrap_or(5155)
        )
    } else {
        "stopped".to_string()
    };
    match env.new_string(status) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}
