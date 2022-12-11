use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub type Passthrough = Option<Arc<dyn Any + Send + Sync>>;
type CmdHandlerReturn = Pin<Box<dyn Future<Output = Result<()>>>>;
type CmdHandler = fn(Vec<String>, Passthrough) -> CmdHandlerReturn;

#[derive(Debug)]
pub struct CommandError {
    msg: String,
}

impl Error for CommandError {}
impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.msg)
    }
}

pub struct AsyncCommandHandler {
    commands: HashMap<String, CmdHandler>,
}

impl AsyncCommandHandler {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn register(&mut self, identifier: impl ToString, handler: CmdHandler) {
        self.commands.insert(identifier.to_string(), handler);
    }

    pub fn get_commands(&self) -> impl Iterator<Item = (&String, &CmdHandler)> {
        self.commands.keys().zip(self.commands.values())
    }

    pub async fn call(
        &self,
        identifier: &str,
        args: Vec<String>,
        passthrough: Passthrough,
    ) -> Result<()> {
        let cmd = self
            .commands
            .get(identifier)
            .ok_or("Command not found!".to_owned())?;
        cmd(args, passthrough).await
    }

    pub async fn handle(&self, input: &str, passthrough: Passthrough) -> Result<()> {
        let mut splits = input.split_whitespace();
        let start_word = splits.next().ok_or("No command specified.".to_owned())?;
        let args = splits.map(|x| x.to_owned()).collect::<Vec<String>>();
        self.call(start_word, args, passthrough).await?;
        Ok(())
    }
}
