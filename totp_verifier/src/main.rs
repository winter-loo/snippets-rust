use hmac::{Hmac, Mac};
use sha1::Sha1;
use base32;
use chrono::Utc;
use std::env;

// 创建 HMAC-SHA1 类型别名
type HmacSha1 = Hmac<Sha1>;

fn verify_totp(secret_base32: &str, user_input_totp: &str) -> bool {
    // 解码 Base32 密钥
    let secret = match base32::decode(base32::Alphabet::RFC4648 { padding: false }, secret_base32) {
        Some(bytes) => bytes,
        None => {
            println!("密钥解码失败");
            return false;
        }
    };

    // 获取当前 Unix 时间戳（秒）
    let now = Utc::now().timestamp() as u64;
    let step = 30; // 步长 30 秒
    let tc = now / step;

    // 检查 ±1 步长窗口
    for offset in -1..=1 {
        let time_counter = (tc as i64 + offset) as u64;
        
        // 将时间计数器转为 8 字节大端字节序
        let time_bytes = time_counter.to_be_bytes();

        // 计算 HMAC-SHA1
        let mut mac = HmacSha1::new_from_slice(&secret).expect("HMAC 初始化失败");
        mac.update(&time_bytes);
        let hmac_result = mac.finalize().into_bytes();

        // 动态截断
        let offset = (hmac_result[hmac_result.len() - 1] & 0x0f) as usize;
        let truncated = u32::from_be_bytes([
            hmac_result[offset] & 0x7f, // 去掉符号位
            hmac_result[offset + 1],
            hmac_result[offset + 2],
            hmac_result[offset + 3],
        ]);
        let code = (truncated % 1_000_000).to_string(); // 6 位验证码
        let padded_code = format!("{:06}", code.parse::<u32>().unwrap()); // 补齐 6 位

        if padded_code == user_input_totp {
            return true;
        }
    }
    false
}

fn main() {
    // 管理员的 TOTP 密钥
    let admin_totp_secret = match std::env::var("TOTP_SECRET") {
        Ok(key) => key,
        Err(_) => panic!("need set TOTP_SECRET environment variable"),
    };

    // 从命令行参数获取用户输入的验证码
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("用法: cargo run <TOTP验证码>");
        return;
    }
    let user_input_totp = &args[1];

    // 验证
    if verify_totp(&admin_totp_secret, user_input_totp) {
        println!("验证通过，欢迎管理员！");
    } else {
        println!("MFA 验证码错误");
    }
}
