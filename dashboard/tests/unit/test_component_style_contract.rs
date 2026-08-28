//! Source-level contracts for Dashboard's generated component stylesheet.

use std::{
	fs,
	path::{Path, PathBuf},
};

const INDEX_HTML: &str = include_str!("../../index.html");
const AUTH_STYLE_SOURCE: &str = include_str!("../../src/apps/auth/client/style.rs");
const DASHBOARD_CLIENT_SOURCE: &str = include_str!("../../src/apps/dashboard/client.rs");
const DASHBOARD_LAYOUT_SOURCE: &str = include_str!("../../src/apps/dashboard/client/layout.rs");
const DASHBOARD_STYLE_SOURCE: &str = include_str!("../../src/apps/dashboard/client/style.rs");
const CLUSTERS_CLIENT_SOURCE: &str = include_str!("../../src/apps/clusters/client.rs");
const CLUSTERS_LIST_SOURCE: &str = include_str!("../../src/apps/clusters/client/pages/list.rs");
const CLUSTERS_STYLE_SOURCE: &str = include_str!("../../src/apps/clusters/client/style.rs");
const SHARED_IMPERATIVE_SOURCES: &[(&str, &str)] = &[
	(
		"entity_select",
		include_str!("../../src/shared/client/components/entity_select.rs"),
	),
	(
		"status_badge",
		include_str!("../../src/shared/client/components/status_badge.rs"),
	),
	(
		"toast",
		include_str!("../../src/shared/client/components/toast.rs"),
	),
	("websocket", include_str!("../../src/shared/client/ws.rs")),
];

const AUTH_PRODUCTION_SOURCES: &[(&str, &str)] = &[
	(
		"auth_layout",
		include_str!("../../src/apps/auth/client/components/auth_layout.rs"),
	),
	(
		"oauth_buttons",
		include_str!("../../src/apps/auth/client/components/oauth_buttons.rs"),
	),
	(
		"account",
		include_str!("../../src/apps/auth/client/pages/account.rs"),
	),
	(
		"login",
		include_str!("../../src/apps/auth/client/pages/login.rs"),
	),
	(
		"register",
		include_str!("../../src/apps/auth/client/pages/register.rs"),
	),
];

fn source_style_violations(source: &str) -> Vec<&'static str> {
	let mut violations = Vec::new();
	let lowercase_source = source.to_ascii_lowercase();

	for framework in ["unocss", "tailwind"] {
		if lowercase_source.contains(framework) {
			violations.push(match framework {
				"unocss" => "unocss reference",
				"tailwind" => "tailwind reference",
				_ => unreachable!("the framework list is fixed"),
			});
		}
	}

	if source.contains("class: \"") {
		violations.push("raw page class literal");
	}
	if source.contains("set_attribute(\"class\", \"") {
		violations.push("imperative utility class literal");
	}
	if source.match_indices("class=\"").any(|(index, _)| {
		source[index + "class=\"".len()..]
			.chars()
			.next()
			.is_some_and(|character| character != '{')
	}) {
		violations.push("raw HTML utility class literal");
	}

	violations
}

fn production_client_sources() -> Vec<PathBuf> {
	let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
	let mut paths = Vec::new();
	collect_rust_sources(&source_root, &mut paths);

	paths
		.into_iter()
		.filter(|path| is_production_client_source(path, &source_root))
		.collect()
}

fn collect_rust_sources(directory: &Path, paths: &mut Vec<PathBuf>) {
	let mut entries = fs::read_dir(directory)
		.unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
		.map(|entry| entry.expect("source directory entry should be readable"))
		.collect::<Vec<_>>();
	entries.sort_by_key(|entry| entry.path());

	for entry in entries {
		let path = entry.path();
		if path.is_dir() {
			collect_rust_sources(&path, paths);
		} else if path.extension().is_some_and(|extension| extension == "rs") {
			paths.push(path);
		}
	}
}

fn is_production_client_source(path: &Path, source_root: &Path) -> bool {
	path.file_name().is_some_and(|name| name == "client.rs")
		|| path
			.strip_prefix(source_root)
			.expect("source path must remain under src")
			.components()
			.any(|component| component.as_os_str() == "client")
}

#[test]
fn source_gate_rejects_forbidden_style_inputs() {
	// Arrange
	let source = r##"
		page!({ div { class: "p-4" } });
		element.set_attribute("class", "flex");
		element.set_inner_html(r#"<div class="gap-4"></div>"#);
		let framework = "TaIlWiNd";
	"##;

	// Act
	let violations = source_style_violations(source);

	// Assert
	assert_eq!(
		violations,
		vec![
			"tailwind reference",
			"raw page class literal",
			"imperative utility class literal",
			"raw HTML utility class literal",
		]
	);
}

#[test]
fn production_client_sources_use_generated_style_tokens() {
	// Arrange
	let sources = production_client_sources();

	// Act
	let diagnostics = sources
		.iter()
		.flat_map(|path| {
			let source = fs::read_to_string(path)
				.unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
			source_style_violations(&source)
				.into_iter()
				.map(move |violation| format!("{}: {violation}", path.display()))
		})
		.collect::<Vec<_>>();

	// Assert
	assert!(
		!sources.is_empty(),
		"the Dashboard must contain client Rust modules"
	);
	assert!(
		diagnostics.is_empty(),
		"production client sources must use generated ClassToken/ClassList accessors:\n{}",
		diagnostics.join("\n")
	);
}

#[test]
fn generated_component_stylesheet_is_the_only_document_style_runtime() {
	// Arrange
	let document = INDEX_HTML.to_ascii_lowercase();

	// Act
	let unocss_references = document.matches("unocss").count();
	let component_stylesheet_links = document.matches("__reinhardt__/components.css").count();
	let has_component_stylesheet_link = document.contains(
		r#"<link rel="stylesheet" href='{{ static_url("__reinhardt__/components.css") }}'>"#,
	);

	// Assert
	assert_eq!(unocss_references, 0);
	assert_eq!(component_stylesheet_links, 1);
	assert!(
		has_component_stylesheet_link,
		"the generated component stylesheet path must be a stylesheet link"
	);
}

#[test]
fn shared_imperative_dom_paths_do_not_embed_utility_class_literals() {
	// Arrange
	let utility_class_literals = [
		"\"bg-",
		"\"border-",
		"\"fixed ",
		"\"flex ",
		"\"gap-",
		"\"items-",
		"\"justify-",
		"\"max-w-",
		"\"min-w-",
		"\"p-",
		"\"px-",
		"\"py-",
		"\"rounded-",
		"\"shadow-",
		"\"text-",
		"\"top-",
		"\"right-",
		"\"z-",
	];

	// Act + Assert
	for (source_name, source) in SHARED_IMPERATIVE_SOURCES {
		for utility_class_literal in utility_class_literals {
			assert_eq!(
				source.matches(utility_class_literal).count(),
				0,
				"{source_name} must use typed shared style tokens"
			);
		}
	}
}

#[test]
fn auth_pages_and_components_use_typed_style_tokens() {
	// Arrange + Act + Assert
	for (source_name, source) in AUTH_PRODUCTION_SOURCES {
		assert!(
			!source.contains("class: \""),
			"{source_name} must use typed shared or auth-local style tokens"
		);
	}
}

#[test]
fn auth_account_grid_retains_the_desktop_two_column_rule() {
	// Arrange + Act
	let has_desktop_breakpoint = AUTH_STYLE_SOURCE.contains("@media (min-width: 1024px)");
	let has_two_columns = AUTH_STYLE_SOURCE
		.contains("grid-template-columns: unchecked_fn!(repeat(2, minmax(0, 1fr)));");

	// Assert
	assert!(
		has_desktop_breakpoint,
		"account layout must define a desktop breakpoint"
	);
	assert!(
		has_two_columns,
		"account layout must restore two desktop columns"
	);
}

#[test]
fn auth_actions_share_one_spacing_token() {
	// Arrange + Act
	let has_shared_spacing =
		AUTH_STYLE_SOURCE.contains(".account_action_spacing {\n\t\tmargin-top: 1.25rem;\n\t}");
	let has_duplicate_spacing = AUTH_STYLE_SOURCE.contains(".account_actions {")
		|| AUTH_STYLE_SOURCE.contains(".account_error_action {");

	// Assert
	assert!(
		has_shared_spacing,
		"auth actions must share one spacing token"
	);
	assert!(
		!has_duplicate_spacing,
		"auth actions must not define duplicate spacing tokens"
	);
}

#[test]
fn dashboard_shell_uses_typed_shared_and_dashboard_style_tokens() {
	// Arrange
	let dashboard_sources = [DASHBOARD_CLIENT_SOURCE, DASHBOARD_LAYOUT_SOURCE];

	// Act + Assert
	for source in dashboard_sources {
		assert!(
			!source.contains("class: \""),
			"dashboard client sources must use typed shared or dashboard-local style tokens"
		);
	}
	assert!(
		DASHBOARD_CLIENT_SOURCE.contains("pub mod style;"),
		"dashboard client module must export its local style module"
	);
}

#[test]
fn dashboard_overview_panels_preserve_the_desktop_asymmetric_ratio() {
	// Arrange + Act
	let has_desktop_ratio = DASHBOARD_STYLE_SOURCE
		.contains("@media (min-width: 1024px) {\n\t\t\tgrid-template-columns: (1.2fr, 0.8fr);");

	// Assert
	assert!(
		has_desktop_ratio,
		"desktop overview panels must retain the 1.2fr to 0.8fr ratio"
	);
}

#[test]
fn clusters_page_uses_typed_shared_and_cluster_style_tokens() {
	// Arrange
	let cluster_sources = [CLUSTERS_LIST_SOURCE];

	// Act + Assert
	for source in cluster_sources {
		assert!(
			!source.contains("class: \""),
			"cluster client sources must use typed shared or cluster-local style tokens"
		);
	}
	assert!(
		CLUSTERS_CLIENT_SOURCE.contains("pub mod style;"),
		"clusters client module must export its local style module"
	);
	assert!(
		CLUSTERS_STYLE_SOURCE.contains("pub static STYLES: ClustersStyles = style!"),
		"clusters must expose a unique generated style collection"
	);
	assert!(
		CLUSTERS_STYLE_SOURCE.contains("word-break: break-all;"),
		"one-time cluster tokens must break unbroken values before overflowing"
	);
	assert!(
		CLUSTERS_LIST_SOURCE.contains("class: STYLES.page_layout()"),
		"cluster page must render its generated responsive layout token"
	);
	assert!(
		CLUSTERS_LIST_SOURCE.contains("class: STYLES.token_value()"),
		"cluster token display must render its generated token token"
	);
	assert!(
		CLUSTERS_LIST_SOURCE.contains("STYLES.cluster_badge() + self::cluster_badge_state"),
		"cluster inventory badges must compose generated base and state tokens"
	);
	assert!(
		CLUSTERS_LIST_SOURCE.contains("class: SHARED_STYLES.shell(),\n\t\t\tdiv {\n\t\t\t\tdiv {"),
		"the page shell must not add a second outer content stack"
	);
	assert!(
		CLUSTERS_LIST_SOURCE.contains("class: SHARED_STYLES.muted() + STYLES.intro()"),
		"the cluster introduction must retain its local top margin"
	);
}
