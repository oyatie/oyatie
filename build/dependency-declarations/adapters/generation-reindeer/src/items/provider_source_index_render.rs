fn render_public_rule_name_resolver_v1() -> Result<String, ReindeerProviderAdaptationErrorV1> {
    render_provider_tokens_text_v1(quote::quote! {
        fn collect_public_targets<'meta>(
            entries: impl IntoIterator<
                Item = ((PackageId, TargetReq<'meta>), Option<&'meta str>),
            >,
        ) -> anyhow::Result<
            BTreeMap<(PackageId, TargetReq<'meta>), Option<&'meta str>>,
        > {
            let mut targets = BTreeMap::new();
            for (target, name) in entries {
                match targets.entry(target) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(name);
                    }
                    std::collections::btree_map::Entry::Occupied(entry)
                        if entry.get() != &name =>
                    {
                        anyhow::bail!("one public target has conflicting logical names");
                    }
                    std::collections::btree_map::Entry::Occupied(_) => {}
                }
            }
            Ok(targets)
        }

        fn collect_public_packages<'meta>(
            targets: &BTreeMap<
                (PackageId, TargetReq<'meta>),
                Option<&'meta str>,
            >,
        ) -> anyhow::Result<BTreeMap<PackageId, Option<&'meta str>>> {
            let mut packages = BTreeMap::new();
            for (&(package_id, _), &name) in targets {
                match packages.entry(package_id) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(name);
                    }
                    std::collections::btree_map::Entry::Occupied(entry)
                        if entry.get() != &name =>
                    {
                        anyhow::bail!("one package identity has conflicting public names");
                    }
                    std::collections::btree_map::Entry::Occupied(_) => {}
                }
            }
            Ok(packages)
        }

        fn resolve_public_rule_names<'meta>(
            public_packages: &BTreeMap<PackageId, Option<&'meta str>>,
            packages: &HashMap<PackageId, &'meta Manifest>,
            collision_info: &CollisionInfo,
        ) -> anyhow::Result<BTreeMap<PackageId, Name>> {
            let mut logical_name_counts = HashMap::default();
            for (&package_id, rename) in public_packages {
                let package = packages[&package_id];
                let logical_name = rename.unwrap_or(package.name.as_str());
                let count = logical_name_counts.entry(logical_name).or_insert(0usize);
                *count = count
                    .checked_add(1)
                    .context("public package name count overflow")?;
            }

            let mut target_owners = HashMap::default();
            let mut public_rule_names = BTreeMap::new();
            for (&package_id, rename) in public_packages {
                let package = packages[&package_id];
                let logical_name = rename.unwrap_or(package.name.as_str());
                let target_name = if logical_name_counts[logical_name] == 1 {
                    logical_name.to_owned()
                } else {
                    format!("{}-{}", logical_name, collision_info.target_version(package))
                };
                if let Some(previous) = target_owners.insert(target_name.clone(), package_id)
                    && previous != package_id
                {
                    anyhow::bail!(
                        "public target name {target_name} has multiple package identities"
                    );
                }
                if public_rule_names
                    .insert(package_id, Name(target_name))
                    .is_some()
                {
                    anyhow::bail!("public package identity occurs more than once");
                }
            }
            Ok(public_rule_names)
        }
    })
}

fn render_public_package_resolution_v1() -> String {
    concat!(
        "index.public_packages = collect_public_packages(&index.public_targets)?;\n",
        "        let collision_info =\n",
        "            CollisionInfo::new(&metadata.packages.iter().collect::<Vec<_>>());\n",
        "        index.public_rule_names = resolve_public_rule_names(\n",
        "            &index.public_packages,\n",
        "            &index.pkgid_to_pkg,\n",
        "            &collision_info,\n",
        "        )?;",
    )
    .to_owned()
}

fn render_public_rule_name_methods_v1(
) -> Result<String, ReindeerProviderAdaptationErrorV1> {
    render_provider_tokens_text_v1(quote::quote! {
        /// Return the package's collision-safe public rule name.
        pub fn public_rule_name(&self, pkg: &'meta Manifest) -> Name {
            self.public_rule_names
                .get(&pkg.id)
                .cloned()
                .unwrap_or_else(|| Name(pkg.name.clone()))
        }

        /// Return every collision-safe public rule name.
        pub fn public_rule_names(&self) -> impl Iterator<Item = &str> {
            self.public_rule_names.values().map(|name| name.0.as_str())
        }
    })
}

fn render_public_rule_name_tests_v1() -> Result<String, ReindeerProviderAdaptationErrorV1> {
    render_provider_tokens_text_v1(quote::quote! {
        #[cfg(test)]
        mod artifact_public_rule_name_tests {
            use super::*;

            #[test]
            fn one_target_with_conflicting_names_refuses() {
                let (package_id, _) = public_name_input("fixture", "1.0.0");
                let error = collect_public_targets([
                    ((package_id, TargetReq::Lib), None),
                    ((package_id, TargetReq::Lib), Some("renamed-fixture")),
                ])
                .unwrap_err();
                assert!(error.to_string().contains("conflicting logical names"));
            }

            #[test]
            fn one_package_with_conflicting_target_names_refuses() {
                let (package_id, _) = public_name_input("fixture", "1.0.0");
                let targets = BTreeMap::from([
                    ((package_id, TargetReq::Lib), None),
                    (
                        (package_id, TargetReq::EveryBin),
                        Some("renamed-fixture"),
                    ),
                ]);
                let error = collect_public_packages(&targets).unwrap_err();
                assert!(error.to_string().contains("conflicting public names"));
            }

            #[test]
            fn same_compatibility_slot_with_multiple_identities_refuses() {
                let first = public_name_input("fixture", "0.5.2");
                let second = public_name_input("fixture", "0.5.3");
                let public_packages = BTreeMap::from([(first.0, None), (second.0, None)]);
                let packages = [(first.0, &first.1), (second.0, &second.1)]
                    .into_iter()
                    .collect();
                let collision_info = CollisionInfo::new(&[&first.1, &second.1]);

                let error = resolve_public_rule_names(
                    &public_packages,
                    &packages,
                    &collision_info,
                )
                .unwrap_err();
                assert!(error.to_string().contains("multiple package identities"));
            }

            #[test]
            fn distinct_compatibility_slots_receive_version_qualified_names() {
                let first = public_name_input("fixture", "0.4.13");
                let second = public_name_input("fixture", "0.5.3");
                let public_packages = BTreeMap::from([(first.0, None), (second.0, None)]);
                let packages = [(first.0, &first.1), (second.0, &second.1)]
                    .into_iter()
                    .collect();
                let collision_info = CollisionInfo::new(&[&first.1, &second.1]);

                let names = resolve_public_rule_names(
                    &public_packages,
                    &packages,
                    &collision_info,
                )
                .unwrap();

                assert_eq!(names[&first.0].0, "fixture-0.4");
                assert_eq!(names[&second.0].0, "fixture-0.5");
            }

            #[test]
            fn one_public_identity_keeps_the_unversioned_name() {
                let package = public_name_input("fixture", "0.5.3");
                let public_packages = BTreeMap::from([(package.0, None)]);
                let packages = [(package.0, &package.1)].into_iter().collect();
                let collision_info = CollisionInfo::new(&[&package.1]);

                let names = resolve_public_rule_names(
                    &public_packages,
                    &packages,
                    &collision_info,
                )
                .unwrap();

                assert_eq!(names[&package.0].0, "fixture");
            }

            fn public_name_input(name: &str, version: &str) -> (PackageId, Manifest) {
                use cargo::core::PackageId;
                use cargo::util::interning::InternedString;

                let version = semver::Version::parse(version).unwrap();
                let source_id = cargo::core::SourceId::from_url(
                    "registry+https://github.com/rust-lang/crates.io-index",
                )
                .unwrap();
                let package_id = PackageId::new(
                    InternedString::new(name),
                    version.clone(),
                    source_id,
                );
                let package = Manifest {
                    name: name.to_owned(),
                    version,
                    id: package_id,
                    license: None,
                    license_file: None,
                    description: None,
                    source: crate::cargo::Source::CratesIo,
                    dependencies: vec![],
                    targets: vec![],
                    features: Default::default(),
                    manifest_path: Default::default(),
                    authors: vec![],
                    readme: None,
                    repository: None,
                    homepage: None,
                    edition: crate::cargo::Edition::Rust2021,
                    links: None,
                    rust_version: None,
                };
                (package_id, package)
            }
        }
    })
}
