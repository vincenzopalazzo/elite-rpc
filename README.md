# Elite RPC

Type Safe JSON RPC like client with different transport and protocol layers.

## Overview

This library provides a type-safe JSON RPC client implementation for Rust, supporting various transport layers. It is designed to be simple to use while ensuring safety and reliability through Rust's strong type system.

## Features

- Type-safe JSON RPC client implementation
- Support for multiple transport layers
- Easy to use and integrate into your projects

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
elite-rpc = "0.0.1"
```

## Usage

Here's a simple example of how to use the library:

use elite_rpc::Client;

``` rust
use elite_rpc::protocol::Protocol;
use elite_rpc::transport::Transport;
use elite_rpc::EliteRPC;

pub struct MockTransport;
#[derive(Clone)]
pub struct MockProtocol;

impl Transport<MockProtocol> for MockTransport {
    fn new(_: &str, _: MockProtocol) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self)
    }

    fn call(
        &self,
        _: &str,
        _: &<MockProtocol as Protocol>::InnerType,
    ) -> anyhow::Result<<MockProtocol as Protocol>::InnerType> {
        Ok(())
    }
}

impl Protocol for MockProtocol {
    type InnerType = ();

    fn from_from_request(
        &self,
        _: &[u8],
        _: Option<crate::protocol::Encoding>,
    ) -> anyhow::Result<Self::InnerType> {
        Ok(())
    }

    fn new() -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(MockProtocol)
    }

    fn to_request(&self, _: &str, _: &Self::InnerType) -> anyhow::Result<Self::InnerType> {
        Ok(())
    }
}

fn main() {
    let rpc = EliteRPC::<MockTransport, MockProtocol>::new("http://localhost:8545")?;
    let response = rpc.call("method_name", &())?;
    assert_eq!(response, ());
}
```

Replace "http://localhost:8545", "method_name", and the parameters with your actual JSON RPC endpoint and method.

## License

<div align="center">
  <img src="https://cdn-icons-png.flaticon.com/512/2111/2111503.png" width="150" height="150"/>
</div>

 Type Safe JSON RPC like client with different transport and protocol layers.

 Copyright (C) 2024 Vincenzo Palazzo vincenzopalazzodev@gmail.com
 
 This program is free software; you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation; either version 2 of the License.
 
 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.
 
 You should have received a copy of the GNU General Public License along
 with this program; if not, write to the Free Software Foundation, Inc.,
 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
