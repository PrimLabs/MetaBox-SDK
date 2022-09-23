use ic_cdk::export::candid::{self, CandidType, Deserialize};
use ic_cdk::api::call::CallResult;
use ic_agent::ic_types::Principal;
use candid::{Nat};

#[derive(CandidType, Deserialize, Debug)]
pub struct State {
    balance: Nat,
    memory_size: Nat,
    stable_memory_size: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct AssetExt {
    pub file_extension: String,
    pub upload_status: bool,
    pub bucket_id: Principal,
    pub aes_pub_key: Option<String>,
    pub file_name: String,
    pub file_key: String,
    pub total_size: u64,
    pub need_query_times: Nat,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct GET { pub flag: Nat, pub file_key: String }

#[derive(CandidType, Deserialize, Debug)]
pub struct OtherFile {
    file_extension: String,
    file_location: FileLocation,
    file_name: String,
    file_key: String,
    file_url: String,
}

#[derive(CandidType, Deserialize, Debug)]
struct ThumbNail { file_extension: String, image: Vec<u8> }

#[derive(CandidType, Deserialize, Debug)]
pub struct Avatar { pub data: Vec<u8>, pub data_type: String }

#[derive(CandidType, Deserialize, Debug)]
pub enum DataErr {
    FileKeyErr,
    FilePublic,
    BlobSizeError,
    PermissionDenied,
    SharedRepeat,
    FlagErr,
    SharedNotSet,
    MemoryInsufficient,
    FileAesPubKeyNotExist,
    UserAccessErr,
    DeviceNotExist,
    FileRepeat,
    ShareRepeat,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum AvlSMResult { ok(u64), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
pub enum CanisterStateResult { ok(State), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
enum SetShareFileResult { ok(String), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
enum GetSharedAesPublicResult { ok(String), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
enum GetDefaultDeviceShareDapResult { ok(String), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
enum DeleteOtherResult { ok(String), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
pub enum DeleteKeyResult { ok(String), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
enum DeleteShareFileResult { ok(String), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
enum DeleteSharedFileResult { ok(String), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
pub enum ClearAllResult { ok(String), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
pub enum CycleBalanceResult { ok(Nat), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
pub enum FileExt {
    EncryptFileExt(AssetExt),
    SharedFileExt{
        file_extension: String,
        other: Principal,
        description: String,
        file_name: String,
        file_key: String,
        isPublic: bool,
    },
    PlainFileExt(AssetExt),
}

#[derive(CandidType, Deserialize, Debug)]
pub enum PutResult { ok(FileExt), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
pub enum GetAssetExtKeyResult { ok(FileExt), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
enum FileLocation { IPFS, Arweave }

#[derive(CandidType, Deserialize, Debug)]
pub enum GetAssetExtsResult {
    ok(Vec<FileExt>,Vec<FileExt>,Vec<FileExt>,Vec<OtherFile>,Vec<OtherFile>,),
    err(DataErr),
}

#[derive(CandidType, Deserialize, Debug)]
enum GetCipherResult { ok(Vec<Vec<u8>>), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
enum GetFileShareOtherResult { ok(Vec<Principal>), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
enum GetOtherKeyResult { ok(OtherFile), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
pub enum GetPlainResult { ok(Vec<u8>), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
enum GetShareFilesResult { ok(Vec<FileExt>), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
enum GetThumbnailResult { ok(ThumbNail), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
pub struct Chunk { pub data: Vec<u8> }

#[derive(CandidType, Deserialize, Debug)]
pub enum PUT {
    segment{
        file_extension: String,
        order: Nat,
        chunk_number: Nat,
        chunk: Chunk,
        aes_pub_key: Option<String>,
        file_name: String,
        file_key: String,
        total_size: u64,
    },
    thumb_nail{
        file_extension: String,
        aes_pub_key: Option<String>,
        file_name: String,
        file_key: String,
        image: Vec<u8>,
    },
}

#[derive(CandidType, Deserialize, Debug)]
pub enum FilePut {
    EncryptFilePut(PUT),
    SharedFilePut{
        file_extension: String,
        other: Principal,
        aes_pub_key: Option<String>,
        description: String,
        file_name: String,
        file_key: String,
        isPublic: bool,
    },
    PlainFilePut(PUT),
}

#[derive(CandidType, Deserialize, Debug)]
enum RecordResult { ok(bool), err(DataErr) }

#[derive(CandidType, Deserialize, Debug)]
pub enum UploadResult { ok, err(DataErr) }

type DataBox = candid::Service;
struct SERVICE(Principal);
impl SERVICE{

    pub async fn avl_sm(&self) -> CallResult<(AvlSMResult,)> {
        ic_cdk::call(self.0, "avlSM", ()).await
    }

    pub async fn canister_state(&self) -> CallResult<(CanisterStateResult,)> {
        ic_cdk::call(self.0, "canisterState", ()).await
    }

    pub async fn clear_all(&self) -> CallResult<(ClearAllResult,)> {
        ic_cdk::call(self.0, "clearall", ()).await
    }

    pub async fn cycle_balance(&self) -> CallResult<(CycleBalanceResult,)> {
        ic_cdk::call(self.0, "cycleBalance", ()).await
    }

    pub async fn delete_share_file(
        &self,
        encrypt_file: String,
        other: Principal,
    ) -> CallResult<(DeleteShareFileResult,)> {
        ic_cdk::call(self.0, "deleteShareFile", (encrypt_file,other,)).await
    }

    pub async fn delete_shared_file(&self, shared_file: String) -> CallResult<
        (DeleteSharedFileResult,)
    > { ic_cdk::call(self.0, "deleteSharedFile", (shared_file,)).await }

    pub async fn delete_file(&self, file_key: String) -> CallResult<(DeleteKeyResult,)> {
        ic_cdk::call(self.0, "deletekey", (file_key,)).await
    }

    pub async fn delete_other(
        &self,
        file_key: String,
        file_location: FileLocation,
    ) -> CallResult<(DeleteOtherResult,)> {
        ic_cdk::call(self.0, "deleteother", (file_key,file_location,)).await
    }

    pub async fn get_asset_ext_key(&self, file_key: String) -> CallResult<(GetAssetExtKeyResult,)> {
        ic_cdk::call(self.0, "getAssetextkey", (file_key,)).await
    }

    pub async fn get_asset_exts(&self) -> CallResult<(GetAssetExtsResult,)> {
        ic_cdk::call(self.0, "getAssetexts", ()).await
    }

    pub async fn get_cipher(&self, g: GET) -> CallResult<(GetCipherResult,)> {
        ic_cdk::call(self.0, "getCipher", (g,)).await
    }

    pub async fn get_default_device_share_dap(&self, encrypt_file: String) -> CallResult<
        (GetDefaultDeviceShareDapResult,)
    > { ic_cdk::call(self.0, "getDefaultDeviceShareDap", (encrypt_file,)).await }

    pub async fn get_file_share_other(&self, encrypt_file: String) -> CallResult<
        (GetFileShareOtherResult,)
    > { ic_cdk::call(self.0, "getFileShareOther", (encrypt_file,)).await }

    pub async fn get_other_key(
        &self,
        file_key: String,
        file_location: FileLocation,
    ) -> CallResult<(GetOtherKeyResult,)> {
        ic_cdk::call(self.0, "getOtherkey", (file_key,file_location,)).await
    }

    pub async fn get_owner(&self) -> CallResult<(Principal,)> {
        ic_cdk::call(self.0, "getOwner", ()).await
    }

    pub async fn get_plain(&self, g: GET) -> CallResult<(GetPlainResult,)> {
        ic_cdk::call(self.0, "getPlain", (g,)).await
    }

    pub async fn get_share_files(&self) -> CallResult<(GetShareFilesResult,)> {
        ic_cdk::call(self.0, "getShareFiles", ()).await
    }

    pub async fn get_shared_aes_public(&self, shared_file: String) -> CallResult<
        (GetSharedAesPublicResult,)
    > { ic_cdk::call(self.0, "getSharedAesPublic", (shared_file,)).await }

    pub async fn get_thumbnail(&self, file_key: String) -> CallResult<(GetThumbnailResult,)> {
        ic_cdk::call(self.0, "getThumbnail", (file_key,)).await
    }

    pub async fn get_version(&self) -> CallResult<(Nat,)> {
        ic_cdk::call(self.0, "getVersion", ()).await
    }

    pub async fn put(&self, file_put: FilePut) -> CallResult<(PutResult,)> {
        ic_cdk::call(self.0, "put", (file_put,)).await
    }

    pub async fn record(&self, other_file: OtherFile) -> CallResult<(RecordResult,)> {
        ic_cdk::call(self.0, "record", (other_file,)).await
    }

    pub async fn set_share_file(
        &self,
        encrypt_file: String,
        other: Principal,
        default_aes_pubkey: String,
    ) -> CallResult<(SetShareFileResult,)> {
        ic_cdk::call(self.0, "setShareFile", (encrypt_file,other,default_aes_pubkey,)).await
    }

    pub async fn upload_avatar(&self, args: Avatar) -> CallResult<(UploadResult,)> {
        ic_cdk::call(self.0, "upload", (args,)).await
    }

}
