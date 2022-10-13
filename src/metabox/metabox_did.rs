use ic_cdk::export::candid::{self, CandidType, Deserialize};
use ic_cdk::api::call::CallResult;

#[derive(CandidType, Deserialize,Debug)]
pub enum BoxType { xid, data_box, profile }

#[derive(CandidType, Deserialize,Debug)]
pub struct BoxMetadata { pub is_private: bool, pub box_name: String, pub box_type: BoxType }

#[derive(CandidType, Deserialize,Debug)]
pub struct CreateBoxArgs {
    pub metadata: BoxMetadata,
    pub install_args: Vec<u8>,
    pub icp_amount: u64,
}

#[derive(CandidType, Deserialize,Debug)]
pub enum Error {
    Named,
    NoBox,
    OnlyDataBoxCanDeleted,
    NameRepeat,
    UnAuthorized,
    SomethingErr,
    LedgerTransferError(candid::Nat),
    Invalid_Operation,
    NotifyCreateError(candid::Nat),
}

#[derive(CandidType, Deserialize,Debug)]
pub enum CreateBoxResult { ok(candid::Principal), err(Error) }

#[derive(CandidType, Deserialize,Debug)]
pub struct DelBoxArgs {
    pub cycleTo: Option<candid::Principal>,
    pub box_type: BoxType,
    pub canisterId: candid::Principal,
}

#[derive(CandidType, Deserialize,Debug)]
pub enum DeleteBoxResult { ok(String), err(Error) }

#[derive(CandidType, Deserialize,Debug)]
pub enum BoxStatus { stopped, running }

#[derive(CandidType, Deserialize,Debug)]
pub struct BoxInfo {
    pub status: BoxStatus,
    pub canister_id: candid::Principal,
    pub is_private: bool,
    pub box_name: String,
    pub box_type: BoxType,
}

#[derive(CandidType, Deserialize,Debug)]
pub enum UpgradeBoxResult { ok, err(Error) }

#[derive(CandidType, Deserialize,Debug)]
pub enum SetNameResult { ok, err(Error) }

#[derive(CandidType, Deserialize,Debug)]
pub enum TopUpBoxResult { ok, err(Error) }

#[derive(CandidType, Deserialize,Debug)]
pub enum InstallCycleWasmResult { ok, err(Error) }

#[derive(CandidType, Deserialize,Debug)]
pub enum UpdateBoxInfoResult { ok, err(Error) }

#[derive(CandidType, Deserialize,Debug)]
pub struct TopUpArgs { pub box_id: candid::Principal, pub icp_amount: u64 }

pub type AccountIdentifier = Vec<u8>;
#[derive(CandidType, Deserialize,Debug)]
pub struct Token { e8s: u64 }

pub type BlockIndex = u64;
#[derive(CandidType, Deserialize,Debug)]
pub enum TransferError {
    TxTooOld{ allowed_window_nanos: u64 },
    BadFee{ expected_fee: Token },
    TxDuplicate{ duplicate_of: BlockIndex },
    TxCreatedInFuture,
    InsufficientFunds{ balance: Token },
}

#[derive(CandidType, Deserialize,Debug)]
pub enum TransferOutICPResult { ok(BlockIndex), err(TransferError) }

#[derive(CandidType, Deserialize,Debug)]
pub struct UpdateWasmArgs { pub wasm: Vec<u8>, pub box_type: BoxType }

#[derive(CandidType, Deserialize,Debug)]
pub enum UpdateWasmResult { ok(String), err(String) }

#[derive(CandidType, Deserialize,Debug)]
pub struct UpgradeBoxArgs { info: BoxInfo, install_args: Vec<u8> }

type MetaBox = candid::Service;
struct SERVICE(candid::Principal);
impl SERVICE{

    pub async fn add_admin(&self, new_admin: candid::Principal) -> CallResult<(bool,)> {
        ic_cdk::call(self.0, "addAdmin", (new_admin,)).await
    }

    pub async fn change_admin(&self, _admins: Vec<candid::Principal>) -> CallResult<
        (bool,)
    > { ic_cdk::call(self.0, "changeAdmin", (_admins,)).await }

    pub async fn clear_log(&self) -> CallResult<()> {
        ic_cdk::call(self.0, "clearLog", ()).await
    }

    pub async fn create_box(&self, args: CreateBoxArgs) -> CallResult<
        (CreateBoxResult,)
    > { ic_cdk::call(self.0, "createBox", (args,)).await }

    pub async fn create_xid(&self) -> CallResult<(candid::Principal,)> {
        ic_cdk::call(self.0, "createXid", ()).await
    }

    pub async fn delete_box(&self, args: DelBoxArgs) -> CallResult<(DeleteBoxResult,)> {
        ic_cdk::call(self.0, "deleteBox", (args,)).await
    }

    pub async fn get_admins(&self) -> CallResult<(Vec<candid::Principal>,)> {
        ic_cdk::call(self.0, "getAdmins", ()).await
    }

    pub async fn get_boxes(&self, who: candid::Principal) -> CallResult<
        (Vec<BoxInfo>,)
    > { ic_cdk::call(self.0, "getBoxes", (who,)).await }

    pub async fn get_log(&self) -> CallResult<(Vec<(candid::Nat,String,)>,)> {
        ic_cdk::call(self.0, "getLog", ()).await
    }

    pub async fn get_name_from_principal(
        &self,
        who: candid::Principal,
    ) -> CallResult<(Option<String>,)> {
        ic_cdk::call(self.0, "getNameFromPrincipal", (who,)).await
    }

    pub async fn get_principal_from_name(&self, name: String) -> CallResult<
        (Option<candid::Principal>,)
    > { ic_cdk::call(self.0, "getPrincipalFromName", (name,)).await }

    pub async fn get_profile(&self, who: candid::Principal) -> CallResult<
        (Option<candid::Principal>,)
    > { ic_cdk::call(self.0, "getProfile", (who,)).await }

    pub async fn get_profile_wasm(&self) -> CallResult<(String,)> {
        ic_cdk::call(self.0, "getProfileWasm", ()).await
    }

    pub async fn get_xid(&self) -> CallResult<(Option<candid::Principal>,)> {
        ic_cdk::call(self.0, "getXid", ()).await
    }

    pub async fn install_cycle_wasm(&self, wasm: Vec<u8>) -> CallResult<(InstallCycleWasmResult,)> {
        ic_cdk::call(self.0, "installCycleWasm", (wasm,)).await
    }

    pub async fn mint_box(
        &self,
        to: candid::Principal,
        box_type: BoxType,
        activity: String,
    ) -> CallResult<(String,)> {
        ic_cdk::call(self.0, "mintBox", (to,box_type,activity,)).await
    }

    pub async fn set_name(&self, name: String) -> CallResult<(SetNameResult,)> {
        ic_cdk::call(self.0, "setName", (name,)).await
    }

    pub async fn start_box(&self, info: BoxInfo) -> CallResult<()> {
        ic_cdk::call(self.0, "startBox", (info,)).await
    }

    pub async fn stop_box(&self, info: BoxInfo) -> CallResult<()> {
        ic_cdk::call(self.0, "stopBox", (info,)).await
    }

    pub async fn top_up_box(&self, args: TopUpArgs) -> CallResult<(TopUpBoxResult,)> {
        ic_cdk::call(self.0, "topUpBox", (args,)).await
    }

    pub async fn transfer_out_icp(
        &self,
        to: AccountIdentifier,
        amount: u64,
    ) -> CallResult<(TransferOutICPResult,)> {
        ic_cdk::call(self.0, "transferOutICP", (to,amount,)).await
    }

    pub async fn update_box_info(&self, info: BoxInfo) -> CallResult<(UpdateBoxInfoResult,)> {
        ic_cdk::call(self.0, "updateBoxInfo", (info,)).await
    }

    pub async fn update_wasm(&self, args: UpdateWasmArgs) -> CallResult<
        (UpdateWasmResult,)
    > { ic_cdk::call(self.0, "update_wasm", (args,)).await }

    pub async fn upgrade_box(&self, args: UpgradeBoxArgs) -> CallResult<
        (UpgradeBoxResult,)
    > { ic_cdk::call(self.0, "upgradeBox", (args,)).await }

    pub async fn wallet_receive(&self) -> CallResult<()> {
        ic_cdk::call(self.0, "wallet_receive", ()).await
    }
}
