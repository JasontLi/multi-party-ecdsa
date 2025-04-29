use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{post, routes, Route};
use std::process::{Command, Stdio};
use std::fs::File;
use std::io::Read;
use serde_json::Value;
use secp256k1::{PublicKey, Secp256k1};
use sha3::{Digest, Keccak256};
use hex;

#[derive(Deserialize)]
struct KeygenRequest {
    keygen_share_path: String, // e.g. "/usr/wallet/0x123456"
    room: String,              // e.g. "xxxxxxxx"
    address: String,           // e.g. "192.168.15.200"
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
    let y_sum_s: Vec<u8> = array.iter()
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
    let mut cmd =
        Command::new("./gg20_keygen");
    cmd.arg("--address")
        .arg(address_send)
        .arg("--output")
        .arg(share_file)
        .arg("--room")
        .arg(room)
        .arg("--threshold")
        .arg(threshold)
        .arg("--number-of-parties")
        .arg(number_of_parties)
        .arg("--index")
        .arg(index)
        .stdout(Stdio::piped());

    println!("cmd: {:?}", cmd);


    // 启动进程
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

    // 解析 stdout 为 UTF-8 字符串
    let sig = String::from_utf8(output.stdout)
        .map_err(|e| format!("invalid utf8 in CLI output: {}", e))?;
    Ok(sig)
}

#[post("/keygen", data = "<req>")]
async fn keygen(req: Json<KeygenRequest>) -> Result<Json<KeygenResponse>, String> {
    // 地址列表【上线后替换为真实 ip】
    let address_list = vec!["192.168.15.200", "192.168.15.201", "192.168.15.202"];
    let port = "33081";

    let file_sharding_suffix = vec!["1", "2", "3"];

    let threshold = "1";
    let number_of_parties = "3";

    let mut index = 0;
    let mut address_send = format!("http://{}:{}", &address_list[index], port);
    let mut share_file = format!("{}-{}.json", &req.keygen_share_path, &index.to_string());
    let mut index_suffix = file_sharding_suffix[index];

    // 调用 spawn_cli_process 来构造并启动第一个进程
    let sig1 = spawn_cli_process(
        &address_send,
        &share_file,
        &req.room,
        &index_suffix,
        &threshold,
        &number_of_parties,
    )
    .await?;

    index = 1;
    address_send = format!("http://{}：{}", &address_list[index], port);
    share_file = format!("{}-{}.json", &req.keygen_share_path, &index.to_string());
    index_suffix = file_sharding_suffix[index];

    // 重新设置参数并启动第二个进程
    let sig2 = spawn_cli_process(
        &address_send,
        &share_file,
        &req.room,
        &index_suffix,
        &threshold,
        &number_of_parties,
    )
    .await?;

    index = 2;
    address_send = format!("http://{}：{}", &address_list[index], port);
    share_file = format!("{}-{}.json", &req.keygen_share_path, &index.to_string());
    index_suffix = file_sharding_suffix[index];

    // 重新设置参数并启动第二个进程
    let sig2 = spawn_cli_process(
        &address_send,
        &share_file,
        &req.room,
        &index_suffix,
        &threshold,
        &number_of_parties,
    )
    .await?;


    let address = &req.address;
    let mut local_ip_idx= 0;
    for (i, addr) in address_list.iter().enumerate() {
        if addr.to_lowercase() == address.to_lowercase() {
            local_ip_idx = i;
        }
    }

    let local_share_file = format!("{}-{}.json", &req.keygen_share_path, &file_sharding_suffix[local_ip_idx].to_string());
    let address = parse_address_by_yum(&local_share_file);

    Ok(Json(KeygenResponse { address: address }))
}

pub fn mount_keygen_routes() -> Vec<Route> {
    routes![keygen]
}
