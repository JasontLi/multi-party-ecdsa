use hex;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{post, routes, Route};
use secp256k1::{PublicKey, Secp256k1};
use serde_json::Value;
use sha3::{Digest, Keccak256};
use std::fs::File;
use std::io::Read;
use std::process::{Command, Stdio};

#[derive(Deserialize)]
struct KeygenRequest {
    keygen_share_path: String, // e.g. "/usr/wallet/0x123456"
    room: String,              // e.g. "xxxxxxxx"
    address: String,           // e.g. "192.168.15.200"
    index: String,
}

#[derive(Serialize)]
struct KeygenResponse {
    address: String,
}

fn parse_address_by_yum(json_file_path: &str) -> String {
    // 尝试读取 JSON 文件内容
    let mut file = match File::open(json_file_path) {
        Ok(f) => f,
        Err(_) => return String::new(), // 文件打开失败，返回空字符串
    };
    let mut json_content = String::new();
    if file.read_to_string(&mut json_content).is_err() {
        return String::new(); // 读取文件失败，返回空字符串
    }

    // 尝试解析 JSON 内容
    let json_object: Value = match serde_json::from_str(&json_content) {
        Ok(v) => v,
        Err(_) => return String::new(), // JSON 解析失败，返回空字符串
    };

    // 尝试获取 y_sum_s 的字节数组
    let array = match json_object["y_sum_s"]["point"].as_array() {
        Some(arr) => arr,
        None => return String::new(), // 获取失败，返回空字符串
    };

    // 将 JSON 数组转换为字节数组
    let y_sum_s: Vec<u8> = array
        .iter()
        .filter_map(|v| v.as_u64().map(|n| n as u8)) // 过滤并转换为 u8
        .collect();

    // 输出结果
    compute_eth_address(&y_sum_s)
}

fn compute_eth_address(compressed_pubkey: &[u8]) -> String {
    let secp = Secp256k1::new();
    let public_key = match PublicKey::from_slice(compressed_pubkey) {
        Ok(pk) => pk,
        Err(_) => return String::new(), // 公钥解析失败，返回空字符串
    };
    let uncompressed = public_key.serialize_uncompressed();
    let public_key_bytes = &uncompressed[1..];
    let mut hasher = Keccak256::new();
    hasher.update(public_key_bytes);
    let hash = hasher.finalize();
    let mut address = [0u8; 20];
    address.copy_from_slice(&hash[12..]);
    format!("0x{}", hex::encode(address))
}

async fn spawn_cli_process(
    address_send: &str,
    share_file: &str,
    room: &str,
    index: &str,
    threshold: &str,
    number_of_parties: &str,
) -> Result<String, String> {
    // 构造命令

    let command = vec![
        "./gg20_keygen",
        "--address",
        address_send,
        "--output",
        share_file,
        "--room",
        room,
        "--threshold",
        threshold,
        "--number-of-parties",
        number_of_parties,
        "--index",
        index,
    ];

    println!("command: {}", command.join(" "));

    // 启动进程
    let mut cmd = Command::new(command[0]);
    cmd.args(&command[1..]).stdout(Stdio::piped());

    let child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn CLI: {}", e))?;

    // 等待进程结束并获取输出
    let output = child
        .wait_with_output()
        .map_err(|e| format!("waiting CLI process failed: {}", e))?;

    if !output.status.success() {
        return Err(format!("CLI process exited with error: {}", output.status));
    }

    // 如果没有输出，返回一个合适的提示
    if output.stdout.is_empty() {
        return Ok("CLI executed successfully, but no output was returned.".to_string());
    }

    // 解析 stdout 为 UTF-8 字符串
    let sig = String::from_utf8(output.stdout)
        .map_err(|e| format!("invalid utf8 in CLI output: {}", e))?;
    Ok(sig)
}

#[post("/keygen", data = "<req>")]
async fn keygen(req: Json<KeygenRequest>) -> Result<Json<KeygenResponse>, String> {
    let mut address_send = format!("http://{}:{}", &req.address, "33081");
    let mut share_file = format!("{}-{}.json", &req.keygen_share_path, &req.index);

    spawn_cli_process(&address_send, &share_file, &req.room, &req.index, "2", "3").await?;

    Ok(Json(KeygenResponse { address: address }))
}

#[post("/start_keygen")]
async fn keygen(req: Json<KeygenRequest>) -> Result<Json<KeygenResponse>, String> {
    let ips = vec![
        "192.168.15.201", // 替换为实际的 IP 地址
        "192.168.15.202",
        "192.168.15.203",
    ];

    // 创建异步客户端
    let client = Client::new();

    // 请求的顺序是同步的，依次请求每个 IP 地址
    let result = request_to_ip(&client, format!("http://{}:33081", &ips[0]), build_request_data(req, "1"))
        .await
        .and_then(|_| request_to_ip(&client, format!("http://{}:33081", &ips[1]), build_request_data(req, "2")).await)
        .and_then(|_| request_to_ip(&client, format!("http://{}:33081", &ips[2]), build_request_data(req, "3")).await);

    match result {
        Ok(_) => Ok("All requests were successful.".to_string()),
        Err(e) => Err(format!("Error occurred: {}", e)),
    }
}


fn build_request_data(req: &Json<KeygenRequest>, index: &str) -> KeygenRequest {
    KeygenRequest {
        keygen_share_path: req.keygen_share_path.clone(),
        room: req.room.clone(),
        address: req.address.clone(),
        index: index.to_string(),
    }
}


// 发送 POST 请求到指定的 IP 地址
async fn request_to_ip(client: &Client, ip: &str, data: &KeygenRequest) -> Result<(), String> {
    let response = client
        .post(ip)
        .json(data)  // 使用 serde 自动将结构体转换为 JSON
        .send()
        .await
        .map_err(|e| format!("Failed to send request to {}: {}", ip, e))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Request to {} failed with status: {}", ip, response.status()))
    }
}

pub fn mount_keygen_routes() -> Vec<Route> {
    routes![keygen]
}
