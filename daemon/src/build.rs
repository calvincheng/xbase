use crate::constants::DAEMON_STATE;
use crate::state::State;
use crate::watch::{Event, Watchable};
use crate::RequestHandler;
use crate::Result;
use async_trait::async_trait;
use tokio::sync::MutexGuard;
use xbase_proto::{BuildRequest, Operation};
use xclog::XCLogger;

#[async_trait]
impl RequestHandler for BuildRequest {
    async fn handle(self) -> Result<()>
    where
        Self: Sized + std::fmt::Debug,
    {
        let state = DAEMON_STATE.clone();
        let ref mut state = state.lock().await;

        match self.ops {
            Operation::Once => self.trigger(state, &Event::default()).await?,
            _ => {
                let watcher = state.watcher.get_mut(&self.client.root)?;
                if self.ops.is_watch() {
                    watcher.add(self)?;
                } else {
                    watcher.remove(&self.to_string())?;
                }
                state.sync_client_state().await?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Watchable for BuildRequest {
    async fn trigger(&self, state: &MutexGuard<State>, _event: &Event) -> Result<()> {
        tracing::info!("Building {}", self.client.abbrev_root());
        let is_once = self.ops.is_once();
        let (root, config) = (&self.client.root, &self.settings);
        // let args = state.projects.get(root)?.build_args(&config, &None)?;
        let args: Vec<String> = vec![];

        let nvim = state.clients.get(&self.client.pid)?;
        let ref mut logger = nvim.logger();

        logger.set_title(format!(
            "{}:{}",
            if is_once { "Build" } else { "Rebuild" },
            config.target
        ));

        tracing::trace!("building with [{}]", args.join(" "));
        let xclogger = XCLogger::new(&root, args)?;
        let success = logger.consume_build_logs(xclogger, false, is_once).await?;

        if !success {
            let ref msg = format!("Failed: {} ", config.to_string());
            nvim.echo_err(msg).await?;
        };

        Ok(())
    }

    /// A function that controls whether a a Watchable should restart
    async fn should_trigger(&self, _state: &MutexGuard<State>, event: &Event) -> bool {
        event.is_content_update_event()
            || event.is_rename_event()
            || event.is_create_event()
            || event.is_remove_event()
            || !(event.path().exists() || event.is_seen())
    }

    /// A function that controls whether a watchable should be droped
    async fn should_discard(&self, _state: &MutexGuard<State>, _event: &Event) -> bool {
        false
    }

    /// Drop watchable for watching a given file system
    async fn discard(&self, _state: &MutexGuard<State>) -> Result<()> {
        Ok(())
    }
}