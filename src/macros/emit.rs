#[macro_export]
macro_rules! emit {
    ($emitter:expr, $event_name:expr, $data:expr) => {
        if let Some(ref emitter) = $emitter {
            emitter.lock().await.emit($event_name, $data);
        }
    };
}