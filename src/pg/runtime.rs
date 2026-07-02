use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc;

use crate::pg::worker::pg_worker;
use crate::pg::{PgCommand, PgEvent};

pub struct PgRuntime {
    runtime: Runtime,
    commands_tx: mpsc::UnboundedSender<PgCommand>,
}

impl PgRuntime {
    pub fn new() -> anyhow::Result<(Self, mpsc::UnboundedReceiver<PgEvent>)> {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .thread_name("loxql-pg")
            .build()?;
        let (commands_tx, commands_rx) = mpsc::unbounded_channel();
        let (events_tx, events_rx) = mpsc::unbounded_channel();

        runtime.spawn(pg_worker(commands_rx, events_tx));

        Ok((
            Self {
                runtime,
                commands_tx,
            },
            events_rx,
        ))
    }

    pub fn spawn_command(&self, command: PgCommand) {
        let _ = self.commands_tx.send(command);
    }
}
