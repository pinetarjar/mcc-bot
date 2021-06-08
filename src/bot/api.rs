use frankenstein::{Api, Error, GetUpdatesParams, MethodResponse, TelegramApi, Update};
use std::sync::{Arc};
use tokio::task;
use tokio::task::JoinHandle;
use tokio::sync::{Mutex, RwLock};

pub struct AsyncApiWrapper {
    api: Arc<Mutex<Api>>,
    update_params: Arc<RwLock<GetUpdatesParams>>,
}

impl AsyncApiWrapper {
    pub fn new(token: &str) -> Self {
        let mut update = GetUpdatesParams::new();
        update.set_timeout(Some(1));
        update.set_allowed_updates(Some(vec!["message".to_string()]));

        Self {
            api: Arc::new(Mutex::new(Api::new(token.to_string()))),
            update_params: Arc::new(RwLock::new(update)),
        }
    }

    pub fn get_updates(&self) -> JoinHandle<Result<Vec<Update>, Error>> {
        let api = self.api.clone();
        let update_params = self.update_params.clone();

        task::spawn(async move {
            {
                let updates: Vec<Update>;

                {
                    let locked_update_params = update_params.read().await;
                    {
                        let locked_api = api.lock().await;
                        updates = locked_api.get_updates(&*locked_update_params)?.result;
                    }
                }

                // Telegram API expect confirmation of update receiving by setting offset
                // greater than latest one by one.
                // We expect that one process messages gracefully or skip it.
                if let Some(latest) = updates.iter().map(|u| u.update_id).max() {
                    let mut locked_update_params = update_params.write().await;
                    locked_update_params.set_offset(Some(latest + 1));
                }

                return Ok(updates);
            }
        })
    }
}

impl Clone for AsyncApiWrapper {
    fn clone(&self) -> Self {
        Self {
            api: self.api.clone(),
            update_params: self.update_params.clone(),
        }
    }
}