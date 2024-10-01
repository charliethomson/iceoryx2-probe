# Iceoryx2 Probe

I need a way to do stable, structured IPC between a host process and many child
processes, this repository has the explorations of using
[iceoryx2](https://docs.rs/iceoryx2/latest/iceoryx2/) for that solution.

This example uses ~9% responsive cpu for the host, and ~6M of ram per publisher,
the "host" in this case would likely in reality be running in its own thread pushing
the messages to a `tokio::sync::mpsc::UnboundedSender`, as part of a larger application.

![📸](https://github.com/charliethomson/iceoryx2-probe/blob/main/.screenshots/SCR-20241001-gccy.png)
