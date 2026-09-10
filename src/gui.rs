use crate::config::{AppConfig, ConfigManager};
use crate::crypto::SecureFaceStore;
use crate::model::{EnrolledSample, FaceIdentity};
use crate::pipeline::FaceRecognitionPipeline;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tao::{
    dpi::LogicalSize,
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use tiny_http::{Header, Response, Server};
use wry::WebViewBuilder;

#[derive(Serialize, Deserialize)]
struct StatusResponse {
    version: &'static str,
    config: AppConfig,
    identities: Vec<FaceIdentitySummary>,
}

#[derive(Serialize, Deserialize)]
struct FaceIdentitySummary {
    id: String,
    name: String,
    sample_count: usize,
    is_active: bool,
    created_at: u64,
}

#[derive(Deserialize)]
struct EnrollRequest {
    name: String,
    passphrase: Option<String>,
    samples: Vec<SampleInput>,
}

#[derive(Deserialize)]
struct SampleInput {
    pose_label: String,
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct VerifyRequest {
    passphrase: Option<String>,
    embedding: Vec<f32>,
    threshold: Option<f32>,
}

#[derive(Serialize)]
struct VerifyResponse {
    matched: bool,
    identity_name: Option<String>,
    similarity: f32,
    threshold: f32,
    message: String,
}

#[derive(Deserialize)]
struct DeleteRequest {
    name: String,
    passphrase: Option<String>,
}

pub struct GuiServer;

impl GuiServer {
    /// Launches the native standalone Desktop Window (Tauri/wry engine)
    pub fn run_desktop_window(port: u16) -> Result<()> {
        let addr = format!("127.0.0.1:{}", port);
        let server = Server::http(&addr)
            .map_err(|e| anyhow::anyhow!("Failed to bind internal API server to {}: {}", addr, e))?;

        let url = format!("http://{}", addr);

        // Spawn backend HTTP server in background thread
        std::thread::spawn(move || {
            let config_state = Arc::new(Mutex::new(ConfigManager::load_config()));

            for mut request in server.incoming_requests() {
                let path = request.url().to_string();
                let method = request.method().to_string();

                if method == "GET" && path == "/" {
                    let html = include_str!("../ui/index.html");
                    let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap();
                    let response = Response::from_string(html).with_header(header);
                    let _ = request.respond(response);
                    continue;
                }

                if method == "GET" && path == "/api/status" {
                    let config = config_state.lock().unwrap().clone();
                    let identities = Self::load_summary_list();
                    let status = StatusResponse {
                        version: env!("CARGO_PKG_VERSION"),
                        config,
                        identities,
                    };
                    let json = serde_json::to_string(&status).unwrap_or_default();
                    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                    let _ = request.respond(Response::from_string(json).with_header(header));
                    continue;
                }

                if method == "POST" && path == "/api/enroll" {
                    let mut body = String::new();
                    let _ = request.as_reader().read_to_string(&mut body);
                    if let Ok(req) = serde_json::from_str::<EnrollRequest>(&body) {
                        let passphrase = req.passphrase.unwrap_or_else(|| "mukalujauh_local_device_key".to_string());
                        let mut identities = Self::load_full_identities(&passphrase);

                        let mut new_identity = FaceIdentity::new(&req.name);
                        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

                        for (i, s) in req.samples.into_iter().enumerate() {
                            new_identity.samples.push(EnrolledSample {
                                id: format!("{}_{}", new_identity.id, i),
                                pose_label: s.pose_label,
                                embedding: s.embedding,
                                created_at: now,
                            });
                        }

                        identities.retain(|id| id.name.to_lowercase() != req.name.to_lowercase());
                        identities.push(new_identity);

                        let _ = Self::save_full_identities(&identities, &passphrase);

                        let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                        let _ = request.respond(Response::from_string(r#"{"success":true}"#).with_header(header));
                        continue;
                    }
                }

                if method == "POST" && path == "/api/verify" {
                    let mut body = String::new();
                    let _ = request.as_reader().read_to_string(&mut body);
                    if let Ok(req) = serde_json::from_str::<VerifyRequest>(&body) {
                        let passphrase = req.passphrase.unwrap_or_else(|| "mukalujauh_local_device_key".to_string());
                        let identities = Self::load_full_identities(&passphrase);
                        let cfg = config_state.lock().unwrap().clone();
                        let threshold = req.threshold.unwrap_or(cfg.match_threshold);

                        let pipeline = FaceRecognitionPipeline::new(threshold);
                        let matched = pipeline.find_best_match(&req.embedding, &identities, Some(threshold));

                        let resp = if let Some(m) = matched {
                            VerifyResponse {
                                matched: true,
                                identity_name: Some(m.identity_name.clone()),
                                similarity: m.similarity,
                                threshold,
                                message: format!("Recognized as {}", m.identity_name),
                            }
                        } else {
                            VerifyResponse {
                                matched: false,
                                identity_name: None,
                                similarity: 0.0,
                                threshold,
                                message: "Face not recognized".to_string(),
                            }
                        };

                        let json = serde_json::to_string(&resp).unwrap_or_default();
                        let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                        let _ = request.respond(Response::from_string(json).with_header(header));
                        continue;
                    }
                }

                if method == "POST" && path == "/api/config" {
                    let mut body = String::new();
                    let _ = request.as_reader().read_to_string(&mut body);
                    if let Ok(new_cfg) = serde_json::from_str::<AppConfig>(&body) {
                        let _ = ConfigManager::save_config(&new_cfg);
                        *config_state.lock().unwrap() = new_cfg;
                        let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                        let _ = request.respond(Response::from_string(r#"{"success":true}"#).with_header(header));
                        continue;
                    }
                }

                if method == "POST" && path == "/api/delete" {
                    let mut body = String::new();
                    let _ = request.as_reader().read_to_string(&mut body);
                    if let Ok(req) = serde_json::from_str::<DeleteRequest>(&body) {
                        let passphrase = req.passphrase.unwrap_or_else(|| "mukalujauh_local_device_key".to_string());
                        let mut identities = Self::load_full_identities(&passphrase);
                        identities.retain(|id| id.name.to_lowercase() != req.name.to_lowercase());
                        let _ = Self::save_full_identities(&identities, &passphrase);
                        let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
                        let _ = request.respond(Response::from_string(r#"{"success":true}"#).with_header(header));
                        continue;
                    }
                }

                let _ = request.respond(Response::from_string("Not Found").with_status_code(404));
            }
        });

        // Initialize Native Desktop Event Loop and Window
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("MukaLuJauh Studio - Face Unlock")
            .with_inner_size(LogicalSize::new(1080.0, 780.0))
            .with_resizable(true)
            .build(&event_loop)
            .map_err(|e| anyhow::anyhow!("Failed to create native desktop window: {}", e))?;

        // Initialize Native WebView inside the Desktop Window
        let _webview = WebViewBuilder::new()
            .with_url(&url)
            .build(&window)
            .map_err(|e| anyhow::anyhow!("Failed to build WebView: {}", e))?;

        println!("✨ Native Desktop Window opened successfully!");

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => {},
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });
    }

    fn load_summary_list() -> Vec<FaceIdentitySummary> {
        let full = Self::load_full_identities("mukalujauh_local_device_key");
        full.into_iter()
            .map(|id| FaceIdentitySummary {
                id: id.id,
                name: id.name,
                sample_count: id.samples.len(),
                is_active: id.is_active,
                created_at: id.created_at,
            })
            .collect()
    }

    fn load_full_identities(passphrase: &str) -> Vec<FaceIdentity> {
        let db_path = ConfigManager::get_database_path();
        if !db_path.exists() {
            return Vec::new();
        }
        let key = SecureFaceStore::derive_key(passphrase);
        match SecureFaceStore::read_and_decrypt(&db_path, &key) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    fn save_full_identities(identities: &[FaceIdentity], passphrase: &str) -> Result<()> {
        let db_path = ConfigManager::get_database_path();
        let key = SecureFaceStore::derive_key(passphrase);
        let bytes = serde_json::to_vec_pretty(identities)?;
        SecureFaceStore::encrypt_and_save(&bytes, &key, &db_path)?;
        Ok(())
    }
}
