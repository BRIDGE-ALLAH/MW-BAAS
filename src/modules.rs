//! Optional integration between Loco application lifecycle and MoonWitness modules.
//!
//! Enable the moonwitness-modules feature and add ModulesInitializer to the
//! application's initializer list. The initializer starts the async module
//! host before serving and the default Loco server shutdown path stops it
//! after the application's on_shutdown hook.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use crate::{
    app::{AppContext, Initializer},
    errors::Error,
    Result,
};

pub use moonwitness_module::*;

/// Loco initializer that starts a MoonWitness module registry.
pub struct ModulesInitializer {
    registry: Mutex<Option<AsyncModuleRegistry>>,
    configuration: BTreeMap<String, String>,
}

impl ModulesInitializer {
    /// Create an initializer from a validated module registry.
    ///
    /// The registry is consumed exactly once when the application starts.
    pub fn new(registry: AsyncModuleRegistry) -> Self {
        Self { registry: Mutex::new(Some(registry)), configuration: BTreeMap::new() }
    }

    /// Add string configuration values exposed to modules through ModuleContext.
    #[must_use]
    pub fn with_configuration(
        mut self,
        configuration: BTreeMap<String, String>,
    ) -> Self {
        self.configuration = configuration;
        self
    }
}

#[async_trait]
impl Initializer for ModulesInitializer {
    fn name(&self) -> String { "moonwitness-modules".to_owned() }

    async fn before_run(&self, app_context: &AppContext) -> Result<()> {
        if app_context.shared_store.contains::<Arc<tokio::sync::Mutex<AsyncModuleHost>>>() {
            return Err(Error::string("MoonWitness module runtime is already registered"));
        }
        let registry = {
            let mut registry = self.registry.lock()
                .map_err(|_| Error::string("MoonWitness module registry lock is poisoned"))?;
            registry.take()
                .ok_or_else(|| Error::string("MoonWitness module initializer has already run"))?
        };

        let mut services = ServiceRegistryBuilder::new();
        services.register(app_context.db.clone())
            .map_err(|error: LifecycleError| Error::string(&error.to_string()))?;
        let context = ModuleContext::with_services(self.configuration.clone(), services.build());
        let mut host = AsyncModuleHost::new(registry, context);
        if let Err(error) = host.start().await {
            if let Err(cleanup_error) = host.stop().await {
                // Preserve the host so a later shutdown call can retry cleanup.
                // Dropping it here would lose the only handles to partially
                // activated modules that failed their rollback.
                app_context.shared_store.insert(Arc::new(
                    tokio::sync::Mutex::new(host),
                ));
                tracing::error!(%cleanup_error, "MoonWitness module rollback remains pending");
            }
            return Err(Error::string(&error.to_string()));
        }

        app_context.shared_store.insert(Arc::new(tokio::sync::Mutex::new(host)));
        Ok(())
    }
}

/// Return the running module host, if the initializer has completed.
pub fn runtime(
    app_context: &AppContext,
) -> Option<Arc<tokio::sync::Mutex<AsyncModuleHost>>> {
    app_context.shared_store.get::<Arc<tokio::sync::Mutex<AsyncModuleHost>>>()
}

/// Stop the registered module host. Safe to call when no module initializer ran.
pub async fn shutdown(app_context: &AppContext) -> Result<()> {
    let Some(host) = runtime(app_context) else { return Ok(()); };
    host.lock().await.stop().await
        .map_err(|error| Error::string(&error.to_string()))?;
    // Keep the host registered when shutdown fails so callers can retry.
    // Once every module has stopped, remove the stale runtime handle.
    app_context.shared_store.remove::<Arc<tokio::sync::Mutex<AsyncModuleHost>>>();
    Ok(())
}
