# MoonWitness module runtime integration

The optional `moonwitness-modules` feature connects the typed MoonWitness
module kernel to the Loco application lifecycle. It is disabled by default and
implies `with-db`.

## Enable the feature

```toml
[dependencies]
loco-rs = { version = "1.3", features = ["moonwitness-modules"] }
```

## Register modules

Build a validated asynchronous registry and add its initializer to the
application's normal initializer list:

```rust,ignore
use loco_rs::{
    app::{AppContext, Initializer, Hooks},
    modules::{AsyncModuleRegistryBuilder, ModulesInitializer},
    Result,
};

async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
    let mut modules = AsyncModuleRegistryBuilder::new();
    modules.register(MyCoreModule::new())?;
    modules.register(MyAuditModule::new())?;

    let registry = modules.build()?;
    Ok(vec![Box::new(ModulesInitializer::new(registry))])
}
```

The initializer starts modules before the application begins serving. It
injects the current SeaORM `DatabaseConnection` into `ModuleContext` as a
typed service, so a module can resolve it with
`context.service::<sea_orm::DatabaseConnection>()`. Other string configuration
values can be passed through `ModulesInitializer::with_configuration`.

## Shutdown

The default Loco `Hooks::serve` path calls the application's `on_shutdown`
hook and then deactivates the module host. If an application overrides
`Hooks::serve`, it must call `loco_rs::modules::shutdown(ctx).await` during
its shutdown path. The running host can be accessed through
`loco_rs::modules::runtime(ctx)`.

## Boundaries

- Module loading is static Rust registration; no native dynamic library loading.
- The runtime is opt-in and does not change existing apps unless the feature
  and initializer are enabled.
- The module kernel does not replace Loco's database migrator. A real
  database-specific migration backend is still required before module-owned
  migrations can execute.
- The reference audit add-on stores records in memory and does not intercept
  ORM writes. Applications must publish its typed event after transaction
  commit; durable audit storage is a separate integration.
- This integration does not claim tenant isolation or production readiness.
