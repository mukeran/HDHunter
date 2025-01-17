from ctypes import *

hdhunter_rt = CDLL("libhdhunter_rt_no_edge.so", RTLD_GLOBAL)

MODE_REQUEST = 1
MODE_RESPONSE = 2
MODE_SCGI = 4
MODE_FASTCGI = 8
MODE_AJP = 16
MODE_UWSGI = 32

def hdhunter_init():
    hdhunter_rt.hdhunter_init()

def hdhunter_set_content_length(length: c_int64, mode: c_int32):
    hdhunter_rt.hdhunter_set_content_length(length, mode)

def hdhunter_set_chunked_encoding(chunked: c_int8, mode: c_int32):
    hdhunter_rt.hdhunter_set_chunked_encoding(chunked, mode)

def hdhunter_inc_consumed_length(length: c_int64, mode: c_int32):
    hdhunter_rt.hdhunter_inc_consumed_length(length, mode)

def hdhunter_inc_body_length(length: c_int64, mode: c_int32):
    hdhunter_rt.hdhunter_inc_body_length(length, mode)

def hdhunter_mark_message_processed(mode: c_int32):
    hdhunter_rt.hdhunter_mark_message_processed(mode)
