pub mod fabric;

pub enum CustomPackage {
    Fabric(fabric::serde::Package),
}