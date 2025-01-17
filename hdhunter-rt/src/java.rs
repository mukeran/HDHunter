use crate::{hdhunter_inc_body_length, hdhunter_inc_consumed_length, hdhunter_init, hdhunter_mark_message_processed, hdhunter_set_chunked_encoding, hdhunter_set_content_length};
use jni::objects::JClass;
use jni::sys::{jbyte, jint, jlong, JavaVM, JNI_VERSION_1_6};
use jni::JNIEnv;

#[no_mangle]
pub unsafe extern "system" fn JNI_OnLoad(_vm: JavaVM, _: *mut std::ffi::c_void) -> jint {
    hdhunter_init();
    JNI_VERSION_1_6
}

#[no_mangle]
pub unsafe extern "system" fn Java_tools_hdhunter_HDHunter_setContentLength<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    length: jlong,
    mode: jint,
) {
    hdhunter_set_content_length(length, mode);
}

#[no_mangle]
pub unsafe extern "system" fn Java_tools_hdhunter_HDHunter_setChunkedEncoding<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    chunked: jbyte,
    mode: jint
) {
    hdhunter_set_chunked_encoding(chunked, mode);
}

#[no_mangle]
pub unsafe extern "system" fn Java_tools_hdhunter_HDHunter_incConsumedLength<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    length: jlong,
    mode: jint
) {
    hdhunter_inc_consumed_length(length, mode);
}

#[no_mangle]
pub unsafe extern "system" fn Java_tools_hdhunter_HDHunter_incBodyLength<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    length: jlong,
    mode: jint
) {
    hdhunter_inc_body_length(length, mode);
}

#[no_mangle]
pub unsafe extern "system" fn Java_tools_hdhunter_HDHunter_markMessageProcessed<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    mode: jint
) {
    hdhunter_mark_message_processed(mode);
}
