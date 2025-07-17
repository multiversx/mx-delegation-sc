use multiversx_sc_snippets::imports::*;

use crate::{latest_proxy, LegacyDelegationInteractor};

/// New implementation.
// const LATEST_CODE_PATH: FilePath = FilePath("../latest/output/delegation_latest_update.wasm");

/// Backwards compatible implementation.
const LATEST_CODE_PATH: FilePath =
    FilePath("../v0_5_9_update/output/delegation_v0_5_9_update.wasm");

fn operation_completion_status(status: OperationCompletionStatus) -> &'static str {
    match status {
        OperationCompletionStatus::Completed => "completed",
        OperationCompletionStatus::InterruptedBeforeOutOfGas => "interrupted",
    }
}

impl LegacyDelegationInteractor {
    pub async fn version(&mut self) {
        let response = self
            .interactor
            .query()
            .to(&self.config.sc_address)
            .typed(latest_proxy::DelegationFullProxy)
            .version()
            .returns(ReturnsResultAs::<String>::new())
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn register_owner(&mut self) -> Address {
        let owner_wallet = Wallet::from_pem_file("legacyDelegationOwner.pem").unwrap();
        self.interactor.register_wallet(owner_wallet).await
    }

    pub async fn upgrade_contract_to_latest(&mut self) {
        let owner_address = self.register_owner().await;

        self.interactor
            .tx()
            .from(&owner_address)
            .to(&self.config.sc_address)
            .gas(300_000_000u64)
            .typed(latest_proxy::DelegationFullProxy)
            .upgrade()
            .code(LATEST_CODE_PATH)
            .code_metadata(CodeMetadata::UPGRADEABLE)
            .returns(ReturnsResultUnmanaged)
            .run()
            .await;

        println!("Upgrade completed");
    }

    pub async fn modify_delegation_cap(&mut self) {
        let owner_address = self.register_owner().await;
        let egld_amount =
            BigUint::<StaticApi>::from(4235000_000000000000000000u128 - 100_000000000000000000u128);

        let status = self
            .interactor
            .tx()
            .from(owner_address)
            .to(&self.config.sc_address)
            .gas(100_000_000u64)
            .typed(latest_proxy::DelegationFullProxy)
            .modify_total_delegation_cap(egld_amount)
            .returns(ReturnsResult)
            .run()
            .await;

        println!(
            "Modified delegation cap: {}",
            operation_completion_status(status)
        );
    }

    pub async fn fix_users(&mut self) {
        let user_address = self.interactor.register_wallet(test_wallets::alice()).await;

        let mut address_multi_vec = MultiValueVec::new();
        for &addres_hex in TESTNET_USER_ADDRESSES {
            address_multi_vec.push(Address::from_slice(&hex::decode(addres_hex).unwrap()));
        }

        let (num_updated, num_not_updated, num_not_found) = self
            .interactor
            .tx()
            .from(user_address)
            .to(&self.config.sc_address)
            .gas(100_000_000u64)
            .typed(latest_proxy::DelegationFullProxy)
            .update_user_address(address_multi_vec)
            .returns(ReturnsResult)
            .run()
            .await
            .into_tuple();

        println!("Success. Updated: {num_updated}. Not updated: {num_not_updated}. Not found: {num_not_found}.");
    }
}

pub const TESTNET_USER_ADDRESSES: &[&str] = &[
    "010771a4db695bc82356795c8f60b8290faeb99faafc0352ac3aa81c0a48bdff",
    "015f69627da26828922dd6a5ea07736acb43db9e56b30c49f64561551bafdad5",
    "054b27ece747c1339ab91bc0e4fb04cb172852d97a6d41d66f14809058480331",
    "0582b9c6744868881d4c9a9214aa62ac40581abfe370e3823c005d0cb3ae895a",
    "07ce363d39e2c3b06667264b1a36b47d208cd6dba26c47083eda84dbed923ff5",
    "0d8b8d14b09b452b86bba496ab2f71b0c40d791cd0426fc0219960c32d28acf7",
    "0e681a93c99b754ed3aada37778a7e19ec73b03d091d656844e9a45a53818edb",
    "0e73975caf036f302a5bc59c19a4fce494d55069c7560cbc65e7e02a592f1898",
    "0f673d970bcc140c91f76ecfa1a828b4ca452802429d1bb64349801b0c105c77",
    "1216741413496726be8643a2cc66aad2bf9eec0058d7e94a5693c869db4ccfe3",
    "17401a302a66fd2f6afb16d123a1ade0dc5f42ef145d3bde214bf60f96b53151",
    "1b1986c8fc0271b926ce39238113c819129fd98e1540a4f58423f124ab2865ee",
    "1d42982b9848dbe939b6b20406fed7f2840f15ef9b808f499cdd86474f628751",
    "1e63eabac32669a67cb2a867fc77183278e7e0e67c64972caada11b937b96eb9",
    "2112aa971d1c3eb2722a133a3df6556abd332f38223caa6930ce6e21ea7ce369",
    "2307dcc9fcff2c9741f169536e5a3dfce933d6b88c9f85de411c8ce20d2d1ed0",
    "23cac2f6cef12136e766079e9d3749efe08a0f1d58175f487621a8295b110530",
    "246737224b33088241d7c120721fe4c3427be530990bcc0f925b413af8976a28",
    "28056b53434fadfc7918ee2ef1032984e992d60e77f4d10b832d89b539d42842",
    "290a3baa20d06d194b10df75ceb9519cbf3836df52c8b822290f929b3b756050",
    "29946cfe40140f1ee17f9edf9ddd65f21110f1ba364e165b09becccb7e7cf442",
    "299bb57b068485583b754634406321d3d0804097bdded10aea8d7f2fe9cdcca8",
    "2f869a21efe75c33d3d7821ad1034ad40cc6ca251463b9fece9f725db0a024df",
    "2fc85531b82753c92faef1c63ed6c005b45a74dd7c2fa806553646e0ed1eeb89",
    "32aed5a6c15075ffc1585103acdce4ae0d8e11066cc5976af297b501bc05a63f",
    "3963a13d4ee8fdafc2f28ffafb5e8793d73f0d6be292cf8fa01663a0fef99dd9",
    "3af0277f19872e9e4bcfca9040726c2dcaefc06933feb4d06f619c790f42e17c",
    "3d27fd6fcd99ed18445044a1e64900855174423ce35e16c31f52c1541895329f",
    "402b05a7deb8e53a61de5bda725d38bc09e2af07849262731f549358bbd42685",
    "409c14b686ebd1be24c11bdc64d75248e8cb930cdf8bd90aa49ef531d0e565d9",
    "437db6e7e9845f854b3db243e761cf5520e9471d9b3a991f92e9db8f7eee6e3f",
    "443df70b822cd7dc04733fe5e675b8939b50f5081229c467ded2ce054ccff9da",
    "48733c35ae0aa16649854d69f8f2ceee3f29ff1e7a1420d371dbcea0f71fac3b",
    "4e28adaa923677dbc6bd1509be7b62ad48c801b84eb32f6a653c173f0d0d10d9",
    "510dadffbbdc76542bc49baba8c15812427e1120d00362bbddd5b65e34eec0e5",
    "59240ca16fc7b84ba2a54b8a1f3767887a008dde1a82283dcf08b26a31222ab8",
    "5a4ba45764c89ff8baa5bb80eff8f2d26b8b62b510f273f24f8c912efa2c2320",
    "5d7d5ca021e359d67f6325f487abf13d7d6ac2c9983c53d89fc65e3d98727278",
    "5f220399243ad44d29f30c5e12beffe803623593dcc0ea90f5a68d4553089860",
    "63320f2d9ce7bcf4bf97a68b5514609a76349f3dbf311a12a1963901aa8c2b20",
    "650942a019c91c4423bf9ee2af491e29eb8ad27eeffc7472fc58205d3a8f2cf7",
    "689264535ed1823fc7844907b89419a417d829a6751b763acf09d3e6851b8924",
    "68f914ca1117cc41dddf40cabfaa9cd02643195dbb32fe87ef0fc94999a03bda",
    "6b63483ff6539e9475ab962a625af5942a831636604d67bf1099cb11cb0503bf",
    "6bff401a31124bb885f80273d21b816e58a63ce15bef88327a1f5fe8a6d38b4d",
    "6eca47bcc8af6f33566909c7646c84037bad32f5a4057c6e42248cb267c5d7e0",
    "70c099af9545e984bac1afa084009087cd591ae90e21c13e93715029fadc0ca3",
    "70f294de265188d66301a3aa1338a03dd0f0be88e21c9b461cfc5d42a3f13aa4",
    "732ad807b79a6ddcbe11fea6a9dfccb2949b297ac2aed0462bb86350df3b4aaf",
    "75fcb354a50ed99c7fff4f3c514fda89cde6d519340f0f3bbd08d0a438f38343",
    "761c75e02c5ff39db210c5b7ccd90154a885e1fb1ffa1ab23ed55bcb92b5caa2",
    "798b4e51de177208f424a548fbae0f90c355cb697c47bbafec0cb1f1b82291e5",
    "7d7e9d7e6794e1291c9b6bbefb631ce699bcaae02586f274a53626138a8aa24c",
    "7f670ff30e8f8403204583f65bd3695b45adda624d93f409102f9b3ddf51d441",
    "896ab4e85a96e903cc990d1023daa6814117b2afee9c01542102865988ca3076",
    "8c6cfaf4656434831dc54c25e5d09e45746462321c6cb9c3cb2e8eb3bf26faa3",
    "9616fe412fc95a99808788051b9c67f8a9eea67d9b6cd214a62464523293770d",
    "9c049a90edb07e4ad21601baa294fc207148adb06ab44c7e849b560bffd252e1",
    "9e0c078312184e90c7e47904ada96760eb931c8b8467790708d144aec7bacc25",
    "a449cb38f6cdf466c1467b5bb1415684ca6304b1cdacdc63462c5661844a3e57",
    "a8b02b2d9d10b79d9848d840e592d93274d2da6dafadae489cf8a843359d5b76",
    "aaf130e88cfae788812d75efce40b77860b5236bce32051ee8417ccedbeba99b",
    "ae8539c205793a0d4e11495e637655579318bcfa44355c91600d3a0520c90257",
    "af9c9a4640dfff1150ab670db535de4a5ac8842403fc9412a51cc4385f8f3bea",
    "afc0e21859fdd12c8dcf587335515f59fb8acd0188424e7f4ce027f0800a4dc2",
    "b45133f81b9f1a09f5273126286763aca4c53d4135164f74f160c7bc96149231",
    "b67f463576ac32d916dc416a0d0556ade2a0f8a9007578879295eafac81b8c82",
    "b70e919d46786e5242ca907d3c80acda53f1f3b47084ff2948f703eab8c76cad",
    "bbbc23b68c8a2578a4c6bb742f034bd38992daff234cb9cb6257b8f3a4d794b6",
    "bd9abb9427d6d6e2af8abcbae0ebc9629e07fa8e72af7f7e8979048f420a9413",
    "c01ca20793489ab04bd58d9256d9477258ca0b3a6348e51b0715e93b6a8e5c96",
    "c047e42f29aba57fbc02ea58f6a2784f46960054e648db952a85037c201a7f35",
    "c149f014a309fe4e696d5226ccffe190a93b7204adb6711ab0dec03e3cd624ba",
    "c2d7f585555ca5909217f79466f217b00f6d74cad2cb51154ad88ee88af38c21",
    "c31e33b08fcde1588f79d2632b42bfde4d0f25beff0b11527c1b07e1a9382949",
    "c848f59b7c32e57f49e745a131cd5c535149862ffb03feb546ab34c32db656d7",
    "ce1f93de128522fe5eb6a68767b7ff37e41a8c533f4190b5c9aff286e323b888",
    "ce6c919859539b000571808515bced5ae187659df235f769a0dba93f466273ee",
    "ced1959a4f58c61dc24fbd37b50a3d45e4154db04facacbe03450f3d57797953",
    "d1018223c80bb060b2cd0018578430adfbc96e97c179624de0e70f7f76ff199e",
    "d15638361d2262a414b91c83602d2b65855da6136d779d6e7a844f9707898992",
    "d2fabe422d086a9ed0ff6dd07df01a71746577d736c7ebcaa3446f67120c7822",
    "d5f2423b319577b39b8706e75ee1ccf0a3a1c077cef1ae9f90203caf58c541bb",
    "d8d1711a436bf8cdc3aef76f16ee37b4eede727e1fae14e5ab04366a26a407de",
    "daede5e7e55e4b79594cee998cf559041352ea380dc279d86f207dfaee981c49",
    "db98e40cd38e0b78c85b5180cdb9a00a70e2bd200f648e5789db3e0b0d356df0",
    "ddb4106326de8467184b0003ac5ae32da9d2e2d81ef4f48b35f840588c41cd30",
    "de3bce0c2712696a9f187881bac607f071a28fd542817936a9635e8463ddc9e0",
    "e24b17959297f227bdfb140bc3f6e870d16beaf6f2d9cc3b367c759da37a973b",
    "e2cb8addd8659accf2c2c085329e5fd95d73091c998b055e46d77def13d1f90a",
    "e6925a12d3704e372e384310df3ee150479419268b06737b5a681b619341a148",
    "e86e682787b3b0185001e02a7bde8bf8e3427885a5e6d2d74d793e5ab3efa1f6",
    "e944fb19f40a6b56f77aa74f06cf15a559a3cc07b9dd2ee6a4e5870fa427ea65",
    "ea994e0e79aca5a50a80c54e2d4da54c54404850d3cbc37133fa6a4d3ba4f2dd",
    "f16cfd38340881832c2be1093363a4fa1855b6a8c2c5dd751149c7e7caeb279e",
    "f5217e59909874cfa2d5d76a475f740278f7b89ec366f9a1dd7ff6813a2159b0",
    "f58803d2bf0fd10c3dc9340d1c148c769716753e9e00cfe90600955d88b72423",
    "f5b5aa9c4474576bb3d8dcaac6b701dbae8fe2ab6f0b2809a4c50e59072ceb79",
    "f8c764f018152aea3dfde91c13137a3b1f8e2dde894fa904fc6ffcc2d6103ab3",
    "f984bbeaad6a791a425cdec351f140b10b2c782c6179c75a821d0d88deae813a",
    "fc44d34fe96d7509fd3746f301110603dbf69fbc0f2628f3df0f7fa3abab1188",
];
