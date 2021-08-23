use crate::netease::crypto::AesMode::{CBC, ECB};
use openssl::hash::{hash, MessageDigest};
use openssl::rsa::{Padding, Rsa};
use openssl::symm::{encrypt, Cipher};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};

static KEY: &[u8] = "0CoJUm6Qyw8W8jud".as_bytes();
static IV: &[u8] = "0102030405060708".as_bytes();
static LINUX_KEY: &[u8] = "rFgB&h#%2?^eDg:Q".as_bytes();
static E_KEY: &[u8] = "e82ckenh8dichen8".as_bytes();
static RSA_KEY: &[u8] = "-----BEGIN PUBLIC KEY-----\nMIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDgtQn2JZ34ZC28NWYpAUd98iZ37BUrX/aKzmFbt7clFSs6sXqHauqKWqdtLkF2KexO40H1YTX8z2lSgBBOAxLsvaklV8k4cBFK9snQXE9/DDaFt6Rr7iVZMldczhC0JNgTz+SHXT6CBHuX3e9SdB1Ua44oncaTWz7OBGLbCiK45wIDAQAB\n-----END PUBLIC KEY-----".as_bytes();
static BASE62: &[u8] = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".as_bytes();

#[allow(non_camel_case_types)]
pub enum AesMode {
    CBC,
    ECB,
}

pub struct Crypto;

//noinspection RsExternalLinter
#[derive(Serialize, Deserialize, Debug)]
#[warn(non_snake_case)]
struct Query {
    params: String,
    encSecKey: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LinuxQuery {
    eparams: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct EQuery {
    params: String,
}

impl Crypto {
    pub fn we_api(data: &str) -> String {
        log::debug!("data: {}", data);
        let mut secret_key = [0u8; 16];
        OsRng.fill_bytes(&mut secret_key);
        let mut key: Vec<u8> = secret_key
            .iter()
            .map(|i| BASE62[(i % 62) as usize])
            .collect();
        log::debug!("key: {}", String::from_utf8(key.clone()).unwrap());

        let enc_data = Crypto::aes_encrypt(data, KEY, CBC, Some(IV), |t| base64::encode(t));
        let params = Crypto::aes_encrypt(&enc_data, &key, CBC, Some(IV), |t| base64::encode(t));

        key.reverse();
        let rev_key = std::str::from_utf8(&key).expect("failed to reverse key");
        let enc_sec_key = Crypto::rsa_encrypt(rev_key, RSA_KEY);
        let query = Query {
            params,
            encSecKey: enc_sec_key,
        };
        log::debug!("data: {:?}", &query);
        serde_qs::to_string(&query).expect("failed to serialize qs")
    }

    pub fn e_api(url: &str, text: &str) -> String {
        let message = format!("nobody{}use{}md5forencrypt", url, text);
        let digest = hex::encode(hash(MessageDigest::md5(), message.as_bytes()).unwrap());
        let data = format!("{}-36cd479b6b5-{}-36cd479b6b5-{}", url, text, digest);
        let params = Crypto::aes_encrypt(&data, E_KEY, ECB, Some(IV), |t| hex::encode_upper(t));
        log::debug!("params: {}", &params);
        let query = EQuery { params };
        serde_qs::to_string(&query).expect("failed to serialize qs")
    }

    pub fn linux_api(data: &str) -> String {
        let params =
            Crypto::aes_encrypt(data, LINUX_KEY, ECB, None, |t| hex::encode(t)).to_uppercase();
        log::debug!("data: {}", &data);
        let query = LinuxQuery { eparams: params };
        serde_qs::to_string(&query).expect("failed to serialize qs")
    }

    pub fn aes_encrypt(
        data: &str,
        key: &[u8],
        mode: AesMode,
        iv: Option<&[u8]>,
        encode: fn(&[u8]) -> String,
    ) -> String {
        let cipher = match mode {
            CBC => Cipher::aes_128_cbc(),
            ECB => Cipher::aes_128_ecb(),
        };
        let cipher_text = encrypt(cipher, key, iv, data.as_bytes()).expect("failed to aes encrypt");
        encode(&cipher_text)
    }

    pub fn rsa_encrypt(data: &str, key: &[u8]) -> String {
        let rsa = Rsa::public_key_from_pem(key).expect("failed to resolve public key");
        let prefix = vec![0u8; 128 - data.len()];
        let data = [&prefix[..], &data.as_bytes()[..]].concat();
        let mut buf = vec![0; rsa.size() as usize];
        rsa.public_encrypt(&data, &mut buf, Padding::NONE)
            .expect("failed to encrypt data");
        hex::encode(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::Crypto;
    use crate::netease::crypto::AesMode::CBC;
    use crate::netease::crypto::{IV, KEY, RSA_KEY};

    #[test]
    fn test_aes_encrypt() {
        let msg1 = r#"{"ids":"[347230]","br":999000}"#;
        let aes = Crypto::aes_encrypt(msg1, KEY, CBC, Some(IV), |t| base64::encode(t));
        assert_eq!(aes, "pgHP1O/hr+IboRMAq6HzpHjyYwNlv1x0G4BBjd1ohdM=");

        let msg2 = aes;
        let key1 = "gLiwKFot44HYFRAy".as_bytes();
        let aes = Crypto::aes_encrypt(&msg2, key1, CBC, Some(IV), |t| base64::encode(t));
        assert_eq!(
            aes,
            "3EC4ojigTl0OgjyYtcd+97P7YKarculWrOxSgNO5clkQftvO1jOvS8aAhK6diyOb"
        );

        let msg3 = r#"{"s":"海阔天空"}"#;
        let aes = Crypto::aes_encrypt(msg3, KEY, CBC, Some(IV), |t| base64::encode(t));
        assert_eq!(aes, "1CH1yTIZN/TXvOMJWH3yAe+iY8c9VfW36l3IfOm58l0=");

        let msg4 = aes;
        let key2 = "05EBdrdgLjgiqaRc".as_bytes();
        let aes = Crypto::aes_encrypt(&msg4, key2, CBC, Some(IV), |t| base64::encode(t));
        assert_eq!(
            aes,
            "uPCj4YGmXlMcix5LDAGFb0ynzwPFpFet8dZZ6ia8d2mS47OlnguVmNjGDWPJY1G3"
        );
    }

    #[test]
    fn test_rsa_encrypt() {
        let key = "yARFYH44toFKwiLg";
        let rsa = Crypto::rsa_encrypt(key, RSA_KEY);
        assert_eq!(rsa, "5ff8bdb3ed3dd15a26e9025e9abcff0d7a3764dafbc70e33859a892584c681f1aab314b8ad1f3418650ff851bdb0685fc5136a88e059c592da104bbeaba666fbe89eb405c7b66eab4db8ee3ab13a3f98cb41b2ac9981ed4e441ed8e1870524d001ee6ebc1c09f7a945677e5b56a3e964a224c3ee75ac43fbf513f6a8bf7472ee");
    }

    #[test]
    fn test_linux_api() {
        let msg = r#"{"method":"POST","url":"https://music.163.com/api/song/lyric?lv=-1&kv=-1&tv=-1","params":{"id":"347230"}}"#;
        log::debug!("msg: {}", msg);
        let res = Crypto::linux_api(msg);
        assert_eq!(res, "eparams=A0D9583F4C5FF68DE851D2893A49DE98FAFB24399F27B4F7E74C64B6FC49A965CFA972FA5EA3D6247CD6247C8198CB873B98A81F6838B428B103E7871611EAC556D5DBE4408FD2751C0E2AD139004A718B72FE3E65ECD467E96A996D93F627A05EB0AAB74EC2E68145C014D505562560");
    }
}
