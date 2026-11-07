//! Uploads of data while it is produced, to prepared requests, such as presigned by object
//! storage: in parts of growing sizes (multipart uploads), or in one request of known size.
//!
//! Parts are cut ahead of their upload, so that programs can have a request prepared for each
//! of them, connections in a program never looping back.

use crate::client::*;
use melodium_core::{common::executive::Output, *};
use melodium_macro::{mel_function, mel_treatment};
use std::sync::Arc;

/// Return the size of the part `part_number` of an upload, from 1.
///
/// Parts are 8 MiB for parts 1 to 1,000, then double every 1,000 parts, up to 4 GiB for parts
/// 9,001 to 10,000: the 10,000 parts object storage takes hold about 8 TiB.
/// This is the size `cut` gives its parts, to tell when preparing requests for them.
#[mel_function]
pub fn part_size(part_number: u32) -> u64 {
    crate::transfer::part_size(part_number)
}

/// Cut data into parts to upload.
///
/// The bytes received through `data` are given through `parts` in parts of the sizes
/// `|part_size` gives, the last one being shorter. There is always at least one part, empty
/// when there is no data.
///
/// Once `data` ends and every part is given, `completed` is emitted. If the data needs more
/// than 10,000 parts, `failed` is emitted and `error` contains the reason.
/// `finished` is emitted in every case.
///
/// Memory holds the part being cut, and those given but not taken yet.
///
/// ```mermaid
/// graph LR
///     T("cut()")
///     B["… 🟦 🟦 🟦 🟦 🟦"] -->|data| T
///
///     T -->|parts| P["… [🟦 🟦] [🟦 🟦 🟦]"]
///
///     style B fill:#ffffff,stroke:#ffffff
///     style P fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input data Stream<byte>
    output parts Stream<Vec<byte>>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output error Block<string>
)]
pub async fn cut() {
    let mut cutter = crate::transfer::Cutter::new(crate::transfer::part_size);
    let result: Result<bool, String> = async {
        while let Ok(bytes) = data.recv_many_as::<u8>().await {
            for part in cutter.push(&bytes)? {
                if !give(&**parts, part).await {
                    return Ok(false);
                }
            }
        }
        Ok(match cutter.finish() {
            Some(part) => give(&**parts, part).await,
            None => true,
        })
    }
    .await;

    data.close();
    parts.close().await;
    match result {
        Ok(true) => {
            let _ = completed.send_one(().into()).await;
        }
        Ok(false) => {}
        Err(reason) => {
            let _ = failed.send_one(().into()).await;
            let _ = error.send_one(reason.into()).await;
        }
    }
    let _ = finished.send_one(().into()).await;
}

/// Gives `part` through `parts`, telling whether it was taken.
async fn give(parts: &dyn Output, part: Vec<u8>) -> bool {
    parts
        .send_one(Value::Packed(PackedArray::Byte(Arc::new(part))))
        .await
        .is_ok()
}

/// Upload parts, each to its prepared request.
///
/// Each part received through `parts`, such as `cut` gives, is sent with the request received
/// through `requests` in the same position, such as one presigned for that part number, with
/// its length. The value of the header named `tag_header` in each answer, such as `etag` for
/// object storage, is given through `tags`, in the order of the parts, to complete the upload.
///
/// A part refused with `5xx`, `429` or `408`, or whose connection fails, is sent again up to
/// `retries` times, waiting longer each time: it is kept until it is uploaded. Memory holds that
/// part, and those given but not taken yet.
///
/// Once `parts` ends and every part is uploaded, `completed` is emitted. If a part is refused
/// otherwise, retries are exhausted, an answer has no `tag_header`, or a part has no request,
/// `failed` is emitted and `error` contains the reason, with the status and the `<Code>` of
/// XML error bodies, never the URL. `finished` is emitted in every case.
///
/// Requests are made as given, without the headers of any `HttpClient`.
///
/// ```mermaid
/// graph LR
///     T("parts()")
///     P["… [🟦 🟦] [🟦 🟦 🟦]"] -->|parts| T
///     R["… 🟨 🟨"] -->|requests| T
///     H["〈etag〉"] -->|tag_header| T
///
///     T -->|tags| O["… 🟪 🟪"]
///     T -->|completed| C["〈🟩〉"]
///
///     style P fill:#ffffff,stroke:#ffffff
///     style R fill:#ffffff,stroke:#ffffff
///     style H fill:#ffffff,stroke:#ffffff
///     style O fill:#ffffff,stroke:#ffffff
///     style C fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    default retries 5
    input parts Stream<Vec<byte>>
    input requests Stream<PreparedRequest>
    input tag_header Block<string>
    output tags Stream<string>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output error Block<string>
)]
pub async fn parts(retries: u32) {
    let result: Result<bool, String> = async {
        let tag_header = tag_header
            .recv_one_as::<String>()
            .await
            .map_err(|_| "no tag header was given".to_string())?;
        let mut number = 0;
        while let Ok(part) = parts.recv_one().await {
            number += 1;
            let part = match part {
                Value::Packed(PackedArray::Byte(bytes)) => bytes,
                Value::Vec(values) => Arc::new(
                    values
                        .into_iter()
                        .map(|value| match value {
                            Value::Byte(byte) => Some(byte),
                            _ => None,
                        })
                        .collect::<Option<Vec<u8>>>()
                        .ok_or_else(|| format!("part {number} is not bytes"))?,
                ),
                _ => return Err(format!("part {number} is not bytes")),
            };
            let request = requests
                .recv_one_as::<Arc<PreparedRequest>>()
                .await
                .map_err(|_| format!("part {number} has no prepared request"))?;

            #[cfg(feature = "real")]
            let tag = crate::transfer::upload_part(
                crate::transfer::client(),
                &request,
                &part,
                &tag_header,
                retries,
            )
            .await
            .map_err(|reason| format!("part {number}: {reason}"))?;
            #[cfg(feature = "mock")]
            let tag: String = {
                let _ = (&request, &part, &tag_header, retries);
                return Err("uploads are not available in mock mode".to_string());
            };

            if tags.send_one(tag.into()).await.is_err() {
                return Ok(false);
            }
        }
        Ok(true)
    }
    .await;

    parts.close();
    requests.close();
    tags.close().await;
    match result {
        Ok(true) => {
            let _ = completed.send_one(().into()).await;
        }
        Ok(false) => {}
        Err(reason) => {
            let _ = failed.send_one(().into()).await;
            let _ = error.send_one(reason.into()).await;
        }
    }
    let _ = finished.send_one(().into()).await;
}

/// Upload data of known size in one prepared request.
///
/// The bytes received through `data` are sent with `request`, such as one presigned for a
/// small file, while they come, with `size` as length: nothing is kept, so a failed upload is
/// not sent again.
///
/// Once the data is sent and accepted, `completed` is emitted. If the data is longer or
/// shorter than `size`, the request is refused, or its connection fails, `failed` is emitted
/// and `error` contains the reason, with the status and the `<Code>` of XML error bodies, never
/// the URL. `finished` is emitted in every case.
///
/// The request is made as given, without the headers of any `HttpClient`.
///
/// ```mermaid
/// graph LR
///     T("put()")
///     R["〈🟨〉"] -->|request| T
///     S["〈11〉"] -->|size| T
///     B["… 🟦 🟦 🟦"] -->|data| T
///
///     T -->|completed| C["〈🟩〉"]
///
///     style R fill:#ffffff,stroke:#ffffff
///     style S fill:#ffffff,stroke:#ffffff
///     style B fill:#ffffff,stroke:#ffffff
///     style C fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input request Block<PreparedRequest>
    input size Block<u64>
    input data Stream<byte>
    output completed Block<void>
    output failed Block<void>
    output finished Block<void>
    output error Block<string>
)]
pub async fn put() {
    let result: Result<(), String> = async {
        let request = request
            .recv_one_as::<Arc<PreparedRequest>>()
            .await
            .map_err(|_| "no prepared request was given".to_string())?;
        let size = size
            .recv_one_as::<u64>()
            .await
            .map_err(|_| "no size was given".to_string())?;

        #[cfg(feature = "real")]
        return crate::transfer::put(crate::transfer::client(), &request, size, || async {
            data.recv_many_as::<u8>().await.ok()
        })
        .await;
        #[cfg(feature = "mock")]
        {
            let _ = (&request, size);
            Err("uploads are not available in mock mode".to_string())
        }
    }
    .await;

    data.close();
    match result {
        Ok(()) => {
            let _ = completed.send_one(().into()).await;
        }
        Err(reason) => {
            let _ = failed.send_one(().into()).await;
            let _ = error.send_one(reason.into()).await;
        }
    }
    let _ = finished.send_one(().into()).await;
}
