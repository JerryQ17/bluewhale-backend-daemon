# daemon

This is a daemon that controls the status of BlueWhale backend.

## API Overview

| Method  |      Endpoint      |                      Description                      |
|:-------:|:------------------:|:-----------------------------------------------------:|
|  `GET`  |     `/backend`     |          Get current status of the backend.           |
|  `PUT`  |     `/backend`     |    Update the backend with the uploaded zip file.     |
| `PATCH` |  `/backend/start`  | Start the backend process (no-op if already started). |
| `PATCH` |  `/backend/stop`   | Stop the backend process (no-op if already stopped).  |
| `PATCH` | `/backend/restart` |             Restart the backend process.              |
