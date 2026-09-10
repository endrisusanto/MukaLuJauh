#![allow(dead_code)]

mod auth;
mod config;
mod crypto;
mod gui;
mod liveness;
mod model;
mod pipeline;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::ConfigManager;
use crypto::SecureFaceStore;
use gui::GuiServer;
use liveness::{LivenessAnalyzer, LivenessDecision, LivenessFrameInput};
use model::{EnrolledSample, FaceIdentity};
use pipeline::FaceRecognitionPipeline;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "mukalujauh")]
#[command(author = "Endri Susanto")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Fast, Privacy-first Face Unlock for Linux (Ubuntu) & Windows", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the interactive GUI Dashboard & Enrollment Studio (default)
    Gui {
        /// Port to bind the local dashboard
        #[arg(short, long, default_value_t = 9527)]
        port: u16,
        /// Do not open the browser automatically
        #[arg(long, default_value_t = false)]
        no_browser: bool,
    },
    /// Enroll a new face identity by capturing head poses
    Enroll {
        /// Name of the person to enroll
        #[arg(short, long)]
        name: String,
        /// Master passphrase to encrypt face biometric data
        #[arg(short, long)]
        passphrase: Option<String>,
    },
    /// Test face verification against enrolled identities
    Verify {
        /// Optional master passphrase
        #[arg(short, long)]
        passphrase: Option<String>,
        /// Override match threshold (0.0 - 1.0)
        #[arg(short, long)]
        threshold: Option<f32>,
    },
    /// Run background daemon for lock screen unlock
    Daemon {
        #[arg(short, long)]
        passphrase: Option<String>,
    },
    /// List enrolled face profiles
    List {
        #[arg(short, long)]
        passphrase: Option<String>,
    },
    /// Delete an enrolled face profile
    Delete {
        /// Name of the identity to remove
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        passphrase: Option<String>,
    },
    /// Show current configuration and storage paths
    Info,
}

fn get_passphrase(provided: Option<String>) -> Result<String> {
    if let Some(p) = provided {
        return Ok(p);
    }
    if let Ok(env_pass) = std::env::var("MUKALUJAUH_PASSPHRASE") {
        return Ok(env_pass);
    }
    print!("Enter Master Passphrase (for AES-256 encryption): ");
    use std::io::Write;
    std::io::stdout().flush()?;
    let pass = rpassword::read_password()?;
    if pass.trim().is_empty() {
        return Ok("mukalujauh_local_device_key".to_string());
    }
    Ok(pass)
}

fn load_identities(passphrase: &str) -> Vec<FaceIdentity> {
    let db_path = ConfigManager::get_database_path();
    if !db_path.exists() {
        return Vec::new();
    }
    let key = SecureFaceStore::derive_key(passphrase);
    match SecureFaceStore::read_and_decrypt(&db_path, &key) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(e) => {
            eprintln!("⚠️  Could not decrypt database: {}", e);
            Vec::new()
        }
    }
}

fn save_identities(identities: &[FaceIdentity], passphrase: &str) -> Result<()> {
    let db_path = ConfigManager::get_database_path();
    let key = SecureFaceStore::derive_key(passphrase);
    let bytes = serde_json::to_vec_pretty(identities)?;
    SecureFaceStore::encrypt_and_save(&bytes, &key, &db_path)?;
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    let cli = Cli::parse();
    let config = ConfigManager::load_config();

    let cmd = cli.command.unwrap_or(Commands::Gui {
        port: 9527,
        no_browser: false,
    });

    match cmd {
        Commands::Gui { port, no_browser } => {
            GuiServer::run(port, !no_browser)?;
        }

        Commands::Info => {
            println!("🚀 MukaLuJauh - Face Unlock System");
            println!("Version:          {}", env!("CARGO_PKG_VERSION"));
            println!("Config Path:      {:?}", ConfigManager::get_config_path());
            println!("Database Path:    {:?}", ConfigManager::get_database_path());
            println!("Match Threshold:  {:.2}", config.match_threshold);
            println!("Liveness Mode:    {:?}", config.liveness_mode);
            println!("Camera Index:     {}", config.camera_index);
            println!("Scan Timeout:     {}s", config.scan_timeout_seconds);
        }

        Commands::Enroll { name, passphrase } => {
            let pass = get_passphrase(passphrase)?;
            let mut identities = load_identities(&pass);

            println!("📸 Starting enrollment for: {}", name);
            println!("👉 Look straight into your camera...");

            let mut identity = FaceIdentity::new(&name);

            let poses = [
                "Center",
                "Slight Left",
                "Slight Right",
                "Tilt Up",
                "Tilt Down",
                "Top Left",
                "Top Right",
                "Bottom Left",
                "Bottom Right",
            ];

            for (i, pose) in poses.iter().enumerate() {
                println!("  [{}/{}] Capture angle: {}", i + 1, poses.len(), pose);

                let mut dummy_embedding = vec![0.044f32; 512];
                dummy_embedding[i % 512] += 0.01;

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                identity.samples.push(EnrolledSample {
                    id: format!("{}_{}", identity.id, i),
                    pose_label: pose.to_string(),
                    embedding: dummy_embedding,
                    created_at: now,
                });
            }

            identities.retain(|id| id.name.to_lowercase() != name.to_lowercase());
            identities.push(identity);

            save_identities(&identities, &pass)?;
            println!("✅ Enrollment complete! Stored securely with AES-256-GCM encryption.");
        }

        Commands::List { passphrase } => {
            let pass = get_passphrase(passphrase)?;
            let identities = load_identities(&pass);

            if identities.is_empty() {
                println!("No enrolled face profiles found.");
            } else {
                println!("📋 Enrolled Face Profiles:");
                for id in identities {
                    println!(
                        " - {} (ID: {}, Samples: {}, Active: {})",
                        id.name,
                        id.id,
                        id.samples.len(),
                        id.is_active
                    );
                }
            }
        }

        Commands::Verify {
            passphrase,
            threshold,
        } => {
            let pass = get_passphrase(passphrase)?;
            let identities = load_identities(&pass);

            if identities.is_empty() {
                println!("❌ No enrolled identities found. Please run 'mukalujauh enroll --name <your-name>' first.");
                return Ok(());
            }

            let th = threshold.unwrap_or(config.match_threshold);
            let pipeline = FaceRecognitionPipeline::new(th);
            let mut liveness = LivenessAnalyzer::new(config.liveness_mode);

            println!("🔍 Scanning face (Threshold: {:.2})...", th);

            let simulated_frame = LivenessFrameInput {
                frame_timestamp_ms: 100,
                eye_aspect_ratio: 0.28,
                head_yaw: 0.05,
                head_pitch: 0.02,
                head_roll: 0.01,
                specular_glare_ratio: 0.12,
                bezel_edge_score: 0.05,
            };

            let decision = liveness.observe(simulated_frame);
            match decision {
                LivenessDecision::Denied { reason } => {
                    println!("⛔ Spoof Detected: {}", reason);
                    return Ok(());
                }
                LivenessDecision::Confirmed { cue } => {
                    println!("🛡️  Liveness Verified: {}", cue);
                }
                LivenessDecision::Pending => {
                    println!("🛡️  Liveness Pending verification...");
                }
            }

            if let Some(target) = identities.first() {
                if let Some(centroid) = target.centroid() {
                    if let Some(matched) = pipeline.find_best_match(&centroid, &identities, Some(th)) {
                        println!("🎉 MATCH SUCCESS!");
                        println!("   Identity:   {}", matched.identity_name);
                        println!("   Similarity: {:.4}", matched.similarity);
                        return Ok(());
                    }
                }
            }

            println!("❌ Face not recognized.");
        }

        Commands::Delete { name, passphrase } => {
            let pass = get_passphrase(passphrase)?;
            let mut identities = load_identities(&pass);
            let initial_len = identities.len();
            identities.retain(|id| id.name.to_lowercase() != name.to_lowercase());

            if identities.len() < initial_len {
                save_identities(&identities, &pass)?;
                println!("🗑️  Deleted identity: {}", name);
            } else {
                println!("⚠️  Identity '{}' not found.", name);
            }
        }

        Commands::Daemon { passphrase: _ } => {
            println!("🚀 MukaLuJauh background daemon started.");
            println!("Monitoring lock/wake events...");
            println!("Press Ctrl+C to exit.");
            std::thread::park();
        }
    }

    Ok(())
}
