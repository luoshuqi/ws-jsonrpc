# jsonrpc

#### 介绍

Rust 编写的 jsonrpc 服务端，通过 websocket 调用。

#### 使用说明

##### 方法包装

```rust
#[rpc]
async fn greeting(name: String) -> Result<String, Infallible> {
    Ok(format!("Hello, {}", name))
}

#[rpc]
fn next_id() -> Result<u64, Infallible> {
    static ID: AtomicU64 = AtomicU64::new(1);
    Ok(ID.fetch_add(1, Ordering::Relaxed))
}
```

使用 `rpc` 宏可以把符合以下条件的函数（包括异步函数）包装成 rpc 方法:

1. 没有参数或者所有参数类型都实现了 `serde::Deserialize`
2. 返回类型为 `Result<T, E> where T: serde::Serialize, jsonrpc::response::Error: From<E>`

```rust

/* async */ fn(/* 0个或多个 */ arg: impl serde::Deserialize) -> Result<T, E> 
    where T: serde::Serialize, 
          jsonrpc::response::Error: From<E>
```

##### 方法注册

```rust
let mut handler = Handler::new();
handler.register(vec![method!(greeting), method!(next_id)]);
```

##### 方法调用

Handler::handle 方法会处理传入的 [Websocket](https://gitee.com/luoshuqi/ws) 对象上的请求。

```
handler.handle(ws)
```

[完整示例](https://gitee.com/luoshuqi/jsonrpc/blob/master/examples/example1.rs)