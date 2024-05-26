use clap::{self, Args, Parser, Subcommand};
use serde::Serialize;
use std::path::PathBuf;
#[allow(unused)]
use bb_lib_config::{configuration::Configuration, AutotaskCfg, JiraCfg, SurrealCfg, SwimlaneCfg};

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
pub struct CfgCreator {
    #[command(subcommand)]
    pub subcmd: SubCmd,
}

#[derive(Subcommand, Debug)]
pub enum SubCmd {
    Surreal(SurrealArgs),
    Autotask(AutotaskArgs),
    Jira(JiraArgs),
    Swimlane(SwimlaneArgs),
}

#[derive(Args, Debug)]
pub struct SurrealArgs {
    #[clap(long)]
    uri: String,
    #[clap(long)]
    ns: String,
    #[clap(long)]
    db: String,
    #[clap(long)]
    user: String,
    #[clap(long)]
    pass: String,
    #[clap(long)]
    auth_level: String,
}

#[derive(Args, Debug)]
pub struct AutotaskArgs {
    #[clap(long)]
    url: String,
    #[clap(long)]
    user: String,
    #[clap(long)]
    api_key: String,
}

#[derive(Args, Debug)]
pub struct JiraArgs {
    #[clap(long)]
    url: String,
    #[clap(long)]
    user: String,
    #[clap(long)]
    api_key: String,
}

#[derive(Args, Debug)]
pub struct SwimlaneArgs {
    #[clap(long)]
    url: String,
    #[clap(long)]
    user: String,
    #[clap(long)]
    api_key: String,
}

// fn write_yaml<T: Configuration + Serialize>(
//     cfg: T,
//     output_file: PathBuf,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let yaml = serde_yaml::to_string(&cfg)?;
//     std::fs::write(output_file, yaml)?;
//     Ok(())
// }

fn write_json<T: Configuration + Serialize>(
    cfg: T,
    output_file: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let env = serde_json::to_string(&cfg)?;
    std::fs::write(output_file, env)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = CfgCreator::parse();
    match cfg.subcmd {
        SubCmd::Surreal(args) => {
            println!("Surreal -> \n {:#?}", args);
            let cfg = SurrealCfg {
                path: args.uri,
                ns: args.ns,
                db: args.db,
                user: args.user,
                pass: args.pass,
                auth_level: args.auth_level,
            };

            let _written = write_json(cfg, ".env.json".into())?;
            // let _written = write_yaml(cfg, ".env".into())?;
        }
        SubCmd::Autotask(args) => {
            println!("Autotask {:#?}", args);
            // let _written = write_yaml(args, ".env".into())?;
        }
        SubCmd::Jira(args) => {
            println!("Jira {:#?}", args);
            // let _written = write_yaml(args, ".env".into())?;
        }
        SubCmd::Swimlane(args) => {
            println!("Swimlane {:#?}", args);
            // let _written = write_yaml(args, ".env".into())?;
        }
    };
    Ok(())
}
