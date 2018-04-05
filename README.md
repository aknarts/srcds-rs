# srcds

Rust library for Source server queries and RCON.

## How to install

Add this your Cargo.toml:
```toml
[dependencies]
srcds = { git = "https://github.com/nstafie/srcds-rs" }
```


## How to use
```rust
extern crate srcds;
```

## Examples

```rust
extern crate srcds;

fn main() {
    let mut query = srcds::query::Query::new().unwrap();
    let info = query.info("server_ip.example.com:27017").unwrap();

    println!("{:?}", info.map);
}

```