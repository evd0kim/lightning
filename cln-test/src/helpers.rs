use bitcoincore_rpc::bitcoin::Address;
use bitcoincore_rpc::{Client, RpcApi};
use std::str::FromStr;

use cln_rpc::{ClnRpc, Request, Response};
use cln_rpc::model::NewaddrRequest;

pub async fn fund_node_wallet(client: &Client, block_num: u64, ln_client: &mut ClnRpc) {
    let result = ln_client
        .call(Request::NewAddr(NewaddrRequest { addresstype: None }))
        .await
        .expect("couldn't connect to peer node");

    match result {
        Response::NewAddr(r) => {
            let address = Address::from_str(r.bech32.expect("Cant get bech32").as_str()).expect("BTC address");
            client
                .generate_to_address(block_num, &address)
                .expect("mined blocks");
        },
        r => println!("Unexpected result {:?} to method call NewAddr", r),
    }
}
