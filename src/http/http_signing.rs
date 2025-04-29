use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{post, routes, Route};
use std::process::{Command, Stdio};

#[derive(Deserialize)]
struct SigningRequest {
    keygen_share_path: String, // e.g. "/usr/wallet/0x123456"
    data_to_sign: String,      // hex or utf8
    parties: Vec<u16>,         // e.g. [1,2]
    address: String,           // e.g. "ip"
    room: String,              // e.g. "xxxxxxxx"
}

#[derive(Serialize)]
struct SigningResponse {
    sign_data: String,
}

async fn spawn_cli_process(
    share_file: &str,
    data_to_sign: &str,
    parties: &str,
    address_send: &str,
    room: &str,
) -> Result<String, String> {
    // 构造命令
    let mut cmd =
        Command::new("./gg20_signing");
    cmd.arg("--local_share")
        .arg(share_file)
        .arg("--data-to-sign")
        .arg(data_to_sign)
        .arg("--parties")
        .arg(parties)
        .arg("--address")
        .arg(address_send)
        .arg("--room")
        .arg(room)
        .stdout(Stdio::piped());

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

#[post("/signing", data = "<req>")]
async fn signing(req: Json<SigningRequest>) -> Result<Json<SigningResponse>, String> {
    // 地址列表【上线后替换为真实 ip】
    let address_list = vec!["192.168.15.200", "192.168.15.201", "192.168.15.202"];
    let port = "33081";

    let shard_file_sharding = vec!["0", "1", "2"];

    let address = &req.address;
    let mut select_list = Vec::new();

    // 循环遍历地址列表
    for (i, addr) in address_list.iter().enumerate() {
        if addr.to_lowercase() != address.to_lowercase() {
            select_list.push(i);
        }
    }

    let parties = select_list[0].to_string() + "," + &select_list[1].to_string();
    let mut index = select_list[0];
    let mut address_send = format!("http://{}:{}", &address_list[index], port);
    let mut share_file = format!("{}-{}.json", &req.keygen_share_path, &index.to_string());

    // 调用 spawn_cli_process 来构造并启动第一个进程
    let sig1 = spawn_cli_process(
        &share_file,
        &req.data_to_sign,
        &parties,
        &address_send,
        &req.room,
    )
    .await?;

    index = select_list[1];
    address_send = format!("http://{}:{}", &address_list[index], port);
    share_file = format!("{}-{}.json", &req.keygen_share_path, &index.to_string());

    // 重新设置参数并启动第二个进程
    let sig2 = spawn_cli_process(
        &share_file,
        &req.data_to_sign,
        &parties,
        &address_send,
        &req.room,
    )
    .await?;

    // 返回给客户端，假设返回 sig1 作为签名数据
    Ok(Json(SigningResponse { sign_data: sig1 }))
}

pub fn mount_signing_routes() -> Vec<Route> {
    routes![signing]
}
