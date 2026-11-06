//! Integration coverage for the opt-in MoonWitness module kernel.

#![cfg(feature = "mw-modules")]

use loco_rs::modules::{Module, ModuleContext, ModuleHost, ModuleManifest, ModuleRegistryBuilder};

struct TestModule {
    manifest: ModuleManifest,
}

impl Module for TestModule {
    fn manifest(&self) -> &ModuleManifest {
        &self.manifest
    }
}

fn parse_manifest(source: &str) -> ModuleManifest {
    ModuleManifest::from_toml(source).expect("test manifest should be valid")
}

#[test]
fn module_kernel_resolves_dependencies_and_runs_lifecycle() {
    let core = TestModule {
        manifest: parse_manifest(
            r#"
name = "mw.core"
version = "0.1.0"
api_version = "0.1.0"
"#,
        ),
    };
    let api = TestModule {
        manifest: parse_manifest(
            r#"
name = "mw.api"
version = "0.1.0"
api_version = "0.1.0"
dependencies = [{ name = "mw.core", version = "^0.1" }]
"#,
        ),
    };

    let mut builder = ModuleRegistryBuilder::new();
    builder.register(api).expect("API module should register");
    builder.register(core).expect("core module should register");

    let registry = builder.build().expect("dependency graph should resolve");
    assert_eq!(
        registry.activation_plan().ordered_modules(),
        &["mw.core".to_owned(), "mw.api".to_owned()]
    );

    let mut host = ModuleHost::new(registry, ModuleContext::default());
    host.start().expect("module lifecycle should start");
    assert_eq!(
        host.active_modules(),
        &["mw.core".to_owned(), "mw.api".to_owned()]
    );

    host.stop().expect("module lifecycle should stop");
    assert!(host.active_modules().is_empty());
}
