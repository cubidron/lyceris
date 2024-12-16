#[macro_export]
macro_rules! emit {
    (mut $emitter:expr, $event_name:expr, $data:expr) => {
        if let Some(ref mut emitter) = $emitter {
            emitter.emit($event_name, $data);
        }
    };
}