# TODO

## Add graceful background-service shutdown

Add a supported `lens stop` command that sends an authenticated shutdown request over the existing per-user local interprocess communication (IPC) channel.

- Stop accepting new requests.
- Gracefully shut down every `ViewerSession` and release its loopback listener.
- Stop document-watcher tasks and release retained memory and filesystem resources.
- Remove the service endpoint.
- Exit after a bounded timeout, with forced task cancellation as a fallback.
- Keep shutdown unavailable through browser-facing HTTP routes.
- Define and test behavior for concurrent open and stop requests, repeated stop requests, an unavailable service, and stale endpoint recovery.
