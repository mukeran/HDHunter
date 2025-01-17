import json

def application(environ, start_response):
    result = {}
    result['host'] = environ['HTTP_HOST'] if 'HTTP_HOST' in environ else None
    result['content_length'] = environ['CONTENT_LENGTH'] if 'CONTENT_LENGTH' in environ else None
    result['transfer_encoding'] = environ['HTTP_TRANSFER_ENCODING'] if 'HTTP_TRANSFER_ENCODING' in environ else None
    result['body_content'] = environ['wsgi.input'].read().decode()
    result['body_length'] = len(result['body_content'])

    rheaders = [('Content-Type', 'application/json')]

    if 'HTTP_X_DESYNC_ID' in environ:
        rheaders.append(('X-Desync-Id', environ['HTTP_X_DESYNC_ID']))

    start_response('200 OK', rheaders)
    return [json.dumps(result).encode()]
