mod test_databox {
    extern crate metabox_sdk;
    use std::io::Write;
    use std::fs::OpenOptions;
    use metabox_sdk::databox;
    use candid::{ Nat, Principal};
    use databox::{DataErr, PutPlainFileResult, FileExt, AvlSMResult, CycleBalanceResult, CanisterStateResult, ClearAllResult, DeleteKeyResult, UploadResult};

    pub async fn test() {

        let response_1 = put_plain_files("source/", "4radi-oqaaa-aaaan-qapwa-cai").await;
        let mut index = 0;
        for i in &response_1 {
            index += 1;
            println!("file index: {:?}", index);
            println!("file name: {:?}", i.file_name);
            println!("file extension: {:?}", i.file_extension);
            println!("file key: {:?}", i.file_key);
            println!("file upload_status: {:?}", i.upload_status);
            println!("file in data box: {:?}", i.databox_canister_id.to_text());
            println!("file total_size: {:?}", i.total_size);
            println!("file chunk number: {:?}", i.chunk_number);
            println!("\n");
        }

        let response_2 = put_plain_file("source/bitcoin.pdf", "4radi-oqaaa-aaaan-qapwa-cai").await;
        println!("file name: {:?}", response_2.file_name);
        println!("file extension: {:?}", response_2.file_extension);
        println!("file key: {:?}", response_2.file_key);
        println!("file upload_status: {:?}", response_2.upload_status);
        println!("file in data box: {:?}", response_2.databox_canister_id.to_text());
        println!("file total_size: {:?}", response_2.total_size);
        println!("file chunk number: {:?}", response_2.chunk_number);

        let response_3 = get_plain_file("4radi-oqaaa-aaaan-qapwa-cai", "14d37b8971e5c73a523de39e0682ba0c08df3a503c49f4f976fe282bc60abfef").await.unwrap();
        let mut file = std::fs::File::create("output/a.pdf").expect("create failed");
        file.write_all(&response_3).expect("write failed");

        println!("upload avatar result:{:?}", upload_avatar("4radi-oqaaa-aaaan-qapwa-cai", "source/avatar.jpg").await);

        println!("delete file result:{:?}", delete_file("4radi-oqaaa-aaaan-qapwa-cai", "4da18028cb05cdb1a8e271c02c48dceef6ad89811adab9f9a3ab9e96db020fb9".to_string()).await);

        println!("clear data box result:{:?}", clear_data_box("4radi-oqaaa-aaaan-qapwa-cai").await);

        match get_file_info("4radi-oqaaa-aaaan-qapwa-cai", "3166112af0dcc940f8e7f2199a4200cfb5e2efb40796391201b8fe9e4ff7ca84").await {
            Ok(file_ext) => {
                match file_ext {
                    FileExt::PlainFileExt(asset_ext) => {
                        println!("file name: {:?}", asset_ext.file_name);
                        println!("file extension: {:?}", asset_ext.file_extension);
                        println!("file key: {:?}", asset_ext.file_key);
                        println!("file total_size: {:?}", asset_ext.total_size);
                        println!("file upload_status: {:?}", asset_ext.upload_status);
                        println!("file in data box: {:?}", asset_ext.bucket_id.to_text());
                        println!("file aes_pub_key: {:?}", asset_ext.aes_pub_key);
                        println!("file need_query_times: {:?}", asset_ext.need_query_times);
                    }
                    _ => {}
                }
            }
            Err(error) => {
                println!("get file info error: {:?}", error);
            }
        }

        match get_all_plain_files_info("4radi-oqaaa-aaaan-qapwa-cai").await {
            Ok(file_ext_s) => {
                let mut index = 0;
                for i in &file_ext_s {
                    index += 1;
                    match i {
                        FileExt::PlainFileExt(asset_ext) => {
                            println!("file index: {:?}", index);
                            println!("file name: {:?}", asset_ext.file_name);
                            println!("file extension: {:?}", asset_ext.file_extension);
                            println!("file key: {:?}", asset_ext.file_key);
                            println!("file total_size: {:?}", asset_ext.total_size);
                            println!("file upload_status: {:?}", asset_ext.upload_status);
                            println!("file in data box: {:?}", asset_ext.bucket_id.to_text());
                            println!("file aes_pub_key: {:?}", asset_ext.aes_pub_key);
                            println!("file need_query_times: {:?}", asset_ext.need_query_times);
                            println!("\n");
                        }
                        _ => {}
                    }
                }
            }
            Err(error) => {
                println!("get all plain files info error: {:?}", error);
            }
        }

        println!("data box version: {:?}", get_version("4radi-oqaaa-aaaan-qapwa-cai").await);

        println!("data box canister state: {:?}", get_canister_state("4radi-oqaaa-aaaan-qapwa-cai").await);

        println!("data box cycle balance: {:?}", get_cycle_balance("4radi-oqaaa-aaaan-qapwa-cai").await);

        println!("data box available stable memory: {:?}", get_avl_sm("4radi-oqaaa-aaaan-qapwa-cai").await);

        println!("data box owner: {:?}", get_owner("4radi-oqaaa-aaaan-qapwa-cai").await.to_text());
    }

    async fn put_plain_files(folder_path: &str, data_box_canister_id_text: &str,) -> Vec<PutPlainFileResult> {
        databox::put_plain_files("identities/identity.pem", folder_path, data_box_canister_id_text).await
    }

    async fn put_plain_file(file_path_str: &str, data_box_canister_id_text: &str,) -> PutPlainFileResult {
        databox::put_plain_file("identities/identity.pem", file_path_str, data_box_canister_id_text).await
    }

    async fn upload_avatar(data_box_canister_id_text: &str, avatar_file_path: &str) -> UploadResult {
        databox::upload_avatar("identities/identity.pem", data_box_canister_id_text, avatar_file_path).await
    }

    async fn delete_file(data_box_canister_id_text: &str, file_key: String) -> DeleteKeyResult {
        databox::delete_file("identities/identity.pem", data_box_canister_id_text, file_key).await
    }

    async fn clear_data_box(data_box_canister_id_text: &str,) -> ClearAllResult {
        databox::clear_data_box("identities/identity.pem", data_box_canister_id_text).await
    }

    async fn get_plain_file(data_box_canister_id_text: &str, file_key: &str) -> Result<Vec<u8>, DataErr> {
        databox::get_plain_file("identities/identity.pem", data_box_canister_id_text, file_key).await
    }

    async fn get_file_info(data_box_canister_id_text: &str, file_key: &str) -> Result<FileExt, DataErr> {
        databox::get_file_info("identities/identity.pem", data_box_canister_id_text, file_key).await
    }

    async fn get_all_plain_files_info(data_box_canister_id_text: &str) -> Result<Vec<FileExt>, DataErr> {
        databox::get_all_plain_files_info("identities/identity.pem", data_box_canister_id_text).await
    }

    async fn get_version(data_box_canister_id_text: &str,) -> Nat {
        databox::get_version("identities/identity.pem", data_box_canister_id_text).await
    }

    async fn get_canister_state(data_box_canister_id_text: &str,) -> CanisterStateResult {
        databox::get_canister_state("identities/identity.pem", data_box_canister_id_text).await
    }

    async fn get_cycle_balance(data_box_canister_id_text: &str,) -> CycleBalanceResult {
        databox::get_cycle_balance("identities/identity.pem", data_box_canister_id_text).await
    }

    async fn get_avl_sm(data_box_canister_id_text: &str,) -> AvlSMResult {
        databox::get_avl_sm("identities/identity.pem", data_box_canister_id_text).await
    }

    async fn get_owner(data_box_canister_id_text: &str,) -> Principal {
        databox::get_owner("identities/identity.pem", data_box_canister_id_text).await
    }
}

#[tokio::main]
async fn main() {
    test_databox::test().await;
}
