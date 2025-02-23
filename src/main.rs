use anyhow::{bail, Context, Result};
use clap::{Parser, ValueHint};
use dialoguer::{Select, Confirm};
use directories::UserDirs;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    time::Duration,
};

/// Interactive mode for recursively selecting a namespace.
fn interactive_select_namespace() -> Result<String> {
    let base_dir = get_default_dir()?;
    let mut current = base_dir.clone();
    loop {
        let entries: Vec<PathBuf> = fs::read_dir(&current)?
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    if entry.file_type().ok()?.is_dir() {
                        Some(entry.path())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        if entries.is_empty() {
            break;
        }
        // 候補のディレクトリ名一覧（相対パス）
        let mut candidates: Vec<String> = entries.iter()
            .filter_map(|p| p.file_name().and_then(|os_str| os_str.to_str()).map(|s| s.to_string()))
            .collect();
        candidates.sort();
        let selection = Select::new()
            .with_prompt(format!("Select a namespace in {}", current.display()))
            .items(&candidates)
            .default(0)
            .interact()?;
        // 更新: 現在のディレクトリを選択したサブディレクトリに変更
        current = entries[selection].clone();
        // サブディレクトリがさらに存在するか確認
        let sub_entries: Vec<PathBuf> = fs::read_dir(&current)?
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    if entry.file_type().ok()?.is_dir() {
                        Some(entry.path())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        if sub_entries.is_empty() {
            break;
        }
        // ユーザーに、さらに深い階層を選択するか確認
        if !Confirm::new()
            .with_prompt("Do you want to select a subdirectory further?")
            .default(true)
            .interact()? {
            break;
        }
    }
    // 最終的な選択結果を、base_dir からの相対パスとして返す
    let rel = current.strip_prefix(&get_default_dir()?)
        .unwrap_or(&current)
        .to_string_lossy()
        .to_string();
    Ok(rel)
}

/// Returns the default configuration directory (e.g., ~/.ferrapi_tester).
fn get_default_dir() -> Result<PathBuf> {
    if let Some(user_dirs) = UserDirs::new() {
        Ok(user_dirs.home_dir().join(".ferrapi_tester"))
    } else {
        bail!("Could not determine user home directory")
    }
}

/// Constructs the configuration file path. Example: ~/.ferrapi_tester/SystemA/example/POST.json
fn get_config_path(base_dir: &PathBuf, target: &str, method: &str) -> PathBuf {
    let method_file = format!("{}.json", method.to_uppercase());
    base_dir.join(target).join(method_file)
}

/// Parses header strings in "Key: Value" format into a HashMap.
fn parse_headers(headers: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for header in headers {
        let parts: Vec<&str> = header.splitn(2, ':').collect();
        if parts.len() != 2 {
            bail!("Invalid header format: {}", header);
        }
        map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
    }
    Ok(map)
}

/// FerrAPI Tester - API testing CLI tool.
///
/// このツールは、HTTP リクエストの設定をコマンドラインで指定し、
/// 必要に応じて設定を保存・読み込みして API のテストを行います。
/// TARGET（名前空間）が指定されなければ、--url オプションのみで API を呼び出します。
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// HTTP メソッド (GET, POST, PUT, DELETE, etc.) [default: GET]
    #[arg(short = 'X', long = "request", default_value = "GET")]
    method: String,

    /// ヘッダーの指定（例: -H "Content-Type: application/json"）
    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,

    /// リクエストボディの文字列（-d または -v で指定）
    #[arg(short = 'd', long = "data")]
    data: Option<String>,

    /// JSON 形式でのリクエストボディ（-v を使う場合、保存済み設定とマージします）
    #[arg(short = 'v', long = "value", conflicts_with = "json")]
    value: Option<String>,

    /// JSON 形式でのリクエストボディ（-j を使う場合、-v と競合します）
    #[arg(short = 'j', long = "json", conflicts_with = "value")]
    json: Option<String>,

    /// リクエスト先の URL。この URL は保存する際にも使用されます。
    #[arg(short = 'u', long = "url")]
    url: Option<String>,

    /// タイムアウト秒数（デフォルトは 30 秒）
    #[arg(long = "timeout", default_value = "30")]
    timeout: u64,

    /// 現在のリクエスト設定を保存するフラグ
    #[arg(short = 's', long = "save")]
    save: bool,

    /// TARGET: 保存済み設定の名前空間パス（例: "SystemA/example"）。
    /// 省略された場合は、--url のみで API を呼び出します。
    /// ValueHint::DirPath により、シェルのネイティブ補完が働きます。
    #[arg(last = true, value_hint = ValueHint::DirPath)]
    target: Option<String>,

    /// TARGET（名前空間）に保存されている設定を削除するフラグ（ファイル単位）
    #[arg(long = "delete")]
    delete: bool,

    /// 指定した名前空間ディレクトリとその内容全体を削除するオプション。
    /// 例: `ferrapi_tester --delete-all SystemB` で ~/.ferrapi_tester/SystemB 以下全体を削除
    #[arg(long = "delete-all")]
    delete_all: bool,

    /// 対話モードで名前空間候補を表示して選択します。
    #[arg(long = "comp")]
    comp: bool,

    /// デフォルト設定ディレクトリを表示します。
    #[arg(long = "show-default-dir")]
    show_default_dir: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct RequestConfig {
    url: Option<String>,
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
    data: Option<Value>,
    timeout: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();

    // --show-default-dir が指定された場合、デフォルト設定ディレクトリを表示して終了
    if args.show_default_dir {
        let dir = get_default_dir()?;
        println!("Default configuration directory: {:?}", dir);
        return Ok(());
    }

    // --comp オプションが指定された場合、対話モードで名前空間を選択
    if args.comp {
        let selected = interactive_select_namespace()?;
        println!("Selected namespace: {}", selected);
        args.target = Some(selected);
    }

    // --delete-all オプションが指定された場合、TARGET に対応するディレクトリ全体を削除して終了
    if args.delete_all {
        if let Some(ref target) = args.target {
            let base_dir = get_default_dir()?;
            let namespace_dir = base_dir.join(target);
            if namespace_dir.exists() {
                fs::remove_dir_all(&namespace_dir)
                    .with_context(|| format!("Failed to delete namespace directory {:?}", namespace_dir))?;
                println!("Namespace directory {:?} deleted.", namespace_dir);
            } else {
                println!("No namespace directory found at {:?}", namespace_dir);
            }
            return Ok(());
        } else {
            bail!("--delete-all requires TARGET to be specified.");
        }
    }
    // --delete オプションが指定された場合（ファイル単位の削除）
    if args.delete {
        if let Some(ref target) = args.target {
            let base_dir = get_default_dir()?;
            let config_path = get_config_path(&base_dir, target, &args.method);
            if config_path.exists() {
                fs::remove_file(&config_path)
                    .with_context(|| format!("Failed to delete configuration at {:?}", config_path))?;
                println!("Configuration at {:?} deleted.", config_path);
            } else {
                println!("No configuration found at {:?}", config_path);
            }
            return Ok(());
        } else {
            bail!("--delete requires TARGET to be specified.");
        }
    }

    // 通常の API 呼び出しモード
    // TARGET が指定されている場合は保存／読み込みモード、指定がない場合は --url のみで実行
    let use_saved_config = args.target.is_some();
    let target_is_url = args
        .target
        .as_ref()
        .map(|t| t.starts_with("http"))
        .unwrap_or(false);
    let url_to_use = if let Some(url) = args.url {
        url
    } else if target_is_url {
        args.target.clone().unwrap_or_default()
    } else {
        String::new()
    };

    let mut config = if use_saved_config && !target_is_url {
        let base_dir = get_default_dir()?;
        let config_path = get_config_path(&base_dir, args.target.as_ref().unwrap(), &args.method);
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config from {:?}", config_path))?;
            serde_json::from_str::<RequestConfig>(&content)
                .with_context(|| "Failed to parse saved configuration")?
        } else {
            RequestConfig::default()
        }
    } else {
        RequestConfig::default()
    };

    config.method = Some(args.method.to_uppercase());
    if !url_to_use.is_empty() {
        config.url = Some(url_to_use);
    }
    let cli_headers = parse_headers(&args.headers)?;
    if let Some(ref mut saved_headers) = config.headers {
        saved_headers.extend(cli_headers);
    } else {
        config.headers = Some(cli_headers);
    }
    if let Some(ref val) = args.value {
        match serde_json::from_str::<Value>(val) {
            Ok(v) => config.data = Some(v),
            Err(_) => config.data = Some(json!(val)),
        }
    } else if let Some(ref j) = args.json {
        match serde_json::from_str::<Value>(j) {
            Ok(v) => config.data = Some(v),
            Err(_) => config.data = Some(json!(j)),
        }
    } else if let Some(ref data) = args.data {
        config.data = Some(json!(data));
    }
    config.timeout = Some(args.timeout);

    if args.save {
        if let Some(ref target) = args.target {
            let base_dir = get_default_dir()?;
            let config_path = get_config_path(&base_dir, target, &args.method);
            if let Some(parent) = config_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory {:?}", parent))?;
            }
            let serialized = serde_json::to_string_pretty(&config)
                .with_context(|| "Failed to serialize configuration")?;
            fs::write(&config_path, serialized)
                .with_context(|| format!("Failed to write configuration to {:?}", config_path))?;
            println!("Configuration saved to {:?}", config_path);
        } else {
            println!("--save is ignored because TARGET is not specified.");
        }
    }

    let url = config.url.as_ref().context("URL is not specified")?;
    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout.unwrap_or(30)))
        .build()?;
    let mut request_builder = match config.method.as_deref() {
        Some("GET") => client.get(url),
        Some("POST") => client.post(url),
        Some("PUT") => client.put(url),
        Some("DELETE") => client.delete(url),
        Some(other) => bail!("Unsupported HTTP method: {}", other),
        None => bail!("HTTP method is not specified"),
    };
    if let Some(headers) = config.headers {
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }
    }
    if let Some(data) = config.data {
        request_builder = request_builder.json(&data);
    }
    let response = request_builder.send().await?;
    let status = response.status();
    let text = response.text().await?;
    println!("Response Status: {}", status);
    println!("Response Body:\n{}", text);

    Ok(())
}
