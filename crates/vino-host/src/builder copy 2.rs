use actix::prelude::*;
use std::sync::RwLock;

use crate::Result;

/// The HostBuilder builds the configuration for a Vino Host
pub struct HostBuilder {}

impl Default for HostBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HostBuilder {
    /// Creates a new host builder
    pub fn new() -> HostBuilder {
        HostBuilder {}
    }

    pub async fn start(self) -> Result<Host> {
        let host = self.build();
        host.start().await?;
        Ok(host)
    }

    /// Constructs an instance of a Vino host.
    pub fn build(self) -> Host {
        Host {
            started: RwLock::new(false),
        }
    }
}

/// A Vino Host wraps a Vino runtime with server functionality like persistence,
pub struct Host {
    started: RwLock<bool>,
}

impl Host {
    /// Starts the host. This call is non-blocking, so it is up to the consumer
    /// to provide some form of parking or waiting (e.g. wait for a Ctrl-C signal).
    ///
    pub async fn start(&self) -> Result<()> {
        *self.started.write().unwrap() = true;

        Ok(())
    }

    /// Stops a running host. Be aware that this function may terminate before the host has
    /// finished disposing of all of its resources.
    pub async fn stop(&self) {
        *self.started.write().unwrap() = false;
        if let Some(system) = System::try_current() {
            system.stop();
        }
    }

    pub async fn wait_for_sigint(&self) {
        actix_rt::signal::ctrl_c().await.unwrap();
    }

    fn ensure_started(&self) -> Result<()> {
        ensure!(
            *self.started.read().unwrap(),
            "Activity cannot be performed, host has not been started"
        );

        ensure!(
            System::try_current().is_some(),
            "No actix rt system is running. Cannot perform host activity."
        );

        Ok(())
    }

    pub fn is_started(&self) -> bool {
        *self.started.read().unwrap()
    }

    pub(crate) fn native_target() -> String {
        format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
    }
}

#[cfg(test)]
mod test {
    use crate::HostBuilder;

    #[test]
    fn is_send() {
        let h = HostBuilder::new().build();
        assert_is_send(h);
    }

    fn assert_is_send<T: Send>(_input: T) {}
}
