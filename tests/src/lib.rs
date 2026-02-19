#[cfg(test)]
mod tests {
    use anyhow::{Result, anyhow};
    use core::panic;
    use once_cell::sync::Lazy;
    use std::{
        net::TcpListener,
        path::PathBuf,
        process::{Child, Command, Stdio},
        time::Duration,
    };

    static COMPONENTIZE_GO_PATH: once_cell::sync::Lazy<PathBuf> = Lazy::new(|| {
        let test_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root_manifest = test_manifest.parent().unwrap();
        let build_output = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .args([
                "--manifest-path",
                root_manifest.join("Cargo.toml").to_str().unwrap(),
            ])
            .output()
            .expect("failed to build componentize-go");

        if !build_output.status.success() {
            panic!("{}", String::from_utf8_lossy(&build_output.stderr));
        }

        root_manifest.join("target/release/componentize-go")
    });

    // TODO: Once the patch is merged in Big Go, this needs to be removed.
    async fn patched_go_path() -> PathBuf {
        let test_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root_manifest = test_manifest.parent().unwrap();

        // Determine OS and architecture
        let os = match std::env::consts::OS {
            "macos" => "darwin",
            "linux" => "linux",
            "windows" => "windows",
            bad_os => panic!("OS not supported: {bad_os}"),
        };

        // Map to Go's naming conventions
        let arch = match std::env::consts::ARCH {
            "aarch64" => "arm64",
            "x86_64" => "amd64",
            bad_arch => panic!("ARCH not supported: {bad_arch}"),
        };

        let go_dir = format!("go-{os}-{arch}-bootstrap");
        let go_path = root_manifest.join(&go_dir);
        let go_bin = go_path.join("bin").join("go");

        // Skip if already installed
        if go_bin.exists() {
            return go_bin;
        }

        // Download the patched Go toolchain
        let archive_name = format!("{go_dir}.tbz");
        let archive_path = root_manifest.join(&archive_name);
        let download_url = format!(
            "https://github.com/dicej/go/releases/download/go1.25.5-wasi-on-idle/{archive_name}"
        );

        println!("Downloading patched Go from {download_url}");
        let response = reqwest::get(&download_url)
            .await
            .expect("Failed to download patched Go");

        std::fs::write(
            &archive_path,
            response.bytes().await.expect("Failed to read download"),
        )
        .expect("Failed to write archive");

        // Extract the archive
        println!("Extracting {} to {}", archive_name, root_manifest.display());
        let tar_file = std::fs::File::open(&archive_path).expect("Failed to open archive");
        let tar_decoder = bzip2::read::BzDecoder::new(tar_file);
        let mut archive = tar::Archive::new(tar_decoder);
        archive
            .unpack(root_manifest)
            .expect("Failed to extract archive");

        // Clean up archive
        std::fs::remove_file(&archive_path).ok();

        go_bin
    }

    struct App<'a> {
        /// The path to the example application
        path: String,
        /// The WIT world to target
        world: String,
        /// The output path of the wasm file
        wasm_path: String,
        /// The path to the directory containing the WIT files
        wit_path: String,
        /// The child process ID of a running wasm app
        process: Option<Child>,
        /// Any tests that need to be compiled and run as such
        tests: Option<&'a [Test<'a>]>,
    }

    struct Test<'a> {
        should_fail: bool,
        pkg_path: &'a str,
    }

    impl<'a> App<'a> {
        /// Create a new app runner.
        fn new(path: &'a str, world: &'a str, tests: Option<&'a [Test<'a>]>) -> Self {
            let path = componentize_go::utils::make_path_absolute(&PathBuf::from(path))
                .expect("failed to make app path absolute");

            App {
                path: path
                    .clone()
                    .to_str()
                    .expect("app path is not valid unicode")
                    .to_string(),
                world: world.to_string(),
                wasm_path: path
                    .join("main.wasm")
                    .to_str()
                    .expect("wasm_path is not valid unicode")
                    .to_string(),
                wit_path: path
                    .join("wit")
                    .to_str()
                    .expect("wit_path is not valid unicode")
                    .to_string(),
                process: None,
                tests,
            }
        }

        // Build unit tests with componentize-go
        fn build_tests(&self, go: Option<&PathBuf>) -> Result<()> {
            let test_pkgs = if let Some(pkgs) = self.tests {
                pkgs
            } else {
                return Err(anyhow!(
                    "Please include the test_pkg_paths when creating App::new()"
                ));
            };

            self.generate_bindings(go)?;

            let mut test_cmd = Command::new(COMPONENTIZE_GO_PATH.as_path());
            test_cmd
                .args(["-w", &self.world])
                .args(["-d", &self.wit_path])
                .arg("test")
                .arg("--wasip1");

            // Add all the paths to the packages that have unit tests to compile
            for test in test_pkgs.iter() {
                test_cmd.args(["--pkg", test.pkg_path]);
            }

            // `go test -c` needs to be in the same path as the go.mod file.
            test_cmd.current_dir(&self.path);

            let test_output = test_cmd.output().expect(&format!(
                "failed to execute componentize-go for \"{}\"",
                self.path
            ));

            if !test_output.status.success() {
                return Err(anyhow!(
                    "failed to build application \"{}\": {}",
                    self.path,
                    String::from_utf8_lossy(&test_output.stderr)
                ));
            }

            Ok(())
        }

        fn run_tests(&self) -> Result<()> {
            let example_dir = PathBuf::from(&self.path);
            if let Some(tests) = self.tests {
                let mut test_errors: Vec<String> = vec![];
                for test in tests.iter() {
                    let wasm_file = example_dir.join(componentize_go::cmd_test::get_test_filename(
                        &PathBuf::from(test.pkg_path),
                    ));
                    match Command::new("wasmtime")
                        .args(["run", wasm_file.to_str().unwrap()])
                        .output()
                    {
                        Ok(output) => {
                            let succeeded = output.status.success();
                            if test.should_fail && succeeded {
                                test_errors.push(format!(
                                    "The '{}' tests should have failed",
                                    test.pkg_path
                                ));
                            } else if !test.should_fail && !succeeded {
                                test_errors.push(format!("The '{}' tests should have passed, but failed with the following output:\n\n{}", test.pkg_path, String::from_utf8_lossy(&output.stdout)));
                            }
                        }
                        Err(e) => {
                            test_errors.push(format!(
                                "Failed to run wasmtime for '{}': {}",
                                test.pkg_path, e
                            ));
                        }
                    }
                }

                if !test_errors.is_empty() {
                    let err_msg = format!(
                        "{}{}{}",
                        "\n====================\n",
                        &test_errors.join("\n\n====================\n"),
                        "\n\n====================\n"
                    );
                    return Err(anyhow!(err_msg));
                }
            } else {
                return Err(anyhow!(
                    "Please include the test_pkg_paths when creating App::new()"
                ));
            }

            Ok(())
        }

        /// Build the app with componentize-go.
        fn build(&self, go: Option<&PathBuf>) -> Result<()> {
            self.generate_bindings(go)?;

            // Build component
            let mut build_cmd = Command::new(COMPONENTIZE_GO_PATH.as_path());
            build_cmd
                .args(["-w", &self.world])
                .args(["-d", &self.wit_path])
                .arg("build")
                .args(["-o", &self.wasm_path]);

            if let Some(go_path) = go.as_ref() {
                build_cmd.args(["--go", go_path.to_str().unwrap()]);
            }

            // Run `go build` in the same directory as the go.mod file.
            build_cmd.current_dir(&self.path);

            let build_output = build_cmd.output().expect(&format!(
                "failed to execute componentize-go for \"{}\"",
                self.path
            ));

            if !build_output.status.success() {
                return Err(anyhow!(
                    "failed to build application \"{}\": {}",
                    self.path,
                    String::from_utf8_lossy(&build_output.stderr)
                ));
            }

            Ok(())
        }

        fn generate_bindings(&self, go: Option<&PathBuf>) -> Result<()> {
            let bindings_output = Command::new(COMPONENTIZE_GO_PATH.as_path())
                .args(["-w", &self.world])
                .args(["-d", &self.wit_path])
                .arg("bindings")
                .args(["-o", &self.path])
                .current_dir(&self.path)
                .output()
                .expect(&format!(
                    "failed to generate bindings for application \"{}\"",
                    &self.path
                ));
            if !bindings_output.status.success() {
                return Err(anyhow!(
                    "{}",
                    String::from_utf8_lossy(&bindings_output.stderr)
                ));
            }

            // Tidy Go mod
            let tidy_output = Command::new(if let Some(path) = go.as_ref() {
                String::from(path.to_str().unwrap())
            } else {
                // Default to PATH
                "go".to_string()
            })
            .arg("mod")
            .arg("tidy")
            .current_dir(&self.path)
            .output()
            .expect("failed to tidy Go mod");
            if !tidy_output.status.success() {
                return Err(anyhow!("{}", String::from_utf8_lossy(&tidy_output.stderr)));
            }

            Ok(())
        }

        /// Run the app and check the output.
        async fn run(&mut self, route: &str, expected_response: &str) -> Result<()> {
            let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a free port");
            let addr = listener.local_addr().expect("Failed to get local address");
            let port = addr.port();
            drop(listener);

            let child = Command::new("wasmtime")
                .arg("serve")
                .args(["--addr", &format!("0.0.0.0:{port}")])
                .arg("-Sp3,cli")
                .arg("-Wcomponent-model-async")
                .arg(&self.wasm_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start wasmtime serve");

            // Storing for cleanup on drop.
            self.process = Some(child);

            let start = std::time::Instant::now();
            loop {
                match reqwest::get(format!("http://localhost:{port}{route}")).await {
                    Ok(r) => {
                        let actual = r.text().await.expect("Failed to read response");
                        assert_eq!(&actual, expected_response);
                        return Ok(());
                    }
                    Err(e) => {
                        if start.elapsed() > Duration::from_secs(5) {
                            return Err(anyhow!("Unable to reach the app: {e}"));
                        }
                    }
                }
            }
        }
    }

    impl<'a> Drop for App<'a> {
        fn drop(&mut self) {
            if let Some(child) = &mut self.process {
                _ = child.kill()
            }
        }
    }

    #[tokio::test]
    async fn example_wasip2() {
        let unit_tests = [
            Test {
                should_fail: false,
                pkg_path: "./unit_tests_should_pass",
            },
            Test {
                should_fail: true,
                pkg_path: "./unit_tests_should_fail",
            },
        ];

        let mut app = App::new("../examples/wasip2", "wasip2-example", Some(&unit_tests));

        app.build(None).expect("failed to build app");

        app.run("/", "Hello, world!")
            .await
            .expect("app failed to run");

        app.build_tests(None)
            .expect("failed to build app unit tests");

        app.run_tests()
            .expect("tests succeeded/failed when they should not have");
    }

    #[tokio::test]
    async fn example_wasip3() {
        let mut app = App::new("../examples/wasip3", "wasip3-example", None);
        app.build(Some(&patched_go_path().await))
            .expect("failed to build app");
        app.run("/hello", "Hello, world!")
            .await
            .expect("app failed to run");
    }
}
