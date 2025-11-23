import subprocess
import threading
import time
import runpod
import requests
import psutil

def run_rust_server():
    process = subprocess.Popen(["./builder"])
    process.wait()

def handler(job):
    repo_url = job['input']['repo_url']

    server_thread = threading.Thread(target=run_rust_server)
    server_thread.start()
    time.sleep(5)  # Wait for server to start

    try:
        response = requests.post(
            'http://127.0.0.1:8080/generate',
            json={'repo_url': repo_url},
            timeout=300  # 5 minutes
        )
        
        if response.status_code == 200:
            # The rust server streams the output, but we can't easily capture that here.
            # We will rely on the final output from the rust server.
            # A more robust solution would be to have the rust server write to a file that this handler can read.
            # For now, we'll assume the last message is the success message.
            
            # This is a hacky way to get the result.
            # We'll try to get the events from the event stream.
            events_res = requests.get('http://127.0.0.1:8080/events', stream=True)
            for line in events_res.iter_lines():
                if line:
                    decoded_line = line.decode('utf-8')
                    if decoded_line.startswith('data: success:'):
                        return {"status": "success", "zip_file": decoded_line.split(': ')[2]}

            return {"status": "error", "message": "Could not determine success"}
        else:
            return {"status": "error", "message": "Failed to start generation"}
    except requests.exceptions.RequestException as e:
        return {"status": "error", "message": str(e)}
    finally:
        # In a real-world scenario, you'd want a more graceful shutdown.
        # For now, we'll just kill the process.
        for proc in psutil.process_iter():
            if proc.name() == "builder":
                proc.kill()


runpod.serverless.start({"handler": handler})