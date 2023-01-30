use bitcoincore_rpc::RpcApi;
use bitcoincore_rpc::bitcoin::Address;
use std::str::FromStr;

use cln_test::runner::*;

// Check that we have node and API operational
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn basic_test() {
    run_cln_test(|btc, cln_back, cln_peer| async move {
        let addr = Address::from_str("bcrt1qsdzqt93xsyewdjvagndw9523m27e52er5ca7hm").unwrap();
        let res = btc
            .generate_to_address(101, &addr)
            .expect("generate to address failed");
        println!("{:?}", res);

        println!("Running basic Bitcoind test");
        let info = btc.get_blockchain_info().expect("blockchain info");
        println!("{:?}", info);
        assert_eq!(info.chain, "regtest");

        /*
println!("Running basic Core Lightning test");
let info = cln_back.getinfo().expect("blockchain info");
println!("{:?}", info);
assert_eq!(info.network, "regtest");
let info = cln_peer.getinfo().expect("blockchain info");
println!("{:?}", info);

let mut uri = format!("{}", info.id);
for addr in info.binding {
    match addr {
        Ipv4 { address, port } => {
            uri = format!("{}@{}:{}", uri, address, port);
            println!("{uri}");
            return;
        }
        _ => println!("{:?}", addr),
    }
}

        println!("Opening channel");
        println!("{}", uri);

        assert_eq!(info.network, "regtest");

        let info = btc.get_blockchain_info().expect("blockchain info");
        println!("{:?}", info);

        let resp = cln_back.newaddr(None).expect("address wasn't provided");
        let addr_str = resp.bech32.unwrap();
        let addr = Address::from_str(addr_str.as_str()).unwrap();
        btc.generate_to_address(101, &addr)
            .expect("generate to address failed");

        //let resp= cln_back.connect(uri.as_str(), None).expect(format!("couldn't connect to {uri}").as_str());
        //println!("{:?}", resp);

        //let channels = cln_back.listchannels(None).expect("all channels info");
        //println!("{:?}", channels);
        //assert_eq!(info.network, "regtest");

        println!("Running basic Core Lightning test");
        let info = cln_back.getinfo().expect("blockchain info");
        println!("{:?}", info);
        assert_eq!(info.network, "regtest");
        let info = cln_peer.getinfo().expect("blockchain info");
        println!("{:?}", info);
        assert_eq!(info.network, "regtest");

        let logs = cln_back.getlog(None).expect("Expected logs");
        println!("{}", logs.log.len());
        for l in logs.log {
            println!("{:?}", l);
        }

        cln_back.stop().expect("CLN client didn't stop");
        cln_peer.stop().expect("CLN client didn't stop");
        */
    })
        .await;
}
