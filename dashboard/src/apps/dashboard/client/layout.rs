//! Dashboard shell layout with header, sidebar, and route outlet.

use reinhardt::pages::component;
use reinhardt::pages::event::ClickEvent;
use reinhardt::pages::page;
use reinhardt::pages::prelude::{Action, ClassToken, Outlet, Page, use_action};
use reinhardt::pages::server_fn::ServerFnError;

#[cfg(wasm)]
use crate::apps::auth::server_fn::logout::logout;
use crate::apps::dashboard::client::style::STYLES;
use crate::shared::client::routes::route_href;
use crate::shared::client::style::STYLES as SHARED_STYLES;

fn nav_item_class(is_active: bool) -> ClassToken {
	if is_active {
		STYLES.navigation_item_active()
	} else {
		STYLES.navigation_item()
	}
}

fn route_is_active(current_path: &str, route_href: &str) -> bool {
	current_path
		.split_once('?')
		.map_or(current_path, |(path, _)| path)
		== route_href
}

#[cfg(wasm)]
async fn end_dashboard_session() -> Result<bool, ServerFnError> {
	logout().await
}

#[cfg(not(wasm))]
async fn end_dashboard_session() -> Result<bool, ServerFnError> {
	Ok(true)
}

#[cfg(wasm)]
fn replace_document(location: &str) -> Result<(), ServerFnError> {
	let window = web_sys::window()
		.ok_or_else(|| ServerFnError::server(500, "Browser window is unavailable"))?;
	window.location().replace(location).map_err(|error| {
		ServerFnError::server(500, format!("Unable to leave dashboard: {error:?}"))
	})
}

#[cfg(not(wasm))]
fn replace_document(_location: &str) -> Result<(), ServerFnError> {
	Ok(())
}

/// Render the shared dashboard chrome around its active child route.
#[reinhardt::pages::layout("/", name = "dashboard:layout")]
pub fn dashboard_layout(outlet: Outlet) -> Page {
	let login_href = route_href("auth:login_page", "/login");
	let logout_action: Action<bool, ServerFnError> = use_action({
		let login_href = login_href.clone();
		move |_: ()| {
			let login_href = login_href.clone();
			async move {
				let logged_out = end_dashboard_session().await?;
				replace_document(&login_href)?;
				Ok(logged_out)
			}
		}
	});
	let current_path = reinhardt::pages::app::try_with_spa_router(|router| *router.current_path());
	let account_href = route_href("auth:account_page", "/account");
	let home_href = route_href("dashboard:home", "/");
	let clusters_href = route_href("clusters:list", "/clusters");
	let deployments_href = route_href("deployments:list", "/deployments");
	let github_href = route_href("github:repositories", "/github");

	page!({
		{
			let current_path = current_path
				.map(|path| path.get())
				.unwrap_or_else(|| "/".to_string());
			let outlet = outlet.clone();
			let account_href = account_href.clone();
			let home_href = home_href.clone();
			let clusters_href = clusters_href.clone();
			let deployments_href = deployments_href.clone();
			let github_href = github_href.clone();
			let logout_action = logout_action;
			page!({
				div {
					class: SHARED_STYLES.app() + STYLES.dashboard_app(),
					header {
						class: STYLES.dashboard_header(),
						div {
							class: STYLES.header_brand(),
							span {
								class: STYLES.brand_mark(),
								"RC"
							}
							div {
								span {
									class: STYLES.brand_name(),
									"Reinhardt Cloud"
								}
								span {
									class: STYLES.brand_subtitle(),
									"Deploy control"
								}
							}
						}
						div {
							class: STYLES.header_actions(),
							span {
								class: STYLES.header_health(),
								"Healthy"
							}
							a {
								href: account_href.clone(),
								class: SHARED_STYLES.link() + STYLES.header_action(),
								"Account"
							}
							button {
								type: "button",
								class: SHARED_STYLES.link() + STYLES.header_action(),
								@click: move |event: ClickEvent| {
									event.prevent_default();
									logout_action.dispatch(());
								},
								"Logout"
							}
						}
					}
					div {
						class: STYLES.dashboard_body(),
						nav {
							class: STYLES.sidebar(),
							div {
								class: STYLES.organization(),
								p {
									class: STYLES.organization_label(),
									"Organization"
								}
								p {
									class: STYLES.organization_name(),
									"current workspace"
								}
							}
							ul {
								class: STYLES.navigation_list(),
								li {
									a {
										href: home_href,
										class: self::nav_item_class(self::route_is_active(&current_path, &home_href)),
										"Overview"
									}
								}
								li {
									a {
										href: clusters_href,
										class: self::nav_item_class(self::route_is_active(&current_path, &clusters_href)),
										"Clusters"
									}
								}
								li {
									a {
										href: deployments_href,
										class: self::nav_item_class(self::route_is_active(&current_path, &deployments_href)),
										"Deployments"
									}
								}
								li {
									a {
										href: github_href,
										class: self::nav_item_class(self::route_is_active(&current_path, &github_href)),
										"GitHub"
									}
								}
								li {
									a {
										href: account_href,
										class: self::nav_item_class(self::route_is_active(&current_path, &account_href)),
										"Account"
									}
								}
							}
						}
						main {
							class: STYLES.dashboard_main(),
							{ outlet }
						}
					}
				}
			})
		}
	})
}

/// Render the main dashboard overview.
#[component("/", name = "dashboard:home")]
pub fn dashboard_shell() -> Page {
	let clusters_href = route_href("clusters:list", "/clusters");
	let deployments_href = route_href("deployments:list", "/deployments");
	let github_href = route_href("github:repositories", "/github");
	page!({
		div {
			class: SHARED_STYLES.shell(),
			div {
				class: SHARED_STYLES.topline(),
				div {
					p {
						class: SHARED_STYLES.kicker(),
						"Control plane"
					}
					h1 {
						class: SHARED_STYLES.title() + STYLES.overview_title(),
						"Deployment Operations"
					}
				}
				p {
					class: SHARED_STYLES.muted() + STYLES.overview_description(),
					"Live workspace for clusters, deployments, source imports, and account access."
				}
			}
			div {
				class: STYLES.overview_metrics(),
				div {
					class: SHARED_STYLES.panel_pad() + STYLES.metric_card() + STYLES.metric_clusters(),
					h3 {
						class: STYLES.metric_label(),
						"Clusters"
					}
					p {
						class: STYLES.metric_value(),
						"0"
					}
					p {
						class: STYLES.metric_detail(),
						"registered targets"
					}
				}
				div {
					class: SHARED_STYLES.panel_pad() + STYLES.metric_card() + STYLES.metric_deployments(),
					h3 {
						class: STYLES.metric_label(),
						"Deployments"
					}
					p {
						class: STYLES.metric_value(),
						"0"
					}
					p {
						class: STYLES.metric_detail(),
						"active releases"
					}
				}
				div {
					class: SHARED_STYLES.panel_pad() + STYLES.metric_card() + STYLES.metric_status(),
					h3 {
						class: STYLES.metric_label(),
						"System Status"
					}
					p {
						class: STYLES.healthy_status(),
						"Healthy"
					}
					p {
						class: STYLES.metric_detail(),
						"router and websocket ready"
					}
				}
			}
			div {
				class: STYLES.overview_panels(),
				section {
					class: SHARED_STYLES.panel(),
					div {
						class: SHARED_STYLES.panel_head(),
						"Runbook"
					}
					div {
						class: STYLES.runbook_list(),
						a {
							href: clusters_href.clone(),
							class: STYLES.runbook_link(),
							"Register cluster" span {
								class: STYLES.runbook_link_action(),
								"Open"
							}
						}
						a {
							href: deployments_href.clone(),
							class: STYLES.runbook_link(),
							"Create deployment" span {
								class: STYLES.runbook_link_action(),
								"Open"
							}
						}
						a {
							href: github_href.clone(),
							class: STYLES.runbook_link(),
							"Import repository" span {
								class: STYLES.runbook_link_action(),
								"Open"
							}
						}
					}
				}
				section {
					class: SHARED_STYLES.panel_pad() + STYLES.control_surface(),
					p {
						class: STYLES.control_surface_label(),
						"Control Surface"
					}
					p {
						class: STYLES.control_surface_title(),
						"Dogfood-ready"
					}
					p {
						class: STYLES.control_surface_description(),
						"Dashboard routes are rendered through the shared Reinhardt application shell."
					}
				}
			}
		}
	})
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	#[rstest]
	#[case::overview("/", "/", true)]
	#[case::account("/account", "/account", true)]
	#[case::clusters("/clusters", "/clusters", true)]
	#[case::deployment_logs("/deployments?logs=42", "/deployments", true)]
	#[case::different_route("/github", "/clusters", false)]
	fn route_active_state_matches_path(
		#[case] path: &str,
		#[case] route: &str,
		#[case] expected: bool,
	) {
		// Arrange
		let active = super::route_is_active(path, route);

		// Assert
		assert_eq!(active, expected);
	}

	#[rstest]
	fn dashboard_shell_renders_overview_links() {
		// Arrange
		let shell = super::dashboard_shell(super::DashboardShellProps {});

		// Act
		let html = shell.render_to_string();

		// Assert
		let hrefs = html
			.split("href=\"")
			.skip(1)
			.map(|fragment| fragment.split('"').next().unwrap_or_default())
			.collect::<Vec<_>>();
		assert_eq!(hrefs, vec!["/clusters", "/deployments", "/github"]);
		let metrics_classes = (super::SHARED_STYLES.panel_pad()
			+ super::STYLES.metric_card()
			+ super::STYLES.metric_clusters())
		.as_str()
		.to_owned();
		let expected_metrics = format!(
			"<div class=\"{}\"><div class=\"{metrics_classes}\">",
			super::STYLES.overview_metrics().as_str()
		);
		assert!(
			html.contains(&expected_metrics),
			"metrics grid owns the cluster metric card"
		);
		let expected_runbook_link = format!(
			"<a href=\"/clusters\" class=\"{}\">Register cluster",
			super::STYLES.runbook_link().as_str()
		);
		assert!(
			html.contains(&expected_runbook_link),
			"runbook link owns its generated token"
		);
	}

	#[test]
	fn nav_item_class_selects_the_exact_generated_token() {
		// Arrange + Act
		let active_class = super::nav_item_class(true);
		let inactive_class = super::nav_item_class(false);

		// Assert
		assert_eq!(
			active_class.as_str(),
			super::STYLES.navigation_item_active().as_str()
		);
		assert_eq!(
			inactive_class.as_str(),
			super::STYLES.navigation_item().as_str()
		);
	}
}
