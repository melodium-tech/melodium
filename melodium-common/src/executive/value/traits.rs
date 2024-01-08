use super::Value;

impl core::ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::I8(me), Value::I8(other)) => Value::I8(me + other),
            (Value::I16(me), Value::I16(other)) => Value::I16(me + other),
            (Value::I32(me), Value::I32(other)) => Value::I32(me + other),
            (Value::I64(me), Value::I64(other)) => Value::I64(me + other),
            (Value::I128(me), Value::I128(other)) => Value::I128(me + other),

            (Value::U8(me), Value::U8(other)) => Value::U8(me + other),
            (Value::U16(me), Value::U16(other)) => Value::U16(me + other),
            (Value::U32(me), Value::U32(other)) => Value::U32(me + other),
            (Value::U64(me), Value::U64(other)) => Value::U64(me + other),
            (Value::U128(me), Value::U128(other)) => Value::U128(me + other),

            (Value::F32(me), Value::F32(other)) => Value::F32(me + other),
            (Value::F64(me), Value::F64(other)) => Value::F64(me + other),
            _ => panic!("Illegal `add` call, this is an internal bug to report."),
        }
    }
}

impl core::fmt::Display for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Value::Void(_) => write!(f, "_"),
            Value::I8(v) => write!(f, "{}", v),
            Value::I16(v) => write!(f, "{}", v),
            Value::I32(v) => write!(f, "{}", v),
            Value::I64(v) => write!(f, "{}", v),
            Value::I128(v) => write!(f, "{}", v),
            Value::U8(v) => write!(f, "{}", v),
            Value::U16(v) => write!(f, "{}", v),
            Value::U32(v) => write!(f, "{}", v),
            Value::U64(v) => write!(f, "{}", v),
            Value::U128(v) => write!(f, "{}", v),
            Value::F32(v) => write!(f, "{}", v),
            Value::F64(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Byte(v) => write!(f, "0x{}", hex::encode([*v])),
            Value::Char(v) => write!(f, "'{}'", v),
            Value::String(v) => write!(f, "\"{}\"", v.replace('"', "\\\"")),
            Value::Option(v) => {
                if let Some(v) = v {
                    write!(f, "{v}")
                } else {
                    write!(f, "_")
                }
            }
            Value::Vec(v) => write!(
                f,
                "[{}]",
                v.iter()
                    .map(|val| val.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}
