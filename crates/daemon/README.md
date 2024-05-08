# bluewhale-backend-daemon

This a daemon that controls the status of BlueWhale backend.

## API Overview

| Method  |  Endpoint  |            Description             |
|:-------:|:----------:|:----------------------------------:|
|  `GET`  |    `/`     | Get current status of the backend. |
| `POST`  | `/update`  |        Updates the backend.        |
| `PATCH` |  `/start`  |    Starts the backend process.     |
| `PATCH` |  `/stop`   |  Terminates the backend process.   |
| `PATCH` | `/restart` |   Restarts the backend process.    |
