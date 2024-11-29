# lyceris
An open source minecraft launcher library. 
It is still under heavy development and using in production not suggested!

! This library will be re-constructed soon!

# Quick Start
```rust
  async fn launch_game() {
      let mut instance = Instance::new();

      instance
          .launch(
              None::<()>,
              Config {
                  ..Config::default()
              },
              |e| println!("{:?}", e),
          )
          .await
          .unwrap();

      loop {
          println!("Polling check");
          if let Some(status) = instance.poll() {
              println!("Closed!");
          }

          tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
      }
  }
```
