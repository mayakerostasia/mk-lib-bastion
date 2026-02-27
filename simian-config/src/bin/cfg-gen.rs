use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;

// Import from parent crate
use simian_config::types::*;

#[derive(Parser, Debug)]
#[command(name = "cfg-gen")]
#[command(version = "2.0.0")]
#[command(about = "Generate Simian configuration files")]
pub struct CfgGen {
    #[command(subcommand)]
    pub subcmd: SubCmd,

    /// Output file path
    #[arg(short, long, default_value = ".env.json")]
    output: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum SubCmd {
    /// Generate SurrealDB configuration
    Surreal(SurrealArgs),
    
    #[cfg(feature = "autotask")]
    /// Generate Autotask configuration
    Autotask(AutotaskArgs),
    
    #[cfg(feature = "jira")]
    /// Generate Jira configuration
    Jira(JiraArgs),
    
    #[cfg(feature = "swimlane")]
    /// Generate Swimlane configuration
    Swimlane(SwimlaneArgs),
}

#[derive(Args, Debug)]
pub struct SurrealArgs {
    #[arg(long)]
    path: String,
    #[arg(long)]
    ns: String,
    #[arg(long)]
    db: String,
    #[arg(long)]
    user: String,
    #[arg(long)]
    pass: String,
    #[arg(long)]
    auth_level: String,
}

#[cfg(feature = "autotask")]
#[derive(Args, Debug)]
pub struct AutotaskArgs {
    #[arg(long)]
    url: String,
    #[arg(long)]
    integration_code: String,
    #[arg(long)]
    username: String,
    #[arg(long)]
    password: String,
}

#[cfg(feature = "jira")]
#[derive(Args, Debug)]
pub struct JiraArgs {
    #[arg(long)]
    url: String,
    #[arg(long)]
    user: String,
    #[arg(long)]
    api_key: String,
}

#[cfg(feature = "swimlane")]
#[derive(Args, Debug)]
pub struct SwimlaneArgs {
    #[arg(long)]
    host: String,
    #[arg(long)]
    pat: String,
    #[arg(long)]
    user: String,
    #[arg(long)]
    pass: String,
}

fn write_json<T: Serialize>(
    cfg: T,
    output_file: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(&cfg)?;
    std::fs::write(&output_file, json)?;
    eprintln!("✓ Configuration written to: {}", output_file.display());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CfgGen::parse();
    
    match args.subcmd {
        SubCmd::Surreal(surreal_args) => {
            let cfg = SurrealCfg {
                path: surreal_args.path,
                ns: surreal_args.ns,
                db: surreal_args.db,
                user: surreal_args.user,
                pass: surreal_args.pass,
                auth_level: surreal_args.auth_level,
            };
            write_json(cfg, args.output)?;
        }
        
        #[cfg(feature = "autotask")]
        SubCmd::Autotask(autotask_args) => {
            let cfg = AutotaskCfg {
                url: autotask_args.url.as_str(),
                integration_code: autotask_args.integration_code.as_str(),
                username: autotask_args.username.as_str(),
                password: autotask_args.password.as_str(),
            };
            write_json(cfg, args.output)?;
        }
        
        #[cfg(feature = "jira")]
        SubCmd::Jira(jira_args) => {
            let cfg = JiraCfg {
                url: jira_args.url.as_str(),
                user: jira_args.user.as_str(),
                api_key: jira_args.api_key.as_str(),
            };
            write_json(cfg, args.output)?;
        }
        
        #[cfg(feature = "swimlane")]
        SubCmd::Swimlane(swimlane_args) => {
            let cfg = SwimlaneCfg {
                host: swimlane_args.host,
                pat: swimlane_args.pat,
                user: swimlane_args.user,
                pass: swimlane_args.pass,
            };
            write_json(cfg, args.output)?;
        }
    }
    
    Ok(())
}
