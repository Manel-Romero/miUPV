//! Integration tests for miUPV services using mock HTTP servers.

use httpmock::prelude::*;
use miupv_core::client::MiupvClient;
use miupv_core::error::CoreError;
use miupv_core::services::{auth_service, cabinas_service, poliformat_service, scores_service};

#[tokio::test]
async fn test_login_success() {
    std::env::set_var("MIUPV_TOKEN_FILE", "/tmp/miupv_test_tokens.json");
    let server = MockServer::start();
    let client = MiupvClient::builder()
        .all_base_urls(server.base_url())
        .build();

    let test_user = "test_alumno_mock";
    let test_pass = "test_password_mock";
    let mock_access = "mock_access_token_123";
    let mock_refresh = "mock_refresh_token_789";
    let mock_ttl_access = 3600;
    let mock_ttl_refresh = 2592000;

    let oauth_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/oauth/tokens/")
            .header("accept", "application/json")
            .body_contains("grant_type=password")
            .body_contains(&format!("username={test_user}"))
            .body_contains(&format!("password={test_pass}"))
            .body_contains("client_id=miUpv")
            .body_contains("code_challenge_method=S256")
            .body_contains("code_verifier=")
            .body_contains("code_challenge=");
        then.status(200)
            .header("content-type", "application/json")
            .body(format!(
                r#"{{
                    "accessToken": "{mock_access}",
                    "refreshToken": "{mock_refresh}",
                    "ttlAccessToken": {mock_ttl_access},
                    "ttlRefreshToken": {mock_ttl_refresh}
                }}"#
            ));
    });

    let tokens = auth_service::login_with_client(&client, test_user, test_pass)
        .await
        .expect("Login should succeed");

    oauth_mock.assert();
    assert_eq!(tokens.access_token, mock_access);
    assert_eq!(tokens.refresh_token, mock_refresh);
    assert_eq!(tokens.ttl_access_token, Some(mock_ttl_access));
    assert!(tokens.is_valid());
}

#[tokio::test]
async fn test_login_wrong_password() {
    let server = MockServer::start();
    let client = MiupvClient::builder()
        .all_base_urls(server.base_url())
        .build();

    let test_user = "test_alumno_mock";
    let wrong_pass = "bad_password_mock";

    let oauth_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/oauth/tokens/")
            .body_contains(&format!("username={test_user}"))
            .body_contains(&format!("password={wrong_pass}"));
        then.status(401)
            .header("content-type", "application/json")
            .body(r#"{"error":"invalid_grant","error_description":"Bad credentials"}"#);
    });

    let result = auth_service::login_with_client(&client, test_user, wrong_pass).await;

    oauth_mock.assert();
    match result {
        Err(CoreError::Auth(msg)) => {
            assert!(
                msg.contains("401")
                    || msg.contains("Bad credentials")
                    || msg.contains("unauthorized")
            );
        }
        res => panic!("Expected Auth error, got: {:?}", res),
    }
}

#[tokio::test]
async fn test_intranet_login_success() {
    std::env::set_var("MIUPV_TOKEN_FILE", "/tmp/miupv_test_tokens.json");
    let server = MockServer::start();
    let client = MiupvClient::builder()
        .all_base_urls(server.base_url())
        .build();

    let test_user = "test_alumno_intranet";
    let test_pass = "test_pass_intranet";
    let execution_token = "mock_exec_token_abc";
    let test_ticket = "ST-12345";
    let mock_tdp = "D11D5A74C4AEA2BB132F9633701955A0";

    let cas_get_mock = server.mock(|when, then| {
        when.method(GET).path("/cas/login");
        then.status(200)
            .header("content-type", "text/html;charset=UTF-8")
            .header("set-cookie", "TGC=mock_tgc; Path=/cas/; Secure; HttpOnly")
            .body(format!(
                r#"<!DOCTYPE html>
                <html><body>
                <form id="fm1" action="login" method="post">
                    <input type="text" name="username" value="" />
                    <input type="password" name="password" value="" />
                    <input type="hidden" name="execution" value="{execution_token}" />
                    <input type="hidden" name="_eventId" value="submit" />
                </form>
                </body></html>"#
            ));
    });

    let cas_post_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/cas/login")
            .body_contains(&format!("username={test_user}"))
            .body_contains(&format!("password={test_pass}"))
            .body_contains(&format!("execution={execution_token}"))
            .body_contains("_eventId=submit");
        then.status(302).header(
            "Location",
            format!(
                "{}/pls/soalu/sic_intracas.app_intranet?P_CUA=miupv&ticket={test_ticket}",
                server.base_url()
            ),
        );
    });

    let bridge_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/pls/soalu/sic_intracas.app_intranet")
            .query_param("P_CUA", "miupv")
            .query_param("ticket", test_ticket);
        then.status(302)
            .header(
                "Set-Cookie",
                format!("TDp={mock_tdp};path=/;domain=.upv.es;secure;HttpOnly"),
            )
            .header(
                "Location",
                format!(
                    "{}/pls/soalu/sic_menu.MiUPV?P_IDIOMA=c&P_MODO=alumno",
                    server.base_url()
                ),
            );
    });

    let tokens = auth_service::login_intranet_with_client(&client, test_user, test_pass)
        .await
        .expect("Intranet login should succeed");

    cas_get_mock.assert();
    cas_post_mock.assert();
    bridge_mock.assert();

    assert_eq!(tokens.access_token, mock_tdp);
    assert!(tokens.is_intranet_session());
    assert!(tokens.is_valid());
}

#[tokio::test]
async fn test_intranet_login_wrong_password() {
    let server = MockServer::start();
    let client = MiupvClient::builder()
        .all_base_urls(server.base_url())
        .build();

    let test_user = "test_alumno_intranet";
    let wrong_pass = "bad_password_mock";
    let execution_token = "mock_exec_fail";

    let cas_get_mock = server.mock(|when, then| {
        when.method(GET).path("/cas/login");
        then.status(200)
            .header("content-type", "text/html;charset=UTF-8")
            .body(format!(
                r#"<!DOCTYPE html><html><body>
                <form id="fm1" action="login" method="post">
                    <input type="hidden" name="execution" value="{execution_token}" />
                    <input type="hidden" name="_eventId" value="submit" />
                </form>
                </body></html>"#
            ));
    });

    let cas_post_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/cas/login")
            .body_contains(&format!("username={test_user}"))
            .body_contains(&format!("password={wrong_pass}"));
        then.status(401).header("content-type", "text/html").body(
            "<html><body><div class='alert-danger'>Credenciales inválidas.</div></body></html>",
        );
    });

    let result = auth_service::login_intranet_with_client(&client, test_user, wrong_pass).await;

    cas_get_mock.assert();
    cas_post_mock.assert();

    match result {
        Err(CoreError::Auth(msg)) => {
            assert!(msg.contains("Credenciales UPV incorrectas"));
        }
        res => panic!("Expected Auth error, got: {:?}", res),
    }
}

#[tokio::test]
async fn test_refresh_token() {
    let server = MockServer::start();
    let client = MiupvClient::builder()
        .all_base_urls(server.base_url())
        .build();

    let old_refresh = "old_refresh_token_123";
    let new_access = "new_access_token_456";
    let new_refresh = "new_refresh_token_789";
    let ttl_acc = 7200;
    let ttl_ref = 2592000;

    let refresh_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/oauth/tokens/")
            .body_contains("grant_type=refresh_token")
            .body_contains(&format!("refresh_token={old_refresh}"))
            .body_contains("client_id=miUpv");
        then.status(200)
            .header("content-type", "application/json")
            .body(format!(
                r#"{{
                    "accessToken": "{new_access}",
                    "refreshToken": "{new_refresh}",
                    "ttlAccessToken": {ttl_acc},
                    "ttlRefreshToken": {ttl_ref}
                }}"#
            ));
    });

    let tokens = auth_service::refresh_with_client(&client, old_refresh)
        .await
        .expect("Refresh should succeed");

    refresh_mock.assert();
    assert_eq!(tokens.access_token, format!("{new_access}:UKQ1.240624.001"));
    assert_eq!(tokens.refresh_token, new_refresh);
    assert_eq!(tokens.ttl_access_token, Some(ttl_acc));
}

#[tokio::test]
async fn test_get_scores() {
    let server = MockServer::start();
    let client = MiupvClient::builder()
        .all_base_urls(server.base_url())
        .build();

    let scores_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/pls/sorest/sic_app_notas.notas_Alu")
            .query_param("p_curso", "2024")
            .query_param("p_idioma", "es")
            .header("authorization", "Bearer test_bearer_token");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"[
                    {
                        "codigo": "11542",
                        "nombre": "Arquitectura Software",
                        "convocatoria": "Ordinaria",
                        "nota": "9.5",
                        "calificacion": "Sobresaliente",
                        "creditos": 6.0,
                        "fecha": "2024-06-25"
                    },
                    {
                        "codigo": "11543",
                        "nombre": "Sistemas Inteligentes",
                        "convocatoria": "Ordinaria",
                        "nota": 8.0,
                        "calificacion": "Notable",
                        "creditos": 4.5,
                        "fecha": "2024-06-28"
                    }
                ]"#,
            );
    });

    let scores = scores_service::get_scores_with_client(&client, "test_bearer_token", "2024", "es")
        .await
        .expect("Scores retrieval should succeed");

    scores_mock.assert();
    assert_eq!(scores.len(), 2);
    assert_eq!(scores[0].codigo, "11542");
    assert_eq!(scores[0].nombre, "Arquitectura Software");
    assert_eq!(scores[0].nota, Some(9.5));
    assert_eq!(scores[0].calificacion.as_deref(), Some("Sobresaliente"));
    assert_eq!(scores[0].creditos, Some(6.0));
    assert_eq!(scores[1].codigo, "11543");
    assert_eq!(scores[1].nota, Some(8.0));
}

#[tokio::test]
async fn test_get_tasks() {
    let server = MockServer::start();
    let client = MiupvClient::builder()
        .all_base_urls(server.base_url())
        .build();

    let tasks_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/direct/assignment/my.json")
            .header("authorization", "Bearer test_bearer_token");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{
                    "assignment_collection": [
                        {
                            "id": "task-uuid-1234",
                            "title": "Entrega Laboratorio 3",
                            "siteTitle": "Programación Paralela",
                            "dueTimeString": "2025-04-10T23:59:00",
                            "instructions": "Subir el informe en formato PDF",
                            "status": "NOT_SUBMITTED",
                            "attachments": [
                                "https://poliformat.upv.es/access/content/attachment1.pdf"
                            ]
                        }
                    ]
                }"#,
            );
    });

    let tasks = poliformat_service::get_tasks_with_client(&client, "test_bearer_token")
        .await
        .expect("Tasks retrieval should succeed");

    tasks_mock.assert();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "task-uuid-1234");
    assert_eq!(tasks[0].title, "Entrega Laboratorio 3");
    assert_eq!(
        tasks[0].site_title.as_deref(),
        Some("Programación Paralela")
    );
    assert_eq!(tasks[0].status.as_deref(), Some("NOT_SUBMITTED"));
    assert_eq!(tasks[0].attachments.len(), 1);
}

#[tokio::test]
async fn test_get_cabinas() {
    let server = MockServer::start();
    let client = MiupvClient::builder()
        .all_base_urls(server.base_url())
        .build();

    let cabinas_mock = server.mock(|when, then| {
        when.method(GET)
            .path("/restreservas-1.0/franjas")
            .query_param("idEntidad", "105")
            .query_param("fecha", "2025-03-15")
            .query_param("sololibres", "S")
            .header("authorization", "Bearer test_bearer_token");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"[
                    {
                        "id": "franja-9901",
                        "nombre": "Cabina 5 (Planta 2)",
                        "biblioteca": "Biblioteca Central",
                        "fecha": "2025-03-15",
                        "hora_ini": "10:00",
                        "hora_fin": "12:00",
                        "libre": "S",
                        "max_personas": 6
                    }
                ]"#,
            );
    });

    let cabinas = cabinas_service::get_cabinas_disponibles_with_client(
        &client,
        "test_bearer_token",
        "2025-03-15",
    )
    .await
    .expect("Cabinas retrieval should succeed");

    cabinas_mock.assert();
    assert_eq!(cabinas.len(), 1);
    assert_eq!(cabinas[0].id, "franja-9901");
    assert_eq!(cabinas[0].nombre, "Cabina 5 (Planta 2)");
    assert_eq!(cabinas[0].hora_ini.as_deref(), Some("10:00"));
    assert!(cabinas[0].libre);
    assert_eq!(cabinas[0].max_personas, Some(6));
}
