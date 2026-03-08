
            /// Returns the `rustc` SemVer version and additional metadata
            /// like the git short hash and build date.
            pub fn version_meta() -> VersionMeta {
                VersionMeta {
                    semver: Version {
                        major: 1,
                        minor: 93,
                        patch: 0,
                        pre: vec![],
                        build: vec![],
                    },
                    host: "aarch64-apple-darwin".to_owned(),
                    short_version_string: "rustc 1.93.0 (254b59607 2026-01-19)".to_owned(),
                    commit_hash: Some("254b59607d4417e9dffbc307138ae5c86280fe4c".to_owned()),
                    commit_date: Some("2026-01-19".to_owned()),
                    build_date: None,
                    channel: Channel::Stable,
                }
            }
            