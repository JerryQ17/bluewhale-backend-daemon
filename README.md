# bluewhale-backend-daemon

The bluewhale-backend-daemon project is a semi-automated CI/CD service for the backend of the BlueWhale project.

It contains:

|           Name            |  Type  |           Description            |
|:-------------------------:|:------:|:--------------------------------:|
| [daemon](./crates/daemon) | binary | a daemon controlling the backend |

## Workflow

1. The daemon is started and listens for incoming requests.
2. The developer commits and pushes changes to the backend repository.
3. The developer sends the latest project archive to the daemon.
4. The daemon extracts the archive and restarts the backend.

## Why Semi-Automated?

The goal is to synchronize the service server with the SeeCoder internal GitLab.

There are two ways to achieve this:

- The daemon behaves as a client, it actively `git pull` from the SeeCoder internal GitLab.
- The daemon behaves as a server, it waits for someone (developer or GitLab CI/CD pipeline) to send the latest project archive.

Unfortunately, The SeeCoder internal GitLab cannot be accessed by the public network, which results in:

- Calling `git pull` from the service server won't work.
- No runner can be registered on the SeeCoder internal GitLab, the triggered GitLab CI/CD pipeline will block forever.

So, the developer has to send the latest project archive to the daemon by itself.
