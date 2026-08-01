//! Client-stack discipline gate (ADR-0185).
//!
//! # Why this crate exists
//!
//! ADR-0185 mandates native-per-platform client rendering for every
//! oyatie end-user product. The discipline is:
//!
//! - Web Phase 1: SvelteKit; Phase 2: Leptos. NEVER both simultaneously
//!   for the same surface.
//! - Apple (iOS / iPadOS / macOS / watchOS / visionOS): Swift + SwiftUI
//!   ONLY. No KMP klib imports.
//! - Android: Kotlin + Jetpack Compose. KMP module is Android-scope only.
//! - Windows: WinUI 3 + .NET. No Avalonia, no MAUI, no Electron.
//! - Linux: GTK 4 + gtk-rs + libadwaita (Rust). No Tauri, no Electron,
//!   no Qt, no Flutter, no Slint, no Iced.
//! - Every client declares its OpenAPI 3.2.0 codegen recipe.
//! - Banned: React, Vue, Flutter, Electron, Cordova.
//! - Linux clients MUST consume the `oya-client-shared-rust` workspace
//!   crate.
//!
//! This kernel evaluates one or more `client-manifest.json` documents
//! against those rules and returns violations.
//!
//! # Layer
//!
//! `domain` (port-in-kernel, ADR-0056).
//!
//! # Naming justification
//!
//! `check-client-stack-discipline` follows the ADR-0532/0533 de-branded grammar:
//! `<group:check>-<axis:client-stack-discipline>`.
//!
//! # Supersession lint (ADR-0393)
//!
//! ADR-0393 makes Leptos the canonical app-shell frontend and retires the
//! ADR-0372 SolidJS stack in full. A superseded stack must never reappear in
//! any active client manifest.
//!
//! # References
//!
//! - ADR-0185 — Workflow Studio client stack.
//! - ADR-0393 — Leptos canonical app-shell frontend (supersedes ADR-0372
//!   and mandates the superseded-reference lint).

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]
#![allow(clippy::result_large_err)]

use std::fmt;

/// One supplied `client-manifest.json` (plus optional surrounding code
/// surface text). Callers load the manifest + any client-tree dependency
/// declarations and pass the concatenated text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientManifest {
    /// Logical surface key (e.g. "web-sveltekit", "apple-ios",
    /// "android", "windows", "linux").
    pub surface: String,
    /// Path the manifest was loaded from.
    pub path: String,
    /// Manifest contents (typically JSON; the validator does substring
    /// detection so any text format works).
    pub contents: String,
}

/// Success report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientStackReport {
    pub manifests_checked: usize,
    pub surfaces_audited: usize,
}

/// Violation record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientStackViolation {
    pub surface: String,
    pub manifest_path: String,
    pub kind: ViolationKind,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ViolationKind {
    /// Web surface declares both SvelteKit AND Leptos simultaneously.
    WebDualStackForbidden,
    /// Web surface declares neither SvelteKit nor Leptos.
    WebStackMissing,
    /// Apple surface imports a KMP klib artifact (per ADR-0185 Apple is
    /// pure Swift; no KMP).
    AppleImportsKmp,
    /// Mobile surface declares an UI framework other than the canonical
    /// one for the platform.
    MobileNonCanonicalUi,
    /// Windows surface declares a non-WinUI3 stack.
    WindowsNonWinUi3,
    /// Linux surface declares a non-GTK4/libadwaita stack.
    LinuxNonGtk4,
    /// Linux surface omits the `oya-client-shared-rust` dependency.
    LinuxMissingSharedRust,
    /// Banned framework reference (React, Vue, Flutter, Electron,
    /// Cordova) detected on any non-fallback surface.
    BannedFrameworkReference,
    /// Manifest does not declare an OpenAPI 3.2.0 codegen recipe.
    OpenApiCodegenMissing,
    /// Superseded stack reference (ADR-0393): SolidJS/SolidStart appears in
    /// an active client manifest.
    SupersededStackReference,
}

impl fmt::Display for ClientStackViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): {:?} — {}",
            self.surface, self.manifest_path, self.kind, self.summary
        )
    }
}

pub fn validate_client_stack<I>(manifests: I) -> Result<ClientStackReport, ClientStackViolation>
where
    I: IntoIterator<Item = ClientManifest>,
{
    let (report, violations) = audit_all_violations(manifests);
    if let Some(first) = violations.into_iter().next() {
        Err(first)
    } else {
        Ok(report)
    }
}

pub fn audit_all_violations<I>(manifests: I) -> (ClientStackReport, Vec<ClientStackViolation>)
where
    I: IntoIterator<Item = ClientManifest>,
{
    let manifests: Vec<ClientManifest> = manifests.into_iter().collect();
    let mut violations = Vec::new();
    let mut surfaces: std::collections::BTreeSet<String> = Default::default();

    for manifest in &manifests {
        surfaces.insert(manifest.surface.clone());
        let lower = manifest.contents.to_ascii_lowercase();
        let surface = manifest.surface.as_str();

        // OpenAPI 3.2.0 codegen recipe required on every client.
        if !declares_openapi_codegen(&lower) {
            violations.push(ClientStackViolation {
                surface: surface.to_string(),
                manifest_path: manifest.path.clone(),
                kind: ViolationKind::OpenApiCodegenMissing,
                summary: "manifest does not declare an OpenAPI 3.2.0 codegen recipe (one of: \
                          openapi-typescript, progenitor, swift-openapi-generator, \
                          openapi-generator-kotlin, kiota)"
                    .to_string(),
            });
        }

        // Banned framework references on any client manifest.
        if declares_banned_framework(&lower) {
            violations.push(ClientStackViolation {
                surface: surface.to_string(),
                manifest_path: manifest.path.clone(),
                kind: ViolationKind::BannedFrameworkReference,
                summary: "manifest references a banned framework (React, Vue, Flutter, \
                          Electron, or Cordova); see ADR-0185 §Alternatives Considered"
                    .to_string(),
            });
        }

        // Supersession lint (ADR-0393): retired stacks must not reappear.
        if declares_superseded_solidjs(&lower) {
            violations.push(ClientStackViolation {
                surface: surface.to_string(),
                manifest_path: manifest.path.clone(),
                kind: ViolationKind::SupersededStackReference,
                summary: "manifest references SolidJS/SolidStart, which ADR-0393 superseded in \
                          full (Leptos is the canonical app-shell frontend); active client \
                          manifests must not carry retired-stack residue"
                    .to_string(),
            });
        }

        // Web stack discipline.
        if surface.starts_with("web") {
            let svelte = declares_sveltekit(&lower);
            let leptos = declares_leptos(&lower);
            if svelte && leptos {
                violations.push(ClientStackViolation {
                    surface: surface.to_string(),
                    manifest_path: manifest.path.clone(),
                    kind: ViolationKind::WebDualStackForbidden,
                    summary: "web surface declares BOTH SvelteKit AND Leptos; per ADR-0185 web \
                         stack lifecycle is sequential SvelteKit-now → Leptos-future, never both \
                         simultaneously for the same surface"
                        .to_string(),
                });
            } else if !svelte && !leptos {
                violations.push(ClientStackViolation {
                    surface: surface.to_string(),
                    manifest_path: manifest.path.clone(),
                    kind: ViolationKind::WebStackMissing,
                    summary: "web surface declares neither SvelteKit nor Leptos as its framework"
                        .to_string(),
                });
            }
        }

        // Apple discipline: SwiftUI only, no KMP klib imports.
        if surface.starts_with("apple")
            || surface.starts_with("ios")
            || surface.starts_with("macos")
            || surface.starts_with("watchos")
            || surface.starts_with("visionos")
        {
            if declares_kmp_klib_import(&lower) {
                violations.push(ClientStackViolation {
                    surface: surface.to_string(),
                    manifest_path: manifest.path.clone(),
                    kind: ViolationKind::AppleImportsKmp,
                    summary:
                        "Apple surface imports a KMP klib artifact; per ADR-0185 Apple targets are \
                         pure Swift + SwiftUI; KMP scope is Android-only"
                            .to_string(),
                });
            }
            if !declares_swiftui(&lower) {
                violations.push(ClientStackViolation {
                    surface: surface.to_string(),
                    manifest_path: manifest.path.clone(),
                    kind: ViolationKind::MobileNonCanonicalUi,
                    summary: "Apple surface does not declare SwiftUI as its UI framework"
                        .to_string(),
                });
            }
        }

        // Android discipline: Compose only.
        if surface.starts_with("android") && !declares_compose(&lower) {
            violations.push(ClientStackViolation {
                surface: surface.to_string(),
                manifest_path: manifest.path.clone(),
                kind: ViolationKind::MobileNonCanonicalUi,
                summary: "Android surface does not declare Jetpack Compose as its UI framework"
                    .to_string(),
            });
        }

        // Windows discipline: WinUI 3 only.
        if surface.starts_with("windows") && !declares_winui3(&lower) {
            violations.push(ClientStackViolation {
                surface: surface.to_string(),
                manifest_path: manifest.path.clone(),
                kind: ViolationKind::WindowsNonWinUi3,
                summary: "Windows surface does not declare WinUI 3 + .NET; per ADR-0185 \
                          Avalonia / MAUI / Electron are rejected"
                    .to_string(),
            });
        }

        // Linux discipline: GTK 4 + libadwaita; no Tauri/Electron/Qt/Flutter/Slint/Iced.
        if surface.starts_with("linux") {
            if !declares_gtk4_libadwaita(&lower) {
                violations.push(ClientStackViolation {
                    surface: surface.to_string(),
                    manifest_path: manifest.path.clone(),
                    kind: ViolationKind::LinuxNonGtk4,
                    summary: "Linux surface does not declare gtk4-rs + libadwaita; per ADR-0185 \
                              Tauri / Electron / Qt / Flutter / Slint / Iced are rejected for \
                              Linux desktop"
                        .to_string(),
                });
            }
            if declares_tauri_or_electron_or_qt(&lower) {
                violations.push(ClientStackViolation {
                    surface: surface.to_string(),
                    manifest_path: manifest.path.clone(),
                    kind: ViolationKind::LinuxNonGtk4,
                    summary: "Linux surface declares a rejected toolkit (Tauri / Electron / Qt / \
                         Flutter / Slint / Iced); per ADR-0185 use gtk4-rs + libadwaita"
                        .to_string(),
                });
            }
            if !declares_shared_rust_dep(&lower) {
                violations.push(ClientStackViolation {
                    surface: surface.to_string(),
                    manifest_path: manifest.path.clone(),
                    kind: ViolationKind::LinuxMissingSharedRust,
                    summary:
                        "Linux surface does not declare the `oya-client-shared-rust` workspace \
                         crate dependency required by ADR-0185"
                            .to_string(),
                });
            }
        }
    }

    let report = ClientStackReport {
        manifests_checked: manifests.len(),
        surfaces_audited: surfaces.len(),
    };
    (report, violations)
}

// ---- token detectors (substring; tolerant of JSON/YAML/TOML) ----

fn declares_openapi_codegen(lower: &str) -> bool {
    lower.contains("openapi-typescript")
        || lower.contains("progenitor")
        || lower.contains("swift-openapi-generator")
        || lower.contains("openapi-generator-kotlin")
        || lower.contains("kiota")
}

fn declares_banned_framework(lower: &str) -> bool {
    let has_react = lower.contains("react")
        && !lower.contains("react flow")
        && !lower.contains("rejected react")
        && !lower.contains("\"react\": false");
    let has_vue = lower.contains("\"vue\":") || lower.contains("nuxt");
    let has_flutter = lower.contains("flutter") && !lower.contains("rejected flutter");
    let has_electron = lower.contains("electron") && !lower.contains("rejected electron");
    let has_cordova = lower.contains("cordova") && !lower.contains("rejected cordova");
    has_react || has_vue || has_flutter || has_electron || has_cordova
}

fn declares_sveltekit(lower: &str) -> bool {
    lower.contains("sveltekit") || lower.contains("svelte 5") || lower.contains("@sveltejs")
}

fn declares_leptos(lower: &str) -> bool {
    lower.contains("\"leptos\"") || lower.contains("leptos = ") || lower.contains("leptos_axum")
}

fn declares_swiftui(lower: &str) -> bool {
    lower.contains("swiftui")
}

fn declares_compose(lower: &str) -> bool {
    lower.contains("jetpack compose")
        || lower.contains("androidx.compose")
        || lower.contains("\"compose\"")
}

fn declares_winui3(lower: &str) -> bool {
    lower.contains("winui 3")
        || lower.contains("winui3")
        || lower.contains("microsoft.windowsappsdk")
}

fn declares_gtk4_libadwaita(lower: &str) -> bool {
    (lower.contains("gtk4") || lower.contains("gtk-rs"))
        && (lower.contains("libadwaita") || lower.contains("adwaita"))
}

fn declares_tauri_or_electron_or_qt(lower: &str) -> bool {
    let banned = ["tauri", "qt6", "qt 6", "qt-rs", "slint", "iced = "];
    banned.iter().any(|t| lower.contains(t))
}

fn declares_shared_rust_dep(lower: &str) -> bool {
    lower.contains("oya-client-shared-rust") || lower.contains("oya_client_shared_rust")
}

fn declares_superseded_solidjs(lower: &str) -> bool {
    lower.lines().any(|line| {
        line.contains("solidjs")
            || line.contains("solidstart")
            || line.contains("solid-js")
            || line.contains("\"solid\"")
    })
}

fn declares_kmp_klib_import(lower: &str) -> bool {
    lower.contains(".klib")
        || lower.contains("kotlin-multiplatform")
        || lower.contains("kmp-shared")
        || lower.contains("shared-kotlin")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(surface: &str, contents: &str) -> ClientManifest {
        ClientManifest {
            surface: surface.to_string(),
            path: format!("microservices/workflow-studio/clients/{surface}/client-manifest.json"),
            contents: contents.to_string(),
        }
    }

    const VALID_WEB_SVELTEKIT: &str = r#"
{
  "surface": "web-sveltekit",
  "framework": "SvelteKit 2.55",
  "codegen": "openapi-typescript",
  "shared_layer": "packages/shared-ts"
}
"#;

    const VALID_APPLE_IOS: &str = r#"
{
  "surface": "apple-ios",
  "framework": "SwiftUI",
  "swift_version": "6.3",
  "codegen": "swift-openapi-generator"
}
"#;

    const VALID_ANDROID: &str = r#"
{
  "surface": "android",
  "framework": "Jetpack Compose",
  "kotlin_version": "2.3.0",
  "codegen": "openapi-generator-kotlin"
}
"#;

    const VALID_WINDOWS: &str = r#"
{
  "surface": "windows",
  "framework": "WinUI 3",
  "sdk": "Microsoft.WindowsAppSDK 1.8",
  "codegen": "kiota"
}
"#;

    const VALID_LINUX: &str = r#"
{
  "surface": "linux",
  "framework": "gtk4-rs + libadwaita 1.8",
  "rust_dependency": "oya-client-shared-rust",
  "codegen": "progenitor"
}
"#;

    #[test]
    fn passes_on_conformant_sveltekit_web() {
        let report = validate_client_stack(vec![mk("web-sveltekit", VALID_WEB_SVELTEKIT)])
            .expect("sveltekit web manifest must pass");
        assert_eq!(report.manifests_checked, 1);
    }

    #[test]
    fn passes_on_conformant_apple_ios() {
        let report = validate_client_stack(vec![mk("apple-ios", VALID_APPLE_IOS)])
            .expect("apple ios manifest must pass");
        assert_eq!(report.manifests_checked, 1);
    }

    #[test]
    fn passes_on_conformant_android() {
        validate_client_stack(vec![mk("android", VALID_ANDROID)])
            .expect("android manifest must pass");
    }

    #[test]
    fn passes_on_conformant_windows() {
        validate_client_stack(vec![mk("windows", VALID_WINDOWS)])
            .expect("windows manifest must pass");
    }

    #[test]
    fn passes_on_conformant_linux() {
        validate_client_stack(vec![mk("linux", VALID_LINUX)])
            .expect("linux gtk4 manifest must pass");
    }

    #[test]
    fn fails_when_web_declares_both_sveltekit_and_leptos() {
        let bad = r#"
{
  "surface": "web-dual",
  "framework_primary": "SvelteKit 2.55",
  "framework_secondary": "leptos = \"0.8\"",
  "codegen": "openapi-typescript"
}
"#;
        let err =
            validate_client_stack(vec![mk("web-dual", bad)]).expect_err("dual web stack must fail");
        assert_eq!(err.kind, ViolationKind::WebDualStackForbidden);
    }

    #[test]
    fn fails_when_apple_imports_kmp_klib() {
        let bad = r#"
{
  "surface": "apple-ios",
  "framework": "SwiftUI",
  "imports": [".klib shared-kotlin"],
  "codegen": "swift-openapi-generator"
}
"#;
        let err = validate_client_stack(vec![mk("apple-ios", bad)])
            .expect_err("apple KMP import must fail");
        assert_eq!(err.kind, ViolationKind::AppleImportsKmp);
    }

    #[test]
    fn fails_when_windows_uses_avalonia() {
        let bad = r#"
{
  "surface": "windows",
  "framework": "Avalonia 11",
  "codegen": "kiota"
}
"#;
        let err = validate_client_stack(vec![mk("windows", bad)])
            .expect_err("non-WinUI3 windows must fail");
        assert_eq!(err.kind, ViolationKind::WindowsNonWinUi3);
    }

    #[test]
    fn fails_when_linux_uses_tauri() {
        let bad = r#"
{
  "surface": "linux",
  "framework": "Tauri 2",
  "rust_dependency": "oya-client-shared-rust",
  "codegen": "progenitor"
}
"#;
        let err = validate_client_stack(vec![mk("linux", bad)]).expect_err("tauri linux must fail");
        // The validator emits LinuxNonGtk4 first (missing gtk4/libadwaita marker).
        assert_eq!(err.kind, ViolationKind::LinuxNonGtk4);
    }

    #[test]
    fn fails_when_react_declared() {
        let bad = r#"
{
  "surface": "web-react",
  "framework": "React 19",
  "codegen": "openapi-typescript"
}
"#;
        let err =
            validate_client_stack(vec![mk("web-react", bad)]).expect_err("react surface must fail");
        assert_eq!(err.kind, ViolationKind::BannedFrameworkReference);
    }

    #[test]
    fn fails_when_openapi_codegen_missing() {
        let bad = r#"
{
  "surface": "web-sveltekit",
  "framework": "SvelteKit 2.55",
  "api_client": "hand-rolled fetch wrapper"
}
"#;
        let err = validate_client_stack(vec![mk("web-sveltekit", bad)])
            .expect_err("missing codegen must fail");
        assert_eq!(err.kind, ViolationKind::OpenApiCodegenMissing);
    }

    #[test]
    fn fails_when_linux_missing_shared_rust_dep() {
        let bad = r#"
{
  "surface": "linux",
  "framework": "gtk4-rs 0.11.3 + libadwaita 1.8",
  "codegen": "progenitor"
}
"#;
        let err = validate_client_stack(vec![mk("linux", bad)])
            .expect_err("linux without shared-rust dep must fail");
        assert_eq!(err.kind, ViolationKind::LinuxMissingSharedRust);
    }

    #[test]
    fn fails_when_solidjs_declared_as_stack() {
        // RED fixture: ADR-0393 superseded SolidJS in full; declaring it as
        // the framework must violate on any surface.
        let manifest = mk(
            "web-app-shell",
            r#"{ "surface": "web-app-shell", "framework": "SolidStart", "stack": "solidjs", "codegen": "openapi-typescript" }"#,
        );
        let err = validate_client_stack([manifest]).unwrap_err();
        assert_eq!(err.kind, ViolationKind::SupersededStackReference);
        assert!(err.summary.contains("ADR-0393"));
    }

    #[test]
    fn audit_all_violations_returns_full_list_on_multi_failure() {
        let bad_web = r#"
{
  "surface": "web-react",
  "framework": "React 19"
}
"#;
        let bad_apple = r#"
{
  "surface": "apple-ios",
  "framework": "SwiftUI",
  "imports": [".klib shared"]
}
"#;
        let (report, violations) =
            audit_all_violations(vec![mk("web-react", bad_web), mk("apple-ios", bad_apple)]);
        assert_eq!(report.manifests_checked, 2);
        assert!(
            violations.len() >= 3,
            "expected >= 3, got {}",
            violations.len()
        );
        let kinds: std::collections::BTreeSet<_> = violations.iter().map(|v| v.kind).collect();
        assert!(kinds.contains(&ViolationKind::BannedFrameworkReference));
        assert!(kinds.contains(&ViolationKind::AppleImportsKmp));
    }
}
