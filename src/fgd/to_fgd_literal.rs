pub trait ToFgdLiteral {
    fn to_fgd_literal(&self) -> String;
}

impl ToFgdLiteral for String {
    fn to_fgd_literal(&self) -> String {
        format!("\"{}\"", self)
    }
}

impl ToFgdLiteral for &str {
    fn to_fgd_literal(&self) -> String {
        format!("\"{}\"", self)
    }
}

impl ToFgdLiteral for i32 {
    fn to_fgd_literal(&self) -> String {
        self.to_string()
    }
}

impl ToFgdLiteral for f32 {
    fn to_fgd_literal(&self) -> String {
        self.to_string()
    }
}

impl ToFgdLiteral for bool {
    fn to_fgd_literal(&self) -> String {
        if *self {
            1.to_string()
        } else {
            0.to_string()
        }
    }
}
