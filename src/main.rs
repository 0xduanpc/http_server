use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

mod thread_pool;
use thread_pool::ThreadPool;

fn handle_connection(mut stream: TcpStream) {
    // 创建一个 1024 字节的缓冲区来读取请求数据
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // 将读取到的请求转换为字符串并打印出来（为了方便调试）
    let request = String::from_utf8_lossy(&buffer[..]);
    println!("Received request: {}", request);

    let (status_line, filename) = match parse_path(&buffer).as_str() {
        "/" => ("HTTP/1.1 200 OK\r\n\r\n", "static/hello.html"),
        "/about" => ("HTTP/1.1 200 OK\r\n\r\n", "static/about.html"),
        _ => ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "static/404.html"),
    };

    // 打开文件并读取内容
    let mut file = File::open(filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // 将状态行与文件内容一起发送回客户端
    let response = format!("{}{}", status_line, contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

/// 从 HTTP 请求中解析路径
fn parse_path(buffer: &[u8]) -> String {
    // 将请求转换为字符串
    let request = String::from_utf8_lossy(buffer);

    // 简单地从请求头中提取路径
    if let Some(path_start) = request.find("GET ") {
        if let Some(path_end) = request[path_start..].find(" HTTP/1.1") {
            let path = &request[path_start + 4..path_end];
            return path.to_string();
        }
    }

    "/".to_string()
}

fn main() {
    // 在本地 7878 端口上绑定 TCP 监听器
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server running on http://127.0.0.1:7878");

    // 创建一个线程池，大小为 4
    let pool = ThreadPool::new(4);

    // 逐个接受连接
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        println!("Connection established!");

        // 将连接处理任务提交给线程池
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}
