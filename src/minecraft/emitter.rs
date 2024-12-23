use event_emitter_rs::EventEmitter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct Emitter {
    pub wrap: Arc<Mutex<EventEmitter>>,
}

pub trait Emit {
    #[allow(async_fn_in_trait)]
    async fn emit<T: Serialize>(&self, event_name: &str, data: T);
}

impl Emit for Option<&Emitter> {
    async fn emit<T: Serialize>(&self, event_name: &str, data: T) {
        if let Some(emitter) = self {
            emitter.wrap.lock().await.emit(event_name, data);
        }
    }
}

impl Emitter {
    pub async fn emit<T: Serialize>(&self, event_name: &str, data: T) {
        self.wrap.lock().await.emit(event_name, data);
    }

    pub async fn on<F, T>(&self, event_name: &str, listener: F)
    where
        F: Fn(T) + Send + Sync + 'static,
        T: for<'de> Deserialize<'de> + Serialize,
    {
        self.wrap.lock().await.on(event_name, listener);
    }
}
