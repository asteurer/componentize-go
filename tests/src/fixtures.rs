use anyhow::Result;
use std::env;
use crate::harness::App;

#[tokio::test]
    async fn fixture_multiple_worlds() -> Result<()> {
        let cwd = env::current_dir()?;
        let app_dir = cwd
            .parent()
            .unwrap()
            .join("tests/fixtures")
            .join("multiple-worlds");
        let mut app = App::new(
            &app_dir,
            &[&app_dir.join("wit1"), &app_dir.join("wit2")],
            &["world1", "world2"],
            None,
            true,
        );
        app.build_component().expect("failed to build app");
        app.run_component("/hello", "Hello, world!")
            .await
            .expect("app failed to run");
        Ok(())
    }