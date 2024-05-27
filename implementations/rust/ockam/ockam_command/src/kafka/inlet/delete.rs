use clap::Args;
use colorful::Colorful;
use ockam_api::fmt_ok;

use ockam_api::nodes::{models, BackgroundNodeClient};
use ockam_core::api::Request;
use ockam_node::Context;

use crate::util::async_cmd;
use crate::{docs, node::NodeOpts, CommandGlobalOpts};

const AFTER_LONG_HELP: &str = include_str!("./static/delete/after_long_help.txt");

/// Delete a Kafka Inlet
#[derive(Clone, Debug, Args)]
#[command(arg_required_else_help = true, after_long_help = docs::after_help(AFTER_LONG_HELP))]
pub struct DeleteCommand {
    #[command(flatten)]
    pub node_opts: NodeOpts,

    /// Kafka inlet service address
    pub address: String,
}

impl DeleteCommand {
    pub fn run(self, opts: CommandGlobalOpts) -> miette::Result<()> {
        async_cmd(&self.name(), opts.clone(), |ctx| async move {
            self.async_run(&ctx, opts).await
        })
    }

    pub fn name(&self) -> String {
        "delete kafka inlet".into()
    }

    async fn async_run(&self, ctx: &Context, opts: CommandGlobalOpts) -> miette::Result<()> {
        let node = BackgroundNodeClient::create(ctx, &opts.state, &self.node_opts.at_node).await?;
        let req = Request::delete("/node/services/kafka_inlet").body(
            models::services::DeleteServiceRequest::new(self.address.clone()),
        );
        node.tell(ctx, req).await?;

        opts.terminal
            .stdout()
            .plain(fmt_ok!(
                "Kafka inlet with address `{}` successfully deleted",
                self.address
            ))
            .write_line()?;

        Ok(())
    }
}