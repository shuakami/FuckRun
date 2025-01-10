from flask import Flask, jsonify
import time
import sys

app = Flask(__name__)

start_time = time.time()

@app.route('/')
def hello():
    uptime = time.time() - start_time
    return f'Hello! Server uptime: {uptime:.2f} seconds'

@app.route('/health')
def health():
    return jsonify({
        'status': 'ok',
        'uptime': time.time() - start_time
    })

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000) 