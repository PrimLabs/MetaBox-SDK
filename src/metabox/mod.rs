use garcon::Delay;
use ic_agent::Agent;
use ic_agent::identity::Secp256k1Identity;
use ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport;
use candid::{Decode, Encode, Principal};
mod metabox_did;
use metabox_did::{CreateBoxArgs, CreateBoxResult, BoxMetadata, BoxInfo};
use crate::metabox::metabox_did::BoxType;

static MetaBox_CANISTER_ID_TEXT: &'static str = "zbzr7-xyaaa-aaaan-qadeq-cai";

pub async fn create_data_box(pem_identity_path: &str, icp_amount: u64, box_name: String, is_private: bool) -> CreateBoxResult{
    let canister_id = Principal::from_text(MetaBox_CANISTER_ID_TEXT).unwrap();
    let agent = build_agent(pem_identity_path);
    let user_principal = agent.get_principal().unwrap();
    let args = CreateBoxArgs {
        metadata: BoxMetadata {
            is_private: is_private.clone(),
            box_name: box_name.clone(),
            box_type: BoxType::data_box,
        },
        install_args: Encode!(&user_principal).expect("encode install args failed"),
        icp_amount: icp_amount.clone(),
    };
    let response_blob = agent
        .update(&canister_id, "createBox")
        .with_arg(Encode!(&args).expect("encode piece failed"))
        .call_and_wait(get_waiter())
        .await
        .expect("response error");
    Decode!(&response_blob, CreateBoxResult).unwrap()
}

pub async fn get_boxes(pem_identity_path: &str, who: candid::Principal) -> Vec<BoxInfo> {
    let canister_id = Principal::from_text(MetaBox_CANISTER_ID_TEXT).unwrap();
    let response_blob = build_agent(pem_identity_path)
        .query(&canister_id, "getBoxes")
        .with_arg(Encode!(&who).expect("encode piece failed"))
        .call()
        .await
        .expect("response error");
    Decode!(&response_blob, Vec<BoxInfo>).unwrap()
}

fn get_waiter() -> Delay {
    let waiter = garcon::Delay::builder()
        .throttle(std::time::Duration::from_millis(500))
        .timeout(std::time::Duration::from_secs(60 * 5))
        .build();
    waiter
}

fn build_agent(pem_identity_path: &str) -> Agent {
    let url = "https://ic0.app".to_string();
    let identity = Secp256k1Identity::from_pem_file(String::from(pem_identity_path)).unwrap();
    let transport = ReqwestHttpReplicaV2Transport::create(url).expect("transport error");
    let agent = Agent::builder()
        .with_transport(transport)
        .with_identity(identity)
        .build()
        .expect("build agent error");
    agent
}