use super::{WatchArguments, WatchError};
use crate::constants::DAEMON_STATE;
use crate::daemon::{WatchKind, WatchTarget};
use crate::nvim::WatchLogger;
use crate::types::{BuildConfiguration, Client};
use crate::xcode;
use anyhow::Result;
use notify::{event::ModifyKind, Event, EventKind};
use std::time::{Duration, SystemTime};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tokio::time::sleep;

pub async fn create(req: WatchArguments) -> Result<(), WatchError> {
    if should_ignore(&req.event, &req.path, &req.last_seen).await {
        return Ok(());
    }

    let WatchArguments {
        info,
        event,
        debounce,
        ..
    } = req;

    let info = info.lock().await;

    let WatchTarget {
        client,
        config,
        kind,
        ..
    } = info
        .try_into_target()
        .map_err(|e| WatchError::Stop(format!("Expected target got {:?}", e)))?;

    let BuildConfiguration { .. } = config;

    let Client { pid, root } = client;

    tracing::debug!("Rebuilding for {:#?}", &event);

    let state = DAEMON_STATE.clone();
    let state = state.lock().await;

    let nvim = state
        .clients
        .get(&pid)
        .ok_or_else(|| WatchError::Stop("Fail to find nvim instance with given pid".to_string()))?;

    let stream = match kind {
        WatchKind::Build => xcode::stream(&root, &["build"], &config)
            .await
            .map_err(WatchError::r#continue)?,

        WatchKind::Run => {
            nvim.log_error("Watch", "Run is not supported yet! .. aborting")
                .await
                .map_err(WatchError::stop)?;

            // NOTE: Update state before exiting
            state
                .sync_client_state()
                .await
                .map_err(|e| WatchError::Stop(format!("Fail to update state {e}")))?;

            return Err(WatchError::Stop("Run not supported yet!".into()));
        }
    };

    WatchLogger::new(nvim, "Build", &config)
        .log_stream(stream, None, false, false)
        .await
        .map_err(WatchError::r#continue)?;

    let mut debounce = debounce.lock().await;

    *debounce = SystemTime::now();

    Ok(())
}

async fn should_ignore(event: &Event, path: &PathBuf, last_seen: &Arc<Mutex<String>>) -> bool {
    use notify::event::DataChange;

    async fn should_ignore(event: &Event, path: &PathBuf, last_seen: &Arc<Mutex<String>>) -> bool {
        if let EventKind::Modify(ModifyKind::Name(_)) = &event.kind {
            let path_string = path.to_string_lossy();
            // HACK: only account for new path and skip duplications
            if !path.exists() || super::is_seen(last_seen.clone(), &path_string).await {
                return true;
            }
            sleep(Duration::new(1, 0)).await;
        }
        false
    }

    fn is_rename_event(event: &Event) -> bool {
        matches!(event.kind, EventKind::Modify(ModifyKind::Name(_)))
    }

    fn is_create_event(event: &Event) -> bool {
        matches!(event.kind, EventKind::Create(_))
    }

    fn is_modified_event(event: &Event) -> bool {
        matches!(
            event.kind,
            EventKind::Modify(ModifyKind::Data(DataChange::Content))
        )
    }

    !(is_modified_event(event) || is_create_event(event) || is_rename_event(event))
        || should_ignore(event, path, last_seen).await
}