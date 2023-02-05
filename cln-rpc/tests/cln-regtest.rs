use std::io::Read;
use bitcoincore_rpc::RpcApi;
use bitcoincore_rpc::bitcoin::Address;
use std::str::FromStr;
use bitcoin::network::address::AddrV2::Ipv4;
use futures_util::TryFutureExt;
use cln_rpc::model::{ConnectRequest, FundchannelRequest, GetinfoBinding, GetinfoBindingType, GetinfoRequest, ListchannelsChannels, ListchannelsRequest, ListfundsRequest, NewaddrRequest, NewaddrResponse};
use cln_rpc::{ClnRpc, primitives, Request, Response};
use cln_rpc::model::NewaddrAddresstype::BECH32;
use cln_rpc::primitives::AmountOrAll;
use cln_rpc::primitives::AmountOrAll::Amount;

use cln_test::runner::*;

// Check that we have node and API operational
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn basic_test() {
    run_cln_test(|btc, mut cln_back, mut cln_peer| async move {
        let r = cln_back
            .call(Request::NewAddr(NewaddrRequest { addresstype: Option::from(BECH32) }))
            .await
            .expect("New address request");

        let addr = NewaddrResponse::try_from(r).expect("Address expected");

        let back_addr = Address::from_str(
            addr.bech32.expect("Address").as_str()
        ).unwrap();

        println!("Minting to backend node address {:?}", back_addr);
        let h = mine_and_sync(&btc, 101, &back_addr, vec![&mut cln_back, &mut cln_peer]).await;
        println!("Network synced to height {}", h);

        println!("Running basic Bitcoind test");
        let info = btc.get_blockchain_info().expect("blockchain info");
        println!("{:?}", info);
        assert_eq!(info.chain, "regtest");

        let mut uri: String;

        println!("Running basic Core Lightning test");
        let info = cln_back
            .call(Request::Getinfo(GetinfoRequest {}))
            .await;

        match info {
            Ok(Response::Getinfo(r)) => {
                println!("{:?}", r);
                assert_eq!(r.network, "regtest");
            },
            _ => {},
        }

        let addr = cln_back
            .call(Request::NewAddr(NewaddrRequest { addresstype: Option::from(BECH32) }))
            .await;

        match addr {
            Ok(Response::NewAddr(r)) => {
                println!("Minting to backend node address {:?}", r.bech32);
                let back_addr = Address::from_str(
                    r.bech32.expect("Address").as_str()
                ).unwrap();
                let h = mine_and_sync(&btc, 101, &back_addr, vec![&mut cln_back, &mut cln_peer]).await;
                println!("Network synced to height {}", h);
            },
            _ => {},
        }

        let funds = cln_back
            .call(Request::ListFunds(ListfundsRequest { spent: Some(false) }))
            .await;

        match funds {
            Ok(Response::ListFunds(r)) => {
                println!("Checking backend node funds {:?}", r.outputs);
            },
            _ => {},
        }

        let info = cln_peer
            .call(Request::Getinfo(GetinfoRequest {}))
            .await;

        match info {
            Ok(Response::Getinfo(r)) => {
                println!("{:?}", r);
                assert_eq!(r.network, "regtest");
                for bind in r.binding.expect("Vector with bindings") {
                    match bind.item_type {
                        GetinfoBindingType::IPV4 => {
                            let id = r.id.clone().to_string();
                            let host = Some(bind.address.unwrap());
                            let port = Some(bind.port.unwrap());

                            uri = format!("{}@{}:{}", id.clone(), host.clone().expect("Host"), port.clone().expect("Port"));
                            println!("Connecting with peer {uri} and opening new channel");

                            let rconn = cln_back
                                .call(Request::Connect(ConnectRequest { id: id.clone(), host: host.clone(), port: port.clone() }))
                                .await;

                            match rconn {
                                Ok(Response::Connect(conn)) => {
                                    println!("Connected: {:?}", conn);
                                },
                                _ => {
                                    println!("Not connected!")
                                },
                            }

                            let rfund = cln_back
                                .call(Request::FundChannel(FundchannelRequest {
                                    id: r.id.clone(),
                                    amount: Amount(primitives::Amount::from_sat(1000000)),
                                    feerate: None,
                                    announce: None,
                                    minconf: None,
                                    push_msat: None,
                                    close_to: None,
                                    request_amt: None,
                                    compact_lease: None,
                                    utxos: None,
                                    mindepth: None,
                                    reserve: None
                                }))
                                .await;

                            match rfund {
                                Ok(Response::FundChannel(fund)) => {
                                    println!("Channel funded: {:?}", fund.txid);
                                },
                                _ => {
                                    println!("Channel not funded!")
                                },
                            }
                        }
                        _ => { },
                    }
                }
            },
            _ => {},
        }

        let h = mine_and_sync(&btc, 12, &back_addr, vec![&mut cln_back, &mut cln_peer]).await;
        println!("Network synced to height {}", h);

        let info = cln_back
            .call(Request::Getinfo(GetinfoRequest {}))
            .await;

        match info {
            Ok(Response::Getinfo(r)) => {
                println!("Channels active {:?}, inactive {:?}, pending {:?}", r.num_active_channels, r.num_inactive_channels, r.num_pending_channels);
                println!("Peers {:?}", r.num_peers);
                assert_eq!(r.network, "regtest");
            },
            _ => {},
        }

        let chans = cln_back
            .call(Request::ListChannels(ListchannelsRequest {
                short_channel_id: None,
                source: None,
                destination: None
            }))
            .await;

        match chans {
            Ok(Response::ListChannels(r)) => {
                println!("Checking backend node channels {:?}", r.channels);
            },
            _ => {
                println!("No channels?")
            },
        }
    })
        .await;
}
