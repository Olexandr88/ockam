use clap::{Args, Subcommand};
use colorful::core::StrMarker;
use colorful::Colorful;
use serde_json::json;

pub(crate) use issue::IssueCommand;
use ockam::identity::models::{CredentialAndPurposeKey, CredentialSchemaIdentifier};
use ockam::identity::{Identifier, TimestampInSeconds};
use ockam_core::compat::collections::HashMap;
pub(crate) use store::StoreCommand;
pub(crate) use verify::VerifyCommand;

use crate::credential::list::ListCommand;
use crate::error::Error;
use crate::output::Output;
use crate::{CommandGlobalOpts, Result};

pub(crate) mod issue;
pub(crate) mod list;
pub(crate) mod store;
pub(crate) mod verify;

/// Manage Credentials
#[derive(Clone, Debug, Args)]
#[command(arg_required_else_help = true, subcommand_required = true)]
pub struct CredentialCommand {
    #[command(subcommand)]
    subcommand: CredentialSubcommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum CredentialSubcommand {
    #[command(display_order = 900)]
    List(ListCommand),
    Issue(IssueCommand),
    Store(StoreCommand),
    Verify(VerifyCommand),
}

impl CredentialCommand {
    pub fn run(self, options: CommandGlobalOpts) {
        match self.subcommand {
            CredentialSubcommand::List(c) => c.run(options),
            CredentialSubcommand::Issue(c) => c.run(options),
            CredentialSubcommand::Store(c) => c.run(options),
            CredentialSubcommand::Verify(c) => c.run(options),
        }
    }

    pub fn name(&self) -> String {
        match &self.subcommand {
            CredentialSubcommand::Issue(_) => "issue credential",
            CredentialSubcommand::List(_) => "list credentials",
            CredentialSubcommand::Store(_) => "store credential",
            CredentialSubcommand::Verify(_) => "verify credential",
        }
        .to_string()
    }
}

pub struct CredentialOutput {
    credential: String,
    subject: Identifier,
    issuer: Identifier,
    created_at: TimestampInSeconds,
    expires_at: TimestampInSeconds,
    is_verified: bool,
    schema: CredentialSchemaIdentifier,
    attributes: HashMap<String, String>,
}

impl CredentialOutput {
    pub fn from_credential(credential: CredentialAndPurposeKey, is_verified: bool) -> Result<Self> {
        let str = hex::encode(credential.encode_as_cbor_bytes()?);
        let credential_data = credential.credential.get_credential_data()?;
        let purpose_key_data = credential.purpose_key_attestation.get_attestation_data()?;

        let subject = credential_data.subject.ok_or(Error::InternalError {
            error_message: "credential subject is missing".to_str(),
            exit_code: 1,
        })?;

        let mut attributes = HashMap::<String, String>::default();
        for (k, v) in &credential_data.subject_attributes.map {
            match (
                String::from_utf8(k.as_slice().to_vec()),
                String::from_utf8(v.as_slice().to_vec()),
            ) {
                (Ok(k), Ok(v)) => _ = attributes.insert(k, v),
                _ => continue,
            }
        }

        let s = Self {
            credential: str,
            subject,
            issuer: purpose_key_data.subject,
            created_at: credential_data.created_at,
            expires_at: credential_data.expires_at,
            is_verified,
            schema: credential_data.subject_attributes.schema,
            attributes,
        };

        Ok(s)
    }
}

impl Output for CredentialOutput {
    fn output(&self) -> Result<String> {
        let is_verified = if self.is_verified {
            "✔︎".light_green()
        } else {
            "✕".light_red()
        };

        let attributes = json!(self.attributes).to_string();

        let output = format!(
            "Credential:\n\
            \tsubject:     {subject}\n\
            \tissuer:      {issuer}\n\
            \tis_verified: {is_verified}\n\
            \tcreated_at:  {created_at}\n\
            \texpires_at:  {expires_at}\n\
            \tschema:      {schema}\n\
            \tattributes:  {attributes}\n\
            \tbinary:      {credential}",
            subject = self.subject,
            issuer = self.issuer,
            is_verified = is_verified,
            created_at = self.created_at.0,
            expires_at = self.expires_at.0,
            schema = self.schema.0,
            attributes = attributes,
            credential = self.credential
        );

        Ok(output)
    }
}
