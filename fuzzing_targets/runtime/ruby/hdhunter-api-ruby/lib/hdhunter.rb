# frozen_string_literal: true

require "ffi"

module HDHunter
  extend FFI::Library
  ffi_lib 'libhdhunter_rt_no_edge.so'

  MODE_REQUEST = 1
  MODE_RESPONSE = 2
  MODE_SCGI = 4
  MODE_FASTCGI = 8
  MODE_AJP = 16
  MODE_UWSGI = 32

  attach_function :hdhunter_init, :hdhunter_init, [], :void
  attach_function :set_content_length, :hdhunter_set_content_length, [:int64, :int32], :void
  attach_function :set_chunked_encoding, :hdhunter_set_chunked_encoding, [:int8, :int32], :void
  attach_function :inc_consumed_length, :hdhunter_inc_consumed_length, [:int64, :int32], :void
  attach_function :inc_body_length, :hdhunter_inc_body_length, [:int64, :int32], :void
  attach_function :mark_message_processed, :hdhunter_mark_message_processed, [:int32], :void
end
