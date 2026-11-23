use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use windows_sys::Win32::Networking::WinHttp::*;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

struct Handle(*mut c_void);

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                WinHttpCloseHandle(self.0);
            }
        }
    }
}

pub fn get(url: &str, on_chunk: &mut dyn FnMut(&[u8], Option<u64>)) -> Result<(), String> {
    unsafe {
        let agent = wide("RadialMenu");
        let session = Handle(WinHttpOpen(agent.as_ptr(), WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, null(), null(), 0));
        if session.0.is_null() {
            return Err("接続を初期化できませんでした".into());
        }
        WinHttpSetTimeouts(session.0, 10_000, 10_000, 30_000, 60_000);

        let mut host = [0u16; 256];
        let mut path = vec![0u16; 4096];
        let mut extra = vec![0u16; 4096];
        let mut uc: URL_COMPONENTS = zeroed();
        uc.dwStructSize = size_of::<URL_COMPONENTS>() as u32;
        uc.lpszHostName = host.as_mut_ptr();
        uc.dwHostNameLength = host.len() as u32;
        uc.lpszUrlPath = path.as_mut_ptr();
        uc.dwUrlPathLength = path.len() as u32;
        uc.lpszExtraInfo = extra.as_mut_ptr();
        uc.dwExtraInfoLength = extra.len() as u32;
        let wurl = wide(url);
        if WinHttpCrackUrl(wurl.as_ptr(), 0, 0, &mut uc) == 0 {
            return Err(format!("URL を解析できません: {}", url));
        }
        let object: Vec<u16> = path[..uc.dwUrlPathLength as usize]
            .iter()
            .chain(extra[..uc.dwExtraInfoLength as usize].iter())
            .copied()
            .chain(std::iter::once(0))
            .collect();

        let connect = Handle(WinHttpConnect(session.0, host.as_ptr(), uc.nPort, 0));
        if connect.0.is_null() {
            return Err("サーバーに接続できませんでした".into());
        }
        let flags = if uc.nScheme == 2 { WINHTTP_FLAG_SECURE } else { 0 };
        let verb = wide("GET");
        let request = Handle(WinHttpOpenRequest(connect.0, verb.as_ptr(), object.as_ptr(), null(), null(), null(), flags));
        if request.0.is_null() {
            return Err("要求を作成できませんでした".into());
        }
        if WinHttpSendRequest(request.0, null(), 0, null(), 0, 0, 0) == 0 || WinHttpReceiveResponse(request.0, null_mut()) == 0 {
            return Err("サーバーから応答がありませんでした".into());
        }

        let mut status: u32 = 0;
        let mut len = size_of::<u32>() as u32;
        WinHttpQueryHeaders(
            request.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            null(),
            &mut status as *mut u32 as *mut c_void,
            &mut len,
            null_mut(),
        );
        if status != 200 {
            return Err(format!("サーバーが {} を返しました", status));
        }

        let mut total: u32 = 0;
        let mut tlen = size_of::<u32>() as u32;
        let has_total = WinHttpQueryHeaders(
            request.0,
            WINHTTP_QUERY_CONTENT_LENGTH | WINHTTP_QUERY_FLAG_NUMBER,
            null(),
            &mut total as *mut u32 as *mut c_void,
            &mut tlen,
            null_mut(),
        ) != 0;
        let total = if has_total { Some(total as u64) } else { None };

        let mut buf = vec![0u8; 65536];
        loop {
            let mut read: u32 = 0;
            if WinHttpReadData(request.0, buf.as_mut_ptr() as *mut c_void, buf.len() as u32, &mut read) == 0 {
                return Err("受信中にエラーが発生しました".into());
            }
            if read == 0 {
                break;
            }
            on_chunk(&buf[..read as usize], total);
        }
    }
    Ok(())
}

pub fn get_bytes(url: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    get(url, &mut |chunk, _| out.extend_from_slice(chunk))?;
    Ok(out)
}
