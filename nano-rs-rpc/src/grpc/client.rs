use nano_rs_core::config::rpc::RpcConfig;
use tower::timeout::Timeout;
use std::time::Duration;

///获取grpc channel
pub async fn get_channel(rpc_config: &RpcConfig) -> Result<Timeout<tonic::transport::Channel>, String> {
    //直连
    if rpc_config.is_direct() {
        if let Some(urls) = rpc_config.clone().direct {
            let endpoints = urls
                .iter()
                .map(|a| tonic::transport::Channel::from_shared(a.clone().into_bytes()).expect("get endpoint failed"));
            let channel = tonic::transport::Channel::balance_list(endpoints);
            Ok(Timeout::new(channel, Duration::from_secs(rpc_config.timeout.unwrap_or(5) as u64)))
        } else {
            Err(String::from("grpc channel get direct failed"))
        }
    } else {
        let rpc_key = rpc_config.key.clone().expect("rpc key is not set");
        //从etcd获取
        let etcd_config = rpc_config.clone().etcd.expect("rpc config etcd get failed");
        let hosts = etcd_config.host.expect("etcd host get failed");
        let mut client;
        if etcd_config.user.is_some() {
            let user = etcd_config.user.expect("etcd user get failed");
            let password = etcd_config.pass_word.expect("etcd password get failed");
            let options = Some(etcd_client::ConnectOptions::new().with_user(
                user,
                password,
            ));
            client = etcd_client::Client::connect(hosts, options).await.expect("etcd client connect failed");
        } else {
            client = etcd_client::Client::connect(hosts, None).await.expect("etcd client connect failed");
        }
        let resp = client.leases().await.expect("etcd leases failed");
        let lease_status = resp.leases();
        let mut urls = vec![];
        for x in lease_status {
            let key = rpc_key.clone() + "/" + x.id().to_string().as_str();
            let resp = client.get(key, None).await.expect("etcd get failed");
            let value = resp.kvs().first().expect("etcd get value failed").value_str().expect("etcd get value failed");
            if value.starts_with("http") {
                urls.push(value.to_string());
            } else {
                urls.push("http://".to_owned() + value);
            }
        }
        let endpoints = urls
            .iter()
            .map(|a| tonic::transport::Channel::from_shared(a.clone().into_bytes()).expect("get endpoint failed"));
        let channel = tonic::transport::Channel::balance_list(endpoints);
        Ok(Timeout::new(channel, Duration::from_secs(rpc_config.timeout.unwrap_or(5) as u64)))
    }
}