use crate::{
    error::{ServiceResult, WorkLoreError},
};

#[cfg(target_os = "windows")]
mod platform {
    use std::{ffi::c_void, ptr, slice};

    use super::*;

    const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x1;

    #[repr(C)]
    struct DataBlob {
        cb_data: u32,
        pb_data: *mut u8,
    }

    #[link(name = "Crypt32")]
    unsafe extern "system" {
        #[link_name = "CryptProtectData"]
        fn crypt_protect_data(
            data_in: *const DataBlob,
            description: *const u16,
            optional_entropy: *const DataBlob,
            reserved: *mut c_void,
            prompt_struct: *mut c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;

        #[link_name = "CryptUnprotectData"]
        fn crypt_unprotect_data(
            data_in: *const DataBlob,
            description: *mut *mut u16,
            optional_entropy: *const DataBlob,
            reserved: *mut c_void,
            prompt_struct: *mut c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;
    }

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        #[link_name = "LocalFree"]
        fn local_free(memory: *mut c_void) -> *mut c_void;
    }

    pub fn protect_secret(secret: &str) -> ServiceResult<String> {
        if secret.is_empty() {
            return Err(secret_error("An empty API key cannot be stored."));
        }
        let mut input = secret.as_bytes().to_vec();
        let input_length = u32::try_from(input.len())
            .map_err(|_| secret_error("The API key is too large to store securely."))?;
        let input_blob = DataBlob {
            cb_data: input_length,
            pb_data: input.as_mut_ptr(),
        };
        let mut output_blob = DataBlob {
            cb_data: 0,
            pb_data: ptr::null_mut(),
        };
        let succeeded = unsafe {
            crypt_protect_data(
                &input_blob,
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output_blob,
            )
        };
        if succeeded == 0 {
            return Err(secret_error(
                "Windows could not protect the API key for the current user.",
            ));
        }
        Ok(hex::encode(copy_and_free(output_blob)?))
    }

    pub fn unprotect_secret(ciphertext: &str) -> ServiceResult<String> {
        let mut encrypted = hex::decode(ciphertext).map_err(|_| {
            secret_error("The saved API key is unreadable. Clear it and enter the key again.")
        })?;
        if encrypted.is_empty() {
            return Err(secret_error(
                "The saved API key is empty. Clear it and enter the key again.",
            ));
        }
        let encrypted_length = u32::try_from(encrypted.len())
            .map_err(|_| secret_error("The saved API key is too large to read securely."))?;
        let input_blob = DataBlob {
            cb_data: encrypted_length,
            pb_data: encrypted.as_mut_ptr(),
        };
        let mut output_blob = DataBlob {
            cb_data: 0,
            pb_data: ptr::null_mut(),
        };
        let succeeded = unsafe {
            crypt_unprotect_data(
                &input_blob,
                ptr::null_mut(),
                ptr::null(),
                ptr::null_mut(),
                ptr::null_mut(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output_blob,
            )
        };
        if succeeded == 0 {
            return Err(secret_error(
                "Windows could not unlock the saved API key for the current user. Clear it and enter the key again.",
            ));
        }
        let bytes = copy_and_free(output_blob)?;
        String::from_utf8(bytes).map_err(|_| {
            secret_error("The saved API key is unreadable. Clear it and enter the key again.")
        })
    }

    fn copy_and_free(blob: DataBlob) -> ServiceResult<Vec<u8>> {
        if blob.pb_data.is_null() || blob.cb_data == 0 {
            return Err(secret_error(
                "Windows returned an empty protected-secret payload.",
            ));
        }
        let bytes = unsafe { slice::from_raw_parts(blob.pb_data, blob.cb_data as usize).to_vec() };
        let _ = unsafe { local_free(blob.pb_data.cast::<c_void>()) };
        Ok(bytes)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn windows_user_secret_round_trips_without_plaintext_storage() {
            let secret = "alpha-test-key-not-a-real-credential";
            let protected = protect_secret(secret).unwrap();
            assert_ne!(protected, secret);
            assert!(!protected.contains(secret));
            assert_eq!(unprotect_secret(&protected).unwrap(), secret);
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::*;

    pub fn protect_secret(_secret: &str) -> ServiceResult<String> {
        Err(secret_error(
            "BYOK secret storage is available only in the Windows alpha build.",
        ))
    }

    pub fn unprotect_secret(_ciphertext: &str) -> ServiceResult<String> {
        Err(secret_error(
            "BYOK secret storage is available only in the Windows alpha build.",
        ))
    }
}

pub use platform::{protect_secret, unprotect_secret};

fn secret_error(message: impl Into<String>) -> WorkLoreError {
    WorkLoreError::ProviderOperation {
        code: "secret_storage_failed",
        message: message.into(),
    }
}
