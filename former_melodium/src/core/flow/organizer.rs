
use crate::core::prelude::*;

macro_rules! impl_Organize {
    ($mod:ident, $mel_name:expr, $mel_type:ident, $type:ident, $recv_func:ident, $send_func:ident) => {
        treatment!($mod,
            core_identifier!("organize";$mel_name),
            formatdoc!(r#"Organize stream of `{type}` into stream of `Vec<{type}>`.

            ℹ️ If some remaining values doesn't fit into the pattern, they are trashed.
            If there are not enough values to fit the pattern, uncomplete vector is trashed.
            
            ```mermaid
            graph LR
                T(Organize)
                A["… 🟨 🟨 🟨 🟨 🟨 🟨"] -->|value| T
                B["[🟦 🟦] [🟦] [🟦 🟦 🟦]"] -->|pattern| T
                
                T -->|values| O["[🟨 🟨] [🟨] [🟨 🟨 🟨]"]
            
                style A fill:#ffff,stroke:#ffff
                style B fill:#ffff,stroke:#ffff
                style O fill:#ffff,stroke:#ffff
            ```"#, type = stringify!($type)),
            models![],
            treatment_sources![],
            parameters![],
            inputs![
                input!("value",Scalar,$mel_type,Stream),
                input!("pattern",Vector,Void,Stream)
            ],
            outputs![
                output!("values",Vector,$mel_type,Stream)
            ],
            host {
                let input = host.get_input("value");
                let pattern = host.get_input("pattern");
                let output = host.get_output("values");
            
                'main: while let Ok(patterns) = pattern.recv_vec_void().await {

                    for pattern in patterns {

                        let mut vector = Vec::with_capacity(pattern.len());
                        for _ in 0..pattern.len() {
                            if let Ok(val) = input.$recv_func().await {
                                vector.push(val);
                            }
                            else {
                                // Uncomplete, we 'trash' vector
                                break 'main;
                            }
                        }

                        ok_or_break!('main, output.$send_func(vector).await);
                    }
                }
            
                ResultStatus::Ok
            }
        );
    }
}

impl_Organize!(organize_u8, "OrganizeU8", U8, u8, recv_one_u8, send_vec_u8);
impl_Organize!(organize_u16, "OrganizeU16", U16, u16, recv_one_u16, send_vec_u16);
impl_Organize!(organize_u32, "OrganizeU32", U32, u32, recv_one_u32, send_vec_u32);
impl_Organize!(organize_u64, "OrganizeU64", U64, u64, recv_one_u64, send_vec_u64);
impl_Organize!(organize_u128, "OrganizeU128", U128, u128, recv_one_u128, send_vec_u128);
impl_Organize!(organize_i8, "OrganizeI8", I8, i8, recv_one_i8, send_vec_i8);
impl_Organize!(organize_i16, "OrganizeI16", I16, i16, recv_one_i16, send_vec_i16);
impl_Organize!(organize_i32, "OrganizeI32", I32, i32, recv_one_i32, send_vec_i32);
impl_Organize!(organize_i64, "OrganizeI64", I64, i64, recv_one_i64, send_vec_i64);
impl_Organize!(organize_i128, "OrganizeI128", I128, i128, recv_one_i128, send_vec_i128);
impl_Organize!(organize_f32, "OrganizeF32", F32, f32, recv_one_f32, send_vec_f32);
impl_Organize!(organize_f64, "OrganizeF64", F64, f64, recv_one_f64, send_vec_f64);
impl_Organize!(organize_bool, "OrganizeBool", Bool, bool, recv_one_bool, send_vec_bool);
impl_Organize!(organize_byte, "OrganizeByte", Byte, byte, recv_one_byte, send_vec_byte);
impl_Organize!(organize_char, "OrganizeChar", Char, char, recv_one_char, send_vec_char);
impl_Organize!(organize_string, "OrganizeString", String, string, recv_one_string, send_vec_string);

pub fn register(mut c: &mut CollectionPool) {

    organize_u8::register(&mut c);
    organize_u16::register(&mut c);
    organize_u32::register(&mut c);
    organize_u64::register(&mut c);
    organize_u128::register(&mut c);
    organize_i8::register(&mut c);
    organize_i16::register(&mut c);
    organize_i32::register(&mut c);
    organize_i64::register(&mut c);
    organize_i128::register(&mut c);
    organize_f32::register(&mut c);
    organize_f64::register(&mut c);
    organize_bool::register(&mut c);
    organize_byte::register(&mut c);
    organize_char::register(&mut c);
    organize_string::register(&mut c);
}
