use crate::ast::Data;

pub trait ImmediateCompat {
    fn can_be_immediate(&self) -> bool;
    fn represent(&self) -> String;
}

impl ImmediateCompat for f32 {
    fn can_be_immediate(&self) -> bool {
        if *self == 0.0 {
            return true;
        }

        for r in -3..=4 {
            let factor = 2_f32.powi(r);
            let n = (*self / factor * 16.0).round();
            if n >= 16.0 && n <= 31.0 && (*self - (n / 16.0 * factor)).abs() < 1e-7 {
                return true;
            }
        }

        false
    }

    fn represent(&self) -> String {
        match self.can_be_immediate() {
            true => format!("{self:e}"),
            false => format!("={}", self.to_bits()),
        }
    }
}

impl ImmediateCompat for i64 {
    fn can_be_immediate(&self) -> bool {
        self.abs() <= 2_i64.pow(16)
    }

    fn represent(&self) -> String {
        match self.can_be_immediate() {
            true => format!("{self}"),
            false => format!("={}", self),
        }
    }
}

impl ImmediateCompat for Data {
    fn can_be_immediate(&self) -> bool {
        match self {
            Data::Float(x) => x.can_be_immediate(),
            x => i64::from(x).can_be_immediate(),
        }
    }

    fn represent(&self) -> String {
        match self {
            Data::Float(x) => x.represent(),
            x => i64::from(x).represent(),
        }
    }
}
