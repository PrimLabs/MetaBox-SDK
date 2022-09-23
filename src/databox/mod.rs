use ic_agent::{Agent, identity::Secp256k1Identity, ic_types::Principal};
use ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport;
use garcon::Delay;
use std::fs;
use std::path::Path;
use std::ffi::OsStr;
use sha256::digest_bytes;
use rayon::prelude::*;
use candid::{Encode, Decode, Nat};
mod databox_did;
pub use databox_did::{ClearAllResult, DeleteKeyResult, UploadResult, Avatar, PUT, Chunk, FilePut, PutResult, DataErr, FileExt, GetAssetExtKeyResult, GET, GetPlainResult, CanisterStateResult, CycleBalanceResult, AvlSMResult, GetAssetExtsResult};

const UPDATE_SIZE: usize = 1992288;

#[derive(Debug)]
pub enum UploadStatus {
    Ok,
    Err(DataErr),
}

#[derive(Debug)]
pub struct PutPlainFileResult {
    pub file_name: String,
    pub file_extension: String,
    pub file_key: String,
    pub upload_status: UploadStatus,
    pub databox_canister_id: Principal,
    pub total_size: u64,
    pub chunk_number: u64,
}

/// Put plain files
///
/// Example code :
/// ``` no_run
/// use metabox_sdk::databox::{self, PutPlainFileResult};
///
/// async fn put_plain_files(folder_path: &str, data_box_canister_id_text: &str,) -> Vec<PutPlainFileResult> {
///     databox::put_plain_files("identities/identity.pem", folder_path, data_box_canister_id_text).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let response_1 = put_plain_files("source/", "4radi-oqaaa-aaaan-qapwa-cai").await;
///     let mut index = 0;
///     for i in &response_1 {
///         index += 1;
///         println!("file index: {:?}", index);
///         println!("file name: {:?}", i.file_name);
///         println!("file extension: {:?}", i.file_extension);
///         println!("file key: {:?}", i.file_key);
///         println!("file upload_status: {:?}", i.upload_status);
///         println!("file in data box: {:?}", i.databox_canister_id.to_text());
///         println!("file total_size: {:?}", i.total_size);
///         println!("file chunk number: {:?}", i.chunk_number);
///         println!("\n");
///     }
/// }
/// ```
pub async fn put_plain_files(pem_identity_path: &str, folder_path: &str, data_box_canister_id_text: &str,) -> Vec<PutPlainFileResult> {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let mut ans: Vec<PutPlainFileResult> = Vec::new();
    let paths = fs::read_dir(&folder_path).unwrap();
    for path in paths {
        let file_path = path.unwrap().file_name().into_string().unwrap();
        let pos: Vec<&str> = file_path.split(".").collect();
        let file_name = String::from(pos[0]);
        let file_extension = String::from(get_file_type(&String::from(pos[1])));
        let s = folder_path.to_owned() + &file_path;

        let (file_size, slice_size, data_slice) = get_file_from_source(&s);

        let puts = build_put_plain_args(
            file_name.clone(),
            file_extension.clone(),
            file_size.try_into().unwrap(),
            slice_size.try_into().unwrap(),
            &data_slice,
        );
        let file_key = match &puts[0] {
            FilePut::PlainFilePut(put) => {
                match put {
                    PUT::segment {file_extension, order, chunk_number, chunk, aes_pub_key, file_name, file_key, total_size} => {
                        file_key.clone()
                    }
                    _ => {"".to_string()}
                }
            }
            _ => {"".to_string()}
        };

        let mut flag = false;
        for put in &puts {
            let _response_blob = build_agent(pem_identity_path)
                .update(&canister_id, "put")
                .with_arg(Encode!(&put).expect("encode piece failed"))
                .call_and_wait(get_waiter())
                .await
                .expect("response error");
            let _response = Decode!(&_response_blob, PutResult).unwrap();
            match _response {
                PutResult::ok(..) => {
                },
                PutResult::err(data_err) => {
                    ans.push(PutPlainFileResult {
                        file_name: file_name.clone(),
                        file_extension: file_extension.clone(),
                        file_key: file_key.clone(),
                        upload_status: UploadStatus::Err(data_err),
                        databox_canister_id: canister_id,
                        total_size: file_size.try_into().unwrap(),
                        chunk_number: slice_size.try_into().unwrap(),
                    });
                    flag = true;
                    break;
                }
            }
        }
        if !flag { ans.push(PutPlainFileResult {
            file_name: file_name.clone(),
            file_extension: file_extension.clone(),
            file_key: file_key.clone(),
            upload_status: UploadStatus::Ok,
            databox_canister_id: canister_id,
            total_size: file_size.try_into().unwrap(),
            chunk_number: slice_size.try_into().unwrap(),
        }); }
    }
    ans
}

/// Put a plain file
///
/// Example code :
/// ``` no_run
/// use metabox_sdk::databox::{self, PutPlainFileResult};
///
/// async fn put_plain_file(file_path_str: &str, data_box_canister_id_text: &str,) -> PutPlainFileResult {
///     databox::put_plain_file("identities/identity.pem", file_path_str, data_box_canister_id_text).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let response_2 = put_plain_file("source/bitcoin.pdf", "4radi-oqaaa-aaaan-qapwa-cai").await;
///     println!("file name: {:?}", response_2.file_name);
///     println!("file extension: {:?}", response_2.file_extension);
///     println!("file key: {:?}", response_2.file_key);
///     println!("file upload_status: {:?}", response_2.upload_status);
///     println!("file in data box: {:?}", response_2.databox_canister_id.to_text());
///     println!("file total_size: {:?}", response_2.total_size);
///     println!("file chunk number: {:?}", response_2.chunk_number);
/// }
/// ```
pub async fn put_plain_file(pem_identity_path: &str, file_path_str: &str, data_box_canister_id_text: &str,) -> PutPlainFileResult {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let file_path = Path::new(file_path_str);
    let file_name = file_path.file_stem().unwrap().to_str().unwrap().to_owned();
    let file_extension = String::from(get_file_type(file_path.extension().unwrap().to_str().unwrap()));

    let (file_size, slice_size, data_slice) = get_file_from_source(file_path_str);
    let puts = build_put_plain_args(
        file_name.clone(),
        file_extension.clone(),
        file_size.try_into().unwrap(),
        slice_size.try_into().unwrap(),
        &data_slice,
    );
    let file_key = match &puts[0] {
        FilePut::PlainFilePut(put) => {
            match put {
                PUT::segment {file_extension, order, chunk_number, chunk, aes_pub_key, file_name, file_key, total_size} => {
                    file_key.clone()
                }
                _ => {"".to_string()}
            }
        }
        _ => {"".to_string()}
    };

    for put in &puts {
        let _response_blob = build_agent(pem_identity_path)
            .update(&canister_id, "put")
            .with_arg(Encode!(&put).expect("encode piece failed"))
            .call_and_wait(get_waiter())
            .await
            .expect("response error");
        let _response = Decode!(&_response_blob, PutResult).unwrap();
        match _response {
            PutResult::ok(..) => {
            },
            PutResult::err(data_err) => {
                return PutPlainFileResult {
                    file_name: file_name.clone(),
                    file_extension: file_extension.clone(),
                    file_key: file_key.clone(),
                    upload_status: UploadStatus::Err(data_err),
                    databox_canister_id: canister_id,
                    total_size: file_size.try_into().unwrap(),
                    chunk_number: slice_size.try_into().unwrap(),
                };
            }
        }
    }
    PutPlainFileResult {
        file_name: file_name.clone(),
        file_extension: file_extension.clone(),
        file_key: file_key.clone(),
        upload_status: UploadStatus::Ok,
        databox_canister_id: canister_id,
        total_size: file_size.try_into().unwrap(),
        chunk_number: slice_size.try_into().unwrap(),
    }
}

/// Upload avatar
///
/// Example code :
/// ``` no_run
/// use metabox_sdk::databox::{self, UploadResult};
///
/// async fn upload_avatar(data_box_canister_id_text: &str, avatar_file_path: &str) -> UploadResult {
///     databox::upload_avatar("identities/identity.pem", data_box_canister_id_text, avatar_file_path).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     println!("upload avatar result:{:?}", upload_avatar("4radi-oqaaa-aaaan-qapwa-cai", "source/avatar.jpg").await);
/// }
/// ```
pub async fn upload_avatar(pem_identity_path: &str, data_box_canister_id_text: &str, avatar_file_path: &str) -> UploadResult {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let context = fs::read(avatar_file_path).expect("read file failed");
    let file_extension = String::from(get_file_type(Path::new(avatar_file_path).extension().unwrap().to_str().unwrap()));
    let upload_args = Avatar {
        data: context,
        data_type: file_extension,
    };
    let response_blob = build_agent(pem_identity_path)
        .update(&canister_id, "upload")
        .with_arg(Encode!(&upload_args).expect("encode piece failed"))
        .call_and_wait(get_waiter())
        .await
        .expect("response error");
    let response = Decode!(&response_blob, UploadResult).unwrap();
    response
}

/// Delete a file
///
/// Example code :
/// ``` no_run
/// use metabox_sdk::databox::{self, DeleteKeyResult};
///
/// async fn delete_file(data_box_canister_id_text: &str, file_key: String) -> DeleteKeyResult {
///     databox::delete_file("identities/identity.pem", data_box_canister_id_text, file_key).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     println!("delete file result:{:?}", delete_file("4radi-oqaaa-aaaan-qapwa-cai", "4da18028cb05cdb1a8e271c02c48dceef6ad89811adab9f9a3ab9e96db020fb9".to_string()).await);
/// }
/// ```
pub async fn delete_file(pem_identity_path: &str, data_box_canister_id_text: &str, file_key: String) -> DeleteKeyResult {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let response_blob = build_agent(pem_identity_path)
        .update(&canister_id, "deletekey")
        .with_arg(Encode!(&file_key).expect("encode piece failed"))
        .call_and_wait(get_waiter())
        .await
        .expect("response error");
    let response = Decode!(&response_blob, DeleteKeyResult).unwrap();
    response
}

/// Clear the DataBox
///
/// Example code :
/// ``` no_run
/// use metabox_sdk::databox::{self, ClearAllResult};
///
/// async fn clear_data_box(data_box_canister_id_text: &str,) -> ClearAllResult {
///     databox::clear_data_box("identities/identity.pem", data_box_canister_id_text).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     println!("clear data box result:{:?}", clear_data_box("4radi-oqaaa-aaaan-qapwa-cai").await);
/// }
/// ```
pub async fn clear_data_box(pem_identity_path: &str, data_box_canister_id_text: &str,) -> ClearAllResult {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let response_blob = build_agent(pem_identity_path)
        .update(&canister_id, "clearall")
        .with_arg(Encode!().expect("encode piece failed"))
        .call_and_wait(get_waiter())
        .await
        .expect("response error");
    let response = Decode!(&response_blob, ClearAllResult).unwrap();
    response
}

/// Get the plain file Data
///
/// Example code :
/// ``` no_run
/// use std::io::Write;
/// use std::fs::OpenOptions;
/// use metabox_sdk::databox::{self, DataErr};
///
/// async fn get_plain_file(data_box_canister_id_text: &str, file_key: &str) -> Result<Vec<u8>, DataErr> {
///     databox::get_plain_file("identities/identity.pem", data_box_canister_id_text, file_key).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let response_3 = get_plain_file("4radi-oqaaa-aaaan-qapwa-cai", "14d37b8971e5c73a523de39e0682ba0c08df3a503c49f4f976fe282bc60abfef").await.unwrap();
///     let mut file = std::fs::File::create("output/a.pdf").expect("create failed");
///     file.write_all(&response_3).expect("write failed");
/// }
/// ```
pub async fn get_plain_file(pem_identity_path: &str, data_box_canister_id_text: &str, file_key: &str) -> Result<Vec<u8>, DataErr> {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let agent = build_agent(pem_identity_path);
    let file_ext = get_file_info(pem_identity_path, data_box_canister_id_text, file_key).await.unwrap();
    match file_ext {
        FileExt::PlainFileExt(asset_ext) => {
            let waiter = get_waiter();
            let mut i = 0;
            let need_query_times = asset_ext.need_query_times;
            let mut ans: Vec<u8> = Vec::new();
            while Nat::from(i) < need_query_times {
                let arg = GET {
                    flag: Nat::from(i),
                    file_key: file_key.to_string(),
                };
                let response_blob = agent
                    .update(&canister_id, "getPlain")
                    .with_arg(Encode!(&arg).expect("encode piece failed"))
                    .call_and_wait(waiter.clone())
                    .await
                    .expect("response error");
                i += 1;
                let response = Decode!(&response_blob, GetPlainResult).unwrap();
                match response {
                    GetPlainResult::ok(mut payload) => {
                        ans.append(&mut payload)
                    },
                    GetPlainResult::err(data_err) => {
                        return Err(data_err);
                    },
                }
            }
            Ok(ans)
        },
        _ => {
            Err(DataErr::FileKeyErr)
        }
    }
}

/// Get a file 's information
///
/// Example code :
/// ``` no_run
/// use metabox_sdk::databox::{self, FileExt, DataErr};
///
/// async fn get_file_info(data_box_canister_id_text: &str, file_key: &str) -> Result<FileExt, DataErr> {
///     databox::get_file_info("identities/identity.pem", data_box_canister_id_text, file_key).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     match get_file_info("4radi-oqaaa-aaaan-qapwa-cai", "3166112af0dcc940f8e7f2199a4200cfb5e2efb40796391201b8fe9e4ff7ca84").await {
///         Ok(file_ext) => {
///             match file_ext {
///                 FileExt::PlainFileExt(asset_ext) => {
///                     println!("file name: {:?}", asset_ext.file_name);
///                     println!("file extension: {:?}", asset_ext.file_extension);
///                     println!("file key: {:?}", asset_ext.file_key);
///                     println!("file total_size: {:?}", asset_ext.total_size);
///                     println!("file upload_status: {:?}", asset_ext.upload_status);
///                     println!("file in data box: {:?}", asset_ext.bucket_id.to_text());
///                     println!("file aes_pub_key: {:?}", asset_ext.aes_pub_key);
///                     println!("file need_query_times: {:?}", asset_ext.need_query_times);
///                 }
///             _ => {}
///             }
///         }
///         Err(error) => {
///             println!("get file info error: {:?}", error);
///         }
///     }
/// }
/// ```
pub async fn get_file_info(pem_identity_path: &str, data_box_canister_id_text: &str, file_key: &str) -> Result<FileExt, DataErr> {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let response_blob = build_agent(pem_identity_path)
        .query(&canister_id, "getAssetextkey")
        .with_arg(Encode!(&file_key).expect("encode piece failed"))
        .call()
        .await
        .expect("response error");
    let response = Decode!(&response_blob, GetAssetExtKeyResult).unwrap();
    match response {
        GetAssetExtKeyResult::ok(file_ext) => Ok(file_ext),
        GetAssetExtKeyResult::err(data_err) => Err(data_err),
    }
}

/// Get all plain files 's information
///
/// Example code :
/// ``` no_run
/// use metabox_sdk::databox::{self, FileExt, DataErr};
///
/// async fn get_all_plain_files_info(data_box_canister_id_text: &str) -> Result<Vec<FileExt>, DataErr> {
///     databox::get_all_plain_files_info("identities/identity.pem", data_box_canister_id_text).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     match get_all_plain_files_info("4radi-oqaaa-aaaan-qapwa-cai").await {
///         Ok(file_ext_s) => {
///             let mut index = 0;
///             for i in &file_ext_s {
///                 index += 1;
///                 match i {
///                     FileExt::PlainFileExt(asset_ext) => {
///                         println!("file index: {:?}", index);
///                         println!("file name: {:?}", asset_ext.file_name);
///                         println!("file extension: {:?}", asset_ext.file_extension);
///                         println!("file key: {:?}", asset_ext.file_key);
///                         println!("file total_size: {:?}", asset_ext.total_size);
///                         println!("file upload_status: {:?}", asset_ext.upload_status);
///                         println!("file in data box: {:?}", asset_ext.bucket_id.to_text());
///                         println!("file aes_pub_key: {:?}", asset_ext.aes_pub_key);
///                         println!("file need_query_times: {:?}", asset_ext.need_query_times);
///                         println!("\n");
///                     }
///                     _ => {}
///                 }
///             }
///         }
///         Err(error) => {
///             println!("get all plain files info error: {:?}", error);
///         }
///     }
/// }
/// ```
pub async fn get_all_plain_files_info(pem_identity_path: &str, data_box_canister_id_text: &str) -> Result<Vec<FileExt>, DataErr> {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let response_blob = build_agent(pem_identity_path)
        .query(&canister_id, "getAssetexts")
        .with_arg(Encode!().expect("encode piece failed"))
        .call()
        .await
        .expect("response error");
    let response = Decode!(&response_blob, GetAssetExtsResult).unwrap();
    match response {
        GetAssetExtsResult::ok(plain_assets, ..) => {
            return Ok(plain_assets);
        },
        GetAssetExtsResult::err(data_err) => {
            return Err(data_err);
        },
    }
}

/// Get DataBox version
///
/// Example code :
/// ``` no_run
/// use candid::Nat;
/// use metabox_sdk::databox;
///
/// async fn get_version(data_box_canister_id_text: &str,) -> Nat {
///     databox::get_version("identities/identity.pem", data_box_canister_id_text).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     println!("data box version: {:?}", get_version("4radi-oqaaa-aaaan-qapwa-cai").await);
/// }
/// ```
pub async fn get_version(pem_identity_path: &str, data_box_canister_id_text: &str,) -> Nat {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let response_blob = build_agent(pem_identity_path)
        .query(&canister_id, "getVersion")
        .with_arg(Encode!().expect("encode piece failed"))
        .call()
        .await
        .expect("response error");
    let response = Decode!(&response_blob, Nat).unwrap();
    response
}

/// Get DataBox canister state
///
/// Example code :
/// ``` no_run
/// use metabox_sdk::databox::{self, CanisterStateResult};
///
/// async fn get_canister_state(data_box_canister_id_text: &str,) -> CanisterStateResult {
///     databox::get_canister_state("identities/identity.pem", data_box_canister_id_text).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     println!("data box canister state: {:?}", get_canister_state("4radi-oqaaa-aaaan-qapwa-cai").await);
/// }
/// ```
pub async fn get_canister_state(pem_identity_path: &str, data_box_canister_id_text: &str,) -> CanisterStateResult {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let response_blob = build_agent(pem_identity_path)
        .query(&canister_id, "canisterState")
        .with_arg(Encode!().expect("encode piece failed"))
        .call()
        .await
        .expect("response error");
    let response = Decode!(&response_blob, CanisterStateResult).unwrap();
    response
}

/// Get DataBox cycle balance
///
/// Example code :
/// ``` no_run
/// use metabox_sdk::databox::{self, CycleBalanceResult};
///
/// async fn get_cycle_balance(data_box_canister_id_text: &str,) -> CycleBalanceResult {
///     databox::get_cycle_balance("identities/identity.pem", data_box_canister_id_text).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     println!("data box cycle balance: {:?}", get_cycle_balance("4radi-oqaaa-aaaan-qapwa-cai").await);
/// }
/// ```
pub async fn get_cycle_balance(pem_identity_path: &str, data_box_canister_id_text: &str,) -> CycleBalanceResult {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let response_blob = build_agent(pem_identity_path)
        .query(&canister_id, "cycleBalance")
        .with_arg(Encode!().expect("encode piece failed"))
        .call()
        .await
        .expect("response error");
    let response = Decode!(&response_blob, CycleBalanceResult).unwrap();
    response
}

/// Get DataBox available stable memory
///
/// Example code :
/// ``` no_run
/// use metabox_sdk::databox::{self, AvlSMResult};
///
/// async fn get_avl_sm(data_box_canister_id_text: &str,) -> AvlSMResult {
///     databox::get_avl_sm("identities/identity.pem", data_box_canister_id_text).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     println!("data box available stable memory: {:?}", get_avl_sm("4radi-oqaaa-aaaan-qapwa-cai").await);
/// }
/// ```
pub async fn get_avl_sm(pem_identity_path: &str, data_box_canister_id_text: &str,) -> AvlSMResult {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let response_blob = build_agent(pem_identity_path)
        .query(&canister_id, "avlSM")
        .with_arg(Encode!().expect("encode piece failed"))
        .call()
        .await
        .expect("response error");
    let response = Decode!(&response_blob, AvlSMResult).unwrap();
    response
}

/// Get DataBox owner
///
/// Example code :
/// ``` no_run
/// use candid::Principal;
/// use metabox_sdk::databox::{self, AvlSMResult};
///
/// async fn get_owner(data_box_canister_id_text: &str,) -> Principal {
///     databox::get_owner("identities/identity.pem", data_box_canister_id_text).await
/// }
///
/// #[tokio::main]
/// async fn main() {
///     println!("data box owner: {:?}", get_owner("4radi-oqaaa-aaaan-qapwa-cai").await.to_text());
/// }
/// ```
pub async fn get_owner(pem_identity_path: &str, data_box_canister_id_text: &str,) -> candid::Principal {
    let canister_id = Principal::from_text(data_box_canister_id_text).unwrap();
    let response_blob = build_agent(pem_identity_path)
        .query(&canister_id, "getOwner")
        .with_arg(Encode!().expect("encode piece failed"))
        .call()
        .await
        .expect("response error");
    let response = Decode!(&response_blob, candid::Principal).unwrap();
    response
}

// Access file from file path, slice and return [each slice] array
fn get_file_from_source(path: &str) -> (usize, usize, Vec<Vec<u8>>) {
    let context = fs::read(path).expect("read file failed");
    let size = context.len();
    let slice_size = if context.len() % UPDATE_SIZE == 0 {
        context.len() / UPDATE_SIZE
    } else {
        context.len() / UPDATE_SIZE + 1
    };
    let mut res = Vec::new();
    for index in 0..slice_size {
        if index == slice_size - 1 {
            res.push(context[index * UPDATE_SIZE..context.len()].to_owned())
        } else {
            res.push(context[index * UPDATE_SIZE..(index + 1) * UPDATE_SIZE].to_owned())
        }
    }
    (size, slice_size, res)
}

fn build_put_plain_args(
    file_name: String,
    file_extension: String,
    total_size: u64,
    chunk_number: u64,
    data_slice: &Vec<Vec<u8>>,
) -> Vec<FilePut> {
    let mut order = 0;
    let mut puts: Vec<FilePut> = Vec::new();
    let file_key = get_file_key(&get_file_sha256_digest(data_slice));
    for data in data_slice {
        puts.push(FilePut::PlainFilePut(PUT::segment {
            aes_pub_key: None,
            file_key: file_key.clone(),
            file_name: file_name.clone(),
            file_extension: file_extension.clone(),
            chunk: Chunk {
                data: data.clone(),
            },
            chunk_number: Nat::from(chunk_number),
            order: Nat::from(order),
            total_size: total_size.clone(),
        }));
        order += 1;
    }
    puts
}

fn get_file_sha256_digest(context: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut digests = vec![vec![0x00 as u8]; context.len()];
    let mut contents = digests.iter_mut().zip(context.iter()).collect::<Vec<_>>();
    contents
        .par_iter_mut()
        .for_each(|(d, text)| **d = digest_bytes(*text).into_bytes()[..32].to_vec());
    digests
}

fn get_file_key(digests: &Vec<Vec<u8>>) -> String {
    let mut digest = vec![0x00 as u8; 32 * digests.len()];
    let mut _index = 0;
    for bytes in digests {
        for byte in bytes {
            digest.push(*byte);
            _index += 1;
        }
    }
    digest_bytes(&digest)
}

fn get_file_type(file_type: &str) -> &str {
    if file_type == "pdf" {
        return "application/pdf";
    } else if file_type == "jpg" || file_type == "jpeg" {
        return "image/jpg";
    } else if file_type == "png" {
        return "image/png";
    } else if file_type == "mp4" {
        return "video/mp4";
    } else if file_type == "mp3" {
        return "audio/mp3";
    } else if file_type == "gif" {
        return "image/gif";
    } else if file_type == "txt" {
        return "text/plain";
    } else if file_type == "ppt" || file_type == "pptx" {
        return "application/vnd.ms-powerpoint";
    } else if file_type == "html" || file_type == "xhtml" {
        return "text/html";
    } else if file_type == "doc" || file_type == "docx" {
        return "application/msword";
    } else if file_type == "xls" {
        return "application/x-xls";
    } else if file_type == "apk" {
        return "application/vnd.android.package-archive";
    } else if file_type == "svg" {
        return "text/xml";
    } else if file_type == "wmv" {
        return "video/x-ms-wmv";
    } else {
        return "application/octet-stream";
    }
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

// pub async fn put_encrypt_files(pem_identity_path: &str, folder_path: &str, databox_canister_id_text: &str,) -> Vec<PutPlainFileResult> {
//     let canister_id = Principal::from_text(databox_canister_id_text).unwrap();
//     let agent = build_agent(pem_identity_path);
//     let waiter = get_waiter();
//
//     let mut ans: Vec<PutPlainFileResult> = Vec::new();
//     let paths = fs::read_dir(&folder_path).unwrap();
//     for path in paths {
//         let file_path = path.unwrap().file_name().into_string().unwrap();
//         let pos: Vec<&str> = file_path.split(".").collect();
//         let file_name = String::from(pos[0]);
//         let file_extension = String::from(get_file_type(&String::from(pos[1])));
//         let s = folder_path.to_owned() + &file_path;
//
//         let (file_size, slice_size, data_slice) = get_file_from_source(&s);
//
//         let puts = build_put_plain_args(
//             file_name.clone(),
//             file_extension.clone(),
//             file_size.try_into().unwrap(),
//             slice_size.try_into().unwrap(),
//             &data_slice,
//         );
//
//         let file_key = match &puts[0] {
//             FilePut::PlainFilePut(put) => {
//                 match put {
//                     PUT::segment {file_extension, order, chunk_number, chunk, aes_pub_key, file_name, file_key, total_size} => {
//                         file_key.clone()
//                     }
//                     _ => {"".to_string()}
//                 }
//             }
//             _ => {"".to_string()}
//         };
//
//         let mut flag = false;
//         for put in &puts {
//             let _response_blob = agent
//                 .update(&canister_id, "put")
//                 .with_arg(Encode!(&put).expect("encode piece failed"))
//                 .call_and_wait(waiter.clone())
//                 .await
//                 .expect("response error");
//             let _response = Decode!(&_response_blob, PutResult).unwrap();
//             match _response {
//                 PutResult::ok(..) => {
//                 },
//                 PutResult::err(data_err) => {
//                     ans.push(PutPlainFileResult {
//                         file_name: file_name.clone(),
//                         file_extension: file_extension.clone(),
//                         file_key: file_key.clone(),
//                         upload_status: UploadStatus::Err(data_err),
//                         databox_canister_id: canister_id,
//                         total_size: file_size.try_into().unwrap(),
//                         chunk_number: slice_size.try_into().unwrap(),
//                     });
//                     flag = true;
//                     break;
//                 }
//             }
//         }
//         if !flag { ans.push(PutPlainFileResult {
//             file_name: file_name.clone(),
//             file_extension: file_extension.clone(),
//             file_key: file_key.clone(),
//             upload_status: UploadStatus::Ok,
//             databox_canister_id: canister_id,
//             total_size: file_size.try_into().unwrap(),
//             chunk_number: slice_size.try_into().unwrap(),
//         }); }
//     }
//     ans
// }

/*fn build_ciphertext_put(
    file_name: String,
    file_extension: String,
    total_size: u64,
    data_slice: &Vec<Vec<u8>>,
    aes_key_text: &Vec<u8>, // aes_key mingwen
    rsa_key: &Rsa<Private>, // rsa密钥
    iv: &Vec<u8>, //
) -> Vec<FilePut> {
    let mut order = 0;
    let mut puts = vec![];
    let mut encrypted_aes_key = vec![];
    let _ = rsa_key.public_encrypt(aes_key_text, &mut encrypted_aes_key, Padding::PKCS1);
    let encrypted_aes_key = hex::encode(&encrypted_aes_key);
    let mut cipher_text = vec![vec![]; data_slice.len()];
    let aes_key = AesKey::new_encrypt(aes_key_text).expect("get aes key failed");
    cipher_text
        .iter_mut()
        .zip(data_slice.iter())
        .collect::<Vec<_>>()
        .par_iter_mut()
        .for_each(|(cipher, data)| {
            let mut iv_temp = iv.clone();
            aes_ige(*data, *cipher, &aes_key, &mut iv_temp, Mode::Encrypt)
        });
    let file_key = get_file_key(&get_file_sha256_digest(&cipher_text));
    for cipher in cipher_text {
        puts.push(FilePut::EncryptFilePut(PUT::segment {
            aes_pub_key: Some(encrypted_aes_key.clone()),
            file_key: file_key.clone(),
            file_name: file_name.clone(),
            file_extension: file_extension.clone(),
            chunk: Chunk {
                data: cipher.to_vec(),
            },
            chunk_number: Nat::from(cipher.len()),
            order: Nat::from(order),
            total_size: total_size.clone(),
        }));
        order += 1;
    }
    puts
}*/
