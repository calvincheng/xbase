use anyhow::Result;

mod build;
mod drop;
mod register;
mod rename_file;
mod run;

pub use build::Build;
pub use drop::Drop;
pub use register::Register;
pub use rename_file::RenameFile;
pub use run::Run;

#[async_trait::async_trait]
#[cfg(feature = "daemon")]
pub trait DaemonCommandExt {
    async fn handle(&self, state: crate::SharedState) -> Result<()>;
}

#[derive(Debug)]
pub enum DaemonCommand {
    Build(Build),
    Run(Run),
    RenameFile(RenameFile),
    Register(Register),
    Drop(Drop),
}

impl DaemonCommand {
    #[cfg(feature = "daemon")]
    pub async fn handle(&self, state: crate::SharedState) -> Result<()> {
        use DaemonCommand::*;

        match self {
            Build(c) => c.handle(state).await,
            Run(c) => c.handle(state).await,
            RenameFile(c) => c.handle(state).await,
            Register(c) => c.handle(state).await,
            Drop(c) => c.handle(state).await,
        }
    }

    pub fn parse(str: &str) -> Result<Self> {
        let mut args = str.split(" ").collect::<Vec<&str>>();
        let cmd = args.remove(0);
        Ok(match cmd {
            "build" => Self::Build(Build::new(args)?),
            "run" => Self::Run(Run::new(args)?),
            "rename_file" => Self::RenameFile(RenameFile::new(args)?),
            "register" => Self::Register(Register::new(args)?),
            "drop" => Self::Drop(Drop::new(args)?),
            _ => anyhow::bail!("Unknown command messsage: {cmd}"),
        })
    }
}
