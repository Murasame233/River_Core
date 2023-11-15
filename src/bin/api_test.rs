use algonaut::transaction::account::Account;
use data_encoding::BASE64;
use serde::{Deserialize, Serialize};

const MEMO:&'static str = "stereo room brief lion actual dress sister elephant cricket age mule elite month coil devote sun deputy vacant great reduce faint response hip abandon recycle";
const MEMO2:&'static str = "crystal tiger wide simple cricket sick clerk cupboard master jump reflect level page solve sound depth jazz party wool wage ill window battle above reason";

#[derive(Deserialize, Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Sign {
    pub address: String,
    pub msg: String,
    pub signature: String,
}

#[tokio::main]
async fn main() {
    let msg = String::from("123");
    let account = Account::from_mnemonic(MEMO).unwrap();
    let signature = BASE64.encode(&account.generate_sig(msg.as_bytes()).0);
    let sign = Sign {
        address: account.address().to_string(),
        msg,
        signature,
    };
    let client = reqwest::Client::new();
    let req = client
        .post("http://127.0.0.1:2001/create_stream")
        .json(&sign);
    let re = req.send().await.unwrap();
    let father = re.text().await.unwrap();
    println!("{:?}", father);

    let account_2 = Account::from_mnemonic(MEMO2).unwrap();
    let msg_2 = String::from(father);
    let signature_2 = BASE64.encode(&account_2.generate_sig(msg_2.as_bytes()).0);
    let sign_2 = Sign {
        address: account_2.address().to_string(),
        msg: msg_2,
        signature: signature_2,
    };
    let req_2 = client.post("http://127.0.0.1:2001/join_stream");
    let re_2 = req_2.json(&sign_2).send().await.unwrap();
    let sub = re_2.text().await.unwrap();
    println!("{:?}", sub);
}
