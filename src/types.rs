use jsonrpc_core::{Call, Output, Params};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Display;
use std::io::Error;
use toml;

pub enum Void {}

// Configuration

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub language: HashMap<String, LanguageConfig>,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub verbosity: u8,
    #[serde(default)]
    pub snippet_support: bool,
}

#[derive(Clone, Deserialize, Debug)]
pub struct ServerConfig {
    #[serde(default)]
    pub session: String,
    #[serde(default)]
    pub timeout: u64,
}

#[derive(Clone, Deserialize, Debug)]
pub struct LanguageConfig {
    pub filetypes: Vec<String>,
    pub roots: Vec<String>,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub initialization_options: Option<Value>,
    #[serde(default = "default_offset_encoding")]
    pub offset_encoding: OffsetEncoding,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            session: String::new(),
            timeout: 0,
        }
    }
}

fn default_offset_encoding() -> OffsetEncoding {
    OffsetEncoding::Utf16
}

// Editor

#[derive(Clone, Debug, Deserialize)]
pub struct EditorMeta {
    pub session: String,
    pub client: Option<String>,
    pub buffile: String,
    pub filetype: String,
    pub version: u64,
    pub fifo: Option<String>,
}

pub type EditorParams = toml::Value;

#[derive(Clone, Debug, Deserialize)]
pub struct EditorRequest {
    #[serde(flatten)]
    pub meta: EditorMeta,
    pub method: String,
    pub params: EditorParams,
}

#[derive(Deserialize)]
pub struct EditorResponse {
    pub meta: EditorMeta,
    pub command: String,
}

pub type SessionId = String;
pub type LanguageId = String;
pub type RootPath = String;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Route {
    pub session: SessionId,
    pub language: LanguageId,
    pub root: RootPath,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EditorCompletion {
    pub offset: u64,
}

#[derive(Deserialize, Debug)]
pub struct TextDocumentDidOpenParams {
    pub draft: String,
}

#[derive(Deserialize, Debug)]
pub struct TextDocumentDidChangeParams {
    pub draft: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextDocumentCompletionParams {
    pub position: KakounePosition,
    pub completion: EditorCompletion,
}

#[derive(Deserialize, Debug)]
pub struct PositionParams {
    pub position: KakounePosition,
}

#[derive(Deserialize, Debug)]
pub enum HoverPrecedence {
    #[serde(rename = "diagnostics_only")]
    DiagnosticsOnly,
    #[serde(rename = "info_only")]
    InfoOnly,
    #[serde(rename = "reverse")]
    DiagnosticsFirst,
    #[serde(rename = "default")]
    InfoFirst,
}

impl Default for HoverPrecedence {
    fn default() -> Self {
        HoverPrecedence::InfoFirst
    }
}

#[derive(Deserialize, Debug)]
pub struct HoverParams {
    pub position: KakounePosition,
    pub info_precedence: HoverPrecedence,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentRenameParams {
    pub position: KakounePosition,
    pub new_name: String,
}

#[derive(Deserialize, Debug)]
pub struct WindowProgress {
    pub title: String,
    pub message: Option<String>,
    pub percentage: Option<String>,
    pub done: Option<bool>,
}

// Language Server

// XXX serde(untagged) ?
#[derive(Debug)]
pub enum ServerMessage {
    Request(Call),
    Response(Output),
}

pub trait ToParams {
    fn to_params(self) -> Result<Params, Error>;
}

impl<T> ToParams for T
where
    T: Serialize,
{
    fn to_params(self) -> Result<Params, Error> {
        let json_value = serde_json::to_value(self)?;

        let params = match json_value {
            Value::Null => Params::None,
            Value::Bool(_) | Value::Number(_) | Value::String(_) => Params::Array(vec![json_value]),
            Value::Array(vec) => Params::Array(vec),
            Value::Object(map) => Params::Map(map),
        };

        Ok(params)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct KakounePosition {
    pub line: u64,
    pub column: u64, // in bytes, not chars!!!
}

#[derive(Debug, PartialEq)]
pub struct KakouneRange {
    pub start: KakounePosition,
    pub end: KakounePosition,
}

impl Display for KakounePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.line, self.column)
    }
}

impl Display for KakouneRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{},{}", self.start, self.end)
    }
}

/// Represents how language server interprets LSP's `Position.character`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OffsetEncoding {
    /// UTF-8 code units aka bytes
    #[serde(rename = "utf-8")]
    Utf8,
    /// UTF-16 code units
    #[serde(rename = "utf-16")]
    Utf16,
}
