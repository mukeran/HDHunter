require 'json'

class MyApplication
    def self.call(env)
      result = {}
      result['host'] = env['HTTP_HOST']
      result['content_length'] = env['CONTENT_LENGTH']
      result['transfer_encoding'] = env['HTTP_TRANSFER_ENCODING']
      result['body_content'] = env['rack.input'].read
      result['body_length'] = result['body_content'].length

      rheaders = {"Content-Type" => "application/json"}

      if env['HTTP_X_DESYNC_ID']
        rheaders['X-Desync-Id'] = env['HTTP_X_DESYNC_ID']
      end

      [200, rheaders, [result.to_json]]
    end
end
